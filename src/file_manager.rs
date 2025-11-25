use crate::models::DailyLog;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FileManager {
    mountains_dir: PathBuf,
}

impl FileManager {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let mountains_dir = home_dir.join(".mountains");

        if !mountains_dir.exists() {
            fs::create_dir_all(&mountains_dir).context("Failed to create .mountains directory")?;
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

    fn daily_log_to_markdown(&self, log: &DailyLog) -> String {
        let mut content = String::new();

        content.push_str(&format!(
            "# Mountains Training Log - {}\n\n",
            log.date.format("%B %d, %Y")
        ));

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

        if !log.food_entries.is_empty() {
            content.push_str("## Food\n");
            for entry in &log.food_entries {
                content.push_str(&format!("- {}\n", entry.name));
            }
            content.push('\n');
        }

        if log.miles_covered.is_some() || log.elevation_gain.is_some() {
            content.push_str("## Running\n");
            if let Some(miles) = log.miles_covered {
                content.push_str(&format!("- **Miles:** {} mi\n", miles));
            }
            if let Some(elevation) = log.elevation_gain {
                content.push_str(&format!("- **Elevation:** {} ft\n", elevation));
            }
            content.push('\n');
        }

        if !log.sokay_entries.is_empty() {
            content.push_str("## Sokay\n");
            for entry in &log.sokay_entries {
                content.push_str(&format!("- {}\n", entry));
            }
            content.push('\n');
        }

        if let Some(strength_mobility) = &log.strength_mobility {
            content.push_str("## Strength & Mobility\n");
            content.push_str(strength_mobility);
            content.push('\n');
        }

        if let Some(notes) = &log.notes {
            content.push_str("## Notes\n");
            content.push_str(notes);
            content.push('\n');
        }

        content
    }

    pub fn delete_daily_log(&self, date: NaiveDate) -> Result<()> {
        let file_path = self.get_file_path(date);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .context(format!("Failed to delete file: {:?}", file_path))?;
        }

        Ok(())
    }
}
