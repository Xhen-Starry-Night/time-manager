use chrono::Utc;
use crate::data::models::Session;
use crate::modules::learning::prediction::algorithm::{PredictionAlgorithm, PredictionResult};
use crate::modules::learning::prediction::fsrs::mapper::quality_to_rating;
use crate::modules::learning::prediction::fsrs::state::FsrsCardState;
use std::sync::Mutex;

pub struct FsrsAdapter {
    fsrs: Mutex<fsrs::FSRS>,
    desired_retention: f32,
}

impl FsrsAdapter {
    pub fn new() -> Self {
        let fsrs = fsrs::FSRS::new(Some(&fsrs::DEFAULT_PARAMETERS)).expect("FSRS init");
        Self {
            fsrs: Mutex::new(fsrs),
            desired_retention: 0.9,
        }
    }

    pub fn with_retention(mut self, retention: f32) -> Self {
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
    fn predict_next_review(
        &self,
        _session_history: &[Session],
        algorithm_state: &[u8],
        quality: &crate::data::models::Quality,
    ) -> PredictionResult {
        let rating = quality_to_rating(quality);
        let current_state: Option<fsrs::MemoryState> = FsrsCardState::from_bytes(algorithm_state)
            .map(|s| fsrs::MemoryState {
                stability: s.stability,
                difficulty: s.difficulty,
            });

        let days_elapsed = current_state.as_ref().map_or(0u32, |_| 1);

        let fsrs = self.fsrs.lock().expect("fsrs lock");
        let next_states = fsrs
            .next_states(current_state, self.desired_retention, days_elapsed)
            .expect("FSRS next_states");

        let item_state = match rating {
            1 => &next_states.again,
            2 => &next_states.hard,
            3 => &next_states.good,
            4 => &next_states.easy,
            _ => &next_states.good,
        };

        let interval_days = item_state.interval;
        let memory = &item_state.memory;

        let new_state = FsrsCardState {
            difficulty: memory.difficulty,
            stability: memory.stability,
            last_date: 0.0,
            due: interval_days,
        };

        let interval_secs = (interval_days * 86400.0).round() as i64;
        let next_review = Utc::now() + chrono::Duration::seconds(interval_secs);

        PredictionResult {
            next_review,
            state_bytes: new_state.to_bytes(),
        }
    }

    fn algorithm_name(&self) -> &'static str {
        "fsrs"
    }
}