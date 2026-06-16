use iced::widget::{button, column, row, text, container, text_input, pick_list, Space};
use iced::{Element, Color};
use crate::data::models::MemoryQuality;
use crate::gui::Message;

pub struct LinkTimerModal {
    pub duration_ms: i64,
    pub card_path: String,
    pub card_dropdown: Vec<String>,
    pub selected_card: Option<String>,
    pub memory_quality: MemoryQuality,
}

impl LinkTimerModal {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let duration_text = format_duration(self.duration_ms);
        
        container(
            column![
                text("计时完成").size(20).color(Color::WHITE),
                
                text(format!("学习时长: {}", duration_text))
                    .size(16)
                    .color(Color::WHITE),
                
                Space::new().height(16),
                
                // 卡片路径输入（手动输入）
                row![
                    text("关联到卡片:").color(Color::WHITE),
                    text_input("输入卡片路径...", &self.card_path)
                        .on_input(Message::TimerCardPathChanged),
                ],
                
                // 或下拉选择
                row![
                    text("或选择:").color(Color::WHITE),
                    pick_list(
                        self.card_dropdown.clone(),
                        self.selected_card.clone(),
                        Message::TimerCardSelected,
                    ),
                ],
                
                // 或创建新卡片
                button(text("创建新卡片..."))
                    .on_press(Message::TimerCreateNewCard),
                
                Space::new().height(16),
                
                // 记忆质量选择
                row![
                    text("记忆质量:").color(Color::WHITE),
                    MemoryQualitySelector::view(&self.memory_quality),
                ],
                
                Space::new().height(24),
                
                // 按钮
                row![
                    button(text("取消"))
                        .on_press(Message::TimerLinkModeClose),
                    button(text("保存并预测"))
                        .on_press(Message::TimerLinkConfirm),
                ]
                .spacing(12),
            ]
            .spacing(8)
        )
        .padding(24)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                ..Default::default()
            }
        })
        .into()
    }
}

fn format_duration(ms: i64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    
    if hours > 0 {
        format!("{}小时 {}分钟 {}秒", hours, minutes % 60, seconds % 60)
    } else if minutes > 0 {
        format!("{}分钟 {}秒", minutes, seconds % 60)
    } else {
        format!("{}秒", seconds)
    }
}

pub struct MemoryQualitySelector;

impl MemoryQualitySelector {
    pub fn view(quality: &MemoryQuality) -> Element<'_, Message> {
        row![
            quality_button("重学", MemoryQuality::Relearn, quality),
            quality_button("困难", MemoryQuality::Hard, quality),
            quality_button("好", MemoryQuality::Good, quality),
            quality_button("简单", MemoryQuality::Easy, quality),
        ]
        .spacing(8)
        .into()
    }
}

fn quality_button<'a>(label: &'a str, quality: MemoryQuality, current: &'a MemoryQuality) -> Element<'a, Message> {
    let is_selected = *current == quality;
    
    button(text(label).color(Color::WHITE))
        .on_press(Message::TimerMemoryQualityChanged(quality))
        .style(move |theme: &iced::Theme, _| {
            let palette = theme.extended_palette();
            if is_selected {
                iced::widget::button::Style {
                    background: Some(Color::from_rgb(0.2, 0.6, 0.86).into()),
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            } else {
                iced::widget::button::Style {
                    background: Some(palette.background.strong.color.into()),
                    text_color: palette.background.base.text,
                    ..Default::default()
                }
            }
        })
        .into()
}
