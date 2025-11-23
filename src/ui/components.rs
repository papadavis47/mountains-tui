/// This module contains helper functions and reusable components that are used
/// across multiple screens.
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
};

/// Creates the standard title styling used throughout the application
///
/// This function demonstrates Rust's approach to creating reusable components.
/// Instead of duplicating styling code, we centralize it in one place.
pub fn create_title_style() -> Style {
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD)
}

/// Creates the standard input field styling (yellow text)
pub fn create_input_style() -> Style {
    Style::default().fg(Color::Yellow)
}

/// Creates the standard highlight style for selected list items
pub fn create_highlight_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Creates a standard three-section layout used by many screens
///
/// This function returns a Layout with three sections:
/// - Top: Fixed height for titles (5 lines to accommodate padding)
/// - Middle: Expandable content area
/// - Bottom: Fixed height for help text (3 lines)
///
/// The `Constraint` enum defines how space is allocated:
/// - `Length(n)`: Fixed height of n lines
/// - `Min(n)`: Takes remaining space, minimum n lines
pub fn create_standard_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1) // 1-character margin on all sides
        .constraints([
            Constraint::Length(5), // Title area (increased for vertical padding)
            Constraint::Min(0),    // Content area (takes remaining space)
            Constraint::Length(3), // Help area
        ])
        .split(area)
}

/// Renders a title widget with the application's standard styling
///
/// This function takes a title string and renders it in a bordered box
/// with green/bold text, orange borders (RGB: 255, 165, 0), rounded borders, and padding on all sides.
pub fn render_title(f: &mut Frame, area: Rect, title: &str) {
    let title_widget = Paragraph::new(title)
        .style(create_title_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(255, 165, 0))) // Orange color
                .padding(Padding::uniform(1))
        );
    f.render_widget(title_widget, area);
}

/// Renders a help text widget with colored keybindings
///
/// Help text is displayed at the bottom of most screens.
/// Keybindings are highlighted in yellow for better visual clarity.
/// Format: "key: description | key: description"
///
/// # Arguments
/// * `f` - The frame to render to
/// * `area` - The area to render in
/// * `help_text` - The help text to display
/// * `show_border` - Whether to show a border around the help text
/// * `centered` - Whether to center the text horizontally
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

/// Formats an input string to show the cursor position
///
/// When the input is empty, this returns a single space so the cursor
/// is visible. This is a common pattern in terminal UIs.
pub fn format_input_with_cursor(input: &str) -> String {
    if input.is_empty() {
        " ".to_string() // Show space for cursor when empty
    } else {
        input.to_string()
    }
}

/// Creates a centered rectangle using a percentage of the available area
///
/// This function is used to create modal/popup dialogs that appear centered
/// on the screen and take up only a portion of the available space.
///
/// # Arguments
/// * `area` - The full terminal area
/// * `percent_x` - Percentage of width to use (0-100)
/// * `percent_y` - Percentage of height to use (0-100)
///
/// # Example
/// ```
/// let popup_area = centered_rect(frame.area(), 50, 30); // 50% width, 30% height
/// ```
pub fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}
