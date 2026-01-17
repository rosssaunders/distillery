use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render_error(frame: &mut Frame, area: Rect, message: &str) {
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
