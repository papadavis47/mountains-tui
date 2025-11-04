/// Event handling logic for the Mountains Food Tracker
///
/// This module contains all keyboard and input event handling functions.
/// By separating event handling from the main application logic, we make
/// the code more modular and easier to understand.
///
/// The main principle here is that each event handler focuses on one specific
/// type of user interaction, making the code easier to maintain and debug.
use crossterm::event::KeyCode;

use crate::db_manager::DbManager;
use crate::file_manager::FileManager;
use crate::models::{AppScreen, AppState, FoodEntry};

/// Handles keyboard input for text editing operations
///
/// This struct manages the state needed for text input, including:
/// - The text buffer being edited
/// - Current cursor position within the text
/// - Different input modes (text vs numeric)
///
/// In Rust, structs group related data together. This is more organized
/// than passing individual parameters around.
pub struct InputHandler {
    pub input_buffer: String,
    pub cursor_position: usize,
}

impl InputHandler {
    /// Creates a new InputHandler with empty buffer
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            cursor_position: 0,
        }
    }

    /// Clears the input buffer and resets cursor position
    ///
    /// This is called when canceling input or after successful submission.
    pub fn clear(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }

    /// Sets the input buffer to a specific value and positions cursor at the end
    ///
    /// Used when pre-filling input fields (like editing existing food entries).
    pub fn set_input(&mut self, text: String) {
        self.cursor_position = text.len();
        self.input_buffer = text;
    }

    /// Inserts a character at the current cursor position
    ///
    /// This function handles text insertion while maintaining cursor position.
    /// It works by either appending to the end or inserting in the middle.
    pub fn insert_char(&mut self, c: char) {
        if self.cursor_position >= self.input_buffer.len() {
            // Cursor is at the end - just append
            self.input_buffer.push(c);
        } else {
            // Cursor is in the middle - insert at position
            self.input_buffer.insert(self.cursor_position, c);
        }
        // Move cursor forward after insertion
        self.cursor_position += 1;
    }

    /// Deletes the character before the cursor (backspace behavior)
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            if self.cursor_position < self.input_buffer.len() {
                self.input_buffer.remove(self.cursor_position);
            }
        }
    }

    /// Deletes the character at the cursor (delete key behavior)
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Moves cursor left (with bounds checking)
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Moves cursor right (with bounds checking)
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    /// Moves cursor to the beginning of the input
    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Moves cursor to the end of the input
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    /// Handles text input key events
    ///
    /// This method processes keyboard input for text editing.
    /// It returns true if the key was handled, false otherwise.
    pub fn handle_text_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                self.insert_char(c);
                true
            }
            KeyCode::Backspace => {
                self.delete_char();
                true
            }
            KeyCode::Delete => {
                self.delete_char_forward();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.move_cursor_end();
                true
            }
            _ => false, // Key not handled
        }
    }

    /// Handles numeric input key events (for weight/waist measurements)
    ///
    /// This is similar to handle_text_input but only allows numeric characters
    /// and decimal points, which is appropriate for measurement input.
    pub fn handle_numeric_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                // Only allow digits and decimal point for numeric input
                if c.is_ascii_digit() || c == '.' {
                    self.insert_char(c);
                }
                true
            }
            KeyCode::Backspace => {
                self.delete_char();
                true
            }
            KeyCode::Delete => {
                self.delete_char_forward();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.move_cursor_end();
                true
            }
            _ => false,
        }
    }

    /// Handles integer-only input key events (for elevation gain)
    ///
    /// This is similar to handle_numeric_input but only allows digits (no decimal point).
    pub fn handle_integer_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                // Only allow digits for integer input
                if c.is_ascii_digit() {
                    self.insert_char(c);
                }
                true
            }
            KeyCode::Backspace => {
                self.delete_char();
                true
            }
            KeyCode::Delete => {
                self.delete_char_forward();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.move_cursor_end();
                true
            }
            _ => false,
        }
    }

    /// Handles multi-line text input key events (for notes editing)
    ///
    /// This is similar to handle_text_input but adds support for:
    /// - Up/Down arrow keys for multi-line navigation
    /// - Better handling of multi-line cursor movement
    /// - Ctrl+J to insert newlines (since Enter saves the notes)
    pub fn handle_multiline_text_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                self.insert_char(c);
                true
            }
            KeyCode::Backspace => {
                self.delete_char();
                true
            }
            KeyCode::Delete => {
                self.delete_char_forward();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Up => {
                self.move_cursor_up();
                true
            }
            KeyCode::Down => {
                self.move_cursor_down();
                true
            }
            KeyCode::Home => {
                self.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.move_cursor_end();
                true
            }
            _ => false, // Key not handled
        }
    }

    /// Handles special key combinations for multi-line input
    ///
    /// Returns true if a special combination was handled, false otherwise.
    /// This allows the caller to handle the key normally if no special action was taken.
    pub fn handle_multiline_special_keys(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> bool {
        use crossterm::event::KeyModifiers;

        match (key, modifiers) {
            // Ctrl+J inserts a newline (since Enter saves the text)
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                self.insert_char('\n');
                true
            }
            _ => false,
        }
    }

    /// Moves cursor up one line in multi-line text
    ///
    /// This method handles vertical cursor movement by finding the current line
    /// and attempting to maintain the same column position on the previous line.
    pub fn move_cursor_up(&mut self) {
        if self.cursor_position == 0 {
            return; // Already at the beginning
        }

        // Find the start of the current line
        let text_up_to_cursor = &self.input_buffer[..self.cursor_position];
        let mut current_line_start = 0;
        let mut prev_line_start = 0;

        for (i, ch) in text_up_to_cursor.char_indices() {
            if ch == '\n' {
                prev_line_start = current_line_start;
                current_line_start = i + 1;
            }
        }

        // Calculate column position on current line
        let current_column = self.cursor_position - current_line_start;

        // If we're not on the first line, move to the previous line
        if current_line_start > 0 {
            // Find the length of the previous line
            let prev_line_end = current_line_start - 1; // -1 to skip the newline
            let prev_line_length = prev_line_end - prev_line_start;

            // Move to the same column on the previous line, or end of line if shorter
            let new_column = std::cmp::min(current_column, prev_line_length);
            self.cursor_position = prev_line_start + new_column;
        } else {
            // Already on first line, move to beginning
            self.cursor_position = 0;
        }
    }

    /// Moves cursor down one line in multi-line text
    ///
    /// Similar to move_cursor_up but moves to the next line.
    pub fn move_cursor_down(&mut self) {
        let total_length = self.input_buffer.len();
        if self.cursor_position >= total_length {
            return; // Already at the end
        }

        // Find the start of the current line and next line
        let text_up_to_cursor = &self.input_buffer[..self.cursor_position];
        let mut current_line_start = 0;

        for (i, ch) in text_up_to_cursor.char_indices() {
            if ch == '\n' {
                current_line_start = i + 1;
            }
        }

        // Calculate column position on current line
        let current_column = self.cursor_position - current_line_start;

        // Find the start of the next line
        let remaining_text = &self.input_buffer[self.cursor_position..];
        if let Some(newline_pos) = remaining_text.find('\n') {
            let next_line_start = self.cursor_position + newline_pos + 1;

            // Find the end of the next line
            let text_from_next_line = &self.input_buffer[next_line_start..];
            let next_line_end = if let Some(next_newline) = text_from_next_line.find('\n') {
                next_line_start + next_newline
            } else {
                total_length // End of text
            };

            let next_line_length = next_line_end - next_line_start;
            let new_column = std::cmp::min(current_column, next_line_length);
            self.cursor_position = next_line_start + new_column;
        } else {
            // No next line, move to end of text
            self.cursor_position = total_length;
        }
    }
}

/// Navigation handler for managing list selections
///
/// This struct manages selection state for different lists in the application.
/// It provides a clean interface for moving up/down in lists while handling
/// wraparound behavior.
pub struct NavigationHandler;

impl NavigationHandler {
    /// Moves selection down in a list with wraparound
    ///
    /// If at the bottom, wraps to the top. This provides a better user experience
    /// than stopping at the bottom of lists.
    pub fn move_selection_down(current_index: Option<usize>, list_len: usize) -> Option<usize> {
        if list_len == 0 {
            return None;
        }

        match current_index {
            Some(i) => {
                if i >= list_len - 1 {
                    Some(0) // Wrap to top
                } else {
                    Some(i + 1)
                }
            }
            None => Some(0), // Start at first item
        }
    }

    /// Moves selection up in a list with wraparound
    ///
    /// If at the top, wraps to the bottom.
    pub fn move_selection_up(current_index: Option<usize>, list_len: usize) -> Option<usize> {
        if list_len == 0 {
            return None;
        }

        match current_index {
            Some(i) => {
                if i == 0 {
                    Some(list_len - 1) // Wrap to bottom
                } else {
                    Some(i - 1)
                }
            }
            None => Some(0), // Start at first item
        }
    }
}

/// Action handler for processing user actions
///
/// This struct contains methods for handling complex user actions that involve
/// multiple operations, like saving food entries or deleting items.
pub struct ActionHandler;

impl ActionHandler {
    /// Saves a new food entry to the daily log
    ///
    /// This function:
    /// 1. Creates a new FoodEntry from the input
    /// 2. Gets or creates the daily log for the selected date
    /// 3. Adds the entry to the log
    /// 4. Saves the log to database (with cloud sync)
    /// 5. Optionally saves to markdown file as backup
    ///
    /// The Result<(), anyhow::Error> return type allows the caller to handle
    /// any errors that might occur during database operations.
    pub fn save_food_entry(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        food_name: String,
        notes: Option<String>,
    ) -> anyhow::Result<()> {
        if !food_name.is_empty() {
            let food_entry = FoodEntry::new(food_name, notes);
            let log = state.get_or_create_daily_log(state.selected_date);
            log.add_food_entry(food_entry);

            // Save to database (primary storage with cloud sync)
            tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;

            // Optionally save to markdown as backup
            let _ = file_manager.save_daily_log(log); // Ignore markdown errors
        }
        Ok(())
    }

    /// Updates an existing food entry
    ///
    /// This function finds the food entry by index and updates its name.
    pub fn update_food_entry(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        food_index: usize,
        new_name: String,
    ) -> anyhow::Result<()> {
        if !new_name.is_empty() {
            if let Some(log) = state
                .daily_logs
                .iter_mut()
                .find(|log| log.date == state.selected_date)
            {
                if food_index < log.food_entries.len() {
                    log.food_entries[food_index].name = new_name;
                    tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
                    let _ = file_manager.save_daily_log(log); // Backup to markdown
                }
            }
        }
        Ok(())
    }

    /// Deletes a food entry from the daily log
    ///
    /// This function removes a food entry by index and updates the database.
    /// It also handles updating the selection state if needed.
    pub fn delete_food_entry(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        food_index: usize,
    ) -> anyhow::Result<()> {
        if let Some(log) = state
            .daily_logs
            .iter_mut()
            .find(|log| log.date == state.selected_date)
        {
            if food_index < log.food_entries.len() {
                log.remove_food_entry(food_index);
                tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
                let _ = file_manager.save_daily_log(log); // Backup to markdown
            }
        }
        Ok(())
    }

    /// Updates the weight measurement for the selected date
    ///
    /// This function parses the input string as a float and saves it.
    /// Empty input clears the weight (sets it to None).
    pub fn update_weight(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        weight_input: String,
    ) -> anyhow::Result<()> {
        let weight: Option<f32> = if weight_input.is_empty() {
            None
        } else {
            weight_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.weight = weight;
        tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
        let _ = file_manager.save_daily_log(log); // Backup to markdown
        Ok(())
    }

    /// Updates the waist measurement for the selected date
    ///
    /// Similar to update_weight but for waist measurements.
    pub fn update_waist(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        waist_input: String,
    ) -> anyhow::Result<()> {
        let waist: Option<f32> = if waist_input.is_empty() {
            None
        } else {
            waist_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.waist = waist;
        tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
        let _ = file_manager.save_daily_log(log); // Backup to markdown
        Ok(())
    }

    /// Handles the Enter key press on the home screen
    ///
    /// This function either creates a new daily log for today or navigates
    /// to an existing daily log based on the user's selection.
    pub fn handle_home_enter(state: &mut AppState, selected_index: Option<usize>) {
        if state.daily_logs.is_empty() {
            // No logs exist - create today's log
            state.selected_date = chrono::Local::now().date_naive();
        } else if let Some(index) = selected_index {
            if index < state.daily_logs.len() {
                state.selected_date = state.daily_logs[index].date;
            }
        }
        state.current_screen = AppScreen::DailyView;
    }

    /// Prepares the edit food screen with the current food entry data
    ///
    /// This function finds the selected food entry and pre-fills the input
    /// buffer with its current name for editing.
    pub fn start_edit_food(state: &AppState, food_index: usize) -> Option<String> {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if food_index < log.food_entries.len() {
                return Some(log.food_entries[food_index].name.clone());
            }
        }
        None
    }

    /// Prepares the edit weight screen with the current weight value
    ///
    /// Returns the current weight as a string, or empty string if not set.
    pub fn start_edit_weight(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(weight) = log.weight {
                return weight.to_string();
            }
        }
        String::new()
    }

    /// Prepares the edit waist screen with the current waist value
    ///
    /// Returns the current waist measurement as a string, or empty string if not set.
    pub fn start_edit_waist(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(waist) = log.waist {
                return waist.to_string();
            }
        }
        String::new()
    }

    /// Updates the strength & mobility exercises for the selected date
    ///
    /// This function saves the strength & mobility text to the daily log.
    /// Empty input clears the strength & mobility field (sets it to None).
    pub fn update_strength_mobility(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        sm_input: String,
    ) -> anyhow::Result<()> {
        let strength_mobility: Option<String> = if sm_input.trim().is_empty() {
            None
        } else {
            Some(sm_input)
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.strength_mobility = strength_mobility;
        tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
        let _ = file_manager.save_daily_log(log); // Backup to markdown
        Ok(())
    }

    /// Prepares the edit strength & mobility screen with the current value
    ///
    /// Returns the current strength & mobility text as a string, or empty string if not set.
    pub fn start_edit_strength_mobility(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(sm) = &log.strength_mobility {
                return sm.clone();
            }
        }
        String::new()
    }

    /// Updates the daily notes for the selected date
    ///
    /// This function saves the notes text to the daily log.
    /// Empty input clears the notes (sets it to None).
    pub fn update_notes(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        notes_input: String,
    ) -> anyhow::Result<()> {
        let notes: Option<String> = if notes_input.trim().is_empty() {
            None
        } else {
            Some(notes_input)
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.notes = notes;
        tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
        let _ = file_manager.save_daily_log(log); // Backup to markdown
        Ok(())
    }

    /// Prepares the edit notes screen with the current notes value
    ///
    /// Returns the current notes as a string, or empty string if not set.
    pub fn start_edit_notes(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(notes) = &log.notes {
                return notes.clone();
            }
        }
        String::new()
    }

    /// Updates the miles covered for the selected date
    ///
    /// This function parses the input string as a float and saves it.
    /// Empty input clears the miles (sets it to None).
    pub fn update_miles(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        miles_input: String,
    ) -> anyhow::Result<()> {
        let miles: Option<f32> = if miles_input.is_empty() {
            None
        } else {
            miles_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.miles_covered = miles;
        tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
        let _ = file_manager.save_daily_log(log); // Backup to markdown
        Ok(())
    }

    /// Updates the elevation gain for the selected date
    ///
    /// This function parses the input string as an integer and saves it.
    /// Empty input clears the elevation (sets it to None).
    pub fn update_elevation(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        elevation_input: String,
    ) -> anyhow::Result<()> {
        let elevation: Option<i32> = if elevation_input.is_empty() {
            None
        } else {
            elevation_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.elevation_gain = elevation;
        tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
        let _ = file_manager.save_daily_log(log); // Backup to markdown
        Ok(())
    }

    /// Prepares the edit miles screen with the current miles value
    ///
    /// Returns the current miles as a string, or empty string if not set.
    pub fn start_edit_miles(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(miles) = log.miles_covered {
                return miles.to_string();
            }
        }
        String::new()
    }

    /// Prepares the edit elevation screen with the current elevation value
    ///
    /// Returns the current elevation as a string, or empty string if not set.
    pub fn start_edit_elevation(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(elevation) = log.elevation_gain {
                return elevation.to_string();
            }
        }
        String::new()
    }

    /// Saves a new sokay entry to the daily log
    ///
    /// This function adds a sokay entry (unhealthy food choice) to the current day's log.
    pub fn save_sokay_entry(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        sokay_text: String,
    ) -> anyhow::Result<()> {
        if !sokay_text.is_empty() {
            let log = state.get_or_create_daily_log(state.selected_date);
            log.add_sokay_entry(sokay_text);
            tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
            let _ = file_manager.save_daily_log(log); // Backup to markdown
        }
        Ok(())
    }

    /// Updates an existing sokay entry
    ///
    /// This function finds the sokay entry by index and updates its text.
    pub fn update_sokay_entry(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        sokay_index: usize,
        new_text: String,
    ) -> anyhow::Result<()> {
        if !new_text.is_empty() {
            if let Some(log) = state
                .daily_logs
                .iter_mut()
                .find(|log| log.date == state.selected_date)
            {
                if sokay_index < log.sokay_entries.len() {
                    log.sokay_entries[sokay_index] = new_text;
                    tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
                    let _ = file_manager.save_daily_log(log); // Backup to markdown
                }
            }
        }
        Ok(())
    }

    /// Deletes a sokay entry from the daily log
    ///
    /// This function removes a sokay entry by index and updates the database.
    pub fn delete_sokay_entry(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        sokay_index: usize,
    ) -> anyhow::Result<()> {
        if let Some(log) = state
            .daily_logs
            .iter_mut()
            .find(|log| log.date == state.selected_date)
        {
            if sokay_index < log.sokay_entries.len() {
                log.remove_sokay_entry(sokay_index);
                tokio::runtime::Handle::current().block_on(db_manager.save_daily_log(log))?;
                let _ = file_manager.save_daily_log(log); // Backup to markdown
            }
        }
        Ok(())
    }

    /// Prepares the edit sokay screen with the current sokay entry text
    ///
    /// Returns the current sokay entry text, or None if index is invalid.
    pub fn start_edit_sokay(state: &AppState, sokay_index: usize) -> Option<String> {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if sokay_index < log.sokay_entries.len() {
                return Some(log.sokay_entries[sokay_index].clone());
            }
        }
        None
    }

    /// Calculates cumulative sokay count up to and including the specified date
    ///
    /// This function sums all sokay entries from all logs dated up to the specified date.
    pub fn calculate_cumulative_sokay(state: &AppState, up_to_date: chrono::NaiveDate) -> usize {
        state
            .daily_logs
            .iter()
            .filter(|log| log.date <= up_to_date)
            .map(|log| log.sokay_entries.len())
            .sum()
    }
}
