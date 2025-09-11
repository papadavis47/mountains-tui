/// Screen rendering logic for the Mountains Food Tracker
///
/// This module contains all the UI rendering functions for different screens.
/// Each screen is responsible for drawing its own interface using ratatui widgets.
///
/// The separation of UI logic into this module makes the code more maintainable
/// and follows the single responsibility principle.
use chrono::NaiveDate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::models::{AppState, DailyLog};
use crate::ui::components::*;

/// Renders the home screen showing all available daily logs
///
/// The home screen displays:
/// - Application title
/// - List of existing daily logs with food entry counts
/// - Help text with keyboard shortcuts
///
/// The `&mut` parameter for list_state allows the function to modify
/// which item is currently selected in the list.
pub fn render_home_screen(f: &mut Frame, state: &AppState, list_state: &mut ListState) {
    // Create the standard three-section layout
    let chunks = create_standard_layout(f.area());

    // Render title
    render_title(
        f,
        chunks[0],
        "Mountains - A Food Tracker for Power to Weight Improvement",
    );

    // Create the list of daily logs
    // The items vector holds each list item to be displayed
    let items: Vec<ListItem> = if state.daily_logs.is_empty() {
        // Show helpful message when no logs exist yet
        vec![ListItem::new(
            "No food logs yet. Press Enter to create one for today.",
        )]
    } else {
        // Map each daily log to a list item showing date and food count
        // The `map` iterator adapter transforms each log into a ListItem
        state
            .daily_logs
            .iter()
            .map(|log| {
                let date_str = log.date.format("%B %d, %Y").to_string();
                let food_count = log.food_entries.len();
                let summary = format!("{} ({} food items)", date_str, food_count);
                ListItem::new(summary)
            })
            .collect() // Collect the iterator results into a Vec
    };

    // Create the List widget with styling
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Mountains Food Log Days")
                // Horizontal padding moves text away from the borders
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .highlight_style(create_highlight_style());

    // Render the list with its state (tracks which item is selected)
    f.render_stateful_widget(list, chunks[1], list_state);

    // Render help text
    render_help(
        f,
        chunks[2],
        "q: quit | ↑/k: up | ↓/j: down | Enter: select/create",
    );
}

/// Renders the daily view screen for a specific date
///
/// This screen shows:
/// - Date title
/// - Measurements (weight and waist)
/// - List of food entries
/// - Help text with available actions
pub fn render_daily_view_screen(f: &mut Frame, state: &AppState, food_list_state: &mut ListState) {
    // Create a more complex layout that includes space for notes
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(4), // Measurements
            Constraint::Min(6),    // Food list (reduced minimum)
            Constraint::Length(4), // Notes section
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Render title with the selected date
    let title = format!(
        "Mountains Food Log - {}",
        state.selected_date.format("%B %d, %Y")
    );
    render_title(f, chunks[0], &title);

    // Render measurements section
    render_measurements_section(f, chunks[1], state.selected_date, &state.daily_logs);

    // Render food items list
    render_food_list_section(
        f,
        chunks[2],
        state.selected_date,
        &state.daily_logs,
        food_list_state,
    );

    // Render notes section
    render_notes_section(f, chunks[3], state.selected_date, &state.daily_logs);

    // Render help text with all available actions (including notes)
    render_help(
        f,
        chunks[4],
        "q: quit | a: add | e: edit | d: delete | w: weight | s: waist | n: notes | ↑/↓: navigate | Esc: back",
    );
}

/// Renders the measurements display section
///
/// Shows current weight and waist measurements for the selected date.
/// If no measurements are recorded, shows "Not set" placeholders.
fn render_measurements_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Format the measurements text
    let measurements_text = if let Some(log) = log {
        let weight_str = if let Some(weight) = log.weight {
            format!("Weight: {} lbs", weight)
        } else {
            "Weight: Not set".to_string()
        };
        let waist_str = if let Some(waist) = log.waist {
            format!("Waist: {} inches", waist)
        } else {
            "Waist: Not set".to_string()
        };
        format!("{} | {}", weight_str, waist_str)
    } else {
        "Weight: Not set | Waist: Not set".to_string()
    };

    // Create and render the measurements widget
    let measurements_widget = Paragraph::new(measurements_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Measurements"));
    f.render_widget(measurements_widget, area);
}

/// Renders the food items list section
///
/// Shows all food entries for the selected date, or a helpful message
/// if no entries exist yet.
fn render_food_list_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    food_list_state: &mut ListState,
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Create the list items
    let items: Vec<ListItem> = if let Some(log) = log {
        if log.food_entries.is_empty() {
            vec![ListItem::new("No food entries yet. Press 'a' to add one.")]
        } else {
            // Enumerate gives us both the index and the item
            log.food_entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    // Format each entry with its number and optional notes
                    let display = if let Some(notes) = &entry.notes {
                        format!("{}. {} - {}", i + 1, entry.name, notes)
                    } else {
                        format!("{}. {}", i + 1, entry.name)
                    };
                    ListItem::new(display)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("No food entries yet. Press 'a' to add one.")]
    };

    // Create and render the food list
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Food Items")
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .highlight_style(create_highlight_style());
    f.render_stateful_widget(list, area, food_list_state);
}

/// Renders the notes display section
///
/// Shows current daily notes for the selected date, or a message indicating
/// that no notes have been written yet.
fn render_notes_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Format the notes text
    let notes_text = if let Some(log) = log {
        if let Some(notes) = &log.notes {
            if notes.trim().is_empty() {
                "No notes for this day yet. Press 'n' to add notes.".to_string()
            } else {
                notes.clone()
            }
        } else {
            "No notes for this day yet. Press 'n' to add notes.".to_string()
        }
    } else {
        "No notes for this day yet. Press 'n' to add notes.".to_string()
    };

    // Create and render the notes widget
    let notes_widget = Paragraph::new(notes_text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Daily Notes"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(notes_widget, area);
}

/// Renders the add food entry screen
///
/// This screen allows users to input a new food item name.
/// It includes a text input field with cursor support.
pub fn render_add_food_screen(
    f: &mut Frame,
    selected_date: NaiveDate,
    input_buffer: &str,
    cursor_position: usize,
) {
    let chunks = create_standard_layout(f.area());

    // Render title
    let title = format!("Add Food - {}", selected_date.format("%B %d, %Y"));
    render_title(f, chunks[0], &title);

    // Render input field with cursor positioning
    render_input_field(f, chunks[1], "Food Name", input_buffer, cursor_position);

    // Render help text
    render_help(
        f,
        chunks[2],
        "Type food name and press Enter to save | Esc: cancel",
    );
}

/// Renders the edit food entry screen
///
/// Similar to add food screen but for editing existing entries.
pub fn render_edit_food_screen(
    f: &mut Frame,
    selected_date: NaiveDate,
    input_buffer: &str,
    cursor_position: usize,
) {
    let chunks = create_standard_layout(f.area());

    let title = format!("Edit Food - {}", selected_date.format("%B %d, %Y"));
    render_title(f, chunks[0], &title);

    render_input_field(f, chunks[1], "Food Name", input_buffer, cursor_position);

    render_help(
        f,
        chunks[2],
        "Edit food name and press Enter to save | Esc: cancel",
    );
}

/// Renders the edit weight screen
///
/// Allows users to input their weight in pounds (numeric input only).
pub fn render_edit_weight_screen(
    f: &mut Frame,
    selected_date: NaiveDate,
    input_buffer: &str,
    cursor_position: usize,
) {
    let chunks = create_standard_layout(f.area());

    let title = format!("Edit Weight - {}", selected_date.format("%B %d, %Y"));
    render_title(f, chunks[0], &title);

    render_input_field(f, chunks[1], "Weight (lbs)", input_buffer, cursor_position);

    render_help(
        f,
        chunks[2],
        "Enter weight in lbs (numbers and decimal only) | Enter: save | Esc: cancel",
    );
}

/// Renders the edit waist measurement screen
///
/// Allows users to input their waist size in inches (numeric input only).
pub fn render_edit_waist_screen(
    f: &mut Frame,
    selected_date: NaiveDate,
    input_buffer: &str,
    cursor_position: usize,
) {
    let chunks = create_standard_layout(f.area());

    let title = format!("Edit Waist - {}", selected_date.format("%B %d, %Y"));
    render_title(f, chunks[0], &title);

    render_input_field(
        f,
        chunks[1],
        "Waist (inches)",
        input_buffer,
        cursor_position,
    );

    render_help(
        f,
        chunks[2],
        "Enter waist size in inches (numbers and decimal only) | Enter: save | Esc: cancel",
    );
}

/// Helper function to render an input field with cursor positioning
///
/// This function handles the common pattern of:
/// 1. Rendering a text input field
/// 2. Setting the terminal cursor position for text editing
///
/// The cursor positioning is crucial for a good user experience in terminal UIs.
fn render_input_field(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    title: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // Format the input text (shows space if empty for cursor visibility)
    let input_with_cursor = format_input_with_cursor(input_buffer);

    // Create and render the input widget
    let input = Paragraph::new(input_with_cursor)
        .style(create_input_style())
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(input, area);

    // Set the terminal cursor position
    let (cursor_x, cursor_y) = calculate_cursor_position(area, cursor_position);
    f.set_cursor_position((cursor_x, cursor_y));
}

/// Renders the edit notes screen
///
/// Allows users to write multi-paragraph notes about their day.
/// This screen provides a larger text area for longer form writing.
pub fn render_edit_notes_screen(
    f: &mut Frame,
    selected_date: NaiveDate,
    input_buffer: &str,
    cursor_position: usize,
) {
    // Create a layout with more space for the notes area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(8),    // Notes area (larger than normal input)
            Constraint::Length(4), // Help text (slightly larger for multi-line help)
        ])
        .split(f.area());

    let title = format!("Edit Notes - {}", selected_date.format("%B %d, %Y"));
    render_title(f, chunks[0], &title);

    // Render a larger text area for notes
    render_multiline_input_field(f, chunks[1], "Daily Notes", input_buffer, cursor_position);

    // Provide more detailed help for notes editing
    let help_text = "Write your thoughts, feelings, or observations about the day\n\
                     Ctrl+J: New line | Enter: Save | Esc: Cancel\n\
                     Use arrow keys to navigate, Home/End to jump";

    let help_widget = Paragraph::new(help_text)
        .style(create_help_style())
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(help_widget, chunks[2]);
}

/// Helper function to render a multi-line input field for notes
///
/// This function creates a larger text area suitable for multi-paragraph input.
/// It handles text wrapping and cursor positioning for longer text.
fn render_multiline_input_field(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    title: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // For multi-line input, we need to handle text wrapping
    let display_text = if input_buffer.is_empty() {
        " ".to_string() // Show space for cursor when empty
    } else {
        input_buffer.to_string()
    };

    // Create and render the input widget with text wrapping
    let input = Paragraph::new(display_text)
        .style(create_input_style())
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(ratatui::widgets::Wrap { trim: false }); // Don't trim for notes
    f.render_widget(input, area);

    // Calculate cursor position for multi-line text
    let (cursor_x, cursor_y) =
        calculate_multiline_cursor_position(area, input_buffer, cursor_position);
    f.set_cursor_position((cursor_x, cursor_y));
}

/// Calculates cursor position for multi-line text input
///
/// This function handles cursor positioning in multi-line text by:
/// 1. Counting newlines up to the cursor position
/// 2. Finding the column position on the current line
/// 3. Accounting for text wrapping within the widget bounds
fn calculate_multiline_cursor_position(
    area: ratatui::layout::Rect,
    text: &str,
    cursor_pos: usize,
) -> (u16, u16) {
    let widget_width = area.width.saturating_sub(2) as usize; // Account for borders
    let text_up_to_cursor = if cursor_pos <= text.len() {
        &text[..cursor_pos]
    } else {
        text
    };

    // Count actual newlines and calculate position
    let mut line = 0;
    let mut col = 0;

    for ch in text_up_to_cursor.chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
            // Handle text wrapping
            if col >= widget_width {
                line += 1;
                col = 0;
            }
        }
    }

    // Convert to terminal coordinates (accounting for borders)
    let cursor_x = area.x + 1 + col as u16;
    let cursor_y = area.y + 1 + line as u16;

    (cursor_x, cursor_y)
}
