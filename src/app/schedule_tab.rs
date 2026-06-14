use uuid::Uuid;

use crate::data::models::{RecurrenceRule, Schedule};

pub struct ScheduleForm {
    pub summary: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub description: String,
    pub location: String,
    pub reminder: String,
    pub rrule: RecurrenceRule,
}

impl Default for ScheduleForm {
    fn default() -> Self {
        Self {
            summary: String::new(),
            start_date: String::new(),
            start_time: String::new(),
            end_date: String::new(),
            end_time: String::new(),
            description: String::new(),
            location: String::new(),
            reminder: String::new(),
            rrule: RecurrenceRule::None,
        }
    }
}

impl ScheduleForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self) -> Result<Schedule, String> {
        if self.summary.trim().is_empty() {
            return Err("标题不能为空".to_string());
        }

        let start_str = format!("{}T{}Z", self.start_date.trim(), self.start_time.trim());
        let end_str = format!("{}T{}Z", self.end_date.trim(), self.end_time.trim());

        let dtstart = chrono::NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%dT%H:%MZ")
            .map_err(|e| format!("开始时间格式错误: {}", e))?;
        let dtend = chrono::NaiveDateTime::parse_from_str(&end_str, "%Y-%m-%dT%H:%MZ")
            .map_err(|e| format!("结束时间格式错误: {}", e))?;

        if dtend <= dtstart {
            return Err("结束时间必须晚于开始时间".to_string());
        }

        let reminder_minutes = if self.reminder.trim().is_empty() {
            None
        } else {
            Some(
                self.reminder
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| "提醒时间必须为数字".to_string())?,
            )
        };

        Ok(Schedule {
            id: Uuid::new_v4(),
            summary: self.summary.trim().to_string(),
            dtstart: chrono::DateTime::from_naive_utc_and_offset(dtstart, chrono::Utc),
            dtend: chrono::DateTime::from_naive_utc_and_offset(dtend, chrono::Utc),
            description: if self.description.trim().is_empty() {
                None
            } else {
                Some(self.description.trim().to_string())
            },
            location: if self.location.trim().is_empty() {
                None
            } else {
                Some(self.location.trim().to_string())
            },
            categories: Vec::new(),
            priority: None,
            rrule: self.rrule,
            reminder_minutes,
        })
    }

    pub fn load_from_schedule(&mut self, schedule: &Schedule) {
        self.summary = schedule.summary.clone();
        self.start_date = schedule.dtstart.format("%Y-%m-%d").to_string();
        self.start_time = schedule.dtstart.format("%H:%M").to_string();
        self.end_date = schedule.dtend.format("%Y-%m-%d").to_string();
        self.end_time = schedule.dtend.format("%H:%M").to_string();
        self.description = schedule.description.clone().unwrap_or_default();
        self.location = schedule.location.clone().unwrap_or_default();
        self.reminder = schedule
            .reminder_minutes
            .map(|m| m.to_string())
            .unwrap_or_default();
        self.rrule = schedule.rrule;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct ScheduleTabState {
    pub schedules: Vec<Schedule>,
    pub form: ScheduleForm,
    pub show_form: bool,
    pub editing_id: Option<Uuid>,
    pub form_error: Option<String>,
}

impl Default for ScheduleTabState {
    fn default() -> Self {
        Self {
            schedules: Vec::new(),
            form: ScheduleForm::new(),
            show_form: false,
            editing_id: None,
            form_error: None,
        }
    }
}
