use chrono::{DateTime, Utc};
use crate::data::models::{PredictionStateInsert, Session};
use tracing::{debug, info, trace};

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

#[tracing::instrument(level = "debug", skip(algo, db, sessions, current_state), fields(category_id, algorithm = %algo.algorithm_name()))]
pub fn update_prediction(
    algo: &dyn PredictionAlgorithm,
    db: &crate::data::Database,
    category_id: i64,
    sessions: &[Session],
    current_state: Option<&[u8]>,
    quality: &crate::data::models::Quality,
) -> crate::data::error::Result<()> {
    tracing::Span::current().record("category_id", category_id);

    debug!(
        "Updating prediction for category {}: {} sessions, quality={:?}",
        category_id,
        sessions.len(),
        quality
    );

    let state = current_state.unwrap_or(&[]);
    trace!("Algorithm state bytes: {} bytes", state.len());

    let result = algo.predict_next_review(sessions, state, quality);
    let now = Utc::now();

    debug!(
        "Prediction result: next_review={}, state_bytes={}",
        result.next_review.format("%Y-%m-%d %H:%M"),
        result.state_bytes.len()
    );

    db.upsert_prediction_state(&PredictionStateInsert {
        category_id,
        algorithm: algo.algorithm_name().into(),
        last_review: now,
        next_review: result.next_review,
        algorithm_state: result.state_bytes,
    })?;

    info!("Prediction updated for category {}", category_id);
    Ok(())
}
