use iced::widget::{column, row, text, container, button};
use iced::{Element, Length, Color};
use crate::timer::TimerState;

pub struct TimerDisplay;

impl TimerDisplay {
    pub fn view(state: &TimerState, elapsed_ms: i64) -> Element<'static, ()> {
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
        
        column![
            text(display).size(48),
            container(text(status_label).size(16).color(status_color))
                .padding([4, 12])
        ]
        .spacing(8)
        .into()
    }
}