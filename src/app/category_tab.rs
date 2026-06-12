#[derive(Default)]
pub struct CategoryTabState {
    pub tree_name: String,
    pub selected_path: Option<String>,
    pub search_query: String,
}