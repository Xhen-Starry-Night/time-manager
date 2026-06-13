# GUI Phase 5 计时器界面完善实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完善计时器界面，实现计时结束后的卡片链接流程、计时记录管理、快捷计时功能

**Architecture:** 
- 计时停止后保存数据到文件，状态恢复空闲，弹出链接模态框
- 计时记录独立管理，支持编辑（时长、时间）和链接到卡片
- 链接行为：将计时数据复制到目标卡片，此后二者无客观联系
- 快捷计时：从分类树/复习看板直接启动计时

**Tech Stack:** Rust, Iced 0.14, chrono

---

## 文件结构

### 新建文件
- `src/gui/components/link_timer_modal.rs` - 计时器链接模态框组件

### 修改文件
- `src/app/timer_tab.rs` - TimerTabState 扩展
- `src/gui/messages.rs` - 新增消息类型
- `src/gui/components/mod.rs` - 导出新组件
- `src/gui/components/timer_display.rs` - 增强显示
- `src/app/mod.rs` - 计时器 Tab 界面、消息处理
- `src/timer/mod.rs` - TimerManager 扩展

---

## Task 1: 扩展 TimerTabState

**Files:**
- Modify: `src/app/timer_tab.rs`

### Step 1: 添加新字段

```rust
// src/app/timer_tab.rs

use chrono::{DateTime, Utc};
use crate::data::models::MemoryQuality;
use crate::timer::TimerState;

pub struct TimerTabState {
    pub state: TimerState,
    pub elapsed_ms: i64,
    pub link_mode: bool,
    pub card_path_input: String,
    pub card_dropdown: Vec<String>,
    pub selected_card: Option<String>,
    pub memory_quality: MemoryQuality,
    pub show_history: bool,
    pub timer_history: Vec<TimerRecord>,
    pub current_card: Option<String>,
}

pub struct TimerRecord {
    pub id: uuid::Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: i64,
    pub linked_card: Option<String>,
    pub memory_quality: Option<MemoryQuality>,
}

impl TimerTabState {
    pub fn new(state: TimerState) -> Self {
        Self {
            state,
            elapsed_ms: 0,
            link_mode: false,
            card_path_input: String::new(),
            card_dropdown: Vec::new(),
            selected_card: None,
            memory_quality: MemoryQuality::Good,
            show_history: false,
            timer_history: Vec::new(),
            current_card: None,
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

## Task 2: 新增消息类型

**Files:**
- Modify: `src/gui/messages.rs`

### Step 1: 添加消息枚举

```rust
// src/gui/messages.rs - Message 枚举中添加

TimerLinkModeOpen,
TimerLinkModeClose,
TimerCardPathChanged(String),
TimerCardSelected(String),
TimerMemoryQualityChanged(MemoryQuality),
TimerLinkConfirm,
TimerCreateNewCard,

TimerHistoryLoad,
TimerHistoryLoaded(Vec<TimerRecord>),
TimerHistoryEdit(uuid::Uuid),
TimerHistoryDelete(uuid::Uuid),
TimerHistoryUpdate(TimerRecord),
TimerHistoryShow,
TimerHistoryHide,

QuickTimerStart(String),
```

### Step 2: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 3: 创建 LinkTimerModal 组件

**Files:**
- Create: `src/gui/components/link_timer_modal.rs`
- Modify: `src/gui/components/mod.rs`

### Step 1: 创建组件文件

```rust
// src/gui/components/link_timer_modal.rs

use iced::widget::{button, column, row, text, container, text_input, pick_list, Space};
use iced::{Element, Length, Color};
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
    pub fn view(&self) -> Element<Message> {
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
                    MemoryQualitySelector::view(&self.memory_quality)
                        .map(Message::TimerMemoryQualityChanged),
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
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.2, 0.2, 0.2).into()),
            ..Default::default()
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

struct MemoryQualitySelector;

impl MemoryQualitySelector {
    pub fn view(quality: &MemoryQuality) -> Element<Message> {
        // 实现记忆质量选择器
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

fn quality_button(label: &str, quality: MemoryQuality, current: &MemoryQuality) -> Element<Message> {
    let is_selected = *current == quality;
    
    button(text(label).color(Color::WHITE))
        .on_press(Message::TimerMemoryQualityChanged(quality))
        .style(move |_, _| {
            if is_selected {
                iced::widget::button::Style {
                    background: Some(Color::from_rgb(0.2, 0.6, 0.86).into()),
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            } else {
                iced::widget::button::Style {
                    background: Some(Color::from_rgb(0.3, 0.3, 0.3).into()),
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            }
        })
        .into()
}
```

### Step 2: 导出组件

```rust
// src/gui/components/mod.rs

pub mod link_timer_modal;
pub use link_timer_modal::LinkTimerModal;
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 4: 增强 TimerDisplay 组件

**Files:**
- Modify: `src/gui/components/timer_display.rs`

### Step 1: 修改显示逻辑

```rust
// src/gui/components/timer_display.rs

use iced::widget::{column, row, text, container, button, Space};
use iced::{Element, Length, Color};
use crate::timer::TimerState;
use crate::gui::Message;

pub struct TimerDisplay;

impl TimerDisplay {
    pub fn view(state: &TimerState, elapsed_ms: i64, current_card: Option<&str>) -> Element<Message> {
        let time_text = format_elapsed(elapsed_ms);
        
        column![
            // 时间显示 - 大尺寸，占窗口高度1/3，宽度2/3
            container(
                text(time_text)
                    .size(72)  // 大字体
                    .color(Color::WHITE)
            )
            .width(Length::FillPortion(2))
            .height(Length::FillPortion(1))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
            
            // 当前学习卡片路径（如果有，无"当前学习："前缀）
            if let Some(card) = current_card {
                container(
                    text(card)
                        .size(14)
                        .color(Color::from_rgb(0.7, 0.7, 0.7))
                )
                .width(Length::Fill)
                .center_x(Length::Fill)
            } else {
                container(Space::new().height(20))
                    .width(Length::Fill)
            },
            
            // 控制按钮
            row![
                start_button(state),
                pause_button(state),
                stop_button(state),
            ]
            .spacing(12)
            .width(Length::Fill)
            .center_x(Length::Fill),
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn format_elapsed(ms: i64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    
    format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
}

fn start_button(state: &TimerState) -> Element<Message> {
    // 实现开始按钮
    button(text("开始").color(Color::WHITE))
        .on_press(Message::TimerStarted)
        .style(|_, _| iced::widget::button::Style {
            background: Some(Color::from_rgb(0.2, 0.7, 0.4).into()),
            text_color: Color::WHITE,
            ..Default::default()
        })
        .into()
}

fn pause_button(state: &TimerState) -> Element<Message> {
    // 实现暂停按钮
    button(text("暂停").color(Color::WHITE))
        .on_press(Message::TimerPaused)
        .style(|_, _| iced::widget::button::Style {
            background: Some(Color::from_rgb(0.95, 0.61, 0.07).into()),
            text_color: Color::WHITE,
            ..Default::default()
        })
        .into()
}

fn stop_button(state: &TimerState) -> Element<Message> {
    // 实现停止按钮
    button(text("停止").color(Color::WHITE))
        .on_press(Message::TimerStopped(Ok(PathBuf::new())))
        .style(|_, _| iced::widget::button::Style {
            background: Some(Color::from_rgb(0.9, 0.3, 0.2).into()),
            text_color: Color::WHITE,
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

## Task 5: 实现计时器消息处理

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 导入新组件

```rust
// src/app/mod.rs

use crate::gui::components::{LinkTimerModal, TimerDisplay};
```

### Step 2: 实现 TimerStopped 消息处理

```rust
// src/app/mod.rs - update 方法中

Message::TimerStopped(result) => {
    match result {
        Ok(_path) => {
            self.timer_manager.stop();
            self.timer_tab.state = TimerState::Idle;
            self.timer_tab.elapsed_ms = 0;
            
            // 如果有预设卡片路径，进入链接模式
            if let Some(card_path) = self.timer_tab.current_card.take() {
                self.timer_tab.link_mode = true;
                self.timer_tab.card_path_input = card_path;
            }
        }
        Err(e) => {
            self.error_message = Some(e);
        }
    }
    Task::none()
}
```

### Step 3: 实现链接相关消息

```rust
// src/app/mod.rs - update 方法中

Message::TimerLinkModeOpen => {
    self.timer_tab.link_mode = true;
    Task::none()
}

Message::TimerLinkModeClose => {
    self.timer_tab.link_mode = false;
    Task::none()
}

Message::TimerCardPathChanged(path) => {
    self.timer_tab.card_path_input = path;
    Task::none()
}

Message::TimerCardSelected(path) => {
    self.timer_tab.selected_card = Some(path);
    Task::none()
}

Message::TimerMemoryQualityChanged(quality) => {
    self.timer_tab.memory_quality = quality;
    Task::none()
}

Message::TimerLinkConfirm => {
    if let Some(card_path) = &self.timer_tab.selected_card {
        let duration_ms = self.timer_tab.elapsed_ms;
        let quality = self.timer_tab.memory_quality.clone();
        
        // 创建 ReviewRecord
        let record = ReviewRecord {
            timestamp: Utc::now(),
            duration_ms,
            memory_quality: quality.clone(),
        };
        
        // 保存到卡片
        if let Ok(mut card) = self.data_fs.get_card(card_path) {
            card.review_records.push(record);
            
            // 触发 FSRS 预测
            if let Ok(predictor) = FsrsPredictor::new() {
                if let Ok((next_review, new_state)) = predictor.predict_from_records(
                    &card.review_records,
                    None,
                    0.9
                ) {
                    card.prediction = Some(Prediction {
                        algorithm: "fsrs".to_string(),
                        next_review,
                        fsrs_state_bytes: FsrsPredictor::memory_state_to_bytes(&new_state),
                        preset_used: "default".to_string(),
                    });
                }
            }
            
            let _ = self.data_fs.save_card(card_path, &card);
        }
        
        self.timer_tab.link_mode = false;
    }
    Task::none()
}
```

### Step 4: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 6: 实现计时器界面布局

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 修改 Timer Tab 界面

```rust
// src/app/mod.rs - view 方法中

TabId::Timer => {
    let timer_display = TimerDisplay::view(
        &self.timer_tab.state,
        self.timer_tab.elapsed_ms,
        self.timer_tab.current_card.as_deref(),
    );
    
    let content = if self.timer_tab.link_mode {
        // 链接模态框
        LinkTimerModal::view(&LinkTimerModal {
            duration_ms: self.timer_tab.elapsed_ms,
            card_path: self.timer_tab.card_path_input.clone(),
            card_dropdown: self.timer_tab.card_dropdown.clone(),
            selected_card: self.timer_tab.selected_card.clone(),
            memory_quality: self.timer_tab.memory_quality.clone(),
        })
    } else {
        // 正常计时器界面
        timer_display
    };
    
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
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

## Task 7: 实现快捷计时功能

**Files:**
- Modify: `src/app/mod.rs`
- Modify: `src/app/category_tab.rs`

### Step 1: 分类树添加快捷计时入口

```rust
// src/app/category_tab.rs - 在分类树右键菜单或操作按钮中添加

fn card_action_buttons(path: &str) -> Element<Message> {
    row![
        button(text("开始计时").color(iced::Color::WHITE))
            .on_press(Message::QuickTimerStart(path.to_string()))
            .style(|_, _| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.3, 0.6, 0.4).into()),
                text_color: iced::Color::WHITE,
                ..Default::default()
            }),
        // ... 其他按钮
    ]
    .spacing(8)
    .into()
}
```

### Step 2: 实现 QuickTimerStart 消息处理

```rust
// src/app/mod.rs - update 方法中

Message::QuickTimerStart(path) => {
    self.timer_tab.current_card = Some(path);
    self.active_tab = TabId::Timer;
    
    // 自动开始计时
    let _ = self.timer_manager.start();
    self.timer_tab.state = TimerState::Running {
        started_at: Utc::now(),
        accumulated: 0,
    };
    
    Task::none()
}
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 8: 实现计时记录管理

**Files:**
- Modify: `src/app/mod.rs`
- Modify: `src/data/fs.rs`

### Step 1: 扩展 DataFs 支持计时记录

```rust
// src/data/fs.rs

impl DataFs {
    pub fn save_timer_record(&self, record: &TimerRecord) -> Result<(), DataError> {
        let path = self.data_dir.join("timers").join("history.json");
        let mut records = self.load_timer_history()?;
        records.push(record.clone());
        
        let json = serde_json::to_string_pretty(&records)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load_timer_history(&self) -> Result<Vec<TimerRecord>, DataError> {
        let path = self.data_dir.join("timers").join("history.json");
        if !path.exists() {
            return Ok(vec![]);
        }
        
        let content = std::fs::read_to_string(path)?;
        let records: Vec<TimerRecord> = serde_json::from_str(&content)?;
        Ok(records)
    }
    
    pub fn delete_timer_record(&self, id: uuid::Uuid) -> Result<(), DataError> {
        let path = self.data_dir.join("timers").join("history.json");
        let mut records = self.load_timer_history()?;
        records.retain(|r| r.id != id);
        
        let json = serde_json::to_string_pretty(&records)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

### Step 2: 实现计时记录消息处理

```rust
// src/app/mod.rs - update 方法中

Message::TimerHistoryShow => {
    self.timer_tab.show_history = true;
    
    // 加载计时记录
    let data_fs = Arc::new(self.data_fs.clone());
    Task::future(async move {
        let records = data_fs.load_timer_history().unwrap_or_default();
        Message::TimerHistoryLoaded(records)
    })
}

Message::TimerHistoryLoaded(records) => {
    self.timer_tab.timer_history = records;
    Task::none()
}

Message::TimerHistoryDelete(id) => {
    let _ = self.data_fs.delete_timer_record(id);
    
    // 重新加载
    let data_fs = Arc::new(self.data_fs.clone());
    Task::future(async move {
        let records = data_fs.load_timer_history().unwrap_or_default();
        Message::TimerHistoryLoaded(records)
    })
}
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 9: 测试验证

### Step 1: 构建

```bash
cargo build --release
```

Expected: 编译成功

### Step 2: 运行测试

```bash
cargo test --release
```

Expected: 所有测试通过

### Step 3: 运行 GUI

```bash
TMD_DATA_DIR=/tmp/tmd-test-data cargo run --bin tmd-gui --release
```

Manual Test Checklist:
- [ ] 计时器时钟显示占窗口高度1/3，宽度2/3
- [ ] 从卡片跳转时显示卡片路径（无"当前学习："前缀）
- [ ] 直接进入时不显示路径信息
- [ ] 计时停止后弹出链接模态框
- [ ] 模态框支持手动输入和下拉选择卡片路径
- [ ] 支持创建新卡片
- [ ] 可以选择记忆质量
- [ ] 保存后自动触发 FSRS 重新预测
- [ ] 从分类树可以快捷启动计时
- [ ] 从复习看板可以快捷启动计时
- [ ] "计时记录"界面显示完整计时记录列表
- [ ] 计时记录支持编辑（时长、时间）
- [ ] 计时记录支持硬删除
- [ ] 计时记录支持链接到卡片

---

## 验收标准

- [ ] TimerTabState 包含所有新字段
- [ ] LinkTimerModal 组件实现
- [ ] TimerDisplay 增强显示
- [ ] 计时停止后弹出链接模态框
- [ ] 支持手动输入和下拉选择卡片路径
- [ ] 支持创建新卡片
- [ ] 支持选择记忆质量
- [ ] 保存后触发 FSRS 重新预测
- [ ] 快捷计时功能实现
- [ ] 计时记录管理功能实现
- [ ] 测试通过

---

## 时间估计

| 任务 | 时间 |
|------|------|
| Task 1: 扩展 TimerTabState | 10 分钟 |
| Task 2: 新增消息类型 | 5 分钟 |
| Task 3: 创建 LinkTimerModal | 30 分钟 |
| Task 4: 增强 TimerDisplay | 20 分钟 |
| Task 5: 实现消息处理 | 30 分钟 |
| Task 6: 实现计时器界面 | 20 分钟 |
| Task 7: 实现快捷计时 | 15 分钟 |
| Task 8: 实现计时记录管理 | 30 分钟 |
| Task 9: 测试验证 | 20 分钟 |
| **总计** | **约 3 小时** |