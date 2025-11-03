/// This module contains the core App struct and its implementation.

/// The App struct manages the overall application state and coordinates
/// between the UI, event handling, and data persistence.
use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, widgets::ListState};
use std::io;
use std::time::{Duration, Instant};

use crate::db_manager::DbManager;
use crate::events::handlers::{ActionHandler, InputHandler, NavigationHandler};
use crate::file_manager::FileManager;
use crate::models::{AppScreen, AppState};
use crate::ui::screens;

/// This struct follows the "composition over inheritance" principle by
/// containing specialized handlers for different concerns:
/// - AppState: Core application state and data
/// - DbManager: Database operations with Turso Cloud sync
/// - FileManager: Markdown file backups
/// - InputHandler: Text input and cursor management
/// - ListState: UI list selection state
pub struct App {
    /// Core application state containing daily logs and current screen
    state: AppState,
    /// Handles database operations with Turso Cloud sync
    db_manager: DbManager,
    /// Handles saving and loading daily logs to/from markdown files (backup)
    file_manager: FileManager,
    /// Manages text input for various input screens
    input_handler: InputHandler,
    /// Tracks which item is selected in the home screen list
    list_state: ListState,
    /// Tracks which item is selected in the daily view food list
    food_list_state: ListState,
    /// Tracks which item is selected in the sokay view list
    sokay_list_state: ListState,
    /// Flag to indicate when the application should exit
    should_quit: bool,
    /// Timestamp of the last cloud sync operation
    last_sync_time: Instant,
}

impl App {
    /// The following constructor:
    /// 1. Creates the data directory (~/.mountains/)
    /// 2. Initializes DbManager with Turso Cloud sync
    /// 3. Creates FileManager for markdown backups
    /// 4. Loads all existing daily logs from the database
    /// 5. Sets up UI state managers

    /// The Result<Self, anyhow::Error> return type allows proper error handling
    /// if database or file operations fail during initialization.

    pub async fn new() -> Result<Self> {
        // Get the user's home directory and create .mountains directory
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let mountains_dir = home_dir.join(".mountains");

        // Create directory if it doesn't exist
        if !mountains_dir.exists() {
            std::fs::create_dir_all(&mountains_dir)
                .context("Failed to create .mountains directory")?;
        }

        // Initialize database manager with Turso Cloud sync
        let db_manager = DbManager::new(&mountains_dir).await?;

        // Initialize file manager for markdown backups
        let file_manager = FileManager::new()?;

        let mut state = AppState::new();

        // Load all daily logs from the database (primary source of truth)
        state.daily_logs = db_manager.load_all_daily_logs().await?;

        Ok(Self {
            state,
            db_manager,
            file_manager,
            input_handler: InputHandler::new(),
            list_state: ListState::default(),
            food_list_state: ListState::default(),
            sokay_list_state: ListState::default(),
            should_quit: false,
            last_sync_time: Instant::now(),
        })
    }

    /// This method runs the application by:
    /// 1. Drawing the current screen
    /// 2. Reading user input events (with timeout)
    /// 3. Processing events and updating state
    /// 4. Periodically syncing with Turso Cloud (every 60 seconds)
    /// 5. Repeating until the user quits
    ///
    /// The loop continues until should_quit becomes true.
    /// Uses a 1-second timeout to allow periodic sync checks without blocking UX.

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Draw the current UI state
            terminal.draw(|f| self.ui(f))?;

            // Read keyboard event with timeout to allow periodic sync checks
            if crossterm::event::poll(Duration::from_secs(1))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    self.handle_key_event_with_modifiers(key.code, key.modifiers)?;
                }
            }

            // Check if we should sync with Turso Cloud (every 60 seconds)
            self.check_and_sync()?;

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
            AppScreen::EditMiles => self.handle_edit_miles_input(key)?,
            AppScreen::EditElevation => self.handle_edit_elevation_input(key)?,
            AppScreen::SokayView => self.handle_sokay_view_input(key)?,
            AppScreen::AddSokay => self.handle_add_sokay_input(key)?,
            AppScreen::EditSokay(sokay_index) => self.handle_edit_sokay_input(key, sokay_index)?,
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
                    &mut self.db_manager,
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
                    &mut self.db_manager,
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
                    &mut self.db_manager,
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
                    &mut self.db_manager,
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
                    &mut self.db_manager,
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

    /// Handles numeric input for editing miles covered.
    fn handle_edit_miles_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                ActionHandler::update_miles(
                    &mut self.state,
                    &mut self.db_manager,
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

    /// Handles integer input for editing elevation gain (no decimal points allowed).
    fn handle_edit_elevation_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                ActionHandler::update_elevation(
                    &mut self.state,
                    &mut self.db_manager,
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
                self.input_handler.handle_integer_input(key);
            }
        }
        Ok(())
    }

    /// Handles keyboard input for the Sokay View screen (list of sokay entries).
    fn handle_sokay_view_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('a') => {
                self.state.current_screen = AppScreen::AddSokay;
            }
            KeyCode::Char('e') => {
                self.handle_edit_sokay();
            }
            KeyCode::Char('d') => {
                self.handle_delete_sokay()?;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_sokay_selection_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_sokay_selection_up();
            }
            KeyCode::Esc => {
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles text input for adding a new sokay entry.
    fn handle_add_sokay_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                ActionHandler::save_sokay_entry(
                    &mut self.state,
                    &mut self.db_manager,
                    &self.file_manager,
                    self.input_handler.input_buffer.clone(),
                )?;
                self.input_handler.clear();
                self.state.current_screen = AppScreen::SokayView;
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::SokayView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    /// Handles text input for editing an existing sokay entry.
    fn handle_edit_sokay_input(&mut self, key: KeyCode, sokay_index: usize) -> Result<()> {
        match key {
            KeyCode::Enter => {
                ActionHandler::update_sokay_entry(
                    &mut self.state,
                    &mut self.db_manager,
                    &self.file_manager,
                    sokay_index,
                    self.input_handler.input_buffer.clone(),
                )?;
                self.input_handler.clear();
                self.state.current_screen = AppScreen::SokayView;
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::SokayView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
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
            KeyCode::Char('m') => {
                // Edit miles (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_miles();
                }
            }
            KeyCode::Char('l') => {
                // Edit elevation (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_elevation();
                }
            }
            KeyCode::Char('c') => {
                // View sokay entries (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_sokay_view();
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
            AppScreen::EditMiles => {
                screens::render_edit_miles_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditElevation => {
                screens::render_edit_elevation_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::SokayView => {
                screens::render_sokay_view_screen(f, &self.state, &mut self.sokay_list_state);
            }
            AppScreen::AddSokay => {
                screens::render_add_sokay_screen(
                    f,
                    self.state.selected_date,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditSokay(_) => {
                screens::render_edit_sokay_screen(
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

    /// Moves the selection down in the sokay view list
    fn move_sokay_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let new_selection = NavigationHandler::move_selection_down(
                self.sokay_list_state.selected(),
                log.sokay_entries.len(),
            );
            self.sokay_list_state.select(new_selection);
        }
    }

    /// Moves the selection up in the sokay view list
    fn move_sokay_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let new_selection = NavigationHandler::move_selection_up(
                self.sokay_list_state.selected(),
                log.sokay_entries.len(),
            );
            self.sokay_list_state.select(new_selection);
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
            ActionHandler::delete_food_entry(&mut self.state, &mut self.db_manager, &self.file_manager, selected_index)?;

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

    /// Pre-fills the input buffer with the current miles value if it exists.
    fn handle_edit_miles(&mut self) {
        let current_miles = ActionHandler::start_edit_miles(&self.state);
        self.input_handler.set_input(current_miles);
        self.state.current_screen = AppScreen::EditMiles;
    }

    /// Pre-fills the input buffer with the current elevation value if it exists.
    fn handle_edit_elevation(&mut self) {
        let current_elevation = ActionHandler::start_edit_elevation(&self.state);
        self.input_handler.set_input(current_elevation);
        self.state.current_screen = AppScreen::EditElevation;
    }

    /// Switches to the Sokay View screen.
    fn handle_sokay_view(&mut self) {
        self.state.current_screen = AppScreen::SokayView;
        self.sokay_list_state.select(None); // Reset selection
    }

    /// Initiates editing of a sokay entry.
    fn handle_edit_sokay(&mut self) {
        if let Some(selected_index) = self.sokay_list_state.selected() {
            if let Some(current_text) =
                ActionHandler::start_edit_sokay(&self.state, selected_index)
            {
                self.input_handler.set_input(current_text);
                self.state.current_screen = AppScreen::EditSokay(selected_index);
            }
        }
    }

    /// Deletes a sokay entry.
    fn handle_delete_sokay(&mut self) -> Result<()> {
        if let Some(selected_index) = self.sokay_list_state.selected() {
            ActionHandler::delete_sokay_entry(
                &mut self.state,
                &mut self.db_manager,
                &self.file_manager,
                selected_index,
            )?;

            // Update selection after deletion
            if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
                if log.sokay_entries.is_empty() {
                    // No items left - clear selection
                    self.sokay_list_state.select(None);
                } else if selected_index >= log.sokay_entries.len() {
                    // Selected index is now out of bounds - select the last item
                    self.sokay_list_state
                        .select(Some(log.sokay_entries.len() - 1));
                }
                // If the selected index is still valid, keep the current selection
            }
        }
        Ok(())
    }

    /// Checks if enough time has passed since last sync and syncs if appropriate
    ///
    /// This method:
    /// 1. Checks if 60 seconds have passed since the last sync
    /// 2. Checks if the user is currently typing (in an input screen)
    /// 3. If enough time has passed and user is not typing, triggers a sync
    ///
    /// This ensures regular syncing while the app is active, but avoids
    /// interrupting the user during input operations.
    fn check_and_sync(&mut self) -> Result<()> {
        const SYNC_INTERVAL: Duration = Duration::from_secs(60);

        // Check if enough time has passed since last sync
        if self.last_sync_time.elapsed() < SYNC_INTERVAL {
            return Ok(()); // Not time to sync yet
        }

        // Check if user is currently typing - if so, skip sync to avoid interruption
        if self.is_user_typing() {
            return Ok(()); // User is typing, don't interrupt
        }

        // Perform the sync (non-blocking, runs in background)
        tokio::runtime::Handle::current().block_on(self.db_manager.sync_now())?;

        // Update last sync time
        self.last_sync_time = Instant::now();

        Ok(())
    }

    /// Returns true if the user is currently in an input screen (typing mode)
    ///
    /// We avoid syncing during input operations to prevent any potential
    /// interruption or lag while the user is actively entering data.
    fn is_user_typing(&self) -> bool {
        matches!(
            self.state.current_screen,
            AppScreen::AddFood
                | AppScreen::EditFood(_)
                | AppScreen::EditWeight
                | AppScreen::EditWaist
                | AppScreen::EditMiles
                | AppScreen::EditElevation
                | AppScreen::AddSokay
                | AppScreen::EditSokay(_)
                | AppScreen::EditNotes
        )
    }
}
