use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

use crate::models::{AppState, ConfigSyncField};
use crate::ui::components::centered_rect;
use super::startup::render_startup_screen;

pub fn render_config_sync_screen(
    f: &mut Frame,
    state: &AppState,
    url_buffer: &str,
    token_buffer: &str,
    sync_enabled: bool,
    has_saved_token: bool,
) {
    // Render startup screen behind as backdrop
    render_startup_screen(f, state);

    let popup_area = centered_rect(f.area(), 60, 50);
    f.render_widget(Clear, popup_area);

    let border_color = Color::Cyan;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(" Configure Cloud Sync ")
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::new(2, 2, 1, 1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let focused = &state.config_sync_focused_field;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // DB URL label
            Constraint::Length(3), // DB URL input (bordered)
            Constraint::Length(1), // spacing
            Constraint::Length(1), // Auth Token label
            Constraint::Length(3), // Auth Token input (bordered)
            Constraint::Length(1), // token hint
            Constraint::Length(1), // spacing
            Constraint::Length(1), // Enable toggle
            Constraint::Length(1), // spacing
            Constraint::Length(1), // status message
            Constraint::Min(0),   // remaining
            Constraint::Length(1), // help line
        ])
        .split(inner_area);

    // DB URL label
    let url_label_style = if *focused == ConfigSyncField::DbUrl {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    f.render_widget(Paragraph::new("Database URL:").style(url_label_style), chunks[0]);

    // DB URL input
    let url_display = if url_buffer.is_empty() { " " } else { url_buffer };
    let url_style = if *focused == ConfigSyncField::DbUrl {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let url_border_color = if *focused == ConfigSyncField::DbUrl {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(url_border_color));
    f.render_widget(
        Paragraph::new(url_display).style(url_style).block(url_block),
        chunks[1],
    );

    // Auth Token label
    let token_label_style = if *focused == ConfigSyncField::AuthToken {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    f.render_widget(Paragraph::new("Auth Token:").style(token_label_style), chunks[3]);

    // Auth Token input — mask with stars, show placeholder if saved token exists
    let token_display = if token_buffer.is_empty() {
        if has_saved_token { "****" } else { " " }
    } else {
        // Show stars for each character
        &"*".repeat(token_buffer.len())
    };
    let token_style = if *focused == ConfigSyncField::AuthToken {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let token_border_color = if *focused == ConfigSyncField::AuthToken {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let token_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(token_border_color));

    // Need to own the string for lifetime
    let token_stars;
    let token_text = if !token_buffer.is_empty() {
        token_stars = "*".repeat(token_buffer.len());
        token_stars.as_str()
    } else {
        token_display
    };

    f.render_widget(
        Paragraph::new(token_text).style(token_style).block(token_block),
        chunks[4],
    );

    // Token hint
    if has_saved_token && token_buffer.is_empty() {
        f.render_widget(
            Paragraph::new(" (leave empty to keep existing)")
                .style(Style::default().fg(Color::DarkGray)),
            chunks[5],
        );
    }

    // Enable toggle
    let toggle_style = if *focused == ConfigSyncField::EnableToggle {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let enabled_span = if sync_enabled {
        Span::styled("[Enabled]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("Enabled", toggle_style)
    };

    let disabled_span = if !sync_enabled {
        Span::styled("[Disabled]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("Disabled", toggle_style)
    };

    let toggle_line = Line::from(vec![
        Span::styled("Cloud Sync: ", toggle_style),
        enabled_span,
        Span::styled(" / ", toggle_style),
        disabled_span,
    ]);
    f.render_widget(Paragraph::new(toggle_line), chunks[7]);

    // Status message
    if let Some(status) = &state.config_sync_status {
        let color = if status.contains("Saved") {
            Color::Green
        } else if status.contains("Error") {
            Color::Red
        } else {
            Color::Yellow
        };
        f.render_widget(
            Paragraph::new(status.as_str()).style(Style::default().fg(color)),
            chunks[9],
        );
    }

    // Help line
    let help_spans = vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::styled(": Next Field | ", Style::default().fg(Color::White)),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::styled(": Toggle | ", Style::default().fg(Color::White)),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::styled(": Save | ", Style::default().fg(Color::White)),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::styled(": Cancel", Style::default().fg(Color::White)),
    ];
    f.render_widget(
        Paragraph::new(Line::from(help_spans)).alignment(ratatui::layout::Alignment::Center),
        chunks[11],
    );
}
