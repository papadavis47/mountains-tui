# Mountains Training Log

A terminal-based training and nutrition tracking application built with Rust and ratatui.

## Project Overview

This is a TUI (Terminal User Interface) application for tracking daily training activities, nutrition, and body measurements with the following features:

- **Daily food logging** with date navigation
- **Body measurements** - weight and waist size tracking
- **Activity tracking** - miles covered (walking/hiking/running) and elevation gain
- **Sokay tracking** - accountability for unhealthy food choices with cumulative counting
- **Strength & mobility tracking** - multi-line text field for logging exercises
- **Daily notes** for observations and reflections
- **Full CRUD operations** - add, edit, and delete entries for food and sokay items, plus delete entire days
- **Cursor-enabled text input** with arrow key navigation
- **Dual persistence** - Turso Cloud database (primary) with markdown file backups
- **Cloud sync** - automatic background syncing with Turso Cloud
- **Clean, responsive interface** with keyboard shortcuts

## Technology Stack

- **Rust** - Systems programming language
- **ratatui** - Terminal UI framework ([docs](https://docs.rs/ratatui/latest/ratatui/index.html))
- **crossterm** - Cross-platform terminal manipulation
- **chrono** - Date/time handling
- **serde** - Serialization for data persistence
- **libsql** - Embedded database with Turso Cloud sync
- **tokio** - Async runtime for database operations

## Key Controls

### Home Screen

- `↑/↓` or `j/k` - Navigate between dates
- `Enter` - Select date or create today's log
- `D` - Delete selected day (with confirmation)
- `q` - Quit application

### Daily View

The daily view shows two scrollable lists: **Food Items** and **Sokay entries**. Both lists are visible on screen with equal size.

#### Focus and Navigation
- `Shift+J` - Switch focus to Sokay list (move down)
- `Shift+K` - Switch focus to Food list (move up)
- `↑/↓` or `j/k` - Navigate within the focused list
- The focused list has a **bright colored border** (yellow for Food, magenta for Sokay)
- The non-focused list has a **dimmed gray border**

#### Actions
- `f` - Add new food item
- `c` - Add new sokay entry
- `e` - Edit selected item in focused list
- `d` - Delete selected item in focused list
- `w` - Edit weight measurement
- `s` - Edit waist measurement
- `m` - Edit miles covered
- `l` - Edit elevation gain
- `t` - Edit strength & mobility exercises
- `n` - Edit daily notes
- `Esc` - Back to home screen

### Add/Edit Food Screens

- **Text input** with full cursor support
- `←/→` - Move cursor within text
- `Home/End` - Jump to beginning/end
- `Backspace/Delete` - Remove characters
- `Enter` - Save entry
- `Esc` - Cancel and return

### Edit Measurements Screens

- **Numeric input** (weight, waist, miles: decimal; elevation: integer only)
- `←/→` - Move cursor within text
- `Home/End` - Jump to beginning/end
- `Backspace/Delete` - Remove characters
- `Enter` - Save measurement
- `Esc` - Cancel and return

### Edit Strength & Mobility Screen

- **Multi-line text input** with cursor support
- `←/→/↑/↓` - Move cursor
- `Home/End` - Jump to beginning/end of line
- `Enter` - Save exercises
- `Esc` - Cancel and return

### Edit Notes Screen

- **Multi-line text input** with cursor support
- `←/→/↑/↓` - Move cursor
- `Home/End` - Jump to beginning/end of line
- `Enter` - Save notes
- `Esc` - Cancel and return

### Delete Day Confirmation Screen

- `Y` - Confirm deletion (uppercase Y required for safety)
- `n` or `Esc` - Cancel deletion and return to home screen

## File Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Main App struct and event loop
├── models.rs            # Data structures (FoodEntry, DailyLog, AppState, AppScreen)
├── db_manager.rs        # Database operations with Turso Cloud sync
├── file_manager.rs      # Markdown file I/O for backups
├── events/
│   └── handlers.rs      # Event handlers (InputHandler, ActionHandler)
└── ui/
    ├── mod.rs           # UI module
    ├── components.rs    # Reusable UI components
    └── screens.rs       # Screen rendering functions

Data storage:
- Database: ~/.mountains/mountains.db (local libsql database)
- Backups: ~/.mountains/mtslog-MM.DD.YYYY.md (markdown files)
- Cloud: Synced to Turso Cloud every 60 seconds
```

### Example Data File Format:

```markdown
# Mountains Training Log - January 09, 2025

## Measurements

- **Weight:** 175.5 lbs
- **Waist:** 34.2 inches

## Food

- Oatmeal
- Chicken Salad
- Green Tea

## Running

- **Miles:** 3.2 mi
- **Elevation:** 450 ft

## Sokay

- Coca Cola
- Chocolate bar

## Strength & Mobility

Pull-ups: 3x8
Push-ups: 3x15
Hip mobility stretches: 10 minutes

## Notes
Feeling strong today. Good hike in the morning.
```

## Development Commands

- `cargo run` - Run the application
- `cargo check` - Check for compilation errors
- `cargo build --release` - Build optimized binary

## Useful Links

- [ratatui Documentation](https://docs.rs/ratatui/latest/ratatui/index.html)
- [ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [crossterm Documentation](https://docs.rs/crossterm/latest/crossterm/)

## Recent Improvements

### Latest Session (Dual-List DailyView with Focus Switching)
- ✅ **Scrollable Sokay list** - converted from Paragraph to List widget with full navigation support
- ✅ **Equal-sized lists** - Sokay section now matches Food section size (Constraint::Min(4))
- ✅ **Focus switching** - Shift+J/K switches focus between Food and Sokay lists
- ✅ **Visual focus indicator** - bright colored borders (yellow/magenta) show focused list, dimmed gray for unfocused
- ✅ **Smart edit/delete** - 'e' and 'd' keys work on the currently focused list
- ✅ **Changed keybindings** - 'f' adds food (was 'a'), 'c' adds sokay directly (was view screen)
- ✅ **Removed SokayView screen** - eliminated separate view, all management now in DailyView
- ✅ **FocusedList state tracking** - new enum tracks which list (Food/Sokay) has user's focus
- ✅ **Focus starts on Food** - consistent initial state when entering DailyView
- ✅ **Updated documentation** - CLAUDE.md reflects new dual-list navigation model

### Previous Session (Remove Ctrl+J Functionality)
- ✅ **Removed Ctrl+J newline insertion** - non-functional keyboard shortcut eliminated
- ✅ **Simplified multi-line editing** - Enter saves, arrow keys navigate (no special key combo needed)
- ✅ **Updated UI help text** - removed Ctrl+J references from strength & mobility and notes screens
- ✅ **Cleaned up code** - removed `handle_multiline_special_keys` method and its calls
- ✅ **Zero warnings** - application compiles cleanly

### Previous Session (Startup Performance Fix)
- ✅ **Fixed startup delay** - removed blocking remote replica connection that was causing 1-2 second hang
- ✅ **True offline-first startup** - always begins with local database, regardless of replica metadata
- ✅ **Simplified initialization** - eliminated complex conditional logic that was trying to connect to Turso Cloud synchronously
- ✅ **Background-only cloud sync** - all remote replica upgrades now happen exclusively in background task
- ✅ **Instant launch** - app now starts immediately as designed, cloud connection happens asynchronously

### Previous Session (Data Model Cleanup - Remove Food Entry Notes)
- ✅ **Removed unused notes field from FoodEntry** - legacy from when app was just a food tracker
- ✅ **Simplified data model** - food entries now just store name (notes belong to daily log only)
- ✅ **Database migration** - automatic removal of notes column from food_entries table
- ✅ **Updated markdown export** - food items display as simple list items
- ✅ **Code cleanup** - removed dead code and unused parameters throughout the codebase

### Previous Session (UI Styling Enhancements)
- ✅ **Rounded borders** - title blocks now use smooth, curved corners instead of sharp edges
- ✅ **Vertical padding** - increased spacing above and below title text for better visual balance
- ✅ **Enhanced visual polish** - improved overall aesthetic with BorderType::Rounded and Padding::vertical

### Previous Session (Offline-First Startup with Deferred Turso Sync)
- ✅ **Instant startup** - app launches immediately without waiting for cloud connection
- ✅ **Offline-first architecture** - local database initializes first, cloud connects in background
- ✅ **Background sync** - Turso Cloud connection established asynchronously after app starts
- ✅ **Connection state tracking** - real-time sync status monitoring (Disconnected, Connecting, Connected, Error)
- ✅ **UI status indicator** - sync status displayed in title bars (⚪ Offline, 🔄 Connecting..., ✓ Synced, ⚠️ Sync Error)
- ✅ **Graceful degradation** - sync operations skip when offline, app works fully offline
- ✅ **Arc<RwLock<>> wrapping** - thread-safe shared access to DbManager for background tasks
- ✅ **Zero startup delay** - eliminates network wait time that previously blocked UI initialization

### Previous Session (Delete Day Functionality)
- ✅ **Full CRUD functionality** - added ability to delete entire days
- ✅ **Delete from Home screen** - select a day and press 'D' to delete
- ✅ **Confirmation prompt** - "Are you sure?" screen with Y/n confirmation
- ✅ **Complete deletion** - removes from database, app state, and markdown backups
- ✅ **Cloud sync** - deletions are synced to Turso Cloud
- ✅ **Uppercase Y required** - safety measure to prevent accidental deletions
- ✅ **Confirmation message** - displays all data that will be deleted

### Previous Session (Critical Bug Fixes)
- ✅ **Fixed async runtime deadlock** - converted all database operations to proper async/await instead of `block_on()`
- ✅ **Fixed terminal freeze on save** - eliminated panic when saving entries by removing nested async runtime calls
- ✅ **Fixed UI message interference** - removed `eprintln!` messages that stayed on screen after TUI initialization
- ✅ **Made event loop fully async** - `run()` method and all handlers are now properly async
- ✅ **Improved app stability** - terminal now properly cleans up on exit instead of freezing
- ✅ **Zero compiler warnings** - application compiles cleanly with no warnings

### Previous Session (Code Cleanup)
- ✅ **Removed unused code** - cleaned up dead code that was never executed
- ✅ **Database methods** - removed `DbManager::load_daily_log` (app loads all logs at startup)
- ✅ **File loading methods** - removed markdown parsing functions (app is database-first now)
- ✅ **AppScreen::AddMeasurements** - removed unused enum variant
- ✅ **AppState::selected_index** - removed field (replaced by ratatui's ListState)
- ✅ **UI helper function** - removed unused `create_daily_view_layout`
- ✅ **Zero warnings** - application now compiles with no warnings

### Previous Session (Strength & Mobility Tracking)
- ✅ **Strength & mobility field** - multi-line text input for logging exercises
- ✅ **`t` keyboard shortcut** - quick access to edit strength & mobility
- ✅ **Cyan-colored display** - positioned between food log and notes
- ✅ **Multi-line editing** - same as notes with arrow key navigation
- ✅ **Database persistence** - strength_mobility column with automatic migration
- ✅ **Markdown export** - "## Strength & Mobility" section in backup files
- ✅ **Cloud sync support** - integrated with Turso Cloud sync

### Previous Session (Training Log Expansion)
- ✅ **Miles covered tracking** - track walking/hiking/running distance with decimal precision
- ✅ **Elevation gain tracking** - integer-only input for feet climbed
- ✅ **Sokay tracking system** - accountability for unhealthy food choices
- ✅ **Cumulative sokay counting** - running total across all days up to current date
- ✅ **Sokay view screen** - dedicated interface for managing sokay entries
- ✅ **Updated markdown title** - "Mountains Training Log" reflects expanded scope
- ✅ **Database schema migration** - automatic column addition for backward compatibility
- ✅ **Extended measurements display** - all tracking fields visible in daily view

### Previous Sessions
- ✅ Turso Cloud integration with local libsql database
- ✅ Dual persistence (cloud database + markdown backups)
- ✅ Automatic background sync every 60 seconds
- ✅ Daily notes with multi-line text editing
- ✅ Cursor visibility and text navigation in input fields
- ✅ Edit and delete functionality for food entries
- ✅ Proper ratatui padding instead of literal spaces
- ✅ Clean list highlighting without arrow symbols
- ✅ Weight and waist size tracking with dedicated input screens
- ✅ Keyboard shortcuts for quick measurement editing
- ✅ Modular code structure (events, ui modules)

## Architecture Notes

- **App struct** - Main application coordinator managing state, database, and UI
- **State management** - AppScreen enum for view routing (17 different screens)
- **Dual persistence** - libsql database (primary) + markdown files (backup)
- **Offline-first design** - Local database initializes instantly, cloud connection deferred to background
- **Cloud sync** - Background sync with Turso Cloud via tokio task, graceful offline handling
- **Connection state tracking** - Real-time monitoring of Turso Cloud connection status
- **Async architecture** - Fully async event loop and database operations using tokio
- **Thread-safe database** - Arc<RwLock<DbManager>> for shared access across async tasks
- **Input handling** - Specialized handlers for text, numeric, integer, and multi-line input
- **Modular design** - Separated concerns (models, events, ui, database, file management)
- **Responsive UI** - Terminal size adaptation with ratatui layout system, live sync status display
- **Data integrity** - Database transactions for atomic operations
- **Error handling** - anyhow for ergonomic error propagation

### Key Data Structures

- **DailyLog** - Main data model with food_entries, measurements, sokay_entries, strength_mobility, notes
- **AppState** - Application state with daily_logs cache and current screen/selection
- **InputHandler** - Cursor position tracking and input validation
- **DbManager** - Async database operations with deferred cloud connection and state tracking
- **ConnectionState** - Enum tracking sync status (Disconnected, Connecting, Connected, Error)
- **FileManager** - Markdown serialization/deserialization for backups

# important-instruction-reminders

Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (\*.md) or README files. Only create documentation files if explicitly requested by the User.
- always update @CLAUDE.md with changes
- clean up dead code as the app evolves and changes are made