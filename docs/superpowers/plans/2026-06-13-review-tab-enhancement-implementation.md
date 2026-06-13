# GUI 复习看板完善实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完善复习看板功能，实现左右分栏布局、详情面板、快捷计时和重新预测功能。

**Architecture:** 
- 左右分栏：卡片列表 + 详情面板
- 统计计算：CardStats 结构聚合 review_records
- 消息驱动：选中卡片、开始计时、重新预测

**Tech Stack:** Rust, Iced 0.14, chrono

---

## 文件结构

### 新建文件
- `src/app/review_tab.rs` - ReviewTabState 扩展和 CardStats

### 修改文件
- `src/gui/messages.rs` - 新增消息类型
- `src/app/mod.rs` - 复习看板布局、消息处理、详情面板

---

## Task 1: 扩展 ReviewTabState

**Files:**
- Modify: `src/app/review_tab.rs`

### Step 1: 添加 selected_card 字段

```rust
// src/app/review_tab.rs

use crate::data::models::Card;

#[derive(Default)]
pub struct ReviewTabState {
    pub cards: Vec<(String, Card)>,
    pub search_query: String,
    pub urgency_filter: Option<u32>,
    pub selected_card: Option<String>,
}
```

### Step 2: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 2: 新增消息类型

**Files:**
- Modify: `src/gui/messages.rs`

### Step 1: 添加消息枚举

```rust
// src/gui/messages.rs - Message 枚举中添加

ReviewCardSelected(String),
StartReviewTimer(String),
RefreshPredictions,
```

### Step 2: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 3: 实现 CardStats 结构

**Files:**
- Modify: `src/app/review_tab.rs`

### Step 1: 添加 CardStats 结构

```rust
// src/app/review_tab.rs

use chrono::{DateTime, Utc};
use crate::data::models::Card;

pub struct CardStats {
    pub sessions: usize,
    pub total_duration_ms: i64,
    pub avg_duration_ms: i64,
    pub last_review: Option<DateTime<Utc>>,
    pub next_review: Option<DateTime<Utc>>,
}

impl CardStats {
    pub fn from_card(card: &Card) -> Self {
        let sessions = card.review_records.len();
        let total_ms: i64 = card.review_records.iter()
            .map(|r| r.duration_ms)
            .sum();
        let avg_ms = if sessions > 0 { total_ms / sessions as i64 } else { 0 };
        let last_review = card.review_records.last()
            .map(|r| r.timestamp);
        let next_review = card.prediction.as_ref()
            .map(|p| p.next_review);
        
        Self {
            sessions,
            total_duration_ms: total_ms,
            avg_duration_ms: avg_ms,
            last_review,
            next_review,
        }
    }
}
```

### Step 2: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 4: 实现消息处理

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 导入 ReviewTabState

```rust
// src/app/mod.rs

mod review_tab;
use review_tab::{ReviewTabState, CardStats};
```

### Step 2: 实现 ReviewCardSelected 消息处理

```rust
// src/app/mod.rs - update 方法中

Message::ReviewCardSelected(path) => {
    self.review_tab.selected_card = Some(path);
    Task::none()
}
```

### Step 3: 实现 StartReviewTimer 消息处理

```rust
// src/app/mod.rs - update 方法中

Message::StartReviewTimer(path) => {
    self.active_tab = TabId::Timer;
    let _ = self.timer_manager.start();
    self.timer_tab.state = self.timer_manager.get_state().clone();
    Task::none()
}
```

### Step 4: 实现 RefreshPredictions 消息处理

```rust
// src/app/mod.rs - update 方法中

Message::RefreshPredictions => {
    let data_fs = Arc::new(self.data_fs.clone());
    return Task::future(async move {
        let trees = data_fs.list_trees().unwrap_or_default();
        let mut all_cards = Vec::new();
        for tree in &trees {
            let cards = data_fs.list_cards(tree).unwrap_or_default();
            all_cards.extend(cards);
        }
        
        for (path, mut card) in all_cards.clone() {
            if !card.review_records.is_empty() {
                use crate::data::models::Prediction;
                card.prediction = Some(Prediction {
                    algorithm: "fsrs".to_string(),
                    next_review: chrono::Utc::now() + chrono::Duration::days(1),
                    fsrs_state_bytes: vec![],
                    preset_used: "default".to_string(),
                });
                let _ = data_fs.save_card(&path, &card);
            }
        }
        
        let trees = data_fs.list_trees().unwrap_or_default();
        let mut all_cards = Vec::new();
        for tree in &trees {
            let cards = data_fs.list_cards(tree).unwrap_or_default();
            all_cards.extend(cards);
        }
        let presets = data_fs.list_presets().unwrap_or_default();
        let todos = data_fs.list_todos().unwrap_or_default();
        
        Message::DataLoaded(Ok(DataSnapshot {
            trees,
            cards: all_cards,
            presets,
            todos,
        }))
    });
}
```

### Step 5: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 5: 修改复习看板布局

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 添加重新预测按钮

找到 `TabId::Review` 分支中的 filter_buttons，修改为：

```rust
// src/app/mod.rs - view 方法中

let filter_buttons = row![
    filter_button("全部", None, self.review_tab.urgency_filter),
    filter_button("已过期", Some(3), self.review_tab.urgency_filter),
    filter_button("今日", Some(2), self.review_tab.urgency_filter),
    filter_button("近期", Some(1), self.review_tab.urgency_filter),
    filter_button("稍后", Some(0), self.review_tab.urgency_filter),
    button(text("重新预测").color(iced::Color::WHITE))
        .on_press(Message::RefreshPredictions)
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.3, 0.5, 0.7).into()),
            text_color: iced::Color::WHITE,
            ..Default::default()
        }),
]
.spacing(8)
.padding(8);
```

### Step 2: 实现左右分栏

找到 `TabId::Review` 分支的 content 赋值，修改为：

```rust
// src/app/mod.rs - view 方法中

let card_list: Element<Message> = if cards_with_urgency.is_empty() {
    container(
        text("暂无需要复习的卡片")
            .size(16)
            .color(iced::Color::WHITE)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
} else {
    scrollable(
        column(
            cards_with_urgency.iter().map(|(path, urgency, _)| {
                let is_selected = self.review_tab.selected_card.as_ref() == Some(path);
                review_card_item(path, *urgency, is_selected)
            })
        )
        .spacing(4)
    )
    .into()
};

let left_panel = container(card_list)
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .padding(8);

let right_panel = if let Some(ref selected_path) = self.review_tab.selected_card {
    if let Some((_, card)) = cards_with_urgency.iter()
        .find(|(path, _, _)| path == selected_path) 
    {
        let stats = CardStats::from_card(card);
        review_detail_panel(selected_path, card, &stats)
    } else {
        container(
            text("选择卡片查看详情")
                .size(16)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
} else {
    container(
        text("选择卡片查看详情")
            .size(16)
            .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
    )
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
};

let content = column![
    filter_buttons,
    rule::horizontal(1.0),
    row![left_panel, right_panel]
        .spacing(1)
        .width(Length::Fill)
        .height(Length::Fill),
]
.width(Length::Fill)
.height(Length::Fill);
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 6: 实现 review_card_item 组件

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 添加组件函数

```rust
// src/app/mod.rs - 文件末尾添加

fn review_card_item(path: &str, urgency: i32, is_selected: bool) -> Element<Message> {
    let (urgency_text, urgency_color) = match urgency {
        3 => ("已过期", iced::Color::from_rgb(0.9, 0.3, 0.2)),
        2 => ("今日", iced::Color::from_rgb(0.9, 0.7, 0.2)),
        1 => ("3天内", iced::Color::from_rgb(0.3, 0.7, 0.4)),
        _ => ("稍后", iced::Color::from_rgb(0.5, 0.5, 0.5)),
    };
    
    let path = path.to_string();
    button(
        column![
            text(path).color(iced::Color::WHITE).size(13),
            text(urgency_text).color(urgency_color).size(11),
        ]
        .spacing(2)
    )
    .on_press(Message::ReviewCardSelected(path))
    .style(move |_, _| {
        if is_selected {
            iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.3, 0.5, 0.6).into()),
                text_color: iced::Color::WHITE,
                ..Default::default()
            }
        } else {
            iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.25, 0.25, 0.25).into()),
                text_color: iced::Color::WHITE,
                ..Default::default()
            }
        }
    })
    .width(Length::Fill)
    .into()
}
```

### Step 2: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 7: 实现 review_detail_panel 组件

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 添加组件函数

```rust
// src/app/mod.rs - 文件末尾添加

fn review_detail_panel(path: &str, card: &Card, stats: &CardStats) -> Element<Message> {
    let path_display = path.split('/')
        .collect::<Vec<_>>()
        .join(" > ");
    
    let total_min = stats.total_duration_ms / 60000;
    let total_hours = total_min / 60;
    let total_mins = total_min % 60;
    let avg_min = stats.avg_duration_ms / 60000;
    
    let last_review = stats.last_review
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "无".to_string());
    
    let next_review = stats.next_review
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "未预测".to_string());
    
    let path = path.to_string();
    container(
        column![
            text("节点详情").size(18).color(iced::Color::WHITE),
            Space::new().height(12),
            
            row![
                text("路径: ").color(iced::Color::WHITE),
                text(path_display).color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            ],
            Space::new().height(12),
            
            text("学习统计").size(14).color(iced::Color::WHITE),
            column![
                row![
                    text("会话次数: ").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!("{} 次", stats.sessions)).color(iced::Color::WHITE),
                ],
                row![
                    text("总学习时长: ").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!("{}h {}m", total_hours, total_mins)).color(iced::Color::WHITE),
                ],
                row![
                    text("平均时长: ").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!("{} 分钟", avg_min)).color(iced::Color::WHITE),
                ],
            ]
            .spacing(4),
            Space::new().height(12),
            
            text("复习状态").size(14).color(iced::Color::WHITE),
            column![
                row![
                    text("上次复习: ").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    text(last_review).color(iced::Color::WHITE),
                ],
                row![
                    text("下次复习: ").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    text(next_review).color(iced::Color::WHITE),
                ],
            ]
            .spacing(4),
            Space::new().height(24),
            
            button(text("开始计时").color(iced::Color::WHITE))
                .on_press(Message::StartReviewTimer(path))
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
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(16)
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
        ..Default::default()
    })
    .into()
}
```

### Step 2: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 8: 测试验证

### Step 1: 构建

```bash
cargo build --release
```

Expected: 编译成功

### Step 2: 运行 GUI

```bash
TMD_DATA_DIR=/tmp/tmd-test-data cargo run --bin tmd-gui --release
```

Manual Test Checklist:
- [ ] 复习看板显示左右分栏布局
- [ ] 点击卡片项，右侧显示详情面板
- [ ] 详情面板显示路径、统计、复习状态
- [ ] 点击"开始计时"，跳转到计时器 Tab 并自动开始
- [ ] 点击"重新预测"，刷新所有卡片预测状态
- [ ] 选中卡片高亮显示
- [ ] 紧迫度过滤正常工作

---

## 验收标准

- [ ] ReviewTabState 包含 selected_card 字段
- [ ] ReviewCardSelected、StartReviewTimer、RefreshPredictions 消息定义
- [ ] CardStats 结构实现 from_card 方法
- [ ] 复习看板左右分栏布局
- [ ] 卡片列表项显示路径和紧迫度
- [ ] 选中状态高亮
- [ ] 详情面板显示完整信息
- [ ] 开始计时按钮跳转并启动计时器
- [ ] 重新预测按钮刷新预测

---

## 时间估计

| 任务 | 时间 |
|------|------|
| Task 1: 扩展状态 | 5 分钟 |
| Task 2: 消息类型 | 5 分钟 |
| Task 3: CardStats | 15 分钟 |
| Task 4: 消息处理 | 20 分钟 |
| Task 5: 布局修改 | 25 分钟 |
| Task 6: card_item | 15 分钟 |
| Task 7: detail_panel | 20 分钟 |
| Task 8: 测试验证 | 15 分钟 |
| **总计** | **约 2 小时** |
