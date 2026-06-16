use iced::widget::{column, text, scrollable, Space, container};
use iced::{Element, Length, Color};

use crate::gui::components::tree_view::{TreeNode, TreeView};
use crate::gui::Message;

pub struct TreePicker;

impl TreePicker {
    pub fn view<'a>(
        tree_view: &'a TreeView,
        tree_nodes: &'a [TreeNode],
        selected_card: Option<&'a str>,
    ) -> Element<'a, Message> {
        container(
            column![
                text("或从树形导航选择:").color(Color::WHITE).size(14),
                Space::new().height(4),
                scrollable(
                    tree_view.view_static(tree_nodes, selected_card.map(|s| s.to_string()))
                        .map(Message::TimerCardSelected)
                )
                .height(Length::Fixed(200.0)),
            ]
            .spacing(4)
        )
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.base.color.into()),
                ..Default::default()
            }
        })
        .padding(8)
        .into()
    }
}