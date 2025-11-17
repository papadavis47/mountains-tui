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
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::models::{AppState, DailyLog, FocusedSection, MeasurementField, RunningField};
use crate::ui::components::*;

/// Renders the home screen showing all available daily logs
///
/// The home screen displays:
/// - Application title with sync status
/// - List of existing daily logs
/// - Help text with keyboard shortcuts
///
/// The `&mut` parameter for list_state allows the function to modify
/// which item is currently selected in the list.
pub fn render_home_screen(
    f: &mut Frame,
    state: &AppState,
    list_state: &mut ListState,
    sync_status: &str,
) {
    // Create the standard three-section layout
    let chunks = create_standard_layout(f.area());

    // Render title with sync status
    let title = format!(
        "Mountains - A Trail Running Training Logger {}",
        sync_status
    );
    render_title(f, chunks[0], &title);

    // Create the list of daily logs
    // The items vector holds each list item to be displayed
    let items: Vec<ListItem> = if state.daily_logs.is_empty() {
        // Show helpful message when no logs exist yet
        vec![ListItem::new(
            "No training logs yet. Press Enter to create one for today.",
        )]
    } else {
        // Map each daily log to a list item showing date and count
        // The `map` iterator adapter transforms each log into a ListItem
        state
            .daily_logs
            .iter()
            .map(|log| {
                let date_str = log.date.format("%B %d, %Y").to_string();
                ListItem::new(date_str)
            })
            .collect() // Collect the iterator results into a Vec
    };

    // Create the List widget with styling
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Daily Training Logs")
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
        " ↑/k: up | ↓/j: down | Enter: select/today | Esc: unfocus | D: delete day | q: quit ",
    );
}

/// Renders the daily view screen for a specific date
///
/// This screen shows:
/// - Date title with sync status
/// - Measurements (weight and waist)
/// - Running activity (miles and elevation)
/// - List of food entries
/// - Sokay entries (unhealthy choices tracker)
/// - Strength & mobility exercises
/// - Daily notes
/// - Help text with available actions
pub fn render_daily_view_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
) {
    // Create layout with all sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // Title (increased for vertical padding)
            Constraint::Length(3), // Measurements (Weight, Waist)
            Constraint::Length(3), // Running (Miles, Elevation)
            Constraint::Min(4),    // Food list (scrollable)
            Constraint::Min(4),    // Sokay list (scrollable, same size as food)
            Constraint::Length(4), // Strength & Mobility section
            Constraint::Length(4), // Notes section
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Render title with the selected date and sync status
    let title = format!(
        "Mountains Training Log - {} {}",
        state.selected_date.format("%B %d, %Y"),
        sync_status
    );
    render_title(f, chunks[0], &title);

    // Render measurements section (Weight, Waist)
    render_measurements_section(f, chunks[1], state.selected_date, &state.daily_logs, &state.focused_section);

    // Render running section (Miles, Elevation)
    render_running_section(f, chunks[2], state.selected_date, &state.daily_logs, &state.focused_section);

    // Render food items list
    render_food_list_section(
        f,
        chunks[3],
        state.selected_date,
        &state.daily_logs,
        food_list_state,
        &state.focused_section,
    );

    // Render sokay section (cumulative count + entries)
    render_sokay_section(f, chunks[4], state.selected_date, &state.daily_logs, sokay_list_state, &state.focused_section);

    // Render strength & mobility section
    render_strength_mobility_section(f, chunks[5], state.selected_date, &state.daily_logs, &state.focused_section);

    // Render notes section
    render_notes_section(f, chunks[6], state.selected_date, &state.daily_logs, &state.focused_section);

    // Render help text with all available actions
    render_help(
        f,
        chunks[7],
        " Shift+J/K: section | Tab: field | Enter: edit | j/k: list | f: food | c: sokay | t: training | n: notes | Esc: back ",
    );
}

/// Renders the measurements display section
///
/// Shows current weight and waist measurements for the selected date.
/// If no measurements are recorded, shows "Not set" placeholders.
/// Highlights the focused field with a ► indicator when this section has focus.
fn render_measurements_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Determine if this section has focus and which field is focused
    let (has_focus, focused_field) = match focused_section {
        FocusedSection::Measurements { focused_field } => (true, Some(focused_field)),
        _ => (false, None),
    };

    // Format the measurements text with focus indicators
    let measurements_text = if let Some(log) = log {
        let weight_str = if let Some(weight) = log.weight {
            format!("Weight: {} lbs", weight)
        } else {
            "Weight: Not set".to_string()
        };
        let waist_str = if let Some(waist) = log.waist {
            format!("Waist Size: {} in", waist)
        } else {
            "Waist Size: Not set".to_string()
        };

        // Add focus indicator (►) to the focused field
        let weight_display = if matches!(focused_field, Some(MeasurementField::Weight)) {
            format!("► {}", weight_str)
        } else {
            weight_str
        };
        let waist_display = if matches!(focused_field, Some(MeasurementField::Waist)) {
            format!("► {}", waist_str)
        } else {
            waist_str
        };

        format!("{} | {}", weight_display, waist_display)
    } else {
        let weight_str = "Weight: Not set".to_string();
        let waist_str = "Waist Size: Not set".to_string();

        let weight_display = if matches!(focused_field, Some(MeasurementField::Weight)) {
            format!("► {}", weight_str)
        } else {
            weight_str
        };
        let waist_display = if matches!(focused_field, Some(MeasurementField::Waist)) {
            format!("► {}", waist_str)
        } else {
            waist_str
        };

        format!("{} | {}", weight_display, waist_display)
    };

    // Determine border style based on focus
    let border_style = if has_focus {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Create and render the measurements widget
    let measurements_widget = Paragraph::new(measurements_text)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("Measurements")
                .padding(ratatui::widgets::Padding::horizontal(1)),
        );
    f.render_widget(measurements_widget, area);
}

/// Renders the running activity display section
///
/// Shows current miles covered and elevation gain for the selected date.
/// If no running data is recorded, shows "Not set" placeholders.
/// Highlights the focused field with a ► indicator when this section has focus.
fn render_running_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Determine if this section has focus and which field is focused
    let (has_focus, focused_field) = match focused_section {
        FocusedSection::Running { focused_field } => (true, Some(focused_field)),
        _ => (false, None),
    };

    // Format the running text with focus indicators
    let running_text = if let Some(log) = log {
        let miles_str = if let Some(miles) = log.miles_covered {
            format!("Miles: {} mi", miles)
        } else {
            "Miles: Not set".to_string()
        };
        let elevation_str = if let Some(elevation) = log.elevation_gain {
            format!("Elevation: {} ft", elevation)
        } else {
            "Elevation: Not set".to_string()
        };

        // Add focus indicator (►) to the focused field
        let miles_display = if matches!(focused_field, Some(RunningField::Miles)) {
            format!("► {}", miles_str)
        } else {
            miles_str
        };
        let elevation_display = if matches!(focused_field, Some(RunningField::Elevation)) {
            format!("► {}", elevation_str)
        } else {
            elevation_str
        };

        format!("{} | {}", miles_display, elevation_display)
    } else {
        let miles_str = "Miles: Not set".to_string();
        let elevation_str = "Elevation: Not set".to_string();

        let miles_display = if matches!(focused_field, Some(RunningField::Miles)) {
            format!("► {}", miles_str)
        } else {
            miles_str
        };
        let elevation_display = if matches!(focused_field, Some(RunningField::Elevation)) {
            format!("► {}", elevation_str)
        } else {
            elevation_str
        };

        format!("{} | {}", miles_display, elevation_display)
    };

    // Determine border style based on focus
    let border_style = if has_focus {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Create and render the running widget
    let running_widget = Paragraph::new(running_text)
        .style(Style::default().fg(Color::LightRed))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("Running")
                .padding(ratatui::widgets::Padding::horizontal(1)),
        );
    f.render_widget(running_widget, area);
}

/// Renders the food items list section
///
/// Shows all food entries for the selected date, or a helpful message
/// if no entries exist yet.
/// The list is visually distinct when it has focus (indicated by focused_section parameter).
fn render_food_list_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    food_list_state: &mut ListState,
    focused_section: &FocusedSection,
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Create the list items
    let items: Vec<ListItem> = if let Some(log) = log {
        if log.food_entries.is_empty() {
            vec![ListItem::new("No food entries yet. Press 'f' to add one.")]
        } else {
            // Format each entry with a bullet point
            log.food_entries
                .iter()
                .map(|entry| {
                    let display = format!("- {}", entry.name);
                    ListItem::new(display)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("No food entries yet. Press 'f' to add one.")]
    };

    // Determine the border style based on focus
    let border_style = if matches!(focused_section, FocusedSection::FoodItems) {
        Style::default().fg(Color::Yellow) // Bright yellow when focused
    } else {
        Style::default().fg(Color::DarkGray) // Dimmed when not focused
    };

    // Determine highlight style based on focus
    let highlight_style = if matches!(focused_section, FocusedSection::FoodItems) {
        create_highlight_style() // Show highlight when focused
    } else {
        Style::default() // No highlight when unfocused
    };

    // Create and render the food list
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("Food Items")
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .highlight_style(highlight_style);
    f.render_stateful_widget(list, area, food_list_state);
}

/// Renders the sokay display section
///
/// Shows cumulative sokay count in the title and a scrollable list of sokay entries.
/// Sokay entries track unhealthy food choices for accountability.
/// The list is visually distinct when it has focus (indicated by focused_section parameter).
fn render_sokay_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    sokay_list_state: &mut ListState,
    focused_section: &FocusedSection,
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Calculate cumulative sokay count up to selected date
    let cumulative_sokay = crate::events::handlers::ActionHandler::calculate_cumulative_sokay(
        &crate::models::AppState {
            current_screen: crate::models::AppScreen::DailyView,
            selected_date,
            daily_logs: daily_logs.to_vec(),
            focused_section: FocusedSection::FoodItems,
        },
        selected_date,
    );

    // Create the title with cumulative count
    let title = format!("Sokay (Total: {})", cumulative_sokay);

    // Create the list items
    let items: Vec<ListItem> = if let Some(log) = log {
        if log.sokay_entries.is_empty() {
            vec![ListItem::new("No sokay entries yet. Press 'c' to add one.")]
        } else {
            // Format each entry with a bullet point
            log.sokay_entries
                .iter()
                .map(|entry| {
                    let display = format!("- {}", entry);
                    ListItem::new(display)
                })
                .collect()
        }
    } else {
        vec![ListItem::new("No sokay entries yet. Press 'c' to add one.")]
    };

    // Determine the border style based on focus
    let border_style = if matches!(focused_section, FocusedSection::Sokay) {
        Style::default().fg(Color::Magenta) // Bright magenta when focused
    } else {
        Style::default().fg(Color::DarkGray) // Dimmed when not focused
    };

    // Determine highlight style based on focus
    let highlight_style = if matches!(focused_section, FocusedSection::Sokay) {
        create_highlight_style() // Show highlight when focused
    } else {
        Style::default() // No highlight when unfocused
    };

    // Create and render the sokay list
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title)
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .highlight_style(highlight_style);
    f.render_stateful_widget(list, area, sokay_list_state);
}

/// Renders the strength & mobility display section
///
/// Shows current strength and mobility exercises for the selected date, or a message indicating
/// that no exercises have been recorded yet.
/// The section is visually distinct when it has focus (bright border).
fn render_strength_mobility_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Determine if this section has focus
    let has_focus = matches!(focused_section, FocusedSection::StrengthMobility);

    // Format the strength & mobility text
    let sm_text = if let Some(log) = log {
        if let Some(sm) = &log.strength_mobility {
            if sm.trim().is_empty() {
                "No exercises recorded yet. Press 't' to add training info.".to_string()
            } else {
                sm.clone()
            }
        } else {
            "No exercises recorded yet. Press 't' to add training info.".to_string()
        }
    } else {
        "No exercises recorded yet. Press 't' to add training info.".to_string()
    };

    // Determine border style based on focus
    let border_style = if has_focus {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Create and render the strength & mobility widget
    let sm_widget = Paragraph::new(sm_text)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("Strength & Mobility")
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(sm_widget, area);
}

/// Renders the notes display section
///
/// Shows current daily notes for the selected date, or a message indicating
/// that no notes have been written yet.
/// The section is visually distinct when it has focus (bright border).
fn render_notes_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
) {
    // Find the log for the selected date
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // Determine if this section has focus
    let has_focus = matches!(focused_section, FocusedSection::Notes);

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

    // Determine border style based on focus
    let border_style = if has_focus {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Create and render the notes widget
    let notes_widget = Paragraph::new(notes_text)
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title("Notes")
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(notes_widget, area);
}

/// Renders the add food entry screen as a centered modal dialog
///
/// This screen allows users to input a new food item name.
/// It overlays the daily view with a small centered dialog box.
pub fn render_add_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (50% width, 25% height)
    let popup_area = centered_rect(f.area(), 50, 25);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Add Food - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the edit food entry screen as a centered modal dialog
///
/// Similar to add food screen but for editing existing entries.
/// It overlays the daily view with a small centered dialog box.
pub fn render_edit_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (50% width, 25% height)
    let popup_area = centered_rect(f.area(), 50, 25);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Food - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the edit weight screen as a centered modal dialog
///
/// Allows users to input their weight in pounds (numeric input only).
/// It overlays the daily view with a small centered dialog box.
pub fn render_edit_weight_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (30% width, 15% height for numeric input)
    let popup_area = centered_rect(f.area(), 30, 15);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Weight - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the edit waist measurement screen as a centered modal dialog
///
/// Allows users to input their waist size in inches (numeric input only).
/// It overlays the daily view with a small centered dialog box.
pub fn render_edit_waist_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (30% width, 15% height for numeric input)
    let popup_area = centered_rect(f.area(), 30, 15);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Waist Size - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the edit strength & mobility screen as a centered modal dialog
///
/// Allows users to write multi-line text about their strength and mobility exercises.
/// It overlays the daily view with a larger centered dialog box for multi-line editing.
pub fn render_edit_strength_mobility_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (60% width, 40% height for multi-line input)
    let popup_area = centered_rect(f.area(), 60, 40);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!(
        "Edit Strength & Mobility - {}",
        state.selected_date.format("%B %d, %Y")
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Cyan))
        .padding(ratatui::widgets::Padding::uniform(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Manually wrap text at character boundaries to match our cursor calculation
    let display_text = if input_buffer.is_empty() {
        " ".to_string()
    } else {
        // Wrap at exact character width to ensure cursor positioning matches display
        wrap_text_at_chars(input_buffer, inner_area.width as usize)
    };

    let input = Paragraph::new(display_text.clone())
        .style(create_input_style());
        // NO .wrap() - we've already wrapped the text manually
    f.render_widget(input, inner_area);

    // Calculate cursor position using the SAME wrapped text that we're displaying
    // This is critical - cursor calc must match the displayed text exactly
    let (cursor_x, cursor_y) =
        calculate_multiline_cursor_position(inner_area, &display_text, cursor_position);
    f.set_cursor_position((cursor_x, cursor_y));
}

/// Renders the edit notes screen as a centered modal dialog
///
/// Allows users to write multi-paragraph notes about their day.
/// It overlays the daily view with a larger centered dialog box for multi-line editing.
pub fn render_edit_notes_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (60% width, 40% height for multi-line input)
    let popup_area = centered_rect(f.area(), 60, 40);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Notes - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Green))
        .padding(ratatui::widgets::Padding::uniform(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Manually wrap text at character boundaries to match our cursor calculation
    let display_text = if input_buffer.is_empty() {
        " ".to_string()
    } else {
        // Wrap at exact character width to ensure cursor positioning matches display
        wrap_text_at_chars(input_buffer, inner_area.width as usize)
    };

    let input = Paragraph::new(display_text.clone())
        .style(create_input_style());
        // NO .wrap() - we've already wrapped the text manually
    f.render_widget(input, inner_area);

    // Calculate cursor position using the SAME wrapped text that we're displaying
    // This is critical - cursor calc must match the displayed text exactly
    let (cursor_x, cursor_y) =
        calculate_multiline_cursor_position(inner_area, &display_text, cursor_position);
    f.set_cursor_position((cursor_x, cursor_y));
}

/// Manually wraps text at character boundaries (not word boundaries)
///
/// This function takes text and wraps it at exactly `width` characters per line,
/// preserving explicit newlines. This allows us to have predictable cursor positioning.
fn wrap_text_at_chars(text: &str, width: usize) -> String {
    if width == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    let mut current_line_len = 0;

    for ch in text.chars() {
        if ch == '\n' {
            // Explicit newline - preserve it
            result.push('\n');
            current_line_len = 0;
        } else {
            // Check if we need to wrap before adding this character
            if current_line_len >= width {
                result.push('\n');
                current_line_len = 0;
            }
            result.push(ch);
            current_line_len += 1;
        }
    }

    result
}

/// Calculates cursor position for multi-line text input with character-based wrapping
///
/// This function works with our manual character-based wrapping (wrap_text_at_chars).
/// Since we wrap at exact character boundaries, cursor calculation is straightforward:
/// just count characters and newlines up to the cursor position.
///
/// IMPORTANT: cursor_pos is a BYTE index into the UTF-8 string, not a character count!
fn calculate_multiline_cursor_position(
    area: ratatui::layout::Rect,
    text: &str,
    cursor_pos_bytes: usize,
) -> (u16, u16) {
    // Convert byte index to character count
    let cursor_pos_chars = if cursor_pos_bytes <= text.len() {
        text[..cursor_pos_bytes].chars().count()
    } else {
        text.chars().count()
    };

    // Simply count newlines and column position up to cursor
    // No complex wrapping logic needed since text is already wrapped
    let mut line = 0;
    let mut col = 0;
    let mut char_count = 0;

    for ch in text.chars() {
        if char_count >= cursor_pos_chars {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        char_count += 1;
    }

    let cursor_x = area.x + col as u16;
    let cursor_y = area.y + line as u16;

    (cursor_x, cursor_y)
}

/// Renders the edit miles screen as a centered modal dialog
///
/// Allows users to input miles covered (numeric input with decimal).
/// It overlays the daily view with a small centered dialog box.
pub fn render_edit_miles_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (30% width, 15% height for numeric input)
    let popup_area = centered_rect(f.area(), 30, 15);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Miles - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::LightRed))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the edit elevation screen as a centered modal dialog
///
/// Allows users to input elevation gain in feet (integer input only).
/// It overlays the daily view with a small centered dialog box.
pub fn render_edit_elevation_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (30% width, 15% height for numeric input)
    let popup_area = centered_rect(f.area(), 30, 15);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Elevation - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::LightRed))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the add sokay screen as a centered modal dialog
///
/// Allows users to add a new sokay entry (text input).
/// It overlays the daily view with a small centered dialog box.
pub fn render_add_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (50% width, 25% height)
    let popup_area = centered_rect(f.area(), 50, 25);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Add Sokay Entry - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Magenta))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the edit sokay screen as a centered modal dialog
///
/// Allows users to edit an existing sokay entry (text input).
/// It overlays the daily view with a small centered dialog box.
pub fn render_edit_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Create centered popup area (50% width, 25% height)
    let popup_area = centered_rect(f.area(), 50, 25);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Sokay Entry - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Magenta))
        .padding(ratatui::widgets::Padding::horizontal(1));

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text)
        .style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((
        inner_area.x + cursor_position as u16,
        inner_area.y,
    ));
}

/// Renders the delete day confirmation screen
///
/// This screen asks the user to confirm deletion of an entire day's log.
/// Shows a warning message and waits for Y/n input.
pub fn render_confirm_delete_day_screen(f: &mut Frame, selected_date: NaiveDate) {
    let chunks = create_standard_layout(f.area());

    let title = "Delete Day - Confirmation Required";
    render_title(f, chunks[0], title);

    // Create the warning message
    let warning_text = format!(
        "Are you sure you want to delete the entire log for {}?\n\n\
        This will permanently delete:\n\
        - All food entries\n\
        - All sokay entries\n\
        - All measurements (weight, waist size, miles, elevation)\n\
        - Strength & mobility exercises\n\
        - Daily notes\n\n\
        This action cannot be undone.\n\n\
        Type 'Y' to confirm deletion or 'n' to cancel.",
        selected_date.format("%B %d, %Y")
    );

    let warning_widget = Paragraph::new(warning_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title("Warning: Permanent Deletion"),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(warning_widget, chunks[1]);

    render_help(f, chunks[2], "Y: delete day | n/Esc: cancel");
}
