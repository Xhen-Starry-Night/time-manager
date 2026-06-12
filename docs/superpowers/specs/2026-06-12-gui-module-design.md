# Time Manager GUI 模块设计文档

**日期**: 2026-06-12  
**状态**: 设计阶段  
**基于**: DESIGN_v3.md §4 GUI 设计 + 当前实现

---

## 1. 概述

### 1.1 目标

为 Time Manager 实现 GUI 界面，提供：
- 学习分类树的可视化浏览与管理
- 复习看板的直观展示
- 计时器的实时显示与控制
- 日程与待办的管理界面
- 参数预设的配置界面

### 1.2 技术选型

**框架**: Iced 0.14

**理由**:
- 纯 Rust 实现，与项目技术栈一致
- 跨平台支持（Linux/Windows/macOS）
- 响应式架构，适合状态管理
- 已有代码基础（DESIGN_v3.md 确定）

**依赖添加**:
```toml
iced = { version = "0.14", features = ["tokio"] }
tokio = { version = "1", features = ["rt-multi-thread"] }
```

### 1.3 设计原则

- **响应式架构**: 使用 Iced 的 Command/Subscription 模式
- **状态分离**: GUI 状态与业务数据分离
- **异步操作**: 文件系统操作使用异步避免阻塞 UI
- **错误处理**: 用户友好的错误提示，不崩溃
- **模块化**: 每个 Tab 对应独立模块

---

## 2. 架构设计

### 2.1 模块结构

```
src/
├── main.rs                    # GUI 入口
├── lib.rs                     # 库根（现有）
├── app.rs                     # 主应用状态
├── gui/
│   ├── mod.rs                 # GUI 模块导出
│   ├── tabs/
│   │   ├── mod.rs             # Tab 定义
│   │   ├── category_tab.rs    # 分类树界面
│   │   ├── review_tab.rs      # 复习看板界面
│   │   ├── timer_tab.rs       # 计时器界面
│   │   ├── schedule_tab.rs    # 日程界面
│   │   ├── todo_tab.rs        # 待办界面
│   │   ├── preset_tab.rs      # 预设界面
│   │   └── settings_tab.rs    # 设置界面
│   ├── components/
│   │   ├── mod.rs             # 组件导出
│   │   ├── tree_view.rs       # 树形视图组件
│   │   ├── card_detail.rs     # 卡片详情组件
│   │   ├── timer_display.rs   # 计时器显示组件
│   │   ├── urgency_badge.rs   # 紧迫度徽章组件
│   │   └── modal.rs           # 模态对话框组件
│   ├── styles/
│   │   ├── mod.rs             # 样式定义
│   │   └── theme.rs           # 主题配置
│   └── messages.rs            # 消息定义
└── data/                      # 数据层（现有）
```

### 2.2 状态架构

```rust
pub struct App {
    pub active_tab: TabId,
    pub data_dir: PathBuf,
    pub data_fs: DataFs,
    
    // 各 Tab 状态
    pub category_tab: CategoryTabState,
    pub review_tab: ReviewTabState,
    pub timer_tab: TimerTabState,
    pub schedule_tab: ScheduleTabState,
    pub todo_tab: TodoTabState,
    pub preset_tab: PresetTabState,
    pub settings_tab: SettingsTabState,
    
    // 全局状态
    pub timer_manager: TimerManager,
    pub error_message: Option<String>,
    pub modal: Option<Modal>,
}
```

### 2.3 消息流

```rust
pub enum Message {
    // Tab 切换
    SwitchTab(TabId),
    
    // 数据操作
    DataLoaded(Result<DataSnapshot, DataError>),
    DataSaved(Result<(), DataError>),
    
    // 计时器
    TimerStarted,
    TimerPaused,
    TimerStopped(Result<PathBuf, String>),
    TimerTick(i64),  // 每秒更新
    
    // 用户交互
    CategorySelected(String),
    CardSelected(String),
    ButtonPressed(ButtonId),
    InputChanged(String),
    
    // 模态框
    ModalOpen(Modal),
    ModalClose,
    ModalConfirm,
    
    // 错误
    Error(String),
    ClearError,
}
```

---

## 3. 界面设计

### 3.1 主窗口布局

```
┌────────────────────────────────────────────────────────────┐
│ Time Manager                                [最小化][关闭] │
├────────────────────────────────────────────────────────────┤
│ [分类树] [复习看板] [计时器] [日程] [待办] [预设] [设置]    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│                     主内容区域                              │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ 状态栏: 数据目录: /path/to/data | 计时器: 00:00:00         │
└────────────────────────────────────────────────────────────┘
```

### 3.2 分类树界面 (CategoryTab)

**布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 分类树                        [搜索...] [新建] [导入]       │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────┬──────────────────────────────────┐ │
│ │ 📁 knowledge         │   节点详情                        │ │
│ │   └ 📁 知识库        │                                  │ │
│ │       └ 📁 则学     │   路径: knowledge/知识库/理则学  │ │
│ │           └ 📄 Rust  │                                  │ │
│ │               ├ 📄 所有权.json                         │ │
│ │               └ 📄 生命周期.json                       │ │
│ │               └ 📄 宏.json                             │ │
│ │                     │   ─────────────────────           │ │
│ │                     │   复习记录: 3 次                  │ │
│ │                     │   总时长: 9.4 秒                  │ │
│ │                     │   下次复习: 7 天后                │ │
│ │                     │                                  │ │
│ │                     │   [开始计时]                      │ │
│ └─────────────────────┴──────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

**功能**:
- 左侧：树形目录浏览
- 右侧：选中节点详情
- 顶部：搜索框、新建按钮、Obsidian 导入按钮
- 右键菜单：新建子目录、新建卡片、启动计时、删除

**状态**:
```rust
pub struct CategoryTabState {
    pub tree_name: String,
    pub selected_path: Option<String>,
    pub search_query: String,
    pub tree_nodes: Vec<TreeNode>,
    pub detail_info: Option<CardDetail>,
}
```

### 3.3 复习看板界面 (ReviewTab)

**布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 复习看板                    [搜索...] [紧急程度▼]          │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────┬──────────────────────────────────┐ │
│ │ 🔴 已过期           │   节点详情                        │ │
│ │   knowledge/数学/微积分    已过期 2 天                  │ │
│ │                     │                                  │ │
│ │ 🟢 三天内           │   路径: knowledge/数学/微积分     │ │
│ │   knowledge/Rust/宏    1 天后                          │ │
│ │                     │   复习记录: 1 次                  │ │
│ │ ⚪ 一周后            │   下次复习: 2026-06-14           │ │
│ │   knowledge/Rust/生命周期    7 天后                    │ │
│ │   knowledge/Rust/所有权    7 天后                      │ │
│ │                     │   [开始计时]                      │ │
│ └─────────────────────┴──────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

**紧迫度颜色**:
- 🔴 红色：已过期 (`urgency == 3`)
- 🟡 黄色：今日到期 (`urgency == 2`)
- 🟢 绿色：三天内 (`urgency == 1`)
- ⚪ 灰色：一周后 (`urgency == 0`)

**状态**:
```rust
pub struct ReviewTabState {
    pub urgency_filter: Option<u32>,  // None=全部, 3=过期, 2=今日, 1=三天内
    pub search_query: String,
    pub cards: Vec<(String, u32, DateTime<Utc>)>,  // (path, urgency, next_review)
    pub selected_card: Option<String>,
}
```

### 3.4 计时器界面 (TimerTab)

**布局（计时中）**:
```
┌────────────────────────────────────────────────────────────┐
│ 计时器                                                      │
├────────────────────────────────────────────────────────────┤
│                                                            │
│                                                            │
│                                                            │
│                     00:05:32                               │
│                                                            │
│                   [暂停]    [停止]                          │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**布局（已停止）**:
```
┌────────────────────────────────────────────────────────────┐
│ 计时结束                                                    │
├────────────────────────────────────────────────────────────┤
│ 有效时长: 5 分钟                                            │
│                                                            │
│ ─────────────────────────────────────────────────────────  │
│                                                            │
│ [创建新卡片]  [链接到现有卡片]  [跳过]                      │
│                                                            │
│ ─────────────────────────────────────────────────────────  │
│                                                            │
│ 卡片路径: [knowledge/Rust/所有权    ]                      │
│                                                            │
│ 记忆表现: [重学] [困难] [好] [简单]                         │
│                                                            │
│                              [取消]  [保存]                 │
└────────────────────────────────────────────────────────────┘
```

**状态**:
```rust
pub struct TimerTabState {
    pub state: TimerState,
    pub elapsed_ms: i64,
    pub link_mode: Option<LinkMode>,  // None=计时中, Some=链接模式
    pub card_path_input: String,
    pub memory_quality: MemoryQuality,
}
```

### 3.5 日程界面 (ScheduleTab)

**布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 日程                                    [新建日程]          │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐│
│ │ 2026-06-23 00:00 - 23:59                                ││
│ │ 程序设计工程实践期末作品提交                              ││
│ │                                                         ││
│ │ 2026-06-13 00:00 - 23:59                                ││
│ │ 英语单词复习                                             ││
│ │                                                         ││
│ └─────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────┘
```

**状态**:
```rust
pub struct ScheduleTabState {
    pub schedules: Vec<(Uuid, String, DateTime<Utc>, DateTime<Utc>)>,
    pub selected_schedule: Option<Uuid>,
}
```

### 3.6 待办界面 (TodoTab)

**布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 待办                     [输入待办内容...] [添加]          │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐│
│ │ ☐ 期末考试复习                          [转日程] [删除] ││
│ │ ☐ 完成作业                              [转日程] [删除] ││
│ └─────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────┘
```

**状态**:
```rust
pub struct TodoTabState {
    pub todos: Vec<Todo>,
    pub new_todo_input: String,
    pub selected_todo: Option<Uuid>,
}
```

### 3.7 预设界面 (PresetTab)

**布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 参数预设                              [新建预设]            │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐│
│ │ default          默认参数预设    [训练] [查看]           ││
│ │ vocabulary       词汇学习预设    [训练] [查看]           ││
│ └─────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────┘
```

**预设详情模态框**:
```
┌────────────────────────────────────────────────────────────┐
│ 预设详情: vocabulary                                         │
├────────────────────────────────────────────────────────────┤
│ 名称: vocabulary                                            │
│ 描述: 词汇学习预设                                           │
│                                                            │
│ 匹配规则:                                                   │
│   knowledge/知识库/理则学/计算机/编程语言/Rust/**           │
│                                                            │
│ 训练状态:                                                   │
│   上次训练: 2026-06-12 12:46                                │
│   参数数量: 17                                              │
│                                                            │
│                              [关闭]                         │
└────────────────────────────────────────────────────────────┘
```

**状态**:
```rust
pub struct PresetTabState {
    pub presets: Vec<Preset>,
    pub selected_preset: Option<String>,
    pub detail_modal: Option<Preset>,
}
```

### 3.8 设置界面 (SettingsTab)

**布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 设置                                                        │
├────────────────────────────────────────────────────────────┤
│ 数据目录: /path/to/data                                     │
│                                                            │
│ 默认参数预设: [default        ▼]                            │
│                                                            │
│ 默认预测算法: [fsrs          ▼]                             │
└────────────────────────────────────────────────────────────┘
```

---

## 4. 数据模型映射

### 4.1 与现有数据层集成

GUI 直接使用现有 `DataFs` 和数据模型：

| GUI 状态 | 数据源 | 方法 |
|----------|--------|------|
| 分类树节点 | `DataFs::list_trees()` | 加载分类树 |
| 卡片列表 | `DataFs::list_cards()` | 加载卡片 |
| 卡片详情 | `DataFs::get_card()` | 获取卡片 |
| 计时器状态 | `TimerManager` | 状态机 |
| 预设列表 | `DataFs::list_presets()` | 加载预设 |
| 日程列表 | `DataFs::list_schedules()` | 加载日程 |
| 待办列表 | `DataFs::list_todos()` | 加载待办 |

### 4.2 现有数据模型使用

| 模型 | GUI 使用 |
|------|----------|
| `Card` | 复习看板、分类树详情 |
| `ReviewRecord` | 卡片详情显示 |
| `Prediction` | 复习看板、下次复习时间 |
| `Timer` | 计时器历史显示 |
| `Preset` | 预设界面 |
| `Todo` | 待办界面 |
| `MemoryQuality` | 计时器链接选择 |

---

## 5. 异步操作设计

### 5.1 Subscription（计时器）

```rust
fn subscription(&self) -> Subscription<Message> {
    if self.timer_tab.state == TimerState::Running {
        Subscription::from_recipe(TimerRecipe)
    } else {
        Subscription::none()
    }
}

struct TimerRecipe;

impl Subscription::Recipe for TimerRecipe {
    type Output = Message;
    
    fn hash(&self, state: &mut Hasher) {
        "timer_tick".hash(state);
    }
    
    fn stream(&self, _: BoxStream) -> BoxStream<Message> {
        stream::interval(Duration::from_secs(1))
            .map(|_| Message::TimerTick(Utc::now().timestamp_millis()))
    }
}
```

### 5.2 Command（文件操作）

```rust
fn load_cards(tree: String, data_dir: PathBuf) -> Command<Message> {
    Command::perform(
        async move {
            let fs = DataFs::init(data_dir)?;
            fs.list_cards(&tree)
        },
        |result| Message::CardsLoaded(result)
    )
}
```

---

## 6. 主题与样式

### 6.1 颜色定义

```rust
pub struct Theme {
    // 紧迫度颜色
    pub urgency_expired: Color,      // 红色 #E74C3C
    pub urgency_today: Color,        // 黄色 #F39C12
    pub urgency_soon: Color,         // 绿色 #27AE60
    pub urgency_later: Color,        // 灰色 #95A5A6
    
    // 主题色
    pub primary: Color,              // #3498DB
    pub secondary: Color,            // #2ECC71
    pub background: Color,           // #FFFFFF
    pub text: Color,                 // #2C3E50
    
    // 状态色
    pub success: Color,              // #27AE60
    pub warning: Color,              // #F39C12
    pub error: Color,                // #E74C3C
}
```

### 6.2 字体与尺寸

```rust
pub struct Typography {
    pub title: Font,
    pub body: Font,
    pub caption: Font,
    
    pub title_size: f32,     // 24.0
    pub body_size: f32,      // 16.0
    pub caption_size: f32,   // 12.0
}
```

---

## 7. 错误处理

### 7.1 错误显示

错误以模态框或状态栏提示显示，不阻塞 UI：

```rust
pub enum AppError {
    DataLoadFailed(String),
    DataSaveFailed(String),
    TimerError(String),
    InvalidInput(String),
}

// 显示错误模态框
Message::Error(error_string) => {
    self.error_message = Some(error_string);
}
```

### 7.2 错误恢复

- 文件不存在：显示空状态，提示创建
- 数据解析失败：显示错误，提供重试按钮
- 计时器错误：恢复到 Idle 状态

---

## 8. 测试策略

### 8.1 单元测试

- 各 Tab 状态更新逻辑
- 消息处理逻辑
- 样式计算

### 8.2 集成测试

- 加载真实数据目录
- 计时器 Subscription 测试
- 文件操作 Command 测试

### 8.3 手动测试

- 完整学习流程测试
- 计时器暂停恢复
- Obsidian 导入
- 预设训练

---

## 9. 实施计划

### Phase 1: 基础框架

1. 添加 Iced 依赖
2. 创建 GUI 模块结构
3. 实现主窗口和 Tab 导航
4. 实现空状态界面

### Phase 2: 分类树界面

1. 实现树形视图组件
2. 实现节点详情显示
3. 实现搜索和新建功能
4. 实现右键菜单

### Phase 3: 复习看板界面

1. 实现紧迫度徽章组件
2. 实现卡片列表显示
3. 实现紧迫度过滤
4. 实现卡片详情面板

### Phase 4: 计时器界面

1. 实现计时器显示组件
2. 实现计时器 Subscription
3. 实现暂停恢复停止
4. 实现链接窗口

### Phase 5: 其他界面

1. 日程界面
2. 待办界面
3. 预设界面
4. 设置界面

---

## 10. 风险与限制

### 10.1 Iced 框架限制

- Iced 0.14 仍在发展中，API 可能变化
- 复杂树形视图可能需要自定义实现
- 异步操作需要合理设计避免阻塞

### 10.2 性能考虑

- 大量卡片加载需要分页或懒加载
- 计时器每秒更新需要轻量处理
- 文件系统操作需要异步

---

## 11. 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-06-12 | 基于 DESIGN_v3.md 和当前实现创建 |