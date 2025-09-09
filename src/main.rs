mod file_manager;
mod models;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;

use crate::file_manager::FileManager;
use crate::models::{AppScreen, AppState, DailyLog, FoodEntry};

struct App {
    state: AppState,
    file_manager: FileManager,
    list_state: ListState,
    food_list_state: ListState,
    should_quit: bool,
    input_buffer: String,
    notes_buffer: String,
    cursor_position: usize,
}

impl App {
    fn new() -> Result<Self> {
        let file_manager = FileManager::new()?;
        let mut state = AppState::new();

        // Load existing logs
        state.daily_logs = file_manager.load_all_daily_logs()?;

        Ok(Self {
            state,
            file_manager,
            list_state: ListState::default(),
            food_list_state: ListState::default(),
            should_quit: false,
            input_buffer: String::new(),
            notes_buffer: String::new(),
            cursor_position: 0,
        })
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.state.current_screen {
                    AppScreen::AddFood => match key.code {
                        KeyCode::Enter => {
                            if !self.input_buffer.is_empty() {
                                let food_entry = FoodEntry::new(
                                    self.input_buffer.clone(),
                                    if self.notes_buffer.is_empty() {
                                        None
                                    } else {
                                        Some(self.notes_buffer.clone())
                                    },
                                );
                                let log =
                                    self.state.get_or_create_daily_log(self.state.selected_date);
                                log.add_food_entry(food_entry);
                                let _ = self.file_manager.save_daily_log(log);
                                self.clear_input();
                                self.state.current_screen = AppScreen::DailyView;
                            }
                        }
                        KeyCode::Esc => {
                            self.clear_input();
                            self.state.current_screen = AppScreen::DailyView;
                        }
                        KeyCode::Char(c) => {
                            self.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            self.delete_char();
                        }
                        KeyCode::Delete => {
                            self.delete_char_forward();
                        }
                        KeyCode::Left => {
                            self.move_cursor_left();
                        }
                        KeyCode::Right => {
                            self.move_cursor_right();
                        }
                        KeyCode::Home => {
                            self.cursor_position = 0;
                        }
                        KeyCode::End => {
                            self.cursor_position = self.input_buffer.len();
                        }
                        _ => {}
                    },
                    AppScreen::EditFood(food_index) => match key.code {
                        KeyCode::Enter => {
                            if !self.input_buffer.is_empty() {
                                if let Some(log) = self
                                    .state
                                    .daily_logs
                                    .iter_mut()
                                    .find(|log| log.date == self.state.selected_date)
                                {
                                    if food_index < log.food_entries.len() {
                                        log.food_entries[food_index].name =
                                            self.input_buffer.clone();
                                        let _ = self.file_manager.save_daily_log(log);
                                    }
                                }
                                self.clear_input();
                                self.state.current_screen = AppScreen::DailyView;
                            }
                        }
                        KeyCode::Esc => {
                            self.clear_input();
                            self.state.current_screen = AppScreen::DailyView;
                        }
                        KeyCode::Char(c) => {
                            self.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            self.delete_char();
                        }
                        KeyCode::Delete => {
                            self.delete_char_forward();
                        }
                        KeyCode::Left => {
                            self.move_cursor_left();
                        }
                        KeyCode::Right => {
                            self.move_cursor_right();
                        }
                        KeyCode::Home => {
                            self.cursor_position = 0;
                        }
                        KeyCode::End => {
                            self.cursor_position = self.input_buffer.len();
                        }
                        _ => {}
                    },
                    _ => match key.code {
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if matches!(self.state.current_screen, AppScreen::DailyView) {
                                self.move_food_selection_down();
                            } else {
                                self.move_selection_down();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if matches!(self.state.current_screen, AppScreen::DailyView) {
                                self.move_food_selection_up();
                            } else {
                                self.move_selection_up();
                            }
                        }
                        KeyCode::Enter => {
                            self.handle_enter();
                        }
                        KeyCode::Esc => {
                            self.handle_escape();
                        }
                        KeyCode::Char('a') => {
                            if matches!(self.state.current_screen, AppScreen::DailyView) {
                                self.state.current_screen = AppScreen::AddFood;
                            }
                        }
                        KeyCode::Char('e') => {
                            if matches!(self.state.current_screen, AppScreen::DailyView) {
                                self.handle_edit_food();
                            }
                        }
                        KeyCode::Char('d') => {
                            if matches!(self.state.current_screen, AppScreen::DailyView) {
                                self.handle_delete_food();
                            }
                        }
                        _ => {}
                    },
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        match self.state.current_screen {
            AppScreen::Home => self.draw_home_screen(f),
            AppScreen::DailyView => self.draw_daily_view_screen(f),
            AppScreen::AddFood => self.draw_add_food_screen(f),
            AppScreen::EditFood(_) => self.draw_edit_food_screen(f),
            _ => self.draw_home_screen(f), // Default to home for now
        }
    }

    fn draw_home_screen(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.area());

        // Title
        let title = Paragraph::new("Mountains Food Tracker")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Days list
        let items: Vec<ListItem> = if self.state.daily_logs.is_empty() {
            vec![ListItem::new(
                "No food logs yet. Press Enter to create one for today.",
            )]
        } else {
            self.state
                .daily_logs
                .iter()
                .map(|log| {
                    let date_str = log.date.format("%B %d, %Y").to_string();
                    let food_count = log.food_entries.len();
                    let summary = format!("{} ({} food items)", date_str, food_count);
                    ListItem::new(summary)
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Food Log Days")
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(list, chunks[1], &mut self.list_state);

        // Help
        let help = Paragraph::new("q: quit | ↑/k: up | ↓/j: down | Enter: select/create")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn draw_daily_view_screen(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.area());

        // Title with date
        let title = format!(
            "Food Log - {}",
            self.state.selected_date.format("%B %d, %Y")
        );
        let title_widget = Paragraph::new(title)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, chunks[0]);

        // Food items list
        let log = self.state.get_daily_log(self.state.selected_date);
        let items: Vec<ListItem> = if let Some(log) = log {
            if log.food_entries.is_empty() {
                vec![ListItem::new("No food entries yet. Press 'a' to add one.")]
            } else {
                log.food_entries
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| {
                        let display = if let Some(notes) = &entry.notes {
                            format!("{}. {} - {}", i + 1, entry.name, notes)
                        } else {
                            format!("{}. {}", i + 1, entry.name)
                        };
                        ListItem::new(display)
                    })
                    .collect()
            }
        } else {
            vec![ListItem::new("No food entries yet. Press 'a' to add one.")]
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Food Items")
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_stateful_widget(list, chunks[1], &mut self.food_list_state);

        // Help
        let help = Paragraph::new(
            "q: quit | a: add | e: edit | d: delete | ↑/↓: navigate | Esc: back to home",
        )
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn draw_add_food_screen(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(f.area());

        // Title
        let title = format!(
            "Add Food - {}",
            self.state.selected_date.format("%B %d, %Y")
        );
        let title_widget = Paragraph::new(title)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, chunks[0]);

        // Input field with cursor
        let input_with_cursor = self.format_input_with_cursor();
        let input = Paragraph::new(input_with_cursor)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Food Name"));
        f.render_widget(input, chunks[1]);

        // Set cursor position in terminal
        let input_area = chunks[1];
        let cursor_x = input_area.x + 1 + self.cursor_position as u16; // +1 for border
        let cursor_y = input_area.y + 1; // +1 for border
        f.set_cursor_position((cursor_x, cursor_y));

        // Help
        let help = Paragraph::new("Type food name and press Enter to save | Esc: cancel")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn format_daily_log_for_display(&self, log: &DailyLog) -> String {
        let mut content = String::new();

        // Measurements
        if log.weight.is_some() || log.waist.is_some() {
            content.push_str("MEASUREMENTS:\n");
            if let Some(weight) = log.weight {
                content.push_str(&format!("  Weight: {} lbs\n", weight));
            }
            if let Some(waist) = log.waist {
                content.push_str(&format!("  Waist: {} inches\n", waist));
            }
            content.push('\n');
        }

        // Food entries
        if !log.food_entries.is_empty() {
            content.push_str("FOOD:\n");
            for entry in &log.food_entries {
                content.push_str(&format!("  • {}", entry.name));
                if let Some(notes) = &entry.notes {
                    content.push_str(&format!(" - {}", notes));
                }
                content.push('\n');
            }
            content.push('\n');
        }

        // Notes
        if let Some(notes) = &log.notes {
            content.push_str("NOTES:\n");
            content.push_str(notes);
        }

        if content.is_empty() {
            "No entries for this day yet.".to_string()
        } else {
            content
        }
    }

    fn move_selection_down(&mut self) {
        if self.state.daily_logs.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.state.daily_logs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.state.selected_index = i;
    }

    fn move_selection_up(&mut self) {
        if self.state.daily_logs.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.state.daily_logs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.state.selected_index = i;
    }

    fn handle_enter(&mut self) {
        match self.state.current_screen {
            AppScreen::Home => {
                if self.state.daily_logs.is_empty() {
                    // Create today's log
                    self.state.selected_date = chrono::Local::now().date_naive();
                } else if let Some(selected) = self.list_state.selected() {
                    if selected < self.state.daily_logs.len() {
                        self.state.selected_date = self.state.daily_logs[selected].date;
                    }
                }
                self.state.current_screen = AppScreen::DailyView;
            }
            _ => {}
        }
    }

    fn handle_escape(&mut self) {
        match self.state.current_screen {
            AppScreen::DailyView => {
                self.state.current_screen = AppScreen::Home;
            }
            _ => {}
        }
    }

    fn draw_edit_food_screen(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(f.area());

        // Title
        let title = format!(
            "Edit Food - {}",
            self.state.selected_date.format("%B %d, %Y")
        );
        let title_widget = Paragraph::new(title)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, chunks[0]);

        // Input field with cursor
        let input_with_cursor = self.format_input_with_cursor();
        let input = Paragraph::new(input_with_cursor)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Food Name"));
        f.render_widget(input, chunks[1]);

        // Set cursor position in terminal
        let input_area = chunks[1];
        let cursor_x = input_area.x + 1 + self.cursor_position as u16; // +1 for border
        let cursor_y = input_area.y + 1; // +1 for border
        f.set_cursor_position((cursor_x, cursor_y));

        // Help
        let help = Paragraph::new("Edit food name and press Enter to save | Esc: cancel")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn move_food_selection_down(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if log.food_entries.is_empty() {
                return;
            }

            let i = match self.food_list_state.selected() {
                Some(i) => {
                    if i >= log.food_entries.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.food_list_state.select(Some(i));
        }
    }

    fn move_food_selection_up(&mut self) {
        if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
            if log.food_entries.is_empty() {
                return;
            }

            let i = match self.food_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        log.food_entries.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.food_list_state.select(Some(i));
        }
    }

    fn handle_edit_food(&mut self) {
        if let Some(selected) = self.food_list_state.selected() {
            if let Some(log) = self.state.get_daily_log(self.state.selected_date) {
                if selected < log.food_entries.len() {
                    // Pre-fill the input buffer with the current food name
                    self.input_buffer = log.food_entries[selected].name.clone();
                    self.cursor_position = self.input_buffer.len();
                    self.state.current_screen = AppScreen::EditFood(selected);
                }
            }
        }
    }

    fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.notes_buffer.clear();
        self.cursor_position = 0;
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor_position >= self.input_buffer.len() {
            self.input_buffer.push(c);
        } else {
            self.input_buffer.insert(self.cursor_position, c);
        }
        self.cursor_position += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            if self.cursor_position < self.input_buffer.len() {
                self.input_buffer.remove(self.cursor_position);
            }
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    fn format_input_with_cursor(&self) -> String {
        if self.input_buffer.is_empty() {
            " ".to_string() // Show space for cursor when empty
        } else {
            self.input_buffer.clone()
        }
    }

    fn handle_delete_food(&mut self) {
        if let Some(selected) = self.food_list_state.selected() {
            if let Some(log) = self
                .state
                .daily_logs
                .iter_mut()
                .find(|log| log.date == self.state.selected_date)
            {
                if selected < log.food_entries.len() {
                    log.remove_food_entry(selected);
                    let _ = self.file_manager.save_daily_log(log);

                    // Adjust selection if needed
                    if log.food_entries.is_empty() {
                        self.food_list_state.select(None);
                    } else if selected >= log.food_entries.len() {
                        self.food_list_state
                            .select(Some(log.food_entries.len() - 1));
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new()?;
    let res = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
