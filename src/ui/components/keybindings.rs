use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppState};

/// Render the keybindings bar at the bottom
pub fn render_keybindings(frame: &mut Frame, app: &App, area: Rect) {
    let keys: Vec<(&str, &str)> = match &app.state {
        AppState::LoadingRepoList
        | AppState::LoadingPrList
        | AppState::LoadingPr
        | AppState::GeneratingStory => {
            vec![("q", "Quit")]
        }
        AppState::RepoSelector => {
            vec![
                ("j/↓", "Down"),
                ("k/↑", "Up"),
                ("Enter", "Select"),
                ("r", "Refresh"),
                ("q", "Quit"),
            ]
        }
        AppState::PrPicker => {
            if !app.repo_list.is_empty() && !app.show_picker {
                vec![
                    ("j/↓", "Down"),
                    ("k/↑", "Up"),
                    ("Enter", "Select"),
                    ("Esc", "Back"),
                    ("r", "Refresh"),
                    ("q", "Quit"),
                ]
            } else {
                vec![
                    ("j/↓", "Down"),
                    ("k/↑", "Up"),
                    ("Enter", "Select"),
                    ("r", "Refresh"),
                    ("Esc", "Cancel"),
                ]
            }
        }
        AppState::Viewing => {
            vec![
                ("j/k", "Scroll"),
                ("Space/b", "Page"),
                ("h/l", "Diff"),
                ("n/p", "Feature"),
                ("v", "Viewed"),
                ("1-3", "Actions"),
                ("o", "PRs"),
                ("O", "Repos"),
                ("q", "Quit"),
            ]
        }
        AppState::EditingAction(action) => {
            vec![
                ("Editing", action.title()),
                ("Type", "Edit text"),
                ("Ctrl+S", "Submit"),
                ("Esc", "Done"),
            ]
        }
        AppState::Submitting(action) => vec![("Submitting", action.title())],
        AppState::Error(_) => vec![("q", "Quit"), ("r", "Retry")],
    };

    // Build the line with key highlights
    let mut spans: Vec<Span> = vec![Span::styled(" ", Style::default())];

    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(
            *key,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::White),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}
