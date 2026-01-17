use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::domain::types::CiStatus;

use super::util::truncate;

pub fn render_picker(frame: &mut Frame, app: &App, area: Rect) {
    render_picker_content(frame, app, area, false);
}

pub fn render_picker_overlay(frame: &mut Frame, app: &App, area: Rect) {
    // Create centered popup area
    let popup_area = centered_rect(80, 70, area);
    frame.render_widget(Clear, popup_area);
    render_picker_content(frame, app, popup_area, true);
}

fn render_picker_content(frame: &mut Frame, app: &App, area: Rect, is_overlay: bool) {
    let mut lines: Vec<Line> = Vec::new();

    // Header with repo name
    let repo_name = app
        .current_repo
        .as_ref()
        .map(|(o, r)| format!("{}/{}", o, r))
        .unwrap_or_else(|| "Unknown".to_string());

    lines.push(Line::from(vec![
        Span::styled(
            "SELECT PR",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(repo_name, Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));

    if app.pr_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "No open PRs found",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Group markers
        let mut last_section: Option<&str> = None;

        for (i, pr) in app.pr_list.iter().enumerate() {
            // Determine section
            let section = if pr.is_draft {
                "DRAFTS"
            } else if pr.review_requested {
                "REVIEW REQUESTED"
            } else {
                "OPEN"
            };

            // Section header if changed
            if last_section != Some(section) {
                if last_section.is_some() {
                    lines.push(Line::from(""));
                }
                let section_color = match section {
                    "REVIEW REQUESTED" => Color::Yellow,
                    "DRAFTS" => Color::DarkGray,
                    _ => Color::White,
                };
                lines.push(Line::from(Span::styled(
                    format!("── {} ──", section),
                    Style::default().fg(section_color),
                )));
                last_section = Some(section);
            }

            let is_selected = i == app.picker_selected;

            // CI status indicator
            let ci_color = match pr.ci_status {
                CiStatus::Success => Color::Green,
                CiStatus::Failure => Color::Red,
                CiStatus::Pending => Color::Yellow,
                CiStatus::Unknown => Color::DarkGray,
            };

            // Build the line
            let marker = if is_selected { "▶ " } else { "  " };
            let title = truncate(&pr.title, 50);

            let line_style = if is_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else if pr.is_draft {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Cyan)),
                Span::styled(pr.ci_status.symbol(), Style::default().fg(ci_color)),
                Span::styled(" ", Style::default()),
                Span::styled(format!("#{:<5}", pr.number), Style::default().fg(Color::Blue)),
                Span::styled(title, line_style),
            ]));

            // Second line with author and stats
            lines.push(Line::from(vec![
                Span::styled("     ", Style::default()),
                Span::styled(pr.author.clone(), Style::default().fg(Color::DarkGray)),
                Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("+{}", pr.additions), Style::default().fg(Color::Green)),
                Span::styled("/", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("-{}", pr.deletions), Style::default().fg(Color::Red)),
                Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                Span::styled(pr.head_branch.clone(), Style::default().fg(Color::Magenta)),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_overlay { Color::Cyan } else { Color::DarkGray }))
        .title(if is_overlay { " PR Picker " } else { " Pull Requests " });

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
