use crate::data::models::PauseRecord;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Idle,
    Running {
        start_time: DateTime<Utc>,
        pause_total_secs: i64,
        pauses: Vec<PauseRecord>,
    },
    Paused {
        start_time: DateTime<Utc>,
        pause_total_secs: i64,
        pauses: Vec<PauseRecord>,
        pause_start: DateTime<Utc>,
    },
}

#[derive(Debug, Clone)]
pub struct TimerStateMachine {
    pub state: TimerState,
    pub category_id: Option<i64>,
}

pub struct StoppedSession {
    pub category_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: i64,
    pub pause_records: Vec<PauseRecord>,
}

impl TimerStateMachine {
    pub fn new() -> Self {
        Self {
            state: TimerState::Idle,
            category_id: None,
        }
    }

    pub fn start(&mut self, category_id: i64) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Idle => {
                self.state = TimerState::Running {
                    start_time: Utc::now(),
                    pause_total_secs: 0,
                    pauses: Vec::new(),
                };
                self.category_id = Some(category_id);
                Ok(())
            }
            _ => Err("cannot start: timer is not idle"),
        }
    }

    pub fn pause(&mut self) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Running { .. } => {
                let now = Utc::now();
                if let TimerState::Running {
                    start_time,
                    pause_total_secs,
                    pauses,
                } = &self.state
                {
                    self.state = TimerState::Paused {
                        start_time: *start_time,
                        pause_total_secs: *pause_total_secs,
                        pauses: pauses.clone(),
                        pause_start: now,
                    };
                }
                Ok(())
            }
            _ => Err("cannot pause: timer is not running"),
        }
    }

    pub fn resume(&mut self) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Paused { .. } => {
                let now = Utc::now();
                if let TimerState::Paused {
                    start_time,
                    pause_total_secs,
                    mut pauses,
                    pause_start,
                } = std::mem::replace(&mut self.state, TimerState::Idle)
                {
                    let pause_secs = (now - pause_start).num_seconds().max(0);
                    pauses.push(PauseRecord {
                        pause_start,
                        resume_time: now,
                    });
                    self.state = TimerState::Running {
                        start_time,
                        pause_total_secs: pause_total_secs + pause_secs,
                        pauses,
                    };
                }
                Ok(())
            }
            _ => Err("cannot resume: timer is not paused"),
        }
    }

    pub fn stop(&mut self) -> Result<StoppedSession, &'static str> {
        let now = Utc::now();
        let (start_time, pause_total_secs, pauses, _extra_pause) = match &self.state {
            TimerState::Running {
                start_time,
                pause_total_secs,
                pauses,
            } => (*start_time, *pause_total_secs, pauses.clone(), 0i64),
            TimerState::Paused {
                start_time,
                pause_total_secs,
                pauses,
                pause_start,
            } => {
                let extra = (now - *pause_start).num_seconds().max(0);
                let mut p = pauses.clone();
                p.push(PauseRecord {
                    pause_start: *pause_start,
                    resume_time: now,
                });
                (*start_time, *pause_total_secs + extra, p, 0)
            }
            TimerState::Idle => return Err("cannot stop: timer is idle"),
        };

        let cat_id = self.category_id.unwrap_or(0);
        self.state = TimerState::Idle;
        self.category_id = None;

        let elapsed = (now - start_time).num_seconds().max(0);
        let effective = (elapsed - pause_total_secs).max(0);

        Ok(StoppedSession {
            category_id: cat_id,
            start_time,
            end_time: now,
            duration_secs: effective,
            pause_records: pauses,
        })
    }

    pub fn effective_secs(&self) -> i64 {
        match &self.state {
            TimerState::Idle => 0,
            TimerState::Running {
                start_time,
                pause_total_secs,
                ..
            } => {
                let elapsed = (Utc::now() - *start_time).num_seconds().max(0);
                (elapsed - pause_total_secs).max(0)
            }
            TimerState::Paused {
                start_time,
                pause_total_secs,
                pause_start,
                ..
            } => {
                let elapsed = (Utc::now() - *start_time).num_seconds().max(0);
                let current_pause = (Utc::now() - *pause_start).num_seconds().max(0);
                (elapsed - pause_total_secs - current_pause).max(0)
            }
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(
            self.state,
            TimerState::Running { .. } | TimerState::Paused { .. }
        )
    }
}

impl Default for TimerStateMachine {
    fn default() -> Self {
        Self::new()
    }
}