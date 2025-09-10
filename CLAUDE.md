# Mountains Food Tracker

A terminal-based food logging application built with Rust and ratatui.

## Project Overview

This is a TUI (Terminal User Interface) application for tracking daily food intake and body measurements with the following features:

- **Daily food logging** with date navigation
- **Weight and waist size tracking** for each day
- **Add, edit, and delete** food entries
- **Cursor-enabled text input** with arrow key navigation
- **Data persistence** to markdown files in `~/.mountains/`
- **Clean, responsive interface** with keyboard shortcuts

## Technology Stack

- **Rust** - Systems programming language
- **ratatui** - Terminal UI framework ([docs](https://docs.rs/ratatui/latest/ratatui/index.html))
- **crossterm** - Cross-platform terminal manipulation
- **chrono** - Date/time handling
- **serde** - Serialization for data persistence

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
- `Esc` - Back to home screen

### Add/Edit Food Screens

- **Text input** with full cursor support
- `←/→` - Move cursor within text
- `Home/End` - Jump to beginning/end
- `Backspace/Delete` - Remove characters
- `Enter` - Save entry
- `Esc` - Cancel and return

### Edit Measurements Screens

- **Numeric input** with decimal point support
- `←/→` - Move cursor within text
- `Home/End` - Jump to beginning/end
- `Backspace/Delete` - Remove characters
- `Enter` - Save measurement
- `Esc` - Cancel and return

## File Structure

```
src/
├── main.rs         # Main application logic and UI
├── models.rs       # Data structures (FoodEntry, DailyLog, AppState)
└── file_manager.rs # File I/O for markdown persistence

Data files stored in: ~/.mountains/mtslog-MM.DD.YYYY.md
```

### Example Data File Format:

```markdown
# Food Log - January 09, 2025

## Measurements

- **Weight:** 175.5 lbs
- **Waist:** 34.2 inches

## Food

- **Oatmeal** - with blueberries
- **Chicken Salad**
- **Green Tea**
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

- ✅ Cursor visibility and text navigation in input fields
- ✅ Edit and delete functionality for food entries
- ✅ Proper ratatui padding instead of literal spaces
- ✅ Clean list highlighting without arrow symbols
- ✅ Weight and waist size tracking with dedicated input screens
- ✅ Measurements display in daily view with current values
- ✅ Keyboard shortcuts for quick measurement editing

## Architecture Notes

- **App struct** manages application state and UI rendering
- **State management** through AppScreen enum for different views
- **File persistence** using markdown format for human readability
- **Input handling** with cursor position tracking for text editing
- **Responsive UI** that adapts to terminal size

# important-instruction-reminders

Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (\*.md) or README files. Only create documentation files if explicitly requested by the User.
