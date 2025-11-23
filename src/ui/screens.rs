/// This module contains all the UI rendering functions for different screens.
/// Each screen is responsible for drawing its own interface using ratatui widgets.
///
/// The separation of UI logic into this module makes the code more maintainable
/// and follows the single responsibility principle.
use chrono::NaiveDate;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::assets::APP_TITLE;
use crate::elevation_stats::{
    calculate_yearly_elevation, count_monthly_1000_days, get_streak_message,
};
use crate::models::{AppState, DailyLog, FocusedSection, MeasurementField, RunningField};
use crate::ui::components::*;

/// Renders the startup screen with ASCII art and elevation statistics
///
/// This screen displays:
/// - ASCII art logo in the middle third (centered horizontally and vertically)
/// - Subtitle: "For Inspiration and Mindfulness"
/// - Monthly count of days with 1000+ feet elevation
/// - Yearly total elevation gain
/// - Current consecutive days streak of 1000+ feet elevation
/// - Help text: N: Today's Log | L: Log List | q: Quit
pub fn render_startup_screen(f: &mut Frame, state: &AppState) {
    let chunks = create_standard_layout(f.area());

    // Calculate statistics
    let monthly_count = count_monthly_1000_days(&state.daily_logs);
    let yearly_total = calculate_yearly_elevation(&state.daily_logs);
    let streak_message = get_streak_message(&state.daily_logs);

    // Get current month name and year
    let now = chrono::Local::now().date_naive();
    let month_name = now.format("%B").to_string();
    let year = now.format("%Y").to_string();

    // Create the content with ASCII art and statistics
    let mut content_lines = Vec::new();

    // Add top spacing to push content to middle area
    // Use less padding to position content higher (about 1/5 from top instead of 1/3)
    let content_area_height = chunks[1].height;
    let top_padding = content_area_height / 5;
    for _ in 0..top_padding {
        content_lines.push(Line::from(""));
    }

    // Add ASCII art (centered)
    for line in APP_TITLE.lines() {
        content_lines.push(Line::from(Span::styled(
            line,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Add subtitle
    content_lines.push(Line::from(""));
    content_lines.push(Line::from(Span::styled(
        "For Inspiration and Mindfulness",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC),
    )));

    // Add spacing
    content_lines.push(Line::from(""));
    content_lines.push(Line::from(""));

    // Add monthly statistic
    let monthly_text = format!(
        "You have {} days of 1000+ feet vert in the month of {}.",
        monthly_count, month_name
    );
    content_lines.push(Line::from(Span::styled(
        monthly_text,
        Style::default().fg(Color::White),
    )));

    // Add yearly statistic
    content_lines.push(Line::from(""));
    let yearly_text = format!("You have {} feet for {}.", yearly_total, year);
    content_lines.push(Line::from(Span::styled(
        yearly_text,
        Style::default().fg(Color::White),
    )));

    // Add streak message
    content_lines.push(Line::from(""));
    content_lines.push(Line::from(Span::styled(
        streak_message,
        Style::default().fg(Color::Green),
    )));

    // Render the content in the main area (centered)
    let content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::NONE))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(content, chunks[1]);

    // Render help text without border for clean appearance, centered horizontally
    render_help(f, chunks[2], " N: Today's Log | L: Log List | q: Quit ", false, true);
}

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
    let title = format!("Mountains - A Trail Running Training Log {}", sync_status);
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
                // Uniform padding provides space on all sides
                .padding(ratatui::widgets::Padding::uniform(1)),
        )
        .highlight_style(create_highlight_style());

    // Render the list with its state (tracks which item is selected)
    f.render_stateful_widget(list, chunks[1], list_state);

    // Render help text
    render_help(
        f,
        chunks[2],
        " ↑/k: Up | ↓/j: Down | Enter: Select/Today | Esc: Unfocus | D: Delete Day | S: Startup Screen | q: Quit ",
        true,
        false,
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
    render_measurements_section(
        f,
        chunks[1],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
    );

    // Render running section (Miles, Elevation)
    render_running_section(
        f,
        chunks[2],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
    );

    // Render food items list
    render_food_list_section(
        f,
        chunks[3],
        state.selected_date,
        &state.daily_logs,
        food_list_state,
        &state.focused_section,
        state.food_list_focused,
    );

    // Render sokay section (cumulative count + entries)
    render_sokay_section(
        f,
        chunks[4],
        state.selected_date,
        &state.daily_logs,
        sokay_list_state,
        &state.focused_section,
        state.sokay_list_focused,
    );

    // Render strength & mobility section
    render_strength_mobility_section(
        f,
        chunks[5],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
    );

    // Render notes section
    render_notes_section(
        f,
        chunks[6],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
    );

    // Render help text with all available actions
    render_help(
        f,
        chunks[7],
        " Shift+J/K: Section | Tab: Field | Enter: Add | j/k: List | E: Edit Item | D: Delete Item | Space: Shortcuts | S: Startup Screen | Esc: Back ",
        true,
        false,
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
    food_list_focused: bool,
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

    // Determine the border style based on section focus
    let border_style = if matches!(focused_section, FocusedSection::FoodItems) {
        Style::default().fg(Color::Yellow) // Bright yellow when section focused
    } else {
        Style::default().fg(Color::DarkGray) // Dimmed when not focused
    };

    // Determine highlight style based on BOTH section focus AND item focus
    let highlight_style =
        if matches!(focused_section, FocusedSection::FoodItems) && food_list_focused {
            create_highlight_style() // Show highlight when section AND item are focused
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
                .padding(ratatui::widgets::Padding::uniform(1)),
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
    sokay_list_focused: bool,
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
            food_list_focused: false,
            sokay_list_focused: false,
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

    // Determine the border style based on section focus
    let border_style = if matches!(focused_section, FocusedSection::Sokay) {
        Style::default().fg(Color::Magenta) // Bright magenta when section focused
    } else {
        Style::default().fg(Color::DarkGray) // Dimmed when not focused
    };

    // Determine highlight style based on BOTH section focus AND item focus
    let highlight_style = if matches!(focused_section, FocusedSection::Sokay) && sokay_list_focused
    {
        create_highlight_style() // Show highlight when section AND item are focused
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
                .padding(ratatui::widgets::Padding::uniform(1)),
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

    // Create centered popup area (50% width, 13% height - half of previous 25%)
    let popup_area = centered_rect(f.area(), 50, 13);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Add Food - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
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

    // Create centered popup area (50% width, 13% height - half of previous 25%)
    let popup_area = centered_rect(f.area(), 50, 13);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!("Edit Food - {}", state.selected_date.format("%B %d, %Y"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
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

    // Create centered popup area (12% width, 8% height for numeric input)
    let popup_area = centered_rect(f.area(), 12, 8);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Edit Weight")
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
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

    // Create centered popup area (12% width, 8% height for numeric input)
    let popup_area = centered_rect(f.area(), 12, 8);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Edit Waist Size")
        .style(Style::default().fg(Color::Yellow))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
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

    // Manually wrap text at character boundaries for predictable cursor positioning
    let width = inner_area.width as usize;
    let wrapped_text = if input_buffer.is_empty() {
        " ".to_string()
    } else {
        wrap_at_width(input_buffer, width)
    };

    let input = Paragraph::new(wrapped_text.clone()).style(create_input_style());
    // NO .wrap() - text is already wrapped manually
    f.render_widget(input, inner_area);

    // Calculate cursor position on the wrapped text
    let (cursor_x, cursor_y) =
        calculate_cursor_in_wrapped_text(inner_area, input_buffer, cursor_position, width);
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

    // Manually wrap text at character boundaries for predictable cursor positioning
    let width = inner_area.width as usize;
    let wrapped_text = if input_buffer.is_empty() {
        " ".to_string()
    } else {
        wrap_at_width(input_buffer, width)
    };

    let input = Paragraph::new(wrapped_text.clone()).style(create_input_style());
    // NO .wrap() - text is already wrapped manually
    f.render_widget(input, inner_area);

    // Calculate cursor position on the wrapped text
    let (cursor_x, cursor_y) =
        calculate_cursor_in_wrapped_text(inner_area, input_buffer, cursor_position, width);
    f.set_cursor_position((cursor_x, cursor_y));
}

/// Wraps text at word boundaries to fit within a given width
///
/// This function performs word-based wrapping similar to text editors.
/// It preserves explicit newlines and tries to keep words together.
/// If a word is too long to fit on one line, it breaks at character boundaries.
/// This gives us complete control over line wrapping for accurate cursor positioning.
fn wrap_at_width(text: &str, width: usize) -> String {
    if width == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    let mut current_line = String::new();
    let mut current_line_width = 0;

    for word in text.split_inclusive(|c: char| c.is_whitespace() || c == '\n') {
        // Check if this "word" contains a newline
        if word.contains('\n') {
            // Handle newlines - could be "\n" or "word\n"
            let parts: Vec<&str> = word.split('\n').collect();

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    // Add the current line before the newline
                    result.push_str(&current_line);
                    result.push('\n');
                    current_line.clear();
                    current_line_width = 0;
                }

                if !part.is_empty() {
                    let part_width = part.chars().count();

                    if current_line_width + part_width > width && current_line_width > 0 {
                        // Current line would overflow, wrap first
                        result.push_str(&current_line);
                        result.push('\n');
                        current_line.clear();
                        current_line_width = 0;
                    }

                    // Add the part
                    if part_width > width {
                        // Word is too long, break it at character boundaries
                        for ch in part.chars() {
                            if current_line_width >= width {
                                result.push_str(&current_line);
                                result.push('\n');
                                current_line.clear();
                                current_line_width = 0;
                            }
                            current_line.push(ch);
                            current_line_width += 1;
                        }
                    } else {
                        current_line.push_str(part);
                        current_line_width += part_width;
                    }
                }
            }
        } else {
            // No newline, just a regular word (possibly with trailing space)
            let word_width = word.chars().count();

            if current_line_width + word_width > width && current_line_width > 0 {
                // Adding this word would overflow, wrap first
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
                current_line_width = 0;
            }

            // If the word itself is longer than width, break it
            if word_width > width {
                for ch in word.chars() {
                    if current_line_width >= width {
                        result.push_str(&current_line);
                        result.push('\n');
                        current_line.clear();
                        current_line_width = 0;
                    }
                    current_line.push(ch);
                    current_line_width += 1;
                }
            } else {
                current_line.push_str(word);
                current_line_width += word_width;
            }
        }
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        result.push_str(&current_line);
    }

    result
}

/// Calculates cursor position in manually-wrapped text with word wrapping
///
/// This function calculates cursor position by using the EXACT same approach
/// as wrap_at_width() - processing text with split_inclusive to match perfectly.
///
/// IMPORTANT: cursor_pos_bytes is a BYTE index into the UTF-8 string!
fn calculate_cursor_in_wrapped_text(
    area: ratatui::layout::Rect,
    original_text: &str,
    cursor_pos_bytes: usize,
    width: usize,
) -> (u16, u16) {
    if width == 0 {
        return (area.x, area.y);
    }

    // Convert byte index to character count
    let cursor_pos_chars = if cursor_pos_bytes <= original_text.len() {
        original_text[..cursor_pos_bytes].chars().count()
    } else {
        original_text.chars().count()
    };

    let mut line = 0;
    let mut col = 0;
    let mut char_count = 0;
    let mut current_line_width = 0;

    // Use split_inclusive EXACTLY like wrap_at_width does
    for word in original_text.split_inclusive(|c: char| c.is_whitespace() || c == '\n') {
        // Check if we've passed the cursor position before processing this word
        let word_char_count = word.chars().count();

        if char_count + word_char_count > cursor_pos_chars {
            // Cursor is somewhere within this word
            let chars_into_word = cursor_pos_chars - char_count;

            // Process the word up to the cursor position
            if word.contains('\n') {
                // Handle newlines
                let parts: Vec<&str> = word.split('\n').collect();
                let mut chars_processed = 0;

                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        line += 1;
                        col = 0;
                        current_line_width = 0;
                        chars_processed += 1; // The newline character

                        if chars_processed >= chars_into_word {
                            break;
                        }
                    }

                    if !part.is_empty() {
                        let part_width = part.chars().count();
                        let chars_to_take = (chars_into_word - chars_processed).min(part_width);

                        if current_line_width + part_width > width && current_line_width > 0 {
                            line += 1;
                            current_line_width = 0;
                        }

                        if part_width > width {
                            // Long word - break character by character
                            for (idx, _ch) in part.chars().enumerate() {
                                if idx >= chars_to_take {
                                    break;
                                }
                                if current_line_width >= width {
                                    line += 1;
                                    current_line_width = 0;
                                }
                                current_line_width += 1;
                                chars_processed += 1;
                            }
                        } else {
                            current_line_width += chars_to_take;
                            chars_processed += chars_to_take;
                        }

                        col = current_line_width;

                        if chars_processed >= chars_into_word {
                            break;
                        }
                    }
                }
            } else {
                // Regular word (possibly with trailing whitespace)
                let word_width = word.chars().count();

                if current_line_width + word_width > width && current_line_width > 0 {
                    line += 1;
                    current_line_width = 0;
                }

                if word_width > width {
                    // Long word - break it
                    for (idx, _ch) in word.chars().enumerate() {
                        if idx >= chars_into_word {
                            break;
                        }
                        if current_line_width >= width {
                            line += 1;
                            current_line_width = 0;
                        }
                        current_line_width += 1;
                    }
                } else {
                    current_line_width += chars_into_word;
                }

                col = current_line_width;
            }

            break;
        }

        // Process complete word (cursor is after this word)
        char_count += word_char_count;

        if word.contains('\n') {
            let parts: Vec<&str> = word.split('\n').collect();

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    line += 1;
                    current_line_width = 0;
                }

                if !part.is_empty() {
                    let part_width = part.chars().count();

                    if current_line_width + part_width > width && current_line_width > 0 {
                        line += 1;
                        current_line_width = 0;
                    }

                    if part_width > width {
                        for _ch in part.chars() {
                            if current_line_width >= width {
                                line += 1;
                                current_line_width = 0;
                            }
                            current_line_width += 1;
                        }
                    } else {
                        current_line_width += part_width;
                    }
                }
            }

            col = current_line_width;
        } else {
            let word_width = word.chars().count();

            if current_line_width + word_width > width && current_line_width > 0 {
                line += 1;
                current_line_width = 0;
            }

            if word_width > width {
                for _ch in word.chars() {
                    if current_line_width >= width {
                        line += 1;
                        current_line_width = 0;
                    }
                    current_line_width += 1;
                }
            } else {
                current_line_width += word_width;
            }

            col = current_line_width;
        }
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

    // Create centered popup area (12% width, 8% height for numeric input)
    let popup_area = centered_rect(f.area(), 12, 8);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Edit Miles")
        .style(Style::default().fg(Color::LightRed))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
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

    // Create centered popup area (12% width, 8% height for numeric input)
    let popup_area = centered_rect(f.area(), 12, 8);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Edit Elevation")
        .style(Style::default().fg(Color::LightRed))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
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

    // Create centered popup area (50% width, 13% height - half of previous 25%)
    let popup_area = centered_rect(f.area(), 50, 13);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!(
        "Add Sokay Entry - {}",
        state.selected_date.format("%B %d, %Y")
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Magenta))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
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

    // Create centered popup area (50% width, 13% height - half of previous 25%)
    let popup_area = centered_rect(f.area(), 50, 13);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let title = format!(
        "Edit Sokay Entry - {}",
        state.selected_date.format("%B %d, %Y")
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Magenta))
        .padding(ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        });

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render the input text
    let input_text = format_input_with_cursor(input_buffer);
    let input = Paragraph::new(input_text).style(create_input_style());
    f.render_widget(input, inner_area);

    // Set cursor position (inner area already accounts for borders and padding)
    f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
}

/// Renders the delete day confirmation screen
///
/// This screen asks the user to confirm deletion of an entire day's log.
/// Shows a warning message and waits for Y/N input.
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
        Type 'Y' to confirm deletion or 'N' to cancel.",
        selected_date.format("%B %d, %Y")
    );

    let warning_widget = Paragraph::new(warning_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title("Warning: Permanent Deletion")
                .padding(ratatui::widgets::Padding::new(1, 0, 1, 0)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(warning_widget, chunks[1]);

    render_help(f, chunks[2], "Y: Delete Day | N/Esc: Cancel", true, false);
}

/// Renders the delete food item confirmation dialog as a centered modal
///
/// Shows a small confirmation dialog asking the user to confirm deletion.
pub fn render_confirm_delete_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    food_index: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Get the food item name
    let food_name = if let Some(log) = state.get_daily_log(state.selected_date) {
        if food_index < log.food_entries.len() {
            log.food_entries[food_index].name.clone()
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    // Create centered popup area (60% width, 20% height)
    let popup_area = centered_rect(f.area(), 60, 20);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the confirmation message
    let message = format!(
        "Delete this food item?\n\n\
        \"{}\"\n\n\
        Press 'Y' to confirm or 'N' to cancel.",
        food_name
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title("Confirm Deletion")
        .padding(ratatui::widgets::Padding::uniform(1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let text = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(text, inner_area);
}

/// Renders the delete sokay item confirmation dialog as a centered modal
///
/// Shows a small confirmation dialog asking the user to confirm deletion.
pub fn render_confirm_delete_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    sokay_index: usize,
) {
    // First render the daily view in the background
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    // Get the sokay item text
    let sokay_text = if let Some(log) = state.get_daily_log(state.selected_date) {
        if sokay_index < log.sokay_entries.len() {
            log.sokay_entries[sokay_index].clone()
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    // Create centered popup area (60% width, 20% height)
    let popup_area = centered_rect(f.area(), 60, 20);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the confirmation message
    let message = format!(
        "Delete this sokay item?\n\n\
        \"{}\"\n\n\
        Press 'Y' to confirm or 'N' to cancel.",
        sokay_text
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title("Confirm Deletion")
        .padding(ratatui::widgets::Padding::uniform(1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let text = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(text, inner_area);
}

pub fn render_shortcuts_help_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let popup_area = centered_rect(f.area(), 70, 50);

    f.render_widget(Clear, popup_area);

    let shortcuts_text = "\
Measurements:
  w - Edit weight
  s - Edit waist size

Activity:
  m - Edit miles covered
  l - Edit elevation gain

Nutrition:
  f - Add food item
  c - Add sokay entry

Training:
  t - Edit strength & mobility
  n - Edit daily notes

Press Space to close";

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title("Shortcuts")
        .padding(ratatui::widgets::Padding::uniform(1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let text = Paragraph::new(shortcuts_text)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(text, inner_area);
}
