/// Data models for the Mountains Food Tracker
///
/// This module contains all the core data structures used throughout the application.
/// These models represent the business domain of food tracking and body measurements.
///
/// The structures here use Rust's type system to ensure data integrity and
/// provide a clear interface for the rest of the application.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Represents a single food entry in a daily log
///
/// Each food entry has:
/// - name: Required name of the food item
/// - notes: Optional additional notes about the food
///
/// The derive attributes provide useful functionality:
/// - Debug: Allows printing the struct for debugging
/// - Clone: Allows creating copies of the struct
/// - Serialize/Deserialize: Enables saving/loading from files via serde
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodEntry {
    /// The name of the food item (e.g., "Chicken Salad", "Apple")
    pub name: String,
    /// Optional notes about the food (e.g., preparation method, portion size)
    pub notes: Option<String>,
}

impl FoodEntry {
    /// Creates a new food entry with the given name and optional notes
    ///
    /// This constructor function provides a clean way to create FoodEntry instances.
    /// Using Option<String> for notes allows for optional data - None means no notes.
    pub fn new(name: String, notes: Option<String>) -> Self {
        Self { name, notes }
    }
}

/// Represents a complete daily log with food entries and measurements
///
/// This is the core data structure that represents one day's worth of data.
/// It includes food tracking and body measurements for health monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLog {
    /// The date this log represents
    pub date: NaiveDate,
    /// List of food entries for this day
    pub food_entries: Vec<FoodEntry>,
    /// Optional weight measurement in pounds
    pub weight: Option<f32>,
    /// Optional waist measurement in inches
    pub waist: Option<f32>,
    /// Optional miles covered (walking/hiking/running)
    pub miles_covered: Option<f32>,
    /// Optional elevation gain in feet (integer)
    pub elevation_gain: Option<i32>,
    /// List of "sokay" entries (unhealthy food choices)
    pub sokay_entries: Vec<String>,
    /// Optional strength and mobility exercises for the day
    pub strength_mobility: Option<String>,
    /// Optional daily notes or observations
    pub notes: Option<String>,
}

impl DailyLog {
    /// Creates a new daily log for the specified date
    ///
    /// The log starts empty - food entries and measurements can be added later.
    /// Using Vec::new() creates an empty vector that can grow as needed.
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            food_entries: Vec::new(),
            weight: None,
            waist: None,
            miles_covered: None,
            elevation_gain: None,
            sokay_entries: Vec::new(),
            strength_mobility: None,
            notes: None,
        }
    }

    /// Adds a food entry to this daily log
    ///
    /// The `&mut self` parameter means this method needs mutable access
    /// to the struct to modify the food_entries vector.
    pub fn add_food_entry(&mut self, entry: FoodEntry) {
        self.food_entries.push(entry);
    }

    /// Removes a food entry at the specified index
    ///
    /// This method includes bounds checking to prevent panics.
    /// If the index is out of bounds, the method does nothing.
    pub fn remove_food_entry(&mut self, index: usize) {
        if index < self.food_entries.len() {
            self.food_entries.remove(index);
        }
    }

    /// Adds a sokay entry to this daily log
    ///
    /// Sokay entries track unhealthy food choices for accountability.
    pub fn add_sokay_entry(&mut self, entry: String) {
        self.sokay_entries.push(entry);
    }

    /// Removes a sokay entry at the specified index
    ///
    /// This method includes bounds checking to prevent panics.
    /// If the index is out of bounds, the method does nothing.
    pub fn remove_sokay_entry(&mut self, index: usize) {
        if index < self.sokay_entries.len() {
            self.sokay_entries.remove(index);
        }
    }
}

/// Represents the different screens/views in the application
///
/// This enum uses Rust's pattern matching capabilities to manage application state.
/// Each variant represents a different screen the user can be on.
///
/// The EditFood variant includes a usize parameter - this is the index of the
/// food entry being edited. This is an example of enums with data.
#[derive(Debug, Clone)]
pub enum AppScreen {
    /// The main screen showing all daily logs
    Home,
    /// Viewing a specific day's food entries and measurements
    DailyView,
    /// Adding a new food entry
    AddFood,
    /// Editing an existing food entry (parameter is the index of the entry)
    EditFood(usize),
    /// Editing weight measurement
    EditWeight,
    /// Editing waist measurement
    EditWaist,
    /// Editing miles covered
    EditMiles,
    /// Editing elevation gain
    EditElevation,
    /// Viewing sokay entries list
    SokayView,
    /// Adding a new sokay entry
    AddSokay,
    /// Editing an existing sokay entry (parameter is the index of the entry)
    EditSokay(usize),
    /// Editing strength and mobility exercises
    EditStrengthMobility,
    /// Editing daily notes
    EditNotes,
    /// Confirming deletion of an entire day's log
    ConfirmDeleteDay,
}

/// Manages the overall application state
///
/// This struct holds all the data needed to run the application:
/// - Which screen is currently displayed
/// - Which date is selected
/// - All loaded daily logs
/// - UI selection state
///
/// The Debug derive allows printing this struct for debugging purposes.
#[derive(Debug)]
pub struct AppState {
    /// Current screen being displayed to the user
    pub current_screen: AppScreen,
    /// The date currently selected for viewing/editing
    pub selected_date: NaiveDate,
    /// All daily logs loaded from disk, sorted newest first
    pub daily_logs: Vec<DailyLog>,
}

impl AppState {
    /// Creates a new application state with default values
    ///
    /// The application starts on the Home screen with today's date selected.
    /// chrono::Local::now().date_naive() gets the current date in the local timezone.
    pub fn new() -> Self {
        Self {
            current_screen: AppScreen::Home,
            selected_date: chrono::Local::now().date_naive(),
            daily_logs: Vec::new(),
        }
    }

    /// Gets a mutable reference to the daily log for the specified date,
    /// creating it if it doesn't exist
    ///
    /// This method is crucial for the application's functionality. It:
    /// 1. Looks for an existing log for the date
    /// 2. If found, returns a mutable reference to it
    /// 3. If not found, creates a new log, adds it to the collection, and returns it
    ///
    /// The return type `&mut DailyLog` is a mutable reference - this allows
    /// the caller to modify the log without taking ownership of it.
    pub fn get_or_create_daily_log(&mut self, date: NaiveDate) -> &mut DailyLog {
        // Try to find an existing log for this date
        if let Some(pos) = self.daily_logs.iter().position(|log| log.date == date) {
            // Found it - return a mutable reference
            &mut self.daily_logs[pos]
        } else {
            // Not found - create a new log
            self.daily_logs.push(DailyLog::new(date));
            // Sort to keep newest logs first for better UX
            self.daily_logs.sort_by(|a, b| b.date.cmp(&a.date));
            // Find and return the newly created log
            // unwrap() is safe here because we just added the log
            self.daily_logs
                .iter_mut()
                .find(|log| log.date == date)
                .unwrap()
        }
    }

    /// Gets a read-only reference to the daily log for the specified date
    ///
    /// This method returns an Option:
    /// - Some(&DailyLog) if a log exists for the date
    /// - None if no log exists for the date
    ///
    /// Using Option<&DailyLog> instead of creating a log makes this method
    /// non-mutating and allows the caller to handle the "not found" case appropriately.
    pub fn get_daily_log(&self, date: NaiveDate) -> Option<&DailyLog> {
        self.daily_logs.iter().find(|log| log.date == date)
    }
}

