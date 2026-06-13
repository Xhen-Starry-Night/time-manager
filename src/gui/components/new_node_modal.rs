use iced::widget::{button, column, row, text, text_input, radio, pick_list, Space};
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
            text("在 ").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            text(format!("\"{}\"", parent_path)).color(iced::Color::from_rgb(0.3, 0.7, 0.9)),
            text(" 下创建").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
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
                    text("预设:").color(iced::Color::WHITE),
                    pick_list(presets_owned, Some(preset), Message::NewNodePresetChanged)
                        .width(Length::Fill),
                ]
                .spacing(4)
            )
        } else {
            None
        };
        
        let buttons = row![
            button(text("取消").color(iced::Color::WHITE))
                .on_press(Message::ModalClose)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.4, 0.4, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            button(text("创建").color(iced::Color::WHITE))
                .on_press(Message::NewNodeConfirm)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.3, 0.6, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(8)
        .width(Length::Fill)
        .push(Space::new().width(Length::Fill));
        
        let mut content = column![
            path_preview,
            Space::new().height(16),
            text("类型:").color(iced::Color::WHITE),
            type_selector,
            Space::new().height(12),
            text("名称:").color(iced::Color::WHITE),
            name_input,
        ]
        .spacing(4);
        
        if let Some(preset_ui) = preset_selector {
            content = content
                .push(Space::new().height(12))
                .push(preset_ui);
        }
        
        content
            .push(Space::new().height(24))
            .push(buttons)
            .padding(16)
            .into()
    }
}
