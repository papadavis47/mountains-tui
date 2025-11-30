use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::components::{centered_rect, create_input_style, format_input_with_cursor};
use crate::ui::screens::{calculate_cursor_in_wrapped_text, wrap_at_width};

/// Types of input modals
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputModalType {
    /// Single-line text input (50% x 13%)
    Text,
    /// Single-line numeric input (12% x 8%)
    Numeric,
    /// Multi-line text input with wrapping (60% x 40%)
    Multiline,
}

impl InputModalType {
    fn dimensions(&self) -> (u16, u16) {
        match self {
            InputModalType::Text => (50, 13),
            InputModalType::Numeric => (12, 8),
            InputModalType::Multiline => (60, 40),
        }
    }

    fn padding(&self) -> ratatui::widgets::Padding {
        match self {
            InputModalType::Text | InputModalType::Numeric => ratatui::widgets::Padding {
                left: 1,
                right: 1,
                top: 1,
                bottom: 0,
            },
            InputModalType::Multiline => ratatui::widgets::Padding::uniform(1),
        }
    }
}

/// Configuration for rendering an input modal
pub struct InputModalConfig {
    pub title: String,
    pub border_color: Color,
    pub modal_type: InputModalType,
}

impl InputModalConfig {
    pub fn new(title: String, border_color: Color, modal_type: InputModalType) -> Self {
        Self {
            title,
            border_color,
            modal_type,
        }
    }

    /// Helper for text input modals
    pub fn text(title: String, border_color: Color) -> Self {
        Self::new(title, border_color, InputModalType::Text)
    }

    /// Helper for numeric input modals
    pub fn numeric(title: String, border_color: Color) -> Self {
        Self::new(title, border_color, InputModalType::Numeric)
    }

    /// Helper for multiline input modals
    pub fn multiline(title: String, border_color: Color) -> Self {
        Self::new(title, border_color, InputModalType::Multiline)
    }
}

/// Renders a generic input modal over the current screen
pub fn render_input_modal(
    f: &mut Frame,
    config: InputModalConfig,
    input_buffer: &str,
    cursor_position: usize,
) {
    let (width_percent, height_percent) = config.modal_type.dimensions();
    let popup_area = centered_rect(f.area(), width_percent, height_percent);

    // Clear the popup area to prevent visual artifacts
    f.render_widget(Clear, popup_area);

    // Create the dialog block with title and padding
    let block = Block::default()
        .borders(Borders::ALL)
        .title(config.title)
        .style(Style::default().fg(config.border_color))
        .padding(config.modal_type.padding());

    // Get the inner area for the input text (after borders and padding)
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Render based on modal type
    match config.modal_type {
        InputModalType::Text | InputModalType::Numeric => {
            // Single-line input rendering
            let input_text = format_input_with_cursor(input_buffer);
            let input = Paragraph::new(input_text).style(create_input_style());
            f.render_widget(input, inner_area);

            // Set cursor position (inner area already accounts for borders and padding)
            f.set_cursor_position((inner_area.x + cursor_position as u16, inner_area.y));
        }
        InputModalType::Multiline => {
            // Multi-line input rendering with word wrapping
            let width = inner_area.width as usize;
            let wrapped_text = if input_buffer.is_empty() {
                " ".to_string()
            } else {
                wrap_at_width(input_buffer, width)
            };

            let input = Paragraph::new(wrapped_text).style(create_input_style());
            f.render_widget(input, inner_area);

            // Calculate cursor position on the wrapped text
            let (cursor_x, cursor_y) =
                calculate_cursor_in_wrapped_text(inner_area, input_buffer, cursor_position, width);
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
