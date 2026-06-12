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
    pub fn view(&self, nodes: &[TreeNode], selected: Option<&str>) -> Element<String> {
        column(nodes.iter().map(|node| self.view_node(node, selected, 0)))
            .spacing(2)
            .into()
    }
    
    fn view_node(&self, node: &TreeNode, selected: Option<&str>, depth: usize) -> Element<String> {
        let indent = "  ".repeat(depth);
        let icon = if node.is_card { "📄" } else { "📁" };
        
        let is_selected = selected == Some(node.path.as_str());
        
        let content = row![
            text(format!("{}{} {}", indent, icon, node.name)),
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
            let is_expanded = self.expanded.get(&node.path).unwrap_or(&false);
            
            column![
                btn,
                if *is_expanded && !node.children.is_empty() {
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
        let current = self.expanded.get(path).unwrap_or(&false);
        self.expanded.insert(path.to_string(), !current);
    }
}