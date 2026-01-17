use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::domain::types::{DiffRole, ReviewAction, Significance};

pub fn render_document(frame: &mut Frame, app: &App, area: Rect) {
    // Build the full document as lines
    let mut lines: Vec<Line> = Vec::new();

    if let Some(story) = &app.story {
        // Summary
        lines.push(Line::from(vec![Span::styled(
            "SUMMARY",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(Span::styled(
            &story.summary,
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(
                "Files: {} │ +{} -{}",
                story.data.files_touched, story.data.additions, story.data.deletions
            ),
            Style::default().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(""));

        // Focus section
        lines.push(Line::from(Span::styled(
            "━".repeat(70),
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(vec![
            Span::styled(
                "⚡ FOCUS: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &story.focus.key_change,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));

        // Review these
        if !story.focus.review_these.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("👁 Review: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    story.focus.review_these.join(" │ "),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        // Skim these
        if !story.focus.skim_these.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("⏭ Skim: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    story.focus.skim_these.join(" │ "),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
        lines.push(Line::from(Span::styled(
            "━".repeat(70),
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled(
            "─".repeat(70),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        // Each feature
        for (i, feature) in story.narrative.iter().enumerate() {
            let is_selected = i == app.selected_feature;
            let marker = if is_selected { "▶ " } else { "  " };

            // Feature title
            lines.push(Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("FEATURE {}: ", i + 1),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &feature.title,
                    if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]));

            // Why
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(&feature.why, Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));

            // Changes
            if !feature.changes.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "   Changes: ",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )]));
                for change in &feature.changes {
                    lines.push(Line::from(vec![
                        Span::styled("   • ", Style::default().fg(Color::Green)),
                        Span::styled(change, Style::default().fg(Color::White)),
                    ]));
                }
            }

            // Risks
            if !feature.risks.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "   Risks: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]));
                for risk in &feature.risks {
                    lines.push(Line::from(vec![
                        Span::styled("   • ", Style::default().fg(Color::Red)),
                        Span::styled(risk, Style::default().fg(Color::White)),
                    ]));
                }
            }

            // Tests
            if !feature.tests.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "   Tests: ",
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                )]));
                for test in &feature.tests {
                    lines.push(Line::from(vec![
                        Span::styled("   • ", Style::default().fg(Color::Blue)),
                        Span::styled(test, Style::default().fg(Color::White)),
                    ]));
                }
            }

            lines.push(Line::from(""));

            // Diff blocks
            for (j, block) in feature.diff_blocks.iter().enumerate() {
                let is_diff_selected = is_selected && j == app.selected_diff;
                let is_viewed = app.is_diff_viewed(i, j);
                let is_noise = block.significance == Significance::Noise;

                let role_color = match block.role {
                    DiffRole::Root => Color::Magenta,
                    DiffRole::Downstream => Color::Blue,
                    DiffRole::Supporting => Color::DarkGray,
                };

                // Significance badge
                let significance_badge = match block.significance {
                    Significance::Key => Span::styled(
                        "★ KEY ",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Significance::Standard => Span::styled("", Style::default()),
                    Significance::Noise => {
                        Span::styled("· noise ", Style::default().fg(Color::DarkGray))
                    }
                };

                // Diff header with viewed status
                let viewed_marker = if is_viewed { " ✓" } else { "" };
                let selection_marker = if is_diff_selected { ">> " } else { "   " };

                // Apply dimming for noise blocks
                let label_style = if is_noise {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(role_color).add_modifier(Modifier::BOLD)
                };
                let role_style = if is_noise {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(role_color)
                };

                lines.push(Line::from(vec![
                    Span::styled(selection_marker, Style::default().fg(Color::Yellow)),
                    Span::styled("┌─ ", Style::default().fg(Color::DarkGray)),
                    significance_badge,
                    Span::styled(&block.label, label_style),
                    Span::styled(format!(" [{}]", block.role.as_str()), role_style),
                    Span::styled(viewed_marker, Style::default().fg(Color::Green)),
                ]));

                // Context (why) - on the right conceptually, but we show it inline
                let context_color = if is_noise { Color::DarkGray } else { Color::White };
                let why_color = if is_noise { Color::DarkGray } else { Color::Yellow };
                lines.push(Line::from(vec![
                    Span::styled("   │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("WHY: ", Style::default().fg(why_color)),
                    Span::styled(&block.context, Style::default().fg(context_color)),
                ]));

                // Hunks
                for hunk in &block.hunks {
                    let header_color = if is_noise { Color::DarkGray } else { Color::Cyan };
                    lines.push(Line::from(vec![
                        Span::styled("   │ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(&hunk.header, Style::default().fg(header_color)),
                    ]));

                    for diff_line in hunk.lines.lines() {
                        let (style, line_text) = if is_noise {
                            // Dim all lines for noise blocks
                            (Style::default().fg(Color::DarkGray), diff_line)
                        } else if diff_line.starts_with('+') {
                            (Style::default().fg(Color::Green), diff_line)
                        } else if diff_line.starts_with('-') {
                            (Style::default().fg(Color::Red), diff_line)
                        } else if diff_line.starts_with("@@") {
                            (Style::default().fg(Color::Cyan), diff_line)
                        } else {
                            (Style::default().fg(Color::DarkGray), diff_line)
                        };

                        lines.push(Line::from(vec![
                            Span::styled("   │ ", Style::default().fg(Color::DarkGray)),
                            Span::styled(line_text, style),
                        ]));
                    }
                }

                lines.push(Line::from(vec![Span::styled(
                    "   └─",
                    Style::default().fg(Color::DarkGray),
                )]));
                lines.push(Line::from(""));
            }

            lines.push(Line::from(Span::styled(
                "─".repeat(70),
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
        }

        // Open questions
        if !story.open_questions.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "OPEN QUESTIONS",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]));
            for q in &story.open_questions {
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(Color::Yellow)),
                    Span::styled(q, Style::default().fg(Color::White)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─".repeat(70),
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
        }

        // Action boxes (just show titles, press key to expand)
        lines.push(Line::from(vec![
            Span::styled(
                "ACTIONS",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " (1: Request Changes, 2: Clarify, 3: Next PR, Enter to edit)",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        // Show selected action preview
        let (action_title, action_text, action_color) = match app.selected_action {
            ReviewAction::RequestChanges => (
                "Request Changes",
                &app.action_texts.request_changes,
                Color::Red,
            ),
            ReviewAction::ClarificationQuestions => (
                "Clarification Questions",
                &app.action_texts.clarification,
                Color::Blue,
            ),
            ReviewAction::NextPr => ("Next PR", &app.action_texts.next_pr, Color::Green),
        };

        lines.push(Line::from(vec![
            Span::styled("▶ ", Style::default().fg(action_color)),
            Span::styled(
                action_title,
                Style::default().fg(action_color).add_modifier(Modifier::BOLD),
            ),
        ]));

        for text_line in action_text.lines().take(5) {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(text_line, Style::default().fg(Color::White)),
            ]));
        }

        if action_text.lines().count() > 5 {
            lines.push(Line::from(vec![Span::styled(
                "  ... (press Enter to edit full text)",
                Style::default().fg(Color::DarkGray),
            )]));
        }
    }

    // Status message if any
    if let Some(status) = &app.status {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            status,
            Style::default().fg(Color::Yellow),
        )));
    }

    // Render with scroll
    let paragraph = Paragraph::new(lines)
        .scroll((app.scroll_offset, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
