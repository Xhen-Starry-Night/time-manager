# GUI 模块 Task 1-2 实现计划

> **日期**: 2026-06-13
> **状态**: 实现计划
> **基于**: gui-task1-2-design.md

---

## Phase 1: Modal 基础设施

### Task 1.1: 实现 Modal 组件

**文件**: `src/gui/components/modal.rs`

- [ ] **Step 1: 创建 Modal 组件基础结构**

```rust
use iced::widget::{column, row, text, button, container, Space};
use iced::{Element, Length, Color};

pub struct ModalView;

impl ModalView {
    pub fn view<'a, Message: Clone + 'a>(
        title: &str,
        content: Element<'a, Message>,
        on_confirm: Message,
        on_cancel: Message,
    ) -> Element<'a, Message> {
        // Header with title and close button
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
        
        // Footer with cancel and confirm buttons
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
        
        // Dialog container
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
        
        // Overlay with centered dialog
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

- [ ] **Step 2: 运行 cargo build 验证编译**

```bash
cargo build
```

Expected: 成功编译

- [ ] **Step 3: Commit**

```bash
git add src/gui/components/modal.rs
git commit -m "feat: implement Modal component with overlay and buttons"
```

---

### Task 1.2: 扩展消息类型

**文件**: `src/gui/messages.rs`

- [ ] **Step 1: 添加待办相关消息**

找到 `pub enum Message` 块，添加：

```rust
pub enum Message {
    // ... 现有消息 ...
    
    // 待办相关
    NewTodoInputChanged(String),
    AddTodo,
    ToggleTodo(uuid::Uuid),
    DeleteTodo(uuid::Uuid),
    ConvertTodoToSchedule(uuid::Uuid),
}
```

- [ ] **Step 2: 运行 cargo build 验证编译**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/gui/messages.rs
git commit -m "feat: add todo-related messages (input, add, toggle, delete, convert)"
```

---

### Task 1.3: App 集成 Modal 渲染

**文件**: `src/app/mod.rs`

- [ ] **Step 1: 在 view 方法末尾添加 Modal 渲染**

找到 `fn view(&self) -> Element<Message>` 方法，在最后的 `.into()` 之前添加：

```rust
fn view(&self) -> Element<Message> {
    // ... 现有的 content 逻辑 ...
    
    let base_view = column![tabs, rule::horizontal(1.0), content]
        .width(Length::Fill)
        .height(Length::Fill);
    
    // Modal overlay
    if let Some(ref modal) = self.modal {
        let modal_content = match modal {
            Modal::ConfirmDelete { item } => {
                column![
                    text(format!("确认删除 \"{}\"?", item))
                        .size(16)
                        .color(Color::WHITE),
                ]
                .spacing(8)
                .into()
            }
            Modal::Error { message } => {
                column![
                    text("错误").size(16).color(Color::from_rgb(0.9, 0.3, 0.2)),
                    text(message.clone()).color(Color::WHITE),
                ]
                .spacing(8)
                .into()
            }
            _ => text("开发中").into()
        };
        
        crate::gui::components::ModalView::view(
            "确认",
            modal_content,
            Message::ModalConfirm,
            Message::ModalClose,
        )
    } else {
        base_view.into()
    }
}
```

- [ ] **Step 2: 添加 ModalConfirm 消息处理**

找到 `fn update(&mut self, message: Message) -> Task<Message>` 方法，在 `ModalClose` 处理后添加：

```rust
Message::ModalConfirm => {
    // 具体逻辑在各个功能中实现
    self.modal = None;
    Task::none()
}
```

- [ ] **Step 3: 运行 cargo build 验证编译**

```bash
cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/app/mod.rs
git commit -m "feat: integrate Modal rendering in App view"
```

---

## Phase 2: 待办输入框 + 添加功能

### Task 2.1: 实现输入框和添加按钮

**文件**: `src/app/mod.rs`

- [ ] **Step 1: 修改 TabId::Todo 分支**

找到 `TabId::Todo =>` 分支，替换为：

```rust
TabId::Todo => {
    // Input and add button
    let input = iced::widget::text_input(
        "输入待办内容...",
        &self.todo_tab.new_todo_input,
    )
    .on_input(Message::NewTodoInputChanged)
    .on_submit(Message::AddTodo)
    .width(Length::Fixed(400.0))
    .style(|_| iced::widget::text_input::Style {
        background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
        text_color: iced::Color::WHITE,
        ..Default::default()
    });
    
    let add_button = button(text("添加").color(iced::Color::WHITE))
        .on_press(Message::AddTodo)
        .style(|_| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
            text_color: iced::Color::WHITE,
            ..Default::default()
        });
    
    let header = row![input, add_button]
        .spacing(12)
        .padding(8);
    
    // Todo list
    let todo_list = if self.todo_tab.todos.is_empty() {
        container(
            text("暂无待办事项")
                .size(16)
                .color(iced::Color::WHITE)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    } else {
        container(
            scrollable(
                column(
                    self.todo_tab.todos.iter().map(|todo| {
                        self.render_todo_item(todo)
                    })
                )
                .spacing(4)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
    };
    
    container(
        column![
            header,
            rule::horizontal(1.0),
            todo_list,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
        ..Default::default()
    })
    .into()
}
```

- [ ] **Step 2: 添加 render_todo_item 辅助方法**

在 `impl App` 块中添加：

```rust
fn render_todo_item(&self, todo: &Todo) -> Element<Message> {
    let checkbox_text = if todo.completed { "☑" } else { "☐" };
    let text_color = if todo.completed {
        iced::Color::from_rgb(0.5, 0.5, 0.5)
    } else {
        iced::Color::WHITE
    };
    
    row![
        button(text(checkbox_text).color(iced::Color::WHITE))
            .on_press(Message::ToggleTodo(todo.id))
            .style(|_| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
                text_color: iced::Color::WHITE,
                ..Default::default()
            }),
        text(&todo.content).color(text_color),
        iced::widget::Space::with_width(Length::Fill),
        button(text("转日程").color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
            .on_press(Message::ConvertTodoToSchedule(todo.id))
            .style(|_| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
                text_color: iced::Color::from_rgb(0.6, 0.6, 0.6),
                ..Default::default()
            }),
        button(text("删除").color(iced::Color::from_rgb(0.9, 0.3, 0.2)))
            .on_press(Message::DeleteTodo(todo.id))
            .style(|_| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
                text_color: iced::Color::from_rgb(0.9, 0.3, 0.2),
                ..Default::default()
            }),
    ]
    .spacing(8)
    .padding(8)
    .width(Length::Fill)
    .into()
}
```

- [ ] **Step 3: 添加消息处理**

在 `update` 方法中添加：

```rust
Message::NewTodoInputChanged(text) => {
    self.todo_tab.new_todo_input = text;
    Task::none()
}

Message::AddTodo => {
    let content = self.todo_tab.new_todo_input.trim();
    if !content.is_empty() {
        let todo = Todo {
            id: uuid::Uuid::new_v4(),
            content: content.to_string(),
            completed: false,
            created_at: chrono::Utc::now(),
        };
        
        match self.data_fs.save_todo(&todo) {
            Ok(_) => {
                self.todo_tab.todos.push(todo);
                self.todo_tab.todos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                self.todo_tab.new_todo_input.clear();
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }
    Task::none()
}

Message::ToggleTodo(id) => {
    if let Some(todo) = self.todo_tab.todos.iter_mut().find(|t| t.id == id) {
        todo.completed = !todo.completed;
        if let Err(e) = self.data_fs.save_todo(todo) {
            self.error_message = Some(e.to_string());
        }
    }
    Task::none()
}

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

Message::ConvertTodoToSchedule(id) => {
    if let Some(todo) = self.todo_tab.todos.iter().find(|t| t.id == id) {
        let now = chrono::Utc::now();
        let ics_content = format!(
            "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nSUMMARY:{}\nDTSTART:{}\nDTEND:{}\nEND:VEVENT\nEND:VCALENDAR",
            todo.content,
            now.format("%Y%m%dT%H%M%SZ"),
            (now + chrono::Duration::hours(1)).format("%Y%m%dT%H%M%SZ"),
        );
        
        let schedule_id = uuid::Uuid::new_v4();
        match self.data_fs.save_schedule(&schedule_id, &ics_content) {
            Ok(_) => {
                let _ = self.data_fs.delete_todo(&id);
                self.todo_tab.todos.retain(|t| t.id != id);
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

- [ ] **Step 4: 添加 delete_todo 方法到 DataFs**

**文件**: `src/data/fs.rs`

在文件末尾 `}` 之前添加：

```rust
pub fn delete_todo(&self, id: &uuid::Uuid) -> Result<()> {
    let file_path = self.data_dir.join("todos").join(format!("{}.json", id));
    std::fs::remove_file(&file_path).map_err(|e| DataError::Io(e.to_string()))
}
```

- [ ] **Step 5: 运行 cargo build 验证编译**

```bash
cargo build
```

Expected: 成功编译

- [ ] **Step 6: 运行测试**

```bash
cargo test
```

Expected: 所有测试通过

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: implement todo CRUD (input, add, toggle, delete, convert to schedule)

- Add input field and add button for creating todos
- Add checkbox to toggle completion status
- Add delete button for each todo item
- Add convert to schedule button
- Gray out completed todos
- Integrate with DataFs for persistence"
```

---

## 验收测试

### 测试准备

```bash
# 确保测试数据目录存在
mkdir -p /tmp/tmd-gui-test/todos
mkdir -p /tmp/tmd-gui-test/schedules
```

### 测试用例

#### TC1: 添加待办
1. 启动 GUI: `TMD_DATA_DIR=/tmp/tmd-gui-test ./target/debug/tmd-gui`
2. 切换到"待办" Tab
3. 在输入框输入"测试待办1"
4. 点击"添加"按钮
5. **预期**: 列表中出现"测试待办1"

#### TC2: 完成待办
1. 点击"测试待办1"前的复选框
2. **预期**: 
   - 复选框变为 ☑
   - 文本变灰

#### TC3: 取消完成
1. 再次点击复选框
2. **预期**:
   - 复选框变为 ☐
   - 文本变白

#### TC4: 删除待办
1. 点击"测试待办1"的"删除"按钮
2. **预期**: 项目从列表消失

#### TC5: 转日程
1. 添加新待办"测试转日程"
2. 点击"转日程"按钮
3. **预期**:
   - 待办从列表消失
   - 切换到日程 Tab 可见新日程

#### TC6: 空输入
1. 输入框留空，点击"添加"
2. **预期**: 不添加任何项目

#### TC7: 回车提交
1. 输入"回车测试"
2. 按回车键
3. **预期**: 添加成功

---

## 文件变更清单

### 修改文件
- `src/gui/components/modal.rs` - Modal 组件
- `src/gui/messages.rs` - 新增消息
- `src/app/mod.rs` - 界面实现 + 消息处理
- `src/data/fs.rs` - delete_todo 方法

### 无新增文件

---

## 时间估计

| 任务 | 时间 |
|------|------|
| Task 1.1: Modal 组件 | 30 分钟 |
| Task 1.2: 消息扩展 | 15 分钟 |
| Task 1.3: App 集成 | 15 分钟 |
| Task 2.1: 输入框+添加 | 40 分钟 |
| Task 2.2: 复选框 | 20 分钟 |
| Task 2.3: 删除 | 20 分钟 |
| Task 2.4: 转日程 | 30 分钟 |
| 测试验证 | 20 分钟 |
| **总计** | **约 3 小时** |
