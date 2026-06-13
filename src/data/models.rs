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
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Option<u32>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

 
 impl Todo {
    pub fn new(content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            completed: false,
            created_at: Utc::now(),
            due_date: None,
            priority: None,
            tags: Vec::new(),
            notes: None,
            completed_at: None,
        }
    }
    
    pub fn to_ics(&self) -> String {
        let status = if self.completed { "COMPLETED" } else { "NEEDS-ACTION" };
        let mut ics = format!(
            "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//time-manager-todo\nBEGIN:VTODO\nUID:{}\nDTSTART:{}\nSUMMARY:{}\nSTATUS:{}",
            self.id,
            self.created_at.format("%Y%m%dT%H%M%SZ"),
            self.content,
            status
        );
        
        if let Some(due) = self.due_date {
            ics.push_str(&format!("\nDUE:{}", due.format("%Y%m%dT%H%M%SZ")));
        }
        
        if let Some(p) = self.priority {
            ics.push_str(&format!("\nPRIORITY:{}", p));
        }
        
        if !self.tags.is_empty() {
            ics.push_str(&format!("\nCATEGORIES:{}", self.tags.join(",")));
        }
        
        if let Some(ref notes) = self.notes {
            ics.push_str(&format!("\nDESCRIPTION:{}", notes));
        }
        
        if let Some(completed) = self.completed_at {
            ics.push_str(&format!("\nCOMPLETED:{}", completed.format("%Y%m%dT%H%M%SZ")));
        }
        
        ics.push_str("\nEND:VTODO\nEND:VCALENDAR\n");
        ics
    }
    
    pub fn from_ics(ics_content: &str) -> Option<Self> {
        let lines: Vec<&str> = ics_content.lines().collect();
        
        let id = lines.iter()
            .find(|line| line.starts_with("UID:"))
            .and_then(|line| line.strip_prefix("UID:"))
            .and_then(|s| Uuid::parse_str(s).ok())?;
        
        let content = lines.iter()
            .find(|line| line.starts_with("SUMMARY:"))
            .and_then(|line| line.strip_prefix("SUMMARY:"))
            .unwrap_or("未知待办")
            .to_string();
        
        let completed = lines.iter()
            .find(|line| line.starts_with("STATUS:"))
            .and_then(|line| line.strip_prefix("STATUS:"))
            .map(|s| s == "COMPLETED")
            .unwrap_or(false);
        
        let created_at = lines.iter()
            .find(|line| line.starts_with("DTSTART:"))
            .and_then(|line| line.strip_prefix("DTSTART:"))
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or_else(Utc::now);
        
        let due_date = lines.iter()
            .find(|line| line.starts_with("DUE:"))
            .and_then(|line| line.strip_prefix("DUE:"))
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc));
        
        let priority = lines.iter()
            .find(|line| line.starts_with("PRIORITY:"))
            .and_then(|line| line.strip_prefix("PRIORITY:"))
            .and_then(|s| s.parse::<u32>().ok());
        
        let tags = lines.iter()
            .find(|line| line.starts_with("CATEGORIES:"))
            .and_then(|line| line.strip_prefix("CATEGORIES:"))
            .map(|s| s.split(",").map(String::from).collect())
            .unwrap_or_default();
        
        let notes = lines.iter()
            .find(|line| line.starts_with("DESCRIPTION:"))
            .and_then(|line| line.strip_prefix("DESCRIPTION:"))
            .map(String::from);
        
        let completed_at = lines.iter()
            .find(|line| line.starts_with("COMPLETED:"))
            .and_then(|line| line.strip_prefix("COMPLETED:"))
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
            .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc));
        
        Some(Self {
            id,
            content,
            completed,
            created_at,
            due_date,
            priority,
            tags,
            notes,
            completed_at,
        })
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
