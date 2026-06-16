use iced::widget::{container, text};
use iced::{Element, Color};

fn urgency_color(urgency: u32) -> Color {
    match urgency {
        3 => Color::from_rgb(0.906, 0.298, 0.235),
        2 => Color::from_rgb(0.953, 0.612, 0.071),
        1 => Color::from_rgb(0.153, 0.682, 0.376),
        _ => Color::from_rgb(0.584, 0.647, 0.651),
    }
}

pub struct UrgencyBadge;

impl UrgencyBadge {
    pub fn view(urgency: u32) -> Element<'static, ()> {
        let label = match urgency {
            3 => "已过期",
            2 => "今日",
            1 => "近期",
            _ => "稍后",
        };
        
        let color = urgency_color(urgency);
        
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