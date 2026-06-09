use crate::data::models::PauseRecord;
use chrono::{DateTime, Utc};
use tracing::{debug, trace, warn};

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
        trace!("Creating new TimerStateMachine");
        Self {
            state: TimerState::Idle,
            category_id: None,
        }
    }

    #[tracing::instrument(level = "info", name = "timer_start", skip(self), fields(category_id))]
    pub fn start(&mut self, category_id: i64) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Idle => {
                debug!("Starting timer for category {}", category_id);
                self.state = TimerState::Running {
                    start_time: Utc::now(),
                    pause_total_secs: 0,
                    pauses: Vec::new(),
                };
                self.category_id = Some(category_id);
                tracing::Span::current().record("category_id", category_id);
                debug!("Timer state transition: Idle -> Running");
                Ok(())
            }
            _ => {
                warn!("Cannot start timer: not idle (current state: {:?})", std::mem::discriminant(&self.state));
                Err("cannot start: timer is not idle")
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn pause(&mut self) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Running { start_time, pause_total_secs, pauses } => {
                let now = Utc::now();
                debug!("Pausing timer at {}, total pause so far: {}s", now.format("%H:%M:%S"), pause_total_secs);

                self.state = TimerState::Paused {
                    start_time: *start_time,
                    pause_total_secs: *pause_total_secs,
                    pauses: pauses.clone(),
                    pause_start: now,
                };
                debug!("Timer state transition: Running -> Paused");
                Ok(())
            }
            _ => {
                warn!("Cannot pause timer: not running");
                Err("cannot pause: timer is not running")
            }
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
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
                    debug!("Resuming timer after {}s pause", pause_secs);

                    pauses.push(PauseRecord {
                        pause_start,
                        resume_time: now,
                    });
                    trace!("Pause record added: {} pauses total", pauses.len());

                    self.state = TimerState::Running {
                        start_time,
                        pause_total_secs: pause_total_secs + pause_secs,
                        pauses,
                    };
                    debug!("Timer state transition: Paused -> Running (total pause: {}s)", pause_total_secs + pause_secs);
                }
                Ok(())
            }
            _ => {
                warn!("Cannot resume timer: not paused");
                Err("cannot resume: timer is not paused")
            }
        }
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub fn stop(&mut self) -> Result<StoppedSession, &'static str> {
        debug!("Stopping timer");

        let now = Utc::now();
        let (start_time, pause_total_secs, pauses, _extra_pause) = match &self.state {
            TimerState::Running {
                start_time,
                pause_total_secs,
                pauses,
            } => {
                trace!("Stopping from Running state");
                (*start_time, *pause_total_secs, pauses.clone(), 0i64)
            }
            TimerState::Paused {
                start_time,
                pause_total_secs,
                pauses,
                pause_start,
            } => {
                trace!("Stopping from Paused state");
                let extra = (now - *pause_start).num_seconds().max(0);
                let mut p = pauses.clone();
                p.push(PauseRecord {
                    pause_start: *pause_start,
                    resume_time: now,
                });
                (*start_time, *pause_total_secs + extra, p, 0)
            }
            TimerState::Idle => {
                warn!("Cannot stop timer: idle");
                return Err("cannot stop: timer is idle");
            }
        };

        let cat_id = self.category_id.unwrap_or(0);
        self.state = TimerState::Idle;
        self.category_id = None;

        let elapsed = (now - start_time).num_seconds().max(0);
        let effective = (elapsed - pause_total_secs).max(0);

        debug!(
            "Timer stopped: category={}, elapsed={}s, paused={}s, effective={}s",
            cat_id, elapsed, pause_total_secs, effective
        );

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
