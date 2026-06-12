use iced::widget::{container, text};
use iced::{Element, Color};
use crate::gui::Theme;

pub struct UrgencyBadge;

impl UrgencyBadge {
    pub fn view(urgency: u32) -> Element<'static, ()> {
        let label = match urgency {
            3 => "已过期",
            2 => "今日",
            1 => "近期",
            _ => "稍后",
        };
        
        let color = Theme::urgency_color(urgency);
        
        container(text(label).color(Color::WHITE).size(12))
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(color.into()),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding([2, 8])
            .into()
    }
}