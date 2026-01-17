use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::App;
use crate::command::Command;
use crate::domain::types::ReviewAction;

use super::helpers;

pub fn handle_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Vec<Command> {
    match (code, modifiers) {
        (KeyCode::Char('q'), _) => {
            app.should_quit = true;
            Vec::new()
        }
        (KeyCode::Char('o'), KeyModifiers::NONE) => {
            let Some((owner, repo)) = helpers::current_repo(app) else {
                return Vec::new();
            };
            vec![Command::FetchPrList { owner, repo }]
        }
        (KeyCode::Char('O'), KeyModifiers::SHIFT) => vec![Command::FetchRepoList],
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => {
            app.scroll_offset = app.scroll_offset.saturating_add(1);
            Vec::new()
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
            Vec::new()
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL)
        | (KeyCode::Char(' '), KeyModifiers::NONE)
        | (KeyCode::PageDown, _) => {
            app.scroll_offset = app.scroll_offset.saturating_add(20);
            Vec::new()
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL)
        | (KeyCode::Char('b'), KeyModifiers::NONE)
        | (KeyCode::PageUp, _) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(20);
            Vec::new()
        }
        (KeyCode::Tab, _) | (KeyCode::Char('n'), KeyModifiers::NONE) => {
            app.next_feature();
            Vec::new()
        }
        (KeyCode::BackTab, _) | (KeyCode::Char('p'), KeyModifiers::NONE) => {
            app.prev_feature();
            Vec::new()
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, _) => {
            app.next_diff();
            Vec::new()
        }
        (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, _) => {
            app.prev_diff();
            Vec::new()
        }
        (KeyCode::Char('v'), KeyModifiers::NONE) => {
            app.toggle_viewed();
            Vec::new()
        }
        (KeyCode::Char('1'), _) => {
            app.selected_action = ReviewAction::RequestChanges;
            Vec::new()
        }
        (KeyCode::Char('2'), _) => {
            app.selected_action = ReviewAction::ClarificationQuestions;
            Vec::new()
        }
        (KeyCode::Char('3'), _) => {
            app.selected_action = ReviewAction::NextPr;
            Vec::new()
        }
        (KeyCode::Enter, _) => {
            app.start_editing();
            Vec::new()
        }
        _ => Vec::new(),
    }
}
