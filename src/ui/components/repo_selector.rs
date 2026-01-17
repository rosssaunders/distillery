use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

use super::util::truncate;

pub fn render_repo_selector(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![Span::styled(
        "SELECT REPOSITORY",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    if app.repo_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "No repositories found",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, repo) in app.repo_list.iter().enumerate() {
            let is_selected = i == app.repo_selected;

            // Build the line
            let marker = if is_selected { "▶ " } else { "  " };

            // Repo name with owner
            let repo_name = format!("{}/{}", repo.owner, repo.name);
            let repo_display = truncate(&repo_name, 40);

            let line_style = if is_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Visibility indicator
            let visibility = if repo.is_private {
                Span::styled(" 🔒", Style::default().fg(Color::Yellow))
            } else {
                Span::styled("", Style::default())
            };

            // Fork indicator
            let fork_indicator = if repo.is_fork {
                Span::styled(" ⑂", Style::default().fg(Color::DarkGray))
            } else {
                Span::styled("", Style::default())
            };

            lines.push(Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Cyan)),
                Span::styled(repo_display, line_style),
                visibility,
                fork_indicator,
            ]));

            // Description on second line (if present and selected or short list)
            if !repo.description.is_empty() {
                let desc = truncate(&repo.description, 60);
                let desc_style = Style::default().fg(Color::DarkGray);
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(desc, desc_style),
                ]));
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Repositories ");

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
