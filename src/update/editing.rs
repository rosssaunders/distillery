use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, AppState};
use crate::command::Command;
use crate::domain::types::ReviewAction;

use super::helpers;

pub fn handle_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Vec<Command> {
    match code {
        KeyCode::Esc => {
            app.stop_editing();
            Vec::new()
        }
        KeyCode::Enter => {
            app.insert_char('\n');
            Vec::new()
        }
        KeyCode::Backspace => {
            app.delete_char();
            Vec::new()
        }
        KeyCode::Left => {
            app.cursor_left();
            Vec::new()
        }
        KeyCode::Right => {
            app.cursor_right();
            Vec::new()
        }
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            let action = app.selected_action;
            let text = app.current_action_text().to_string();

            if text.is_empty() {
                app.status = Some("Cannot submit empty text".to_string());
                return Vec::new();
            }

            let Some((owner, repo, number)) = helpers::current_pr_ref(app) else {
                app.status = Some("Missing PR context".to_string());
                app.state = AppState::Viewing;
                return Vec::new();
            };

            app.state = AppState::Submitting(action);

            match action {
                ReviewAction::RequestChanges => vec![Command::PostReview {
                    owner,
                    repo,
                    number,
                    body: text,
                }],
                ReviewAction::ClarificationQuestions => vec![Command::PostComment {
                    owner,
                    repo,
                    number,
                    body: text,
                }],
                ReviewAction::NextPr => {
                    let mut iter = text.lines();
                    let title = iter
                        .next()
                        .unwrap_or("Follow-up work")
                        .to_string();
                    let body = iter.collect::<Vec<&str>>().join("\n");
                    vec![Command::CreateNextPrIssue {
                        owner,
                        repo,
                        number,
                        title,
                        body,
                    }]
                }
            }
        }
        KeyCode::Char(c) => {
            app.insert_char(c);
            Vec::new()
        }
        _ => Vec::new(),
    }
}
