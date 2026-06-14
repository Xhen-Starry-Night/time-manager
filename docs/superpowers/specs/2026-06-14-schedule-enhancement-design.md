# GUI Phase 6 设计文档 - 日程完善

**日期**: 2026-06-14
**状态**: 设计阶段
**基于**: Phase 1-5 已完成实现

---

## 1. 概述

### 1.1 目标

完善日程界面，实现完整的日程管理功能：
- ICS 内容解析为结构化数据
- 日程列表显示优化（时间范围、标题、地点）
- 新建日程 Modal（完整字段支持）
- 日程删除功能

### 1.2 当前状态

Phase 5 已完成：
- 计时器实时显示与控制
- 计时结束后的卡片链接流程
- 树形选择器与输入框同步
- 创建目录/学习卡片支持

日程标签页当前状态：
- 基础列表显示（仅 DTSTART + SUMMARY）
- 使用原始 ICS 字符串存储，在 view 中内联解析
- 无新建功能
- 无删除功能
- `ScheduleTabState` 存储 `Vec<(Uuid, String)>`

### 1.3 Phase 6 新增功能

1. **Schedule 结构体**：解析 ICS 为结构化数据模型
2. **ICS 解析层**：parse_ics / format_ics 工具函数
3. **日程列表显示优化**：时间范围 + 标题 + 地点，按时间排序
4. **新建日程 Modal**：标题、起止时间、地点、描述、提醒、重复规则
5. **日程编辑功能**：点击编辑按钮，打开带现有数据的 Modal，修改后保存
6. **日程删除功能**

---

## 2. 数据模型

### 2.1 Schedule 结构体

新增 `src/data/models.rs`：

```rust
pub struct Schedule {
    pub id: Uuid,
    pub summary: String,
    pub dtstart: DateTime<Utc>,
    pub dtend: DateTime<Utc>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub categories: Vec<String>,
    pub priority: Option<i32>,
    pub rrule: Option<String>,
    pub reminder_minutes: Option<i32>,
}
```

**字段说明**：

| 字段 | 类型 | ICS 映射 | 说明 |
|------|------|----------|------|
| id | Uuid | UID | 日程唯一标识 |
| summary | String | SUMMARY | 日程标题 |
| dtstart | DateTime | DTSTART | 开始时间 |
| dtend | DateTime | DTEND | 结束时间 |
| description | Option\<String\> | DESCRIPTION | 详细描述 |
| location | Option\<String\> | LOCATION | 地点 |
| categories | Vec\<String\> | CATEGORIES | 分类标签 |
| priority | Option\<i32\> | PRIORITY | 优先级 1-9 |
| rrule | Option\<String\> | RRULE | 重复规则 |
| reminder_minutes | Option\<i32\> | VALARM TRIGGER | 提前提醒分钟数 |

### 2.2 ScheduleTabState 变更

```rust
pub struct ScheduleTabState {
    pub schedules: Vec<Schedule>,
    pub show_form: bool,
    pub form: ScheduleForm,
}

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

pub enum RecurrenceRule {
    None,
    Daily,
    Weekly,
    Monthly,
}
```

---

## 3. ICS 解析层

### 3.1 文件位置

新增 `src/data/schedule.rs`，提供两个核心函数：

```rust
pub fn parse_ics(content: &str) -> Result<Schedule>
pub fn format_ics(schedule: &Schedule) -> String
```

### 3.2 parse_ics

**输入**：完整的 ICS 字符串
**输出**：`Result<Schedule>`
**逻辑**：
- 逐行解析 ICS 内容
- 提取 UID、SUMMARY、DTSTART、DTEND、DESCRIPTION、LOCATION、CATEGORIES、PRIORITY、RRULE
- 解析 VALARM TRIGGER 提取提前分钟数
- DTSTART/DTEND 格式：`%Y%m%dT%H%M%SZ`
- CATEGORIES 支持逗号分隔的多值
- PRIORITY 解析为 i32 (1-9)
- 未找到的字段设为 None 或默认值

### 3.3 format_ics

**输入**：`&Schedule`
**输出**：标准 ICS 字符串
**逻辑**：
- 生成 `BEGIN:VCALENDAR` / `VERSION:2.0` / `PRODID:-//Time Manager//EN`
- 生成 `BEGIN:VEVENT`
- 按字段映射写入 ICS 行
- 可选字段为 None 时跳过
- reminder 写入 `BEGIN:VALARM` 块
- rrule 写入 `RRULE:` 行
- 以 `END:VEVENT` / `END:VCALENDAR` 结尾

---

## 4. 模块变更

### 4.1 src/data/fs.rs

```rust
// 现有方法签名不变，内部适配 Schedule 类型
pub fn list_schedules(&self) -> Result<Vec<Schedule>>
pub fn save_schedule(&self, schedule: &Schedule) -> Result<()>
pub fn delete_schedule(&self, id: &Uuid) -> Result<()>
```

- `list_schedules`：读取 `.ics` 文件，调用 `parse_ics` 解析
- `save_schedule`：调用 `format_ics` 生成 ICS，写入文件
- `delete_schedule`：按 id 删除 `.ics` 文件

### 4.2 src/app/schedule_tab.rs

```rust
pub struct ScheduleTabState {
    pub schedules: Vec<Schedule>,
    pub form: ScheduleForm,
    pub show_form: bool,
    pub editing_id: Option<Uuid>,
}

impl ScheduleTabState {
    pub fn new() -> Self
    pub fn reset_form(&mut self)
    pub fn load_for_edit(&mut self, schedule: &Schedule)
    pub fn validate_form(&self) -> Result<Schedule>
}
```

### 4.3 src/app/mod.rs

- `DataLoaded`：适配新 `ScheduleTabState.schedules` 类型
- 新增 `TabId::Schedule` 视图逻辑：排序 + 渲染列表 + Modal

### 4.4 src/gui/messages.rs

新增消息：

```rust
enum Message {
    // ... 现有消息

    // 日程新建
    ScheduleCreateOpen,
    ScheduleCreateConfirm,
    ScheduleFormSummaryChanged(String),
    ScheduleFormStartDateChanged(String),
    ScheduleFormStartTimeChanged(String),
    ScheduleFormEndDateChanged(String),
    ScheduleFormEndTimeChanged(String),
    ScheduleFormDescriptionChanged(String),
    ScheduleFormLocationChanged(String),
    ScheduleFormReminderChanged(String),
    ScheduleFormRruleChanged(RecurrenceRule),

    // 日程编辑
    ScheduleEditOpen(Uuid),
    ScheduleEditConfirm,

    // 日程操作
    ScheduleDelete(Uuid),
}
```

---

## 5. 界面设计

### 5.1 日程列表

```
┌────────────────────────────────────────────────────────────┐
│ 日程                                    [新建日程]          │
├────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐│
│ │                                                         ││
│ │  06/01 (一)  14:00 - 15:00  复习英语单词                  ││
│ │  地点: 图书馆                     [编辑] [删除]          ││
│ │                                                         ││
│ │  06/02 (二)  09:00 - 10:30  数学期中考试                  ││
│ │  地点: 教学楼                     [编辑] [删除]          ││
│ │                                                         ││
│ │  06/03 (三)  19:00 - 20:00  Rust 学习小组                ││
│ │  地点: 线上                       [编辑] [删除]          ││
│ │                                                         ││
│ └─────────────────────────────────────────────────────────┘│
│                                                             │
└────────────────────────────────────────────────────────────┘
```

**每项显示**：
- 日期 + 星期 + 时间范围（主行）
- 标题（主行粗体）
- 地点（次行，小号灰色）
- 操作按钮：编辑 / 删除（右对齐）

**排序规则**：按 DTSTART 升序排列。

### 5.2 新建日程 Modal

```
┌──────────────────────────────────────────────┐
│  新建日程                              [✕]  │
├──────────────────────────────────────────────┤
│                                              │
│  标题:    [________________________]          │
│                                              │
│  开始:    [2026-06-14] [14:00]               │
│                                              │
│  结束:    [2026-06-14] [15:00]               │
│                                              │
│  地点:    [________________________]          │
│                                              │
│  描述:    [________________________]          │
│                                              │
│  提醒:    [15     ] 分钟前                     │
│                                              │
│  重复:    [无 ▼]                             │
│                                              │
│                              [取消]  [创建]   │
└──────────────────────────────────────────────┘
```

**字段说明**：

| 字段 | 控件 | 必需 | 默认值 |
|------|------|------|--------|
| 标题 | text_input | 是 | 空 |
| 开始日期 | text_input (date) | 是 | 今天 |
| 开始时间 | text_input (time) | 是 | 当前时间+1h |
| 结束日期 | text_input (date) | 是 | 今天 |
| 结束时间 | text_input (time) | 是 | 当前时间+2h |
| 地点 | text_input | 否 | 空 |
| 描述 | text_input | 否 | 空 |
| 提醒 | text_input (number) | 否 | 空（不提醒） |
| 重复 | pick_list | 否 | 无 |

**重复选项**：无 / 每天 / 每周 / 每月

---

## 6. 数据流

### 6.1 新建日程流程

```
用户点击 [新建日程] → ScheduleCreateOpen
  → ScheduleTabState.show_form = true
  → 显示 Modal

用户填写表单 → ScheduleForm*Changed 消息
  → ScheduleTabState.form 各字段更新

用户点击 [创建] → ScheduleCreateConfirm
  → 验证表单完整性（标题非空、时间合法）
  → ScheduleTabState.validate_form() 返回 Schedule
  → DataFs.save_schedule(&schedule)
  → DataFs.list_schedules() → 刷新列表
  → Moda 关闭，列表更新
```

### 6.2 编辑日程流程

```
用户点击 [编辑] → ScheduleEditOpen(id)
  → 查询 ScheduleTabState.schedules 中对应 Schedule
  → 将现有字段填入 ScheduleTabState.form
  → 显示 Modal（数据已预填）

用户修改字段 → ScheduleForm*Changed 消息
  → ScheduleTabState.form 各字段更新

用户点击 [创建/保存] → ScheduleEditConfirm
  → 验证表单完整性
  → ScheduleTabState.validate_form() 返回更新后的 Schedule
  → DataFs.save_schedule(&schedule)
  → DataFs.list_schedules() → 刷新列表
  → Modal 关闭，列表更新
```

### 6.3 删除日程流程

```
用户点击 [删除] → ScheduleDelete(id)
  → DataFs.delete_schedule(&id)
  → 从 ScheduleTabState.schedules 中移除
  → 列表即时更新
```

### 6.4 数据加载流程

```
App 启动 / 刷新
  → DataFs.list_schedules()
  → 遍历 schedules/*.ics
  → parse_ics() 逐个解析为 Schedule
  → 按 dtstart 排序
  → ScheduleTabState.schedules = 排序后的 Vec<Schedule>
```

---

## 7. 错误处理

| 场景 | 处理方式 |
|------|----------|
| ICS 文件格式错误 | parse_ics 返回 Err，跳过该文件继续加载其他日程 |
| 新建时标题为空 | 禁用"创建"按钮，或显示错误提示 |
| 起始时间 > 结束时间 | 显示错误提示，阻止创建 |
| 日期格式错误 | 显示错误提示，阻止创建 |
| 文件写入失败 | 显示错误信息，保持 Modal 打开 |

---

## 8. 测试策略

### 8.1 单元测试

- `parse_ics`：解析完整 ICS、缺失字段 ICS、无效 ICS
- `format_ics`：生成标准 ICS、可选字段为 None 时省略
- `parse_ics` + `format_ics` 往返一致性验证
- `validate_form`：合法表单成功、非法表单失败

### 8.2 集成测试

- 新建日程 → 列表显示 → 内容正确
- 编辑日程 → 修改字段 → 保存后列表更新
- 删除日程 → 列表移除 → 文件删除
- 多个日程按时间排序

---

## 9. 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| src/data/models.rs | 修改 | 新增 Schedule 结构体 |
| src/data/schedule.rs | 新增 | parse_ics / format_ics |
| src/data/fs.rs | 修改 | 适配 Schedule 类型 + 新增 delete_schedule |
| src/app/schedule_tab.rs | 重写 | ScheduleTabState + ScheduleForm |
| src/gui/messages.rs | 修改 | 新增日程相关 Message |
| src/app/mod.rs | 修改 | 日程视图 + 消息处理 + Modal 渲染 |
| src/lib.rs | 修改 | 新增 mod schedule |

---

## 10. 与 DESIGN_v3.md 的对齐

| DESIGN_v3.md 要求 | 本设计覆盖情况 |
|-------------------|---------------|
| 日程列表显示时间范围 + 标题 | Paragraph 5.1 - 时间范围 + 标题 + 地点 |
| 排序 | Paragraph 6.3 - 按 DTSTART 排序 |
| 新建日程按钮 + Modal | Paragraph 5.2 - 完整字段 Modal |
| ICS 格式支持完整字段 | Paragraph 2.1/3.3 - 支持所有 DESIGN_v3 定义字段 |
