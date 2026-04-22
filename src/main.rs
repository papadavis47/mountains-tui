mod app;
mod assets;
mod config;
mod db_manager;
mod elevation_stats;
mod events;
mod file_manager;
mod miles_stats;
mod models;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let data_dir = config::data_dir()?;

    // One-time migration from .env to config.toml
    config::migrate_from_env(&data_dir).ok();

    let app_config = config::AppConfig::load()?;

    setup_terminal()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Separate scope ensures app is dropped before terminal cleanup
    let result = {
        let mut app = App::new(app_config).await?;
        app.run(&mut terminal).await
    };

    cleanup_terminal(&mut terminal)?;
    result
}

/// Enables raw mode and alternate screen for TUI
fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

/// Restores terminal to normal mode and ensures cursor is visible
fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
