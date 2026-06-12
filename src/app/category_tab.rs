use crate::gui::components::TreeView;
use crate::gui::components::tree_view::TreeNode;

#[derive(Default)]
pub struct CategoryTabState {
    pub tree_name: String,
    pub selected_path: Option<String>,
    pub search_query: String,
    pub tree_nodes: Vec<TreeNode>,
    pub tree_view: TreeView,
}