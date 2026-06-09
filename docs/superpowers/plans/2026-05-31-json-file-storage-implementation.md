# Time Manager 重构实施计划

> **Feature**: JSON 文件存储架构重构
> **Architecture**: 文件系统存储替代 SQLite，CLI 优先，GUI 导航调整
> **Approach**: 分阶段实施，每阶段独立可测试

---

## Phase 1: 数据层重构

**目标**: 实现文件系统存储，替代 SQLite

### Task 1: 移除 SQLite 依赖

**Files:**
- `Cargo.toml`
- `src/data/database.rs` (删除)
- `src/data/mod.rs`

**Steps:**
- [ ] 编辑 Cargo.toml，移除 rusqlite 依赖
- [ ] 删除 src/data/database.rs 文件
- [ ] 创建 src/data/fs.rs 文件，定义文件系统操作接口

```rust
// src/data/fs.rs
pub struct DataFs {
    data_dir: PathBuf,
}

impl DataFs {
    pub fn init(data_dir: PathBuf) -> Result<Self>;
    pub fn get_card(&self, path: &str) -> Result<Card>;
    pub fn save_card(&self, path: &str, card: &Card) -> Result<()>;
    pub fn list_cards(&self, tree: &str) -> Result<Vec<Card>>;
    pub fn get_timer(&self, filename: &str) -> Result<Timer>;
    pub fn save_timer(&self, timer: &Timer) -> Result<()>;
    pub fn list_timers(&self) -> Result<Vec<Timer>>;
}
```

### Task 2: 实现分类树目录结构

**Files:**
- `src/data/fs.rs`
- `src/modules/learning/category/tree.rs`

**Test:**
```rust
#[test]
fn test_create_category_tree() {
    let fs = DataFs::init(temp_dir);
    fs.create_tree("main").unwrap();
    assert!(temp_dir.join("categories/main").exists());
}

#[test]
fn test_create_card() {
    let fs = DataFs::init(temp_dir);
    fs.create_tree("main").unwrap();
    fs.save_card("main/语言/英语/Unit1", &card).unwrap();
    let path = temp_dir.join("categories/main/语言/英语/Unit1.json");
    assert!(path.exists());
}
```

**Steps:**
- [ ] 实现 create_tree() 方法，创建 data/categories/<tree>/ 目录
- [ ] 实现 save_card() 方法，创建目录结构并写入 JSON 文件
- [ ] 实现 get_card() 方法，从 JSON 文件读取卡片
- [ ] 实现 list_cards() 方法，遍历目录获取所有卡片
- [ ] 运行测试确认通过

### Task 3: 实现计时器文件存储

**Files:**
- `src/data/fs.rs`

**Test:**
```rust
#[test]
fn test_save_timer() {
    let fs = DataFs::init(temp_dir);
    let timer = Timer {
        started_at: "2026-05-31T10:30:00Z",
        stopped_at: "2026-05-31T11:15:30Z",
        duration_ms: 2730000,
    };
    fs.save_timer(&timer).unwrap();
    let path = temp_dir.join("timers/2026-05-31T10-30-00.json");
    assert!(path.exists());
}
```

**Steps:**
- [ ] 实现 save_timer() 方法，生成时间戳文件名并写入 JSON
- [ ] 实现 get_timer() 方法，读取计时器文件
- [ ] 实现 list_timers() 方法，列出所有计时文件
- [ ] 运行测试确认通过

### Task 4: 实现参数预设存储

**Files:**
- `src/data/fs.rs`

**Test:**
```rust
#[test]
fn test_save_preset() {
    let fs = DataFs::init(temp_dir);
    let preset = Preset { name: "default", ... };
    fs.save_preset(&preset).unwrap();
    let path = temp_dir.join("presets/default.json");
    assert!(path.exists());
}
```

**Steps:**
- [ ] 实现 save_preset() 方法
- [ ] 实现 get_preset() 方法
- [ ] 实现 list_presets() 方法
- [ ] 实现路径匹配逻辑（使用 globset）
- [ ] 运行测试确认通过

### Task 5: 实现待办和日程存储

**Files:**
- `src/data/fs.rs`
- `src/modules/todo/mod.rs` (新建)
- `src/modules/schedule/ics.rs` (新建)

**Test:**
```rust
#[test]
fn test_save_todo() {
    let fs = DataFs::init(temp_dir);
    let todo = Todo { id: uuid, content: "复习英语" };
    fs.save_todo(&todo).unwrap();
    let path = temp_dir.join("todos/{uuid}.json");
    assert!(path.exists());
}

#[test]
fn test_save_schedule_ics() {
    let schedule = Schedule { ... };
    let ics_content = schedule_to_ics(&schedule);
    fs.save_schedule(&uuid, &ics_content).unwrap();
    let path = temp_dir.join("schedules/{uuid}.ics");
    assert!(path.exists());
}
```

**Steps:**
- [ ] 创建 src/modules/todo/mod.rs，定义 Todo 结构
- [ ] 创建 src/modules/schedule/ics.rs，实现 .ics 生成和解析
- [ ] 实现 save_todo/get_todo/list_todos 方法
- [ ] 实现 save_schedule/get_schedule/list_schedules 方法
- [ ] 运行测试确认通过

### Task 6: 实现配置文件存储

**Files:**
- `src/config/mod.rs` (新建)
- `src/data/fs.rs`

**Test:**
```rust
#[test]
fn test_config_location() {
    // Linux: ~/.config/time-manager/config.json
    // Windows: %APPDATA%\time-manager\config.json
    let config = Config::load().unwrap();
    assert_eq!(config.default_preset, "default");
}
```

**Steps:**
- [ ] 创建 src/config/mod.rs，使用 directories crate 获取平台配置目录
- [ ] 实现 Config::load() 和 Config::save()
- [ ] 运行测试确认通过

### Task 7: 调整数据模型

**Files:**
- `src/data/models.rs`

**Steps:**
- [ ] 移除 Category/CateogryInsert 结构中的 tree 相关字段
- [ ] 调整 Session 相关结构，移除 SQL 特定字段
- [ ] 添加 Card 结构（替代 Category 作为学习卡片）
- [ ] 添加 Timer/Todo/Schedule 结构
- [ ] 确认所有结构使用 serde derive

### Task 8: 移除旧测试并验证

**Files:**
- `tests/data_tests.rs`

**Steps:**
- [ ] 移除所有 SQLite 相关测试
- [ ] 运行 cargo test 确认新测试通过
- [ ] 确认无编译错误

---

## Phase 2: CLI 重构

**目标**: 实现 16 个 CLI 命令

### Task 9: CLI 命令结构重构

**Files:**
- `src/bin/tmd-cli.rs`
- `src/cli/commands.rs` (新建)

**Steps:**
- [ ] 创建 src/cli/mod.rs 和 src/cli/commands.rs
- [ ] 定义 Command enum，包含所有 16 个命令
- [ ] 实现命令解析逻辑

```rust
pub enum Command {
    Init,
    TreeCreate { name: String, description: Option<String>, import_path: Option<String> },
    TreeList { name: Option<String> },
    CardCreate { path: String },
    CardList { path: String },
    CardPredict { path: String },
    CardLink { timer: String, path: String, memory_quality: String },
    PresetCreate { name: String, description: Option<String> },
    PresetList { name: Option<String> },
    PresetTrain { name: String },
    TimerStart { name: Option<String> },
    TimerPause { name: String },
    TimerStop { name: String },
    TimerGet { name: String },
    TodoCreate { content: String },
    TodoList,
    TodoToSchedule { id: String, start: String, end: String },
    ScheduleCreate { start: String, end: String, summary: String },
    ScheduleList,
    ReviewList { urgency: Option<String> },
}
```

### Task 10: 实现 init 命令

**Test:**
```rust
#[test]
fn test_cli_init() {
    let dir = tempfile::tempdir();
    run_command(Command::Init { data_dir: Some(dir.path()) });
    assert!(dir.path().join("categories").exists());
    assert!(dir.path().join("timers").exists());
    assert!(dir.path().join("presets").exists());
    assert!(dir.path().join("presets/default.json").exists());
}
```

**Steps:**
- [ ] 实现 init 命令，创建基础目录结构
- [ ] 创建默认预设文件
- [ ] 运行测试确认通过

### Task 11: 实现分类树命令

**Test:**
```rust
#[test]
fn test_cli_tree_create() {
    run_command(Command::TreeCreate { name: "main", .. });
    assert!(data_dir.join("categories/main").exists());
}

#[test]
fn test_cli_tree_list() {
    run_command(Command::TreeList { name: None });
    // 验证输出包含目录结构
}
```

**Steps:**
- [ ] 实现 tree-create 命令
- [ ] 实现 tree-list 命令（遍历目录输出结构）
- [ ] 实现 Obsidian 导入逻辑（使用 walkdir + globset）
- [ ] 运行测试确认通过

### Task 12: 实现学习卡片命令

**Test:**
```rust
#[test]
fn test_cli_card_create() {
    run_command(Command::CardCreate { path: "main/语言/英语" });
    assert!(data_dir.join("categories/main/语言/英语.json").exists());
}

#[test]
fn test_cli_card_link() {
    run_command(Command::CardLink { 
        timer: "2026-05-31T10-30-00",
        path: "main/语言/英语",
        memory_quality: "好"
    });
    let card = fs.get_card("main/语言/英语").unwrap();
    assert_eq!(card.review_records.len(), 1);
}
```

**Steps:**
- [ ] 实现 card-create 命令
- [ ] 实现 card-list 命令
- [ ] 实现 card-predict 命令（调用 FSRS）
- [ ] 实现 card-link 命令（读取计时器，更新卡片）
- [ ] 运行测试确认通过

### Task 13: 实现参数预设命令

**Steps:**
- [ ] 实现 preset-create 命令
- [ ] 实现 preset-list 命令
- [ ] 实现 preset-train 命令（遍历匹配卡片，调用 FSRS 训练）
- [ ] 运行测试确认通过

### Task 14: 实现计时器命令

**Steps:**
- [ ] 实现 timer-start 命令（创建计时状态）
- [ ] 实现 timer-pause 命令
- [ ] 实现 timer-stop 命令（保存计时文件）
- [ ] 实现 timer-get 命令
- [ ] 运行测试确认通过

### Task 15: 实现复习看板命令

**Steps:**
- [ ] 实现 review-list 命令（遍历所有卡片，按紧迫度排序）
- [ ] 实现紧急程度筛选逻辑
- [ ] 运行测试确认通过

### Task 16: 实现待办和日程命令

**Steps:**
- [ ] 实现 todo-create 命令
- [ ] 实现 todo-list 命令
- [ ] 实现 todo-to-schedule 命令（创建 .ics 文件，删除待办）
- [ ] 实现 schedule-create 命令
- [ ] 实现 schedule-list 命令
- [ ] 运行测试确认通过

---

## Phase 3: GUI 重构

**目标**: 实现 7 个界面，顶部标签页导航

### Task 17: 调整导航结构

**Files:**
- `src/app.rs`

**Steps:**
- [ ] 添加 Screen enum，包含 7 个界面
- [ ] 调整 view_menu_bar 为顶部标签页布局
- [ ] 实现界面切换逻辑

```rust
pub enum Screen {
    CategoryTree,
    ReviewDashboard,
    TimerData,
    Schedule,
    Todo,
    Preset,
    Settings,
}
```

### Task 18: 重构分类树界面

**Files:**
- `src/app.rs`

**Steps:**
- [ ] 调整 view_category_tree 左右分栏布局
- [ ] 实现右键菜单（新建子节点、新建学习卡片、启动计时、删除节点）
- [ ] 实现删除确认对话框
- [ ] 调整详情面板显示统计信息

### Task 19: 重构复习看板界面

**Steps:**
- [ ] 移除显式分组（过期/今日/未来）
- [ ] 实现紧急程度高光（红/黄/绿/无）
- [ ] 实现分类树模糊搜索
- [ ] 实现紧急程度筛选下拉菜单
- [ ] 调整详情面板布局

### Task 20: 实现计时数据界面

**Steps:**
- [ ] 创建 view_timer_data 方法
- [ ] 实现计时记录列表（保存时间 + 有效时长）
- [ ] 实现双击跳转到链接窗口

### Task 21: 调整计时器界面

**Steps:**
- [ ] 移除路径显示，仅保留时间和按钮
- [ ] 调整链接窗口布局（三选一选项）
- [ ] 实现"创建新学习卡片"表单
- [ ] 实现"链接到现有卡片"表单（路径搜索）
- [ ] 调整 memory_quality 按钮为四个档位

### Task 22: 实现日程界面

**Files:**
- `src/modules/schedule/ui/mod.rs` (新建)

**Steps:**
- [ ] 创建日程列表视图
- [ ] 实现新建日程按钮和对话框
- [ ] 读取 .ics 文件显示日程信息

### Task 23: 实现待办界面

**Files:**
- `src/modules/todo/ui/mod.rs` (新建)

**Steps:**
- [ ] 创建待办列表视图
- [ ] 实现顶部输入框快速添加
- [ ] 实现转日程按钮和对话框
- [ ] 实现删除按钮

### Task 24: 实现预设界面

**Files:**
- `src/app.rs`

**Steps:**
- [ ] 创建预设列表视图
- [ ] 实现新建预设按钮和对话框
- [ ] 实现训练、查看、删除按钮
- [ ] 实现预设详情对话框（可编辑名称、描述、训练规则）

### Task 25: 重构设置界面

**Steps:**
- [ ] 简化为仅两个全局设置
- [ ] 实现默认参数预设下拉菜单
- [ ] 实现默认预测算法下拉菜单

---

## Phase 4: 集成与测试

**目标**: FSRS 集成调整，完整功能测试

### Task 26: 调整 FSRS 参数映射

**Files:**
- `src/modules/learning/prediction/fsrs/mapper.rs`

**Steps:**
- [ ] 调整 quality_to_rating，映射：重学→Again, 困难→Hard, 好→Good, 简单→Easy
- [ ] 运行现有 FSRS 测试确认通过

### Task 27: 调整路径解析逻辑

**Files:**
- `src/data/fs.rs`
- `src/cli/commands.rs`

**Steps:**
- [ ] 确认所有路径格式为 `<tree>/<path>`
- [ ] 实现路径解析函数（从文件路径提取 tree 和 path）
- [ ] 调整所有 CLI 命令的路径参数处理

### Task 28: 集成测试

**Steps:**
- [ ] 编写完整学习流程测试（创建树 → 创建卡片 → 计时 → 链接 → 预测）
- [ ] 编写 Obsidian 导入测试
- [ ] 编写参数预设训练测试
- [ ] 编写待办转日程测试
- [ ] 运行 cargo test 确认所有测试通过

### Task 29: CLI 功能验证

**Steps:**
- [ ] 手动运行所有 CLI 命令
- [ ] 验证输出格式正确
- [ ] 验证数据文件生成正确

### Task 30: GUI 功能验证

**Steps:**
- [ ] 运行 cargo run 启动 GUI
- [ ] 验证所有 7 个界面可正常切换
- [ ] 验证右键菜单功能
- [ ] 验证计时器流程
- [ ] 验证链接窗口流程

### Task 31: 清理与提交

**Steps:**
- [ ] 移除所有废弃代码
- [ ] 运行 cargo clippy 确认无警告
- [ ] 运行 cargo fmt 格式化代码
- [ ] 提交所有变更

---

## 文件变更清单

### 新建文件
- `src/data/fs.rs`
- `src/modules/todo/mod.rs`
- `src/modules/todo/ui/mod.rs`
- `src/modules/schedule/mod.rs`
- `src/modules/schedule/ics.rs`
- `src/modules/schedule/ui/mod.rs`
- `src/cli/mod.rs`
- `src/cli/commands.rs`
- `src/config/mod.rs`

### 删除文件
- `src/data/database.rs`

### 修改文件
- `Cargo.toml`
- `src/data/mod.rs`
- `src/data/models.rs`
- `src/data/export.rs`
- `src/bin/tmd-cli.rs`
- `src/app.rs`
- `src/modules/learning/category/tree.rs`
- `src/modules/learning/category/obsidian/vault_parser.rs`
- `src/modules/learning/prediction/fsrs/mapper.rs`
- `src/settings/defaults.rs`
- `tests/data_tests.rs`

---

## 测试策略

每个 Task 包含：
1. 单元测试（测试单个函数/方法）
2. 集成测试（测试模块间交互）
3. 手动验证（CLI/GUI 功能确认）

TDD 流程：
1. 编写失败测试
2. 运行确认失败
3. 编写最小代码
4. 运行确认通过
5. 提交变更