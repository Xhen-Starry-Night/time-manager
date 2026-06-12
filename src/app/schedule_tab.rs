pub struct ScheduleTabState {
    pub selected_schedule: Option<uuid::Uuid>,
    pub schedules: Vec<(uuid::Uuid, String)>,
}

impl Default for ScheduleTabState {
    fn default() -> Self {
        Self {
            selected_schedule: None,
            schedules: Vec::new(),
        }
    }
}