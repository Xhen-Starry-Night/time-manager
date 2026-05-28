use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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