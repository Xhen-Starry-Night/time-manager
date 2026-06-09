use chrono::{DateTime, Utc};
use fsrs::{DEFAULT_PARAMETERS, FSRS, MemoryState};

use crate::data::models::MemoryQuality;

pub struct FsrsPredictor {
    engine: FSRS,
}

impl FsrsPredictor {
    pub fn new() -> Result<Self, String> {
        let engine = FSRS::new(Some(&DEFAULT_PARAMETERS))
            .map_err(|e| format!("Failed to initialize FSRS: {:?}", e))?;
        Ok(Self { engine })
    }

    pub fn predict_next_review(
        &self,
        memory_state: Option<MemoryState>,
        quality: MemoryQuality,
        days_elapsed: u32,
        desired_retention: f32,
    ) -> Result<(i32, MemoryState), String> {
        let next_states = self
            .engine
            .next_states(memory_state, desired_retention, days_elapsed)
            .map_err(|e| format!("Failed to calculate next states: {:?}", e))?;

        let (interval, new_state) = match quality {
            MemoryQuality::Relearn => (next_states.again.interval, next_states.again.memory),
            MemoryQuality::Hard => (next_states.hard.interval, next_states.hard.memory),
            MemoryQuality::Good => (next_states.good.interval, next_states.good.memory),
            MemoryQuality::Easy => (next_states.easy.interval, next_states.easy.memory),
        };

        Ok((interval as i32, new_state))
    }

    pub fn calculate_urgency(next_review: DateTime<Utc>) -> i32 {
        let now = Utc::now();
        let days_until = (next_review - now).num_days();

        match days_until {
            d if d < 0 => 3,
            d if d == 0 => 2,
            d if d <= 3 => 1,
            _ => 0,
        }
    }

    pub fn memory_state_to_bytes(state: &MemoryState) -> Vec<u8> {
        vec![
            state.stability.to_le_bytes(),
            state.difficulty.to_le_bytes(),
        ]
        .concat()
    }

    pub fn bytes_to_memory_state(bytes: &[u8]) -> Option<MemoryState> {
        if bytes.len() < 8 {
            return None;
        }

        let stability = f32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let difficulty = f32::from_le_bytes(bytes[4..8].try_into().ok()?);

        Some(MemoryState {
            stability,
            difficulty,
        })
    }
}

impl Default for FsrsPredictor {
    fn default() -> Self {
        Self::new().expect("Failed to create default FSRS predictor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsrs_predictor_creation() {
        let predictor = FsrsPredictor::new();
        assert!(predictor.is_ok());
    }

    #[test]
    fn test_predict_first_review() {
        let predictor = FsrsPredictor::new().unwrap();
        let (interval, state) = predictor
            .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
            .unwrap();

        assert!(interval > 0);
        assert!(state.stability > 0.0);
    }

    #[test]
    fn test_urgency_calculation() {
        let now = Utc::now();

        assert_eq!(
            FsrsPredictor::calculate_urgency(now - chrono::Duration::days(1)),
            3
        );
        assert_eq!(FsrsPredictor::calculate_urgency(now), 2);
        assert_eq!(
            FsrsPredictor::calculate_urgency(now + chrono::Duration::days(2)),
            1
        );
        assert_eq!(
            FsrsPredictor::calculate_urgency(now + chrono::Duration::days(7)),
            0
        );
    }

    #[test]
    fn test_memory_state_serialization() {
        let state = MemoryState {
            stability: 5.5,
            difficulty: 0.3,
        };

        let bytes = FsrsPredictor::memory_state_to_bytes(&state);
        let recovered = FsrsPredictor::bytes_to_memory_state(&bytes);

        assert!(recovered.is_some());
        let recovered = recovered.unwrap();
        assert!((recovered.stability - state.stability).abs() < 0.01);
        assert!((recovered.difficulty - state.difficulty).abs() < 0.01);
    }
}
