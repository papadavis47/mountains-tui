/// Elevation statistics calculation utilities
///
/// This module provides functions to calculate various elevation-based statistics
/// for the Mountains Training Log application.

use chrono::{Datelike, Local};
use crate::models::DailyLog;

const ELEVATION_THRESHOLD: i32 = 1000;

/// Counts the number of days in the current calendar month with elevation >= 1000 feet
///
/// This function:
/// 1. Gets the current month and year
/// 2. Filters logs to only include current month
/// 3. Counts days where elevation_gain >= 1000
///
/// Returns a count from 0 to 31 depending on how many qualifying days exist
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

/// Calculates the total elevation gain for the current calendar year
///
/// This function:
/// 1. Gets the current year
/// 2. Filters logs to only include current year
/// 3. Sums ALL elevation gain values (not just 1000+ days)
///
/// Returns the total feet of elevation gain for the year
pub fn calculate_yearly_elevation(logs: &[DailyLog]) -> i32 {
    let now = Local::now().date_naive();
    let current_year = now.year();

    logs.iter()
        .filter(|log| log.date.year() == current_year)
        .filter_map(|log| log.elevation_gain)
        .sum()
}

/// Calculates the current active streak of consecutive days with 1000+ feet elevation
///
/// This function:
/// 1. Finds the most recent day with ANY data (not necessarily 1000+ feet)
/// 2. Counts backward from that day, looking for consecutive days with 1000+ feet
/// 3. Missing data breaks the streak (no entry = broken streak)
/// 4. Returns Some(count) only if streak is active (extends to most recent logged day)
/// 5. Returns None if no active streak (or streak is less than 2 days)
///
/// A streak must be at least 2 days to count.
pub fn calculate_current_streak(logs: &[DailyLog]) -> Option<usize> {
    if logs.is_empty() {
        return None;
    }

    // Sort logs by date descending (most recent first)
    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by(|a, b| b.date.cmp(&a.date));

    // Find the most recent day with data
    let most_recent_date = sorted_logs.first()?.date;

    // Check if the most recent day has 1000+ feet
    let most_recent_has_threshold = sorted_logs
        .first()?
        .elevation_gain
        .unwrap_or(0) >= ELEVATION_THRESHOLD;

    if !most_recent_has_threshold {
        // No active streak
        return None;
    }

    // Count consecutive days with 1000+ feet, working backward from most recent
    let mut streak_count = 0;
    let mut current_date = most_recent_date;

    loop {
        // Find the log for current_date
        if let Some(log) = sorted_logs.iter().find(|log| log.date == current_date) {
            // Check if this day has 1000+ feet
            if log.elevation_gain.unwrap_or(0) >= ELEVATION_THRESHOLD {
                streak_count += 1;
                // Move to previous day
                current_date = current_date.pred_opt()?;
            } else {
                // Day exists but doesn't meet threshold - streak broken
                break;
            }
        } else {
            // No data for this day - streak broken
            break;
        }
    }

    // Only return streak if it's 2 or more days
    if streak_count >= 2 {
        Some(streak_count)
    } else {
        None
    }
}

/// Gets the display message for the current elevation streak
///
/// Returns either:
/// - "You currently have X consecutive days of 1000+ vert!" (if active streak >= 2)
/// - "Think about starting a streak of 1000+ feet of gain." (if no active streak)
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
