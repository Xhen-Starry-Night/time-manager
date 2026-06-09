# Time Manager 设计文档 - JSON 文件存储架构

## 1. 概述

### 1.1 项目定位

学习时间管理工具，核心功能：
- 管理学习内容的层级分类树
- 自由计时模式跟踪学习投入
- 基于 FSRS 算法预测复习时间
- 复习看板展示待复习任务

### 1.2 核心设计原则

- **文件系统优先**：所有数据使用 JSON 文件存储，不引入数据库
- **目录结构即数据**：分类树通过目录结构自然体现
- **CLI 优先**：先实现命令行交互，再实现 GUI
- **本地优先**：数据完全本地化，用户拥有完全控制权

### 1.3 开发路线图

```
v1.0 基础功能（当前）
├── 分类树管理（Obsidian 导入）
├── 计时器
├── 学习卡片与复习记录
├── FSRS 预测
├── 参数预设训练
├── 日程管理（.ics 格式）
├── 待办管理
└── 复习看板
```

---

## 2. 数据存储架构

### 2.1 目录结构

```
data/
├── main/                           # 分类树（目录名即树名）
│   └── 语言/
│       └── 英语/
│           └── 六级/
│               ├── 单词-Unit1.json  # 学习卡片
│               └── 语法.json
├── work/                           # 另一个分类树
│   └── ...
├── timers/                         # 计时器文件
│   ├── 2026-05-31T10-30-00.json
│   └── 2026-05-31T14-15-30.json
├── presets/                        # 参数预设
│   ├── default.json
│   └── focused.json
├── todos/                          # 待办
│   └── 550e8400-e29b-41d4-a716-446655440000.json
├── schedules/                      # 日程（.ics 格式）
│   └── 550e8400-e29b-41d4-a716-446655440001.ics
└── config.json                     # 全局配置
```

### 2.2 学习卡片文件

**路径规则**：
```
data/<tree>/<path>.json
例如：data/main/语言/英语/六级/单词-Unit1.json
```

**文件结构**：
```json
{
  "review_records": [
    {
      "timestamp": "2026-05-31T10:30:00Z",
      "duration_ms": 2730000,
      "quality": "高"
    },
    {
      "timestamp": "2026-06-02T14:00:00Z",
      "duration_ms": 1800000,
      "quality": "中"
    }
  ],
  "prediction": {
    "algorithm": "fsrs",
    "next_review": "2026-06-05T10:00:00Z",
    "state_bytes": "base64...",
    "preset_used": "default"
  }
}
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| review_records | Array | 复习记录列表 |
| review_records[].timestamp | String | ISO 8601 时间戳 |
| review_records[].duration_ms | Number | 有效时长（毫秒） |
| review_records[].quality | String | 记忆表现：极低/低/中/高/完整 |
| prediction | Object | 预测状态（首次复习后生成） |
| prediction.algorithm | String | 算法名称 |
| prediction.next_review | String | 下次复习时间 |
| prediction.state_bytes | String | 算法状态（base64 编码） |
| prediction.preset_used | String | 使用的参数预设名称 |

**路径信息获取**：
- `tree`：从文件路径解析（data 后的第一级目录）
- `path`：从文件路径解析（去除 tree 和文件名）

```
data/main/语言/英语/六级/单词-Unit1.json
→ tree: main
→ path: 语言/英语/六级/单词-Unit1
```

### 2.3 计时器文件

**文件命名**：
```
data/timers/<ISO8601时间戳>.json
例如：data/timers/2026-05-31T10-30-00.json
```

**文件结构**：
```json
{
  "started_at": "2026-05-31T10:30:00Z",
  "stopped_at": "2026-05-31T11:15:30Z",
  "duration_ms": 2730000
}
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| started_at | String | 开始时间（ISO 8601） |
| stopped_at | String | 停止时间（ISO 8601） |
| duration_ms | Number | 有效时长（毫秒） |

**独立性**：
- 计时器文件是独立辅助模块
- 学习卡片读取计时数据后复制到 review_records
- 计时器文件可安全删除，不影响学习卡片

### 2.4 参数预设文件

**文件结构**：
```json
{
  "name": "default",
  "description": "默认参数预设",
  "training_rules": [
    {
      "tree": "main",
      "path_pattern": "语言/英语/**"
    },
    {
      "tree": "work",
      "path_pattern": "**"
    }
  ],
  "fsrs_state_bytes": "base64...",
  "trained_at": "2026-05-31T10:00:00Z"
}
```

**字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| name | String | 预设名称 |
| description | String | 预设描述 |
| training_rules | Array | 训练规则列表 |
| training_rules[].tree | String | 分类树名称，`**` 表示所有树 |
| training_rules[].path_pattern | String | 路径匹配模式 |
| fsrs_state_bytes | String | FSRS 算法状态（训练后生成） |
| trained_at | String | 训练时间 |

**路径匹配规则**：

| 模式 | 含义 |
|------|------|
| `**` | 匹配任意层级路径 |
| `*` | 匹配单层路径 |
| `语言/英语/**` | 匹配"语言/英语"下所有层级 |
| `语言/英语/*` | 仅匹配"语言/英语"下一级 |
| `语言/*/六级` | 匹配"语言/X/六级"（X 为任意值） |

### 2.5 日程文件（.ics 格式）

**文件命名**：
```
data/schedules/<UUID>.ics
例如：data/schedules/550e8400-e29b-41d4-a716-446655440001.ics
```

**文件结构**：
```ics
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Time Manager//EN
CALSCALE:GREGORIAN
METHOD:PUBLISH
BEGIN:VEVENT
UID:550e8400-e29b-41d4-a716-446655440001@time-manager
DTSTART:20260601T140000Z
DTEND:20260601T150000Z
SUMMARY:复习英语单词
DESCRIPTION:复习六级词汇Unit1-Unit5
LOCATION:图书馆
CATEGORIES:学习,英语
CREATED:20260531T100000Z
LAST-MODIFIED:20260531T100000Z
DTSTAMP:20260531T100000Z
STATUS:CONFIRMED
CLASS:PRIVATE
PRIORITY:5
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:日程提醒:复习英语单词
TRIGGER:-PT15M
END:VALARM
END:VEVENT
END:VCALENDAR
```

**支持的功能**：
- 基础时间字段：DTSTART, DTEND, CREATED, DTSTAMP
- 描述性字段：SUMMARY, DESCRIPTION, LOCATION, CATEGORIES
- 状态字段：STATUS, CLASS, PRIORITY
- 提醒设置：VALARM
- 重复规则：RRULE（需要时）

### 2.6 待办文件

**文件命名**：
```
data/todos/<UUID>.json
例如：data/todos/550e8400-e29b-41d4-a716-446655440000.json
```

**文件结构**：
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "复习英语单词",
  "created_at": "2026-05-31T10:00:00Z"
}
```

### 2.7 全局配置文件

**文件结构**：
```json
{
  "default_preset": "default",
  "default_algorithm": "fsrs"
}
```

---

## 3. CLI 命令设计

### 3.1 命令列表（16 个）

#### 初始化
```bash
init [data-dir]
```
创建基础目录结构和默认配置。

#### 分类树管理
```bash
tree-create <name> [description] [import-path] [--ignore-file <path>]
```
创建分类树目录，可选从 Obsidian 导入目录结构。

**过滤规则文件格式**（类似 .gitignore）：
```
.obsidian/
.trash/
.git/
.templates/
附件/
*.tmp
```

```bash
tree-list [name]
```
列出所有分类树或指定树的目录结构。

#### 学习卡片管理
```bash
card-create <tree>/<path>
```
创建学习卡片文件。

```bash
card-list <tree>/[path]
```
列出指定路径下的学习卡片。

```bash
card-predict <tree>/<path>
```
预测指定卡片的下次复习时间。

```bash
card-link <timer-file> <tree>/<path> --quality <quality>
```
将计时器数据链接到学习卡片，创建复习记录。

#### 参数预设管理
```bash
preset-create <name> [description]
```
创建参数预设文件。

```bash
preset-list [name]
```
列出所有参数预设或显示预设详情。

```bash
preset-train <name>
```
根据训练规则匹配的卡片，训练 FSRS 参数。

#### 计时器管理
```bash
timer-start [name]
```
开始计时，创建计时器状态文件（内存中或临时文件）。

```bash
timer-pause <name>
```
暂停计时。

```bash
timer-stop <name>
```
停止计时，保存计时器文件。

```bash
timer-get <name>
```
获取计时器状态或计时数据。

#### 复习看板
```bash
review-list [--tree <name>] [--overdue]
```
列出待复习卡片，按紧迫度排序。

输出格式：
```
┌─────────────────────────────────────────────────────────┐
│ 过期                                                     │
├─────────────────────────────────────────────────────────┤
│ 🔴 语言/英语/六级/单词-Unit1    已过期 2 天              │
│ 🔴 语言/数学/微积分/极限        已过期 1 天              │
├─────────────────────────────────────────────────────────┤
│ 今日到期                                                 │
├─────────────────────────────────────────────────────────┤
│ 🟡 语言/英语/语法               今日 14:00              │
├─────────────────────────────────────────────────────────┤
│ 未来                                                     │
├─────────────────────────────────────────────────────────┤
│ ⚪ 语言/日语/N2/阅读            3 天后                   │
└─────────────────────────────────────────────────────────┘
```

#### 待办管理
```bash
todo-create <content>
```
创建待办。

```bash
todo-list
```
列出所有待办。

```bash
todo-to-schedule <todo-id> --start <datetime> --end <datetime> [options]
```
将待办转为日程。

选项：
- `--start`: 开始时间（必需）
- `--end`: 结束时间
- `--duration`: 时长（分钟，与 --end 二选一）
- `--location`: 地点
- `--description`: 详细描述

#### 日程管理
```bash
schedule-create --start <datetime> --end <datetime> --summary <text> [options]
```
创建日程文件（.ics 格式）。

```bash
schedule-list
```
列出所有日程。

### 3.2 删除操作

不提供删除命令，用户直接操作文件系统：
```bash
rm -rf data/main/                    # 删除分类树
rm data/main/语言/英语/六级/单词.json  # 删除学习卡片
rm data/timers/2026-05-31T10-30-00.json  # 删除计时器
rm data/presets/focused.json         # 删除参数预设
rm data/todos/<uuid>.json            # 删除待办
rm data/schedules/<uuid>.ics         # 删除日程
```

### 3.3 修改操作

不提供修改命令，原因：
- 目录结构即数据真相
- 用户可直接操作文件系统
- 避免字段同步问题

用户修改方式：
```bash
# 重命名分类树
mv data/main data/study

# 移动卡片
mv data/main/语言/英语/六级/单词.json data/main/语言/英语/四级/单词.json
```

---

## 4. GUI 设计

### 4.1 技术选型

- **框架**：Iced 0.14
- **理由**：纯 Rust、跨平台、已有代码基础

### 4.2 导航结构

**顶部标签页导航**：

```
┌──────────────────────────────────────────────────────────┐
│ [分类树] [复习看板] [计时数据] [日程] [待办] [设置]        │
├──────────────────────────────────────────────────────────┤
│                                                          │
│                    主内容区域                             │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### 4.3 界面设计

#### 4.3.1 分类树界面

- 左侧：目录树控件
- 右侧：选中节点详情（统计信息、历史摘要）
- 顶部：搜索栏、创建按钮、Obsidian 导入按钮
- 交互：单击选中、双击启动计时、右键菜单

#### 4.3.2 复习看板界面

- 左侧：卡片列表，按紧迫度排序（🔴 过期、🟡 今日、⚪ 未来）
- 右侧：选中卡片详情
- 顶部：重新预测按钮、筛选选项

#### 4.3.3 计时数据界面

- 列表显示所有计时文件
- 按保存时间排序
- 每项显示：保存时间（大号）、有效时长（小号）
- 双击跳转到链接界面

#### 4.3.4 计时器界面

全屏模式：
- 中央大号时间显示（HH:MM:SS）
- 当前节点名称
- 暂停/继续按钮
- 停止按钮

停止后弹出链接窗口：
- 选项：创建新学习卡片 / 现有学习卡片 / 跳过
- 表单填写：路径、记忆表现

#### 4.3.5 日程界面

- 日程列表（显示 SUMMARY、DTSTART、DTEND）
- 创建日程按钮
- 编辑日程功能

#### 4.3.6 待办界面

- 待办列表
- 创建待办输入框
- 待办转日程按钮

#### 4.3.7 设置界面

- 参数预设管理（创建、训练、列表）
- 全局配置

---

## 5. 核心流程

### 5.1 完整学习流程

```
1. 用户创建分类树（或从 Obsidian 导入目录结构）
2. 用户创建学习卡片
3. 用户启动计时
4. 计时过程中可暂停/恢复
5. 用户停止计时，保存计时文件
6. 用户链接计时文件到学习卡片，填写记忆表现
7. 系统自动调用 FSRS 算法更新预测状态
8. 复习看板显示新的待复习项
```

### 5.2 Obsidian 导入流程

```
1. 用户提供 Obsidian 仓库路径
2. 系统遍历目录结构
3. 应用过滤规则（.gitignore 风格）
4. 创建对应的目录结构到 data/<tree>/
5. 不导入 Markdown 文件
```

### 5.3 参数预设训练流程

```
1. 用户调用 preset-train <name>
2. 系统解析 training_rules
3. 遍历匹配的学习卡片
4. 收集所有 review_records
5. 调用 FSRS 优化器
6. 生成新的 fsrs_state_bytes
7. 写入预设文件
```

### 5.4 待办转日程流程

```
1. 用户调用 todo-to-schedule <id> --start ... --end ...
2. 系统读取待办文件
3. 创建 .ics 日程文件
4. 删除待办文件
5. 返回新日程 ID
```

---

## 6. 数据目录配置

### 6.1 配置优先级

1. 命令行参数：`--data-dir <path>`
2. 环境变量：`TIME_MANAGER_DATA`
3. 配置文件：`~/.config/time-manager/config.toml`
4. 平台默认路径

### 6.2 配置文件

**位置**：`~/.config/time-manager/config.toml`

**内容**：
```toml
data_dir = "/custom/path"
```

### 6.3 平台默认路径

| 平台 | 默认路径 |
|------|----------|
| Linux | `~/.local/share/time-manager/` |
| Windows | `%LOCALAPPDATA%\time-manager\` |
| macOS | `~/Library/Application Support/time-manager/` |

---

## 7. 技术选型

### 7.1 依赖列表

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
directories = "6"
walkdir = "2"
fsrs = "5"
iced = { version = "0.14", features = ["tokio"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
globset = "0.4"          # .gitignore 风格路径匹配
ics = "0.5"              # .ics 文件生成和解析
toml = "0.8"             # 配置文件解析
uuid = { version = "1", features = ["v4"] }  # UUID 生成
```

### 7.2 移除的依赖

```toml
rusqlite = { version = "0.34", features = ["bundled"] }  # 已移除
```

---

## 8. 模块结构

```
src/
├── main.rs                    # GUI 入口
├── lib.rs                     # 库根
├── app.rs                     # 主应用逻辑
├── bin/
│   └── tmd-cli.rs             # CLI 入口
├── data/
│   ├── mod.rs
│   ├── fs.rs                  # 文件系统操作
│   ├── models.rs              # 数据模型
│   ├── error.rs               # 错误类型
│   └── export.rs              # 导出功能
├── modules/
│   ├── learning/
│   │   ├── timer/
│   │   │   ├── mod.rs
│   │   │   └── state_machine.rs
│   │   ├── category/
│   │   │   ├── mod.rs
│   │   │   ├── tree.rs
│   │   │   └── obsidian/
│   │   │       └── vault_parser.rs
│   │   └── prediction/
│   │       ├── mod.rs
│   │       ├── algorithm.rs
│   │       └── fsrs/
│   │           ├── mod.rs
│   │           ├── adapter.rs
│   │           ├── mapper.rs
│   │           └── state.rs
│   ├── schedule/
│   │   ├── mod.rs
│   │   └── ics.rs
│   └── todo/
│       └── mod.rs
├── cli/
│   ├── mod.rs
│   └── commands.rs
├── config/
│   └── mod.rs
└── settings/
    └── defaults.rs
```

---

## 9. 测试策略

### 9.1 单元测试

- 文件系统操作：创建、读取、写入、删除
- 路径匹配规则：globset 各种模式
- FSRS 算法：预测、训练
- .ics 文件：解析、生成
- 计时状态机：状态转换、时长计算

### 9.2 集成测试

- 完整学习流程：创建树 → 创建卡片 → 计时 → 链接 → 预测
- Obsidian 导入：模拟目录结构导入
- 参数预设训练：多卡片训练
- 待办转日程：完整转换流程

### 9.3 CLI 测试

```bash
# 初始化
tmd-cli init /tmp/test-data

# 创建分类树
tmd-cli tree-create main "主学习"

# 创建卡片
tmd-cli card-create main/语言/英语/Unit1

# 计时
tmd-cli timer-start study
sleep 5
tmd-cli timer-stop study

# 链接
tmd-cli card-link timers/20260531T100000 main/语言/英语/Unit1 --quality 高

# 预测
tmd-cli card-predict main/语言/英语/Unit1

# 复习看板
tmd-cli review-list
```

---

## 10. 迁移计划

### 10.1 从 SQLite 迁移到文件系统

**当前数据**：
- SQLite 数据库：`app.db`
- 表：categories, sessions, prediction_states, settings

**迁移步骤**：
1. 读取 SQLite 数据
2. 转换为 JSON 格式
3. 按新目录结构写入文件
4. 验证数据完整性

**迁移工具**：
```bash
tmd-cli migrate-from-sqlite <db-path> <output-dir>
```

### 10.2 数据映射

| SQLite 表 | 文件系统位置 |
|-----------|--------------|
| categories | data/<tree>/<path>.json |
| sessions | data/timers/ + 卡片内 review_records |
| prediction_states | 卡片内 prediction 字段 |
| settings | data/config.json + presets/ |

---

## 11. 设计决策记录

### 11.1 为什么不用数据库？

- FIX_3.md 明确要求"不引入数据库"
- 文件系统存储更透明，用户可直接操作
- 符合本地优先原则
- 避免数据库依赖和版本兼容问题

### 11.2 为什么移除 tree/path 字段？

- 目录结构已经表达了这些信息
- 避免字段与实际路径不同步的问题
- 用户可直接操作文件系统修改
- 符合"结构即数据"原则

### 11.3 为什么不提供删除/修改命令？

- 用户可直接操作文件系统
- 减少命令复杂度
- 符合 Unix 哲学

### 11.4 为什么日程使用 .ics 格式？

- 标准格式，可被其他日历应用导入
- 支持完整的日程功能（重复、提醒等）
- 互操作性强

---

## 12. 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-05-31 | 初始设计，基于 FIX_3.md 要求 |
