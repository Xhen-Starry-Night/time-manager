use iced::widget::{column, row, text, container};
use iced::{Element, Length};
use crate::data::models::Card;

pub struct CardDetail;

impl CardDetail {
    pub fn view(path: &str, card: &Card) -> Element<'static, ()> {
        let path = path.to_string();
        
        let prediction_info = if let Some(ref pred) = card.prediction {
            let next_review = pred.next_review.format("%Y-%m-%d").to_string();
            let algorithm = pred.algorithm.clone();
            let preset = pred.preset_used.clone();
            
            column![
                row![
                    text("下次复习:").width(Length::Fixed(100.0)),
                    text(next_review).width(Length::Fill),
                ],
                row![
                    text("算法:").width(Length::Fixed(100.0)),
                    text(algorithm).width(Length::Fill),
                ],
                row![
                    text("预设:").width(Length::Fixed(100.0)),
                    text(preset).width(Length::Fill),
                ],
            ]
        } else {
            column![text("未预测")]
        };
        
        let review_count = card.review_records.len();
        
        column![
            text(path).size(18),
            container(
                column![
                    row![
                        text("复习记录:").width(Length::Fixed(100.0)),
                        text(format!("{} 次", review_count)).width(Length::Fill),
                    ],
                    prediction_info,
                ]
                .spacing(8)
            )
            .padding(16)
            .width(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Color::from_rgb(0.96, 0.96, 0.96).into()),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
        .spacing(16)
        .into()
    }
}