# 数据层 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立SQLite数据访问层，包含数据模型定义、数据库CRUD操作、数据导出功能。

**Architecture:** 使用rusqlite嵌入式SQLite，数据模型定义为Rust结构体，Database结构体封装所有SQL操作，Exporter trait + 具体实现支持JSONL/CSV/Markdown导出。数据库文件按XDG规范存放于`~/.local/share/time-manager/data/app.db`。

**Tech Stack:** Rust, rusqlite 0.40, serde (JSON序列化), chrono (时间处理), directories (XDG路径)

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/data/mod.rs` | 模块入口，re-export |
| `src/data/models.rs` | 数据模型定义 (Category, Session, PredictionState, SessionParams, Preset, Setting) |
| `src/data/database.rs` | Database结构体，SQLite连接管理，建表，CRUD操作 |
| `src/data/export.rs` | Exporter trait + JsonExporter/CsvExporter/MarkdownExporter |
| `src/data/error.rs` | 数据层错误类型定义 |

---

### Task 1: Cargo.tom 依赖与项目基础

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: 添加依赖到 Cargo.tom**

```toml
[package]
name = "time-manager"
version = "0.1.0"
edition = "2024"

[dependencies]
rusqlite = { version = "0.40", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
directories = "6"
thiserror = "2"
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: 编译成功（仅有unused warning）

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add data layer dependencies"
```

---

### Task 2: 数据层错误类型

**Files:**
- Create: `src/data/error.rs`
- Create: `src/data/mod.rs`

- [ ] **Step 1: 创建 error.rs**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("record not found: {table} id={id}")]
    NotFound { table: String, id: i64 },

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("export error: {0}")]
    Export(String),
}

pub type Result<T> = std::result::Result<T, DataError>;
```

- [ ] **Step 2: 创建 data/mod.rs**

```rust
pub mod database;
pub mod error;
pub mod export;
pub mod models;

pub use database::Database;
pub use error::{DataError, Result};
```

- [ ] **Step 3: 验证编译**

Run: `cargo check`
Expected: FAIL - modules export/database/export/models not yet created

- [ ] **Step 4: 创建占位文件使编译通过**

创建 `src/data/models.rs`:
```rust
```

创建 `src/data/database.rs`:
```rust
```

创建 `src/data/export.rs`:
```rust
```

- [ ] **Step 5: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/data/
git commit -m "feat: add data layer module structure and error types"
```

---

### Task 3: 数据模型定义

**Files:**
- Modify: `src/data/models.rs`

- [ ] **Step 1: 定义所有数据模型**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Quality {
    VeryLow,
    Low,
    Medium,
    High,
    Complete,
}

impl Quality {
    pub fn all() -> Vec<Self> {
        vec![
            Self::VeryLow,
            Self::Low,
            Self::Medium,
            Self::High,
            Self::Complete,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VeryLow => "极低",
            Self::Low => "低",
            Self::Medium => "中",
            Self::High => "高",
            Self::Complete => "完整",
        }
    }
}

impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Medium,
    Hard,
    VeryHard,
}

impl Difficulty {
    pub fn all() -> Vec<Self> {
        vec![
            Self::VeryEasy,
            Self::Easy,
            Self::Medium,
            Self::Hard,
            Self::VeryHard,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VeryEasy => "极易",
            Self::Easy => "易",
            Self::Medium => "中",
            Self::Hard => "难",
            Self::VeryHard => "极难",
        }
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParams {
    pub quality: Quality,
    pub understanding_difficulty: Difficulty,
    pub memory_difficulty: Difficulty,
    pub completion_rate: u8,
}

impl Default for SessionParams {
    fn default() -> Self {
        Self {
            quality: Quality::Medium,
            understanding_difficulty: Difficulty::Medium,
            memory_difficulty: Difficulty::Medium,
            completion_rate: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub path: String,
    pub source: Option<String>,
    pub default_quality: Option<Quality>,
    pub default_understanding_difficulty: Option<Difficulty>,
    pub default_memory_difficulty: Option<Difficulty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInsert {
    pub parent_id: Option<i64>,
    pub name: String,
    pub path: String,
    pub source: Option<String>,
    pub default_quality: Option<Quality>,
    pub default_understanding_difficulty: Option<Difficulty>,
    pub default_memory_difficulty: Option<Difficulty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseRecord {
    pub pause_start: DateTime<Utc>,
    pub resume_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub category_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: i64,
    pub pause_records: Vec<PauseRecord>,
    pub quality: Quality,
    pub understanding_difficulty: Difficulty,
    pub memory_difficulty: Difficulty,
    pub completion_rate: u8,
    pub note: Option<String>,
    pub is_manual_edit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInsert {
    pub category_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: i64,
    pub pause_records: Vec<PauseRecord>,
    pub params: SessionParams,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionState {
    pub id: i64,
    pub category_id: i64,
    pub algorithm: String,
    pub last_review: DateTime<Utc>,
    pub next_review: DateTime<Utc>,
    pub algorithm_state: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionStateInsert {
    pub category_id: i64,
    pub algorithm: String,
    pub last_review: DateTime<Utc>,
    pub next_review: DateTime<Utc>,
    pub algorithm_state: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub quality: Quality,
    pub understanding_difficulty: Difficulty,
    pub memory_difficulty: Difficulty,
    pub completion_rate: u8,
}

impl Preset {
    pub fn built_in() -> Vec<Self> {
        vec![
            Self {
                name: "专注学习".into(),
                quality: Quality::High,
                understanding_difficulty: Difficulty::Medium,
                memory_difficulty: Difficulty::Medium,
                completion_rate: 100,
            },
            Self {
                name: "轻松复习".into(),
                quality: Quality::Medium,
                understanding_difficulty: Difficulty::Easy,
                memory_difficulty: Difficulty::Easy,
                completion_rate: 80,
            },
            Self {
                name: "快速浏览".into(),
                quality: Quality::Low,
                understanding_difficulty: Difficulty::Easy,
                memory_difficulty: Difficulty::Easy,
                completion_rate: 50,
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/data/models.rs
git commit -m "feat: define data models for categories, sessions, predictions, presets"
```

---

### Task 4: 数据库初始化与建表

**Files:**
- Modify: `src/data/database.rs`

- [ ] **Step 1: 实现 Database 结构体与建表逻辑**

```rust
use crate::data::error::{DataError, Result};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

const DB_FILENAME: &str = "app.db";

pub struct Database {
    conn: Connection,
    db_path: PathBuf,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let db = Self {
            conn,
            db_path: path.to_path_buf(),
        };
        db.create_tables()?;
        Ok(db)
    }

    pub fn open_default() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("", "", "time-manager")
            .ok_or_else(|| DataError::InvalidData("cannot determine data directory".into()))?;
        let data_dir = proj_dirs.data_dir().join("data");
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join(DB_FILENAME);
        Self::open(&db_path)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    fn create_tables(&self) -> Result<()> {
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
        Ok(())
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/data/database.rs
git commit -m "feat: implement Database with table creation and default path"
```

---

### Task 5: 数据库 CRUD - Categories

**Files:**
- Modify: `src/data/database.rs`

- [ ] **Step 1: 在 Database impl 中添加 Category CRUD 方法**

在 `Database` impl 块中追加：

```rust
    pub fn insert_category(&self, cat: &crate::data::models::CategoryInsert) -> Result<i64> {
        let q = cat.default_quality.as_ref().map(|q| q.as_str());
        let ud = cat.default_understanding_difficulty.as_ref().map(|d| d.as_str());
        let md = cat.default_memory_difficulty.as_ref().map(|d| d.as_str());
        self.conn.execute(
            "INSERT INTO categories (parent_id, name, path, source, default_quality, default_understanding_difficulty, default_memory_difficulty)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![cat.parent_id, cat.name, cat.path, cat.source, q, ud, md],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_category(&self, id: i64) -> Result<crate::data::models::Category> {
        self.conn.query_row(
            "SELECT id, parent_id, name, path, source, default_quality, default_understanding_difficulty, default_memory_difficulty
             FROM categories WHERE id = ?1",
            params![id],
            |row| {
                Ok(crate::data::models::Category {
                    id: row.get(0)?,
                    parent_id: row.get(1)?,
                    name: row.get(2)?,
                    path: row.get(3)?,
                    source: row.get(4)?,
                    default_quality: row.get::<_, Option<String>>(5)?.as_deref().and_then(parse_quality),
                    default_understanding_difficulty: row.get::<_, Option<String>>(6)?.as_deref().and_then(parse_difficulty),
                    default_memory_difficulty: row.get::<_, Option<String>>(7)?.as_deref().and_then(parse_difficulty),
                })
            },
        ).map_err(Into::into)
    }

    pub fn get_all_categories(&self) -> Result<Vec<crate::data::models::Category>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, name, path, source, default_quality, default_understanding_difficulty, default_memory_difficulty
             FROM categories ORDER BY path",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::data::models::Category {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                source: row.get(4)?,
                default_quality: row.get::<_, Option<String>>(5)?.as_deref().and_then(parse_quality),
                default_understanding_difficulty: row.get::<_, Option<String>>(6)?.as_deref().and_then(parse_difficulty),
                default_memory_difficulty: row.get::<_, Option<String>>(7)?.as_deref().and_then(parse_difficulty),
            })
        })?;
        rows.collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    pub fn get_children(&self, parent_id: Option<i64>) -> Result<Vec<crate::data::models::Category>> {
        let mut stmt = if parent_id.is_some() {
            self.conn.prepare(
                "SELECT id, parent_id, name, path, source, default_quality, default_understanding_difficulty, default_memory_difficulty
                 FROM categories WHERE parent_id = ?1 ORDER BY name",
            )?
        } else {
            self.conn.prepare(
                "SELECT id, parent_id, name, path, source, default_quality, default_understanding_difficulty, default_memory_difficulty
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

    pub fn update_category_name(&self, id: i64, name: &str) -> Result<()> {
        self.conn.execute("UPDATE categories SET name = ?1 WHERE id = ?2", params![name, id])?;
        Ok(())
    }

    pub fn delete_category(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        Ok(())
    }
```

在 `database.rs` 底部添加辅助函数：

```rust
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
    Ok(crate::data::models::Category {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        name: row.get(2)?,
        path: row.get(3)?,
        source: row.get(4)?,
        default_quality: row.get::<_, Option<String>>(5)?.as_deref().and_then(parse_quality),
        default_understanding_difficulty: row.get::<_, Option<String>>(6)?.as_deref().and_then(parse_difficulty),
        default_memory_difficulty: row.get::<_, Option<String>>(7)?.as_deref().and_then(parse_difficulty),
    })
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/data/database.rs
git commit -m "feat: add category CRUD operations to Database"
```

---

### Task 6: 数据库 CRUD - Sessions

**Files:**
- Modify: `src/data/database.rs`

- [ ] **Step 1: 在 Database impl 中添加 Session CRUD 方法**

```rust
    pub fn insert_session(&self, s: &crate::data::models::SessionInsert) -> Result<i64> {
        let pause_json = serde_json::to_string(&s.pause_records)?;
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
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_session(&self, id: i64) -> Result<crate::data::models::Session> {
        self.conn.query_row(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions WHERE id = ?1",
            params![id],
            map_session_row,
        ).map_err(Into::into)
    }

    pub fn get_sessions_by_category(&self, category_id: i64) -> Result<Vec<crate::data::models::Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions WHERE category_id = ?1 ORDER BY start_time DESC",
        )?;
        let rows = stmt.query_map(params![category_id], map_session_row)?;
        rows.collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    pub fn get_sessions_in_range(&self, start: &chrono::DateTime<chrono::Utc>, end: &chrono::DateTime<chrono::Utc>) -> Result<Vec<crate::data::models::Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions WHERE start_time >= ?1 AND start_time < ?2 ORDER BY start_time",
        )?;
        let rows = stmt.query_map(
            params![start.to_rfc3339(), end.to_rfc3339()],
            map_session_row,
        )?;
        rows.collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    pub fn get_all_sessions(&self) -> Result<Vec<crate::data::models::Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, start_time, end_time, duration_secs, pause_records, quality, understanding_difficulty, memory_difficulty, completion_rate, note, is_manual_edit
             FROM sessions ORDER BY start_time DESC",
        )?;
        let rows = stmt.query_map([], map_session_row)?;
        rows.collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    pub fn update_session_note(&self, id: i64, note: &str) -> Result<()> {
        self.conn.execute("UPDATE sessions SET note = ?1, is_manual_edit = 1 WHERE id = ?2", params![note, id])?;
        Ok(())
    }

    pub fn delete_session(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        Ok(())
    }
```

在 `database.rs` 底部添加：

```rust
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
```

需要在 `models.rs` 中为 `Quality` 和 `Difficulty` 添加 `TryFrom<&str>` 实现：

```rust
impl TryFrom<&str> for Quality {
    type Error = ();
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "极低" => Ok(Self::VeryLow),
            "低" => Ok(Self::Low),
            "中" => Ok(Self::Medium),
            "高" => Ok(Self::High),
            "完整" => Ok(Self::Complete),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Difficulty {
    type Error = ();
    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s {
            "极易" => Ok(Self::VeryEasy),
            "易" => Ok(Self::Easy),
            "中" => Ok(Self::Medium),
            "难" => Ok(Self::Hard),
            "极难" => Ok(Self::VeryHard),
            _ => Err(()),
        }
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/data/
git commit -m "feat: add session CRUD operations to Database"
```

---

### Task 7: 数据库 CRUD - PredictionStates & Settings

**Files:**
- Modify: `src/data/database.rs`

- [ ] **Step 1: 添加 PredictionState 和 Setting CRUD 方法**

```rust
    pub fn upsert_prediction_state(&self, ps: &crate::data::models::PredictionStateInsert) -> Result<()> {
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
        Ok(())
    }

    pub fn get_prediction_state(&self, category_id: i64) -> Result<Option<crate::data::models::PredictionState>> {
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
            Ok(ps) => Ok(Some(ps)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_due_predictions(&self, before: &chrono::DateTime<chrono::Utc>) -> Result<Vec<crate::data::models::PredictionState>> {
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
        rows.collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    pub fn get_all_prediction_states(&self) -> Result<Vec<crate::data::models::PredictionState>> {
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
        rows.collect::<std::result::Result<_, _>>().map_err(Into::into)
    }

    pub fn delete_prediction_state(&self, category_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM prediction_states WHERE category_id = ?1", params![category_id])?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=?2",
            params![key, value],
        )?;
        Ok(())
    }
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/data/database.rs
git commit -m "feat: add prediction_state and settings CRUD operations"
```

---

### Task 8: 数据导出

**Files:**
- Modify: `src/data/export.rs`

- [ ] **Step 1: 实现 Exporter trait 和三种格式**

```rust
use crate::data::error::{DataError, Result};
use crate::data::models::{PredictionState, Session};
use std::path::Path;

pub trait Exporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf>;
    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf>;
}

pub struct JsonExporter;
pub struct CsvExporter;
pub struct MarkdownExporter;

impl Exporter for JsonExporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("sessions.jsonl");
        let mut file = std::fs::File::create(&path)?;
        use std::io::Write;
        for session in sessions {
            let line = serde_json::to_string(session)?;
            writeln!(file, "{}", line)?;
        }
        Ok(path)
    }

    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("prediction_states.jsonl");
        let mut file = std::fs::File::create(&path)?;
        use std::io::Write;
        for pred in predictions {
            let line = serde_json::to_string(pred)?;
            writeln!(file, "{}", line)?;
        }
        Ok(path)
    }
}

impl Exporter for CsvExporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("sessions.csv");
        let mut file = std::fs::File::create(&path)?;
        use std::io::Write;
        writeln!(file, "id,category_id,start_time,end_time,duration_secs,quality,understanding_difficulty,memory_difficulty,completion_rate,note,is_manual_edit")?;
        for s in sessions {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},{}",
                s.id,
                s.category_id,
                s.start_time.to_rfc3339(),
                s.end_time.to_rfc3339(),
                s.duration_secs,
                s.quality,
                s.understanding_difficulty,
                s.memory_difficulty,
                s.completion_rate,
                s.note.as_deref().unwrap_or(""),
                s.is_manual_edit as i32,
            )?;
        }
        Ok(path)
    }

    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("prediction_states.csv");
        let mut file = std::fs::File::create(&path)?;
        use std::io::Write;
        writeln!(file, "id,category_id,algorithm,last_review,next_review")?;
        for p in predictions {
            writeln!(
                file,
                "{},{},{},{},{}",
                p.id, p.category_id, p.algorithm, p.last_review.to_rfc3339(), p.next_review.to_rfc3339()
            )?;
        }
        Ok(path)
    }
}

impl Exporter for MarkdownExporter {
    fn export_sessions(&self, sessions: &[Session], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("sessions.md");
        let mut file = std::fs::File::create(&path)?;
        use std::io::Write;
        writeln!(file, "# Sessions")?;
        writeln!(file)?;
        writeln!(file, "| ID | Category | Start | End | Duration | Quality | Note |")?;
        writeln!(file, "|----|----------|-------|-----|----------|---------|------|")?;
        for s in sessions {
            let dur = format_duration(s.duration_secs);
            writeln!(
                file,
                "| {} | {} | {} | {} | {} | {} | {} |",
                s.id,
                s.category_id,
                s.start_time.format("%Y-%m-%d %H:%M"),
                s.end_time.format("%Y-%m-%d %H:%M"),
                dur,
                s.quality,
                s.note.as_deref().unwrap_or("-"),
            )?;
        }
        Ok(path)
    }

    fn export_predictions(&self, predictions: &[PredictionState], output_dir: &Path) -> Result<std::path::PathBuf> {
        std::fs::create_dir_all(output_dir)?;
        let path = output_dir.join("prediction_states.md");
        let mut file = std::fs::File::create(&path)?;
        use std::io::Write;
        writeln!(file, "# Prediction States")?;
        writeln!(file)?;
        writeln!(file, "| Category | Algorithm | Last Review | Next Review |")?;
        writeln!(file, "|----------|-----------|-------------|-------------|")?;
        for p in predictions {
            writeln!(
                file,
                "| {} | {} | {} | {} |",
                p.category_id,
                p.algorithm,
                p.last_review.format("%Y-%m-%d"),
                p.next_review.format("%Y-%m-%d"),
            )?;
        }
        Ok(path)
    }
}

fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/data/export.rs
git commit -m "feat: implement JSONL/CSV/Markdown exporters"
```

---

### Task 9: 更新 main.rs 集成数据层

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 更新 main.rs**

```rust
mod data;

fn main() {
    match data::Database::open_default() {
        Ok(db) => {
            println!("Database opened at: {}", db.db_path().display());
        }
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate data layer into main"
```

---

### Task 10: 数据层单元测试

**Files:**
- Create: `tests/data_tests.rs`

- [ ] **Step 1: 编写测试**

```rust
use chrono::Utc;
use std::path::PathBuf;
use time_manager::data::{
    models::{CategoryInsert, Difficulty, PredictionStateInsert, Quality, SessionInsert, SessionParams},
    Database,
};

fn test_db() -> Database {
    let dir = std::env::temp_dir().join("time-manager-test");
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("test.db");
    Database::open(&path).expect("open test db")
}

#[test]
fn test_open_and_create_tables() {
    let db = test_db();
    assert!(db.db_path().exists());
}

#[test]
fn test_category_crud() {
    let db = test_db();
    let id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "语言".into(),
        path: "语言".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert");
    let cat = db.get_category(id).expect("get");
    assert_eq!(cat.name, "语言");
    assert_eq!(cat.path, "语言");

    let child_id = db.insert_category(&CategoryInsert {
        parent_id: Some(id),
        name: "英语".into(),
        path: "语言/英语".into(),
        source: None,
        default_quality: Some(Quality::High),
        default_understanding_difficulty: Some(Difficulty::Medium),
        default_memory_difficulty: Some(Difficulty::Medium),
    }).expect("insert child");

    let children = db.get_children(Some(id)).expect("children");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id, child_id);

    db.update_category_name(child_id, "日语").expect("update");
    let updated = db.get_category(child_id).expect("get");
    assert_eq!(updated.name, "日语");

    db.delete_category(child_id).expect("delete");
    let children = db.get_children(Some(id)).expect("children");
    assert!(children.is_empty());
}

#[test]
fn test_session_crud() {
    let db = test_db();
    let cat_id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "数学".into(),
        path: "数学".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert cat");

    let now = Utc::now();
    let session_id = db.insert_session(&SessionInsert {
        category_id: cat_id,
        start_time: now,
        end_time: now + chrono::Duration::seconds(3600),
        duration_secs: 3600,
        pause_records: vec![],
        params: SessionParams::default(),
        note: Some("test note".into()),
    }).expect("insert session");

    let session = db.get_session(session_id).expect("get");
    assert_eq!(session.category_id, cat_id);
    assert_eq!(session.duration_secs, 3600);
    assert_eq!(session.note.as_deref(), Some("test note"));

    let sessions = db.get_sessions_by_category(cat_id).expect("list");
    assert_eq!(sessions.len(), 1);
}

#[test]
fn test_prediction_state_crud() {
    let db = test_db();
    let cat_id = db.insert_category(&CategoryInsert {
        parent_id: None,
        name: "Rust".into(),
        path: "Rust".into(),
        source: None,
        default_quality: None,
        default_understanding_difficulty: None,
        default_memory_difficulty: None,
    }).expect("insert cat");

    let now = Utc::now();
    db.upsert_prediction_state(&PredictionStateInsert {
        category_id: cat_id,
        algorithm: "fsrs".into(),
        last_review: now,
        next_review: now + chrono::Duration::days(3),
        algorithm_state: vec![1, 2, 3],
    }).expect("upsert");

    let ps = db.get_prediction_state(cat_id).expect("get");
    assert!(ps.is_some());
    assert_eq!(ps.unwrap().algorithm, "fsrs");

    let due = db.get_due_predictions(&(now + chrono::Duration::days(4))).expect("due");
    assert_eq!(due.len(), 1);
}

#[test]
fn test_settings_crud() {
    let db = test_db();
    assert!(db.get_setting("test_key").expect("get missing").is_none());
    db.set_setting("test_key", "test_value").expect("set");
    assert_eq!(db.get_setting("test_key").expect("get").unwrap(), "test_value");
    db.set_setting("test_key", "new_value").expect("update");
    assert_eq!(db.get_setting("test_key").expect("get").unwrap(), "new_value");
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test`
Expected: All 5 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/
git commit -m "test: add data layer unit tests"
```

---

### Task 11: 导出测试

**Files:**
- Modify: `tests/data_tests.rs`

- [ ] **Step 1: 添加导出测试**

在 `tests/data_tests.rs` 末尾追加：

```rust
use time_manager::data::export::{CsvExporter, JsonExporter, MarkdownExporter, Exporter};
use time_manager::data::models::{PauseRecord, PredictionState, Session};

#[test]
fn test_json_export() {
    let sessions = vec![Session {
        id: 1,
        category_id: 1,
        start_time: Utc::now(),
        end_time: Utc::now() + chrono::Duration::seconds(1800),
        duration_secs: 1800,
        pause_records: vec![],
        quality: Quality::High,
        understanding_difficulty: Difficulty::Medium,
        memory_difficulty: Difficulty::Medium,
        completion_rate: 100,
        note: None,
        is_manual_edit: false,
    }];
    let dir = std::env::temp_dir().join("tm-export-test-json");
    let _ = std::fs::remove_dir_all(&dir);
    let exporter = JsonExporter;
    let path = exporter.export_sessions(&sessions, &dir).expect("export");
    assert!(path.exists());
    let content = std::fs::read_to_string(path).expect("read");
    assert!(content.contains("\"duration_secs\":1800"));
}

#[test]
fn test_csv_export() {
    let sessions = vec![Session {
        id: 1,
        category_id: 1,
        start_time: Utc::now(),
        end_time: Utc::now(),
        duration_secs: 600,
        pause_records: vec![],
        quality: Quality::Medium,
        understanding_difficulty: Difficulty::Easy,
        memory_difficulty: Difficulty::Easy,
        completion_rate: 80,
        note: Some("csv test".into()),
        is_manual_edit: false,
    }];
    let dir = std::env::temp_dir().join("tm-export-test-csv");
    let _ = std::fs::remove_dir_all(&dir);
    let exporter = CsvExporter;
    let path = exporter.export_sessions(&sessions, &dir).expect("export");
    assert!(path.exists());
    let content = std::fs::read_to_string(path).expect("read");
    assert!(content.starts_with("id,category_id"));
}

#[test]
fn test_markdown_export() {
    let predictions = vec![PredictionState {
        id: 1,
        category_id: 1,
        algorithm: "fsrs".into(),
        last_review: Utc::now(),
        next_review: Utc::now() + chrono::Duration::days(1),
        algorithm_state: vec![],
    }];
    let dir = std::env::temp_dir().join("tm-export-test-md");
    let _ = std::fs::remove_dir_all(&dir);
    let exporter = MarkdownExporter;
    let path = exporter.export_predictions(&predictions, &dir).expect("export");
    assert!(path.exists());
    let content = std::fs::read_to_string(path).expect("read");
    assert!(content.contains("# Prediction States"));
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test`
Expected: All 8 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/data_tests.rs
git commit -m "test: add export tests"
```
