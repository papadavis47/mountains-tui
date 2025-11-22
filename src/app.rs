/// The App struct manages the overall application state and coordinates
/// between the UI, event handling, and data persistence.
use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, widgets::ListState};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::db_manager::{ConnectionState, DbManager};
use crate::events::handlers::{ActionHandler, InputHandler, NavigationHandler, SectionNavigator};
use crate::file_manager::FileManager;
use crate::models::{AppScreen, AppState, FocusedSection, MeasurementField, RunningField};
use crate::ui::screens;

/// This struct follows the "composition over inheritance" principle by
/// containing specialized handlers for different concerns:
/// - AppState: Core application state and data
/// - DbManager: Database operations with Turso Cloud sync
/// - FileManager: Markdown file backups
/// - InputHandler: Text input and cursor management
/// - ListState: UI list selection state
pub struct App {
    state: AppState,
    db_manager: Arc<RwLock<DbManager>>,
    file_manager: FileManager,
    input_handler: InputHandler,
    list_state: ListState,
    food_list_state: ListState,
    sokay_list_state: ListState,
    should_quit: bool,
    last_sync_time: Instant,
    sync_status: String,
}

impl App {
    /// The following constructor:
    /// 1. Creates the data directory (~/.mountains/)
    /// 2. Initializes DbManager with local-first approach (no cloud blocking)
    /// 3. Creates FileManager for markdown backups
    /// 4. Loads all existing daily logs from the database
    /// 5. Sets up UI state managers
    /// 6. Spawns background task to establish Turso Cloud connection

    /// The Result<Self, anyhow::Error> return type allows proper error handling
    /// if database or file operations fail during initialization.

    pub async fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let mountains_dir = home_dir.join(".mountains");

        if !mountains_dir.exists() {
            std::fs::create_dir_all(&mountains_dir)
                .context("Failed to create .mountains directory")?;
        }

        // Initialize database manager with local-first approach (instant startup)
        let db_manager = DbManager::new_local_first(&mountains_dir).await?;

        // Initialize file manager for markdown backups
        let file_manager = FileManager::new()?;

        let mut state = AppState::new();

        // Load all daily logs from the database (primary source of truth)
        state.daily_logs = db_manager.load_all_daily_logs().await?;

        // Wrap db_manager in Arc<RwLock> for shared access
        let db_manager = Arc::new(RwLock::new(db_manager));

        // Spawn background task to upgrade to remote replica
        // This runs on every startup to ensure cloud sync is active
        let db_manager_clone = Arc::clone(&db_manager);
        let mountains_dir_clone = mountains_dir.clone();
        tokio::spawn(async move {
            // Check if we have credentials
            if let (Ok(url), Ok(token)) = (
                std::env::var("TURSO_DATABASE_URL"),
                std::env::var("TURSO_AUTH_TOKEN"),
            ) {
                let db_path = mountains_dir_clone.join("mountains.db");
                if let Some(db_path_str) = db_path.to_str() {
                    let mut db = db_manager_clone.write().await;
                    let _ = db.upgrade_to_remote_replica(db_path_str, url, token).await;
                }
            }
        });

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
            sync_status: String::new(), // Will be set on first render
        })
    }

    /// This method runs the application by:
    /// 1. Drawing the current screen
    /// 2. Reading user input events (with timeout)
    /// 3. Processing events and updating state
    /// 4. Periodically syncing with Turso Cloud (every 4 minutes)
    /// 5. Repeating until the user quits
    ///
    /// The loop continues until should_quit becomes true.
    /// Uses a 1-second timeout to allow periodic sync checks without blocking UX.

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        loop {
            // Update sync status before drawing
            self.update_sync_status().await;

            // Draw the current UI state
            terminal.draw(|f| self.ui(f))?;

            // Read keyboard event with timeout to allow periodic sync checks
            if crossterm::event::poll(Duration::from_secs(1))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    self.handle_key_event_with_modifiers(key.code, key.modifiers)
                        .await?;
                }
            }

            // Check if we should sync with Turso Cloud (every 4 minutes)
            self.check_and_sync().await?;

            // Exit the loop if the user wants to quit
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// This method routes keyboard input to the appropriate handler based on
    /// the current screen. It also handles special key combinations with modifiers.
    async fn handle_key_event_with_modifiers(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        match self.state.current_screen {
            AppScreen::AddFood => self.handle_add_food_input(key).await?,
            AppScreen::EditFood(food_index) => self.handle_edit_food_input(key, food_index).await?,
            AppScreen::EditWeight => self.handle_edit_weight_input(key).await?,
            AppScreen::EditWaist => self.handle_edit_waist_input(key).await?,
            AppScreen::EditMiles => self.handle_edit_miles_input(key).await?,
            AppScreen::EditElevation => self.handle_edit_elevation_input(key).await?,
            AppScreen::AddSokay => self.handle_add_sokay_input(key).await?,
            AppScreen::EditSokay(sokay_index) => {
                self.handle_edit_sokay_input(key, sokay_index).await?
            }
            AppScreen::EditStrengthMobility => {
                self.handle_edit_strength_mobility_input_with_modifiers(key, modifiers)
                    .await?
            }
            AppScreen::EditNotes => {
                self.handle_edit_notes_input_with_modifiers(key, modifiers)
                    .await?
            }
            AppScreen::ConfirmDeleteDay => {
                self.handle_confirm_delete_day_input(key).await?
            }
            AppScreen::ConfirmDeleteFood(food_index) => {
                self.handle_confirm_delete_food_input(key, food_index).await?
            }
            AppScreen::ConfirmDeleteSokay(sokay_index) => {
                self.handle_confirm_delete_sokay_input(key, sokay_index).await?
            }
            _ => self.handle_navigation_input(key, modifiers).await?,
        }
        Ok(())
    }

    /// This method processes text input for adding new food entries.
    /// Enter saves the entry, Escape cancels, and other keys are handled
    /// by the InputHandler for text editing.

    async fn handle_add_food_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update: update state and switch screen immediately
                if let Some(log) = ActionHandler::save_food_entry(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                ) {
                    // Switch to DailyView immediately for instant feedback
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    // Persist in background (non-blocking)
                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
                    // Empty input - just go back without persisting
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;
                }
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
    async fn handle_edit_food_input(&mut self, key: KeyCode, food_index: usize) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update: update state and switch screen immediately
                if let Some(log) = ActionHandler::update_food_entry(
                    &mut self.state,
                    food_index,
                    self.input_handler.input_buffer.clone(),
                ) {
                    // Switch to DailyView immediately for instant feedback
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    // Persist in background (non-blocking)
                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
                    // Invalid input - just go back
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;
                }
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
    async fn handle_edit_weight_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update: update state and switch screen immediately
                let log = ActionHandler::update_weight(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );

                // Switch to DailyView immediately for instant feedback
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;

                // Persist in background (non-blocking)
                let db_manager = Arc::clone(&self.db_manager);
                let file_manager = self.file_manager.clone();
                tokio::spawn(async move {
                    ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                });
            }
            KeyCode::Esc => {
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

    async fn handle_edit_waist_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update
                let log = ActionHandler::update_waist(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;

                // Persist in background
                let db_manager = Arc::clone(&self.db_manager);
                let file_manager = self.file_manager.clone();
                tokio::spawn(async move {
                    ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                });
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

    /// This method processes multi-line text input for editing strength & mobility exercises.
    async fn handle_edit_strength_mobility_input_with_modifiers(
        &mut self,
        key: KeyCode,
        _modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update
                let log = ActionHandler::update_strength_mobility(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;

                // Persist in background
                let db_manager = Arc::clone(&self.db_manager);
                let file_manager = self.file_manager.clone();
                tokio::spawn(async move {
                    ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                });
            }
            KeyCode::Esc => {
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

    /// This method processes multi-line text input for editing daily notes.
    async fn handle_edit_notes_input_with_modifiers(
        &mut self,
        key: KeyCode,
        _modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update
                let log = ActionHandler::update_notes(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;

                // Persist in background
                let db_manager = Arc::clone(&self.db_manager);
                let file_manager = self.file_manager.clone();
                tokio::spawn(async move {
                    ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                });
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_multiline_text_input(key);
            }
        }
        Ok(())
    }

    /// Handles numeric input for editing miles covered.
    async fn handle_edit_miles_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update
                let log = ActionHandler::update_miles(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;

                // Persist in background
                let db_manager = Arc::clone(&self.db_manager);
                let file_manager = self.file_manager.clone();
                tokio::spawn(async move {
                    ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                });
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
    async fn handle_edit_elevation_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update
                let log = ActionHandler::update_elevation(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                );
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;

                // Persist in background
                let db_manager = Arc::clone(&self.db_manager);
                let file_manager = self.file_manager.clone();
                tokio::spawn(async move {
                    ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                });
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

    /// Handles text input for adding a new sokay entry.
    async fn handle_add_sokay_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update
                if let Some(log) = ActionHandler::save_sokay_entry(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    // Persist in background
                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
                    // Empty input - just go back
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    /// Handles text input for editing an existing sokay entry.
    async fn handle_edit_sokay_input(&mut self, key: KeyCode, sokay_index: usize) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Optimistic update
                if let Some(log) = ActionHandler::update_sokay_entry(
                    &mut self.state,
                    sokay_index,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    // Persist in background
                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
                    // Invalid input - just go back
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
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
    /// Shift+J/K switches focus between sections in DailyView.
    /// Tab toggles between fields within Measurements and Running sections.
    async fn handle_navigation_input(&mut self, key: KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<()> {
        // Handle Shift+J and Shift+K for switching focus between sections in DailyView
        if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            match key {
                KeyCode::Char('J') => {
                    // Shift+J: Move focus to next section
                    if matches!(self.state.current_screen, AppScreen::DailyView) {
                        self.state.focused_section = SectionNavigator::move_focus_down(&self.state.focused_section);
                    }
                    return Ok(());
                }
                KeyCode::Char('K') => {
                    // Shift+K: Move focus to previous section
                    if matches!(self.state.current_screen, AppScreen::DailyView) {
                        self.state.focused_section = SectionNavigator::move_focus_up(&self.state.focused_section);
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        match key {
            KeyCode::Char('q') => {
                // Quit the application
                self.should_quit = true;
            }
            KeyCode::Tab => {
                // Toggle internal field focus (Weight<->Waist, Miles<->Elevation)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.focused_section = SectionNavigator::toggle_internal_focus(&self.state.focused_section);
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Move selection down within the focused list (Food/Sokay only)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.move_food_selection_down(),
                        FocusedSection::Sokay => self.move_sokay_selection_down(),
                        _ => {} // Other sections don't have scrollable lists
                    }
                } else {
                    self.move_selection_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Move selection up within the focused list (Food/Sokay only)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.move_food_selection_up(),
                        FocusedSection::Sokay => self.move_sokay_selection_up(),
                        _ => {} // Other sections don't have scrollable lists
                    }
                } else {
                    self.move_selection_up();
                }
            }
            KeyCode::Enter => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_section_enter().await?;
                } else {
                    self.handle_enter();
                }
            }
            KeyCode::Esc => {
                self.handle_escape();
            }
            KeyCode::Char('D') => {
                // Delete confirmation - works on both Home (delete day) and DailyView (delete item)
                if matches!(self.state.current_screen, AppScreen::Home) {
                    // Delete entire day
                    self.handle_delete_day_confirmation();
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    // Delete individual item in focused list (only if item is focused)
                    match self.state.focused_section {
                        FocusedSection::FoodItems => {
                            if self.state.food_list_focused {
                                if let Some(selected_index) = self.food_list_state.selected() {
                                    self.state.current_screen = AppScreen::ConfirmDeleteFood(selected_index);
                                }
                            }
                        }
                        FocusedSection::Sokay => {
                            if self.state.sokay_list_focused {
                                if let Some(selected_index) = self.sokay_list_state.selected() {
                                    self.state.current_screen = AppScreen::ConfirmDeleteSokay(selected_index);
                                }
                            }
                        }
                        _ => {} // Other sections don't have individual items to delete
                    }
                }
            }
            KeyCode::Char('f') => {
                // Add food (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::AddFood;
                }
            }
            KeyCode::Char('E') => {
                // Edit item in focused list (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.handle_edit_food(),
                        FocusedSection::Sokay => self.handle_edit_sokay(),
                        _ => {} // Other sections use Enter key for editing
                    }
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
            KeyCode::Char('t') => {
                // Edit strength & mobility (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_strength_mobility();
                }
            }
            KeyCode::Char('N') => {
                // Go to today's log (only available on startup screen)
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    // Set selected_date to today
                    self.state.selected_date = chrono::Local::now().date_naive();
                    // Create the log if it doesn't exist
                    self.state.get_or_create_daily_log(self.state.selected_date);
                    // Switch to DailyView
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            KeyCode::Char('L') => {
                // Go to log list (only available on startup screen)
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.current_screen = AppScreen::Home;
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
                // Add sokay entry (only available in daily view)
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::AddSokay;
                }
            }
            KeyCode::Char('S') => {
                // Go back to Startup screen (available from Home or DailyView)
                if matches!(self.state.current_screen, AppScreen::Home | AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::Startup;
                }
            }
            KeyCode::Char(' ') => {
                // Toggle shortcuts help overlay
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::ShortcutsHelp;
                } else if matches!(self.state.current_screen, AppScreen::ShortcutsHelp) {
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            _ => {
                // Ignore other keys
            }
        }
        Ok(())
    }

    /// Handles Enter key press based on the currently focused section
    ///
    /// This method opens the appropriate edit screen for the focused section,
    /// or adds a new entry for list sections (Food/Sokay).
    async fn handle_section_enter(&mut self) -> Result<()> {
        match &self.state.focused_section {
            FocusedSection::Measurements { focused_field } => {
                match focused_field {
                    MeasurementField::Weight => self.handle_edit_weight(),
                    MeasurementField::Waist => self.handle_edit_waist(),
                }
            }
            FocusedSection::Running { focused_field } => {
                match focused_field {
                    RunningField::Miles => self.handle_edit_miles(),
                    RunningField::Elevation => self.handle_edit_elevation(),
                }
            }
            FocusedSection::FoodItems => {
                // Add new food entry
                self.state.current_screen = AppScreen::AddFood;
            }
            FocusedSection::Sokay => {
                // Add new sokay entry
                self.state.current_screen = AppScreen::AddSokay;
            }
            FocusedSection::StrengthMobility => {
                self.handle_edit_strength_mobility();
            }
            FocusedSection::Notes => {
                self.handle_edit_notes();
            }
        }
        Ok(())
    }

    /// This method acts as a router, delegating to the appropriate screen
    /// rendering function based on the current screen enum value.
    fn ui(&mut self, f: &mut Frame) {
        match self.state.current_screen {
            AppScreen::Startup => {
                screens::render_startup_screen(f, &self.state);
            }
            AppScreen::Home => {
                screens::render_home_screen(f, &self.state, &mut self.list_state, &self.sync_status);
            }
            AppScreen::DailyView => {
                screens::render_daily_view_screen(f, &self.state, &mut self.food_list_state, &mut self.sokay_list_state, &self.sync_status);
            }
            AppScreen::AddFood => {
                screens::render_add_food_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditFood(_) => {
                screens::render_edit_food_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditWeight => {
                screens::render_edit_weight_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditWaist => {
                screens::render_edit_waist_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditStrengthMobility => {
                screens::render_edit_strength_mobility_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditNotes => {
                screens::render_edit_notes_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditMiles => {
                screens::render_edit_miles_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditElevation => {
                screens::render_edit_elevation_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::AddSokay => {
                screens::render_add_sokay_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::EditSokay(_) => {
                screens::render_edit_sokay_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
                );
            }
            AppScreen::ConfirmDeleteDay => {
                screens::render_confirm_delete_day_screen(f, self.state.selected_date);
            }
            AppScreen::ConfirmDeleteFood(food_index) => {
                screens::render_confirm_delete_food_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    food_index,
                );
            }
            AppScreen::ConfirmDeleteSokay(sokay_index) => {
                screens::render_confirm_delete_sokay_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                    sokay_index,
                );
            }
            AppScreen::ShortcutsHelp => {
                screens::render_shortcuts_help_screen(
                    f,
                    &self.state,
                    &mut self.food_list_state,
                    &mut self.sokay_list_state,
                    &self.sync_status,
                );
            }
        }
    }

    // Navigation helper methods
    // These methods update the list selection state using the NavigationHandler

    /// Moves the selection down in the home screen daily logs list
    /// If the list is unfocused, this will focus the first item
    fn move_selection_down(&mut self) {
        if self.list_state.selected().is_none() && !self.state.daily_logs.is_empty() {
            // List is unfocused - focus the first item (most recent date)
            self.list_state.select(Some(0));
        } else {
            // List is focused - move down normally
            let new_selection = NavigationHandler::move_selection_down(
                self.list_state.selected(),
                self.state.daily_logs.len(),
            );
            self.list_state.select(new_selection);
        }
    }

    /// Moves the selection up in the home screen daily logs list
    /// If the list is unfocused, this will focus the last item
    fn move_selection_up(&mut self) {
        if self.list_state.selected().is_none() && !self.state.daily_logs.is_empty() {
            // List is unfocused - focus the last item (oldest date)
            self.list_state.select(Some(self.state.daily_logs.len() - 1));
        } else {
            // List is focused - move up normally
            let new_selection = NavigationHandler::move_selection_up(
                self.list_state.selected(),
                self.state.daily_logs.len(),
            );
            self.list_state.select(new_selection);
        }
    }

    /// Moves the selection down in the daily view food list
    /// If list is unfocused, focuses the first item
    fn move_food_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if !self.state.food_list_focused && !log.food_entries.is_empty() {
                // List is unfocused - focus the first item
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(0));
            } else {
                // List is focused - move down normally
                let new_selection = NavigationHandler::move_selection_down(
                    self.food_list_state.selected(),
                    log.food_entries.len(),
                );
                self.food_list_state.select(new_selection);
            }
        }
    }

    /// Moves the selection up in the daily view food list
    /// If list is unfocused, focuses the last item
    fn move_food_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let list_len = log.food_entries.len();
            let is_focused = self.state.food_list_focused;

            if !is_focused && list_len > 0 {
                // List is unfocused - focus the last item
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(list_len - 1));
            } else {
                // List is focused - move up normally
                let new_selection = NavigationHandler::move_selection_up(
                    self.food_list_state.selected(),
                    list_len,
                );
                self.food_list_state.select(new_selection);
            }
        }
    }

    /// Moves the selection down in the sokay view list
    /// If list is unfocused, focuses the first item
    fn move_sokay_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if !self.state.sokay_list_focused && !log.sokay_entries.is_empty() {
                // List is unfocused - focus the first item
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(0));
            } else {
                // List is focused - move down normally
                let new_selection = NavigationHandler::move_selection_down(
                    self.sokay_list_state.selected(),
                    log.sokay_entries.len(),
                );
                self.sokay_list_state.select(new_selection);
            }
        }
    }

    /// Moves the selection up in the sokay view list
    /// If list is unfocused, focuses the last item
    fn move_sokay_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let list_len = log.sokay_entries.len();
            let is_focused = self.state.sokay_list_focused;

            if !is_focused && list_len > 0 {
                // List is unfocused - focus the last item
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(list_len - 1));
            } else {
                // List is focused - move up normally
                let new_selection = NavigationHandler::move_selection_up(
                    self.sokay_list_state.selected(),
                    list_len,
                );
                self.sokay_list_state.select(new_selection);
            }
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

    /// Generally used to go back to the previous screen or unfocus lists:
    /// - Home: Unfocus the list
    /// - Daily View: Unfocus list item if focused, otherwise return to Home
    /// - Input screens: Handle separately in their input methods
    fn handle_escape(&mut self) {
        match self.state.current_screen {
            AppScreen::Home => {
                // Unfocus the list on home screen
                self.list_state.select(None);
            }
            AppScreen::DailyView => {
                // Check if we're in a list section with a focused item
                match self.state.focused_section {
                    FocusedSection::FoodItems => {
                        if self.state.food_list_focused {
                            // Unfocus the food list item
                            self.state.food_list_focused = false;
                            self.food_list_state.select(None);
                        } else {
                            // No item focused - return to home
                            self.state.current_screen = AppScreen::Home;
                        }
                    }
                    FocusedSection::Sokay => {
                        if self.state.sokay_list_focused {
                            // Unfocus the sokay list item
                            self.state.sokay_list_focused = false;
                            self.sokay_list_state.select(None);
                        } else {
                            // No item focused - return to home
                            self.state.current_screen = AppScreen::Home;
                        }
                    }
                    _ => {
                        // Not in a list section - return to home
                        self.state.current_screen = AppScreen::Home;
                    }
                }
            }
            _ => {
                // Other screens handle Escape in their input methods
            }
        }
    }

    /// This method:
    /// 1. Checks if an item is focused
    /// 2. Gets the currently selected food index
    /// 3. Pre-fills the input buffer with the current food name
    /// 4. Switches to the EditFood screen
    fn handle_edit_food(&mut self) {
        // Only allow editing if an item is focused
        if !self.state.food_list_focused {
            return;
        }

        if let Some(selected_index) = self.food_list_state.selected() {
            if let Some(current_name) = ActionHandler::start_edit_food(&self.state, selected_index)
            {
                self.input_handler.set_input(current_name);
                self.state.current_screen = AppScreen::EditFood(selected_index);
            }
        }
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

    /// Pre-fills the input buffer with the current strength & mobility value if it exists.
    fn handle_edit_strength_mobility(&mut self) {
        let current_sm = ActionHandler::start_edit_strength_mobility(&self.state);
        self.input_handler.set_input(current_sm);
        self.state.current_screen = AppScreen::EditStrengthMobility;
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

    /// Initiates editing of a sokay entry.
    fn handle_edit_sokay(&mut self) {
        // Only allow editing if an item is focused
        if !self.state.sokay_list_focused {
            return;
        }

        if let Some(selected_index) = self.sokay_list_state.selected() {
            if let Some(current_text) = ActionHandler::start_edit_sokay(&self.state, selected_index)
            {
                self.input_handler.set_input(current_text);
                self.state.current_screen = AppScreen::EditSokay(selected_index);
            }
        }
    }

    /// Initiates the delete day confirmation flow.
    /// Only proceeds if a day is selected in the Home screen list.
    fn handle_delete_day_confirmation(&mut self) {
        if let Some(selected_index) = self.list_state.selected() {
            if selected_index < self.state.daily_logs.len() {
                // Set the selected_date to the day we want to delete
                self.state.selected_date = self.state.daily_logs[selected_index].date;
                // Switch to confirmation screen
                self.state.current_screen = AppScreen::ConfirmDeleteDay;
            }
        }
    }

    /// Handles keyboard input on the delete confirmation screen.
    /// 'Y' confirms deletion, 'n' or Esc cancels.
    async fn handle_confirm_delete_day_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('Y') => {
                // Confirmed - delete the day
                let date_to_delete = self.state.selected_date;

                {
                    let mut db = self.db_manager.write().await;
                    ActionHandler::delete_daily_log(
                        &mut self.state,
                        &mut *db,
                        &self.file_manager,
                        date_to_delete,
                    )
                    .await?;
                }

                // Return to home screen
                self.state.current_screen = AppScreen::Home;

                // Clear selection since the list has changed
                self.list_state.select(None);
            }
            KeyCode::Char('N') | KeyCode::Esc => {
                // Cancelled - return to home screen without deleting
                self.state.current_screen = AppScreen::Home;
            }
            _ => {
                // Ignore other keys
            }
        }
        Ok(())
    }

    /// Handles confirmation input for deleting a food item
    async fn handle_confirm_delete_food_input(&mut self, key: KeyCode, food_index: usize) -> Result<()> {
        match key {
            KeyCode::Char('Y') => {
                // Confirmed - delete the food item with optimistic update
                if let Some(log) = ActionHandler::delete_food_entry(
                    &mut self.state,
                    food_index,
                ) {
                    // Update selection after deletion
                    if let Some(current_log) = self.state.get_daily_log(self.state.selected_date) {
                        if current_log.food_entries.is_empty() {
                            self.food_list_state.select(None);
                        } else if food_index >= current_log.food_entries.len() {
                            self.food_list_state
                                .select(Some(current_log.food_entries.len() - 1));
                        }
                    }

                    // Return to daily view
                    self.state.current_screen = AppScreen::DailyView;

                    // Persist in background
                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
                    // Invalid operation - just go back
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            KeyCode::Char('N') | KeyCode::Esc => {
                // Cancelled - return to daily view without deleting
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                // Ignore other keys
            }
        }
        Ok(())
    }

    /// Handles confirmation input for deleting a sokay item
    async fn handle_confirm_delete_sokay_input(&mut self, key: KeyCode, sokay_index: usize) -> Result<()> {
        match key {
            KeyCode::Char('Y') => {
                // Confirmed - delete the sokay item with optimistic update
                if let Some(log) = ActionHandler::delete_sokay_entry(
                    &mut self.state,
                    sokay_index,
                ) {
                    // Update selection after deletion
                    if let Some(current_log) = self.state.get_daily_log(self.state.selected_date) {
                        if current_log.sokay_entries.is_empty() {
                            self.sokay_list_state.select(None);
                        } else if sokay_index >= current_log.sokay_entries.len() {
                            self.sokay_list_state
                                .select(Some(current_log.sokay_entries.len() - 1));
                        }
                    }

                    // Return to daily view
                    self.state.current_screen = AppScreen::DailyView;

                    // Persist in background
                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
                    // Invalid operation - just go back
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            KeyCode::Char('N') | KeyCode::Esc => {
                // Cancelled - return to daily view without deleting
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                // Ignore other keys
            }
        }
        Ok(())
    }

    /// Checks if enough time has passed since last sync and syncs if appropriate
    ///
    /// This method:
    /// 1. Checks if 4 minutes have passed since the last sync
    /// 2. Checks if the user is currently typing (in an input screen)
    /// 3. If enough time has passed and user is not typing, triggers a sync
    ///
    /// This ensures regular syncing while the app is active, but avoids
    /// interrupting the user during input operations.
    async fn check_and_sync(&mut self) -> Result<()> {
        const SYNC_INTERVAL: Duration = Duration::from_secs(240);

        // Check if enough time has passed since last sync
        if self.last_sync_time.elapsed() < SYNC_INTERVAL {
            return Ok(()); // Not time to sync yet
        }

        // Check if user is currently typing - if so, skip sync to avoid interruption
        if self.is_user_typing() {
            return Ok(()); // User is typing, don't interrupt
        }

        // Perform the sync (now async)
        {
            let db = self.db_manager.read().await;
            db.sync_now().await?;
        }

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
                | AppScreen::EditStrengthMobility
                | AppScreen::EditNotes
        )
    }

    /// Updates the sync status field based on current connection state
    async fn update_sync_status(&mut self) {
        let db = self.db_manager.read().await;
        let state = db.get_connection_state().await;

        self.sync_status = match state {
            ConnectionState::Disconnected => "⚪ Offline".to_string(),
            ConnectionState::Connected => "✓ Synced".to_string(),
            ConnectionState::Error(_) => "⚠️ Sync Error".to_string(),
        };
    }
}
