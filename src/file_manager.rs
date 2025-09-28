use crate::models::{DailyLog, FoodEntry};
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

    /// Loads a daily log from disk for the specified date
    ///
    /// This method returns Option<DailyLog>:
    /// - Some(log) if a file exists for the date and can be parsed
    /// - None if no file exists for the date
    ///
    /// If a file exists but can't be parsed, this returns an error.
    pub fn load_daily_log(&self, date: NaiveDate) -> Result<Option<DailyLog>> {
        let file_path = self.get_file_path(date);

        // Check if the file exists first
        if !file_path.exists() {
            return Ok(None);
        }

        // Read the file content
        let content = fs::read_to_string(&file_path)
            .context(format!("Failed to read file: {:?}", file_path))?;

        // Parse the markdown content into a DailyLog
        let log = self.markdown_to_daily_log(date, &content)?;
        Ok(Some(log))
    }

    /// Loads all daily logs from the data directory
    ///
    /// This method:
    /// 1. Scans the .mountains directory for .md files
    /// 2. Parses each file that matches our naming convention
    /// 3. Returns a vector of all successfully loaded daily logs
    /// 4. Sorts the logs with newest first for better UX
    ///
    /// Files that can't be parsed are skipped rather than causing the entire
    /// operation to fail. This makes the application more resilient.
    pub fn load_all_daily_logs(&self) -> Result<Vec<DailyLog>> {
        let mut logs = Vec::new();

        // Return empty vector if the directory doesn't exist
        if !self.mountains_dir.exists() {
            return Ok(logs);
        }

        // Iterate through all files in the directory
        for entry in fs::read_dir(&self.mountains_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Check if this looks like one of our log files
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("mtslog-") && filename.ends_with(".md") {
                    // Try to parse the date from the filename
                    if let Some(date) = self.parse_date_from_filename(filename) {
                        // Try to read and parse the file content
                        let content = fs::read_to_string(&path)?;
                        if let Ok(log) = self.markdown_to_daily_log(date, &content) {
                            logs.push(log);
                        }
                        // If parsing fails, we skip this file and continue
                    }
                }
            }
        }

        // Sort newest first for better user experience
        logs.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(logs)
    }

    /// Extracts the date from a filename
    ///
    /// Expected format: "mtslog-MM.DD.YYYY.md"
    /// Returns None if the filename doesn't match the expected format.
    ///
    /// This method uses Option<T> to handle the case where parsing fails gracefully.
    fn parse_date_from_filename(&self, filename: &str) -> Option<NaiveDate> {
        // Strip the prefix and suffix to get just the date part
        let date_part = filename.strip_prefix("mtslog-")?.strip_suffix(".md")?;

        // Try to parse the date using the expected format
        NaiveDate::parse_from_str(date_part, "%m.%d.%Y").ok()
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
            "# Food Log - {}\n\n",
            log.date.format("%B %d, %Y")
        ));

        // Add measurements section if any measurements exist
        if log.weight.is_some() || log.waist.is_some() {
            content.push_str("## Measurements\n");
            if let Some(weight) = log.weight {
                content.push_str(&format!("- **Weight:** {} lbs\n", weight));
            }
            if let Some(waist) = log.waist {
                content.push_str(&format!("- **Waist:** {} inches\n", waist));
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

        // Add daily notes section if notes exist
        if let Some(notes) = &log.notes {
            content.push_str("## Notes\n");
            content.push_str(notes);
            content.push('\n');
        }

        content
    }

    /// Parses markdown content into a DailyLog struct
    ///
    /// This method parses the markdown format created by daily_log_to_markdown().
    /// It uses a simple state machine approach to track which section it's currently
    /// parsing (measurements, food, or notes).
    ///
    /// The parsing is forgiving - if some parts can't be parsed, it continues
    /// processing the rest of the file.
    fn markdown_to_daily_log(&self, date: NaiveDate, content: &str) -> Result<DailyLog> {
        let mut log = DailyLog::new(date);
        let lines: Vec<&str> = content.lines().collect();

        // State tracking for which section we're currently parsing
        let mut in_measurements = false;
        let mut in_food = false;
        let mut in_notes = false;
        let mut notes_content = String::new();

        for line in lines {
            let trimmed = line.trim();

            // Check for section headers to update our parsing state
            if trimmed.starts_with("## Measurements") {
                in_measurements = true;
                in_food = false;
                in_notes = false;
            } else if trimmed.starts_with("## Food") {
                in_measurements = false;
                in_food = true;
                in_notes = false;
            } else if trimmed.starts_with("## Notes") {
                in_measurements = false;
                in_food = false;
                in_notes = true;
            } else if trimmed.starts_with("##") || trimmed.starts_with("#") {
                // Any other section header resets our state
                in_measurements = false;
                in_food = false;
                in_notes = false;
            } else if in_measurements && trimmed.starts_with("- **Weight:**") {
                // Parse weight measurement
                if let Some(weight_str) = trimmed
                    .strip_prefix("- **Weight:**")
                    .and_then(|s| s.strip_suffix(" lbs"))
                {
                    if let Ok(weight) = weight_str.trim().parse::<f32>() {
                        log.weight = Some(weight);
                    }
                }
            } else if in_measurements && trimmed.starts_with("- **Waist:**") {
                // Parse waist measurement
                if let Some(waist_str) = trimmed
                    .strip_prefix("- **Waist:**")
                    .and_then(|s| s.strip_suffix(" inches"))
                {
                    if let Ok(waist) = waist_str.trim().parse::<f32>() {
                        log.waist = Some(waist);
                    }
                }
            } else if in_food && trimmed.starts_with("- **") {
                // Parse food entry: "- **Food Name** - notes"
                if let Some(rest) = trimmed.strip_prefix("- **") {
                    let parts: Vec<&str> = rest.split("**").collect();
                    if parts.len() >= 2 {
                        let name = parts[0].trim().to_string();
                        // Check if there are notes after the food name
                        let notes = if parts.len() > 2 && parts[1].trim().starts_with(" - ") {
                            Some(
                                parts[1]
                                    .trim()
                                    .strip_prefix(" - ")
                                    .unwrap_or("")
                                    .to_string(),
                            )
                        } else {
                            None
                        };
                        log.add_food_entry(FoodEntry::new(name, notes));
                    }
                }
            } else if in_notes && !trimmed.is_empty() {
                // Accumulate notes content
                if !notes_content.is_empty() {
                    notes_content.push('\n');
                }
                notes_content.push_str(line);
            }
        }

        // Save accumulated notes if any were found
        if !notes_content.is_empty() {
            log.notes = Some(notes_content);
        }

        Ok(log)
    }
}
