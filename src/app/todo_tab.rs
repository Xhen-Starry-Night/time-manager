use crate::data::models::Todo;

#[derive(Default)]
pub struct TodoTabState {
    pub form_content: String,
    pub form_priority: Option<u32>,
    pub form_due_year: String,
    pub form_due_month: String,
    pub form_due_day: String,
    pub form_due_hour: String,
    pub form_due_minute: String,
    pub form_tags: String,
    pub editing_todo_id: Option<uuid::Uuid>,
    pub selected_todo: Option<uuid::Uuid>,
    pub todos: Vec<Todo>,
}

impl TodoTabState {
    pub fn set_default_due_date(&mut self) {
        use chrono::Datelike;
        let now = chrono::Local::now();
        self.form_due_year = now.year().to_string();
        self.form_due_month = format!("{:02}", now.month());
        self.form_due_day = format!("{:02}", now.day());
        self.form_due_hour = "23".to_string();
        self.form_due_minute = "59".to_string();
    }
    
    pub fn load_todo_for_edit(&mut self, todo: &Todo) {
        use chrono::Datelike;
        use chrono::Timelike;
        
        self.editing_todo_id = Some(todo.id);
        self.form_content = todo.content.clone();
        self.form_priority = todo.priority;
        
        if let Some(due) = todo.due_date {
            self.form_due_year = due.year().to_string();
            self.form_due_month = format!("{:02}", due.month());
            self.form_due_day = format!("{:02}", due.day());
            self.form_due_hour = format!("{:02}", due.hour());
            self.form_due_minute = format!("{:02}", due.minute());
        } else {
            self.set_default_due_date();
        }
        
        self.form_tags = todo.tags.join(" ");
    }
}