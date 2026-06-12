use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum MemoryQuality {
    Relearn,
    Hard,
    #[default]
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecord {
    pub timestamp: DateTime<Utc>,
    pub duration_ms: i64,
    pub memory_quality: MemoryQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub algorithm: String,
    pub next_review: DateTime<Utc>,
    pub fsrs_state_bytes: Vec<u8>,
    pub preset_used: String,
}

impl Default for Prediction {
    fn default() -> Self {
        Self {
            algorithm: "fsrs".to_string(),
            next_review: Utc::now(),
            fsrs_state_bytes: vec![],
            preset_used: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub review_records: Vec<ReviewRecord>,
    pub prediction: Option<Prediction>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            review_records: Vec::new(),
            prediction: None,
        }
    }

    pub fn new_with_preset(preset: String) -> Self {
        Self {
            review_records: Vec::new(),
            prediction: Some(Prediction {
                preset_used: preset,
                ..Default::default()
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub started_at: DateTime<Utc>,
    pub stopped_at: DateTime<Utc>,
    pub duration_ms: i64,
}

impl Timer {
    pub fn new(started_at: DateTime<Utc>, stopped_at: DateTime<Utc>, duration_ms: i64) -> Self {
        Self {
            started_at,
            stopped_at,
            duration_ms,
        }
    }

    pub fn filename(&self) -> String {
        self.started_at.format("%Y-%m-%dT%H-%M-%S").to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: Option<String>,
    pub match_rules: Vec<String>,
    pub fsrs_parameters: Option<Vec<f32>>,
    pub trained_at: Option<DateTime<Utc>>,
}

impl Preset {
    pub fn default_preset() -> Self {
        Self {
            name: "default".into(),
            description: Some("默认预设".into()),
            match_rules: Vec::new(),
            fsrs_parameters: None,
            trained_at: None,
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
