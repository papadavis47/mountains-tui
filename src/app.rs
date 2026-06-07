use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, widgets::ListState};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::db_manager::{ConnectionState, DbManager};
use crate::events::handlers::{ActionHandler, InputHandler, NavigationHandler, SectionNavigator};
use crate::file_manager::FileManager;
use crate::models::{AppScreen, AppState, ConfigSyncField, FocusedSection, MeasurementField, RunningField};
use crate::ui::screens;

pub struct App {
    state: AppState,
    config: AppConfig,
    db_manager: Arc<RwLock<DbManager>>,
    file_manager: FileManager,
    input_handler: InputHandler,
    list_state: ListState,
    food_list_state: ListState,
    sokay_list_state: ListState,
    should_quit: bool,
    sync_status: String,
    config_url_buffer: String,
    config_token_buffer: String,
    config_sync_enabled: bool,
}

impl App {
    /// Creates app with instant startup, spawns background cloud sync if configured
    pub async fn new(config: AppConfig) -> Result<Self> {
        let mountains_dir = crate::config::data_dir()?;

        if !mountains_dir.exists() {
            std::fs::create_dir_all(&mountains_dir)
                .context("Failed to create .mountains directory")?;
        }

        let db_manager = DbManager::new_local_first(&mountains_dir).await?;
        let file_manager = FileManager::new()?;

        let mut state = AppState::new();
        state.daily_logs = db_manager.load_all_daily_logs().await?;

        let db_manager = Arc::new(RwLock::new(db_manager));

        // Spawn background cloud sync only if config has valid credentials
        if config.sync.is_configured() {
            let db_manager_clone = Arc::clone(&db_manager);
            let mountains_dir_clone = mountains_dir.clone();
            let url = config.sync.db_url.clone();
            let token = config.sync.auth_token.clone();
            tokio::spawn(async move {
                let db_path = mountains_dir_clone.join("mountains.db");
                if let Some(db_path_str) = db_path.to_str() {
                    let mut db = db_manager_clone.write().await;
                    let _ = db.upgrade_to_remote_replica(db_path_str, url, token).await;
                }
            });
        }

        Ok(Self {
            state,
            config,
            db_manager,
            file_manager,
            input_handler: InputHandler::new(),
            list_state: ListState::default(),
            food_list_state: ListState::default(),
            sokay_list_state: ListState::default(),
            should_quit: false,
            sync_status: String::new(),
            config_url_buffer: String::new(),
            config_token_buffer: String::new(),
            config_sync_enabled: false,
        })
    }

    /// Main event loop
    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        loop {
            self.update_sync_status().await;

            // Handle syncing screen
            if matches!(self.state.current_screen, AppScreen::Syncing) {
                terminal.draw(|f| self.ui(f))?;
                self.perform_shutdown_sync().await;
                terminal.draw(|f| self.ui(f))?;
                std::thread::sleep(Duration::from_millis(1000));
            }

            terminal.draw(|f| self.ui(f))?;

            if crossterm::event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = crossterm::event::read()? {
                    self.handle_key_event_with_modifiers(key.code, key.modifiers)
                        .await?;
                }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    async fn handle_key_event_with_modifiers(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<()> {
        match self.state.current_screen {
            AppScreen::AddFood => self.handle_add_food_input(key).await?,
            AppScreen::EditFood(food_index) => self.handle_edit_food_input(key, food_index).await?,
            AppScreen::AddSokay => self.handle_add_sokay_input(key).await?,
            AppScreen::EditSokay(sokay_index) => {
                self.handle_edit_sokay_input(key, sokay_index).await?
            }
            AppScreen::InputField(field_type) => {
                self.handle_field_input(key, modifiers, field_type).await?;
            }
            AppScreen::ConfirmDelete(target) => {
                self.handle_delete_confirmation_input(key, target).await?;
            }
            AppScreen::DateInput => self.handle_date_input(key).await?,
            AppScreen::ConfigSync => self.handle_config_sync_input(key).await?,
            _ => self.handle_navigation_input(key, modifiers).await?,
        }
        Ok(())
    }

    async fn handle_add_food_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                if let Some(log) = ActionHandler::save_food_entry(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    // Persist in background for instant UI feedback
                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
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

    async fn handle_edit_food_input(&mut self, key: KeyCode, food_index: usize) -> Result<()> {
        match key {
            KeyCode::Enter => {
                if let Some(log) = ActionHandler::update_food_entry(
                    &mut self.state,
                    food_index,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
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

    /// Generic handler for all field inputs - consolidates 6 separate handlers
    async fn handle_field_input(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
        field_type: crate::models::field_accessor::FieldType,
    ) -> Result<()> {
        use crate::models::field_accessor::FieldType;

        match key {
            KeyCode::Enter => {
                let is_multiline = matches!(field_type, FieldType::StrengthMobility | FieldType::Notes);
                // Use Alt modifier for newline insertion (most reliable across terminals)
                let has_alt = modifiers.contains(crossterm::event::KeyModifiers::ALT);

                // Alt+Enter in multiline inputs inserts newline, regular Enter saves
                if is_multiline && has_alt {
                    // Insert newline and stay in edit mode
                    self.input_handler.insert_newline();
                } else {
                    // Save and exit
                    let log = ActionHandler::update_field(
                        &mut self.state,
                        field_type,
                        self.input_handler.input_buffer.clone(),
                    );
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {
                match field_type {
                    FieldType::Weight | FieldType::Waist | FieldType::Miles => {
                        self.input_handler.handle_numeric_input(key);
                    }
                    FieldType::Elevation => {
                        self.input_handler.handle_integer_input(key);
                    }
                    FieldType::StrengthMobility | FieldType::Notes => {
                        self.input_handler.handle_multiline_text_input(key, modifiers);
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_add_sokay_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                if let Some(log) = ActionHandler::save_sokay_entry(
                    &mut self.state,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
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

    async fn handle_edit_sokay_input(&mut self, key: KeyCode, sokay_index: usize) -> Result<()> {
        match key {
            KeyCode::Enter => {
                if let Some(log) = ActionHandler::update_sokay_entry(
                    &mut self.state,
                    sokay_index,
                    self.input_handler.input_buffer.clone(),
                ) {
                    self.input_handler.clear();
                    self.state.current_screen = AppScreen::DailyView;

                    let db_manager = Arc::clone(&self.db_manager);
                    let file_manager = self.file_manager.clone();
                    tokio::spawn(async move {
                        ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                    });
                } else {
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

    async fn handle_date_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                let input = self.input_handler.input_buffer.clone();
                match chrono::NaiveDate::parse_from_str(&input, "%m.%d.%Y") {
                    Ok(date) => {
                        let today = chrono::Local::now().date_naive();
                        if date > today {
                            self.state.date_input_error = Some("Future dates not allowed".to_string());
                        } else {
                            self.input_handler.clear();
                            self.state.date_input_error = None;
                            self.state.selected_date = date;
                            self.state.get_or_create_daily_log(date);
                            self.state.current_screen = AppScreen::DailyView;
                        }
                    }
                    Err(_) => {
                        self.state.date_input_error = Some("Invalid date format".to_string());
                    }
                }
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.date_input_error = None;
                self.state.current_screen = AppScreen::Home;
            }
            KeyCode::Char(c) => {
                if c.is_ascii_digit() || c == '.' {
                    self.state.date_input_error = None;
                    self.input_handler.handle_text_input(key);
                }
            }
            _ => {
                self.state.date_input_error = None;
                self.input_handler.handle_text_input(key);
            }
        }
        Ok(())
    }

    async fn handle_navigation_input(&mut self, key: KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<()> {
        // Shift+J/K switches section focus in DailyView
        if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            match key {
                KeyCode::Char('J') => {
                    if matches!(self.state.current_screen, AppScreen::DailyView) {
                        // Reset scroll when leaving expanded sections
                        self.state.strength_mobility_scroll = 0;
                        self.state.notes_scroll = 0;
                        self.state.focused_section = SectionNavigator::move_focus_down(&self.state.focused_section);
                    }
                    return Ok(());
                }
                KeyCode::Char('K') => {
                    if matches!(self.state.current_screen, AppScreen::DailyView) {
                        // Reset scroll when leaving expanded sections
                        self.state.strength_mobility_scroll = 0;
                        self.state.notes_scroll = 0;
                        self.state.focused_section = SectionNavigator::move_focus_up(&self.state.focused_section);
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        match key {
            KeyCode::Char('q') => {
                self.state.current_screen = AppScreen::Syncing;
            }
            KeyCode::Tab => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.focused_section = SectionNavigator::toggle_internal_focus(&self.state.focused_section);
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.move_food_selection_down(),
                        FocusedSection::Sokay => self.move_sokay_selection_down(),
                        FocusedSection::StrengthMobility => {
                            let max = self.strength_mobility_max_scroll();
                            self.state.strength_mobility_scroll =
                                self.state.strength_mobility_scroll.saturating_add(1).min(max);
                        }
                        FocusedSection::Notes => {
                            let max = self.notes_max_scroll();
                            self.state.notes_scroll =
                                self.state.notes_scroll.saturating_add(1).min(max);
                        }
                        _ => {}
                    }
                } else {
                    self.move_selection_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.move_food_selection_up(),
                        FocusedSection::Sokay => self.move_sokay_selection_up(),
                        FocusedSection::StrengthMobility => {
                            self.state.strength_mobility_scroll = self.state.strength_mobility_scroll.saturating_sub(1);
                        }
                        FocusedSection::Notes => {
                            self.state.notes_scroll = self.state.notes_scroll.saturating_sub(1);
                        }
                        _ => {}
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
            KeyCode::Char('d') => {
                if matches!(self.state.current_screen, AppScreen::Home) {
                    self.handle_delete_day_confirmation();
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    use crate::models::DeleteTarget;
                    match self.state.focused_section {
                        FocusedSection::FoodItems => {
                            if self.state.food_list_focused
                                && let Some(selected_index) = self.food_list_state.selected() {
                                    self.state.current_screen = AppScreen::ConfirmDelete(DeleteTarget::Food(selected_index));
                                }
                        }
                        FocusedSection::Sokay => {
                            if self.state.sokay_list_focused
                                && let Some(selected_index) = self.sokay_list_state.selected() {
                                    self.state.current_screen = AppScreen::ConfirmDelete(DeleteTarget::Sokay(selected_index));
                                }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char('f') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::AddFood;
                }
            }
            KeyCode::Char('e') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    match self.state.focused_section {
                        FocusedSection::FoodItems => self.handle_edit_food(),
                        FocusedSection::Sokay => self.handle_edit_sokay(),
                        _ => {}
                    }
                }
            }
            KeyCode::Char('w') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_weight();
                }
            }
            KeyCode::Char('s') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_waist();
                }
            }
            KeyCode::Char('t') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_strength_mobility();
                }
            }
            KeyCode::Char('n') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.selected_date = chrono::Local::now().date_naive();
                    self.state.get_or_create_daily_log(self.state.selected_date);
                    self.state.current_screen = AppScreen::DailyView;
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_notes();
                }
            }
            KeyCode::Char('m') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_miles();
                }
            }
            KeyCode::Char('l') => {
                if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.state.current_screen = AppScreen::Home;
                } else if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.handle_edit_elevation();
                }
            }
            KeyCode::Char('c') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::AddSokay;
                } else if matches!(self.state.current_screen, AppScreen::Startup) {
                    self.open_config_sync();
                }
            }
            KeyCode::Char('S') => {
                if matches!(self.state.current_screen, AppScreen::Home | AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::Startup;
                }
            }
            KeyCode::Char('a') => {
                if matches!(self.state.current_screen, AppScreen::Home | AppScreen::Startup) {
                    self.input_handler.clear();
                    self.state.date_input_error = None;
                    self.state.current_screen = AppScreen::DateInput;
                }
            }
            KeyCode::Char(' ') => {
                if matches!(self.state.current_screen, AppScreen::DailyView) {
                    self.state.current_screen = AppScreen::ShortcutsHelp;
                } else if matches!(self.state.current_screen, AppScreen::ShortcutsHelp) {
                    self.state.current_screen = AppScreen::DailyView;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn open_config_sync(&mut self) {
        self.config_url_buffer = self.config.sync.db_url.clone();
        self.config_token_buffer = String::new();
        self.config_sync_enabled = self.config.sync.enabled;
        self.state.config_sync_focused_field = ConfigSyncField::DbUrl;
        self.state.config_sync_status = None;
        self.input_handler.set_input(self.config.sync.db_url.clone());
        self.state.current_screen = AppScreen::ConfigSync;
    }

    async fn handle_config_sync_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Tab => {
                // Save current field buffer before switching
                match self.state.config_sync_focused_field {
                    ConfigSyncField::DbUrl => {
                        self.config_url_buffer = self.input_handler.input_buffer.clone();
                        self.state.config_sync_focused_field = ConfigSyncField::AuthToken;
                        self.input_handler.set_input(self.config_token_buffer.clone());
                    }
                    ConfigSyncField::AuthToken => {
                        self.config_token_buffer = self.input_handler.input_buffer.clone();
                        self.state.config_sync_focused_field = ConfigSyncField::EnableToggle;
                        self.input_handler.clear();
                    }
                    ConfigSyncField::EnableToggle => {
                        self.state.config_sync_focused_field = ConfigSyncField::DbUrl;
                        self.input_handler.set_input(self.config_url_buffer.clone());
                    }
                }
            }
            KeyCode::Enter => {
                // Save current field buffer
                match self.state.config_sync_focused_field {
                    ConfigSyncField::DbUrl => {
                        self.config_url_buffer = self.input_handler.input_buffer.clone();
                    }
                    ConfigSyncField::AuthToken => {
                        self.config_token_buffer = self.input_handler.input_buffer.clone();
                    }
                    ConfigSyncField::EnableToggle => {}
                }

                // Build updated config
                let token = if self.config_token_buffer.is_empty() {
                    self.config.sync.auth_token.clone()
                } else {
                    self.config_token_buffer.clone()
                };

                self.config.sync.db_url = self.config_url_buffer.clone();
                self.config.sync.auth_token = token;
                self.config.sync.enabled = self.config_sync_enabled;

                match self.config.save() {
                    Ok(()) => {
                        self.state.config_sync_status = Some("Saved!".to_string());
                    }
                    Err(e) => {
                        self.state.config_sync_status =
                            Some(format!("Error: {}", e));
                        return Ok(());
                    }
                }

                // If newly configured, spawn background cloud connection
                if self.config.sync.is_configured() {
                    let db_manager_clone = Arc::clone(&self.db_manager);
                    let home_dir = dirs::home_dir().context("Could not find home directory")?;
                    let mountains_dir = home_dir.join(".mountains");
                    let url = self.config.sync.db_url.clone();
                    let token = self.config.sync.auth_token.clone();
                    tokio::spawn(async move {
                        let db_path = mountains_dir.join("mountains.db");
                        if let Some(db_path_str) = db_path.to_str() {
                            let mut db = db_manager_clone.write().await;
                            let _ = db
                                .upgrade_to_remote_replica(db_path_str, url, token)
                                .await;
                        }
                    });
                }

                self.input_handler.clear();
                self.state.current_screen = AppScreen::Startup;
            }
            KeyCode::Esc => {
                self.input_handler.clear();
                self.state.config_sync_status = None;
                self.state.current_screen = AppScreen::Startup;
            }
            _ => {
                match self.state.config_sync_focused_field {
                    ConfigSyncField::DbUrl | ConfigSyncField::AuthToken => {
                        self.input_handler.handle_text_input(key);
                    }
                    ConfigSyncField::EnableToggle => {
                        if matches!(key, KeyCode::Char(' ')) {
                            self.config_sync_enabled = !self.config_sync_enabled;
                        }
                    }
                }
            }
        }
        Ok(())
    }

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
                self.state.current_screen = AppScreen::AddFood;
            }
            FocusedSection::Sokay => {
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

    fn ui(&mut self, f: &mut Frame) {
        self.state.frame_width = f.area().width;
        self.state.frame_height = f.area().height;
        match self.state.current_screen {
            AppScreen::Startup => {
                screens::render_startup_screen(f, &self.state);
            }
            AppScreen::Home => {
                screens::render_home_screen(f, &self.state, &mut self.list_state, &self.sync_status);
            }
            AppScreen::DailyView => {
                screens::render_daily_view_screen(f, &self.state, &mut self.food_list_state, &mut self.sokay_list_state, &self.sync_status, None);
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
            AppScreen::InputField(field_type) => {
                use crate::models::field_accessor::FieldType;
                match field_type {
                    // Numeric fields edit in place inside their daily-view row.
                    FieldType::Weight | FieldType::Waist | FieldType::Miles | FieldType::Elevation => {
                        let edit = screens::InPlaceEdit {
                            field: field_type,
                            buffer: &self.input_handler.input_buffer,
                            cursor: self.input_handler.cursor_position,
                        };
                        screens::render_daily_view_screen(
                            f, &self.state, &mut self.food_list_state,
                            &mut self.sokay_list_state, &self.sync_status, Some(edit),
                        );
                    }
                    FieldType::StrengthMobility => screens::render_edit_strength_mobility_screen(
                        f, &self.state, &mut self.food_list_state, &mut self.sokay_list_state,
                        &self.sync_status, &self.input_handler.input_buffer, self.input_handler.cursor_position,
                    ),
                    FieldType::Notes => screens::render_edit_notes_screen(
                        f, &self.state, &mut self.food_list_state, &mut self.sokay_list_state,
                        &self.sync_status, &self.input_handler.input_buffer, self.input_handler.cursor_position,
                    ),
                }
            }
            AppScreen::ConfirmDelete(target) => {
                use crate::models::DeleteTarget;
                match target {
                    DeleteTarget::Day => {
                        screens::render_confirm_delete_day_screen(f, self.state.selected_date);
                    }
                    DeleteTarget::Food(food_index) => {
                        screens::render_confirm_delete_food_screen(
                            f, &self.state, &mut self.food_list_state, &mut self.sokay_list_state,
                            &self.sync_status, food_index,
                        );
                    }
                    DeleteTarget::Sokay(sokay_index) => {
                        screens::render_confirm_delete_sokay_screen(
                            f, &self.state, &mut self.food_list_state, &mut self.sokay_list_state,
                            &self.sync_status, sokay_index,
                        );
                    }
                }
            }
            AppScreen::DateInput => {
                screens::render_date_input_screen(
                    f,
                    &self.state,
                    &mut self.list_state,
                    &self.sync_status,
                    &self.input_handler.input_buffer,
                    self.input_handler.cursor_position,
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
            AppScreen::ConfigSync => {
                screens::render_config_sync_screen(
                    f,
                    &self.state,
                    &self.config_url_buffer,
                    &self.config_token_buffer,
                    self.config_sync_enabled,
                    !self.config.sync.auth_token.is_empty(),
                );
            }
            AppScreen::Syncing => {
                screens::render_syncing_screen(f, &self.sync_status);
            }
        }
    }

    fn move_selection_down(&mut self) {
        if self.list_state.selected().is_none() && !self.state.daily_logs.is_empty() {
            self.list_state.select(Some(0));
        } else {
            let new_selection = NavigationHandler::move_selection_down(
                self.list_state.selected(),
                self.state.daily_logs.len(),
            );
            self.list_state.select(new_selection);
        }
    }

    fn move_selection_up(&mut self) {
        if self.list_state.selected().is_none() && !self.state.daily_logs.is_empty() {
            self.list_state.select(Some(self.state.daily_logs.len() - 1));
        } else {
            let new_selection = NavigationHandler::move_selection_up(
                self.list_state.selected(),
                self.state.daily_logs.len(),
            );
            self.list_state.select(new_selection);
        }
    }

    fn move_food_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if !self.state.food_list_focused && !log.food_entries.is_empty() {
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(0));
            } else {
                let new_selection = NavigationHandler::move_selection_down(
                    self.food_list_state.selected(),
                    log.food_entries.len(),
                );
                self.food_list_state.select(new_selection);
            }
        }
    }

    fn move_food_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let list_len = log.food_entries.len();
            let is_focused = self.state.food_list_focused;

            if !is_focused && list_len > 0 {
                self.state.food_list_focused = true;
                self.food_list_state.select(Some(list_len - 1));
            } else {
                let new_selection = NavigationHandler::move_selection_up(
                    self.food_list_state.selected(),
                    list_len,
                );
                self.food_list_state.select(new_selection);
            }
        }
    }

    fn move_sokay_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if !self.state.sokay_list_focused && !log.sokay_entries.is_empty() {
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(0));
            } else {
                let new_selection = NavigationHandler::move_selection_down(
                    self.sokay_list_state.selected(),
                    log.sokay_entries.len(),
                );
                self.sokay_list_state.select(new_selection);
            }
        }
    }

    fn move_sokay_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            let list_len = log.sokay_entries.len();
            let is_focused = self.state.sokay_list_focused;

            if !is_focused && list_len > 0 {
                self.state.sokay_list_focused = true;
                self.sokay_list_state.select(Some(list_len - 1));
            } else {
                let new_selection = NavigationHandler::move_selection_up(
                    self.sokay_list_state.selected(),
                    list_len,
                );
                self.sokay_list_state.select(new_selection);
            }
        }
    }

    fn handle_enter(&mut self) {
        if let AppScreen::Home = self.state.current_screen {
            ActionHandler::handle_home_enter(&mut self.state, self.list_state.selected());
        }
    }

    fn strength_mobility_max_scroll(&self) -> u16 {
        let text = self
            .state
            .get_daily_log(self.state.selected_date)
            .and_then(|l| l.strength_mobility.clone())
            .unwrap_or_default();
        screens::max_scroll_offset(&text, self.state.frame_width, self.state.frame_height)
    }

    fn notes_max_scroll(&self) -> u16 {
        let text = self
            .state
            .get_daily_log(self.state.selected_date)
            .and_then(|l| l.notes.clone())
            .unwrap_or_default();
        screens::max_scroll_offset(&text, self.state.frame_width, self.state.frame_height)
    }

    fn handle_escape(&mut self) {
        match self.state.current_screen {
            AppScreen::Home => {
                self.list_state.select(None);
            }
            AppScreen::ShortcutsHelp => {
                self.state.current_screen = AppScreen::DailyView;
            }
            AppScreen::DailyView => {
                match self.state.focused_section {
                    FocusedSection::FoodItems => {
                        if self.state.food_list_focused {
                            self.state.food_list_focused = false;
                            self.food_list_state.select(None);
                        } else {
                            self.state.current_screen = AppScreen::Home;
                        }
                    }
                    FocusedSection::Sokay => {
                        if self.state.sokay_list_focused {
                            self.state.sokay_list_focused = false;
                            self.sokay_list_state.select(None);
                        } else {
                            self.state.current_screen = AppScreen::Home;
                        }
                    }
                    _ => {
                        self.state.current_screen = AppScreen::Home;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_edit_food(&mut self) {
        if !self.state.food_list_focused {
            return;
        }

        if let Some(selected_index) = self.food_list_state.selected()
            && let Some(current_name) = ActionHandler::start_edit_food(&self.state, selected_index)
            {
                self.input_handler.set_input(current_name);
                self.state.current_screen = AppScreen::EditFood(selected_index);
            }
    }

    fn handle_edit_weight(&mut self) {
        use crate::models::field_accessor::FieldType;
        let current_weight = ActionHandler::start_edit_weight(&self.state);
        self.input_handler.set_input(current_weight);
        self.state.current_screen = AppScreen::InputField(FieldType::Weight);
    }

    fn handle_edit_waist(&mut self) {
        use crate::models::field_accessor::FieldType;
        let current_waist = ActionHandler::start_edit_waist(&self.state);
        self.input_handler.set_input(current_waist);
        self.state.current_screen = AppScreen::InputField(FieldType::Waist);
    }

    fn handle_edit_strength_mobility(&mut self) {
        use crate::models::field_accessor::FieldType;
        let current_sm = ActionHandler::start_edit_strength_mobility(&self.state);
        self.input_handler.set_input(current_sm);
        self.state.current_screen = AppScreen::InputField(FieldType::StrengthMobility);
    }

    fn handle_edit_notes(&mut self) {
        use crate::models::field_accessor::FieldType;
        let current_notes = ActionHandler::start_edit_notes(&self.state);
        self.input_handler.set_input(current_notes);
        self.state.current_screen = AppScreen::InputField(FieldType::Notes);
    }

    fn handle_edit_miles(&mut self) {
        use crate::models::field_accessor::FieldType;
        let current_miles = ActionHandler::start_edit_miles(&self.state);
        self.input_handler.set_input(current_miles);
        self.state.current_screen = AppScreen::InputField(FieldType::Miles);
    }

    fn handle_edit_elevation(&mut self) {
        use crate::models::field_accessor::FieldType;
        let current_elevation = ActionHandler::start_edit_elevation(&self.state);
        self.input_handler.set_input(current_elevation);
        self.state.current_screen = AppScreen::InputField(FieldType::Elevation);
    }

    fn handle_edit_sokay(&mut self) {
        if !self.state.sokay_list_focused {
            return;
        }

        if let Some(selected_index) = self.sokay_list_state.selected()
            && let Some(current_text) = ActionHandler::start_edit_sokay(&self.state, selected_index)
            {
                self.input_handler.set_input(current_text);
                self.state.current_screen = AppScreen::EditSokay(selected_index);
            }
    }

    fn handle_delete_day_confirmation(&mut self) {
        use crate::models::DeleteTarget;
        if let Some(selected_index) = self.list_state.selected()
            && selected_index < self.state.daily_logs.len() {
                self.state.selected_date = self.state.daily_logs[selected_index].date;
                self.state.current_screen = AppScreen::ConfirmDelete(DeleteTarget::Day);
            }
    }

    /// Generic handler for all delete confirmations - consolidates 3 separate handlers
    async fn handle_delete_confirmation_input(
        &mut self,
        key: KeyCode,
        target: crate::models::DeleteTarget,
    ) -> Result<()> {
        use crate::models::DeleteTarget;

        match key {
            KeyCode::Char('y') => {
                match target {
                    DeleteTarget::Day => {
                        let date_to_delete = self.state.selected_date;
                        {
                            let mut db = self.db_manager.write().await;
                            ActionHandler::delete_daily_log(
                                &mut self.state,
                                &mut db,
                                &self.file_manager,
                                date_to_delete,
                            )
                            .await?;
                        }
                        self.state.current_screen = AppScreen::Home;
                        self.list_state.select(None);
                    }
                    DeleteTarget::Food(food_index) => {
                        if let Some(log) = ActionHandler::delete_food_entry(&mut self.state, food_index) {
                            if let Some(current_log) = self.state.get_daily_log(self.state.selected_date) {
                                if current_log.food_entries.is_empty() {
                                    self.food_list_state.select(None);
                                } else if food_index >= current_log.food_entries.len() {
                                    self.food_list_state.select(Some(current_log.food_entries.len() - 1));
                                }
                            }
                            self.state.current_screen = AppScreen::DailyView;

                            let db_manager = Arc::clone(&self.db_manager);
                            let file_manager = self.file_manager.clone();
                            tokio::spawn(async move {
                                ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                            });
                        } else {
                            self.state.current_screen = AppScreen::DailyView;
                        }
                    }
                    DeleteTarget::Sokay(sokay_index) => {
                        if let Some(log) = ActionHandler::delete_sokay_entry(&mut self.state, sokay_index) {
                            if let Some(current_log) = self.state.get_daily_log(self.state.selected_date) {
                                if current_log.sokay_entries.is_empty() {
                                    self.sokay_list_state.select(None);
                                } else if sokay_index >= current_log.sokay_entries.len() {
                                    self.sokay_list_state.select(Some(current_log.sokay_entries.len() - 1));
                                }
                            }
                            self.state.current_screen = AppScreen::DailyView;

                            let db_manager = Arc::clone(&self.db_manager);
                            let file_manager = self.file_manager.clone();
                            tokio::spawn(async move {
                                ActionHandler::persist_daily_log(db_manager, &file_manager, log).await;
                            });
                        } else {
                            self.state.current_screen = AppScreen::DailyView;
                        }
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                match target {
                    DeleteTarget::Day => {
                        self.state.current_screen = AppScreen::Home;
                    }
                    DeleteTarget::Food(_) | DeleteTarget::Sokay(_) => {
                        self.state.current_screen = AppScreen::DailyView;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn update_sync_status(&mut self) {
        let db = self.db_manager.read().await;
        let state = db.get_connection_state().await;

        self.sync_status = match state {
            ConnectionState::Disconnected => "⚪ Offline".to_string(),
            ConnectionState::Connected => "✓ Synced".to_string(),
            ConnectionState::Error(_) => "⚠️ Sync Error".to_string(),
        };
    }

    /// Performs shutdown sync and updates sync_status with result
    pub async fn perform_shutdown_sync(&mut self) {
        let db = self.db_manager.read().await;
        let connection_state = db.get_connection_state().await;

        match connection_state {
            ConnectionState::Connected => {
                self.sync_status = "Syncing with Turso Cloud...".to_string();
                drop(db);

                let db = self.db_manager.read().await;
                match db.sync_now().await {
                    Ok(_) => {
                        self.sync_status = "Sync complete!".to_string();
                    }
                    Err(_) => {
                        self.sync_status = "Offline - changes will sync when network is available".to_string();
                    }
                }
            }
            _ => {
                self.sync_status = "Offline - changes will sync when network is available".to_string();
            }
        }

        self.should_quit = true;
    }
}
