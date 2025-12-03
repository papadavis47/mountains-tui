use chrono::{Datelike, Local};
use crate::models::DailyLog;

const ELEVATION_THRESHOLD: i32 = 1000;

pub fn count_monthly_1000_days(logs: &[DailyLog]) -> usize {
    let now = Local::now().date_naive();
    let current_year = now.year();
    let current_month = now.month();

    logs.iter()
        .filter(|log| {
            log.date.year() == current_year
                && log.date.month() == current_month
                && log.elevation_gain.unwrap_or(0) >= ELEVATION_THRESHOLD
        })
        .count()
}

pub fn calculate_yearly_elevation(logs: &[DailyLog]) -> i32 {
    let now = Local::now().date_naive();
    let current_year = now.year();

    logs.iter()
        .filter(|log| log.date.year() == current_year)
        .filter_map(|log| log.elevation_gain)
        .sum()
}

/// Returns streak count only if active (extends to most recent logged day)
pub fn calculate_current_streak(logs: &[DailyLog]) -> Option<usize> {
    if logs.is_empty() {
        return None;
    }

    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by(|a, b| b.date.cmp(&a.date));

    let most_recent_date = sorted_logs.first()?.date;

    let most_recent_has_threshold = sorted_logs
        .first()?
        .elevation_gain
        .unwrap_or(0) >= ELEVATION_THRESHOLD;

    if !most_recent_has_threshold {
        return None;
    }

    let mut streak_count = 0;
    let mut current_date = most_recent_date;

    while let Some(log) = sorted_logs.iter().find(|log| log.date == current_date) {
        if log.elevation_gain.unwrap_or(0) >= ELEVATION_THRESHOLD {
            streak_count += 1;
            current_date = match current_date.pred_opt() {
                Some(date) => date,
                None => break,
            };
        } else {
            break;
        }
    }

    if streak_count >= 2 {
        Some(streak_count)
    } else {
        None
    }
}

pub fn get_streak_message(logs: &[DailyLog]) -> String {
    if let Some(streak_count) = calculate_current_streak(logs) {
        format!("You currently have {} consecutive days of 1000+ vert!", streak_count)
    } else {
        "Think about starting a streak of 1000+ feet of gain.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_count_monthly_1000_days() {
        let now = Local::now().date_naive();
        let current_year = now.year();
        let current_month = now.month();

        let logs = vec![
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap(),
                elevation_gain: Some(1200),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, current_month, 2).unwrap(),
                elevation_gain: Some(800),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, current_month, 2).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, current_month, 3).unwrap(),
                elevation_gain: Some(1500),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, current_month, 3).unwrap())
            },
        ];

        assert_eq!(count_monthly_1000_days(&logs), 2);
    }

    #[test]
    fn test_calculate_yearly_elevation() {
        let now = Local::now().date_naive();
        let current_year = now.year();

        let logs = vec![
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
                elevation_gain: Some(1200),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap(),
                elevation_gain: Some(800),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year - 1, 1, 1).unwrap(),
                elevation_gain: Some(2000),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year - 1, 1, 1).unwrap())
            },
        ];

        assert_eq!(calculate_yearly_elevation(&logs), 2000); // Only current year
    }

    #[test]
    fn test_calculate_current_streak() {
        // Create a streak of 3 consecutive days ending today
        let today = Local::now().date_naive();
        let yesterday = today.pred_opt().unwrap();
        let two_days_ago = yesterday.pred_opt().unwrap();

        let logs = vec![
            DailyLog {
                date: today,
                elevation_gain: Some(1200),
                ..DailyLog::new(today)
            },
            DailyLog {
                date: yesterday,
                elevation_gain: Some(1500),
                ..DailyLog::new(yesterday)
            },
            DailyLog {
                date: two_days_ago,
                elevation_gain: Some(1100),
                ..DailyLog::new(two_days_ago)
            },
        ];

        assert_eq!(calculate_current_streak(&logs), Some(3));
    }
}
