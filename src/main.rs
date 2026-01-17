#![recursion_limit = "256"]

mod action;
mod app;
mod command;
mod config;
mod domain;
mod ui;
mod update;

use std::collections::VecDeque;
use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use action::Action;
use app::{App, AppState};
use command::{execute_command, Command};
use config::AppConfig;
use update::update;

#[derive(Parser)]
#[command(name = "dstl")]
#[command(about = "Distillery - Distill PR diffs into reviewable narratives")]
struct Cli {
    /// PR reference: owner/repo#123 or GitHub URL (optional - starts repo selector if omitted)
    pr_ref: Option<String>,

    /// Repo for PR picker (owner/repo format)
    #[arg(short = 'R', long)]
    repo: Option<String>,

    /// OpenAI model to use
    #[arg(short, long, default_value = "gpt-5.2")]
    model: String,

    /// Use cached response (skip LLM call)
    #[arg(long)]
    cache: bool,

    /// Path to cache file
    #[arg(long, default_value = ".dstl-cache.json")]
    cache_file: String,
}

/// Startup mode determined from CLI args
enum StartupMode {
    /// Start with repo selector (no args provided)
    RepoSelector,
    /// Start with PR picker for a specific repo
    PrPicker { owner: String, repo: String },
    /// Load a specific PR directly
    DirectPr { owner: String, repo: String, number: u32 },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    // Determine startup mode
    let mode = if let Some(pr_ref) = &cli.pr_ref {
        // Have a PR reference - could be owner/repo#num or just owner/repo
        if pr_ref.contains('#') || pr_ref.contains("github.com") {
            // Full PR reference
            let (owner, repo, number) = domain::github::parse_pr_reference(pr_ref)
                .context("Invalid PR reference")?;
            StartupMode::DirectPr { owner, repo, number }
        } else {
            // Just owner/repo - start picker
            let (owner, repo) = pr_ref
                .split_once('/')
                .context("Invalid repo format. Use owner/repo")?;
            StartupMode::PrPicker {
                owner: owner.to_string(),
                repo: repo.to_string(),
            }
        }
    } else if let Some(repo_spec) = &cli.repo {
        // Have --repo flag - start picker
        let (owner, repo) = repo_spec
            .split_once('/')
            .context("Invalid repo format. Use owner/repo")?;
        StartupMode::PrPicker {
            owner: owner.to_string(),
            repo: repo.to_string(),
        }
    } else {
        // No args - start with repo selector
        StartupMode::RepoSelector
    };

    // Get API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable not set")?;

    let config = AppConfig {
        api_key,
        model: cli.model,
        use_cache: cli.cache,
        cache_file: cli.cache_file,
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let initial_commands = bootstrap(&mut app, &mode, &config);

    let result = run_event_loop(&mut terminal, &mut app, &config, initial_commands).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn bootstrap(app: &mut App, mode: &StartupMode, config: &AppConfig) -> Vec<Command> {
    match mode {
        StartupMode::RepoSelector => {
            app.state = AppState::LoadingRepoList;
            vec![Command::FetchRepoList]
        }
        StartupMode::PrPicker { owner, repo } => {
            app.state = AppState::LoadingPrList;
            app.current_repo = Some((owner.clone(), repo.clone()));
            vec![Command::FetchPrList {
                owner: owner.clone(),
                repo: repo.clone(),
            }]
        }
        StartupMode::DirectPr {
            owner,
            repo,
            number,
        } => {
            app.state = AppState::LoadingPr;
            app.current_repo = Some((owner.clone(), repo.clone()));
            app.current_pr_number = Some(*number);
            if config.use_cache {
                vec![Command::LoadCache {
                    path: config.cache_file.clone(),
                }]
            } else {
                vec![Command::FetchPr {
                    owner: owner.clone(),
                    repo: repo.clone(),
                    number: *number,
                }]
            }
        }
    }
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    config: &AppConfig,
    initial_commands: Vec<Command>,
) -> Result<()> {
    let mut actions: VecDeque<Action> = VecDeque::new();

    run_commands(terminal, app, config, initial_commands, &mut actions).await?;
    process_actions(terminal, app, config, &mut actions).await?;

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            actions.push_back(Action::Input {
                code: key.code,
                modifiers: key.modifiers,
            });
            process_actions(terminal, app, config, &mut actions).await?;
        }
    }

    Ok(())
}

async fn process_actions(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    config: &AppConfig,
    actions: &mut VecDeque<Action>,
) -> Result<()> {
    while let Some(action) = actions.pop_front() {
        let commands = update(app, action, config);
        if app.should_quit {
            break;
        }
        run_commands(terminal, app, config, commands, actions).await?;
    }

    Ok(())
}

async fn run_commands(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &App,
    config: &AppConfig,
    commands: Vec<Command>,
    actions: &mut VecDeque<Action>,
) -> Result<()> {
    for command in commands {
        terminal.draw(|f| ui::render(f, app))?;
        if let Some(action) = execute_command(command, config).await {
            actions.push_back(action);
        }
    }

    Ok(())
}
