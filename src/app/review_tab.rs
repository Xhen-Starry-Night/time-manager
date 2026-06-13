use chrono::{DateTime, Utc};
use crate::data::models::Card;

#[derive(Default)]
pub struct ReviewTabState {
    pub urgency_filter: Option<u32>,
    pub search_query: String,
    pub selected_card: Option<String>,
    pub cards: Vec<(String, Card)>,
}

pub struct CardStats {
    pub sessions: usize,
    pub total_duration_ms: i64,
    pub avg_duration_ms: i64,
    pub last_review: Option<DateTime<Utc>>,
    pub next_review: Option<DateTime<Utc>>,
}

impl CardStats {
    pub fn from_card(card: &Card) -> Self {
        let sessions = card.review_records.len();
        let total_ms: i64 = card.review_records.iter()
            .map(|r| r.duration_ms)
            .sum();
        let avg_ms = if sessions > 0 { total_ms / sessions as i64 } else { 0 };
        let last_review = card.review_records.last()
            .map(|r| r.timestamp);
        let next_review = card.prediction.as_ref()
            .map(|p| p.next_review);
        
        Self {
            sessions,
            total_duration_ms: total_ms,
            avg_duration_ms: avg_ms,
            last_review,
            next_review,
        }
    }
}