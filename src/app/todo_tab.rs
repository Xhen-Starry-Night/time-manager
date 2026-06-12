#[derive(Default)]
pub struct TodoTabState {
    pub new_todo_input: String,
    pub selected_todo: Option<uuid::Uuid>,
}