use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppState};
use crate::types::{CiStatus, DiffRole, Significance};

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Split into main area and bottom keybindings bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),     // Main content
            Constraint::Length(3), // Keybindings bar
        ])
        .split(size);

    let main_area = chunks[0];
    let keys_area = chunks[1];

    // Render main content based on state
    match &app.state {
        AppState::LoadingRepoList => render_loading(frame, main_area, "Fetching repositories..."),
        AppState::RepoSelector => render_repo_selector(frame, app, main_area),
        AppState::LoadingPrList => render_loading(frame, main_area, "Fetching PR list..."),
        AppState::LoadingPr => render_loading(frame, main_area, "Fetching PR from GitHub..."),
        AppState::GeneratingStory => render_loading(frame, main_area, "Generating story with AI..."),
        AppState::Error(msg) => render_error(frame, main_area, msg),
        AppState::PrPicker => render_picker(frame, app, main_area),
        AppState::Viewing | AppState::EditingAction(_) | AppState::Submitting(_) => {
            render_main(frame, app, main_area);
            // Show picker as overlay if open
            if app.show_picker {
                render_picker_overlay(frame, app, main_area);
            }
        }
    }

    // Always render keybindings bar at bottom
    render_keybindings(frame, app, keys_area);
}

/// Render the keybindings bar at the bottom
fn render_keybindings(frame: &mut Frame, app: &App, area: Rect) {
    let keys: Vec<(&str, &str)> = match &app.state {
        AppState::LoadingRepoList | AppState::LoadingPrList | AppState::LoadingPr | AppState::GeneratingStory => {
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
        AppState::EditingAction(_) => {
            vec![
                ("Type", "Edit text"),
                ("Ctrl+S", "Submit"),
                ("Esc", "Done"),
            ]
        }
        AppState::Submitting(_) => {
            vec![("", "Submitting...")]
        }
        AppState::Error(_) => {
            vec![("q", "Quit"), ("r", "Retry")]
        }
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

    let paragraph = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray))
        );

    frame.render_widget(paragraph, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    // Split into header and content area
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header (app name, repo, title)
            Constraint::Min(10),   // Content area
        ])
        .split(area);

    render_header(frame, app, vertical_chunks[0]);

    // Split content into sidebar and main document
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32), // Sidebar
            Constraint::Min(50),    // Main content
        ])
        .split(vertical_chunks[1]);

    render_sidebar(frame, app, horizontal_chunks[0]);
    render_document(frame, app, horizontal_chunks[1]);
}

/// Render the fixed header with PR info
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if let Some(pr) = &app.pr {
        lines.push(Line::from(vec![
            Span::styled("Distillery", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}/{}#{}", pr.owner, pr.repo, pr.number), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(&pr.title, Style::default().fg(Color::Yellow)),
        ]));
    }

    let header = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(header, area);
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Progress header
    let (viewed, total) = app.total_progress();
    let progress_pct = if total > 0 {
        (viewed * 100) / total
    } else {
        0
    };

    lines.push(Line::from(vec![
        Span::styled("PROGRESS ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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
    lines.push(Line::from(Span::styled("─".repeat(30), Style::default().fg(Color::DarkGray))));
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

fn render_document(frame: &mut Frame, app: &App, area: Rect) {
    // Build the full document as lines
    let mut lines: Vec<Line> = Vec::new();

    if let Some(story) = &app.story {
        // Summary
        lines.push(Line::from(vec![
            Span::styled("SUMMARY", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(Span::styled(&story.summary, Style::default().fg(Color::White))));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("Files: {} │ +{} -{}", story.data.files_touched, story.data.additions, story.data.deletions),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        // Focus section
        lines.push(Line::from(Span::styled("━".repeat(70), Style::default().fg(Color::Yellow))));
        lines.push(Line::from(vec![
            Span::styled("⚡ FOCUS: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&story.focus.key_change, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));

        // Review these
        if !story.focus.review_these.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("👁 Review: ", Style::default().fg(Color::Cyan)),
                Span::styled(story.focus.review_these.join(" │ "), Style::default().fg(Color::White)),
            ]));
        }

        // Skim these
        if !story.focus.skim_these.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("⏭ Skim: ", Style::default().fg(Color::DarkGray)),
                Span::styled(story.focus.skim_these.join(" │ "), Style::default().fg(Color::DarkGray)),
            ]));
        }
        lines.push(Line::from(Span::styled("━".repeat(70), Style::default().fg(Color::Yellow))));
        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled("─".repeat(70), Style::default().fg(Color::DarkGray))));
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
                lines.push(Line::from(vec![
                    Span::styled("   Changes: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
                for change in &feature.changes {
                    lines.push(Line::from(vec![
                        Span::styled("   • ", Style::default().fg(Color::Green)),
                        Span::styled(change, Style::default().fg(Color::White)),
                    ]));
                }
            }

            // Risks
            if !feature.risks.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("   Risks: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]));
                for risk in &feature.risks {
                    lines.push(Line::from(vec![
                        Span::styled("   • ", Style::default().fg(Color::Red)),
                        Span::styled(risk, Style::default().fg(Color::White)),
                    ]));
                }
            }

            // Tests
            if !feature.tests.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("   Tests: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                ]));
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
                    Significance::Key => Span::styled("★ KEY ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Significance::Standard => Span::styled("", Style::default()),
                    Significance::Noise => Span::styled("· noise ", Style::default().fg(Color::DarkGray)),
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

                lines.push(Line::from(vec![
                    Span::styled("   └─", Style::default().fg(Color::DarkGray)),
                ]));
                lines.push(Line::from(""));
            }

            lines.push(Line::from(Span::styled("─".repeat(70), Style::default().fg(Color::DarkGray))));
            lines.push(Line::from(""));
        }

        // Open questions
        if !story.open_questions.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("OPEN QUESTIONS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            for q in &story.open_questions {
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(Color::Yellow)),
                    Span::styled(q, Style::default().fg(Color::White)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("─".repeat(70), Style::default().fg(Color::DarkGray))));
            lines.push(Line::from(""));
        }

        // Action boxes (just show titles, press key to expand)
        lines.push(Line::from(vec![
            Span::styled("ACTIONS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" (1: Request Changes, 2: Clarify, 3: Next PR, Enter to edit)", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));

        // Show selected action preview
        let (action_title, action_text, action_color) = match app.selected_action {
            crate::types::ReviewAction::RequestChanges => {
                ("Request Changes", &app.action_texts.request_changes, Color::Red)
            }
            crate::types::ReviewAction::ClarificationQuestions => {
                ("Clarification Questions", &app.action_texts.clarification, Color::Blue)
            }
            crate::types::ReviewAction::NextPr => {
                ("Next PR", &app.action_texts.next_pr, Color::Green)
            }
        };

        lines.push(Line::from(vec![
            Span::styled("▶ ", Style::default().fg(action_color)),
            Span::styled(action_title, Style::default().fg(action_color).add_modifier(Modifier::BOLD)),
        ]));

        for text_line in action_text.lines().take(5) {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(text_line, Style::default().fg(Color::White)),
            ]));
        }

        if action_text.lines().count() > 5 {
            lines.push(Line::from(vec![
                Span::styled("  ... (press Enter to edit full text)", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    // Status message if any
    if let Some(status) = &app.status {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(status, Style::default().fg(Color::Yellow))));
    }

    // Render with scroll
    let paragraph = Paragraph::new(lines)
        .scroll((app.scroll_offset, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_loading(frame: &mut Frame, area: Rect, message: &str) {
    let loading = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("⏳ {}", message),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
    ])
    .wrap(Wrap { trim: false });

    frame.render_widget(loading, area);
}

fn render_error(frame: &mut Frame, area: Rect, message: &str) {
    let error = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Error: {}", message),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
    ])
    .wrap(Wrap { trim: false });

    frame.render_widget(error, area);
}

fn render_repo_selector(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled(
            "SELECT REPOSITORY",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]));
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
                let desc_style = if is_selected {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
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

fn render_picker(frame: &mut Frame, app: &App, area: Rect) {
    render_picker_content(frame, app, area, false);
}

fn render_picker_overlay(frame: &mut Frame, app: &App, area: Rect) {
    // Create centered popup area
    let popup_area = centered_rect(80, 70, area);
    frame.render_widget(Clear, popup_area);
    render_picker_content(frame, app, popup_area, true);
}

fn render_picker_content(frame: &mut Frame, app: &App, area: Rect, is_overlay: bool) {
    let mut lines: Vec<Line> = Vec::new();

    // Header with repo name
    let repo_name = app.current_repo
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

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max_len - 1).collect::<String>())
    }
}
