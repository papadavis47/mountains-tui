pub mod startup;
pub mod home;
pub mod daily_view;
pub mod inputs;
pub mod confirmations;
pub mod help;

// Re-export all public functions for backward compatibility
pub use startup::render_startup_screen;
pub use home::render_home_screen;
pub use daily_view::render_daily_view_screen;
pub use inputs::{
    render_add_food_screen,
    render_edit_food_screen,
    render_edit_weight_screen,
    render_edit_waist_screen,
    render_edit_miles_screen,
    render_edit_elevation_screen,
    render_edit_strength_mobility_screen,
    render_edit_notes_screen,
    render_add_sokay_screen,
    render_edit_sokay_screen,
    wrap_at_width,
    calculate_cursor_in_wrapped_text,
};
pub use confirmations::{
    render_confirm_delete_day_screen,
    render_confirm_delete_food_screen,
    render_confirm_delete_sokay_screen,
};
pub use help::{
    render_shortcuts_help_screen,
    render_syncing_screen,
};
