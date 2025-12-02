use chrono::{Datelike, NaiveDate};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::miles_stats::{calculate_yearly_miles, calculate_monthly_miles};
use crate::models::{AppState, DailyLog, FocusedSection, MeasurementField, RunningField};
use crate::ui::components::{create_highlight_style, render_help, render_title};

/// Renders the daily view screen for a specific date
pub fn render_daily_view_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
) {
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

    let title = format!(
        "Mountains Training Log - {} {}",
        state.selected_date.format("%B %d, %Y"),
        sync_status
    );
    render_title(f, chunks[0], &title);

    render_measurements_section(
        f,
        chunks[1],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
    );

    let yearly_miles = calculate_yearly_miles(&state.daily_logs);
    let monthly_miles = calculate_monthly_miles(&state.daily_logs);
    render_running_section(
        f,
        chunks[2],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
        yearly_miles,
        monthly_miles,
    );

    render_food_list_section(
        f,
        chunks[3],
        state.selected_date,
        &state.daily_logs,
        food_list_state,
        &state.focused_section,
        state.food_list_focused,
    );

    render_sokay_section(
        f,
        chunks[4],
        state.selected_date,
        &state.daily_logs,
        sokay_list_state,
        &state.focused_section,
        state.sokay_list_focused,
    );

    render_strength_mobility_section(
        f,
        chunks[5],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
    );

    render_notes_section(
        f,
        chunks[6],
        state.selected_date,
        &state.daily_logs,
        &state.focused_section,
    );

    render_help(
        f,
        chunks[7],
        " Shift+J/K: Section | Tab: Field | Enter: Add | j/k: List | E: Edit Item | D: Delete Item | Space: Shortcuts | S: Startup Screen | Esc: Back ",
        true,
        false,
    );

    // Render expanded overlay for multi-line sections when focused
    match &state.focused_section {
        FocusedSection::StrengthMobility => {
            render_strength_mobility_expanded(
                f,
                chunks[5],
                state.selected_date,
                &state.daily_logs,
                state.strength_mobility_scroll,
            );
        }
        FocusedSection::Notes => {
            render_notes_expanded(
                f,
                chunks[6],
                state.selected_date,
                &state.daily_logs,
                state.notes_scroll,
            );
        }
        _ => {}
    }
}

/// Renders the measurements display section
fn render_measurements_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let (has_focus, focused_field) = match focused_section {
        FocusedSection::Measurements { focused_field } => (true, Some(focused_field)),
        _ => (false, None),
    };

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

    let border_style = if has_focus {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

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
fn render_running_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
    yearly_miles: f32,
    monthly_miles: f32,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let (has_focus, focused_field) = match focused_section {
        FocusedSection::Running { focused_field } => (true, Some(focused_field)),
        _ => (false, None),
    };

    let now = chrono::Local::now();
    let current_year = now.year();
    let current_month = now.month();

    let month_name = match current_month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };

    let yearly_text = format!("You have {:.1} miles covered for {}", yearly_miles, current_year);
    let monthly_text = if monthly_miles == 0.0 {
        format!("No miles covered yet for the month of {}", month_name)
    } else {
        format!("{:.1} miles covered for the month of {}", monthly_miles, month_name)
    };

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

        format!("{} | {} | {} | {}", miles_display, elevation_display, yearly_text, monthly_text)
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

        format!("{} | {} | {} | {}", miles_display, elevation_display, yearly_text, monthly_text)
    };

    let border_style = if has_focus {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default().fg(Color::DarkGray)
    };

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
fn render_food_list_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    food_list_state: &mut ListState,
    focused_section: &FocusedSection,
    food_list_focused: bool,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let items: Vec<ListItem> = if let Some(log) = log {
        if log.food_entries.is_empty() {
            vec![ListItem::new("No food entries yet. Press 'f' to add one.")]
        } else {
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

    let border_style = if matches!(focused_section, FocusedSection::FoodItems) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let highlight_style =
        if matches!(focused_section, FocusedSection::FoodItems) && food_list_focused {
            create_highlight_style()
        } else {
            Style::default()
        };

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
fn render_sokay_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    sokay_list_state: &mut ListState,
    focused_section: &FocusedSection,
    sokay_list_focused: bool,
) {
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
            strength_mobility_scroll: 0,
            notes_scroll: 0,
        },
        selected_date,
    );

    let title = format!("Sokay (Total: {})", cumulative_sokay);

    let items: Vec<ListItem> = if let Some(log) = log {
        if log.sokay_entries.is_empty() {
            vec![ListItem::new("No sokay entries yet. Press 'c' to add one.")]
        } else {
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

    let border_style = if matches!(focused_section, FocusedSection::Sokay) {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let highlight_style = if matches!(focused_section, FocusedSection::Sokay) && sokay_list_focused
    {
        create_highlight_style()
    } else {
        Style::default()
    };

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
fn render_strength_mobility_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let has_focus = matches!(focused_section, FocusedSection::StrengthMobility);

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

    let border_style = if has_focus {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

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
fn render_notes_section(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    focused_section: &FocusedSection,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let has_focus = matches!(focused_section, FocusedSection::Notes);

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

    let border_style = if has_focus {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

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

/// Calculates the number of display lines needed for text at given width
fn calculate_text_height(text: &str, width: usize) -> usize {
    if text.is_empty() || width == 0 {
        return 1;
    }

    let mut total_lines = 0;
    for line in text.lines() {
        if line.is_empty() {
            total_lines += 1;
        } else {
            let line_len = line.chars().count();
            let wrapped_lines = line_len.div_ceil(width);
            total_lines += wrapped_lines.max(1);
        }
    }

    total_lines.max(1)
}

/// Renders expanded Strength & Mobility section when focused
fn render_strength_mobility_expanded(
    f: &mut Frame,
    original_area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    scroll_offset: u16,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let text = if let Some(log) = log {
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

    let default_height = 4;
    let width = original_area.width.saturating_sub(4) as usize;
    let content_height = calculate_text_height(&text, width);
    let needed_height = (content_height as u16) + 2;

    if needed_height <= default_height {
        return;
    }

    let max_height = (f.area().height * 60 / 100).max(default_height);
    let expanded_height = needed_height.min(max_height);

    let expanded_area = ratatui::layout::Rect {
        x: original_area.x,
        y: original_area.y + original_area.height - expanded_height,
        width: original_area.width,
        height: expanded_height,
    };

    f.render_widget(Clear, expanded_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title("Strength & Mobility")
        .padding(ratatui::widgets::Padding::horizontal(1));

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((scroll_offset, 0));

    f.render_widget(paragraph, expanded_area);
}

/// Renders expanded Notes section when focused
fn render_notes_expanded(
    f: &mut Frame,
    original_area: ratatui::layout::Rect,
    selected_date: NaiveDate,
    daily_logs: &[DailyLog],
    scroll_offset: u16,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let text = if let Some(log) = log {
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

    let default_height = 4;
    let width = original_area.width.saturating_sub(4) as usize;
    let content_height = calculate_text_height(&text, width);
    let needed_height = (content_height as u16) + 2;

    if needed_height <= default_height {
        return;
    }

    let max_height = (f.area().height * 60 / 100).max(default_height);
    let expanded_height = needed_height.min(max_height);

    let expanded_area = ratatui::layout::Rect {
        x: original_area.x,
        y: original_area.y + original_area.height - expanded_height,
        width: original_area.width,
        height: expanded_height,
    };

    f.render_widget(Clear, expanded_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title("Notes")
        .padding(ratatui::widgets::Padding::horizontal(1));

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Green))
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((scroll_offset, 0));

    f.render_widget(paragraph, expanded_area);
}
