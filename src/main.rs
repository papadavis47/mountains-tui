/// The main.rs file in Rust applications is responsible for:
/// 1. Setting up the application environment
/// 2. Initializing necessary components (like the terminal)
/// 3. Running the main application loop
/// 4. Cleaning up resources when the application exits
///
/// This file is kept minimal to focus only on application setup and teardown,
/// while the actual application logic is organized in separate modules.
///
mod app; // Main application logic
mod db_manager; // Database operations with Turso
mod events; // Event handling modules
mod file_manager; // File I/O operations
mod models; // Data structures and types
mod ui; // User interface modules

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::app::App;

/// The main function is responsible for:
/// 1. Loading environment variables for Turso configuration
/// 2. Setting up the terminal for TUI (Text User Interface) mode
/// 3. Creating and running the application
/// 4. Properly cleaning up terminal state when done
///
/// The Result<()> return type allows proper error propagation.
/// If any operation fails, the error bubbles up and can be handled.
#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    // This is required for Turso database URL and auth token
    dotenvy::dotenv().ok(); // ok() makes it optional - won't fail if .env doesn't exist
    // Terminal setup phase
    // These operations prepare the terminal for TUI mode
    setup_terminal()?;

    // Create the terminal backend
    // CrosstermBackend works across different operating systems
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Application execution phase
    // We use a separate scope here so that app is dropped before cleanup
    let result = {
        // Create and run the application (async to initialize database)
        let mut app = App::new().await?;
        app.run(&mut terminal).await
    };

    // Terminal cleanup phase
    // This runs regardless of whether the app succeeded or failed
    cleanup_terminal(&mut terminal)?;

    // Return the result of running the application
    // If there was an error, it will be propagated to the caller
    result
}

/// Sets up the terminal for TUI mode
///
/// This function configures the terminal to:
/// - Enable raw mode: Disables line buffering and echo
/// - Enter alternate screen: Uses a separate screen buffer
/// - Enable mouse capture: Allows mouse events (though we don't use them currently)
///
/// Raw mode is essential for terminal UIs because it allows the application
/// to receive individual keystrokes immediately rather than waiting for Enter.
fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

/// Cleans up the terminal state and restores normal mode
///
/// This function:
/// - Disables raw mode: Restores normal terminal behavior
/// - Leaves alternate screen: Returns to the main screen buffer
/// - Disables mouse capture: Stops capturing mouse events
/// - Shows the cursor: Ensures the cursor is visible after exit
///
/// This cleanup is crucial for leaving the terminal in a usable state
/// when the application exits.
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
