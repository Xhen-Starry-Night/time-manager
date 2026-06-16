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
        Self::view_with_options(title, content, on_confirm, on_cancel, "确认", true)
    }
    
    pub fn view_with_options<'a, Message: Clone + 'a>(
        title: &'a str,
        content: Element<'a, Message>,
        on_confirm: Message,
        on_cancel: Message,
        confirm_label: &'a str,
        show_cancel: bool,
    ) -> Element<'a, Message> {
        let close_btn = button(text("✕").color(Color::WHITE))
            .on_press(on_cancel.clone())
            .style(move |theme: &iced::Theme, _| {
                let palette = theme.extended_palette();
                iced::widget::button::Style {
                    background: Some(Color::TRANSPARENT.into()),
                    text_color: palette.background.weak.text,
                    ..Default::default()
                }
            });

        let header = row![
            text(title).size(18),
            Space::new().width(Length::Fill),
            close_btn,
        ]
        .padding(16)
        .width(Length::Fill)
        .align_y(Alignment::Center);

        let confirm_btn = button(text(confirm_label).color(Color::WHITE))
            .on_press(on_confirm)
            .style(|_, _| iced::widget::button::Style {
                background: Some(Color::from_rgb(0.2, 0.6, 0.86).into()),
                text_color: Color::WHITE,
                ..Default::default()
            });

        let footer = if show_cancel {
            let cancel_btn = button(text("取消").color(Color::WHITE))
                .on_press(on_cancel)
            .style(move |theme: &iced::Theme, _| {
                let palette = theme.extended_palette();
                iced::widget::button::Style {
                    background: Some(palette.background.strong.color.into()),
                    text_color: palette.background.base.text,
                    ..Default::default()
                }
            });
            
            row![
                Space::new().width(Length::Fill),
                cancel_btn,
                confirm_btn,
            ]
            .spacing(12)
            .padding(16)
        } else {
            row![
                Space::new().width(Length::Fill),
                confirm_btn,
            ]
            .spacing(12)
            .padding(16)
        };

        let dialog = container(
            column![
                header,
                container(content).padding(16),
                footer,
            ]
        )
        .width(Length::Fixed(400.0))
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.weak.color.into()),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
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
