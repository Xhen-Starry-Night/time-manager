use crate::data::models::Schedule;

pub struct ScheduleTabState {
    pub schedules: Vec<Schedule>,
}

impl Default for ScheduleTabState {
    fn default() -> Self {
        Self {
            schedules: Vec::new(),
        }
    }
}
