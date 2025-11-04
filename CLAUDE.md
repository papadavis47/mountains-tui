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
- **Add, edit, and delete** entries for food and sokay items
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
- `q` - Quit application

### Daily View

- `↑/↓` or `j/k` - Navigate between food items
- `a` - Add new food item
- `e` - Edit selected food item
- `d` - Delete selected food item
- `w` - Edit weight measurement
- `s` - Edit waist measurement
- `m` - Edit miles covered
- `l` - Edit elevation gain
- `c` - View/manage sokay entries
- `t` - Edit strength & mobility exercises
- `n` - Edit daily notes
- `Esc` - Back to home screen

### Sokay View

- `↑/↓` or `j/k` - Navigate between sokay entries
- `a` - Add new sokay entry
- `e` - Edit selected sokay entry
- `d` - Delete selected sokay entry
- `Esc` - Back to daily view

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
- `Ctrl+J` - Insert newline (Enter saves the exercises)
- `Enter` - Save exercises
- `Esc` - Cancel and return

### Edit Notes Screen

- **Multi-line text input** with cursor support
- `←/→/↑/↓` - Move cursor
- `Home/End` - Jump to beginning/end of line
- `Ctrl+J` - Insert newline (Enter saves the notes)
- `Enter` - Save notes
- `Esc` - Cancel and return

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
- **Miles:** 3.2 mi
- **Elevation:** 450 ft
- **Sokay:** 5 total

## Food

- **Oatmeal** - with blueberries
- **Chicken Salad**
- **Green Tea**

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

### Latest Session (Code Cleanup)
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
- ✅ **Multi-line editing** - same as notes with Ctrl+J for newlines
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
- **State management** - AppScreen enum for view routing (16 different screens)
- **Dual persistence** - libsql database (primary) + markdown files (backup)
- **Cloud sync** - Background sync with Turso Cloud, local-first approach
- **Input handling** - Specialized handlers for text, numeric, integer, and multi-line input
- **Modular design** - Separated concerns (models, events, ui, database, file management)
- **Responsive UI** - Terminal size adaptation with ratatui layout system
- **Data integrity** - Database transactions for atomic operations
- **Error handling** - anyhow for ergonomic error propagation

### Key Data Structures

- **DailyLog** - Main data model with food_entries, measurements, sokay_entries, strength_mobility, notes
- **AppState** - Application state with daily_logs cache and current screen/selection
- **InputHandler** - Cursor position tracking and input validation
- **DbManager** - Async database operations with automatic sync
- **FileManager** - Markdown serialization/deserialization for backups

# important-instruction-reminders

Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (\*.md) or README files. Only create documentation files if explicitly requested by the User.
- always update @CLAUDE.md with changes
- clean up dead code as the app evolves and changes are made