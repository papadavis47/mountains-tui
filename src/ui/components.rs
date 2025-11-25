use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
};

pub fn create_title_style() -> Style {
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD)
}

pub fn create_input_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn create_highlight_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

pub fn create_standard_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area)
}

pub fn render_title(f: &mut Frame, area: Rect, title: &str) {
    let title_widget = Paragraph::new(title)
        .style(create_title_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(255, 165, 0)))
                .padding(Padding::uniform(1))
        );
    f.render_widget(title_widget, area);
}

pub fn render_help(f: &mut Frame, area: Rect, help_text: &str, show_border: bool, centered: bool) {
    let mut spans = Vec::new();

    // Split by pipe separator to get individual commands
    for (i, segment) in help_text.split('|').enumerate() {
        if i > 0 {
            // Add the pipe separator in white
            spans.push(Span::styled(" | ", Style::default().fg(Color::White)));
        }

        let trimmed = segment.trim();

        // Split by colon to separate key from description
        if let Some(colon_pos) = trimmed.find(':') {
            let key_part = trimmed[..colon_pos].trim();
            let desc_part = trimmed[colon_pos + 1..].trim();

            // Key in yellow
            spans.push(Span::styled(
                key_part.to_string(),
                Style::default().fg(Color::Yellow)
            ));

            // Colon and description in white
            spans.push(Span::styled(
                format!(": {}", desc_part),
                Style::default().fg(Color::White)
            ));
        } else {
            // If no colon, just display in white
            spans.push(Span::styled(
                trimmed.to_string(),
                Style::default().fg(Color::White)
            ));
        }
    }

    let block = if show_border {
        Block::default().borders(Borders::ALL)
    } else {
        Block::default().borders(Borders::NONE)
    };

    let mut help_widget = Paragraph::new(Line::from(spans))
        .block(block);

    if centered {
        help_widget = help_widget.alignment(ratatui::layout::Alignment::Center);
    }

    f.render_widget(help_widget, area);
}

pub fn format_input_with_cursor(input: &str) -> String {
    if input.is_empty() {
        " ".to_string() // Show space for cursor when empty
    } else {
        input.to_string()
    }
}

pub fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}
