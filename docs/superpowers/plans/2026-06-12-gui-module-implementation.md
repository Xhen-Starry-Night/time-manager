# Time Manager GUI 模块实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Time Manager 的 GUI 界面，提供分类树浏览、复习看板、计时器、日程待办管理等功能。

**Architecture:** 使用 Iced 0.14 框架，采用响应式架构。状态分离设计，GUI 状态与业务数据分离。每个 Tab 对应独立模块，复用现有 DataFs 和数据模型。

**Tech Stack:** Rust, Iced 0.14, tokio, chrono, serde_json

---

## Phase 1: 基础框架

### Task 1.1: 添加 Iced 依赖

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: 添加 Iced 和 tokio 依赖**

```toml
[dependencies]
# ... 现有依赖 ...
iced = { version = "0.14", features = ["tokio"] }
tokio = { version = "1", features = ["rt-multi-thread"] }
```

- [ ] **Step 2: 运行 cargo build 验证依赖**

Run: `cargo build`
Expected: 成功编译，无错误

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "feat: add iced and tokio dependencies for GUI"
```

### Task 1.2: 创建 GUI 模块结构

**Files:**
- Create: `src/gui/mod.rs`
- Create: `src/gui/tabs/mod.rs`
- Create: `src/gui/components/mod.rs`
- Create: `src/gui/styles/mod.rs`
- Create: `src/gui/messages.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: 创建 gui 模块目录**

Run: `mkdir -p src/gui/tabs src/gui/components src/gui/styles`

- [ ] **Step 2: 创建 src/gui/messages.rs**

```rust
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use uuid::Uuid;

use crate::data::{DataError, DataFs};
use crate::data::models::{Card, MemoryQuality, Preset, Todo};
use crate::timer::TimerState;

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(TabId),
    
    DataLoaded(Result<DataSnapshot, DataError>),
    CardsLoaded(Result<Vec<(String, Card)>, DataError>),
    PresetsLoaded(Result<Vec<Preset>, DataError>),
    TodosLoaded(Result<Vec<Todo>, DataError>),
    
    TimerStarted,
    TimerPaused,
    TimerStopped(Result<PathBuf, String>),
    TimerTick(i64),
    
    CategorySelected(String),
    CardSelected(String),
    SearchChanged(String),
    
    ButtonPressed(ButtonId),
    InputChanged(String),
    
    ModalOpen(Modal),
    ModalClose,
    ModalConfirm,
    
    Error(String),
    ClearError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    Category,
    Review,
    Timer,
    Schedule,
    Todo,
    Preset,
    Settings,
}

#[derive(Debug, Clone)]
pub enum ButtonId {
    StartTimer,
    PauseTimer,
    StopTimer,
    NewCard,
    NewTodo,
    NewSchedule,
    NewPreset,
    ImportObsidian,
    TrainPreset,
    Delete,
}

#[derive(Debug, Clone)]
pub enum Modal {
    NewCard { path: String },
    LinkTimer { timer_path: PathBuf, duration_ms: i64 },
    NewTodo,
    NewSchedule,
    PresetDetail { name: String },
    ConfirmDelete { item: String },
    Error { message: String },
}

#[derive(Debug, Clone)]
pub struct DataSnapshot {
    pub trees: Vec<String>,
    pub cards: Vec<(String, Card)>,
    pub presets: Vec<Preset>,
    pub todos: Vec<Todo>,
}
```

- [ ] **Step 3: 创建 src/gui/tabs/mod.rs**

```rust
pub mod category_tab;
pub mod review_tab;
pub mod timer_tab;
pub mod schedule_tab;
pub mod todo_tab;
pub mod preset_tab;
pub mod settings_tab;

pub use category_tab::CategoryTab;
pub use review_tab::ReviewTab;
pub use timer_tab::TimerTab;
pub use schedule_tab::ScheduleTab;
pub use todo_tab::TodoTab;
pub use preset_tab::PresetTab;
pub use settings_tab::SettingsTab;
```

- [ ] **Step 4: 创建 src/gui/components/mod.rs**

```rust
pub mod tree_view;
pub mod card_detail;
pub mod timer_display;
pub mod urgency_badge;
pub mod modal;

pub use tree_view::TreeView;
pub use card_detail::CardDetail;
pub use timer_display::TimerDisplay;
pub use urgency_badge::UrgencyBadge;
pub use modal::ModalView;
```

- [ ] **Step 5: 创建 src/gui/styles/mod.rs**

```rust
pub mod theme;

pub use theme::Theme;
```

- [ ] **Step 6: 创建 src/gui/styles/theme.rs**

```rust
use iced::Color;

pub struct Theme {
    pub urgency_expired: Color,
    pub urgency_today: Color,
    pub urgency_soon: Color,
    pub urgency_later: Color,
    
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_secondary: Color,
    
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            urgency_expired: Color::from_rgb(0.906, 0.298, 0.235),
            urgency_today: Color::from_rgb(0.953, 0.612, 0.071),
            urgency_soon: Color::from_rgb(0.153, 0.682, 0.376),
            urgency_later: Color::from_rgb(0.584, 0.647, 0.651),
            
            primary: Color::from_rgb(0.204, 0.596, 0.859),
            secondary: Color::from_rgb(0.180, 0.800, 0.443),
            background: Color::from_rgb(1.0, 1.0, 1.0),
            surface: Color::from_rgb(0.961, 0.961, 0.961),
            text: Color::from_rgb(0.173, 0.243, 0.314),
            text_secondary: Color::from_rgb(0.4, 0.4, 0.4),
            
            success: Color::from_rgb(0.153, 0.682, 0.376),
            warning: Color::from_rgb(0.953, 0.612, 0.071),
            error: Color::from_rgb(0.906, 0.298, 0.235),
        }
    }
}

impl Theme {
    pub fn urgency_color(urgency: u32) -> Color {
        match urgency {
            3 => Self::default().urgency_expired,
            2 => Self::default().urgency_today,
            1 => Self::default().urgency_soon,
            _ => Self::default().urgency_later,
        }
    }
}
```

- [ ] **Step 7: 创建 src/gui/mod.rs**

```rust
pub mod tabs;
pub mod components;
pub mod styles;
pub mod messages;

pub use messages::{Message, TabId, ButtonId, Modal, DataSnapshot};
pub use styles::Theme;
```

- [ ] **Step 8: 修改 src/lib.rs 添加 gui 模块**

```rust
pub mod cli;
pub mod data;
pub mod fsrs;
pub mod gui;
pub mod obsidian;
pub mod timer;
pub mod training;

pub use cli::Cli;
```

- [ ] **Step 9: 运行 cargo build 验证编译**

Run: `cargo build`
Expected: 成功编译

- [ ] **Step 10: Commit**

```bash
git add src/gui/ src/lib.rs
git commit -m "feat: create GUI module structure with messages, theme, and empty tabs"
```

### Task 1.3: 创建主应用结构

**Files:**
- Create: `src/app.rs`
- Create: `src/main.rs` (GUI 入口)
- Modify: `Cargo.toml` (添加 binary)

- [ ] **Step 1: 创建 src/app.rs**

```rust
use std::path::PathBuf;

use iced::{Application, Command, Element, Settings, Subscription, Theme as IcedTheme};

use crate::data::DataFs;
use crate::gui::{Message, TabId, Modal};
use crate::timer::TimerManager;

mod category_tab;
mod review_tab;
mod timer_tab;
mod schedule_tab;
mod todo_tab;
mod preset_tab;
mod settings_tab;

use category_tab::CategoryTabState;
use review_tab::ReviewTabState;
use timer_tab::TimerTabState;
use schedule_tab::ScheduleTabState;
use todo_tab::TodoTabState;
use preset_tab::PresetTabState;
use settings_tab::SettingsTabState;

pub struct App {
    active_tab: TabId,
    data_dir: PathBuf,
    data_fs: DataFs,
    
    category_tab: CategoryTabState,
    review_tab: ReviewTabState,
    timer_tab: TimerTabState,
    schedule_tab: ScheduleTabState,
    todo_tab: TodoTabState,
    preset_tab: PresetTabState,
    settings_tab: SettingsTabState,
    
    timer_manager: TimerManager,
    error_message: Option<String>,
    modal: Option<Modal>,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = IcedTheme;
    type Flags = PathBuf;

    fn new(data_dir: PathBuf) -> (Self, Command<Message>) {
        let data_fs = DataFs::init(data_dir.clone()).expect("Failed to init data dir");
        let timer_manager = TimerManager::new(data_dir.clone());
        
        (
            Self {
                active_tab: TabId::Category,
                data_dir: data_dir.clone(),
                data_fs,
                
                category_tab: CategoryTabState::default(),
                review_tab: ReviewTabState::default(),
                timer_tab: TimerTabState::new(timer_manager.get_state().clone()),
                schedule_tab: ScheduleTabState::default(),
                todo_tab: TodoTabState::default(),
                preset_tab: PresetTabState::default(),
                settings_tab: SettingsTabState::new(data_dir.clone()),
                
                timer_manager,
                error_message: None,
                modal: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Time Manager")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SwitchTab(tab_id) => {
                self.active_tab = tab_id;
                Command::none()
            }
            
            Message::Error(msg) => {
                self.error_message = Some(msg);
                Command::none()
            }
            
            Message::ClearError => {
                self.error_message = None;
                Command::none()
            }
            
            Message::ModalOpen(modal) => {
                self.modal = Some(modal);
                Command::none()
            }
            
            Message::ModalClose => {
                self.modal = None;
                Command::none()
            }
            
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        iced::widget::text("Time Manager GUI - Under Construction").into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

impl App {
    pub fn run(data_dir: PathBuf) -> iced::Result {
        Self::run(Settings::with_flags(data_dir))
    }
}
```

- [ ] **Step 2: 创建各 Tab 状态结构（占位）**

创建 `src/app/category_tab.rs`:
```rust
use crate::timer::TimerState;

#[derive(Default)]
pub struct CategoryTabState {
    pub tree_name: String,
    pub selected_path: Option<String>,
    pub search_query: String,
}
```

创建 `src/app/review_tab.rs`:
```rust
#[derive(Default)]
pub struct ReviewTabState {
    pub urgency_filter: Option<u32>,
    pub search_query: String,
    pub selected_card: Option<String>,
}
```

创建 `src/app/timer_tab.rs`:
```rust
use crate::timer::TimerState;
use crate::data::models::MemoryQuality;

pub struct TimerTabState {
    pub state: TimerState,
    pub elapsed_ms: i64,
    pub card_path_input: String,
    pub memory_quality: MemoryQuality,
}

impl TimerTabState {
    pub fn new(state: TimerState) -> Self {
        Self {
            state,
            elapsed_ms: 0,
            card_path_input: String::new(),
            memory_quality: MemoryQuality::Good,
        }
    }
}
```

创建 `src/app/schedule_tab.rs`:
```rust
#[derive(Default)]
pub struct ScheduleTabState {
    pub selected_schedule: Option<uuid::Uuid>,
}
```

创建 `src/app/todo_tab.rs`:
```rust
#[derive(Default)]
pub struct TodoTabState {
    pub new_todo_input: String,
    pub selected_todo: Option<uuid::Uuid>,
}
```

创建 `src/app/preset_tab.rs`:
```rust
#[derive(Default)]
pub struct PresetTabState {
    pub selected_preset: Option<String>,
}
```

创建 `src/app/settings_tab.rs`:
```rust
use std::path::PathBuf;

pub struct SettingsTabState {
    pub data_dir: PathBuf,
}

impl SettingsTabState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }
}
```

- [ ] **Step 3: 创建 src/main.rs**

```rust
use std::path::PathBuf;

fn main() -> iced::Result {
    let data_dir = directories::ProjectDirs::from("com", "time-manager", "time-manager")
        .map(|p| p.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./data"));
    
    time_manager::app::App::run(data_dir)
}
```

- [ ] **Step 4: 修改 Cargo.toml 添加 binary**

```toml
[[bin]]
name = "tmd-gui"
path = "src/main.rs"

[[bin]]
name = "tmd"
path = "src/bin/tmd.rs"
```

- [ ] **Step 5: 修改 src/lib.rs 导出 app**

```rust
pub mod app;
pub mod cli;
pub mod data;
pub mod fsrs;
pub mod gui;
pub mod obsidian;
pub mod timer;
pub mod training;

pub use cli::Cli;
```

- [ ] **Step 6: 运行 cargo build 验证编译**

Run: `cargo build`
Expected: 成功编译

- [ ] **Step 7: Commit**

```bash
git add src/app/ src/main.rs src/lib.rs Cargo.toml
git commit -m "feat: create main application structure with Iced"
```

### Task 1.4: 实现基础界面框架

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: 实现 Tab 导航栏组件**

修改 `src/app.rs` 添加 view 实现：

```rust
fn view(&self) -> Element<Message> {
    use iced::widget::{button, column, row, text, container, rule};
    use iced::{Alignment, Length};
    
    let tabs = row![
        tab_button("分类树", TabId::Category, self.active_tab),
        tab_button("复习看板", TabId::Review, self.active_tab),
        tab_button("计时器", TabId::Timer, self.active_tab),
        tab_button("日程", TabId::Schedule, self.active_tab),
        tab_button("待办", TabId::Todo, self.active_tab),
        tab_button("预设", TabId::Preset, self.active_tab),
        tab_button("设置", TabId::Settings, self.active_tab),
    ]
    .spacing(4)
    .padding(8);
    
    let content = container(
        text(format!("{:?} Tab - Under Construction", self.active_tab))
            .size(24)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y();
    
    column![tabs, rule::Rule::horizontal(1), content]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn tab_button(label: &str, tab_id: TabId, active_tab: TabId) -> Element<Message> {
    let is_active = tab_id == active_tab;
    
    button(text(label))
        .on_press(Message::SwitchTab(tab_id))
        .style(if is_active {
            iced::theme::Button::Primary
        } else {
            iced::theme::Button::Secondary
        })
        .into()
}
```

- [ ] **Step 2: 运行 cargo build 验证**

Run: `cargo build`
Expected: 成功编译

- [ ] **Step 3: 测试运行 GUI**

Run: `cargo run --bin tmd-gui 2>&1 | head -20`
Expected: 窗口打开（或显示平台相关输出）

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: implement basic tab navigation UI"
```

---

## Phase 2: 分类树界面

### Task 2.1: 实现树形视图组件

**Files:**
- Create: `src/gui/components/tree_view.rs`

- [ ] **Step 1: 创建树形视图组件**

```rust
use iced::widget::{column, row, text, button, container};
use iced::{Element, Length};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_card: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub enum TreeMessage {
    NodeSelected(String),
    NodeExpanded(String),
}

pub struct TreeView {
    expanded: HashMap<String, bool>,
}

impl Default for TreeView {
    fn default() -> Self {
        Self {
            expanded: HashMap::new(),
        }
    }
}

impl TreeView {
    pub fn view(&self, nodes: &[TreeNode], selected: Option<&str>) -> Element<String> {
        column(nodes.iter().map(|node| self.view_node(node, selected, 0)))
            .spacing(2)
            .into()
    }
    
    fn view_node(&self, node: &TreeNode, selected: Option<&str>, depth: usize) -> Element<String> {
        let indent = "  ".repeat(depth);
        let icon = if node.is_card { "📄" } else { "📁" };
        
        let is_selected = selected == Some(node.path.as_str());
        
        let content = row![
            text(format!("{}{} {}", indent, icon, node.name)),
        ]
        .spacing(4);
        
        let btn = button(content)
            .on_press(node.path.clone())
            .style(if is_selected {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Text
            })
            .width(Length::Fill);
        
        if node.is_card {
            btn.into()
        } else {
            column![
                btn,
                if self.expanded.get(&node.path).unwrap_or(&false) {
                    column(node.children.iter().map(|c| self.view_node(c, selected, depth + 1)))
                        .spacing(1)
                } else {
                    column![]
                }
            ]
            .into()
        }
    }
}
```

- [ ] **Step 2: 运行 cargo build 验证**

Run: `cargo build`
Expected: 成功编译

- [ ] **Step 3: Commit**

```bash
git add src/gui/components/tree_view.rs
git commit -m "feat: implement tree view component"
```

### Task 2.2: 实现分类树界面

**Files:**
- Modify: `src/app/category_tab.rs`
- Modify: `src/app.rs`

（此处省略详细步骤，遵循相同的 TDD 和提交模式）

---

## Phase 3: 复习看板界面

### Task 3.1: 实现紧迫度徽章组件

**Files:**
- Create: `src/gui/components/urgency_badge.rs`

- [ ] **Step 1: 创建紧迫度徽章组件**

```rust
use iced::widget::{container, text};
use iced::{Element, Length, Color};
use crate::gui::Theme;

pub struct UrgencyBadge;

impl UrgencyBadge {
    pub fn view(urgency: u32, label: &str) -> Element<'static, ()> {
        let color = Theme::urgency_color(urgency);
        
        container(text(label).color(Color::WHITE))
            .style(move |_: &iced::Theme| iced::widget::container::Appearance {
                background: Some(color.into()),
                border_radius: 4.0.into(),
                ..Default::default()
            })
            .padding([2, 8])
            .into()
    }
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo build`
Expected: 成功编译

- [ ] **Step 3: Commit**

```bash
git add src/gui/components/urgency_badge.rs
git commit -m "feat: implement urgency badge component"
```

---

## 测试检查点

完成 Phase 1-3 后，运行完整测试：

```bash
cargo test
cargo run --bin tmd-gui
```

---

## 实施计划总结

| Phase | 任务数 | 预计时间 |
|-------|--------|----------|
| Phase 1: 基础框架 | 4 | 2-3 小时 |
| Phase 2: 分类树界面 | 3 | 2 小时 |
| Phase 3: 复习看板界面 | 2 | 1.5 小时 |
| Phase 4: 计时器界面 | 3 | 2 小时 |
| Phase 5: 其他界面 | 4 | 2 小时 |

**总计**: 16 个任务，约 9.5 小时