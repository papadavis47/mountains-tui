/// This module contains the core App struct and its implementation.

/// The App struct manages the overall application state and coordinates
/// between the UI, event handling, and data persistence.
use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, widgets::ListState};
use std::io;

use crate::events::handlers::{ActionHandler, InputHandler, NavigationHandler};
use crate::file_manager::FileManager;
use crate::models::{AppScreen, AppState};
use crate::ui::screens;

/// This struct follows the "composition over inheritance" principle by
/// containing specialized handlers for different concerns:
/// - AppState: Core application state and data
/// - FileManager: File I/O operations
/// - InputHandler: Text input and cursor management
/// - ListState: UI list selection state
pub struct App {
    /// Core application state containing daily logs and current screen
    state: AppState,
    /// Handles saving and loading daily logs to/from markdown files
    file_manager: FileManager,
    /// Manages text input for various input screens
    input_handler: InputHandler,
    /// Tracks which item is selected in the home screen list
    list_state: ListState,
    /// Tracks which item is selected in the daily view food list
    food_list_state: ListState,
    /// Flag to indicate when the application should exit
    should_quit: bool,
}

impl App {
    /// The following constructor:
    /// 1. Creates a new FileManager to handle data persistence
    /// 2. Initializes the application state
    /// 3. Loads all existing daily logs from disk
    /// 4. Sets up UI state managers

    /// The Result<Self, anyhow::Error> return type allows proper error handling
    /// if file operations fail during initialization.

    pub fn new() -> Result<Self> {
        let file_manager = FileManager::new()?;
        let mut state = AppState::new();

        // This populates the state with any previously saved daily logs
        state.daily_logs = file_manager.load_all_daily_logs()?;

        Ok(Self {
            state,
            file_manager,
            input_handler: InputHandler::new(),
            list_state: ListState::default(),
            food_list_state: ListState::default(),
            should_quit: false,
        })
    }

    /// This method runs the application by:
    /// 1. Drawing the current screen
    /// 2. Reading user input events
    /// 3. Processing events and updating state
    /// 4. Repeating until the user quits
    ///
    /// The loop continues until should_quit becomes true.

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Draw the current UI state
            terminal.draw(|f| self.ui(f))?;

            // Read and process the next keyboard event
            if let Event::Key(key) = crossterm::event::read()? {
                self.handle_key_event_with_modifiers(key.code, key.modifiers)?;
            }

            // Exit the loop if the user wants to quit
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// This method routes keyboard input to the appropriate handler based on
    /// the current screen. It also handles special key combinations with modifiers.
    fn handle_key_event_with_modifiers(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        match self.state.current_screen {
            AppScreen::AddFood => self.handle_add_food_input(key)?,
            AppScreen::EditFood(food_index) => self.handle_edit_food_input(key, food_index)?,
            AppScreen::EditWeight => self.handle_edit_weight_input(key)?,
            AppScreen::EditWaist => self.handle_edit_waist_input(key)?,
            AppScreen::EditNotes => self.handle_edit_notes_input_with_modifiers(key, modifiers)?,
            _ => self.handle_navigation_input(key)?,
        }
        Ok(())
    }

    /// This method processes text input for adding new food entries.
    /// Enter saves the entry, Escape cancels, and other keys are handled
    /// by the InputHandler for text editing.

    fn handle_add_food_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Save the food entry and return to daily view
                ActionHandler::save_food_entry(
                    &mut self.state,
                    &self.file_manager,
                    self.input_handler.input_buffer.clone(),
                    None, // No notes support in current implementation
                )?;
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            KeyCode::Esc => {
                // Cancel input and return to daily view
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                // Handle text editing (cursor movement, character input, etc.)
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    /// Similar to add food input, but updates an existing food entry.
    fn handle_edit_food_input(&mut self, key: KeyCode, food_index: usize) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Update the existing food entry
                ActionHandler::update_food_entry(
                    &mut self.state,
                    &self.file_manager,
                    food_index,
                    self.input_handler.input_buffer.clone(),
                )?;
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            KeyCode::Esc => {
                // Cancel editing
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    /// Uses numeric input handling to only allow numbers and decimal points.
    fn handle_edit_weight_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Save the weight measurement
                ActionHandler::update_weight(
                    &mut self.state,
                    &self.file_manager,
                    self.input_handler.input_buffer.clone(),
                )?;
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            KeyCode::Esc => {
                // Cancel weight editing
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                // Handle numeric input only
                self.input_handler.handle_numeric_input(key);
            }
        }
        Ok(())
    }

    /// Similar to weight input but for waist measurements.
    fn handle_edit_waist_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                ActionHandler::update_waist(
                    &mut self.state,
                    &self.file_manager,
                    self.input_handler.input_buffer.clone(),
                )?;
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_numeric_input(key);
            }
        }
        Ok(())
    }

    /// This method processes multi-line text input for editing daily notes.
    /// It supports special key combinations like Ctrl+J for newlines.
    /// Handles input for the Edit Notes screen with modifier support
    ///
    /// This method processes multi-line text input for editing daily notes.
    /// It supports special key combinations like Ctrl+J for newlines.
    fn handle_edit_notes_input_with_modifiers(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        // First check for special key combinations
        if self
            .input_handler
            .handle_multiline_special_keys(key, modifiers)
        {
            return Ok(());
        }

        match key {
            KeyCode::Enter => {
                // Save the notes and return to daily view
                ActionHandler::update_notes(
                    &mut self.state,
                    &self.file_manager,
                    self.input_handler.input_buffer.clone(),
                )?;
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            KeyCode::Esc => {
                // Cancel notes editing
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                // Handle multi-line text editing
                self.input_handler.handle_multiline_text_input(key);
            }
        }
        Ok(())
    }

    /// This method handles keyboard input for the Home and Daily View screens,
    /// including navigation (up/down), actions (add, edit, delete), and
    /// screen transitions.
    fn handle_navigation_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => {
                // Quit the application
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Move selection down
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.move_food_selection_down();
                } else {
                    self.move_selection_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Move selection up
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.move_food_selection_up();
                } else {
                    self.move_selection_up();
                }
            }
            KeyCode::Enter => {
                self.handle_enter();
            }
            KeyCode::Esc => {
                self.handle_escape();
            }
            KeyCode::Char('a') => {
                // Add food (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::AddFood;
                }
            }
            KeyCode::Char('e') => {
                // Edit food (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_food();
                }
            }
            KeyCode::Char('d') => {
                // Delete food (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_delete_food()?;
                }
            }
            KeyCode::Char('w') => {
                // Edit weight (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_weight();
                }
            }
            KeyCode::Char('s') => {
                // Edit waist (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_waist();
                }
            }
            KeyCode::Char('n') => {
                // Edit notes (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_notes();
                }
            }
            _ => {
                // Ignore other keys
            }
        }
        Ok(())
    }

    /// This method acts as a router, delegating to the appropriate screen
    /// rendering function based on the current screen enum value.
    fn ui(&mut self, f: &mut Frame) {
        match self.state.current_screen {
            AppScreen::Home => {
                screens::render_home_screen(f, &self.state, &mut self.list_state);
            }
            AppScreen::DailyView => {
                screens::render_daily_view_screen(f, &self.state, &mut self.food_list_state);
            }
            AppScreen::AddFood => {
                screens::render_add_food_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditFood(_) => {
                screens::render_edit_food_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditWeight => {
                screens::render_edit_weight_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditWaist => {
                screens::render_edit_waist_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditNotes => {
                screens::render_edit_notes_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            _ => {
                // Default to home screen for any unhandled screen types
                screens::render_home_screen(f, &self.state, &mut self.list_state);
            }
        }
    }

    // Navigation helper methods
    // These methods update the list selection state using the NavigationHandler

    /// Moves the selection down in the home screen daily logs list
    fn move_selection_down(&mut self) {
        let new_selection = NavigationHandler::move_selection_down(
            self.list_state.selected(),
            self.state.daily_logs.len(),
        );
        self.list_state.select(new_selection);
    }

    /// Moves the selection up in the home screen daily logs list
    fn move_selection_up(&mut self) {
        let new_selection = NavigationHandler::move_selection_up(
            self.list_state.selected(),
            self.state.daily_logs.len(),
        );
        self.list_state.select(new_selection);
    }

    /// Moves the selection down in the daily view food list
    fn move_food_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let new_selection = NavigationHandler::move_selection_down(
                self.food_list_state.selected(),
                log.food_entries.len(),
            );
            self.food_list_state.select(new_selection);
        }
    }

    /// Moves the selection up in the daily view food list
    fn move_food_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let new_selection = NavigationHandler::move_selection_up(
                self.food_list_state.selected(),
                log.food_entries.len(),
            );
            self.food_list_state.select(new_selection);
        }
    }

    /// Behavior depends on the current screen:
    /// - Home: Navigate to selected daily log or create today's log
    /// - Other screens: No action (input screens handle Enter separately)
    fn handle_enter(&mut self) {
        match self.state.current_screen {
            AppScreen::Home => {
                ActionHandler::handle_home_enter(&mut self.state, self.list_state.selected());
            }
            _ => {
                // Other screens don't need Enter handling in navigation mode
            }
        }
    }

    /// Generally used to go back to the previous screen:
    /// - Daily View: Return to Home
    /// - Input screens: Handle separately in their input methods
    fn handle_escape(&mut self) {
        match self.state.current_screen {
            AppScreen::DailyView => {
                self.state.current_screen = AppScreen::Home;
            }
            _ => {
                // Other screens handle Escape in their input methods
            }
        }
    }

    /// This method:
    /// 1. Gets the currently selected food index
    /// 2. Pre-fills the input buffer with the current food name
    /// 3. Switches to the EditFood screen
    fn handle_edit_food(&mut self) {
        if let Some(selected_index) = self.food_list_state.selected() {
            if let Some(current_name) = ActionHandler::start_edit_food(&self.state, selected_index)
            {
                self.input_handler.set_input(current_name);
                self.state.current_screen = AppScreen::EditFood(selected_index);
            }
        }
    }

    /// This method also handles updating the selection state after deletion.
    fn handle_delete_food(&mut self) -> Result<()> {
        if let Some(selected_index) = self.food_list_state.selected() {
            ActionHandler::delete_food_entry(&mut self.state, &self.file_manager, selected_index)?;

            // Update selection after deletion
            if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
                if log.food_entries.is_empty() {
                    // No items left - clear selection
                    self.food_list_state.select(None);
                } else if selected_index >= log.food_entries.len() {
                    // Selected index is now out of bounds - select the last item
                    self.food_list_state
                        .select(Some(log.food_entries.len() - 1));
                }
                // If the selected index is still valid, keep the current selection
            }
        }
        Ok(())
    }

    /// Pre-fills the input buffer with the current weight value if it exists.
    fn handle_edit_weight(&mut self) {
        let current_weight = ActionHandler::start_edit_weight(&self.state);
        self.input_handler.set_input(current_weight);
        self.state.current_screen = AppScreen::EditWeight;
    }

    /// Pre-fills the input buffer with the current waist value if it exists.
    fn handle_edit_waist(&mut self) {
        let current_waist = ActionHandler::start_edit_waist(&self.state);
        self.input_handler.set_input(current_waist);
        self.state.current_screen = AppScreen::EditWaist;
    }

    /// Pre-fills the input buffer with the current notes value if it exists.
    fn handle_edit_notes(&mut self) {
        let current_notes = ActionHandler::start_edit_notes(&self.state);
        self.input_handler.set_input(current_notes);
        self.state.current_screen = AppScreen::EditNotes;
    }
}
