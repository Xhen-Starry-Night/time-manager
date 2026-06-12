use crate::timer::TimerState;

pub struct TimerTabState {
    pub state: TimerState,
    pub elapsed_ms: i64,
    pub card_path_input: String,
}

impl TimerTabState {
    pub fn new(state: TimerState) -> Self {
        Self {
            state,
            elapsed_ms: 0,
            card_path_input: String::new(),
        }
    }
}

impl Default for TimerTabState {
    fn default() -> Self {
        Self::new(TimerState::default())
    }
}