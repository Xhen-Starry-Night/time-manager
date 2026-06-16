use iced::widget::{column, row, text, text_input, radio, pick_list, Space};
use iced::{Element, Length};
use crate::gui::{Message, NodeType};
use crate::app::category_tab::NewNodeForm;

pub struct NewNodeModal;

impl NewNodeModal {
    pub fn view(form: &NewNodeForm, presets: &[String]) -> Element<'static, Message> {
        let parent_path = form.parent_path.clone();
        let name = form.name.clone();
        let preset = form.preset.clone();
        let presets_owned = presets.to_vec();
        
        let path_preview = row![
            text("在 "),
            text(format!("\"{}\"", parent_path)).color(iced::Color::from_rgb(0.3, 0.7, 0.9)),
            text(" 下创建"),
        ];
        
        let type_selector = row![
            radio(
                "文件夹",
                NodeType::Folder,
                Some(form.node_type),
                Message::NewNodeTypeChanged,
            ),
            radio(
                "学习卡片",
                NodeType::Card,
                Some(form.node_type),
                Message::NewNodeTypeChanged,
            ),
        ]
        .spacing(16);
        
        let name_input = text_input("名称", &name)
            .on_input(Message::NewNodeNameChanged)
            .width(Length::Fill);
        
        let preset_selector = if form.node_type == NodeType::Card {
            Some(
                column![
                    text("预设:"),
                    pick_list(presets_owned, Some(preset), Message::NewNodePresetChanged)
                        .width(Length::Fill),
                ]
                .spacing(4)
            )
        } else {
            None
        };
        
        let mut content = column![
            path_preview,
            Space::new().height(16),
            text("类型:"),
            type_selector,
            Space::new().height(12),
            text("名称:"),
            name_input,
        ]
        .spacing(4);
        
        if let Some(preset_ui) = preset_selector {
            content = content
                .push(Space::new().height(12))
                .push(preset_ui);
        }
        
        content
            .padding(16)
            .into()
    }
}
