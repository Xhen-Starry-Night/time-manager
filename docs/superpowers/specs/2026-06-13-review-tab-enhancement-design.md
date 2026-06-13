# GUI 复习看板完善设计文档

> **日期**: 2026-06-13
> **状态**: 设计阶段
> **基于**: DESIGN_v3.md §4.3.2 + 开发路线图 Phase 4

---

## 1. 功能概述

**目标**: 完善复习看板功能，提供与分类树一致的交互体验，增加快捷计时入口。

**核心需求**:
- 左右分栏布局：左侧卡片列表 + 右侧详情面板
- 详情面板显示选中卡片的完整信息
- 快捷计时入口：一键开始复习
- 重新预测功能：刷新所有卡片的预测状态

---

## 2. 界面布局

### 2.1 当前布局

```
┌────────────────────────────────────────────────────────────┐
│ 复习看板                    [搜索...] [紧急程度▼]          │
├────────────────────────────────────────────────────────────┤
│                                                             │
│ 🔴 语言/英语/六级/单词   已过期 2 天                        │
│ 🔴 数学/微积分/极限      已过期 1 天                        │
│ 🟡 语言/英语/语法        2 天后                             │
│ 🟢 语言/日语/N2/阅读     5 天后                             │
│ ...                                                         │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### 2.2 目标布局

```
┌────────────────────────────────────────────────────────────┐
│ 复习看板                    [搜索...] [紧急程度▼] [重新预测]│
├────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─────────────────────┬──────────────────────────────────┐ │
│ │                     │                                  │ │
│ │ 🔴 语言/英语/六级/单词│   节点详情                        │ │
│ │   已过期 2 天       │                                  │ │
│ │                     │   路径: 语言 > 英语 > 六级 > 单词 │ │
│ │ 🔴 数学/微积分/极限 │                                  │ │
│ │   已过期 1 天       │   学习统计                        │ │
│ │                     │   会话次数: 5 次                  │ │
│ │ 🟡 语言/英语/语法   │   总学习时长: 2h 15m              │ │
│ │   2 天后           │   平均时长: 27 分钟                │ │
│ │                     │                                  │ │
│ │ 🟢 语言/日语/N2/阅读│   上次复习: 2026-05-29            │ │
│ │   5 天后           │   下次复习: 2026-06-01            │ │
│ │                     │                                  │ │
│ │                     │   ────────────────────────        │ │
│ │                     │                                  │ │
│ │                     │   [开始计时]                      │ │
│ │                     │                                  │ │
│ └─────────────────────┴──────────────────────────────────┘ │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

---

## 3. 功能详细设计

### 3.1 左右分栏布局

**实现方式**:
- 使用 `row![]` 容器
- 左侧：`Length::FillPortion(2)` - 卡片列表
- 右侧：`Length::FillPortion(3)` - 详情面板

**状态管理**:
- 新增 `review_tab.selected_card: Option<String>` - 选中卡片路径
- 点击卡片列表项时更新 `selected_card`

### 3.2 卡片列表增强

**当前**: 仅显示路径和紧迫度状态

**增强**:
- 选中状态高亮（背景色变化）
- 点击选中 + 显示详情
- 保持紧迫度颜色标识

### 3.3 详情面板内容

**显示内容**:

| 信息 | 数据来源 |
|------|----------|
| 路径 | card_path |
| 会话次数 | card.review_records.len() |
| 总学习时长 | sum(review_records.duration_ms) |
| 平均时长 | 总时长 / 会话次数 |
| 上次复习 | review_records.last().timestamp |
| 下次复习 | prediction.next_review |
| 预设 | prediction.preset_used |

**统计计算**:
```rust
fn calculate_stats(card: &Card) -> (usize, i64, i64) {
    let sessions = card.review_records.len();
    let total_ms: i64 = card.review_records.iter()
        .map(|r| r.duration_ms)
        .sum();
    let avg_ms = if sessions > 0 { total_ms / sessions as i64 } else { 0 };
    (sessions, total_ms, avg_ms)
}
```

### 3.4 开始计时按钮

**功能**:
1. 点击按钮
2. 跳转到计时器 Tab
3. 自动开始计时
4. 记录关联路径（可选）

**实现**:
```rust
Message::StartReviewTimer(path) => {
    self.active_tab = TabId::Timer;
    self.timer_manager.start();
    self.timer_tab.state = self.timer_manager.get_state().clone();
    // 可选：记录 path 供停止时链接
    Task::none()
}
```

### 3.5 重新预测按钮

**功能**:
- 遍历所有卡片
- 对有 review_records 的卡片调用 FSRS 预测
- 更新 prediction 字段

**实现**:
```rust
Message::RefreshPredictions => {
    let data_fs = Arc::new(self.data_fs.clone());
    return Task::future(async move {
        let trees = data_fs.list_trees().unwrap_or_default();
        let mut all_cards = Vec::new();
        for tree in &trees {
            let cards = data_fs.list_cards(tree).unwrap_or_default();
            all_cards.extend(cards);
        }
        
        // 对每个卡片重新预测
        for (path, mut card) in all_cards {
            if !card.review_records.is_empty() {
                // 调用 FSRS 预测
                let prediction = predict_next_review(&card.review_records, "default");
                card.prediction = Some(prediction);
                data_fs.save_card(&path, &card).ok();
            }
        }
        
        // 重新加载数据
        Message::DataLoaded(Ok(DataSnapshot { ... }))
    });
}
```

---

## 4. 消息类型

### 4.1 新增消息

```rust
pub enum Message {
    // 现有...
    
    // 复习看板相关
    ReviewCardSelected(String),     // 选中卡片
    StartReviewTimer(String),       // 开始复习计时
    RefreshPredictions,             // 重新预测所有卡片
}
```

### 4.2 消息处理

```rust
Message::ReviewCardSelected(path) => {
    self.review_tab.selected_card = Some(path);
    Task::none()
}

Message::StartReviewTimer(path) => {
    self.active_tab = TabId::Timer;
    self.timer_manager.start();
    self.timer_tab.state = self.timer_manager.get_state().clone();
    // TODO: 记录 path 供停止时链接
    Task::none()
}

Message::RefreshPredictions => {
    // 见 3.5 实现
}
```

---

## 5. 数据模型

### 5.1 ReviewTabState 扩展

```rust
pub struct ReviewTabState {
    pub cards: Vec<(String, Card)>,
    pub search_query: String,
    pub urgency_filter: Option<u32>,
    pub selected_card: Option<String>,  // 新增
}
```

### 5.2 统计数据结构

```rust
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

---

## 6. UI 组件

### 6.1 卡片列表项

```rust
fn review_card_item(
    path: &str,
    urgency: i32,
    is_selected: bool,
) -> Element<Message> {
    let (urgency_text, urgency_color) = match urgency {
        3 => ("已过期", iced::Color::from_rgb(0.9, 0.3, 0.2)),
        2 => ("今日", iced::Color::from_rgb(0.9, 0.7, 0.2)),
        1 => ("3天内", iced::Color::from_rgb(0.3, 0.7, 0.4)),
        _ => ("稍后", iced::Color::from_rgb(0.5, 0.5, 0.5)),
    };
    
    let path = path.to_string();
    button(
        column![
            text(path).color(iced::Color::WHITE),
            text(urgency_text).color(urgency_color).size(12),
        ]
        .spacing(4)
    )
    .on_press(Message::ReviewCardSelected(path))
    .style(move |_, _| {
        if is_selected {
            iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.3, 0.5, 0.6).into()),
                ..Default::default()
            }
        } else {
            iced::widget::button::Style::default()
        }
    })
    .width(Length::Fill)
    .into()
}
```

### 6.2 详情面板

```rust
fn review_detail_panel(
    path: &str,
    card: &Card,
    stats: &CardStats,
) -> Element<Message> {
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
    column![
        text("节点详情").size(18).color(iced::Color::WHITE),
        Space::new().height(12),
        
        row![
            text("路径: ").color(iced::Color::WHITE),
            text(path_display).color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
        ],
        Space::new().height(12),
        
        text("学习统计").color(iced::Color::WHITE),
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
        
        text("复习状态").color(iced::Color::WHITE),
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
    .padding(16)
    .spacing(8)
    .into()
}
```

---

## 7. 实现清单

### Task 4.1: 扩展 ReviewTabState
- 新增 `selected_card: Option<String>` 字段
- 修改 `src/app/review_tab.rs`

### Task 4.2: 新增消息类型
- 新增 `ReviewCardSelected`, `StartReviewTimer`, `RefreshPredictions`
- 修改 `src/gui/messages.rs`

### Task 4.3: 修改复习看板布局
- 实现左右分栏
- 修改 `src/app/mod.rs` view 方法

### Task 4.4: 实现详情面板
- 显示路径、统计、复习状态
- 添加开始计时按钮

### Task 4.5: 实现统计计算
- 新增 `CardStats` 结构
- 实现 `from_card` 方法

### Task 4.6: 实现开始计时功能
- 跳转到计时器 Tab
- 自动开始计时

### Task 4.7: 实现重新预测功能
- 遍历所有卡片
- 调用 FSRS 预测
- 更新并保存

---

## 8. 验收标准

- [ ] 复习看板左右分栏布局
- [ ] 点击卡片项显示详情面板
- [ ] 详情面板显示路径
- [ ] 详情面板显示学习统计
- [ ] 详情面板显示复习状态
- [ ] 开始计时按钮跳转到计时器 Tab
- [ ] 开始计时按钮自动启动计时
- [ ] 重新预测按钮刷新所有卡片预测
- [ ] 选中状态高亮显示

---

## 9. 时间估计

| 任务 | 时间 |
|------|------|
| Task 4.1: 扩展状态 | 5 分钟 |
| Task 4.2: 消息类型 | 5 分钟 |
| Task 4.3: 分栏布局 | 20 分钟 |
| Task 4.4: 详情面板 | 30 分钟 |
| Task 4.5: 统计计算 | 15 分钟 |
| Task 4.6: 开始计时 | 10 分钟 |
| Task 4.7: 重新预测 | 15 分钟 |
| **总计** | **约 1.5 小时** |

---

## 10. 后续优化

**暂不实现**:
- 详情面板编辑功能
- 批量操作
- 导出统计报告
- 卡片对比视图

**理由**: 保持 MVP 简洁，后续根据用户反馈迭代。
