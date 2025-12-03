# Mountains Training Log

A terminal-based training and nutrition tracking application built with Rust and ratatui.

## Project Overview

This is a TUI (Terminal User Interface) application for tracking daily training activities, nutrition, and body measurements with the following features:

- **Startup screen** - ASCII art logo with elevation statistics (monthly 1000+ days, yearly total, active streaks)
- **Daily food logging** with date navigation
- **Body measurements** - weight and waist size tracking
- **Activity tracking** - miles covered (walking/hiking/running) and elevation gain with yearly and monthly miles totals
- **Sokay tracking** - accountability for unhealthy food choices with cumulative counting
- **Strength & mobility tracking** - multi-line text field for logging exercises
- **Daily notes** for observations and reflections
- **Full CRUD operations** - add, edit, and delete entries for food and sokay items, plus delete entire days
- **Cursor-enabled text input** with arrow key navigation
- **Dual persistence** - Turso Cloud database (primary) with markdown file backups
- **Cloud sync** - syncs on startup and shutdown with visual progress indicator
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

### Startup Screen

The startup screen appears every time the application launches. It displays:

- **ASCII art logo** - "MOUNTAINS" in large text art (centered)
- **Subtitle** - "For Inspiration and Mindfulness"
- **Monthly 1000+ days** - Count of days in current month with ≥1000 feet elevation
- **Yearly total** - Total feet of elevation gain for the current calendar year
- **Streak tracker** - Current consecutive days with ≥1000 feet elevation (minimum 2 days)

#### Startup Screen Controls:

- `N` - Go to Today's Log (creates if doesn't exist and opens DailyView)
- `L` - Go to Log List (opens Home screen with all daily logs)
- `q` - Quit application (syncs with Turso Cloud if online)

### Home Screen

The home screen starts with the list **unfocused** (no item highlighted).

#### When list is unfocused:

- `Enter` - Go to today's log (creating if needed)
- `j` or `↓` - Focus list and select first item (most recent date)
- `k` or `↑` - Focus list and select last item (oldest date)

#### When list is focused:

- `↑/↓` or `j/k` - Navigate between dates
- `Enter` - Go to selected date's log
- `Esc` - Unfocus the list (remove highlight)
- `D` - Delete selected day (with confirmation)

#### Always available:

- `S` - Go to Startup Screen
- `q` - Quit application (syncs with Turso Cloud if online)

### Daily View

The daily view shows all sections for tracking your training day: **Measurements**, **Running**, **Food Items**, **Sokay**, **Strength & Mobility**, and **Notes**.

#### Section Navigation

- `Shift+J` - Move focus to next section (down)
- `Shift+K` - Move focus to previous section (up)
- **Navigation order:** Measurements → Running → Food Items → Sokay → Strength & Mobility → Notes → (wraps to Measurements)
- The focused section has a **bright colored border** (yellow, red, cyan, magenta, or green depending on section)
- Unfocused sections have **dimmed gray borders**

#### Field Navigation (Measurements & Running sections only)

- `Tab` - Toggle between fields within a section
  - **Measurements:** Weight ↔ Waist
  - **Running:** Miles ↔ Elevation
- The focused field is indicated with a **► symbol**

#### List Navigation (Food Items & Sokay sections only)

Lists start **unfocused** (no item highlighted) for quick access to adding new entries.

**When list is unfocused:**

- `j/↓` - Focus first item in the list
- `k/↑` - Focus last item in the list
- `Enter` - Add new entry (same as when focused)
- `E` and `D` - Do nothing (no item to edit/delete)
- `Esc` - Return to home screen

**When list is focused (item highlighted):**

- `↑/↓` or `j/k` - Navigate between items
- `E` - Edit selected item
- `D` - Delete selected item (with confirmation)
- `Esc` - Unfocus the item (remove highlight), next Esc returns to home

#### Expanded View (Strength & Mobility and Notes sections only)

When you navigate to the **Strength & Mobility** or **Notes** section, the section automatically expands if needed to show the full text content.

**Expansion behavior:**

- Expands **vertically upward** (bottom border stays fixed, top border moves up)
- **Overlays** sections above it (no other sections move or resize)
- Only expands if content doesn't fit in the default 4-line height
- Maximum expansion: **60% of screen height**
- If content exceeds 60%, the section becomes **scrollable**

**Scrolling:**

- `↑/k` - Scroll up (when section is expanded and has more content)
- `↓/j` - Scroll down (when section is expanded and has more content)
- Scroll position **resets to top** when navigating away with Shift+J/K
- Press `Enter` to edit (same as before)

**Use case:** Easily browse longer training notes and exercise logs without squinting at the small default section. Navigate to the section with Shift+J/K, read the full content, then press Enter if you want to edit.

#### Editing Data

- `Enter` - Edit the focused section/field or add new entry (for Food/Sokay)
  - **Measurements/Running:** Opens input for the focused field
  - **Food Items:** Opens "Add Food" dialog
  - **Sokay:** Opens "Add Sokay" dialog
  - **Strength & Mobility/Notes:** Opens editor for that section

#### Shortcuts Overlay

- `Space` - Toggle shortcuts help overlay (shows all quick access shortcuts)
  - Press Space again to close the overlay
  - Displays data entry shortcuts grouped by category (Measurements, Activity, Nutrition, Training)
  - Only works when not typing in an input field

#### Quick Access Shortcuts

These shortcuts allow quick data entry without navigating sections. Press **Space** to see the full list in an overlay.

- `w` - Edit weight measurement
- `s` - Edit waist measurement
- `m` - Edit miles covered
- `l` - Edit elevation gain
- `f` - Add new food item
- `c` - Add new sokay entry
- `t` - Edit strength & mobility exercises
- `n` - Edit daily notes
- `S` - Go to Startup Screen
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

- **Multi-line text input** with cursor support and newline insertion
- `←/→/↑/↓` - Move cursor
- `Home/End` - Jump to beginning/end of entire text
- `Alt+Enter` - Insert newline (hard line break, max 200 lines)
- `Enter` - Save exercises
- `Esc` - Cancel and return

### Edit Notes Screen

- **Multi-line text input** with cursor support and newline insertion
- `←/→/↑/↓` - Move cursor
- `Home/End` - Jump to beginning/end of entire text
- `Alt+Enter` - Insert newline (hard line break, max 200 lines)
- `Enter` - Save notes
- `Esc` - Cancel and return

### Delete Day Confirmation Screen

- `Y` - Confirm deletion (uppercase Y required for safety)
- `n` or `Esc` - Cancel deletion and return to home screen

### Syncing Screen

The syncing screen appears automatically when quitting the application (pressing `q`).

- **Online mode:** Displays a centered modal with "Syncing with Turso Cloud..." message and a progress gauge
- **Offline mode:** Shows "Offline - changes will sync when network is available" message
- The screen automatically closes after sync completes (or immediately if offline)
- Uses a visual Gauge widget to show sync progress
- Border color changes based on status:
  - **Cyan** - Syncing in progress
  - **Green** - Sync complete
  - **Orange** - Offline mode

## File Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Main App struct and event loop
├── models.rs            # Data structures (FoodEntry, DailyLog, AppState, AppScreen)
├── assets.rs            # ASCII art and UI constants
├── elevation_stats.rs   # Elevation statistics calculations
├── miles_stats.rs       # Miles statistics calculations
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
- Cloud: Synced to Turso Cloud on startup and shutdown
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

### Latest Session (Alt+Enter Newline Insertion)

- ✅ **Newline insertion** - Alt+Enter now inserts hard line breaks in multi-line text inputs
- ✅ **Applied to both editors** - Works in Strength & Mobility and Notes sections
- ✅ **200 line maximum** - Prevents excessive line creation while allowing extensive notes
- ✅ **Natural display** - Newlines appear as line breaks naturally without special indicators
- ✅ **Full persistence** - Newlines saved to database and preserved in markdown backups
- ✅ **Cursor positioning** - Cursor moves to beginning of next line after Alt+Enter
- ✅ **Existing navigation preserved** - Up/Down arrows already worked with newlines
- ✅ **Word wrapping maintained** - Automatic word wrapping continues alongside hard line breaks
- ✅ **Enter key unchanged** - Regular Enter still saves and exits as before
- ✅ **Terminal compatibility** - Alt+Enter chosen for reliable detection across different terminals
- ✅ **Shortcuts modal updated** - Alt+Enter help text added to Training section
- ✅ **Dead code cleanup** - Removed debug logging and unused Shift+Enter handler code
- ✅ **Zero compilation warnings** - Clean build with all tests passing

### Previous Session (Monthly Miles Tracking)

- ✅ **Monthly miles total** - Running section now displays cumulative miles for the current calendar month
- ✅ **Dynamic month display** - Shows current month name (e.g., "25.0 miles covered for the month of December")
- ✅ **Smart zero-state messaging** - Displays "No miles covered yet for the month of December" when monthly total is 0.0
- ✅ **Automatic reset** - Total resets to 0 at the beginning of each new month
- ✅ **Real-time updates** - Monthly total updates immediately when daily miles are entered
- ✅ **Added to miles_stats module** - Extended existing module with `calculate_monthly_miles()` function
- ✅ **Comprehensive tests** - Full test coverage for monthly miles calculation (6 tests)
- ✅ **Pipe-separated display** - Monthly total appears after yearly total: "Miles | Elevation | Yearly | Monthly"
- ✅ **Red styling** - Monthly total uses same red color as other Running section data
- ✅ **Clean integration** - Fits naturally alongside Miles, Elevation, and Yearly totals
- ✅ **Zero compilation warnings** - Clean build with proper implementation

### Previous Session (Yearly Miles Tracking)

- ✅ **Yearly miles total** - Running section now displays cumulative miles for the current calendar year
- ✅ **Dynamic year display** - Shows current year (e.g., "You have 40.1 miles covered for 2025")
- ✅ **Automatic reset** - Total resets to 0 at the beginning of each new year
- ✅ **Real-time updates** - Yearly total updates immediately when daily miles are entered
- ✅ **Separate module** - Created dedicated `miles_stats.rs` module for maintainability
- ✅ **Comprehensive tests** - Full test coverage for yearly miles calculation
- ✅ **Red styling** - Yearly total uses same red color as Miles and Elevation display
- ✅ **Clean integration** - Fits naturally alongside existing Miles and Elevation fields
- ✅ **Zero compilation warnings** - Clean build with proper implementation

### Previous Session (Expanded View for Multi-line Sections)

- ✅ **Automatic section expansion** - Strength & Mobility and Notes sections now expand vertically when focused to show full content
- ✅ **Upward expansion overlay** - Sections expand upward (bottom stays fixed), overlaying sections above without moving them
- ✅ **Smart expansion logic** - Only expands if content doesn't fit in default 4-line height
- ✅ **Maximum expansion cap** - Sections expand up to 60% of screen height, then become scrollable
- ✅ **Arrow key scrolling** - Use ↑/↓ or k/j to scroll within expanded sections when content exceeds display area
- ✅ **Automatic scroll reset** - Scroll position resets to top when navigating away with Shift+J/K
- ✅ **Read-only preview** - Expanded view is for reading; press Enter to edit as before
- ✅ **State tracking** - Added `strength_mobility_scroll` and `notes_scroll` fields to AppState
- ✅ **Helper functions** - `calculate_text_height()`, `render_strength_mobility_expanded()`, and `render_notes_expanded()`
- ✅ **Zero compilation warnings** - Clean build with proper implementation

### Previous Session (Simplified Sync - Startup & Shutdown Only)

- ✅ **Removed periodic sync** - Eliminated 4-minute interval syncing for simpler, more predictable behavior
- ✅ **Sync on quit** - App now syncs with Turso Cloud when user presses 'q' to quit
- ✅ **Visual sync modal** - Centered modal dialog with Gauge widget shows sync progress
- ✅ **Offline detection** - Gracefully handles offline mode with clear messaging
- ✅ **Smart status messages** - Different messages for syncing, complete, and offline states
- ✅ **Color-coded borders** - Cyan (syncing), Green (complete), Orange (offline)
- ✅ **Brief display pause** - 1 second pause after sync to show completion/offline status message
- ✅ **Startup sync retained** - Still syncs on startup via background task
- ✅ **Cleaner event loop** - Removed periodic sync checks and timer logic
- ✅ **Zero compilation warnings** - All changes verified with cargo check

### Previous Session (Code Comment Cleanup)

- ✅ **Removed verbose comments** - Cleaned up overly detailed and pedagogical comments across entire codebase
- ✅ **Focused on "why" not "what"** - Kept only essential doc comments that explain purpose, removed obvious inline narration
- ✅ **Simplified function documentation** - Reduced multi-paragraph explanations to concise single-line doc comments or removed them entirely
- ✅ **Files cleaned up** - main.rs, app.rs, events/handlers.rs, models.rs, elevation_stats.rs, file_manager.rs, ui/components.rs, db_manager.rs
- ✅ **Zero compilation warnings** - All changes verified with cargo check, code remains fully functional
- ✅ **Cleaner codebase** - More professional, production-ready code with minimal commentary

### Previous Session (Startup Screen Help Text Centering)

- ✅ **Centered help text** - Help text on Startup screen now centered horizontally while maintaining vertical position
- ✅ **Added `centered` parameter** - Extended `render_help()` function with optional horizontal centering control
- ✅ **Isolated change** - Only Startup screen affected; all other screens maintain left-aligned help text

### Previous Session (Startup Screen UI Polish)

- ✅ **Borderless help text** - Removed border from help text on Startup screen for cleaner, less cluttered appearance
- ✅ **Selective border control** - Added `show_border` parameter to `render_help()` function for fine-grained control
- ✅ **Maintained consistency** - All other screens retain bordered help text (only Startup screen affected)

### Previous Session (Startup Screen with Elevation Statistics)

- ✅ **New startup screen** - Displays on every app launch with ASCII art logo and motivational subtitle
- ✅ **Monthly 1000+ tracker** - Shows count of days in current calendar month with ≥1000 feet elevation gain
- ✅ **Yearly elevation total** - Displays total feet of elevation gain for current calendar year (all amounts)
- ✅ **Active streak tracking** - Shows consecutive days with ≥1000 feet elevation if streak extends to most recent logged day
- ✅ **Streak encouragement** - Displays motivational message when no active streak (minimum 2 days required)
- ✅ **Smart streak logic** - Missing data breaks streak (no entry = broken), only counts active streaks
- ✅ **Navigation shortcuts** - N key goes to today's log, L key goes to log list, S key returns to startup
- ✅ **Centered ASCII art** - "MOUNTAINS" logo displayed prominently with cyan/bold styling
- ✅ **Color-coded stats** - White for counts, green for streak, yellow for subtitle
- ✅ **New elevation_stats module** - Pure calculation functions with comprehensive tests
- ✅ **Help text format** - Uppercase actions with lowercase keys (N: Today's Log | L: Log List | S: Startup Screen)
- ✅ **S shortcut added** - Go back to Startup Screen from Home or DailyView (placed second-to-last in help text)
- ✅ **Zero compilation warnings** - Clean build with proper module organization

### Previous Session (List Item Unfocus Functionality)

- ✅ **Unfocus list items** - Press Esc to remove item highlight while staying in Food/Sokay section
- ✅ **Two-stage Esc behavior** - First Esc unfocuses item, second Esc returns to home screen
- ✅ **Smart navigation from unfocused state** - j/↓ focuses first item, k/↑ focuses last item
- ✅ **Initial unfocused state** - Lists start without highlighted items for quick access to adding entries
- ✅ **Safe edit/delete** - e and D keys only work when an item is highlighted
- ✅ **Conditional highlighting** - Item highlights only appear when both section AND item are focused
- ✅ **Consistent UX** - Mirrors home screen unfocus behavior in daily view lists
- ✅ **State tracking** - Added `food_list_focused` and `sokay_list_focused` flags to AppState
- ✅ **Updated rendering** - List highlight styles now check both section focus and item focus
- ✅ **Zero compilation warnings** - Clean build with proper borrow checker handling

### Previous Session (Individual Item Deletion with Confirmation)

- ✅ **Delete individual items** - Press 'D' (uppercase) to delete highlighted food or sokay items
- ✅ **Confirmation dialogs** - Small centered modal (60% x 20%) overlays daily view before deletion
- ✅ **y/n prompts** - Simple confirmation: 'y' to delete, 'n' or Esc to cancel
- ✅ **Instant feedback** - Optimistic updates with background persistence
- ✅ **Context-aware** - Same 'D' key works on Home screen (delete day) and DailyView (delete item)
- ✅ **Safe deletion** - Requires explicit confirmation to prevent accidental data loss
- ✅ **Centered modals** - Confirmation overlays keep daily view visible in background
- ✅ **Item preview** - Shows the exact item being deleted in the confirmation dialog

### Previous Session (Optimistic UI Updates - Instant Feedback)

- ✅ **Instant visual feedback** - All input operations (food, sokay, measurements, notes) now respond immediately without waiting for disk I/O
- ✅ **Optimistic update pattern** - State updates happen instantly, persistence happens in background
- ✅ **Sub-second responsiveness** - Press Enter and see changes immediately (previously had 1-second delay)
- ✅ **Background persistence** - Database and file writes happen asynchronously using tokio::spawn
- ✅ **Refactored handlers** - All save/update/delete methods separated state updates from persistence
- ✅ **Fire-and-forget I/O** - UI never blocks on database or file operations
- ✅ **Reliable data integrity** - Background sync task still ensures all changes are persisted to Turso Cloud
- ✅ **Snappy user experience** - App now feels instant and responsive, no more sluggish delays
- ✅ **Zero data loss risk** - In-memory state serves as write-ahead cache before persistence
- ✅ **Improved architecture** - Clean separation between UI responsiveness and data durability

### Previous Session (List Padding & Scrolling Enhancement)

- ✅ **Added vertical padding to all lists** - Home screen, Food Items, and Sokay lists now have uniform padding (top, bottom, left, right)
- ✅ **Improved visual spacing** - List items no longer touch the top and bottom borders of their sections
- ✅ **Better readability** - Consistent padding across all list widgets for a cleaner appearance
- ✅ **Automatic scrolling support** - Lists use `ListState` which automatically handles scrolling when navigating with j/k keys
- ✅ **Scalable design** - Application can handle 50-100+ training log entries with seamless scrolling

### Previous Session (UI Refinements - Modal Sizes & Title Styling)

- ✅ **Reduced measurement modal sizes** - Weight, Waist, Miles, and Elevation input dialogs now 12% width x 8% height (previously 30% x 15%)
- ✅ **Reduced Food/Sokay modal heights** - Add/Edit Food and Add/Edit Sokay dialogs now 50% width x 13% height (previously 50% x 25%)
- ✅ **Removed redundant dates** - Measurement modal titles no longer show date (already visible at top of screen)
- ✅ **Added top padding to all inputs** - All input modals now have 1 line of top padding for better visual spacing
- ✅ **Updated title colors** - Title text changed from Cyan to Green with orange borders (RGB: 255, 165, 0)
- ✅ **Compact and clean** - smaller input dialogs reduce visual clutter while maintaining usability
- ✅ **Still centered** - all modals remain perfectly centered in the terminal
- ✅ **Improved focus** - smaller dialogs draw less attention away from the daily view in the background

### Previous Session (Environment Variable Loading Fix)

- ✅ **Fixed cloud sync for installed binary** - binary now loads `.env` from data directory (`~/.mountains/.env`)
- ✅ **Directory-independent operation** - app now syncs to Turso Cloud from any directory
- ✅ **Dual .env loading** - checks data directory first, falls back to current directory for development
- ✅ **Zero configuration changes** - existing setup works unchanged
- ✅ **Improved portability** - installed binary behaves consistently regardless of working directory

### Previous Session (Multi-line Input Cursor Sync Fix with Word Wrapping)

- ✅ **Fixed cursor desync in multi-line inputs** - cursor now stays perfectly synced with typed text even after line wrapping
- ✅ **Word-based wrapping** - implemented smart word-wrapping that keeps words together when possible
- ✅ **Graceful long-word handling** - words longer than line width break at character boundaries
- ✅ **Controlled wrapping algorithm** - implemented `wrap_at_width()` function using `split_inclusive` for word-aware logic
- ✅ **Identical cursor calculation** - `calculate_cursor_in_wrapped_text()` uses exact same `split_inclusive` approach as display
- ✅ **Fixed space bar visibility** - cursor now moves forward immediately when pressing space
- ✅ **Fixed word-wrap sync** - rewrote cursor calculation to process words identically to display wrapping
- ✅ **Eliminated lag and drift** - cursor position matches displayed text character-for-character, even at wrap boundaries
- ✅ **Applied to both editors** - Strength & Mobility and Notes multi-line inputs both use synchronized word wrapping
- ✅ **Removed deprecated code** - cleaned up old wrapping implementation
- ✅ **Zero compilation warnings** - clean build with proper implementation

### Previous Session (Home Screen Unfocus & Today Quick Access)

- ✅ **Unfocused initial state** - Home screen starts with no item highlighted for quick access to today
- ✅ **Esc to unfocus** - Press Esc on home screen to remove list highlight
- ✅ **Enter for today** - When unfocused, Enter goes directly to today's log (creating if needed)
- ✅ **Smart focusing** - j/↓ focuses first item, k/↑ focuses last item when list unfocused
- ✅ **Cleaner workflow** - Quick access to today without navigating through the list
- ✅ **Updated help text** - Home screen help now shows "Enter: select/today | Esc: unfocus"
- ✅ **Zero behavioral changes to focused mode** - List navigation works exactly as before when focused

### Previous Session (Section-Based Navigation System)

- ✅ **Comprehensive section navigation** - Navigate through all sections with Shift+J/K (Measurements → Running → Food → Sokay → Strength & Mobility → Notes)
- ✅ **Field-level focus** - Tab key toggles between fields within Measurements (Weight/Waist) and Running (Miles/Elevation) sections
- ✅ **Visual focus indicators** - Focused sections show bright colored borders, focused fields show ► symbol
- ✅ **Enter key context awareness** - Enter opens appropriate input based on focused section/field
- ✅ **Backward compatible shortcuts** - All existing keyboard shortcuts (f, c, w, s, m, l, t, n) still work
- ✅ **New data model** - `FocusedSection` enum with nested `MeasurementField` and `RunningField` enums for type-safe navigation
- ✅ **SectionNavigator** - Pure function-based navigation logic for moving between sections and toggling fields
- ✅ **Updated all render functions** - All six sections now accept `focused_section` parameter and render focus state
- ✅ **Smart list navigation** - j/k keys only navigate within Food/Sokay lists when those sections have focus
- ✅ **Updated help text** - Concise help bar showing new navigation model
- ✅ **Zero compilation warnings** - Clean build with proper type safety

### Previous Session (UI Refinements - List Formatting, Delete Confirmation & Focus Highlighting)

- ✅ **Removed numbering from Food Items list** - Food entries now display with bullet points (`-`) instead of numbers
- ✅ **Removed numbering from Sokay list** - Sokay entries now display with bullet points (`-`) instead of numbers
- ✅ **Consistent formatting** - Terminal UI now matches markdown export format (both use bullets)
- ✅ **Simplified list display** - Cleaner visual presentation without unnecessary numbering
- ✅ **Improved delete confirmation styling** - Warning text now white instead of red, red border provides visual distinction
- ✅ **Focus-based highlighting** - List item highlights now only appear on the currently focused list (Food or Sokay)
- ✅ **Conditional highlight style** - Unfocused lists use `Style::default()` for invisible highlight, focused lists use reversed style
- ✅ **Cleaner focus indication** - Highlight disappears when switching focus with Shift+J/K, reappears when switching back

### Previous Session (Modal Dialog Refinements - Padding, Cursor & Wrapping Fixes)

- ✅ **Fixed critical cursor lag bug** - cursor now properly tracks character position in real-time
- ✅ **Implemented character-based wrapping** - replaced ratatui's word-wrapping with manual character-boundary wrapping
- ✅ **Added `wrap_text_at_chars()` helper** - wraps text at exact character width for predictable cursor positioning
- ✅ **Eliminated word-wrap desync** - text now wraps exactly where cursor calculation expects it to
- ✅ **Arrow key navigation fully functional** - left/right/up/down arrow keys work in multiline inputs (already implemented)
- ✅ **Byte index vs character count fix** - cursor_position is a UTF-8 byte index, now correctly converted to character count
- ✅ **Proper UTF-8 handling** - multi-byte characters (emojis, special punctuation) no longer cause cursor desync
- ✅ **Boundary condition handling** - cursor correctly wraps when positioned right after last character that fills the line width
- ✅ **Fixed multi-line cursor positioning** - cursor now correctly follows text in Strength & Mobility and Notes inputs
- ✅ **Fixed line wrapping cursor bug** - cursor now stays in sync when text wraps to next line (off-by-one error fixed)
- ✅ **Updated `calculate_multiline_cursor_position()`** - removed incorrect border offset since inner_area already excludes borders/padding
- ✅ **Improved wrapping logic** - now checks if character would exceed width BEFORE incrementing, not after (matches ratatui's Paragraph behavior)
- ✅ **Added horizontal padding** - all single-line inputs now have `Padding::horizontal(1)` for comfortable spacing
- ✅ **Added uniform padding** - multi-line inputs have `Padding::uniform(1)` for vertical and horizontal spacing
- ✅ **Reduced numeric input sizes** - Weight/Waist/Miles/Elevation modals now 30% width, 15% height (was 40% x 20%)
- ✅ **Compact and clean** - smaller dialogs for numeric inputs, proper spacing between text and borders
- ✅ **Padding properly accounted** - `block.inner()` removes both borders AND padding before cursor calculation
- ✅ **Zero compiler warnings** - application builds cleanly

### Previous Session (Modal Dialog Input Screens)

- ✅ **Centered modal dialogs** - all input screens now appear as small centered popups instead of full-screen
- ✅ **Daily view remains visible** - background view stays visible during input for better context
- ✅ **Clear widget usage** - prevents visual artifacts by clearing popup area before rendering
- ✅ **Optimized sizes** - single-line inputs (50% width, 25% height), numeric inputs (30% width, 15% height), multi-line inputs (60% width, 40% height)
- ✅ **Colored borders** - Yellow for food/measurements, Magenta for sokay, Cyan for strength/mobility, Green for notes, LightRed for running metrics
- ✅ **`centered_rect()` helper** - reusable function in ui/components.rs using Layout with Flex::Center
- ✅ **Updated all input screens** - Add/Edit Food, Add/Edit Sokay, Edit Weight/Waist/Miles/Elevation, Edit Strength & Mobility, Edit Notes
- ✅ **Removed dead code** - cleaned up unused `render_input_field`, `render_multiline_input_field`, and `calculate_cursor_position` helper functions
- ✅ **Zero compiler warnings** - application builds cleanly
- ✅ **Better UX** - users can see the daily log while entering data, reducing need to remember context

### Previous Session (Dual-List DailyView with Focus Switching)

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
- ✅ Automatic background sync every 4 minutes
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
- **State management** - AppScreen enum for view routing (19 different screens including Startup and Syncing)
- **Dual persistence** - libsql database (primary) + markdown files (backup)
- **Offline-first design** - Local database initializes instantly, cloud connection deferred to background
- **Cloud sync** - Syncs on startup (background) and shutdown (with visual feedback), graceful offline handling
- **Connection state tracking** - Real-time monitoring of Turso Cloud connection status
- **Async architecture** - Fully async event loop and database operations using tokio
- **Thread-safe database** - Arc<RwLock<DbManager>> for shared access across async tasks
- **Input handling** - Specialized handlers for text, numeric, integer, and multi-line input
- **Elevation statistics** - Dedicated module for calculating monthly/yearly stats and streaks
- **Modular design** - Separated concerns (models, events, ui, database, file management, stats)
- **Responsive UI** - Terminal size adaptation with ratatui layout system, live sync status display
- **Data integrity** - Database transactions for atomic operations
- **Error handling** - anyhow for ergonomic error propagation

### Key Data Structures

- **DailyLog** - Main data model with food_entries, measurements, sokay_entries, strength_mobility, notes
- **AppState** - Application state with daily_logs cache, current screen/selection, and focused_section
- **FocusedSection** - Enum tracking which section has focus (Measurements, Running, FoodItems, Sokay, StrengthMobility, Notes)
- **MeasurementField** - Enum for tracking focus within Measurements section (Weight, Waist)
- **RunningField** - Enum for tracking focus within Running section (Miles, Elevation)
- **SectionNavigator** - Pure function-based navigation logic for section and field traversal
- **InputHandler** - Cursor position tracking and input validation
- **DbManager** - Async database operations with deferred cloud connection and state tracking
- **ConnectionState** - Enum tracking sync status (Disconnected, Connected, Error)
- **FileManager** - Markdown serialization/deserialization for backups

# Rust coding guidelines

- Prioritize code correctness and clarity. Speed and efficiency are secondary priorities unless otherwise specified.
- Do not write comments that summarize the code. Comments should only be written in order to explain "why" the code is written in some way in the case there is a reason that is tricky / non-obvious. But do write comments that document APIs.

# important-instruction-reminders

Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (\*.md) or README files. Only create documentation files if explicitly requested by the User.

- always update @CLAUDE.md with changes
- clean up dead code as the app evolves and changes are made
