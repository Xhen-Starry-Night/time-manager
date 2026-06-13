use chrono::{DateTime, Utc, TimeZone};
use fsrs::{DEFAULT_PARAMETERS, FSRS, MemoryState};

use crate::data::models::{MemoryQuality, ReviewRecord};

pub struct FsrsPredictor {
    engine: FSRS,
    parameters: Vec<f32>,
}

impl FsrsPredictor {
    pub fn new() -> Result<Self, String> {
        let engine = FSRS::new(Some(&DEFAULT_PARAMETERS))
            .map_err(|e| format!("Failed to initialize FSRS: {:?}", e))?;
        Ok(Self {
            engine,
            parameters: DEFAULT_PARAMETERS.to_vec(),
        })
    }

    pub fn with_parameters(parameters: Vec<f32>) -> Result<Self, String> {
        let engine = FSRS::new(Some(&parameters))
            .map_err(|e| format!("Failed to initialize FSRS: {:?}", e))?;
        Ok(Self { engine, parameters })
    }

    pub fn get_parameters(&self) -> &[f32] {
        &self.parameters
    }

    pub fn get_default_parameters() -> &'static [f32] {
        &DEFAULT_PARAMETERS
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

    pub fn predict_from_records(
        &self,
        records: &[ReviewRecord],
        current_memory_state: Option<MemoryState>,
        desired_retention: f32,
    ) -> Result<(DateTime<Utc>, MemoryState), String> {
        if records.is_empty() {
            let now = Utc::now();
            let next_states = self
                .engine
                .next_states(None, desired_retention, 0)
                .map_err(|e| format!("Failed to calculate initial states: {:?}", e))?;
            
            return Ok((now, next_states.good.memory));
        }

        let mut memory_state = current_memory_state;
        
        for i in 0..records.len() {
            let record = &records[i];
            let days_elapsed = if i == 0 {
                0
            } else {
                let prev_time = records[i - 1].timestamp;
                ((record.timestamp - prev_time).num_days().max(0) as u32)
            };

            let next_states = self
                .engine
                .next_states(memory_state, desired_retention, days_elapsed)
                .map_err(|e| format!("Failed to calculate next states: {:?}", e))?;

            memory_state = match record.memory_quality {
                MemoryQuality::Relearn => Some(next_states.again.memory),
                MemoryQuality::Hard => Some(next_states.hard.memory),
                MemoryQuality::Good => Some(next_states.good.memory),
                MemoryQuality::Easy => Some(next_states.easy.memory),
            };
        }

        let last_record = records.last().unwrap();
        let days_since_last = (Utc::now() - last_record.timestamp).num_days().max(0) as u32;
        
        let next_states = self
            .engine
            .next_states(memory_state, desired_retention, days_since_last)
            .map_err(|e| format!("Failed to calculate final states: {:?}", e))?;

        let interval = next_states.good.interval as i64;
        let next_review = Utc::now() + chrono::Duration::days(interval);

        Ok((next_review, next_states.good.memory))
    }

    pub fn calculate_urgency(next_review: DateTime<Utc>) -> i32 {
        let now = Utc::now();
        let hours_until = (next_review - now).num_hours();
        
        if hours_until < 0 {
            3
        } else if hours_until <= 12 {
            2
        } else if hours_until <= 72 {
            1
        } else {
            0
        }
    }

    pub fn memory_state_to_bytes(state: &MemoryState) -> Vec<u8> {
        [state.stability.to_le_bytes(),
            state.difficulty.to_le_bytes()]
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
    fn test_predictor_with_custom_parameters() {
        let params = FsrsPredictor::get_default_parameters().to_vec();
        let predictor = FsrsPredictor::with_parameters(params).unwrap();

        let (interval, _) = predictor
            .predict_next_review(None, MemoryQuality::Good, 0, 0.9)
            .unwrap();

        assert!(interval >= 0);
    }

    #[test]
    fn test_urgency_calculation() {
        let now = Utc::now();

        assert_eq!(
            FsrsPredictor::calculate_urgency(now - chrono::Duration::hours(1)),
            3
        );
        assert_eq!(FsrsPredictor::calculate_urgency(now + chrono::Duration::hours(12)), 2);
        assert_eq!(
            FsrsPredictor::calculate_urgency(now + chrono::Duration::hours(48)),
            1
        );
        assert_eq!(
            FsrsPredictor::calculate_urgency(now + chrono::Duration::hours(100)),
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
