use crossterm::event::KeyCode;

use crate::app::{App, AppState};
use crate::command::Command;

use super::helpers;

pub fn handle_input(app: &mut App, code: KeyCode) -> Vec<Command> {
    match code {
        KeyCode::Char('q') => {
            if app.story.is_some() {
                app.close_picker();
            } else {
                app.should_quit = true;
            }
            Vec::new()
        }
        KeyCode::Esc | KeyCode::Backspace => {
            if app.story.is_some() {
                app.close_picker();
            } else if !app.repo_list.is_empty() {
                app.back_to_repo_selector();
            } else {
                app.should_quit = true;
            }
            Vec::new()
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.picker_down();
            Vec::new()
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.picker_up();
            Vec::new()
        }
        KeyCode::Char('r') => {
            let Some((owner, repo)) = helpers::current_repo(app) else {
                return Vec::new();
            };
            vec![Command::FetchPrList { owner, repo }]
        }
        KeyCode::Enter => {
            let Some(pr) = app.selected_pr() else {
                return Vec::new();
            };
            let Some((owner, repo)) = helpers::current_repo(app) else {
                return Vec::new();
            };

            let number = pr.number;
            app.reset_for_new_pr();
            app.current_pr_number = Some(number);
            app.state = AppState::LoadingPr;
            vec![Command::FetchPr {
                owner,
                repo,
                number,
            }]
        }
        _ => Vec::new(),
    }
}
