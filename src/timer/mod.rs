use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimerState {
    Idle,
    Running {
        started_at: DateTime<Utc>,
    },
    Paused {
        started_at: DateTime<Utc>,
        paused_at: DateTime<Utc>,
        accumulated: i64,
    },
    Stopped {
        started_at: DateTime<Utc>,
        stopped_at: DateTime<Utc>,
        duration_ms: i64,
    },
}

impl Default for TimerState {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerState {
    pub fn new() -> Self {
        Self::Idle
    }

    pub fn start(&mut self) -> Result<(), String> {
        match self {
            Self::Idle => {
                *self = Self::Running {
                    started_at: Utc::now(),
                };
                Ok(())
            }
            Self::Paused {
                started_at,
                paused_at: _,
                accumulated: _,
            } => {
                *self = Self::Running {
                    started_at: *started_at,
                };
                Ok(())
            }
            _ => Err("Timer already running".to_string()),
        }
    }

    pub fn pause(&mut self) -> Result<(), String> {
        match self {
            Self::Running { started_at } => {
                let paused_at = Utc::now();
                let elapsed = (paused_at - *started_at).num_milliseconds();
                *self = Self::Paused {
                    started_at: *started_at,
                    paused_at,
                    accumulated: elapsed,
                };
                Ok(())
            }
            Self::Paused { .. } => Err("Timer already paused".to_string()),
            _ => Err("Timer not running".to_string()),
        }
    }

    pub fn stop(&mut self) -> Result<(DateTime<Utc>, i64), String> {
        match self {
            Self::Running { started_at } => {
                let stopped_at = Utc::now();
                let duration_ms = (stopped_at - *started_at).num_milliseconds();
                *self = Self::Stopped {
                    started_at: *started_at,
                    stopped_at,
                    duration_ms,
                };
                Ok((stopped_at, duration_ms))
            }
            Self::Paused {
                started_at,
                paused_at: _,
                accumulated,
            } => {
                let stopped_at = Utc::now();
                let total_ms = *accumulated;
                *self = Self::Stopped {
                    started_at: *started_at,
                    stopped_at,
                    duration_ms: total_ms,
                };
                Ok((stopped_at, total_ms))
            }
            _ => Err("Timer not running".to_string()),
        }
    }

    pub fn elapsed_ms(&self) -> i64 {
        match self {
            Self::Running { started_at } => (Utc::now() - *started_at).num_milliseconds(),
            Self::Paused { accumulated, .. } => *accumulated,
            Self::Stopped { duration_ms, .. } => *duration_ms,
            Self::Idle => 0,
        }
    }

    pub fn elapsed_string(&self) -> String {
        let ms = self.elapsed_ms();
        let seconds = ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;

        format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
    }
}

pub struct TimerManager {
    state_file: PathBuf,
    state: TimerState,
}

impl TimerManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let state_file = data_dir.join(".timer_state.json");
        let state = if state_file.exists() {
            let json = fs::read_to_string(&state_file).unwrap_or_default();
            serde_json::from_str(&json).unwrap_or(TimerState::Idle)
        } else {
            TimerState::Idle
        };

        Self { state_file, state }
    }

    pub fn start(&mut self) -> Result<(), String> {
        self.state.start()?;
        self.save_state()?;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.state.pause()?;
        self.save_state()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<PathBuf, String> {
        let (stopped_at, duration_ms) = self.state.stop()?;
        self.save_state()?;

        let started_at = match &self.state {
            TimerState::Stopped { started_at, .. } => *started_at,
            _ => return Err("Invalid state after stop".to_string()),
        };

        self.save_timer_file(started_at, stopped_at, duration_ms)
    }

    pub fn get_state(&self) -> &TimerState {
        &self.state
    }

    fn save_state(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.state)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        fs::write(&self.state_file, json).map_err(|e| format!("Failed to write state: {}", e))?;
        Ok(())
    }

    fn save_timer_file(
        &self,
        started_at: DateTime<Utc>,
        stopped_at: DateTime<Utc>,
        duration_ms: i64,
    ) -> Result<PathBuf, String> {
        let data_dir = self
            .state_file
            .parent()
            .ok_or("Invalid state file path".to_string())?;

        let timers_dir = data_dir.join("timers");
        fs::create_dir_all(&timers_dir)
            .map_err(|e| format!("Failed to create timers dir: {}", e))?;

        let filename = started_at.format("%Y-%m-%dT%H-%M-%S").to_string();
        let timer_file = timers_dir.join(format!("{}.json", filename));

        let timer_data = serde_json::json!({
            "started_at": started_at.to_rfc3339(),
            "stopped_at": stopped_at.to_rfc3339(),
            "duration_ms": duration_ms,
        });

        let json = serde_json::to_string_pretty(&timer_data)
            .map_err(|e| format!("Failed to serialize timer: {}", e))?;

        fs::write(&timer_file, json).map_err(|e| format!("Failed to write timer: {}", e))?;

        Ok(timer_file)
    }

    pub fn clear_state(&mut self) -> Result<(), String> {
        self.state = TimerState::Idle;
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)
                .map_err(|e| format!("Failed to remove state: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_timer_state_transitions() {
        let mut state = TimerState::new();
        assert_eq!(state, TimerState::Idle);

        state.start().unwrap();
        assert!(matches!(state, TimerState::Running { .. }));

        std::thread::sleep(std::time::Duration::from_millis(10));

        state.pause().unwrap();
        assert!(matches!(state, TimerState::Paused { .. }));

        state.start().unwrap();
        assert!(matches!(state, TimerState::Running { .. }));

        let (_stopped_at, duration) = state.stop().unwrap();
        assert!(matches!(state, TimerState::Stopped { .. }));
        assert!(duration >= 0);
    }

    #[test]
    fn test_timer_elapsed_time() {
        let mut state = TimerState::new();
        assert_eq!(state.elapsed_ms(), 0);

        state.start().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(state.elapsed_ms() > 0);

        state.pause().unwrap();
        let paused_elapsed = state.elapsed_ms();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert_eq!(state.elapsed_ms(), paused_elapsed);
    }

    #[test]
    fn test_timer_elapsed_string() {
        let mut state = TimerState::new();
        assert_eq!(state.elapsed_string(), "00:00:00");

        state.start().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let elapsed = state.elapsed_string();
        assert!(elapsed.starts_with("00:00:"));
    }

    #[test]
    fn test_timer_manager() {
        let dir = tempdir().unwrap();
        let mut manager = TimerManager::new(dir.path().to_path_buf());

        manager.start().unwrap();
        assert!(matches!(manager.get_state(), TimerState::Running { .. }));

        std::thread::sleep(std::time::Duration::from_millis(100));
        manager.pause().unwrap();
        assert!(matches!(manager.get_state(), TimerState::Paused { .. }));

        manager.start().unwrap();
        let timer_file = manager.stop().unwrap();
        assert!(timer_file.exists());

        manager.clear_state().unwrap();
        assert!(matches!(manager.get_state(), TimerState::Idle));
    }

    #[test]
    fn test_timer_errors() {
        let mut state = TimerState::new();

        assert!(state.pause().is_err());
        assert!(state.stop().is_err());

        state.start().unwrap();
        assert!(state.start().is_err());

        state.pause().unwrap();
        assert!(state.pause().is_err());
    }
}
