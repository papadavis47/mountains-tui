use crate::db_manager::DbManager;
use crate::file_manager::FileManager;
use crate::models::{
    AppScreen, AppState, DailyLog, FocusedSection, FoodEntry, MeasurementField, RunningField,
};
use crossterm::event::KeyCode;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InputHandler {
    pub input_buffer: String,
    pub cursor_position: usize,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            cursor_position: 0,
        }
    }

    pub fn clear(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }

    pub fn set_input(&mut self, text: String) {
        self.cursor_position = text.len();
        self.input_buffer = text;
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_position >= self.input_buffer.len() {
            self.input_buffer.push(c);
        } else {
            self.input_buffer.insert(self.cursor_position, c);
        }
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            if self.cursor_position < self.input_buffer.len() {
                self.input_buffer.remove(self.cursor_position);
            }
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

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
            _ => false,
        }
    }

    pub fn handle_numeric_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
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

    pub fn handle_integer_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
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
            _ => false,
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let text_up_to_cursor = &self.input_buffer[..self.cursor_position];
        let mut current_line_start = 0;
        let mut prev_line_start = 0;

        for (i, ch) in text_up_to_cursor.char_indices() {
            if ch == '\n' {
                prev_line_start = current_line_start;
                current_line_start = i + 1;
            }
        }

        let current_column = self.cursor_position - current_line_start;

        if current_line_start > 0 {
            let prev_line_end = current_line_start - 1;
            let prev_line_length = prev_line_end - prev_line_start;

            let new_column = std::cmp::min(current_column, prev_line_length);
            self.cursor_position = prev_line_start + new_column;
        } else {
            self.cursor_position = 0;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let total_length = self.input_buffer.len();
        if self.cursor_position >= total_length {
            return;
        }

        let text_up_to_cursor = &self.input_buffer[..self.cursor_position];
        let mut current_line_start = 0;

        for (i, ch) in text_up_to_cursor.char_indices() {
            if ch == '\n' {
                current_line_start = i + 1;
            }
        }

        let current_column = self.cursor_position - current_line_start;

        let remaining_text = &self.input_buffer[self.cursor_position..];
        if let Some(newline_pos) = remaining_text.find('\n') {
            let next_line_start = self.cursor_position + newline_pos + 1;

            let text_from_next_line = &self.input_buffer[next_line_start..];
            let next_line_end = if let Some(next_newline) = text_from_next_line.find('\n') {
                next_line_start + next_newline
            } else {
                total_length
            };

            let next_line_length = next_line_end - next_line_start;
            let new_column = std::cmp::min(current_column, next_line_length);
            self.cursor_position = next_line_start + new_column;
        } else {
            self.cursor_position = total_length;
        }
    }
}

pub struct SectionNavigator;

impl SectionNavigator {
    pub fn move_focus_down(current: &FocusedSection) -> FocusedSection {
        match current {
            FocusedSection::Measurements { .. } => FocusedSection::Running {
                focused_field: RunningField::Miles,
            },
            FocusedSection::Running { .. } => FocusedSection::FoodItems,
            FocusedSection::FoodItems => FocusedSection::Sokay,
            FocusedSection::Sokay => FocusedSection::StrengthMobility,
            FocusedSection::StrengthMobility => FocusedSection::Notes,
            FocusedSection::Notes => FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            },
        }
    }

    pub fn move_focus_up(current: &FocusedSection) -> FocusedSection {
        match current {
            FocusedSection::Measurements { .. } => FocusedSection::Notes,
            FocusedSection::Running { .. } => FocusedSection::Measurements {
                focused_field: MeasurementField::Weight,
            },
            FocusedSection::FoodItems => FocusedSection::Running {
                focused_field: RunningField::Miles,
            },
            FocusedSection::Sokay => FocusedSection::FoodItems,
            FocusedSection::StrengthMobility => FocusedSection::Sokay,
            FocusedSection::Notes => FocusedSection::StrengthMobility,
        }
    }

    pub fn toggle_internal_focus(current: &FocusedSection) -> FocusedSection {
        match current {
            FocusedSection::Measurements { focused_field } => {
                let new_field = match focused_field {
                    MeasurementField::Weight => MeasurementField::Waist,
                    MeasurementField::Waist => MeasurementField::Weight,
                };
                FocusedSection::Measurements {
                    focused_field: new_field,
                }
            }
            FocusedSection::Running { focused_field } => {
                let new_field = match focused_field {
                    RunningField::Miles => RunningField::Elevation,
                    RunningField::Elevation => RunningField::Miles,
                };
                FocusedSection::Running {
                    focused_field: new_field,
                }
            }
            _ => current.clone(),
        }
    }
}

pub struct NavigationHandler;

impl NavigationHandler {
    pub fn move_selection_down(current_index: Option<usize>, list_len: usize) -> Option<usize> {
        if list_len == 0 {
            return None;
        }

        match current_index {
            Some(i) => {
                if i >= list_len - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            }
            None => Some(0),
        }
    }

    pub fn move_selection_up(current_index: Option<usize>, list_len: usize) -> Option<usize> {
        if list_len == 0 {
            return None;
        }

        match current_index {
            Some(i) => {
                if i == 0 {
                    Some(list_len - 1)
                } else {
                    Some(i - 1)
                }
            }
            None => Some(0),
        }
    }
}

pub struct ActionHandler;

impl ActionHandler {
    pub fn save_food_entry(
        state: &mut AppState,
        food_name: String,
    ) -> Option<DailyLog> {
        if !food_name.is_empty() {
            let food_entry = FoodEntry::new(food_name);
            let log = state.get_or_create_daily_log(state.selected_date);
            log.add_food_entry(food_entry);
            return Some(log.clone());
        }
        None
    }

    /// Background persistence to avoid blocking UI
    pub async fn persist_daily_log(
        db_manager: Arc<RwLock<DbManager>>,
        file_manager: &FileManager,
        log: DailyLog,
    ) {
        let mut db = db_manager.write().await;
        let _ = db.save_daily_log(&log).await;
        let _ = file_manager.save_daily_log(&log);
    }

    pub fn update_food_entry(
        state: &mut AppState,
        food_index: usize,
        new_name: String,
    ) -> Option<DailyLog> {
        if !new_name.is_empty() {
            if let Some(log) = state
                .daily_logs
                .iter_mut()
                .find(|log| log.date == state.selected_date)
            {
                if food_index < log.food_entries.len() {
                    log.food_entries[food_index].name = new_name;
                    return Some(log.clone());
                }
            }
        }
        None
    }

    pub fn delete_food_entry(
        state: &mut AppState,
        food_index: usize,
    ) -> Option<DailyLog> {
        if let Some(log) = state
            .daily_logs
            .iter_mut()
            .find(|log| log.date == state.selected_date)
        {
            if food_index < log.food_entries.len() {
                log.remove_food_entry(food_index);
                return Some(log.clone());
            }
        }
        None
    }

    pub fn update_weight(
        state: &mut AppState,
        weight_input: String,
    ) -> DailyLog {
        let weight: Option<f32> = if weight_input.is_empty() {
            None
        } else {
            weight_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.weight = weight;
        log.clone()
    }

    pub fn update_waist(
        state: &mut AppState,
        waist_input: String,
    ) -> DailyLog {
        let waist: Option<f32> = if waist_input.is_empty() {
            None
        } else {
            waist_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.waist = waist;
        log.clone()
    }

    pub fn handle_home_enter(state: &mut AppState, selected_index: Option<usize>) {
        if let Some(index) = selected_index {
            if index < state.daily_logs.len() {
                state.selected_date = state.daily_logs[index].date;
            }
        } else {
            state.selected_date = chrono::Local::now().date_naive();
        }
        state.current_screen = AppScreen::DailyView;
    }

    pub fn start_edit_food(state: &AppState, food_index: usize) -> Option<String> {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if food_index < log.food_entries.len() {
                return Some(log.food_entries[food_index].name.clone());
            }
        }
        None
    }

    pub fn start_edit_weight(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(weight) = log.weight {
                return weight.to_string();
            }
        }
        String::new()
    }

    pub fn start_edit_waist(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(waist) = log.waist {
                return waist.to_string();
            }
        }
        String::new()
    }

    pub fn update_strength_mobility(
        state: &mut AppState,
        sm_input: String,
    ) -> DailyLog {
        let strength_mobility: Option<String> = if sm_input.trim().is_empty() {
            None
        } else {
            Some(sm_input)
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.strength_mobility = strength_mobility;
        log.clone()
    }

    pub fn start_edit_strength_mobility(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(sm) = &log.strength_mobility {
                return sm.clone();
            }
        }
        String::new()
    }

    pub fn update_notes(
        state: &mut AppState,
        notes_input: String,
    ) -> DailyLog {
        let notes: Option<String> = if notes_input.trim().is_empty() {
            None
        } else {
            Some(notes_input)
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.notes = notes;
        log.clone()
    }

    pub fn start_edit_notes(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(notes) = &log.notes {
                return notes.clone();
            }
        }
        String::new()
    }

    pub fn update_miles(
        state: &mut AppState,
        miles_input: String,
    ) -> DailyLog {
        let miles: Option<f32> = if miles_input.is_empty() {
            None
        } else {
            miles_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.miles_covered = miles;
        log.clone()
    }

    pub fn update_elevation(
        state: &mut AppState,
        elevation_input: String,
    ) -> DailyLog {
        let elevation: Option<i32> = if elevation_input.is_empty() {
            None
        } else {
            elevation_input.parse().ok()
        };
        let log = state.get_or_create_daily_log(state.selected_date);
        log.elevation_gain = elevation;
        log.clone()
    }

    pub fn start_edit_miles(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(miles) = log.miles_covered {
                return miles.to_string();
            }
        }
        String::new()
    }

    pub fn start_edit_elevation(state: &AppState) -> String {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if let Some(elevation) = log.elevation_gain {
                return elevation.to_string();
            }
        }
        String::new()
    }

    pub fn save_sokay_entry(
        state: &mut AppState,
        sokay_text: String,
    ) -> Option<DailyLog> {
        if !sokay_text.is_empty() {
            let log = state.get_or_create_daily_log(state.selected_date);
            log.add_sokay_entry(sokay_text);
            return Some(log.clone());
        }
        None
    }

    pub fn update_sokay_entry(
        state: &mut AppState,
        sokay_index: usize,
        new_text: String,
    ) -> Option<DailyLog> {
        if !new_text.is_empty() {
            if let Some(log) = state
                .daily_logs
                .iter_mut()
                .find(|log| log.date == state.selected_date)
            {
                if sokay_index < log.sokay_entries.len() {
                    log.sokay_entries[sokay_index] = new_text;
                    return Some(log.clone());
                }
            }
        }
        None
    }

    pub fn delete_sokay_entry(
        state: &mut AppState,
        sokay_index: usize,
    ) -> Option<DailyLog> {
        if let Some(log) = state
            .daily_logs
            .iter_mut()
            .find(|log| log.date == state.selected_date)
        {
            if sokay_index < log.sokay_entries.len() {
                log.remove_sokay_entry(sokay_index);
                return Some(log.clone());
            }
        }
        None
    }

    pub fn start_edit_sokay(state: &AppState, sokay_index: usize) -> Option<String> {
        if let Some(log) = state.get_daily_log(state.selected_date) {
            if sokay_index < log.sokay_entries.len() {
                return Some(log.sokay_entries[sokay_index].clone());
            }
        }
        None
    }

    pub fn calculate_cumulative_sokay(state: &AppState, up_to_date: chrono::NaiveDate) -> usize {
        state
            .daily_logs
            .iter()
            .filter(|log| log.date <= up_to_date)
            .map(|log| log.sokay_entries.len())
            .sum()
    }

    pub async fn delete_daily_log(
        state: &mut AppState,
        db_manager: &mut DbManager,
        file_manager: &FileManager,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<()> {
        db_manager.delete_daily_log(date).await?;
        state.daily_logs.retain(|log| log.date != date);
        let _ = file_manager.delete_daily_log(date);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod navigation_handler {
        use super::*;

        #[test]
        fn test_move_selection_down_empty_list() {
            let result = NavigationHandler::move_selection_down(None, 0);
            assert_eq!(result, None);
        }

        #[test]
        fn test_move_selection_down_single_item() {
            // Starting at None should select the first item
            let result = NavigationHandler::move_selection_down(None, 1);
            assert_eq!(result, Some(0));

            // Moving down from the only item should wrap to top
            let result = NavigationHandler::move_selection_down(Some(0), 1);
            assert_eq!(result, Some(0));
        }

        #[test]
        fn test_move_selection_down_multiple_items() {
            let list_len = 5;

            // Starting at None should select first item
            let result = NavigationHandler::move_selection_down(None, list_len);
            assert_eq!(result, Some(0));

            // Normal navigation down
            let result = NavigationHandler::move_selection_down(Some(0), list_len);
            assert_eq!(result, Some(1));

            let result = NavigationHandler::move_selection_down(Some(1), list_len);
            assert_eq!(result, Some(2));

            let result = NavigationHandler::move_selection_down(Some(3), list_len);
            assert_eq!(result, Some(4));
        }

        #[test]
        fn test_move_selection_down_wraparound() {
            let list_len = 3;

            // At the bottom (index 2), should wrap to top (index 0)
            let result = NavigationHandler::move_selection_down(Some(2), list_len);
            assert_eq!(result, Some(0));
        }
    }
}
