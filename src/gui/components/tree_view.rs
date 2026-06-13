use iced::widget::{column, row, text, button};
use iced::{Element, Length};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_card: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct TreeView {
    expanded: HashMap<String, bool>,
}

impl Default for TreeView {
    fn default() -> Self {
        Self {
            expanded: HashMap::new(),
        }
    }
}

impl TreeView {
    pub fn view<'a>(&'a self, nodes: &'a [TreeNode], selected: Option<&'a str>) -> Element<'a, String> {
        column(nodes.iter().map(|node| self.view_node(node, selected, 0)))
            .spacing(2)
            .into()
    }
    
    fn view_node(&self, node: &TreeNode, selected: Option<&str>, depth: usize) -> Element<String> {
        let indent = "  ".repeat(depth);
        let icon = if node.is_card { "📄" } else { "📁" };
        
        let is_selected = selected == Some(node.path.as_str());
        
        let content = row![
            text(format!("{}{} {}", indent, icon, node.name)).color(iced::Color::WHITE),
        ]
        .spacing(4);
        
        let btn = button(content)
            .on_press(node.path.clone())
            .style(move |_, _| {
                if is_selected {
                    iced::widget::button::Style {
                        background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    }
                } else {
                    iced::widget::button::Style::default()
                }
            })
            .width(Length::Fill);
        
        if node.is_card {
            btn.into()
        } else {
            let is_expanded = self.expanded.get(&node.path).copied().unwrap_or(true);
            
            column![
                btn,
                if is_expanded && !node.children.is_empty() {
                    column(node.children.iter().map(|c| self.view_node(c, selected, depth + 1)))
                        .spacing(1)
                } else {
                    column![]
                }
            ]
            .into()
        }
    }
    
    pub fn toggle(&mut self, path: &str) {
        let current = self.expanded.get(path).copied().unwrap_or(true);
        self.expanded.insert(path.to_string(), !current);
    }
    
    pub fn expand_all(&mut self, nodes: &[TreeNode]) {
        for node in nodes {
            if !node.is_card {
                self.expanded.insert(node.path.clone(), true);
                self.expand_all(&node.children);
            }
        }
    }
    
    pub fn expand_to_path(&mut self, nodes: &[TreeNode], target_path: &str) {
        for node in nodes {
            if target_path.starts_with(&node.path) {
                if !node.is_card {
                    self.expanded.insert(node.path.clone(), true);
                    self.expand_to_path(&node.children, target_path);
                }
                return;
            }
        }
    }
    
    pub fn is_expanded(&self, path: &str) -> bool {
        self.expanded.get(path).copied().unwrap_or(true)
    }
}