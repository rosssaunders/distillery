#![recursion_limit = "256"]

mod app;
mod github;
mod llm;
mod prompt;
mod types;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppState};
use types::ReviewAction;

/// Result from PR picker input handling
enum PickerResult {
    /// User selected a PR
    Selected(u32),
    /// User wants to go back to repo selector
    BackToRepoSelector,
    /// No action taken
    None,
}

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
            let (owner, repo, number) = github::parse_pr_reference(pr_ref)
                .context("Invalid PR reference")?;
            StartupMode::DirectPr { owner, repo, number }
        } else {
            // Just owner/repo - start picker
            let (owner, repo) = pr_ref.split_once('/')
                .context("Invalid repo format. Use owner/repo")?;
            StartupMode::PrPicker { owner: owner.to_string(), repo: repo.to_string() }
        }
    } else if let Some(repo_spec) = &cli.repo {
        // Have --repo flag - start picker
        let (owner, repo) = repo_spec.split_once('/')
            .context("Invalid repo format. Use owner/repo")?;
        StartupMode::PrPicker { owner: owner.to_string(), repo: repo.to_string() }
    } else {
        // No args - start with repo selector
        StartupMode::RepoSelector
    };

    // Get API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable not set")?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app based on startup mode
    let mut app = match &mode {
        StartupMode::RepoSelector => App::new_with_repo_selector(),
        StartupMode::PrPicker { owner, repo } => App::new_with_picker(owner, repo),
        StartupMode::DirectPr { owner, repo, number: _ } => {
            let mut app = App::new();
            app.current_repo = Some((owner.clone(), repo.clone()));
            app
        }
    };

    // Get initial owner/repo/number (may change during execution)
    let (initial_owner, initial_repo, initial_number) = match mode {
        StartupMode::RepoSelector => (String::new(), String::new(), 0u32),
        StartupMode::PrPicker { owner, repo } => (owner, repo, 0u32),
        StartupMode::DirectPr { owner, repo, number } => (owner, repo, number),
    };

    // Run the app
    let result = run_app(
        &mut terminal,
        &mut app,
        initial_owner,
        initial_repo,
        initial_number,
        &api_key,
        &cli.model,
        cli.cache,
        &cli.cache_file,
    ).await;

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

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut owner: String,
    mut repo: String,
    mut number: u32,
    api_key: &str,
    model: &str,
    use_cache: bool,
    cache_file: &str,
) -> Result<()> {
    // Initial render
    terminal.draw(|f| ui::render(f, app))?;

    // If starting with repo selector, load repo list first
    if matches!(app.state, AppState::LoadingRepoList) {
        match github::fetch_repo_list() {
            Ok(repo_list) => {
                app.repo_list = repo_list;
                app.state = AppState::RepoSelector;
            }
            Err(e) => {
                app.state = AppState::Error(format!("Failed to fetch repo list: {}", e));
            }
        }
        terminal.draw(|f| ui::render(f, app))?;

        // Run repo selector loop until a repo is selected
        loop {
            if app.should_quit {
                return Ok(());
            }

            terminal.draw(|f| ui::render(f, app))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if let Some((selected_owner, selected_repo)) = handle_repo_selector_input(app, key.code) {
                        owner = selected_owner;
                        repo = selected_repo;
                        app.state = AppState::LoadingPrList;
                        break;
                    }
                }
            }
        }
    }

    // If starting with or transitioning to PR picker, load PR list
    if matches!(app.state, AppState::LoadingPrList) {
        // Use current_repo if set, otherwise use passed owner/repo
        let (o, r) = app.current_repo.clone().unwrap_or((owner.clone(), repo.clone()));
        owner = o;
        repo = r;

        match github::fetch_pr_list(&owner, &repo) {
            Ok(pr_list) => {
                app.pr_list = pr_list;
                app.state = AppState::PrPicker;
            }
            Err(e) => {
                app.state = AppState::Error(format!("Failed to fetch PR list: {}", e));
            }
        }
        terminal.draw(|f| ui::render(f, app))?;

        // Run picker loop until a PR is selected
        loop {
            if app.should_quit {
                return Ok(());
            }

            terminal.draw(|f| ui::render(f, app))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match handle_picker_input(app, key.code, &owner, &repo) {
                        PickerResult::Selected(selected_number) => {
                            number = selected_number;
                            app.state = AppState::LoadingPr;
                            app.show_picker = false;
                            break;
                        }
                        PickerResult::BackToRepoSelector => {
                            // Go back to repo selector
                            app.state = AppState::RepoSelector;
                            terminal.draw(|f| ui::render(f, app))?;

                            // Re-enter repo selector loop
                            loop {
                                if app.should_quit {
                                    return Ok(());
                                }

                                terminal.draw(|f| ui::render(f, app))?;

                                if event::poll(Duration::from_millis(100))? {
                                    if let Event::Key(key) = event::read()? {
                                        if let Some((selected_owner, selected_repo)) = handle_repo_selector_input(app, key.code) {
                                            owner = selected_owner;
                                            repo = selected_repo;
                                            app.current_repo = Some((owner.clone(), repo.clone()));
                                            app.state = AppState::LoadingPrList;

                                            // Load PR list for new repo
                                            match github::fetch_pr_list(&owner, &repo) {
                                                Ok(pr_list) => {
                                                    app.pr_list = pr_list;
                                                    app.picker_selected = 0;
                                                    app.state = AppState::PrPicker;
                                                }
                                                Err(e) => {
                                                    app.state = AppState::Error(format!("Failed to fetch PR list: {}", e));
                                                }
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        PickerResult::None => {}
                    }
                }
            }
        }
    }

    // Try to load from cache first (only for direct PR mode)
    if use_cache && !app.show_picker && number > 0 {
        if let Ok(cached) = std::fs::read_to_string(cache_file) {
            if let Ok(story) = serde_json::from_str::<types::Story>(&cached) {
                app.populate_from_story(&story);
                app.story = Some(story);
                app.state = AppState::Viewing;
                app.status = Some("Loaded from cache".to_string());

                // Still need PR context for actions
                app.pr = Some(types::PrContext {
                    owner: owner.clone(),
                    repo: repo.clone(),
                    number,
                    title: "Cached PR".to_string(),
                    body: String::new(),
                    diff: String::new(),
                    author: String::new(),
                    base_branch: String::new(),
                    head_branch: String::new(),
                });

                return run_event_loop(terminal, app, &owner, &repo, number, api_key, model, cache_file).await;
            }
        }
    }

    // Fetch PR (only if we have a number)
    if number > 0 {
        app.state = AppState::LoadingPr;
        terminal.draw(|f| ui::render(f, app))?;

        match github::fetch_pr(&owner, &repo, number).await {
            Ok(pr) => {
                app.pr = Some(pr);
                app.state = AppState::GeneratingStory;
            }
            Err(e) => {
                app.state = AppState::Error(e.to_string());
            }
        }

        terminal.draw(|f| ui::render(f, app))?;

        // Generate story if PR loaded
        if matches!(app.state, AppState::GeneratingStory) {
            if let Some(pr) = &app.pr {
                match llm::generate_story(pr, api_key, model).await {
                    Ok(story) => {
                        // Save to cache
                        if let Ok(json) = serde_json::to_string_pretty(&story) {
                            let _ = std::fs::write(cache_file, json);
                        }

                        app.populate_from_story(&story);
                        app.story = Some(story);
                        app.state = AppState::Viewing;
                    }
                    Err(e) => {
                        app.state = AppState::Error(e.to_string());
                    }
                }
            }
        }
    }

    run_event_loop(terminal, app, &owner, &repo, number, api_key, model, cache_file).await
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    owner: &str,
    repo: &str,
    mut number: u32,
    api_key: &str,
    model: &str,
    cache_file: &str,
) -> Result<()> {
    // Track current owner/repo (may change if user navigates to different repos)
    let mut current_owner = owner.to_string();
    let mut current_repo = repo.to_string();

    // Main event loop
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if app.should_quit {
            break;
        }

        // Poll for events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match &app.state {
                    AppState::RepoSelector => {
                        if let Some((selected_owner, selected_repo)) = handle_repo_selector_input(app, key.code) {
                            current_owner = selected_owner;
                            current_repo = selected_repo.clone();
                            app.current_repo = Some((current_owner.clone(), current_repo.clone()));

                            // Load PR list for selected repo
                            app.state = AppState::LoadingPrList;
                            terminal.draw(|f| ui::render(f, app))?;

                            match github::fetch_pr_list(&current_owner, &current_repo) {
                                Ok(pr_list) => {
                                    app.pr_list = pr_list;
                                    app.picker_selected = 0;
                                    app.state = AppState::PrPicker;
                                    app.show_picker = true;
                                }
                                Err(e) => {
                                    app.state = AppState::Error(format!("Failed to fetch PR list: {}", e));
                                }
                            }
                        }
                    }
                    AppState::PrPicker => {
                        match handle_picker_input(app, key.code, &current_owner, &current_repo) {
                            PickerResult::Selected(selected_number) => {
                                // Load the selected PR
                                number = selected_number;
                                app.reset_for_new_pr();
                                app.state = AppState::LoadingPr;
                                terminal.draw(|f| ui::render(f, app))?;

                                // Fetch and generate story for new PR
                                match github::fetch_pr(&current_owner, &current_repo, number).await {
                                    Ok(pr) => {
                                        app.pr = Some(pr);
                                        app.state = AppState::GeneratingStory;
                                        terminal.draw(|f| ui::render(f, app))?;

                                        if let Some(pr) = &app.pr {
                                            match llm::generate_story(pr, api_key, model).await {
                                                Ok(story) => {
                                                    if let Ok(json) = serde_json::to_string_pretty(&story) {
                                                        let _ = std::fs::write(cache_file, json);
                                                    }
                                                    app.populate_from_story(&story);
                                                    app.story = Some(story);
                                                    app.state = AppState::Viewing;
                                                }
                                                Err(e) => {
                                                    app.state = AppState::Error(e.to_string());
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        app.state = AppState::Error(e.to_string());
                                    }
                                }
                            }
                            PickerResult::BackToRepoSelector => {
                                // Go back to repo selector
                                app.state = AppState::RepoSelector;
                                app.show_picker = false;
                            }
                            PickerResult::None => {}
                        }
                    }
                    AppState::Viewing => handle_viewing_input(app, key.code, key.modifiers, &current_owner, &current_repo, number),
                    AppState::EditingAction(_) => handle_editing_input(app, key.code, key.modifiers, &current_owner, &current_repo, number),
                    AppState::Error(_) => handle_error_input(app, key.code),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

/// Handle repo selector input, returns Some((owner, repo)) if a repo was selected
fn handle_repo_selector_input(app: &mut App, code: KeyCode) -> Option<(String, String)> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            None
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.repo_selector_down();
            None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.repo_selector_up();
            None
        }
        KeyCode::Char('r') => {
            // Refresh repo list
            if let Ok(repo_list) = github::fetch_repo_list() {
                app.repo_list = repo_list;
                app.repo_selected = 0;
            }
            None
        }
        KeyCode::Enter => {
            // Select repo
            app.selected_repo().map(|r| (r.owner.clone(), r.name.clone()))
        }
        _ => None
    }
}

/// Handle picker input, returns PickerResult
fn handle_picker_input(app: &mut App, code: KeyCode, owner: &str, repo: &str) -> PickerResult {
    match code {
        KeyCode::Char('q') => {
            if app.story.is_some() {
                app.close_picker();
            } else {
                app.should_quit = true;
            }
            PickerResult::None
        }
        KeyCode::Esc | KeyCode::Backspace => {
            if app.story.is_some() {
                app.close_picker();
                PickerResult::None
            } else if !app.repo_list.is_empty() {
                // Go back to repo selector if we have a repo list
                PickerResult::BackToRepoSelector
            } else {
                app.should_quit = true;
                PickerResult::None
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.picker_down();
            PickerResult::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.picker_up();
            PickerResult::None
        }
        KeyCode::Char('r') => {
            // Refresh PR list
            if let Ok(pr_list) = github::fetch_pr_list(owner, repo) {
                app.pr_list = pr_list;
                app.picker_selected = 0;
            }
            PickerResult::None
        }
        KeyCode::Enter => {
            // Select PR
            match app.selected_pr().map(|pr| pr.number) {
                Some(n) => PickerResult::Selected(n),
                None => PickerResult::None,
            }
        }
        _ => PickerResult::None
    }
}

fn handle_viewing_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers, owner: &str, repo: &str, _number: u32) {
    match (code, modifiers) {
        (KeyCode::Char('q'), _) => app.should_quit = true,
        // Open PR picker for current repo
        (KeyCode::Char('o'), KeyModifiers::NONE) => {
            // Load PR list and open picker
            if let Ok(pr_list) = github::fetch_pr_list(owner, repo) {
                app.pr_list = pr_list;
                app.picker_selected = 0;
                app.open_picker();
            }
        }
        // Open repo selector (shift-O)
        (KeyCode::Char('O'), KeyModifiers::SHIFT) => {
            // Load repo list and switch to selector
            if let Ok(repo_list) = github::fetch_repo_list() {
                app.repo_list = repo_list;
                app.repo_selected = 0;
                app.state = AppState::RepoSelector;
            }
        }
        // Line scrolling: j/k or arrows
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
            app.scroll_offset = app.scroll_offset.saturating_add(1)
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1)
        }
        // Page scrolling: Ctrl+d/u or Space/b (vim style)
        (KeyCode::Char('d'), KeyModifiers::CONTROL) | (KeyCode::Char(' '), KeyModifiers::NONE) | (KeyCode::PageDown, _) => {
            app.scroll_offset = app.scroll_offset.saturating_add(20)
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::Char('b'), KeyModifiers::NONE) | (KeyCode::PageUp, _) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(20)
        }
        // Feature navigation: n/p or Tab
        (KeyCode::Tab, _) | (KeyCode::Char('n'), KeyModifiers::NONE) => app.next_feature(),
        (KeyCode::BackTab, _) | (KeyCode::Char('p'), KeyModifiers::NONE) => app.prev_feature(),
        // Diff navigation: h/l or arrows
        (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, _) => app.next_diff(),
        (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, _) => app.prev_diff(),
        // Toggle viewed
        (KeyCode::Char('v'), KeyModifiers::NONE) => app.toggle_viewed(),
        // Action selection
        (KeyCode::Char('1'), _) => app.selected_action = ReviewAction::RequestChanges,
        (KeyCode::Char('2'), _) => app.selected_action = ReviewAction::ClarificationQuestions,
        (KeyCode::Char('3'), _) => app.selected_action = ReviewAction::NextPr,
        // Edit action
        (KeyCode::Enter, _) => app.start_editing(),
        _ => {}
    }
}

fn handle_editing_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers, owner: &str, repo: &str, number: u32) {
    match code {
        KeyCode::Esc => app.stop_editing(),
        KeyCode::Enter => app.insert_char('\n'),
        KeyCode::Backspace => app.delete_char(),
        KeyCode::Left => app.cursor_left(),
        KeyCode::Right => app.cursor_right(),
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            // Submit action
            let action = app.selected_action;
            let text = app.current_action_text().to_string();

            if text.is_empty() {
                app.status = Some("Cannot submit empty text".to_string());
                return;
            }

            app.state = AppState::Submitting(action);

            let result = match action {
                ReviewAction::RequestChanges => {
                    github::post_review(owner, repo, number, &text)
                }
                ReviewAction::ClarificationQuestions => {
                    github::post_comment(owner, repo, number, &text)
                }
                ReviewAction::NextPr => {
                    // Extract title from first line
                    let lines: Vec<&str> = text.lines().collect();
                    let title = lines.first().unwrap_or(&"Follow-up work");
                    let body = lines.get(1..).map(|l| l.join("\n")).unwrap_or_default();
                    github::create_next_pr_issue(owner, repo, number, title, &body).map(|_| ())
                }
            };

            match result {
                Ok(_) => {
                    app.status = Some(format!("{} submitted successfully!", action.title()));
                    app.state = AppState::Viewing;
                }
                Err(e) => {
                    app.status = Some(format!("Error: {}", e));
                    app.state = AppState::Viewing;
                }
            }
        }
        KeyCode::Char(c) => app.insert_char(c),
        _ => {}
    }
}

fn handle_error_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('r') => {
            // Reset to try again
            app.state = AppState::LoadingPr;
        }
        _ => {}
    }
}
