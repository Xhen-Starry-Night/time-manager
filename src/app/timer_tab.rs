use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::models::MemoryQuality;
use crate::timer::TimerState;

pub struct TimerTabState {
    pub state: TimerState,
    pub elapsed_ms: i64,
    pub link_mode: bool,
    pub card_path_input: String,
    pub card_dropdown: Vec<String>,
    pub selected_card: Option<String>,
    pub memory_quality: MemoryQuality,
    pub show_history: bool,
    pub timer_history: Vec<TimerRecord>,
    pub current_card: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerRecord {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: i64,
    pub linked_card: Option<String>,
    pub memory_quality: Option<MemoryQuality>,
}

impl TimerTabState {
    pub fn new(state: TimerState) -> Self {
        Self {
            state,
            elapsed_ms: 0,
            link_mode: false,
            card_path_input: String::new(),
            card_dropdown: Vec::new(),
            selected_card: None,
            memory_quality: MemoryQuality::Good,
            show_history: false,
            timer_history: Vec::new(),
            current_card: None,
        }
    }
}

impl Default for TimerTabState {
    fn default() -> Self {
        Self::new(TimerState::default())
    }
}