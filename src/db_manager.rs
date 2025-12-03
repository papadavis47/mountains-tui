use anyhow::{Context, Result};
use chrono::NaiveDate;
use libsql::{Builder, Connection, Database};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::{DailyLog, FoodEntry};

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected,
    Error(String),
}

pub struct DbManager {
    db: Database,
    conn: Connection,
    connection_state: Arc<RwLock<ConnectionState>>,
}

impl DbManager {
    pub async fn new_local_first(data_dir: &Path) -> Result<Self> {
        // Get local database path
        let db_path = data_dir.join("mountains.db");
        let db_path_str = db_path
            .to_str()
            .context("Failed to convert database path to string")?
            .to_string();

        // Always start with local connection for instant startup
        let db = Builder::new_local(&db_path_str).build().await?;
        let conn = db.connect()?;

        // Start disconnected - will upgrade to cloud replica in background if credentials available
        let state = ConnectionState::Disconnected;

        let mut manager = Self {
            db,
            conn,
            connection_state: Arc::new(RwLock::new(state)),
        };

        // Always initialize schema (needed even for in-memory placeholder)
        manager.init_schema().await?;

        Ok(manager)
    }

    /// Upgrades local database to remote replica (recreates as libsql can't convert in-place)
    pub async fn upgrade_to_remote_replica(
        &mut self,
        db_path_str: &str,
        url: String,
        token: String,
    ) -> Result<()> {
        use std::path::Path;

        *self.connection_state.write().await = ConnectionState::Disconnected;

        // Check if metadata file exists (indicating this is already a replica)
        let metadata_path = format!("{}-info", db_path_str);
        let is_already_replica = Path::new(&metadata_path).exists();

        if !is_already_replica {
            // Delete the local database files to start fresh
            // libsql cannot convert a local database to a remote replica
            let db_path = Path::new(db_path_str);
            if db_path.exists() {
                std::fs::remove_file(db_path).ok();
            }
            // Also remove WAL and SHM files if they exist
            let wal_path = format!("{}-wal", db_path_str);
            let shm_path = format!("{}-shm", db_path_str);
            std::fs::remove_file(&wal_path).ok();
            std::fs::remove_file(&shm_path).ok();
        }

        // Create or connect to remote replica
        match Builder::new_remote_replica(db_path_str, url, token)
            .build()
            .await
        {
            Ok(new_db) => {
                match new_db.connect() {
                    Ok(new_conn) => {
                        // Replace the database connection
                        self.db = new_db;
                        self.conn = new_conn;

                        // Only reinitialize schema if we deleted the database
                        if !is_already_replica {
                            self.init_schema().await?;
                        }

                        *self.connection_state.write().await = ConnectionState::Connected;
                        Ok(())
                    }
                    Err(e) => {
                        *self.connection_state.write().await =
                            ConnectionState::Error(format!("Failed to connect: {}", e));
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                *self.connection_state.write().await =
                    ConnectionState::Error(format!("Failed to create replica: {}", e));
                Err(e.into())
            }
        }
    }

    pub async fn get_connection_state(&self) -> ConnectionState {
        self.connection_state.read().await.clone()
    }

    async fn init_schema(&mut self) -> Result<()> {
        // Create daily_logs table with all columns
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS daily_logs (
                    date TEXT PRIMARY KEY,
                    weight REAL,
                    waist REAL,
                    miles_covered REAL,
                    elevation_gain INTEGER,
                    strength_mobility TEXT,
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
                log.strength_mobility.as_deref(),
                log.notes.as_deref(),
            ],
        )
        .await
        .context("Failed to save daily log")?;

        // Delete existing food entries for this date
        tx.execute(
            "DELETE FROM food_entries WHERE date = ?1",
            [date_str.as_str()],
        )
        .await
        .context("Failed to delete old food entries")?;

        // Insert all food entries
        for entry in &log.food_entries {
            tx.execute(
                "INSERT INTO food_entries (date, name) VALUES (?1, ?2)",
                libsql::params![date_str.clone(), entry.name.clone(),],
            )
            .await
            .context("Failed to insert food entry")?;
        }

        // Delete existing sokay entries for this date
        tx.execute(
            "DELETE FROM sokay_entries WHERE date = ?1",
            [date_str.as_str()],
        )
        .await
        .context("Failed to delete old sokay entries")?;

        // Insert all sokay entries
        for entry in &log.sokay_entries {
            tx.execute(
                "INSERT INTO sokay_entries (date, entry_text) VALUES (?1, ?2)",
                libsql::params![date_str.clone(), entry.clone(),],
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
                    "SELECT name FROM food_entries WHERE date = ?1 ORDER BY id",
                    [date_str.as_str()],
                )
                .await
                .context("Failed to query food entries")?;

            let mut food_entries = Vec::new();
            while let Some(food_row) = food_rows.next().await? {
                let name: String = food_row.get(0)?;
                food_entries.push(FoodEntry::new(name));
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

    /// Best-effort sync after save/delete operations
    async fn sync(&self) {
        // Only sync if we're connected to Turso
        let state = self.connection_state.read().await;
        if *state != ConnectionState::Connected {
            return; // Skip sync if not connected
        }
        drop(state); // Release lock before sync

        let _ = self.db.sync().await; // Ignore sync errors - best effort
    }

    /// Periodic sync (called every 4 minutes by background task)
    pub async fn sync_now(&self) -> Result<()> {
        // Only sync if we're connected to Turso
        let state = self.connection_state.read().await;
        if *state != ConnectionState::Connected {
            return Ok(()); // Skip sync if not connected, but don't error
        }
        drop(state); // Release lock before sync

        self.db
            .sync()
            .await
            .context("Failed to sync with Turso Cloud")?;
        Ok(())
    }

    pub async fn delete_daily_log(&mut self, date: NaiveDate) -> Result<()> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Start a transaction for atomic deletion
        let tx = self.conn.transaction().await?;

        // Delete the daily_logs record (this will cascade to food_entries and sokay_entries)
        tx.execute(
            "DELETE FROM daily_logs WHERE date = ?1",
            [date_str.as_str()],
        )
        .await
        .context("Failed to delete daily log")?;

        // Commit the transaction
        tx.commit().await.context("Failed to commit transaction")?;

        // Trigger manual sync after deletion
        self.sync().await;

        Ok(())
    }
}
