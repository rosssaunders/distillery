use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render_loading(frame: &mut Frame, area: Rect, message: &str) {
    let loading = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("⏳ {}", message),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
    ]);
    let loading = loading.wrap(Wrap { trim: false });

    frame.render_widget(loading, area);
}
