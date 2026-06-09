use chrono::Utc;
use crate::data::models::Session;
use crate::modules::learning::prediction::algorithm::{PredictionAlgorithm, PredictionResult};
use crate::modules::learning::prediction::fsrs::mapper::quality_to_rating;
use crate::modules::learning::prediction::fsrs::state::FsrsCardState;
use std::sync::Mutex;
use tracing::{debug, trace, warn};

pub struct FsrsAdapter {
    fsrs: Mutex<fsrs::FSRS>,
    desired_retention: f32,
}

impl FsrsAdapter {
    pub fn new() -> Self {
        trace!("Initializing FSRS adapter with default parameters");
        let fsrs = fsrs::FSRS::new(Some(&fsrs::DEFAULT_PARAMETERS)).expect("FSRS init");
        debug!("FSRS adapter created with desired_retention=0.9");
        Self {
            fsrs: Mutex::new(fsrs),
            desired_retention: 0.9,
        }
    }

    pub fn with_retention(mut self, retention: f32) -> Self {
        debug!("Setting desired retention to {}", retention);
        self.desired_retention = retention;
        self
    }
}

impl Default for FsrsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictionAlgorithm for FsrsAdapter {
    #[tracing::instrument(level = "debug", skip(self, _session_history, algorithm_state), fields(quality, algorithm = "fsrs"))]
    fn predict_next_review(
        &self,
        _session_history: &[Session],
        algorithm_state: &[u8],
        quality: &crate::data::models::Quality,
    ) -> PredictionResult {
        let rating = quality_to_rating(quality);
        tracing::Span::current().record("quality", quality.as_str());
        debug!("Predicting next review: quality={}, rating={}", quality.as_str(), rating);

        let current_state: Option<fsrs::MemoryState> = FsrsCardState::from_bytes(algorithm_state)
            .map(|s| {
                trace!("Loaded FSRS state: stability={}, difficulty={}", s.stability, s.difficulty);
                fsrs::MemoryState {
                    stability: s.stability,
                    difficulty: s.difficulty,
                }
            });

        if current_state.is_none() {
            trace!("No existing FSRS state, starting fresh");
        }

        let days_elapsed = current_state.as_ref().map_or(0u32, |_| 1);
        trace!("Days elapsed: {}", days_elapsed);

        let fsrs = self.fsrs.lock().expect("fsrs lock");
        let next_states = fsrs
            .next_states(current_state, self.desired_retention, days_elapsed)
            .expect("FSRS next_states");

        let item_state = match rating {
            1 => {
                trace!("Selected 'again' state");
                &next_states.again
            }
            2 => {
                trace!("Selected 'hard' state");
                &next_states.hard
            }
            3 => {
                trace!("Selected 'good' state");
                &next_states.good
            }
            4 => {
                trace!("Selected 'easy' state");
                &next_states.easy
            }
            _ => {
                warn!("Invalid rating {}, defaulting to 'good'", rating);
                &next_states.good
            }
        };

        let interval_days = item_state.interval;
        let memory = &item_state.memory;

        debug!(
            "FSRS calculation: interval={} days, stability={}, difficulty={}",
            interval_days, memory.stability, memory.difficulty
        );

        let new_state = FsrsCardState {
            difficulty: memory.difficulty,
            stability: memory.stability,
            last_date: 0.0,
            due: interval_days,
        };

        let interval_secs = (interval_days * 86400.0).round() as i64;
        let next_review = Utc::now() + chrono::Duration::seconds(interval_secs);

        debug!(
            "Next review scheduled: {} ({} days from now)",
            next_review.format("%Y-%m-%d"),
            interval_days
        );

        PredictionResult {
            next_review,
            state_bytes: new_state.to_bytes(),
        }
    }

    fn algorithm_name(&self) -> &'static str {
        "fsrs"
    }
}
