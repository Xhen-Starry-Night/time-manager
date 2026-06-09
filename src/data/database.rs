use crate::data::error::{DataError, Result};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument, span, trace, Level};

const DB_FILENAME: &str = "app.db";

pub struct Database {
    conn: Connection,
    db_path: PathBuf,
}

impl Database {
    #[instrument(level = "info", name = "Database::open", skip(path), fields(db_path = %path.display()))]
    pub fn open(path: &Path) -> Result<Self> {
        let span = span!(Level::DEBUG, "database_setup");
        let _enter = span.enter();

        if let Some(parent) = path.parent() {
            debug!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent)?;
        }

        debug!("Opening SQLite connection");
        let conn = Connection::open(path)?;

        let db = Self {
            conn,
            db_path: path.to_path_buf(),
        };

        info!("Database opened successfully at: {}", path.display());

        trace!("Creating tables");
        db.create_tables()?;

        trace!("Running migrations");
        db.migrate()?;

        Ok(db)
    }

    #[instrument(level = "info", name = "Database::open_default")]
    pub fn open_default() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("", "", "time-manager")
            .ok_or_else(|| DataError::InvalidData("cannot determine data directory".into()))?;
        let data_dir = proj_dirs.data_dir().join("data");

        debug!("Creating data directory: {}", data_dir.display());
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join(DB_FILENAME);
        Self::open(&db_path)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    #[instrument(level = "debug", skip(self))]
    fn create_tables(&self) -> Result<()> {
        debug!("Executing table creation batch");
        self.conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;

             CREATE TABLE IF NOT EXISTS settings (
                 key   TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS categories (
                 id                              INTEGER PRIMARY KEY AUTOINCREMENT,
                 parent_id                       INTEGER,
                 name                            TEXT NOT NULL,
                 path                            TEXT NOT NULL UNIQUE,
                 node_type                       TEXT NOT NULL DEFAULT 'directory',
                 source                          TEXT,
                 default_quality                 TEXT,
                 default_understanding_difficulty TEXT,
                 default_memory_difficulty       TEXT,
                 FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE
             );

             CREATE TABLE IF NOT EXISTS sessions (
                 id                       INTEGER PRIMARY KEY AUTOINCREMENT,
                 category_id              INTEGER NOT NULL,
                 start_time               TEXT NOT NULL,
                 end_time                 TEXT NOT NULL,
                 duration_secs            INTEGER NOT NULL,
                 pause_records            TEXT NOT NULL DEFAULT '[]',
                 quality                  TEXT NOT NULL,
                 understanding_difficulty TEXT NOT NULL,
                 memory_difficulty        TEXT NOT NULL,
                 completion_rate          INTEGER NOT NULL,
                 note                     TEXT,
                 is_manual_edit           INTEGER NOT NULL DEFAULT 0,
                 FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
             );

             CREATE TABLE IF NOT EXISTS prediction_states (
                 id              INTEGER PRIMARY KEY AUTOINCREMENT,
                 category_id     INTEGER NOT NULL UNIQUE,
                 algorithm       TEXT NOT NULL,
                 last_review     TEXT NOT NULL,
                 next_review     TEXT NOT NULL,
                 algorithm_state BLOB NOT NULL,
                 FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
             );

             CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
             CREATE INDEX IF NOT EXISTS idx_categories_path ON categories(path);
             CREATE INDEX IF NOT EXISTS idx_sessions_category ON sessions(category_id);
             CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_time);
             CREATE INDEX IF NOT EXISTS idx_prediction_next_review ON prediction_states(next_review);",
        )?;

        debug!("Tables created successfully");
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn migrate(&self) -> Result<()> {
        debug!("Checking for required migrations");

        let has_node_type: bool = self.conn
            .prepare("SELECT node_type FROM categories LIMIT 0")
            .is_ok();

        if !has_node_type {
            info!("Running migration: adding node_type column");
            self.conn.execute_batch(
                "ALTER TABLE categories ADD COLUMN node_type TEXT NOT NULL DEFAULT 'directory';",
            )?;
            debug!("Migration completed successfully");
        } else {
            trace!("No migrations required");
        }

        Ok(())
    }

    #[instrument(level = "debug", skip(self, cat), fields(name = %cat.name, path = %cat.path))]
    pub fn insert_category(&self, cat: &crate::data::models::CategoryInsert) -> Result<i64> {
        trace!("Preparing insert category statement");
        let q = cat.default_quality.as_ref().map(|q| q.as_str());
        let ud = cat.default_understanding_difficulty.as_ref().map(|d| d.as_str());
        let md = cat.default_memory_difficulty.as_ref().map(|d| d.as_str());

        self.conn.execute(
            "INSERT INTO categories (parent_id, name, path, node_type, source, default_quality, default_understanding_difficulty, default_memory_difficulty)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![cat.parent_id, cat.name, cat.path, cat.node_type.as_str(), cat.source, q, ud, md],
        )?;

        let id = self.conn.last_insert_rowid();
        debug!("Category inserted with id: {}", id);
        Ok(id)
    }

    #[instrument(level = "trace", skip(self))]
    pub fn get_category(&self, id: i64) -> Result<crate::data::models::Category> {
        trace!("Querying category id: {}", id);
        self.conn.query_row(
            "SELECT id, parent_id, name, path, node_type, source, default_quality, default_understanding_difficulty, default_memory_difficulty
             FROM categories WHERE id = ?1",
            params![id],
            map_category_row,
        ).map_err(Into::into)
    }

    #[instrument(level = "debug", skip(self))]
    pub fn get_all_categories(&self) -> Result<Vec<crate::data::models::Category>> {
        debug!("Querying all categories");
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, name, path, node_type, source, default_quality, default_understanding_difficulty, default_memory_difficulty
             FROM categories ORDER BY path",
        )?;
        let rows = stmt.query_map([], map_category_row)?;
        let categories = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        trace!("Retrieved {} categories", categories.len());
        Ok(categories)
    }

    #[instrument(level = "trace", skip(self))]
    pub fn get_children(&self, parent_id: Option<i64>) -> Result<Vec<crate::data::models::Category>> {
        trace!("Querying children of parent_id: {:?}", parent_id);
        let mut stmt = if parent_id.is_some() {
            self.conn.prepare(
                "SELECT id, parent_id, name, path, node_type, source, default_quality, default_understanding_difficulty, default_memory_difficulty
                 FROM categories WHERE parent_id = ?1 ORDER BY name",
            )?
        } else {
            self.conn.prepare(
                "SELECT id, parent_id, name, path, node_type, source, default_quality, default_understanding_difficulty, default_memory_difficulty
                 FROM categories WHERE parent_id IS NULL ORDER BY name",
            )?
        };
        let rows = if let Some(pid) = parent_id {
            stmt.query_map(params![pid], map_category_row)?
        } else {
            stmt.query_map([], map_category_row)?
        };
        rows.collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    #[instrument(level = "debug", skip(self), fields(id, new_name))]
    pub fn update_category_name(&self, id: i64, name: &str) -> Result<()> {
        debug!("Updating category {} name to: {}", id, name);
        self.conn.execute("UPDATE categories SET name = ?1 WHERE id = ?2", params![name, id])?;
        trace!("Category name updated successfully");
        Ok(())
    }

    #[instrument(level = "info", skip(self), fields(id))]
    pub fn delete_category(&self, id: i64) -> Result<()> {
        info!("Deleting category id: {}", id);
        self.conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        debug!("Category deleted successfully");
        Ok(())
    }

    #[instrument(level = "info", skip(self, s), fields(category_id = s.category_id, duration_secs = s.duration_secs))]
    pub fn insert_session(&self, s: &crate::data::models::SessionInsert) -> Result<i64> {
        debug!("Inserting session for category {}", s.category_id);
        let pause_json = serde_json::to_string(&s.pause_records)?;
        trace!("Pause records JSON length: {} bytes", pause_json.len());

        self.conn.execute(
            "INSERT INTO sessions (category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                s.category_id,
                s.start_time.to_rfc3339(),
                s.end_time.to_rfc3339(),
                s.duration_secs,
                pause_json,
                s.params.quality.as_str(),
                s.params.understanding_difficulty.as_str(),
                s.params.memory_difficulty.as_str(),
                s.params.completion_rate,
                s.note,
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        info!("Session inserted with id: {}, duration: {}s", id, s.duration_secs);
        Ok(id)
    }

    #[instrument(level = "trace", skip(self))]
    pub fn get_session(&self, id: i64) -> Result<crate::data::models::Session> {
        trace!("Querying session id: {}", id);
        self.conn.query_row(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions WHERE id = ?1",
            params![id],
            map_session_row,
        ).map_err(Into::into)
    }

    #[instrument(level = "debug", skip(self), fields(category_id))]
    pub fn get_sessions_by_category(&self, category_id: i64) -> Result<Vec<crate::data::models::Session>> {
        debug!("Querying sessions for category {}", category_id);
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions WHERE category_id = ?1 ORDER BY start_time DESC",
        )?;
        let rows = stmt.query_map(params![category_id], map_session_row)?;
        let sessions = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        trace!("Retrieved {} sessions", sessions.len());
        Ok(sessions)
    }

    #[instrument(level = "debug", skip(self), fields(start, end))]
    pub fn get_sessions_in_range(
        &self,
        start: &chrono::DateTime<chrono::Utc>,
        end: &chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<crate::data::models::Session>> {
        debug!("Querying sessions in range {} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"));
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions WHERE start_time >= ?1 AND start_time < ?2 ORDER BY start_time",
        )?;
        let rows = stmt.query_map(
            params![start.to_rfc3339(), end.to_rfc3339()],
            map_session_row,
        )?;
        let sessions = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        trace!("Retrieved {} sessions in range", sessions.len());
        Ok(sessions)
    }

    #[instrument(level = "debug", skip(self))]
    pub fn get_all_sessions(&self) -> Result<Vec<crate::data::models::Session>> {
        debug!("Querying all sessions");
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions ORDER BY start_time DESC",
        )?;
        let rows = stmt.query_map([], map_session_row)?;
        let sessions = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        trace!("Retrieved {} sessions total", sessions.len());
        Ok(sessions)
    }

    #[instrument(level = "debug", skip(self), fields(id))]
    pub fn update_session_note(&self, id: i64, note: &str) -> Result<()> {
        debug!("Updating session {} note", id);
        self.conn.execute(
            "UPDATE sessions SET note = ?1, is_manual_edit = 1 WHERE id = ?2",
            params![note, id],
        )?;
        trace!("Session note updated");
        Ok(())
    }

    #[instrument(level = "info", skip(self), fields(id))]
    pub fn delete_session(&self, id: i64) -> Result<()> {
        info!("Deleting session id: {}", id);
        self.conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        debug!("Session deleted");
        Ok(())
    }

    #[instrument(level = "debug", skip(self, ps), fields(category_id = ps.category_id, algorithm = %ps.algorithm))]
    pub fn upsert_prediction_state(&self, ps: &crate::data::models::PredictionStateInsert) -> Result<()> {
        debug!("Upserting prediction state for category {}", ps.category_id);
        self.conn.execute(
            "INSERT INTO prediction_states (category_id, algorithm, last_review, next_review, algorithm_state)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(category_id) DO UPDATE SET algorithm=?2, last_review=?3, next_review=?4, algorithm_state=?5",
            params![
                ps.category_id,
                ps.algorithm,
                ps.last_review.to_rfc3339(),
                ps.next_review.to_rfc3339(),
                ps.algorithm_state,
            ],
        )?;
        trace!("Prediction state upserted: next_review = {}", ps.next_review.format("%Y-%m-%d"));
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    pub fn get_prediction_state(&self, category_id: i64) -> Result<Option<crate::data::models::PredictionState>> {
        trace!("Querying prediction state for category {}", category_id);
        let result = self.conn.query_row(
            "SELECT id, category_id, algorithm, last_review, next_review, algorithm_state
             FROM prediction_states WHERE category_id = ?1",
            params![category_id],
            |row| {
                Ok(crate::data::models::PredictionState {
                    id: row.get(0)?,
                    category_id: row.get(1)?,
                    algorithm: row.get(2)?,
                    last_review: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
                    next_review: row.get::<_, String>(4)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
                    algorithm_state: row.get(5)?,
                })
            },
        );
        match result {
            Ok(ps) => {
                trace!("Found prediction state: algorithm = {}, next_review = {}", ps.algorithm, ps.next_review.format("%Y-%m-%d"));
                Ok(Some(ps))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                trace!("No prediction state found for category {}", category_id);
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn get_due_predictions(&self, before: &chrono::DateTime<chrono::Utc>) -> Result<Vec<crate::data::models::PredictionState>> {
        debug!("Querying due predictions before {}", before.format("%Y-%m-%d %H:%M"));
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, algorithm, last_review, next_review, algorithm_state
             FROM prediction_states WHERE next_review <= ?1 ORDER BY next_review ASC",
        )?;
        let rows = stmt.query_map(params![before.to_rfc3339()], |row| {
            Ok(crate::data::models::PredictionState {
                id: row.get(0)?,
                category_id: row.get(1)?,
                algorithm: row.get(2)?,
                last_review: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
                next_review: row.get::<_, String>(4)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
                algorithm_state: row.get(5)?,
            })
        })?;
        let predictions = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        trace!("Found {} due predictions", predictions.len());
        Ok(predictions)
    }

    #[instrument(level = "debug", skip(self))]
    pub fn get_all_prediction_states(&self) -> Result<Vec<crate::data::models::PredictionState>> {
        debug!("Querying all prediction states");
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, algorithm, last_review, next_review, algorithm_state
             FROM prediction_states ORDER BY next_review ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::data::models::PredictionState {
                id: row.get(0)?,
                category_id: row.get(1)?,
                algorithm: row.get(2)?,
                last_review: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
                next_review: row.get::<_, String>(4)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
                algorithm_state: row.get(5)?,
            })
        })?;
        let states = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        trace!("Retrieved {} prediction states", states.len());
        Ok(states)
    }

    #[instrument(level = "info", skip(self), fields(category_id))]
    pub fn delete_prediction_state(&self, category_id: i64) -> Result<()> {
        info!("Deleting prediction state for category {}", category_id);
        self.conn.execute("DELETE FROM prediction_states WHERE category_id = ?1", params![category_id])?;
        debug!("Prediction state deleted");
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        trace!("Querying setting: {}", key);
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(v) => {
                trace!("Setting '{}' = {}", key, v);
                Ok(Some(v))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                trace!("Setting '{}' not found", key);
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(level = "debug", skip(self), fields(key))]
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        debug!("Setting '{}' = {}", key, value);
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=?2",
            params![key, value],
        )?;
        trace!("Setting saved");
        Ok(())
    }
}

fn parse_quality(s: &str) -> Option<crate::data::models::Quality> {
    use crate::data::models::Quality::*;
    match s {
        "极低" => Some(VeryLow),
        "低" => Some(Low),
        "中" => Some(Medium),
        "高" => Some(High),
        "完整" => Some(Complete),
        _ => None,
    }
}

fn parse_difficulty(s: &str) -> Option<crate::data::models::Difficulty> {
    use crate::data::models::Difficulty::*;
    match s {
        "极易" => Some(VeryEasy),
        "易" => Some(Easy),
        "中" => Some(Medium),
        "难" => Some(Hard),
        "极难" => Some(VeryHard),
        _ => None,
    }
}

fn map_category_row(row: &rusqlite::Row<'_>) -> std::result::Result<crate::data::models::Category, rusqlite::Error> {
    let node_type_str: String = row.get(4)?;
    Ok(crate::data::models::Category {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        name: row.get(2)?,
        path: row.get(3)?,
        node_type: crate::data::models::NodeType::from_str(&node_type_str)
            .unwrap_or_default(),
        source: row.get(5)?,
        default_quality: row.get::<_, Option<String>>(6)?.as_deref().and_then(parse_quality),
        default_understanding_difficulty: row.get::<_, Option<String>>(7)?.as_deref().and_then(parse_difficulty),
        default_memory_difficulty: row.get::<_, Option<String>>(8)?.as_deref().and_then(parse_difficulty),
    })
}

fn map_session_row(row: &rusqlite::Row<'_>) -> std::result::Result<crate::data::models::Session, rusqlite::Error> {
    use crate::data::models::*;
    let pause_json: String = row.get(5)?;
    let pause_records: Vec<PauseRecord> = serde_json::from_str(&pause_json).unwrap_or_default();
    Ok(Session {
        id: row.get(0)?,
        category_id: row.get(1)?,
        start_time: row.get::<_, String>(2)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
        end_time: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
        duration_secs: row.get(4)?,
        pause_records,
        quality: row.get::<_, String>(6)?.as_str().try_into().unwrap_or(Quality::Medium),
        understanding_difficulty: row.get::<_, String>(7)?.as_str().try_into().unwrap_or(Difficulty::Medium),
        memory_difficulty: row.get::<_, String>(8)?.as_str().try_into().unwrap_or(Difficulty::Medium),
        completion_rate: row.get(9)?,
        note: row.get(10)?,
        is_manual_edit: row.get::<_, i32>(11)? != 0,
    })
}