use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::models::AppState;
use crate::ui::components::{create_highlight_style, create_standard_layout, render_help, render_title};

/// Renders the home screen showing all available daily logs
pub fn render_home_screen(
    f: &mut Frame,
    state: &AppState,
    list_state: &mut ListState,
    sync_status: &str,
) {
    let chunks = create_standard_layout(f.area());

    // Render title with sync status
    let title = format!("Mountains - A Trail Running Training Log {}", sync_status);
    render_title(f, chunks[0], &title);

    // Create the list of daily logs
    let items: Vec<ListItem> = if state.daily_logs.is_empty() {
        vec![ListItem::new(
            "No training logs yet. Press Enter to create one for today.",
        )]
    } else {
        state
            .daily_logs
            .iter()
            .map(|log| {
                let date_str = log.date.format("%B %d, %Y").to_string();
                ListItem::new(date_str)
            })
            .collect()
    };

    // Create the List widget with styling
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Daily Training Logs")
                .padding(ratatui::widgets::Padding::uniform(1)),
        )
        .highlight_style(create_highlight_style());

    f.render_stateful_widget(list, chunks[1], list_state);

    // Render help text
    render_help(
        f,
        chunks[2],
        " ↑/k: Up | ↓/j: Down | Enter: Select/Today | Esc: Unfocus | D: Delete Day | S: Startup Screen | q: Quit ",
        true,
        false,
    );
}
