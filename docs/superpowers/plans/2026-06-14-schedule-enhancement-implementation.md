# Phase 6 日程完善 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现日程标签页的完整 CRUD：结构化数据、ICS 解析/序列化、新建/编辑/删除 Modal、列表显示优化

**Architecture:** 新增 `Schedule` 结构体承载结构化数据；新增 `schedule.rs` 实现 ICS <-> Schedule 的双向转换；`DataFs` 适配 Schedule 类型；`ScheduleTabState` 管理表单状态；复用现有 `ModalView` 组件。

**Tech Stack:** Rust, Iced 0.14, chrono, uuid

---

## File Structure

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/data/models.rs` | 修改 | 新增 Schedule 结构体、RecurrenceRule 枚举 |
| `src/data/schedule.rs` | 新建 | parse_ics / format_ics |
| `src/data/mod.rs` | 修改 | 注册 schedule 模块 |
| `src/lib.rs` | 修改 | 注册 data::schedule |
| `src/data/fs.rs` | 修改 | list_schedules / save_schedule / delete_schedule 适配 Schedule |
| `src/gui/messages.rs` | 修改 | 新增日程相关 Message |
| `src/app/schedule_tab.rs` | 重写 | ScheduleTabState + ScheduleForm |
| `src/app/mod.rs` | 修改 | 日程视图 + 消息处理 + Modal 渲染 |
| `tests/data_tests.rs` | 修改 | 新增 parse_ics / format_ics 测试 |

---

### Task 1: 新增 Schedule 数据模型

**Files:**
- Modify: `src/data/models.rs:140` — 在 Todo struct 之后新增 Schedule

- [ ] **Step 1: 添加 RecurrenceRule 枚举和 Schedule 结构体**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecurrenceRule {
    None,
    Daily,
    Weekly,
    Monthly,
}

impl RecurrenceRule {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "无",
            Self::Daily => "每天",
            Self::Weekly => "每周",
            Self::Monthly => "每月",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "每天" => Self::Daily,
            "每周" => Self::Weekly,
            "每月" => Self::Monthly,
            _ => Self::None,
        }
    }

    pub fn to_rrule(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Daily => Some("FREQ=DAILY"),
            Self::Weekly => Some("FREQ=WEEKLY"),
            Self::Monthly => Some("FREQ=MONTHLY"),
        }
    }

    pub fn from_rrule(s: &str) -> Self {
        match s {
            "FREQ=DAILY" => Self::Daily,
            "FREQ=WEEKLY" => Self::Weekly,
            "FREQ=MONTHLY" => Self::Monthly,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: Uuid,
    pub summary: String,
    pub dtstart: DateTime<Utc>,
    pub dtend: DateTime<Utc>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub categories: Vec<String>,
    pub priority: Option<i32>,
    pub rrule: RecurrenceRule,
    pub reminder_minutes: Option<i32>,
}
```

- [ ] **Step 2: 编译验证**

Run: `cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/data/models.rs
git commit -m "feat: add Schedule struct and RecurrenceRule enum"
```

---

### Task 2: ICS 解析/序列化层

**Files:**
- Create: `src/data/schedule.rs`

- [ ] **Step 1: 实现 parse_ics**

```rust
use crate::data::models::{RecurrenceRule, Schedule};
use crate::data::{DataError, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub fn parse_ics(content: &str) -> Result<Schedule> {
    let lines: Vec<&str> = content.lines().collect();

    let id = lines.iter()
        .find(|l| l.starts_with("UID:"))
        .and_then(|l| l.strip_prefix("UID:"))
        .and_then(|s| s.split('@').next())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| DataError::InvalidData("Missing UID".into()))?;

    let summary = lines.iter()
        .find(|l| l.starts_with("SUMMARY:"))
        .and_then(|l| l.strip_prefix("SUMMARY:"))
        .unwrap_or("")
        .to_string();

    let dtstart = lines.iter()
        .find(|l| l.starts_with("DTSTART:"))
        .and_then(|l| l.strip_prefix("DTSTART:"))
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        .ok_or_else(|| DataError::InvalidData("Missing DTSTART".into()))?;

    let dtend = lines.iter()
        .find(|l| l.starts_with("DTEND:"))
        .and_then(|l| l.strip_prefix("DTEND:"))
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        .ok_or_else(|| DataError::InvalidData("Missing DTEND".into()))?;

    let description = lines.iter()
        .find(|l| l.starts_with("DESCRIPTION:"))
        .and_then(|l| l.strip_prefix("DESCRIPTION:"))
        .map(String::from);

    let location = lines.iter()
        .find(|l| l.starts_with("LOCATION:"))
        .and_then(|l| l.strip_prefix("LOCATION:"))
        .map(String::from);

    let categories = lines.iter()
        .find(|l| l.starts_with("CATEGORIES:"))
        .and_then(|l| l.strip_prefix("CATEGORIES:"))
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();

    let priority = lines.iter()
        .find(|l| l.starts_with("PRIORITY:"))
        .and_then(|l| l.strip_prefix("PRIORITY:"))
        .and_then(|s| s.parse::<i32>().ok());

    let rrule = lines.iter()
        .find(|l| l.starts_with("RRULE:"))
        .and_then(|l| l.strip_prefix("RRULE:"))
        .map(RecurrenceRule::from_rrule)
        .unwrap_or(RecurrenceRule::None);

    let reminder_minutes = lines.iter()
        .skip_while(|l| !l.starts_with("BEGIN:VALARM"))
        .skip(1)
        .take_while(|l| !l.starts_with("END:VALARM"))
        .find(|l| l.starts_with("TRIGGER:-PT"))
        .and_then(|l| l.strip_prefix("TRIGGER:-PT"))
        .and_then(|s| s.strip_suffix('M'))
        .and_then(|s| s.parse::<i32>().ok());

    Ok(Schedule {
        id, summary, dtstart, dtend,
        description, location, categories, priority,
        rrule, reminder_minutes,
    })
}
```

- [ ] **Step 2: 实现 format_ics**

```rust
pub fn format_ics(schedule: &Schedule) -> String {
    let mut ics = format!(
        "BEGIN:VCALENDAR\nVERSION:2.0\nPRODID:-//Time Manager//EN\nCALSCALE:GREGORIAN\nBEGIN:VEVENT\nUID:{}@time-manager\nDTSTART:{}\nDTEND:{}\nSUMMARY:{}\nCREATED:{}\nDTSTAMP:{}\n",
        schedule.id,
        schedule.dtstart.format("%Y%m%dT%H%M%SZ"),
        schedule.dtend.format("%Y%m%dT%H%M%SZ"),
        schedule.summary,
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
    );

    if let Some(ref desc) = schedule.description {
        ics.push_str(&format!("DESCRIPTION:{}\n", desc));
    }
    if let Some(ref loc) = schedule.location {
        ics.push_str(&format!("LOCATION:{}\n", loc));
    }
    if !schedule.categories.is_empty() {
        ics.push_str(&format!("CATEGORIES:{}\n", schedule.categories.join(",")));
    }
    if let Some(p) = schedule.priority {
        ics.push_str(&format!("PRIORITY:{}\n", p));
    }
    if let Some(rrule_str) = schedule.rrule.to_rrule() {
        ics.push_str(&format!("RRULE:{}\n", rrule_str));
    }
    if let Some(minutes) = schedule.reminder_minutes {
        ics.push_str(&format!(
            "BEGIN:VALARM\nACTION:DISPLAY\nDESCRIPTION:{}\nTRIGGER:-PT{}M\nEND:VALARM\n",
            schedule.summary, minutes
        ));
    }

    ics.push_str("END:VEVENT\nEND:VCALENDAR\n");
    ics
}
```

- [ ] **Step 3: 编译验证**

Run: `cargo check`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/data/schedule.rs
git commit -m "feat: add ICS parse/format for Schedule"
```

---

### Task 3: 注册 schedule 模块

**Files:**
- Modify: `src/data/mod.rs` — 注册 schedule 模块
- Modify: `src/lib.rs` — 注册 data::schedule 模块

- [ ] **Step 1: 更新 data/mod.rs**

```rust
pub mod error;
pub mod fs;
pub mod models;
pub mod schedule;

pub use error::{DataError, Result};
pub use fs::DataFs;
```

- [ ] **Step 2: 编译验证**

Run: `cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/data/mod.rs
git commit -m "chore: register schedule module"
```

---

### Task 4: 适配 DataFs 使用 Schedule 类型

**Files:**
- Modify: `src/data/fs.rs` — 替换 `Vec<(Uuid, String)>` 为 `Vec<Schedule>`

- [ ] **Step 1: 修改 list_schedules 方法**

```rust
pub fn list_schedules(&self) -> Result<Vec<Schedule>> {
    let schedules_dir = self.data_dir.join("schedules");
    if !schedules_dir.exists() {
        return Ok(Vec::new());
    }

    let mut schedules: Vec<Schedule> = WalkDir::new(&schedules_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "ics").unwrap_or(false))
        .filter_map(|e| {
            let content = std::fs::read_to_string(e.path()).ok()?;
            crate::data::schedule::parse_ics(&content).ok()
        })
        .collect();

    schedules.sort_by(|a, b| a.dtstart.cmp(&b.dtstart));
    Ok(schedules)
}
```

- [ ] **Step 2: 修改 save_schedule 方法**

```rust
pub fn save_schedule(&self, schedule: &Schedule) -> Result<()> {
    let file_path = self.data_dir.join("schedules").join(format!("{}.ics", schedule.id));
    let ics = crate::data::schedule::format_ics(schedule);
    std::fs::write(&file_path, ics).map_err(|e| DataError::Io(e.to_string()))?;
    Ok(())
}
```

- [ ] **Step 3: 修改 get_schedule 方法返回 Schedule**

```rust
pub fn get_schedule(&self, id: &Uuid) -> Result<Schedule> {
    let file_path = self.data_dir.join("schedules").join(format!("{}.ics", id));
    if !file_path.exists() {
        return Err(DataError::ScheduleNotFound(id.to_string()));
    }
    let content = std::fs::read_to_string(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
    crate::data::schedule::parse_ics(&content)
}
```

- [ ] **Step 4: 添加 delete_schedule 方法**

在 `get_schedule` 之后添加：

```rust
pub fn delete_schedule(&self, id: &Uuid) -> Result<()> {
    let file_path = self.data_dir.join("schedules").join(format!("{}.ics", id));
    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
    }
    Ok(())
}
```

- [ ] **Step 5: 移除旧的方法签名**

将旧的 `list_schedules`、`save_schedule`、`get_schedule` 方法替换为上面的实现。

- [ ] **Step 6: 编译验证**

Run: `cargo check`
Expected: 编译成功

- [ ] **Step 7: Commit**

```bash
git add src/data/fs.rs
git commit -m "feat: adapt DataFs schedule methods to use Schedule type"
```

---

### Task 5: 新增日程相关 Message

**Files:**
- Modify: `src/gui/messages.rs`

- [ ] **Step 1: 在 Message 枚举中添加日程相关变体**

在 `ClearError` 之前添加：

```rust
    // 日程
    ScheduleCreateOpen,
    ScheduleCreateConfirm,
    ScheduleEditOpen(Uuid),
    ScheduleEditConfirm,
    ScheduleDelete(Uuid),
    ScheduleFormSummaryChanged(String),
    ScheduleFormStartDateChanged(String),
    ScheduleFormStartTimeChanged(String),
    ScheduleFormEndDateChanged(String),
    ScheduleFormEndTimeChanged(String),
    ScheduleFormDescriptionChanged(String),
    ScheduleFormLocationChanged(String),
    ScheduleFormReminderChanged(String),
    ScheduleFormRruleChanged(crate::data::models::RecurrenceRule),
```

- [ ] **Step 2: 导入 RecurrenceRule**

在文件顶部添加：
```rust
use crate::data::models::RecurrenceRule;
```
（或在使用处用全路径引用）

- [ ] **Step 3: 编译验证**

Run: `cargo check`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/gui/messages.rs
git commit -m "feat: add schedule-related Message variants"
```

---

### Task 6: 重写 ScheduleTabState

**Files:**
- Modify: `src/app/schedule_tab.rs` — 完全重写

- [ ] **Step 1: 实现 ScheduleTabState + ScheduleForm**

```rust
use uuid::Uuid;
use crate::data::models::{Schedule, RecurrenceRule};

pub struct ScheduleForm {
    pub summary: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub description: String,
    pub location: String,
    pub reminder: String,
    pub rrule: RecurrenceRule,
}

impl Default for ScheduleForm {
    fn default() -> Self {
        Self {
            summary: String::new(),
            start_date: String::new(),
            start_time: String::new(),
            end_date: String::new(),
            end_time: String::new(),
            description: String::new(),
            location: String::new(),
            reminder: String::new(),
            rrule: RecurrenceRule::None,
        }
    }
}

impl ScheduleForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self) -> Result<Schedule, String> {
        if self.summary.trim().is_empty() {
            return Err("标题不能为空".to_string());
        }

        let start_date = self.start_date.trim();
        let start_time = self.start_time.trim();
        let end_date = self.end_date.trim();
        let end_time = self.end_time.trim();

        let start_str = format!("{}T{}Z", start_date, start_time);
        let end_str = format!("{}T{}Z", end_date, end_time);

        let dtstart = chrono::NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%dT%H:%MZ")
            .map_err(|e| format!("开始时间格式错误: {}", e))?;
        let dtend = chrono::NaiveDateTime::parse_from_str(&end_str, "%Y-%m-%dT%H:%MZ")
            .map_err(|e| format!("结束时间格式错误: {}", e))?;

        if dtend <= dtstart {
            return Err("结束时间必须晚于开始时间".to_string());
        }

        let reminder_minutes = if self.reminder.trim().is_empty() {
            None
        } else {
            Some(self.reminder.trim().parse::<i32>()
                .map_err(|_| "提醒时间必须为数字".to_string())?)
        };

        Ok(Schedule {
            id: Uuid::new_v4(),
            summary: self.summary.trim().to_string(),
            dtstart: chrono::DateTime::from_naive_utc_and_offset(dtstart, chrono::Utc),
            dtend: chrono::DateTime::from_naive_utc_and_offset(dtend, chrono::Utc),
            description: if self.description.trim().is_empty() { None } else { Some(self.description.trim().to_string()) },
            location: if self.location.trim().is_empty() { None } else { Some(self.location.trim().to_string()) },
            categories: Vec::new(),
            priority: None,
            rrule: self.rrule,
            reminder_minutes,
        })
    }

    pub fn load_from_schedule(&mut self, schedule: &Schedule) {
        self.summary = schedule.summary.clone();
        self.start_date = schedule.dtstart.format("%Y-%m-%d").to_string();
        self.start_time = schedule.dtstart.format("%H:%M").to_string();
        self.end_date = schedule.dtend.format("%Y-%m-%d").to_string();
        self.end_time = schedule.dtend.format("%H:%M").to_string();
        self.description = schedule.description.clone().unwrap_or_default();
        self.location = schedule.location.clone().unwrap_or_default();
        self.reminder = schedule.reminder_minutes.map(|m| m.to_string()).unwrap_or_default();
        self.rrule = schedule.rrule;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct ScheduleTabState {
    pub schedules: Vec<Schedule>,
    pub form: ScheduleForm,
    pub show_form: bool,
    pub editing_id: Option<Uuid>,
    pub form_error: Option<String>,
}

impl Default for ScheduleTabState {
    fn default() -> Self {
        Self {
            schedules: Vec::new(),
            form: ScheduleForm::new(),
            show_form: false,
            editing_id: None,
            form_error: None,
        }
    }
}
```

- [ ] **Step 2: 编译验证**

Run: `cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/app/schedule_tab.rs
git commit -m "feat: rewrite ScheduleTabState with form support"
```

---

### Task 7: 集成到 App（消息处理 + 视图）

**Files:**
- Modify: `src/app/mod.rs` — 在所有相关位置添加日程的 view 和 update 逻辑

- [ ] **Step 1: 在 DataLoaded 中适配新 schedules 类型**

找到 `self.schedule_tab.schedules = self.data_fs.list_schedules().unwrap_or_default();`（DataLoaded 中），确认它自动适配新返回类型。

- [ ] **Step 2: 添加日程消息处理（在 `update` 方法的 `_ => Task::none()` 之前）**

```rust
            Message::ScheduleCreateOpen => {
                self.schedule_tab.form.reset();
                self.schedule_tab.editing_id = None;
                self.schedule_tab.show_form = true;
                self.schedule_tab.form_error = None;
                self.modal = Some(Modal::NewSchedule);
                Task::none()
            }

            Message::ScheduleCreateConfirm => {
                match self.schedule_tab.form.validate() {
                    Ok(schedule) => {
                        match self.data_fs.save_schedule(&schedule) {
                            Ok(()) => {
                                self.schedule_tab.show_form = false;
                                self.modal = None;
                                self.schedule_tab.schedules = self.data_fs.list_schedules().unwrap_or_default();
                            }
                            Err(e) => {
                                self.schedule_tab.form_error = Some(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.schedule_tab.form_error = Some(e);
                    }
                }
                Task::none()
            }

            Message::ScheduleEditOpen(id) => {
                if let Some(schedule) = self.schedule_tab.schedules.iter().find(|s| s.id == id) {
                    self.schedule_tab.form.load_from_schedule(schedule);
                    self.schedule_tab.editing_id = Some(id);
                    self.schedule_tab.show_form = true;
                    self.schedule_tab.form_error = None;
                    self.modal = Some(Modal::NewSchedule);
                }
                Task::none()
            }

            Message::ScheduleEditConfirm => {
                match self.schedule_tab.form.validate() {
                    Ok(mut schedule) => {
                        if let Some(editing_id) = self.schedule_tab.editing_id {
                            schedule.id = editing_id;
                        }
                        match self.data_fs.save_schedule(&schedule) {
                            Ok(()) => {
                                self.schedule_tab.show_form = false;
                                self.modal = None;
                                self.schedule_tab.editing_id = None;
                                self.schedule_tab.schedules = self.data_fs.list_schedules().unwrap_or_default();
                            }
                            Err(e) => {
                                self.schedule_tab.form_error = Some(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.schedule_tab.form_error = Some(e);
                    }
                }
                Task::none()
            }

            Message::ScheduleDelete(id) => {
                match self.data_fs.delete_schedule(&id) {
                    Ok(()) => {
                        self.schedule_tab.schedules.retain(|s| s.id != id);
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }

            Message::ScheduleFormSummaryChanged(val) => {
                self.schedule_tab.form.summary = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormStartDateChanged(val) => {
                self.schedule_tab.form.start_date = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormStartTimeChanged(val) => {
                self.schedule_tab.form.start_time = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormEndDateChanged(val) => {
                self.schedule_tab.form.end_date = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormEndTimeChanged(val) => {
                self.schedule_tab.form.end_time = val;
                self.schedule_tab.form_error = None;
                Task::none()
            }
            Message::ScheduleFormDescriptionChanged(val) => {
                self.schedule_tab.form.description = val;
                Task::none()
            }
            Message::ScheduleFormLocationChanged(val) => {
                self.schedule_tab.form.location = val;
                Task::none()
            }
            Message::ScheduleFormReminderChanged(val) => {
                self.schedule_tab.form.reminder = val;
                Task::none()
            }
            Message::ScheduleFormRruleChanged(val) => {
                self.schedule_tab.form.rrule = val;
                Task::none()
            }
```

- [ ] **Step 3: 替换日程标签页视图内容**

将现有的 `TabId::Schedule => { ... }` 块替换为：

```rust
            TabId::Schedule => {
                let add_button = button(text("新建日程").color(iced::Color::WHITE))
                    .on_press(Message::ScheduleCreateOpen)
                    .style(|_, _| iced::widget::button::Style {
                        background: Some(iced::Color::from_rgb(0.2, 0.6, 0.86).into()),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    });

                let header = row![
                    text("日程").size(20).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    add_button,
                ]
                .padding(8);

                let schedule_list: Element<Message> = if self.schedule_tab.schedules.is_empty() {
                    container(
                        text("暂无日程安排")
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
                            self.schedule_tab.schedules.iter().map(|schedule| {
                                let start_str = format!("{} ({})",
                                    schedule.dtstart.format("%m/%d").to_string(),
                                    weekday_cn(schedule.dtstart.weekday().num_days_from_monday()));
                                let time_range = format!("{} - {}",
                                    schedule.dtstart.format("%H:%M"),
                                    schedule.dtend.format("%H:%M"));

                                let edit_btn = button(text("编辑").color(iced::Color::from_rgb(0.4, 0.7, 0.9)))
                                    .on_press(Message::ScheduleEditOpen(schedule.id))
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.25, 0.25, 0.25).into()),
                                        text_color: iced::Color::from_rgb(0.4, 0.7, 0.9),
                                        ..Default::default()
                                    });

                                let delete_btn = button(text("删除").color(iced::Color::from_rgb(0.9, 0.3, 0.2)))
                                    .on_press(Message::ScheduleDelete(schedule.id))
                                    .style(|_, _| iced::widget::button::Style {
                                        background: Some(iced::Color::from_rgb(0.25, 0.25, 0.25).into()),
                                        text_color: iced::Color::from_rgb(0.9, 0.3, 0.2),
                                        ..Default::default()
                                    });

                                let location_text = schedule.location.as_ref()
                                    .map(|loc| format!("地点: {}", loc))
                                    .unwrap_or_default();

                                container(
                                    row![
                                        column![
                                            row![
                                                text(start_str).color(iced::Color::from_rgb(0.7, 0.7, 0.7)).size(13),
                                                text("  ").into(),
                                                text(time_range).color(iced::Color::from_rgb(0.6, 0.6, 0.6)).size(13),
                                                text("  ").into(),
                                                text(&schedule.summary).color(iced::Color::WHITE).size(14),
                                            ],
                                            if !location_text.is_empty() {
                                                row![
                                                    text(location_text).color(iced::Color::from_rgb(0.5, 0.5, 0.5)).size(12),
                                                ].into()
                                            } else {
                                                row![].into()
                                            },
                                        ]
                                        .spacing(2),
                                        Space::new().width(Length::Fill),
                                        edit_btn,
                                        delete_btn,
                                    ]
                                    .spacing(8)
                                    .padding(8)
                                )
                                .style(|_| iced::widget::container::Style {
                                    background: Some(iced::Color::from_rgb(0.22, 0.22, 0.22).into()),
                                    ..Default::default()
                                })
                                .width(Length::Fill)
                                .into()
                            })
                        )
                        .spacing(4)
                    )
                    .into()
                };

                container(
                    column![
                        header,
                        rule::horizontal(1.0),
                        schedule_list,
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

- [ ] **Step 4: 添加 weekday_cn 辅助函数**

在文件底部添加：

```rust
fn weekday_cn(days_from_monday: u32) -> &'static str {
    match days_from_monday {
        0 => "一", 1 => "二", 2 => "三", 3 => "四",
        4 => "五", 5 => "六", 6 => "日",
        _ => "",
    }
}
```

- [ ] **Step 5: 在 Modal 枚举中添加 NewSchedule 变体**

在 `src/gui/messages.rs` 的 `Modal` 枚举中添加：

```rust
    NewSchedule,
```

- [ ] **Step 6: 在 App::view 的 Modal 渲染中添加 NewSchedule 处理**

在 `src/app/mod.rs` 的 modal 匹配中添加：

```rust
                Modal::NewSchedule => {
                    use crate::data::models::RecurrenceRule;
                    let form = &self.schedule_tab.form;
                    let rrule_options = vec!["无", "每天", "每周", "每月"];

                    let content = column![
                        row![
                            text("标题:").color(iced::Color::WHITE),
                            text_input("日程标题", &form.summary)
                                .on_input(Message::ScheduleFormSummaryChanged)
                                .width(Length::Fill),
                        ].spacing(8).padding(4),
                        row![
                            text("开始:").color(iced::Color::WHITE),
                            text_input("日期", &form.start_date)
                                .on_input(Message::ScheduleFormStartDateChanged)
                                .width(Length::Fixed(120.0)),
                            text(" ").into(),
                            text_input("时间", &form.start_time)
                                .on_input(Message::ScheduleFormStartTimeChanged)
                                .width(Length::Fixed(80.0)),
                        ].spacing(8).padding(4),
                        row![
                            text("结束:").color(iced::Color::WHITE),
                            text_input("日期", &form.end_date)
                                .on_input(Message::ScheduleFormEndDateChanged)
                                .width(Length::Fixed(120.0)),
                            text(" ").into(),
                            text_input("时间", &form.end_time)
                                .on_input(Message::ScheduleFormEndTimeChanged)
                                .width(Length::Fixed(80.0)),
                        ].spacing(8).padding(4),
                        row![
                            text("地点:").color(iced::Color::WHITE),
                            text_input("可选", &form.location)
                                .on_input(Message::ScheduleFormLocationChanged)
                                .width(Length::Fill),
                        ].spacing(8).padding(4),
                        row![
                            text("描述:").color(iced::Color::WHITE),
                            text_input("可选", &form.description)
                                .on_input(Message::ScheduleFormDescriptionChanged)
                                .width(Length::Fill),
                        ].spacing(8).padding(4),
                        row![
                            text("提醒:").color(iced::Color::WHITE),
                            text_input("分钟前", &form.reminder)
                                .on_input(Message::ScheduleFormReminderChanged)
                                .width(Length::Fixed(80.0)),
                        ].spacing(8).padding(4),
                        row![
                            text("重复:").color(iced::Color::WHITE),
                            pick_list(
                                rrule_options,
                                Some(form.rrule.as_str().to_string()),
                                |s| Message::ScheduleFormRruleChanged(
                                    RecurrenceRule::from_str(&s)
                                ),
                            ).width(Length::Fixed(100.0)),
                        ].spacing(8).padding(4),
                        if let Some(ref err) = self.schedule_tab.form_error {
                            row![
                                text(err).color(iced::Color::from_rgb(0.9, 0.3, 0.2)).size(13),
                            ].into()
                        } else {
                            row![].into()
                        },
                    ].spacing(4).padding(8);

                    let confirm_msg = if self.schedule_tab.editing_id.is_some() {
                        Message::ScheduleEditConfirm
                    } else {
                        Message::ScheduleCreateConfirm
                    };

                    (content, (confirm_msg, "保存", true))
                }
```

注意：上方的 `text(" ").into()` 中 `text` 需要引用或不使用，需要在实际编译中处理。更简单的做法是用 `Space::new().width(Length::Fixed(8.0))`。

- [ ] **Step 7: 在 Modal 的 title 匹配中添加 NewSchedule**

找到 `let modal_title = match modal {` 块，添加：

```rust
                Modal::NewSchedule => "新建日程",
```

- [ ] **Step 8: 编译验证**

Run: `cargo check`
Expected: 编译成功，无报错

- [ ] **Step 9: Commit**

```bash
git add src/app/mod.rs src/gui/messages.rs
git commit -m "feat: integrate schedule CRUD into App view and update"
```

---

### Task 8: 单元测试

**Files:**
- Modify: `tests/data_tests.rs` — 新增 parse_ics / format_ics 测试

- [ ] **Step 1: 添加 ICS 往返测试**

```rust
#[test]
fn test_schedule_ics_roundtrip() {
    use time_manager::data::schedule::{parse_ics, format_ics};
    use time_manager::data::models::{Schedule, RecurrenceRule};
    use chrono::{DateTime, Utc, TimeZone};
    use uuid::Uuid;

    let schedule = Schedule {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        summary: "测试日程".to_string(),
        dtstart: Utc.with_ymd_and_hms(2026, 6, 14, 14, 0, 0).unwrap(),
        dtend: Utc.with_ymd_and_hms(2026, 6, 14, 15, 0, 0).unwrap(),
        description: Some("描述内容".to_string()),
        location: Some("图书馆".to_string()),
        categories: vec!["学习".to_string(), "Rust".to_string()],
        priority: Some(5),
        rrule: RecurrenceRule::Weekly,
        reminder_minutes: Some(15),
    };

    let ics = format_ics(&schedule);
    let parsed = parse_ics(&ics).expect("Failed to parse generated ICS");

    assert_eq!(parsed.id, schedule.id);
    assert_eq!(parsed.summary, schedule.summary);
    assert_eq!(parsed.dtstart, schedule.dtstart);
    assert_eq!(parsed.dtend, schedule.dtend);
    assert_eq!(parsed.description, schedule.description);
    assert_eq!(parsed.location, schedule.location);
    assert_eq!(parsed.categories, schedule.categories);
    assert_eq!(parsed.priority, schedule.priority);
    assert_eq!(parsed.rrule, schedule.rrule);
    assert_eq!(parsed.reminder_minutes, schedule.reminder_minutes);
}
```

- [ ] **Step 2: 添加无效 ICS 解析测试**

```rust
#[test]
fn test_parse_invalid_ics() {
    use time_manager::data::schedule::parse_ics;

    assert!(parse_ics("").is_err());
    assert!(parse_ics("BEGIN:VCALENDAR\nEND:VCALENDAR").is_err());
    assert!(parse_ics("INVALID").is_err());
}
```

- [ ] **Step 3: 运行测试**

Run: `cargo test test_schedule_ics_roundtrip test_parse_invalid_ics -- --nocapture`
Expected: 全部通过

- [ ] **Step 4: Commit**

```bash
git add tests/data_tests.rs
git commit -m "test: add schedule ICS parse/format roundtrip tests"
```
