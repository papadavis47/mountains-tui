# Keyboard Shortcuts Reference

This document lists all keyboard shortcuts available in the Mountains TUI application, organized by context/screen.

---

## Global Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit application (triggers sync before exit) |

---

## Startup Screen

| Key | Action |
|-----|--------|
| `n` | Create new entry for today and go to Daily View |
| `l` | Go to Log list (Home screen) |

---

## Home Screen (Log List)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down in the log list |
| `k` / `↑` | Move selection up in the log list |
| `Enter` | Open selected day's Daily View (or today if none selected) |
| `Esc` | Deselect current selection |
| `d` | Delete selected day (opens confirmation dialog) |
| `S` | Return to Startup screen |

---

## Daily View Screen

### Section Navigation

| Key | Action |
|-----|--------|
| `Shift+J` | Move focus to next section |
| `Shift+K` | Move focus to previous section |
| `Tab` | Toggle internal focus within current section |

### List Navigation (when Food Items or Sokay section is focused)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down in list / scroll content down |
| `k` / `↑` | Move selection up in list / scroll content up |
| `Esc` | Unfocus list (if list focused) / Return to Home (if not) |

### Scrolling (when Strength/Mobility or Notes section is focused)

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll content down |
| `k` / `↑` | Scroll content up |

### Quick Edit Shortcuts

| Key | Action |
|-----|--------|
| `f` | Add new food entry |
| `c` | Add new Sokay entry |
| `w` | Edit weight |
| `s` | Edit waist measurement |
| `t` | Edit strength/mobility notes |
| `n` | Edit notes |
| `m` | Edit miles |
| `l` | Edit elevation |
| `e` | Edit selected item (when Food or Sokay list is focused) |
| `d` | Delete selected item (when Food or Sokay list is focused) |

### Section Enter Actions

| Section Focused | Enter Action |
|-----------------|--------------|
| Measurements (Weight) | Edit weight |
| Measurements (Waist) | Edit waist |
| Running (Miles) | Edit miles |
| Running (Elevation) | Edit elevation |
| Food Items | Add new food entry |
| Sokay | Add new Sokay entry |
| Strength/Mobility | Edit strength/mobility notes |
| Notes | Edit notes |

### Other

| Key | Action |
|-----|--------|
| `Space` | Toggle shortcuts help overlay |
| `S` | Return to Startup screen |

---

## Shortcuts Help Screen

| Key | Action |
|-----|--------|
| `Space` | Close help and return to Daily View |

---

## Delete Confirmation Dialogs

| Key | Action |
|-----|--------|
| `y` | Confirm deletion |
| `n` | Cancel deletion |
| `Esc` | Cancel deletion |

---

## Text Input Modes (Add/Edit Food, Add/Edit Sokay)

| Key | Action |
|-----|--------|
| `Enter` | Save entry and return to Daily View |
| `Esc` | Cancel and return to Daily View |
| Any character | Insert character at cursor |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character after cursor |
| `←` | Move cursor left |
| `→` | Move cursor right |
| `Home` | Move cursor to beginning |
| `End` | Move cursor to end |

---

## Numeric Input Modes (Weight, Waist, Miles)

| Key | Action |
|-----|--------|
| `Enter` | Save value and return to Daily View |
| `Esc` | Cancel and return to Daily View |
| `0-9` | Insert digit |
| `.` | Insert decimal point |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character after cursor |
| `←` | Move cursor left |
| `→` | Move cursor right |
| `Home` | Move cursor to beginning |
| `End` | Move cursor to end |

---

## Integer Input Mode (Elevation)

| Key | Action |
|-----|--------|
| `Enter` | Save value and return to Daily View |
| `Esc` | Cancel and return to Daily View |
| `0-9` | Insert digit |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character after cursor |
| `←` | Move cursor left |
| `→` | Move cursor right |
| `Home` | Move cursor to beginning |
| `End` | Move cursor to end |

---

## Multiline Text Input Modes (Strength/Mobility, Notes)

| Key | Action |
|-----|--------|
| `Enter` | Save entry and return to Daily View |
| `Alt+Enter` | Insert newline (stay in edit mode) |
| `Esc` | Cancel and return to Daily View |
| Any character | Insert character at cursor |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character after cursor |
| `←` | Move cursor left |
| `→` | Move cursor right |
| `↑` | Move cursor up one line |
| `↓` | Move cursor down one line |
| `Home` | Move cursor to beginning |
| `End` | Move cursor to end |

---

## Section Focus Order (Daily View)

The sections cycle in this order with `Shift+J` / `Shift+K`:

1. **Measurements** (Weight / Waist - toggle with `Tab`)
2. **Running** (Miles / Elevation - toggle with `Tab`)
3. **Food Items**
4. **Sokay**
5. **Strength/Mobility**
6. **Notes**
