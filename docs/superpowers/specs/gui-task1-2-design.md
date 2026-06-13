# GUI 模块 Task 1-2 设计文档

> **日期**: 2026-06-13
> **状态**: 设计阶段
> **基于**: DESIGN_v3.md §4 GUI 设计 + gui-remaining-development-roadmap.md

---

## Task 1: Modal 基础设施

### 1.1 概述

**目标**: 实现可复用的模态对话框组件，为新建卡片、新建日程、计时链接等功能提供基础支持。

**复杂度**: 低
**预计时间**: 1 小时

### 1.2 组件设计

#### Modal 结构

```rust
// src/gui/components/modal.rs

pub struct ModalView;

pub enum ModalAction {
    Confirm,
    Cancel,
}

impl ModalView {
    /// 创建模态对话框
    /// 
    /// # 参数
    /// - title: 标题文本
    /// - content: 内容元素
    /// - on_confirm: 确认消息
    /// - on_cancel: 取消消息
    pub fn view<'a, Message: Clone + 'a>(
        title: &str,
        content: Element<'a, Message>,
        on_confirm: Message,
        on_cancel: Message,
    ) -> Element<'a, Message> {
        // 实现遮罩层 + 居中对话框
    }
}
```

#### Modal 消息

```rust
// src/gui/messages.rs (已有，需扩展)

pub enum Modal {
    NewCard { path: String },
    LinkTimer { timer_path: PathBuf, duration_ms: i64 },
    NewTodo,
    NewSchedule,
    PresetDetail { name: String },
    ConfirmDelete { item: String },
    Error { message: String },
}
```

### 1.3 视觉设计

```
┌────────────────────────────────────────────────────────────┐
│ ██████████████████████ 遮罩层 █████████████████████████████│
│                                                            │
│         ┌────────────────────────────────────┐            │
│         │ 标题                         [X]   │            │
│         ├────────────────────────────────────┤            │
│         │                                    │            │
│         │         内容区域                    │            │
│         │                                    │            │
│         ├────────────────────────────────────┤            │
│         │         [取消]    [确认]           │            │
│         └────────────────────────────────────┘            │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**样式规范**:
- 遮罩层：半透明黑色背景 (rgba(0, 0, 0, 0.5))
- 对话框：深灰色背景 (RGB 0.2, 0.2, 0.2)
- 标题：白色文本，18px
- 按钮：取消(灰色)、确认(蓝色)

### 1.4 实现细节

#### Task 1.1: Modal 组件实现

**文件**: `src/gui/components/modal.rs`

```rust
use iced::widget::{column, row, text, button, container, Space};
use iced::{Element, Length, Color, Alignment};

pub struct ModalView;

impl ModalView {
    pub fn view<'a, Message: Clone + 'a>(
        title: &str,
        content: Element<'a, Message>,
        on_confirm: Message,
        on_cancel: Message,
    ) -> Element<'a, Message> {
        let header = row![
            text(title).size(18).color(Color::WHITE),
            Space::with_width(Length::Fill),
            button(text("✕").color(Color::WHITE))
                .on_press(on_cancel.clone())
                .style(|_| iced::widget::button::Style {
                    background: Some(Color::TRANSPARENT.into()),
                    text_color: Color::from_rgb(0.6, 0.6, 0.6),
                    ..Default::default()
                }),
        ]
        .padding(16)
        .width(Length::Fill);
        
        let footer = row![
            Space::with_width(Length::Fill),
            button(text("取消").color(Color::WHITE))
                .on_press(on_cancel)
                .style(|_| iced::widget::button::Style {
                    background: Some(Color::from_rgb(0.3, 0.3, 0.3).into()),
                    text_color: Color::WHITE,
                    ..Default::default()
                }),
            button(text("确认").color(Color::WHITE))
                .on_press(on_confirm)
                .style(|_| iced::widget::button::Style {
                    background: Some(Color::from_rgb(0.2, 0.6, 0.86).into()),
                    text_color: Color::WHITE,
                    ..Default::default()
                }),
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
```

#### Task 1.2: App 集成 Modal

**文件**: `src/app/mod.rs`

在 `App::view` 方法末尾添加 Modal 渲染：

```rust
fn view(&self) -> Element<Message> {
    let base_view = /* 现有的 view 逻辑 */;
    
    if let Some(ref modal) = self.modal {
        let modal_content = match modal {
            Modal::NewCard { path } => {
                // Task 7 实现
                text(format!("新建卡片: {}", path)).into()
            }
            Modal::LinkTimer { timer_path, duration_ms } => {
                // Task 5 实现
                text(format!("链接计时器: {:?}", timer_path)).into()
            }
            Modal::ConfirmDelete { item } => {
                column![
                    text(format!("确认删除 {}?", item)).color(Color::WHITE),
                ].into()
            }
            _ => text("开发中").into()
        };
        
        ModalView::view(
            "确认",
            modal_content,
            Message::ModalConfirm,
            Message::ModalClose,
        )
    } else {
        base_view
    }
}
```

### 1.5 验收标准

- [ ] 点击遮罩层关闭 Modal
- [ ] 点击 X 按钮关闭 Modal
- [ ] 点击取消按钮关闭 Modal
- [ ] 点击确认按钮触发 ModalConfirm 消息
- [ ] Modal 居中显示
- [ ] 遮罩层半透明

---

## Task 2: 待办完整 CRUD

### 2.1 概述

**目标**: 实现待办的创建、完成状态切换、删除、转日程功能。

**复杂度**: 低（创建/删除/完成）、中（转日程）
**预计时间**: 2 小时

### 2.2 界面设计

#### 完整布局

```
┌────────────────────────────────────────────────────────────┐
│ 待办                     [输入待办内容...] [添加]          │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐│
│ │                                                         ││
│ │  ☑ 复习英语单词                    [转日程] [删除]      ││
│ │                                                         ││
│ │  ☐ 完成数学作业                    [转日程] [删除]      ││
│ │                                                         ││
│ │  ☐ 准备期末考试                    [转日程] [删除]      ││
│ │                                                         ││
│ └─────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────┘
```

**状态说明**:
- ☑ 已完成（文本灰色，可点击取消完成）
- ☐ 未完成（文本白色，可点击标记完成）

### 2.3 数据模型

#### Todo 结构（已有）

```rust
// src/data/models.rs

pub struct Todo {
    pub id: Uuid,
    pub content: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}
```

#### TodoTabState 扩展

```rust
// src/app/todo_tab.rs

pub struct TodoTabState {
    pub new_todo_input: String,      // 输入框内容
    pub selected_todo: Option<Uuid>, // 选中的待办
    pub todos: Vec<Todo>,            // 待办列表
}
```

### 2.4 消息设计

#### 新增消息

```rust
// src/gui/messages.rs

pub enum Message {
    // ... 现有消息 ...
    
    // 待办相关
    NewTodoInputChanged(String),          // 输入框内容变化
    AddTodo,                               // 添加待办
    ToggleTodo(Uuid),                      // 切换完成状态
    DeleteTodo(Uuid),                      // 删除待办
    ConvertTodoToSchedule(Uuid),           // 转日程
}
```

#### ButtonId 扩展

```rust
// src/gui/messages.rs

pub enum ButtonId {
    // ... 现有按钮 ...
    AddTodo,
    ToggleTodo(Uuid),
    DeleteTodo(Uuid),
    ConvertTodoToSchedule(Uuid),
}
```

### 2.5 实现细节

#### Task 2.1: 待办输入框 + 添加按钮

**文件**: `src/app/mod.rs`

```rust
TabId::Todo => {
    let input = text_input("输入待办内容...", &self.todo_tab.new_todo_input)
        .on_input(Message::NewTodoInputChanged)
        .on_submit(Message::AddTodo)
        .width(Length::Fixed(300.0))
        .style(|_| iced::widget::text_input::Style {
            background: Some(Color::from_rgb(0.3, 0.3, 0.3).into()),
            text_color: Color::WHITE,
            ..Default::default()
        });
    
    let add_button = button(text("添加").color(Color::WHITE))
        .on_press(Message::AddTodo)
        .style(|_| iced::widget::button::Style {
            background: Some(Color::from_rgb(0.2, 0.6, 0.86).into()),
            text_color: Color::WHITE,
            ..Default::default()
        });
    
    let header = row![input, add_button]
        .spacing(8)
        .padding(8);
    
    // ... 其余逻辑 ...
}
```

**消息处理**:

```rust
Message::NewTodoInputChanged(text) => {
    self.todo_tab.new_todo_input = text;
    Task::none()
}

Message::AddTodo => {
    let content = self.todo_tab.new_todo_input.trim();
    if !content.is_empty() {
        let todo = Todo {
            id: Uuid::new_v4(),
            content: content.to_string(),
            completed: false,
            created_at: Utc::now(),
        };
        
        match self.data_fs.save_todo(&todo) {
            Ok(_) => {
                self.todo_tab.todos.push(todo);
                self.todo_tab.new_todo_input.clear();
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }
    Task::none()
}
```

#### Task 2.2: 待办复选框（完成状态）

**文件**: `src/app/mod.rs`

```rust
fn render_todo_item(&self, todo: &Todo) -> Element<Message> {
    let checkbox_text = if todo.completed { "☑" } else { "☐" };
    let text_color = if todo.completed {
        Color::from_rgb(0.5, 0.5, 0.5)  // 已完成：灰色
    } else {
        Color::WHITE                      // 未完成：白色
    };
    
    let checkbox = button(text(checkbox_text).color(Color::WHITE))
        .on_press(Message::ToggleTodo(todo.id))
        .style(|_| iced::widget::button::Style {
            background: Some(Color::from_rgb(0.3, 0.3, 0.3).into()),
            text_color: Color::WHITE,
            ..Default::default()
        });
    
    row![
        checkbox,
        text(&todo.content).color(text_color),
        Space::with_width(Length::Fill),
        button(text("转日程").color(Color::from_rgb(0.6, 0.6, 0.6)))
            .on_press(Message::ConvertTodoToSchedule(todo.id)),
        button(text("删除").color(Color::from_rgb(0.9, 0.3, 0.2)))
            .on_press(Message::DeleteTodo(todo.id)),
    ]
    .spacing(8)
    .padding(8)
    .width(Length::Fill)
    .into()
}
```

**消息处理**:

```rust
Message::ToggleTodo(id) => {
    if let Some(todo) = self.todo_tab.todos.iter_mut().find(|t| t.id == id) {
        todo.completed = !todo.completed;
        if let Err(e) = self.data_fs.save_todo(todo) {
            self.error_message = Some(e.to_string());
        }
    }
    Task::none()
}
```

#### Task 2.3: 待办删除

**消息处理**:

```rust
Message::DeleteTodo(id) => {
    match self.data_fs.delete_todo(&id) {
        Ok(_) => {
            self.todo_tab.todos.retain(|t| t.id != id);
        }
        Err(e) => {
            self.error_message = Some(e.to_string());
        }
    }
    Task::none()
}
```

**DataFs 新增方法**:

```rust
// src/data/fs.rs

pub fn delete_todo(&self, id: &Uuid) -> Result<()> {
    let file_path = self.data_dir.join("todos").join(format!("{}.json", id));
    std::fs::remove_file(&file_path).map_err(|e| DataError::Io(e.to_string()))
}
```

#### Task 2.4: 待办转日程

**消息处理**:

```rust
Message::ConvertTodoToSchedule(id) => {
    if let Some(todo) = self.todo_tab.todos.iter().find(|t| t.id == id) {
        // 创建日程 ICS 内容
        let now = Utc::now();
        let ics_content = format!(
            "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nSUMMARY:{}\nDTSTART:{}\nDTEND:{}\nEND:VEVENT\nEND:VCALENDAR",
            todo.content,
            now.format("%Y%m%dT%H%M%SZ"),
            (now + chrono::Duration::hours(1)).format("%Y%m%dT%H%M%SZ"),
        );
        
        let schedule_id = Uuid::new_v4();
        match self.data_fs.save_schedule(&schedule_id, &ics_content) {
            Ok(_) => {
                // 删除待办
                let _ = self.data_fs.delete_todo(&id);
                self.todo_tab.todos.retain(|t| t.id != id);
                
                // 刷新日程列表
                self.schedule_tab.schedules = self.data_fs.list_schedules().unwrap_or_default();
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }
    Task::none()
}
```

### 2.6 验收标准

#### Task 2.1: 输入框 + 添加
- [ ] 输入框可输入文本
- [ ] 按回车键添加待办
- [ ] 点击添加按钮添加待办
- [ ] 空内容不添加
- [ ] 添加后清空输入框
- [ ] 新待办出现在列表顶部

#### Task 2.2: 复选框
- [ ] 未完成显示 ☐
- [ ] 已完成显示 ☑
- [ ] 点击切换状态
- [ ] 已完成文本变灰色
- [ ] 状态持久化到文件

#### Task 2.3: 删除
- [ ] 每项显示删除按钮
- [ ] 点击后从列表移除
- [ ] 文件被删除

#### Task 2.4: 转日程
- [ ] 每项显示转日程按钮
- [ ] 点击后创建日程（标题 = 待办内容）
- [ ] 待办从列表移除
- [ ] 日程列表更新

---

## 3. 依赖关系

```
Task 1 (Modal) ──┐
                  ├──> Task 7 (新建卡片)
                  ├──> Task 5 (计时链接)
                  └──> Task 6 (新建日程)

Task 2.1 (输入框+添加) ──> 无依赖
Task 2.2 (复选框) ──> 无依赖
Task 2.3 (删除) ──> 无依赖
Task 2.4 (转日程) ──> 依赖 DataFs.save_schedule
```

---

## 4. 测试数据准备

```bash
# 创建测试待办
./target/debug/tmd todo-create "测试待办1" --data-dir /tmp/tmd-gui-test
./target/debug/tmd todo-create "测试待办2" --data-dir /tmp/tmd-gui-test

# 验证数据
ls /tmp/tmd-gui-test/todos/
```

---

## 5. 文件清单

### 新增文件
- 无

### 修改文件
- `src/gui/components/modal.rs` - Modal 组件实现
- `src/gui/messages.rs` - 新增消息类型
- `src/app/mod.rs` - Tab 界面实现 + 消息处理
- `src/app/todo_tab.rs` - TodoTabState 扩展
- `src/data/fs.rs` - delete_todo 方法

---

## 6. 时间估计

| 任务 | 时间 |
|------|------|
| Task 1.1: Modal 组件 | 30 分钟 |
| Task 1.2: App 集成 | 30 分钟 |
| Task 2.1: 输入框+添加 | 30 分钟 |
| Task 2.2: 复选框 | 20 分钟 |
| Task 2.3: 删除 | 20 分钟 |
| Task 2.4: 转日程 | 40 分钟 |
| **总计** | **约 3 小时** |
