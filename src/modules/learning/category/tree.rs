use crate::data::models::{Category, Difficulty, NodeType, Quality};
use tracing::{debug, trace};

#[derive(Debug, Clone)]
pub struct CategoryNode {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub node_type: NodeType,
    pub source: Option<String>,
    pub default_quality: Option<Quality>,
    pub default_understanding_difficulty: Option<Difficulty>,
    pub default_memory_difficulty: Option<Difficulty>,
    pub children: Vec<CategoryNode>,
}

impl CategoryNode {
    #[tracing::instrument(level = "trace", skip(cat))]
    pub fn from_category(cat: &Category) -> Self {
        trace!("Creating CategoryNode from category id={}", cat.id);
        Self {
            id: cat.id,
            name: cat.name.clone(),
            path: cat.path.clone(),
            node_type: cat.node_type.clone(),
            source: cat.source.clone(),
            default_quality: cat.default_quality.clone(),
            default_understanding_difficulty: cat.default_understanding_difficulty.clone(),
            default_memory_difficulty: cat.default_memory_difficulty.clone(),
            children: Vec::new(),
        }
    }

    pub fn find_mut(&mut self, id: i64) -> Option<&mut CategoryNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find(&self, id: i64) -> Option<&CategoryNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_by_path(&self, path: &str) -> Option<&CategoryNode> {
        if self.path == path {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_path(path) {
                return Some(found);
            }
        }
        None
    }

    pub fn collect_all(&self) -> Vec<&CategoryNode> {
        let mut result = vec![self];
        for child in &self.children {
            result.extend(child.collect_all());
        }
        result
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn is_learning(&self) -> bool {
        matches!(self.node_type, NodeType::Learning)
    }

    pub fn leaf_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        if self.is_leaf() {
            result.push(self);
        } else {
            for child in &self.children {
                result.extend(child.leaf_nodes());
            }
        }
        result
    }

    pub fn learning_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        if self.is_learning() {
            result.push(self);
        }
        for child in &self.children {
            result.extend(child.learning_nodes());
        }
        result
    }
}

pub struct CategoryForest {
    pub roots: Vec<CategoryNode>,
}

impl CategoryForest {
    #[tracing::instrument(level = "debug", skip(categories))]
    pub fn from_categories(categories: &[Category]) -> Self {
        debug!("Building CategoryForest from {} categories", categories.len());

        let mut nodes: std::collections::HashMap<i64, CategoryNode> = categories
            .iter()
            .map(|c| (c.id, CategoryNode::from_category(c)))
            .collect();
        trace!("Created {} node entries", nodes.len());

        let mut root_ids = Vec::new();
        let mut children_map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();

        for cat in categories {
            if let Some(parent_id) = cat.parent_id {
                children_map.entry(parent_id).or_default().push(cat.id);
                trace!("Category {} -> parent {}", cat.id, parent_id);
            } else {
                root_ids.push(cat.id);
                trace!("Root category: {}", cat.id);
            }
        }

        fn build_tree(
            root_id: i64,
            nodes: &mut std::collections::HashMap<i64, CategoryNode>,
            children_map: &std::collections::HashMap<i64, Vec<i64>>,
        ) -> Option<CategoryNode> {
            let mut node = nodes.remove(&root_id)?;
            if let Some(child_ids) = children_map.get(&root_id) {
                trace!("Building children for node {}: {} children", root_id, child_ids.len());
                for &cid in child_ids {
                    if let Some(child) = build_tree(cid, nodes, children_map) {
                        node.children.push(child);
                    }
                }
            }
            Some(node)
        }

        let roots: Vec<CategoryNode> = root_ids
            .into_iter()
            .filter_map(|rid| build_tree(rid, &mut nodes, &children_map))
            .collect();

        debug!("CategoryForest built with {} root nodes", roots.len());
        Self { roots }
    }

    pub fn find(&self, id: i64) -> Option<&CategoryNode> {
        for root in &self.roots {
            if let Some(found) = root.find(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_mut(&mut self, id: i64) -> Option<&mut CategoryNode> {
        for root in &mut self.roots {
            if let Some(found) = root.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_by_path(&self, path: &str) -> Option<&CategoryNode> {
        for root in &self.roots {
            if let Some(found) = root.find_by_path(path) {
                return Some(found);
            }
        }
        None
    }

    pub fn all_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        for root in &self.roots {
            result.extend(root.collect_all());
        }
        result
    }

    pub fn leaf_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        for root in &self.roots {
            result.extend(root.leaf_nodes());
        }
        result
    }

    pub fn learning_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        for root in &self.roots {
            result.extend(root.learning_nodes());
        }
        result
    }
}