use std::collections::HashSet;
use chrono::{DateTime, Utc};
use crate::gui::components::TreeView;
use crate::gui::components::tree_view::TreeNode;
use crate::gui::NodeType;
use crate::data::models::{Card, ReviewRecord, MemoryQuality, Prediction};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub is_card: bool,
}

#[derive(Debug, Clone)]
pub struct NewNodeForm {
    pub parent_path: String,
    pub node_type: NodeType,
    pub name: String,
    pub preset: String,
}

impl NewNodeForm {
    pub fn new(parent_path: String, default_type: NodeType, default_preset: &str) -> Self {
        Self {
            parent_path,
            node_type: default_type,
            name: String::new(),
            preset: default_preset.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditCardForm {
    pub path: String,
    pub name: String,
    pub original_name: String,
    pub new_name: String,
    pub preset: String,
    pub next_review: Option<chrono::DateTime<chrono::Utc>>,
    pub clear_prediction: bool,
    pub review_records: Vec<ReviewRecord>,
    pub new_review_duration: String,
    pub new_review_quality: MemoryQuality,
    pub expanded_groups: HashSet<String>,
    pub fsrs_state_bytes: Vec<u8>,
    pub prediction: Option<Prediction>,
}

impl EditCardForm {
    pub fn new(path: String, card: &Card) -> Self {
        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
        let preset = card.prediction.as_ref()
            .map(|p| p.preset_used.clone())
            .unwrap_or_else(|| "default".to_string());
        
        let mut expanded_groups = HashSet::new();
        expanded_groups.insert("basic".to_string());
        expanded_groups.insert("prediction".to_string());
        
        Self {
            path: path.clone(),
            name: name.clone(),
            original_name: name.clone(),
            new_name: name,
            preset,
            next_review: card.prediction.as_ref().map(|p| p.next_review),
            clear_prediction: false,
            review_records: card.review_records.clone(),
            expanded_groups,
            new_review_duration: String::new(),
            new_review_quality: MemoryQuality::Good,
            fsrs_state_bytes: card.prediction.as_ref()
                .map(|p| p.fsrs_state_bytes.clone())
                .unwrap_or_default(),
            prediction: card.prediction.clone(),
        }
    }
}

#[derive(Default)]
pub struct CategoryTabState {
    pub tree_name: String,
    pub selected_path: Option<String>,
    pub search_query: String,
    pub tree_nodes: Vec<TreeNode>,
    pub tree_view: TreeView,
    pub search_results: Vec<SearchResult>,
    pub search_selected_index: Option<usize>,
    pub search_focused: bool,
    
    pub new_tree_name: String,
    pub new_tree_import_path: String,
    pub new_tree_import_rules_path: String,
    pub new_node_form: Option<NewNodeForm>,
    pub edit_card_form: Option<EditCardForm>,
    pub delete_target: Option<(String, bool)>,
}