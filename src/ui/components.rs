/// Reusable UI components and utilities for the Mountains Food Tracker
///
/// This module contains helper functions and reusable components that are used
/// across multiple screens. This follows the DRY (Don't Repeat Yourself) principle.
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

/// Creates the standard title styling used throughout the application
///
/// This function demonstrates Rust's approach to creating reusable components.
/// Instead of duplicating styling code, we centralize it in one place.
pub fn create_title_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

/// Creates the standard help text styling used throughout the application
pub fn create_help_style() -> Style {
    Style::default().fg(Color::Gray)
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
/// - Top: Fixed height for titles (3 lines)
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
            Constraint::Length(3), // Title area
            Constraint::Min(0),    // Content area (takes remaining space)
            Constraint::Length(3), // Help area
        ])
        .split(area)
}

/// Creates a four-section layout used by screens with measurements
///
/// Used by the daily view screen which needs:
/// - Title
/// - Measurements display
/// - Main content
/// - Help text
pub fn create_daily_view_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(4), // Measurements
            Constraint::Min(0),    // Food list
            Constraint::Length(3), // Help
        ])
        .split(area)
}

/// Renders a title widget with the application's standard styling
///
/// This function takes a title string and renders it in a bordered box
/// with the standard cyan/bold styling.
pub fn render_title(f: &mut Frame, area: Rect, title: &str) {
    let title_widget = Paragraph::new(title)
        .style(create_title_style())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_widget, area);
}

/// Renders a help text widget with the application's standard styling
///
/// Help text is displayed in gray at the bottom of most screens.
pub fn render_help(f: &mut Frame, area: Rect, help_text: &str) {
    let help_widget = Paragraph::new(help_text)
        .style(create_help_style())
        .block(Block::default().borders(Borders::ALL));
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

/// Calculates the terminal cursor position for an input field
///
/// Returns the (x, y) coordinates where the terminal cursor should be placed.
/// Takes into account the widget's border (hence the +1 offsets).
pub fn calculate_cursor_position(input_area: Rect, cursor_pos: usize) -> (u16, u16) {
    let cursor_x = input_area.x + 1 + cursor_pos as u16; // +1 for border
    let cursor_y = input_area.y + 1; // +1 for border
    (cursor_x, cursor_y)
}
