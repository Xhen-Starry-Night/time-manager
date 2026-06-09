use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryQuality {
    Relearn,
    Hard,
    Good,
    Easy,
}

impl MemoryQuality {
    pub fn all() -> Vec<Self> {
        vec![Self::Relearn, Self::Hard, Self::Good, Self::Easy]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Relearn => "重学",
            Self::Hard => "困难",
            Self::Good => "好",
            Self::Easy => "简单",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "重学" => Some(Self::Relearn),
            "困难" => Some(Self::Hard),
            "好" => Some(Self::Good),
            "简单" => Some(Self::Easy),
            _ => None,
        }
    }
}

impl std::fmt::Display for MemoryQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for MemoryQuality {
    fn default() -> Self {
        Self::Good
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecord {
    pub timer_path: String,
    pub reviewed_at: DateTime<Utc>,
    pub memory_quality: MemoryQuality,
    pub state_bytes: Vec<u8>,
    pub fsrs_state_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub path: String,
    pub source: Option<String>,
    pub review_records: Vec<ReviewRecord>,
}

impl Card {
    pub fn new(path: String) -> Self {
        Self {
            path,
            source: None,
            review_records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub duration_ms: i64,
    pub pause_records: Vec<PauseRecord>,
}

impl Timer {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            started_at,
            stopped_at: None,
            duration_ms: 0,
            pause_records: Vec::new(),
        }
    }

    pub fn filename(&self) -> String {
        self.started_at.format("%Y-%m-%dT%H-%M-%S").to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PauseRecord {
    pub pause_start: DateTime<Utc>,
    pub resume_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: Option<String>,
    pub match_rules: Vec<String>,
}

impl Preset {
    pub fn default_preset() -> Self {
        Self {
            name: "default".into(),
            description: Some("默认预设".into()),
            match_rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Todo {
    pub fn new(content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_preset: String,
    pub default_algorithm: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_preset: "default".into(),
            default_algorithm: "fsrs".into(),
        }
    }
}
