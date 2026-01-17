use crossterm::event::KeyCode;

use crate::app::App;
use crate::command::Command;

pub fn handle_input(app: &mut App, code: KeyCode) -> Vec<Command> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            Vec::new()
        }
        _ => Vec::new(),
    }
}
