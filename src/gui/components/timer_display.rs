use iced::widget::{column, row, text, container, button, Space};
use iced::{Element, Length, Color};
use crate::timer::TimerState;
use crate::gui::Message;

pub struct TimerDisplay;

impl TimerDisplay {
    pub fn view<'a>(state: &'a TimerState, elapsed_ms: i64, current_card: Option<&'a str>) -> Element<'a, Message> {
        let seconds = elapsed_ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        
        let display = format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60);
        
        let (status_color, status_label) = match state {
            TimerState::Idle => (Color::from_rgb(0.4, 0.4, 0.4), "空闲"),
            TimerState::Running { .. } => (Color::from_rgb(0.2, 0.6, 0.86), "运行中"),
            TimerState::Paused { .. } => (Color::from_rgb(0.95, 0.61, 0.07), "已暂停"),
            TimerState::Stopped { .. } => (Color::from_rgb(0.4, 0.4, 0.4), "已停止"),
        };
        
        let card_element: Element<Message> = if let Some(card) = current_card {
            container(
                text(card)
                    .size(14)
                    .color(Color::from_rgb(0.7, 0.7, 0.7))
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
        } else {
            container(Space::new().height(20))
                .width(Length::Fill)
                .into()
        };
        
        column![
            // 时间显示 - 大尺寸
            container(
                text(display)
                    .size(72)
                    .color(Color::WHITE)
            )
            .width(Length::FillPortion(2))
            .height(Length::FillPortion(1))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
            
            // 当前学习卡片路径（如果有）
            card_element,
            
            // 状态标签
            container(text(status_label).size(16).color(status_color))
                .padding([4, 12])
                .center_x(Length::Fill)
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}