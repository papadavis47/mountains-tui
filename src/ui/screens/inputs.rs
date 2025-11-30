use ratatui::{Frame, style::Color, widgets::ListState};

use crate::models::AppState;
use crate::ui::modals::{render_input_modal, InputModalConfig};
use super::daily_view::render_daily_view_screen;

/// Renders the add food entry screen as a centered modal dialog
pub fn render_add_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let title = format!("Add Food - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::text(title, Color::Yellow);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit food entry screen as a centered modal dialog
pub fn render_edit_food_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let title = format!("Edit Food - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::text(title, Color::Yellow);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit weight screen as a centered modal dialog
pub fn render_edit_weight_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let config = InputModalConfig::numeric("Edit Weight".to_string(), Color::Yellow);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit waist measurement screen as a centered modal dialog
pub fn render_edit_waist_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let config = InputModalConfig::numeric("Edit Waist Size".to_string(), Color::Yellow);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit miles screen as a centered modal dialog
pub fn render_edit_miles_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let config = InputModalConfig::numeric("Edit Miles".to_string(), Color::LightRed);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit elevation screen as a centered modal dialog
pub fn render_edit_elevation_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let config = InputModalConfig::numeric("Edit Elevation".to_string(), Color::LightRed);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit strength & mobility screen as a centered modal dialog
pub fn render_edit_strength_mobility_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let title = format!("Edit Strength & Mobility - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::multiline(title, Color::Cyan);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit notes screen as a centered modal dialog
pub fn render_edit_notes_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let title = format!("Edit Notes - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::multiline(title, Color::Green);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the add sokay screen as a centered modal dialog
pub fn render_add_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let title = format!("Add Sokay Entry - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::text(title, Color::Magenta);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Renders the edit sokay screen as a centered modal dialog
pub fn render_edit_sokay_screen(
    f: &mut Frame,
    state: &AppState,
    food_list_state: &mut ListState,
    sokay_list_state: &mut ListState,
    sync_status: &str,
    input_buffer: &str,
    cursor_position: usize,
) {
    render_daily_view_screen(f, state, food_list_state, sokay_list_state, sync_status);

    let title = format!("Edit Sokay Entry - {}", state.selected_date.format("%B %d, %Y"));
    let config = InputModalConfig::text(title, Color::Magenta);
    render_input_modal(f, config, input_buffer, cursor_position);
}

/// Wraps text at word boundaries to fit within a given width
pub fn wrap_at_width(text: &str, width: usize) -> String {
    if width == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    let mut current_line = String::new();
    let mut current_line_width = 0;

    for word in text.split_inclusive(|c: char| c.is_whitespace() || c == '\n') {
        if word.contains('\n') {
            let parts: Vec<&str> = word.split('\n').collect();

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    result.push_str(&current_line);
                    result.push('\n');
                    current_line.clear();
                    current_line_width = 0;
                }

                if !part.is_empty() {
                    let part_width = part.chars().count();

                    if current_line_width + part_width > width && current_line_width > 0 {
                        result.push_str(&current_line);
                        result.push('\n');
                        current_line.clear();
                        current_line_width = 0;
                    }

                    if part_width > width {
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
            let word_width = word.chars().count();

            if current_line_width + word_width > width && current_line_width > 0 {
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
                current_line_width = 0;
            }

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

    if !current_line.is_empty() {
        result.push_str(&current_line);
    }

    result
}

/// Calculates cursor position in manually-wrapped text with word wrapping
pub fn calculate_cursor_in_wrapped_text(
    area: ratatui::layout::Rect,
    original_text: &str,
    cursor_pos_bytes: usize,
    width: usize,
) -> (u16, u16) {
    if width == 0 {
        return (area.x, area.y);
    }

    let cursor_pos_chars = if cursor_pos_bytes <= original_text.len() {
        original_text[..cursor_pos_bytes].chars().count()
    } else {
        original_text.chars().count()
    };

    let mut line = 0;
    let mut col = 0;
    let mut char_count = 0;
    let mut current_line_width = 0;

    for word in original_text.split_inclusive(|c: char| c.is_whitespace() || c == '\n') {
        let word_char_count = word.chars().count();

        if char_count + word_char_count > cursor_pos_chars {
            let chars_into_word = cursor_pos_chars - char_count;

            if word.contains('\n') {
                let parts: Vec<&str> = word.split('\n').collect();
                let mut chars_processed = 0;

                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        line += 1;
                        col = 0;
                        current_line_width = 0;
                        chars_processed += 1;

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
                let word_width = word.chars().count();

                if current_line_width + word_width > width && current_line_width > 0 {
                    line += 1;
                    current_line_width = 0;
                }

                if word_width > width {
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
