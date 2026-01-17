use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Render the fixed header with PR info
pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if let Some(pr) = &app.pr {
        lines.push(Line::from(vec![
            Span::styled(
                "Distillery",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}#{}", pr.owner, pr.repo, pr.number),
                Style::default().fg(Color::White),
            ),
        ]));
        lines.push(Line::from(vec![Span::styled(
            &pr.title,
            Style::default().fg(Color::Yellow),
        )]));
    }

    let header = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(header, area);
}
