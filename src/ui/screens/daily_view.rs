use chrono::{Datelike, NaiveDate};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::miles_stats::{calculate_monthly_miles, calculate_yearly_miles};
use crate::models::field_accessor::FieldType;
use crate::models::{AppState, DailyLog, FocusedSection, MeasurementField, RunningField};
use crate::ui::components::{create_highlight_style, render_help, render_title};

/// Active in-place edit of a numeric field, rendered directly inside its section
/// row (Measurements / Running) instead of in a popup modal.
pub struct InPlaceEdit<'a> {
    pub field: FieldType,
    pub buffer: &'a str,
    pub cursor: usize,
}

/// Renders the daily view screen for a specific date
pub fn render_daily_view_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    edit: Option<InPlaceEdit>,
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
        edit.as_ref(),
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
        edit.as_ref(),
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

    let help_tiers: &[&str] = if edit.is_some() {
        &[
            " Editing — type value | Enter: Save | Esc: Cancel",
            " Enter: Save | Esc: Cancel",
        ]
    } else {
        &[
            " Shift+J/K: Section | Tab: Toggle Num Fields | Enter: Add | j/k: List | e: Edit Item | d: Delete Item | Space: Shortcuts | S: Startup Screen | Esc: Back",
            " Shift+J/K: Section | Tab: Fields | Enter: Add | j/k: List | e: Edit | d: Delete | Space: Shortcuts | S: Startup | Esc: Back",
            " Shift+J/K: Section | Enter: Add | e: Edit | d: Delete | Space: More | Esc: Back",
            " Space: Shortcuts | Esc: Back",
        ]
    };
    render_help(f, chunks[7], help_tiers, true, false);

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
    edit: Option<&InPlaceEdit>,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    // A field in this section being actively edited in place (Weight or Waist).
    let editing_field = match edit.map(|e| e.field) {
        Some(FieldType::Weight) => Some(MeasurementField::Weight),
        Some(FieldType::Waist) => Some(MeasurementField::Waist),
        _ => None,
    };

    let section_focused = matches!(focused_section, FocusedSection::Measurements { .. });
    let has_focus = section_focused || editing_field.is_some();

    // The field showing the ► marker: the edited field while editing, else the
    // section's focused field.
    let marked_field: Option<MeasurementField> =
        editing_field.clone().or_else(|| match focused_section {
            FocusedSection::Measurements { focused_field } => Some(focused_field.clone()),
            _ => None,
        });

    let weight_value = log.and_then(|l| l.weight).map(|w| format!("{} lbs", w));
    let waist_value = log.and_then(|l| l.waist).map(|w| format!("{} in", w));

    let base = Style::default().fg(Color::Yellow);
    let mut spans: Vec<Span> = Vec::new();
    let mut width: u16 = 0;
    let mut caret_col: Option<u16> = None;

    push_field(
        &mut spans,
        &mut caret_col,
        &mut width,
        base,
        marked_field.as_ref() == Some(&MeasurementField::Weight),
        "Weight: ",
        if editing_field == Some(MeasurementField::Weight) {
            edit
        } else {
            None
        },
        weight_value.as_deref(),
        " lbs",
        "Press 'w' to add",
    );
    push_span(&mut spans, &mut width, " | ".to_string(), base);
    push_field(
        &mut spans,
        &mut caret_col,
        &mut width,
        base,
        marked_field.as_ref() == Some(&MeasurementField::Waist),
        "Waist Size: ",
        if editing_field == Some(MeasurementField::Waist) {
            edit
        } else {
            None
        },
        waist_value.as_deref(),
        " in",
        "Press 's' to add",
    );

    let border_style = if has_focus {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Measurements")
        .padding(ratatui::widgets::Padding::horizontal(1));
    let inner = block.inner(area);

    let measurements_widget = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(measurements_widget, area);

    if let Some(col) = caret_col {
        f.set_cursor_position((inner.x + col, inner.y));
    }
}

/// Dimmed style for inline "Press 'x' to add" placeholders shown when a numeric
/// field is unset, matching the dimmed placeholders used by the list sections.
fn placeholder_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Pushes a styled span and advances the running display width (in cells) used
/// for caret positioning.
fn push_span(spans: &mut Vec<Span<'static>>, width: &mut u16, text: String, style: Style) {
    *width += Span::raw(text.as_str()).width() as u16;
    spans.push(Span::styled(text, style));
}

/// Appends one labelled field to a section row, recording the caret column when
/// the field is being edited in place. `marked` adds the ► focus marker; `edit`
/// (when `Some`) substitutes the input buffer for the value and sets the caret.
/// When `value` is `None` and the field isn't being edited, the dimmed `help`
/// placeholder is shown in place of the value.
fn push_field(
    spans: &mut Vec<Span<'static>>,
    caret_col: &mut Option<u16>,
    width: &mut u16,
    base_style: Style,
    marked: bool,
    label: &str,
    edit: Option<&InPlaceEdit>,
    value: Option<&str>,
    unit: &str,
    help: &str,
) {
    if marked {
        push_span(spans, width, "► ".to_string(), base_style);
    }
    push_span(spans, width, label.to_string(), base_style);

    if let Some(edit) = edit {
        // Caret sits within the buffer; digits/dots are width-1 so char count == cells.
        *caret_col = Some(*width + edit.cursor as u16);
        push_span(spans, width, edit.buffer.to_string(), base_style);
        // Extra leading space so the block cursor doesn't sit flush against the unit.
        push_span(spans, width, format!(" {}", unit), base_style);
    } else if let Some(value) = value {
        push_span(spans, width, value.to_string(), base_style);
    } else {
        push_span(spans, width, help.to_string(), placeholder_style());
    }
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
    edit: Option<&InPlaceEdit>,
) {
    let log = daily_logs.iter().find(|log| log.date == selected_date);

    let editing_field = match edit.map(|e| e.field) {
        Some(FieldType::Miles) => Some(RunningField::Miles),
        Some(FieldType::Elevation) => Some(RunningField::Elevation),
        _ => None,
    };

    let section_focused = matches!(focused_section, FocusedSection::Running { .. });
    let has_focus = section_focused || editing_field.is_some();

    let marked_field: Option<RunningField> =
        editing_field.clone().or_else(|| match focused_section {
            FocusedSection::Running { focused_field } => Some(focused_field.clone()),
            _ => None,
        });

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

    let yearly_text = format!(
        "You have {:.1} miles covered for {}",
        yearly_miles, current_year
    );
    let monthly_text = if monthly_miles == 0.0 {
        format!("No miles covered yet for the month of {}", month_name)
    } else {
        format!(
            "{:.1} miles covered for the month of {}",
            monthly_miles, month_name
        )
    };

    let miles_value = log
        .and_then(|l| l.miles_covered)
        .map(|m| format!("{} mi", m));
    let elevation_value = log
        .and_then(|l| l.elevation_gain)
        .map(|e| format!("{} ft", e));

    let base = Style::default().fg(Color::LightRed);
    let mut spans: Vec<Span> = Vec::new();
    let mut width: u16 = 0;
    let mut caret_col: Option<u16> = None;

    push_field(
        &mut spans,
        &mut caret_col,
        &mut width,
        base,
        marked_field.as_ref() == Some(&RunningField::Miles),
        "Miles: ",
        if editing_field == Some(RunningField::Miles) {
            edit
        } else {
            None
        },
        miles_value.as_deref(),
        " mi",
        "Press 'm' to add",
    );
    push_span(&mut spans, &mut width, " | ".to_string(), base);
    push_field(
        &mut spans,
        &mut caret_col,
        &mut width,
        base,
        marked_field.as_ref() == Some(&RunningField::Elevation),
        "Elevation: ",
        if editing_field == Some(RunningField::Elevation) {
            edit
        } else {
            None
        },
        elevation_value.as_deref(),
        " ft",
        "Press 'l' to add",
    );
    push_span(
        &mut spans,
        &mut width,
        format!(" | {} | {}", yearly_text, monthly_text),
        base,
    );

    let border_style = if has_focus {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Running")
        .padding(ratatui::widgets::Padding::horizontal(1));
    let inner = block.inner(area);

    let running_widget = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(running_widget, area);

    if let Some(col) = caret_col {
        f.set_cursor_position((inner.x + col, inner.y));
    }
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
            date_input_error: None,
            config_sync_focused_field: crate::models::ConfigSyncField::DbUrl,
            config_sync_status: None,
            frame_width: 0,
            frame_height: 0,
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

/// Maximum vertical scroll offset (in lines) for an expandable multi-line
/// section, given the full frame size. Mirrors the geometry used by the
/// expanded renderers so scrolling can be clamped to the content; returns 0
/// when the content fits without scrolling.
pub fn max_scroll_offset(text: &str, frame_width: u16, frame_height: u16) -> u16 {
    let default_height = 4;
    // The daily view layout insets the frame by a 1-cell margin on each side,
    // and each section block has a border + 1 cell of horizontal padding.
    let section_width = frame_width.saturating_sub(2);
    let width = section_width.saturating_sub(4) as usize;
    let content_height = calculate_text_height(text, width) as u16;
    let needed_height = content_height + 2;

    if needed_height <= default_height {
        return 0;
    }

    let max_height = (frame_height * 60 / 100).max(default_height);
    let expanded_height = needed_height.min(max_height);
    let visible_text_lines = expanded_height.saturating_sub(2); // top + bottom border
    // `content_height` is a character-based estimate; word wrapping can render a
    // line or two more than estimated, so add a 1-line buffer to guarantee the
    // last line is always reachable.
    content_height.saturating_sub(visible_text_lines) + 1
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
