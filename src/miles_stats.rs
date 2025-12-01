use chrono::{Datelike, Local};
use crate::models::DailyLog;

pub fn calculate_yearly_miles(logs: &[DailyLog]) -> f32 {
    let now = Local::now().date_naive();
    let current_year = now.year();

    let total: f32 = logs.iter()
        .filter(|log| log.date.year() == current_year)
        .filter_map(|log| log.miles_covered)
        .sum();

    (total * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, NaiveDate};

    #[test]
    fn test_calculate_yearly_miles() {
        let now = Local::now().date_naive();
        let current_year = now.year();

        let logs = vec![
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
                miles_covered: Some(5.5),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap(),
                miles_covered: Some(3.2),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year - 1, 1, 1).unwrap(),
                miles_covered: Some(10.0),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year - 1, 1, 1).unwrap())
            },
        ];

        assert_eq!(calculate_yearly_miles(&logs), 8.7); // Only current year
    }

    #[test]
    fn test_calculate_yearly_miles_empty() {
        let logs: Vec<DailyLog> = vec![];
        assert_eq!(calculate_yearly_miles(&logs), 0.0);
    }

    #[test]
    fn test_calculate_yearly_miles_none_values() {
        let now = Local::now().date_naive();
        let current_year = now.year();

        let logs = vec![
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
                miles_covered: None,
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap(),
                miles_covered: Some(5.0),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap())
            },
        ];

        assert_eq!(calculate_yearly_miles(&logs), 5.0);
    }

    #[test]
    fn test_calculate_yearly_miles_rounding() {
        let now = Local::now().date_naive();
        let current_year = now.year();

        let logs = vec![
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
                miles_covered: Some(7.64),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap())
            },
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap(),
                miles_covered: Some(30.476),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 2, 1).unwrap())
            },
        ];

        // 7.64 + 30.476 = 38.116, rounded to 1 decimal = 38.1
        assert_eq!(calculate_yearly_miles(&logs), 38.1);
    }

    #[test]
    fn test_calculate_yearly_miles_rounding_up() {
        let now = Local::now().date_naive();
        let current_year = now.year();

        let logs = vec![
            DailyLog {
                date: NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
                miles_covered: Some(7.65),
                ..DailyLog::new(NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap())
            },
        ];

        // 7.65 rounded to 1 decimal = 7.7 (rounds up)
        assert_eq!(calculate_yearly_miles(&logs), 7.7);
    }
}
