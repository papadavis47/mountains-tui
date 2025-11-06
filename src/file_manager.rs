use crate::models::DailyLog;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::fs;
use std::path::PathBuf;

/// This module handles all file I/O operations for the application.
/// It manages saving and loading daily logs to/from markdown files
/// in the user's home directory.
///
/// The file format used is human-readable markdown, which allows users
/// to view and edit their data outside of the application if needed.
/// Manages file operations for daily logs
///
/// The FileManager is responsible for:
/// - Creating and managing the data directory (~/.mountains/)
/// - Converting between DailyLog structs and markdown files
/// - Loading all existing daily logs on startup
/// - Saving individual daily logs when they change
///
/// PathBuf is Rust's cross-platform path type that works on Windows, macOS, and Linux.
pub struct FileManager {
    /// Path to the directory where all daily log files are stored
    mountains_dir: PathBuf,
}

impl FileManager {
    /// Creates a new FileManager and ensures the data directory exists
    ///
    /// This constructor:
    /// 1. Gets the user's home directory using the `dirs` crate
    /// 2. Creates a `.mountains` subdirectory for storing data files
    /// 3. Creates the directory if it doesn't already exist
    ///
    /// The `?` operator propagates any errors that occur during directory
    /// creation or home directory detection.
    pub fn new() -> Result<Self> {
        // Get the user's home directory (cross-platform)
        let home_dir = dirs::home_dir().context("Could not find home directory")?;

        // Create the path to our data directory
        let mountains_dir = home_dir.join(".mountains");

        // Create directory if it doesn't exist
        // create_dir_all() creates parent directories as needed and doesn't fail if the directory exists
        if !mountains_dir.exists() {
            fs::create_dir_all(&mountains_dir).context("Failed to create .mountains directory")?;
        }

        Ok(Self { mountains_dir })
    }

    /// Generates the file path for a daily log based on its date
    ///
    /// Files are named using the format: "mtslog-MM.DD.YYYY.md"
    /// For example: "mtslog-01.15.2025.md" for January 15, 2025
    ///
    /// This naming convention makes files easy to identify and sort chronologically.
    fn get_file_path(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("mtslog-{}.md", date.format("%m.%d.%Y"));
        self.mountains_dir.join(filename)
    }

    /// Saves a daily log to disk as a markdown file
    ///
    /// This method:
    /// 1. Converts the DailyLog struct to markdown format
    /// 2. Writes the markdown content to the appropriate file
    /// 3. Returns an error if the file operation fails
    ///
    /// The `&DailyLog` parameter is a read-only reference - we don't need to
    /// modify the log, just read its data for saving.
    pub fn save_daily_log(&self, log: &DailyLog) -> Result<()> {
        let file_path = self.get_file_path(log.date);
        let content = self.daily_log_to_markdown(log);

        // Write the markdown content to the file
        // The context() method adds helpful error information if the write fails
        fs::write(&file_path, content)
            .context(format!("Failed to write to file: {:?}", file_path))?;

        Ok(())
    }

    /// Converts a DailyLog struct to markdown format
    ///
    /// This method creates a human-readable markdown representation of the daily log.
    /// The format includes:
    /// - A main title with the date
    /// - A measurements section (if any measurements exist)
    /// - A food section (if any food entries exist)
    /// - A notes section (if notes exist)
    ///
    /// The markdown format allows users to view and edit their data in any text editor.
    fn daily_log_to_markdown(&self, log: &DailyLog) -> String {
        let mut content = String::new();

        // Add the main title
        content.push_str(&format!(
            "# Mountains Training Log - {}\n\n",
            log.date.format("%B %d, %Y")
        ));

        // Add measurements section if any measurements exist
        if log.weight.is_some()
            || log.waist.is_some()
            || log.miles_covered.is_some()
            || log.elevation_gain.is_some()
            || !log.sokay_entries.is_empty()
        {
            content.push_str("## Measurements\n");
            // Body measurements
            if let Some(weight) = log.weight {
                content.push_str(&format!("- **Weight:** {} lbs\n", weight));
            }
            if let Some(waist) = log.waist {
                content.push_str(&format!("- **Waist:** {} inches\n", waist));
            }
            // Activity measurements
            if let Some(miles) = log.miles_covered {
                content.push_str(&format!("- **Miles:** {} mi\n", miles));
            }
            if let Some(elevation) = log.elevation_gain {
                content.push_str(&format!("- **Elevation:** {} ft\n", elevation));
            }
            // Sokay cumulative count (Note: this is just for display in markdown,
            // actual calculation happens elsewhere)
            if !log.sokay_entries.is_empty() {
                content.push_str(&format!("- **Sokay:** {} items\n", log.sokay_entries.len()));
            }
            content.push('\n'); // Add blank line after section
        }

        // Add food entries section if any entries exist
        if !log.food_entries.is_empty() {
            content.push_str("## Food\n");
            for entry in &log.food_entries {
                content.push_str(&format!("- **{}**", entry.name));
                if let Some(notes) = &entry.notes {
                    content.push_str(&format!(" - {}", notes));
                }
                content.push('\n');
            }
            content.push('\n'); // Add blank line after section
        }

        // Add sokay entries section if any entries exist
        if !log.sokay_entries.is_empty() {
            content.push_str("## Sokay\n");
            for entry in &log.sokay_entries {
                content.push_str(&format!("- {}\n", entry));
            }
            content.push('\n'); // Add blank line after section
        }

        // Add strength & mobility section if it exists
        if let Some(strength_mobility) = &log.strength_mobility {
            content.push_str("## Strength & Mobility\n");
            content.push_str(strength_mobility);
            content.push('\n');
        }

        // Add daily notes section if notes exist
        if let Some(notes) = &log.notes {
            content.push_str("## Notes\n");
            content.push_str(notes);
            content.push('\n');
        }

        content
    }

    /// Deletes a daily log markdown file from disk
    ///
    /// This method:
    /// 1. Determines the file path for the given date
    /// 2. Deletes the file if it exists
    /// 3. Returns Ok(()) even if the file doesn't exist (idempotent operation)
    ///
    /// Returns an error only if the deletion fails for reasons other than
    /// the file not existing.
    pub fn delete_daily_log(&self, date: NaiveDate) -> Result<()> {
        let file_path = self.get_file_path(date);

        // Only try to delete if the file exists
        if file_path.exists() {
            fs::remove_file(&file_path)
                .context(format!("Failed to delete file: {:?}", file_path))?;
        }

        Ok(())
    }
}
