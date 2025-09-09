use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use crate::models::{DailyLog, FoodEntry};

pub struct FileManager {
    mountains_dir: PathBuf,
}

impl FileManager {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let mountains_dir = home_dir.join(".mountains");
        
        // Create directory if it doesn't exist
        if !mountains_dir.exists() {
            fs::create_dir_all(&mountains_dir)
                .context("Failed to create .mountains directory")?;
        }

        Ok(Self { mountains_dir })
    }

    fn get_file_path(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("mtslog-{}.md", date.format("%m.%d.%Y"));
        self.mountains_dir.join(filename)
    }

    pub fn save_daily_log(&self, log: &DailyLog) -> Result<()> {
        let file_path = self.get_file_path(log.date);
        let content = self.daily_log_to_markdown(log);
        
        fs::write(&file_path, content)
            .context(format!("Failed to write to file: {:?}", file_path))?;
        
        Ok(())
    }

    pub fn load_daily_log(&self, date: NaiveDate) -> Result<Option<DailyLog>> {
        let file_path = self.get_file_path(date);
        
        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path)
            .context(format!("Failed to read file: {:?}", file_path))?;
        
        let log = self.markdown_to_daily_log(date, &content)?;
        Ok(Some(log))
    }

    pub fn load_all_daily_logs(&self) -> Result<Vec<DailyLog>> {
        let mut logs = Vec::new();
        
        if !self.mountains_dir.exists() {
            return Ok(logs);
        }

        for entry in fs::read_dir(&self.mountains_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("mtslog-") && filename.ends_with(".md") {
                    if let Some(date) = self.parse_date_from_filename(filename) {
                        let content = fs::read_to_string(&path)?;
                        if let Ok(log) = self.markdown_to_daily_log(date, &content) {
                            logs.push(log);
                        }
                    }
                }
            }
        }
        
        logs.sort_by(|a, b| b.date.cmp(&a.date)); // Sort newest first
        Ok(logs)
    }

    fn parse_date_from_filename(&self, filename: &str) -> Option<NaiveDate> {
        // Extract date from "mtslog-MM.DD.YYYY.md"
        let date_part = filename.strip_prefix("mtslog-")?.strip_suffix(".md")?;
        NaiveDate::parse_from_str(date_part, "%m.%d.%Y").ok()
    }

    fn daily_log_to_markdown(&self, log: &DailyLog) -> String {
        let mut content = String::new();
        
        content.push_str(&format!("# Food Log - {}\n\n", log.date.format("%B %d, %Y")));
        
        // Weight and waist measurements
        if log.weight.is_some() || log.waist.is_some() {
            content.push_str("## Measurements\n");
            if let Some(weight) = log.weight {
                content.push_str(&format!("- **Weight:** {} lbs\n", weight));
            }
            if let Some(waist) = log.waist {
                content.push_str(&format!("- **Waist:** {} inches\n", waist));
            }
            content.push('\n');
        }

        // Food entries
        if !log.food_entries.is_empty() {
            content.push_str("## Food\n");
            for entry in &log.food_entries {
                content.push_str(&format!("- **{}**", entry.name));
                if let Some(notes) = &entry.notes {
                    content.push_str(&format!(" - {}", notes));
                }
                content.push('\n');
            }
            content.push('\n');
        }

        // Daily notes
        if let Some(notes) = &log.notes {
            content.push_str("## Notes\n");
            content.push_str(notes);
            content.push('\n');
        }

        content
    }

    fn markdown_to_daily_log(&self, date: NaiveDate, content: &str) -> Result<DailyLog> {
        let mut log = DailyLog::new(date);
        let lines: Vec<&str> = content.lines().collect();
        
        let mut in_measurements = false;
        let mut in_food = false;
        let mut in_notes = false;
        let mut notes_content = String::new();

        for line in lines {
            let trimmed = line.trim();
            
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
                in_measurements = false;
                in_food = false;
                in_notes = false;
            } else if in_measurements && trimmed.starts_with("- **Weight:**") {
                if let Some(weight_str) = trimmed.strip_prefix("- **Weight:**").and_then(|s| s.strip_suffix(" lbs")) {
                    if let Ok(weight) = weight_str.trim().parse::<f32>() {
                        log.weight = Some(weight);
                    }
                }
            } else if in_measurements && trimmed.starts_with("- **Waist:**") {
                if let Some(waist_str) = trimmed.strip_prefix("- **Waist:**").and_then(|s| s.strip_suffix(" inches")) {
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
                        let notes = if parts.len() > 2 && parts[1].trim().starts_with(" - ") {
                            Some(parts[1].trim().strip_prefix(" - ").unwrap_or("").to_string())
                        } else {
                            None
                        };
                        log.add_food_entry(FoodEntry::new(name, notes));
                    }
                }
            } else if in_notes && !trimmed.is_empty() {
                if !notes_content.is_empty() {
                    notes_content.push('\n');
                }
                notes_content.push_str(line);
            }
        }

        if !notes_content.is_empty() {
            log.notes = Some(notes_content);
        }

        Ok(log)
    }
}
