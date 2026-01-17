use crossterm::event::KeyCode;

use crate::app::{App, AppState};
use crate::command::Command;

use super::helpers;

pub fn handle_input(app: &mut App, code: KeyCode) -> Vec<Command> {
    match code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            Vec::new()
        }
        KeyCode::Char('r') => retry_from_error(app),
        _ => Vec::new(),
    }
}

fn retry_from_error(app: &mut App) -> Vec<Command> {
    if let Some((owner, repo, number)) = helpers::current_pr_ref(app) {
        app.state = AppState::LoadingPr;
        return vec![Command::FetchPr {
            owner,
            repo,
            number,
        }];
    }

    if let Some((owner, repo)) = helpers::current_repo(app) {
        app.state = AppState::LoadingPrList;
        return vec![Command::FetchPrList { owner, repo }];
    }

    app.state = AppState::LoadingRepoList;
    vec![Command::FetchRepoList]
}
