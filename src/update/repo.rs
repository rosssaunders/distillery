use crossterm::event::KeyCode;

use crate::app::{App, AppState};
use crate::command::Command;

pub fn handle_input(app: &mut App, code: KeyCode) -> Vec<Command> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            Vec::new()
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.repo_selector_down();
            Vec::new()
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.repo_selector_up();
            Vec::new()
        }
        KeyCode::Char('r') => {
            app.state = AppState::LoadingRepoList;
            vec![Command::FetchRepoList]
        }
        KeyCode::Enter => {
            let Some(repo) = app.selected_repo() else {
                return Vec::new();
            };
            let owner = repo.owner.clone();
            let repo_name = repo.name.clone();
            app.current_repo = Some((owner.clone(), repo_name.clone()));
            app.current_pr_number = None;
            app.state = AppState::LoadingPrList;
            app.show_picker = false;
            vec![Command::FetchPrList {
                owner,
                repo: repo_name,
            }]
        }
        _ => Vec::new(),
    }
}
