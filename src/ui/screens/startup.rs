use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::assets::APP_TITLE;
use crate::elevation_stats::{
    calculate_yearly_elevation, count_monthly_1000_days, get_streak_message,
};
use crate::models::AppState;
use crate::ui::components::{create_standard_layout, render_help};

/// Renders the startup screen with ASCII art and elevation statistics
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
