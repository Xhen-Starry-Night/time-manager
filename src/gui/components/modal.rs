use iced::widget::{column, row, text, button, container, Space};
use iced::{Element, Length, Color, Alignment};

pub struct ModalView;

impl ModalView {
    pub fn view<'a, Message: Clone + 'a>(
        title: &'a str,
        content: Element<'a, Message>,
        on_confirm: Message,
        on_cancel: Message,
    ) -> Element<'a, Message> {
        let close_btn = button(text("✕").color(Color::WHITE))
            .on_press(on_cancel.clone())
            .style(|_, _| iced::widget::button::Style {
                background: Some(Color::TRANSPARENT.into()),
                text_color: Color::from_rgb(0.6, 0.6, 0.6),
                ..Default::default()
            });

        let header = row![
            text(title).size(18).color(Color::WHITE),
            Space::new().width(Length::Fill),
            close_btn,
        ]
        .padding(16)
        .width(Length::Fill)
        .align_y(Alignment::Center);

        let cancel_btn = button(text("取消").color(Color::WHITE))
            .on_press(on_cancel)
            .style(|_, _| iced::widget::button::Style {
                background: Some(Color::from_rgb(0.3, 0.3, 0.3).into()),
                text_color: Color::WHITE,
                ..Default::default()
            });

        let confirm_btn = button(text("确认").color(Color::WHITE))
            .on_press(on_confirm)
            .style(|_, _| iced::widget::button::Style {
                background: Some(Color::from_rgb(0.2, 0.6, 0.86).into()),
                text_color: Color::WHITE,
                ..Default::default()
            });

        let footer = row![
            Space::new().width(Length::Fill),
            cancel_btn,
            confirm_btn,
        ]
        .spacing(12)
        .padding(16);

        let dialog = container(
            column![
                header,
                container(content).padding(16),
                footer,
            ]
        )
        .width(Length::Fixed(400.0))
        .style(|_| iced::widget::container::Style {
            background: Some(Color::from_rgb(0.2, 0.2, 0.2).into()),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                ..Default::default()
            })
            .into()
    }
}
