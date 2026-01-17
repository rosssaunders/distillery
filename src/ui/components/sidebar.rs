use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::domain::types::Significance;

use super::util::truncate;

pub fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Progress header
    let (viewed, total) = app.total_progress();
    let progress_pct = if total > 0 {
        (viewed * 100) / total
    } else {
        0
    };

    lines.push(Line::from(vec![
        Span::styled(
            "PROGRESS ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}/{} ({}%)", viewed, total, progress_pct),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));

    // Progress bar
    let bar_width = 28;
    let filled = if total > 0 { (viewed * bar_width) / total } else { 0 };
    let empty = bar_width - filled;
    lines.push(Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(Color::Green)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─".repeat(30),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    // Feature list
    if let Some(story) = &app.story {
        for (i, feature) in story.narrative.iter().enumerate() {
            let is_selected = i == app.selected_feature;
            let (feat_viewed, feat_total) = app.feature_progress(i);
            let all_viewed = feat_viewed == feat_total && feat_total > 0;

            // Feature marker
            let marker = if is_selected {
                "▶ "
            } else if all_viewed {
                "✓ "
            } else {
                "  "
            };

            let marker_color = if all_viewed {
                Color::Green
            } else if is_selected {
                Color::Cyan
            } else {
                Color::DarkGray
            };

            // Feature title (truncated)
            let title = truncate(&feature.title, 20);
            let title_style = if is_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else if all_viewed {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(marker, Style::default().fg(marker_color)),
                Span::styled(title, title_style),
            ]));

            // Progress for this feature
            let progress_style = if all_viewed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(format!("{}/{} diffs", feat_viewed, feat_total), progress_style),
            ]));

            // If selected, show diff list
            if is_selected {
                for (j, block) in feature.diff_blocks.iter().enumerate() {
                    let is_diff_selected = j == app.selected_diff;
                    let is_viewed = app.is_diff_viewed(i, j);

                    let diff_marker = if is_diff_selected {
                        "→ "
                    } else if is_viewed {
                        "✓ "
                    } else {
                        "  "
                    };

                    let diff_marker_color = if is_viewed {
                        Color::Green
                    } else if is_diff_selected {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    };

                    // Significance marker
                    let (sig_marker, sig_color) = match block.significance {
                        Significance::Key => ("★", Color::Yellow),
                        Significance::Standard => (" ", Color::DarkGray),
                        Significance::Noise => ("·", Color::DarkGray),
                    };

                    let label = truncate(&block.label, 20);
                    let label_style = if block.significance == Significance::Noise {
                        Style::default().fg(Color::DarkGray)
                    } else if is_diff_selected {
                        Style::default().fg(Color::Yellow)
                    } else if is_viewed {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(diff_marker, Style::default().fg(diff_marker_color)),
                        Span::styled(sig_marker, Style::default().fg(sig_color)),
                        Span::styled(" ", Style::default()),
                        Span::styled(label, label_style),
                    ]));
                }
            }

            lines.push(Line::from(""));
        }
    }

    let sidebar = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(sidebar, area);
}
