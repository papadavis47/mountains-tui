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

/// Renders a footer help bar, choosing the widest tier that fits the area.
///
/// `tiers` lists candidate help strings ordered from fullest to most minimal;
/// the widest one whose rendered width fits the (border-adjusted) area is shown,
/// so the footer never truncates on narrow/split terminals. The last tier is the
/// guaranteed fallback when even it overflows.
pub fn render_help(f: &mut Frame, area: Rect, tiers: &[&str], show_border: bool, centered: bool) {
    let available = if show_border {
        area.width.saturating_sub(2)
    } else {
        area.width
    } as usize;

    let spans = build_help_spans(choose_help_tier(tiers, available));

    let block = if show_border {
        Block::default().borders(Borders::ALL)
    } else {
        Block::default().borders(Borders::NONE)
    };

    let mut help_widget = Paragraph::new(Line::from(spans)).block(block);

    if centered {
        help_widget = help_widget.alignment(ratatui::layout::Alignment::Center);
    }

    f.render_widget(help_widget, area);
}

/// Picks the widest tier whose rendered width fits `available`, falling back to
/// the last (most minimal) tier when none fit and to `""` when `tiers` is empty.
fn choose_help_tier<'a>(tiers: &[&'a str], available: usize) -> &'a str {
    tiers
        .iter()
        .find(|t| help_line_width(t) <= available)
        .or_else(|| tiers.last())
        .copied()
        .unwrap_or("")
}

/// Rendered display width of a help string once styled into spans.
fn help_line_width(help_text: &str) -> usize {
    build_help_spans(help_text)
        .iter()
        .map(|s| s.width())
        .sum()
}

/// Parses a `key: desc | key: desc` help string into styled spans.
fn build_help_spans(help_text: &str) -> Vec<Span<'static>> {
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

    spans
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

#[cfg(test)]
mod tests {
    use super::*;

    const TIERS: &[&str] = &[
        "Shift+J/K: Section | Enter: Add | Space: Shortcuts | Esc: Back",
        "Enter: Add | Space: More | Esc: Back",
        "Esc: Back",
    ];

    #[test]
    fn choose_help_tier_picks_full_when_it_fits() {
        let full = help_line_width(TIERS[0]);
        assert_eq!(choose_help_tier(TIERS, full), TIERS[0]);
        assert_eq!(choose_help_tier(TIERS, full + 100), TIERS[0]);
    }

    #[test]
    fn choose_help_tier_steps_down_as_width_shrinks() {
        let full = help_line_width(TIERS[0]);
        let mid = help_line_width(TIERS[1]);

        // One column short of the full tier -> next tier down.
        assert_eq!(choose_help_tier(TIERS, full - 1), TIERS[1]);
        // Exact fit of the middle tier.
        assert_eq!(choose_help_tier(TIERS, mid), TIERS[1]);
        // Too narrow for the middle tier -> minimal tier.
        assert_eq!(choose_help_tier(TIERS, mid - 1), TIERS[2]);
    }

    #[test]
    fn choose_help_tier_falls_back_to_last_when_none_fit() {
        let min = help_line_width(TIERS[2]);
        assert_eq!(choose_help_tier(TIERS, 0), TIERS[2]);
        assert_eq!(choose_help_tier(TIERS, min - 1), TIERS[2]);
    }

    #[test]
    fn choose_help_tier_handles_empty() {
        assert_eq!(choose_help_tier(&[], 40), "");
    }

    #[test]
    fn help_line_width_matches_rendered_width() {
        // Separators render as " | " (3 cols) and the leading key/desc keep their
        // spacing, so measured width tracks the displayed line, not the raw source.
        assert_eq!(help_line_width("Esc: Back"), 9);
        assert_eq!(help_line_width("a: A | b: B"), 11);
    }
}
