/// Mountains Food Tracker - Main Entry Point
///
/// This is the main entry point for the Mountains Food Tracker application.
///
/// The main.rs file in Rust applications is responsible for:
/// 1. Setting up the application environment
/// 2. Initializing necessary components (like the terminal)
/// 3. Running the main application loop
/// 4. Cleaning up resources when the application exits
///
/// This file is kept minimal to focus only on application setup and teardown,
/// while the actual application logic is organized in separate modules.
// Module declarations
// In Rust, you need to declare modules before you can use them
mod app; // Main application logic
mod db_manager; // Database operations with Turso
mod events; // Event handling modules
mod file_manager; // File I/O operations
mod models; // Data structures and types
mod ui; // User interface modules

// Import statements
// We import only what we need for the main function
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::app::App;

/// Main function - Entry point of the application
///
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

/*
Error handling demonstration

This application uses the `anyhow` crate for error handling, which provides:
- Easy error propagation with the `?` operator
- Rich error context and chaining
- Simple Result<T> types without specifying specific error types

The `?` operator is Rust's error propagation operator. When used after
a function call that returns Result<T, E>, it:
- If Ok(value): unwraps and returns the value
- If Err(error): immediately returns the error from the current function

This allows for clean, readable error handling without explicit match statements.

Module organization explanation

The module structure follows Rust best practices:

- main.rs: Application entry point and setup
- app.rs: Core application logic and state management
- models.rs: Data structures and business logic types
- file_manager.rs: File I/O operations and data persistence
- ui/: UI-related modules organized in a subdirectory
  - mod.rs: Module declarations for the ui module
  - screens.rs: Screen rendering logic
  - components.rs: Reusable UI components and utilities
- events/: Event handling modules
  - mod.rs: Module declarations for the events module
  - handlers.rs: Input and keyboard event handling logic

This organization provides:
- Clear separation of concerns
- Easy navigation and understanding
- Scalability as the application grows
- Testability through isolated modules
*/
