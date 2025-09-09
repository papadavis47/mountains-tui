use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodEntry {
    pub name: String,
    pub notes: Option<String>,
}

impl FoodEntry {
    pub fn new(name: String, notes: Option<String>) -> Self {
        Self { name, notes }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLog {
    pub date: NaiveDate,
    pub food_entries: Vec<FoodEntry>,
    pub weight: Option<f32>,
    pub waist: Option<f32>,
    pub notes: Option<String>,
}

impl DailyLog {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            food_entries: Vec::new(),
            weight: None,
            waist: None,
            notes: None,
        }
    }

    pub fn add_food_entry(&mut self, entry: FoodEntry) {
        self.food_entries.push(entry);
    }

    pub fn remove_food_entry(&mut self, index: usize) {
        if index < self.food_entries.len() {
            self.food_entries.remove(index);
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppScreen {
    Home,
    DailyView,
    AddFood,
    EditFood(usize),
    AddMeasurements,
    EditNotes,
}

#[derive(Debug)]
pub struct AppState {
    pub current_screen: AppScreen,
    pub selected_date: NaiveDate,
    pub daily_logs: Vec<DailyLog>,
    pub selected_index: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: AppScreen::Home,
            selected_date: chrono::Local::now().date_naive(),
            daily_logs: Vec::new(),
            selected_index: 0,
        }
    }

    pub fn get_or_create_daily_log(&mut self, date: NaiveDate) -> &mut DailyLog {
        if let Some(pos) = self.daily_logs.iter().position(|log| log.date == date) {
            &mut self.daily_logs[pos]
        } else {
            self.daily_logs.push(DailyLog::new(date));
            self.daily_logs.sort_by(|a, b| b.date.cmp(&a.date)); // Sort newest first
            self.daily_logs.iter_mut().find(|log| log.date == date).unwrap()
        }
    }

    pub fn get_daily_log(&self, date: NaiveDate) -> Option<&DailyLog> {
        self.daily_logs.iter().find(|log| log.date == date)
    }
}
