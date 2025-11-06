/// Database manager for Turso Cloud integration
///
/// This module handles all database operations using libsql with Turso Cloud.
/// It implements an embedded replica strategy where:
/// - Local database file is stored in ~/.mountains/mountains.db
/// - Changes are automatically synced to Turso Cloud
/// - The app works offline and syncs when connected
///
/// The database schema consists of two tables:
/// 1. daily_logs: Stores date, measurements, and notes for each day
/// 2. food_entries: Stores individual food items linked to daily logs

use anyhow::{Context, Result};
use chrono::NaiveDate;
use libsql::{Builder, Connection, Database};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

use crate::models::{DailyLog, FoodEntry};

/// Database manager that handles Turso Cloud sync
pub struct DbManager {
    /// The libsql database instance with embedded replica
    db: Database,
    /// Active connection to the database
    conn: Connection,
}

impl DbManager {
    /// Creates a new database manager with Turso Cloud sync
    ///
    /// This function:
    /// 1. Gets the local database path (~/.mountains/mountains.db)
    /// 2. Reads Turso credentials from environment variables
    /// 3. Sets up an embedded replica that syncs with Turso Cloud
    /// 4. Initializes the database schema
    ///
    /// The embedded replica approach means:
    /// - Reads happen from the local database (fast)
    /// - Writes go to both local and remote (synced automatically)
    /// - Works offline with automatic sync when connection is restored
    pub async fn new(data_dir: &PathBuf) -> Result<Self> {
        // Get local database path
        let db_path = data_dir.join("mountains.db");
        let db_path_str = db_path
            .to_str()
            .context("Failed to convert database path to string")?;

        // Read Turso configuration from environment
        let turso_url = env::var("TURSO_DATABASE_URL");
        let turso_token = env::var("TURSO_AUTH_TOKEN");

        // Create database connection based on whether Turso credentials are available
        let db = match (turso_url, turso_token) {
            (Ok(url), Ok(token)) => {
                // Both credentials available - use embedded replica with cloud sync
                Builder::new_remote_replica(db_path_str, url, token)
                    .sync_interval(Duration::from_secs(300)) // Sync every 5 minutes
                    .build()
                    .await
                    .context("Failed to create Turso embedded replica")?
            }
            _ => {
                // Missing credentials - use local-only database
                Builder::new_local(db_path_str)
                    .build()
                    .await
                    .context("Failed to create local database")?
            }
        };

        // Get connection from database
        let conn = db.connect().context("Failed to connect to database")?;

        let mut manager = Self { db, conn };

        // Initialize database schema
        manager.init_schema().await?;

        Ok(manager)
    }

    /// Initializes the database schema
    ///
    /// Creates tables if they don't exist:
    /// - daily_logs: Primary table for daily records
    /// - food_entries: Food items with foreign key to daily_logs
    async fn init_schema(&mut self) -> Result<()> {
        // Create daily_logs table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS daily_logs (
                    date TEXT PRIMARY KEY,
                    weight REAL,
                    waist REAL,
                    notes TEXT
                )",
                (),
            )
            .await
            .context("Failed to create daily_logs table")?;

        // Create food_entries table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS food_entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    name TEXT NOT NULL,
                    notes TEXT,
                    FOREIGN KEY (date) REFERENCES daily_logs(date) ON DELETE CASCADE
                )",
                (),
            )
            .await
            .context("Failed to create food_entries table")?;

        // Create index on date for faster queries
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_food_entries_date ON food_entries(date)",
                (),
            )
            .await
            .context("Failed to create index on food_entries")?;

        // Run migrations to add new columns to daily_logs table
        // SQLite allows ALTER TABLE ADD COLUMN if the column doesn't exist
        let _ = self
            .conn
            .execute("ALTER TABLE daily_logs ADD COLUMN miles_covered REAL", ())
            .await; // Ignore error if column already exists

        let _ = self
            .conn
            .execute("ALTER TABLE daily_logs ADD COLUMN elevation_gain INTEGER", ())
            .await; // Ignore error if column already exists

        let _ = self
            .conn
            .execute("ALTER TABLE daily_logs ADD COLUMN strength_mobility TEXT", ())
            .await; // Ignore error if column already exists

        // Create sokay_entries table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS sokay_entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    date TEXT NOT NULL,
                    entry_text TEXT NOT NULL,
                    FOREIGN KEY (date) REFERENCES daily_logs(date) ON DELETE CASCADE
                )",
                (),
            )
            .await
            .context("Failed to create sokay_entries table")?;

        // Create index on date for faster queries
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_sokay_entries_date ON sokay_entries(date)",
                (),
            )
            .await
            .context("Failed to create index on sokay_entries")?;

        Ok(())
    }

    /// Saves a daily log to the database
    ///
    /// This performs a complete save operation:
    /// 1. Upserts the daily_logs record (updates if exists, inserts if not)
    /// 2. Deletes all existing food entries for the date
    /// 3. Inserts all current food entries
    ///
    /// Uses a transaction to ensure all-or-nothing behavior.
    pub async fn save_daily_log(&mut self, log: &DailyLog) -> Result<()> {
        let date_str = log.date.format("%Y-%m-%d").to_string();

        // Start a transaction for atomic operations
        let tx = self.conn.transaction().await?;

        // Upsert daily_logs record
        tx.execute(
            "INSERT OR REPLACE INTO daily_logs (date, weight, waist, miles_covered, elevation_gain, strength_mobility, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            libsql::params![
                date_str.clone(),
                log.weight,
                log.waist,
                log.miles_covered,
                log.elevation_gain,
                log.strength_mobility.as_ref().map(|s| s.as_str()),
                log.notes.as_ref().map(|s| s.as_str()),
            ],
        )
        .await
        .context("Failed to save daily log")?;

        // Delete existing food entries for this date
        tx.execute("DELETE FROM food_entries WHERE date = ?1", [date_str.as_str()])
            .await
            .context("Failed to delete old food entries")?;

        // Insert all food entries
        for entry in &log.food_entries {
            tx.execute(
                "INSERT INTO food_entries (date, name, notes) VALUES (?1, ?2, ?3)",
                libsql::params![
                    date_str.clone(),
                    entry.name.clone(),
                    entry.notes.clone(),
                ],
            )
            .await
            .context("Failed to insert food entry")?;
        }

        // Delete existing sokay entries for this date
        tx.execute("DELETE FROM sokay_entries WHERE date = ?1", [date_str.as_str()])
            .await
            .context("Failed to delete old sokay entries")?;

        // Insert all sokay entries
        for entry in &log.sokay_entries {
            tx.execute(
                "INSERT INTO sokay_entries (date, entry_text) VALUES (?1, ?2)",
                libsql::params![
                    date_str.clone(),
                    entry.clone(),
                ],
            )
            .await
            .context("Failed to insert sokay entry")?;
        }

        // Commit the transaction
        tx.commit().await.context("Failed to commit transaction")?;

        // Trigger manual sync after save
        self.sync().await;

        Ok(())
    }

    /// Loads all daily logs from the database
    ///
    /// Returns a vector of all daily logs, sorted by date (newest first).
    pub async fn load_all_daily_logs(&self) -> Result<Vec<DailyLog>> {
        // Query all dates from daily_logs
        let mut rows = self
            .conn
            .query(
                "SELECT date, weight, waist, miles_covered, elevation_gain, strength_mobility, notes FROM daily_logs ORDER BY date DESC",
                (),
            )
            .await
            .context("Failed to query all daily logs")?;

        let mut daily_logs = Vec::new();

        while let Some(row) = rows.next().await? {
            let date_str: String = row.get(0)?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse date from database")?;

            let weight: Option<f32> = row.get::<Option<f64>>(1)?.map(|v| v as f32);
            let waist: Option<f32> = row.get::<Option<f64>>(2)?.map(|v| v as f32);
            let miles_covered: Option<f32> = row.get::<Option<f64>>(3)?.map(|v| v as f32);
            let elevation_gain: Option<i32> = row.get::<Option<i64>>(4)?.map(|v| v as i32);
            let strength_mobility: Option<String> = row.get(5)?;
            let notes: Option<String> = row.get(6)?;

            // Query food entries for this date
            let mut food_rows = self
                .conn
                .query(
                    "SELECT name, notes FROM food_entries WHERE date = ?1 ORDER BY id",
                    [date_str.as_str()],
                )
                .await
                .context("Failed to query food entries")?;

            let mut food_entries = Vec::new();
            while let Some(food_row) = food_rows.next().await? {
                let name: String = food_row.get(0)?;
                let food_notes: Option<String> = food_row.get(1)?;
                food_entries.push(FoodEntry::new(name, food_notes));
            }

            // Query sokay entries for this date
            let mut sokay_rows = self
                .conn
                .query(
                    "SELECT entry_text FROM sokay_entries WHERE date = ?1 ORDER BY id",
                    [date_str.as_str()],
                )
                .await
                .context("Failed to query sokay entries")?;

            let mut sokay_entries = Vec::new();
            while let Some(sokay_row) = sokay_rows.next().await? {
                let entry_text: String = sokay_row.get(0)?;
                sokay_entries.push(entry_text);
            }

            daily_logs.push(DailyLog {
                date,
                food_entries,
                weight,
                waist,
                miles_covered,
                elevation_gain,
                sokay_entries,
                strength_mobility,
                notes,
            });
        }

        Ok(daily_logs)
    }

    /// Manually triggers a sync with Turso Cloud
    ///
    /// This is called after save operations to ensure changes are synced promptly.
    /// Errors are silently ignored since sync is a best-effort operation.
    /// Changes are saved locally and will sync when connection is restored.
    async fn sync(&self) {
        let _ = self.db.sync().await; // Ignore sync errors - best effort
    }

    /// Public method to manually trigger a sync with Turso Cloud
    ///
    /// This is called periodically by the application (every 60 seconds) to keep
    /// the local database in sync with the cloud. Unlike the private sync() method,
    /// this returns a Result to allow the caller to handle errors if needed.
    ///
    /// Returns Ok(()) on successful sync, or an error if sync fails.
    pub async fn sync_now(&self) -> Result<()> {
        self.db
            .sync()
            .await
            .context("Failed to sync with Turso Cloud")?;
        Ok(())
    }

    /// Deletes an entire daily log and all associated entries
    ///
    /// This performs a complete deletion:
    /// 1. Deletes all food entries for the date (cascaded by foreign key)
    /// 2. Deletes all sokay entries for the date (cascaded by foreign key)
    /// 3. Deletes the daily_logs record
    ///
    /// The foreign key constraints with ON DELETE CASCADE ensure that
    /// deleting the daily_logs record automatically deletes related entries.
    pub async fn delete_daily_log(&mut self, date: NaiveDate) -> Result<()> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Start a transaction for atomic deletion
        let tx = self.conn.transaction().await?;

        // Delete the daily_logs record (this will cascade to food_entries and sokay_entries)
        tx.execute("DELETE FROM daily_logs WHERE date = ?1", [date_str.as_str()])
            .await
            .context("Failed to delete daily log")?;

        // Commit the transaction
        tx.commit().await.context("Failed to commit transaction")?;

        // Trigger manual sync after deletion
        self.sync().await;

        Ok(())
    }
}
