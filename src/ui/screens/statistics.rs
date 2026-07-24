use chrono::{Datelike, Days, NaiveDate};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::elevation_stats::{
    calculate_monthly_elevation, calculate_weekly_elevation, calculate_yearly_elevation,
    count_monthly_1000_days, get_streak_message,
};
use crate::miles_stats::{calculate_monthly_miles, calculate_weekly_miles, calculate_yearly_miles};
use crate::models::AppState;
use crate::ui::components::{create_standard_layout, render_help, render_title};
use crate::ui::{ClickAction, ClickTarget};

pub fn render_statistics_screen(
    f: &mut Frame,
    state: &AppState,
    reference_date: NaiveDate,
    click_targets: &mut Vec<ClickTarget>,
) {
    let chunks = create_standard_layout(f.area());
    let title = format!(
        "Mountains Statistics - {}",
        reference_date.format("%B %d, %Y")
    );
    render_title(f, chunks[0], &title);

    let weekly_miles = calculate_weekly_miles(&state.daily_logs, reference_date);
    let monthly_miles = calculate_monthly_miles(&state.daily_logs, reference_date);
    let yearly_miles = calculate_yearly_miles(&state.daily_logs, reference_date);
    let weekly_elevation = calculate_weekly_elevation(&state.daily_logs, reference_date);
    let monthly_elevation = calculate_monthly_elevation(&state.daily_logs, reference_date);
    let yearly_elevation = calculate_yearly_elevation(&state.daily_logs, reference_date);
    let monthly_1000_days = count_monthly_1000_days(&state.daily_logs, reference_date);

    let week = reference_date.iso_week();
    let monday = reference_date
        .checked_sub_days(Days::new(
            reference_date.weekday().num_days_from_monday() as u64
        ))
        .unwrap_or(reference_date);
    let sunday = monday.checked_add_days(Days::new(6)).unwrap_or(monday);
    let week_label = format!(
        "Week {} ({}–{})",
        week.week(),
        monday.format("%b %d"),
        sunday.format("%b %d")
    );
    let month_label = reference_date.format("%B %Y").to_string();
    let year_label = reference_date.year().to_string();

    let lines = if chunks[1].height < 12 {
        compact_lines(
            &week_label,
            &month_label,
            &year_label,
            weekly_miles,
            monthly_miles,
            yearly_miles,
            weekly_elevation,
            monthly_elevation,
            yearly_elevation,
            monthly_1000_days,
            &get_streak_message(&state.daily_logs),
        )
    } else {
        detailed_lines(
            &week_label,
            &month_label,
            &year_label,
            weekly_miles,
            monthly_miles,
            yearly_miles,
            weekly_elevation,
            monthly_elevation,
            yearly_elevation,
            monthly_1000_days,
            &get_streak_message(&state.daily_logs),
        )
    };

    let statistics = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Activity Totals")
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(statistics, chunks[1]);

    let help_regions = render_help(
        f,
        chunks[2],
        &[" Esc: Startup | q: Quit", " Esc: Back | q: Quit"],
        true,
        true,
    );
    for region in help_regions {
        let action = match region.key.as_str() {
            "Esc" => Some(ClickAction::BackToStartup),
            "q" => Some(ClickAction::Quit),
            _ => None,
        };
        if let Some(action) = action {
            click_targets.push(ClickTarget::new(region.area, action));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn detailed_lines(
    week_label: &str,
    month_label: &str,
    year_label: &str,
    weekly_miles: f32,
    monthly_miles: f32,
    yearly_miles: f32,
    weekly_elevation: i32,
    monthly_elevation: i32,
    yearly_elevation: i32,
    monthly_1000_days: usize,
    streak_message: &str,
) -> Vec<Line<'static>> {
    let heading = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let value = Style::default().fg(Color::White);

    vec![
        Line::from(Span::styled(format!("This Week — {week_label}"), heading)),
        totals_line(weekly_miles, weekly_elevation, value),
        Line::default(),
        Line::from(Span::styled(format!("This Month — {month_label}"), heading)),
        totals_line(monthly_miles, monthly_elevation, value),
        Line::default(),
        Line::from(Span::styled(format!("This Year — {year_label}"), heading)),
        totals_line(yearly_miles, yearly_elevation, value),
        Line::from(Span::styled(
            format!("1000+ ft days this month: {monthly_1000_days}"),
            Style::default().fg(Color::LightRed),
        )),
        Line::from(Span::styled(
            streak_message.to_string(),
            Style::default().fg(Color::Green),
        )),
    ]
}

#[allow(clippy::too_many_arguments)]
fn compact_lines(
    week_label: &str,
    month_label: &str,
    year_label: &str,
    weekly_miles: f32,
    monthly_miles: f32,
    yearly_miles: f32,
    weekly_elevation: i32,
    monthly_elevation: i32,
    yearly_elevation: i32,
    monthly_1000_days: usize,
    streak_message: &str,
) -> Vec<Line<'static>> {
    let value = Style::default().fg(Color::White);
    vec![
        compact_totals_line(week_label, weekly_miles, weekly_elevation, value),
        compact_totals_line(month_label, monthly_miles, monthly_elevation, value),
        compact_totals_line(year_label, yearly_miles, yearly_elevation, value),
        Line::default(),
        Line::from(Span::styled(
            format!("1000+ ft days this month: {monthly_1000_days}"),
            Style::default().fg(Color::LightRed),
        )),
        Line::from(Span::styled(
            streak_message.to_string(),
            Style::default().fg(Color::Green),
        )),
    ]
}

fn totals_line(miles: f32, elevation: i32, style: Style) -> Line<'static> {
    Line::from(Span::styled(
        format!("Miles: {miles:.1} mi | Elevation: {elevation} ft"),
        style,
    ))
}

fn compact_totals_line(label: &str, miles: f32, elevation: i32, style: Style) -> Line<'static> {
    Line::from(Span::styled(
        format!("{label}: {miles:.1} mi | {elevation} ft"),
        style,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DailyLog;
    use ratatui::{Terminal, backend::TestBackend};

    fn rendered_text(state: &AppState, date: NaiveDate, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut targets = Vec::new();
        terminal
            .draw(|frame| render_statistics_screen(frame, state, date, &mut targets))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn renders_week_month_year_totals_and_existing_stats() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let mut state = AppState::new();
        state.daily_logs = vec![DailyLog {
            date,
            miles_covered: Some(7.5),
            elevation_gain: Some(1200),
            ..DailyLog::new(date)
        }];

        let text = rendered_text(&state, date, 100, 26);
        assert!(text.contains("This Week"));
        assert!(text.contains("Week 30 (Jul 20–Jul 26)"));
        assert!(text.contains("This Month — July 2026"));
        assert!(text.contains("This Year — 2026"));
        assert!(text.contains("Miles: 7.5 mi | Elevation: 1200 ft"));
        assert!(text.contains("1000+ ft days this month: 1"));
    }

    #[test]
    fn compact_empty_screen_keeps_all_periods_and_zero_values() {
        let date = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let text = rendered_text(&AppState::new(), date, 60, 20);

        assert!(text.contains("Week 30"));
        assert!(text.contains("July 2026"));
        assert!(text.contains("2026: 0.0 mi | 0 ft"));
        assert!(text.contains("1000+ ft days this month: 0"));
        assert!(text.contains("Esc: Startup"));
    }

    #[test]
    fn footer_registers_back_and_quit_targets() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut targets = Vec::new();
        let date = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();

        terminal
            .draw(|frame| {
                render_statistics_screen(frame, &AppState::new(), date, &mut targets);
            })
            .unwrap();

        assert!(
            targets
                .iter()
                .any(|target| target.action == ClickAction::BackToStartup)
        );
        assert!(
            targets
                .iter()
                .any(|target| target.action == ClickAction::Quit)
        );
    }
}
