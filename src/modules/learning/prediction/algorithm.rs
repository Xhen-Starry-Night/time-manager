use chrono::{DateTime, Utc};
use crate::data::models::{PredictionStateInsert, Session};

pub trait PredictionAlgorithm: Send + Sync {
    fn predict_next_review(
        &self,
        session_history: &[Session],
        algorithm_state: &[u8],
        quality: &crate::data::models::Quality,
    ) -> PredictionResult;

    fn algorithm_name(&self) -> &'static str;
}

pub struct PredictionResult {
    pub next_review: DateTime<Utc>,
    pub state_bytes: Vec<u8>,
}

pub fn update_prediction(
    algo: &dyn PredictionAlgorithm,
    db: &crate::data::Database,
    category_id: i64,
    sessions: &[Session],
    current_state: Option<&[u8]>,
    quality: &crate::data::models::Quality,
) -> crate::data::error::Result<()> {
    let state = current_state.unwrap_or(&[]);
    let result = algo.predict_next_review(sessions, state, quality);
    let now = Utc::now();
    db.upsert_prediction_state(&PredictionStateInsert {
        category_id,
        algorithm: algo.algorithm_name().into(),
        last_review: now,
        next_review: result.next_review,
        algorithm_state: result.state_bytes,
    })
}