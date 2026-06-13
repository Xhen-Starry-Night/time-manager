use iced::widget::{column, row, text, text_input, radio, Space};
use iced::{Element, Length, Color};
use crate::gui::Message;

pub struct NewTodoForm;

impl NewTodoForm {
    pub fn view<'a>(
        content: &str,
        priority: Option<u32>,
        due_year: &str,
        due_month: &str,
        due_day: &str,
        due_hour: &str,
        due_minute: &str,
        tags: &str,
    ) -> Element<'a, Message> {
        let content_label = text("内容 *").color(Color::WHITE);
        let content_input = text_input("输入待办内容...", content)
            .on_input(Message::NewTodoContentChanged)
            .width(Length::Fill);
        
        let priority_label = text("优先级").color(Color::WHITE);
        let priority_none = radio(
            "无",
            None,
            Some(priority),
            Message::NewTodoPriorityChanged,
        );
        let priority_1 = radio(
            "P1",
            Some(1),
            Some(priority),
            Message::NewTodoPriorityChanged,
        );
        let priority_2 = radio(
            "P2",
            Some(2),
            Some(priority),
            Message::NewTodoPriorityChanged,
        );
        let priority_3 = radio(
            "P3",
            Some(3),
            Some(priority),
            Message::NewTodoPriorityChanged,
        );
        
        let priority_row = row![
            priority_none,
            priority_1,
            priority_2,
            priority_3,
        ]
        .spacing(12);
        
        let due_label = text("截止日期").color(Color::WHITE);
        
        let year_input = text_input("年", due_year)
            .on_input(Message::NewTodoDueYearChanged)
            .width(Length::Fixed(60.0));
        let month_input = text_input("月", due_month)
            .on_input(Message::NewTodoDueMonthChanged)
            .width(Length::Fixed(40.0));
        let day_input = text_input("日", due_day)
            .on_input(Message::NewTodoDueDayChanged)
            .width(Length::Fixed(40.0));
        let hour_input = text_input("时", due_hour)
            .on_input(Message::NewTodoDueHourChanged)
            .width(Length::Fixed(40.0));
        let minute_input = text_input("分", due_minute)
            .on_input(Message::NewTodoDueMinuteChanged)
            .width(Length::Fixed(40.0));
        
        let due_row = row![
            year_input,
            text("-").color(Color::WHITE),
            month_input,
            text("-").color(Color::WHITE),
            day_input,
            text("  ").color(Color::WHITE),
            hour_input,
            text(":").color(Color::WHITE),
            minute_input,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);
        
        let tags_label = text("标签").color(Color::WHITE);
        let tags_input = text_input("多个标签用空格分隔...", tags)
            .on_input(Message::NewTodoTagsChanged)
            .width(Length::Fill);
        let tags_hint = text("示例: 工作 紧急").color(Color::from_rgb(0.5, 0.5, 0.5));
        
        column![
            content_label,
            content_input,
            Space::new().height(Length::Fixed(12.0)),
            priority_label,
            priority_row,
            Space::new().height(Length::Fixed(12.0)),
            due_label,
            due_row,
            Space::new().height(Length::Fixed(12.0)),
            tags_label,
            tags_input,
            tags_hint,
        ]
        .spacing(4)
        .width(Length::Fill)
        .into()
    }
}