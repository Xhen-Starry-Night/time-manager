use crate::data::models::Card;

#[derive(Default)]
pub struct ReviewTabState {
    pub urgency_filter: Option<u32>,
    pub search_query: String,
    pub selected_card: Option<String>,
    pub cards: Vec<(String, Card)>,
}