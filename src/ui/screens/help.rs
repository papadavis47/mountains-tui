use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Gauge, ListState, Paragraph},
};

use crate::models::AppState;
use crate::ui::components::centered_rect;
use super::daily_view::render_daily_view_screen;

/// Renders the shortcuts help overlay on the daily view screen
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
  Alt+Enter - Insert newline (in multiline fields)

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

/// Renders the syncing screen with a centered modal and progress gauge
pub fn render_syncing_screen(f: &mut Frame, sync_status: &str) {
    let popup_area = centered_rect(f.area(), 60, 25);

    f.render_widget(Clear, popup_area);

    let is_offline = sync_status.contains("Offline") || sync_status.contains("network");
    let is_complete = sync_status.contains("complete");

    let border_color = if is_offline {
        Color::Rgb(255, 165, 0) // Orange for offline
    } else if is_complete {
        Color::Green
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(if is_offline { "Offline" } else { "Syncing" })
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .padding(ratatui::widgets::Padding::uniform(1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Message
            Constraint::Length(1), // Gauge
            Constraint::Min(0),    // Spacing
        ])
        .split(inner_area);

    let message = Paragraph::new(sync_status)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(message, chunks[0]);

    if !is_offline {
        let gauge_percent = if is_complete { 100 } else { 50 };
        let gauge_color = if is_complete { Color::Green } else { Color::Cyan };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(gauge_color))
            .ratio(gauge_percent as f64 / 100.0)
            .use_unicode(true);

        f.render_widget(gauge, chunks[1]);
    } else {
        let offline_note = Paragraph::new("Changes will sync on next startup")
            .style(Style::default().fg(Color::Rgb(255, 165, 0)))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(offline_note, chunks[1]);
    }
}
