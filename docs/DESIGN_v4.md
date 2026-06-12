# Time Manager 设计文档 v4 - 实现版本

**版本**: v4.0  
**日期**: 2026-06-12  
**状态**: 已实现并验证

---

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
- **设计一致性**：严格遵循设计文档，定期同步代码与文档

### 1.3 已实现功能

```
v1.0 基础功能（已实现）
├── 分类树管理（Obsidian 导入） ✅
├── 计时器（支持暂停/恢复） ✅
├── 学习卡片与复习记录 ✅
├── FSRS 预测 ✅
├── 参数预设训练 ✅
├── 日程管理（.ics 格式） ✅
├── 待办管理 ✅
└── 复习看板 ✅
```

---

## 2. 数据存储架构

### 2.1 目录结构

```
data/
├── categories/                     # 分类树统一存放
│   └── <tree>/                     # 分类树名
│       └── <path>/                 # 分类路径
│           └── <card>.json         # 学习卡片
├── timers/                         # 计时器文件
│   └── <ISO8601时间戳>.json
├── presets/                        # 参数预设
│   └── <preset-name>.json
├── todos/                          # 待办
│   └── <UUID>.json
└── schedules/                      # 日程（.ics 格式）
    └── <UUID>.ics
```

---

## 3. 学习卡片

### 3.1 文件路径规则

```
data/categories/<tree>/<path>.json
例如：data/categories/knowledge/知识库/理则学/计算机/编程语言/Rust/所有权.json
```

### 3.2 文件结构（v4 实现）

```json
{
  "review_records": [
    {
      "timestamp": "2026-06-12T10:30:00Z",
      "duration_ms": 2730000,
      "memory_quality": "Good"
    }
  ],
  "prediction": {
    "algorithm": "fsrs",
    "next_review": "2026-06-05T10:00:00Z",
    "fsrs_state_bytes": [],
    "preset_used": "default"
  }
}
```

### 3.3 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| review_records | Array | 复习记录列表 |
| review_records[].timestamp | DateTime | 复习时间（ISO 8601） |
| review_records[].duration_ms | Number | 有效时长（毫秒） |
| review_records[].memory_quality | Enum | 记忆表现：Relearn/Hard/Good/Easy |
| prediction | Object | 预测状态（可选） |
| prediction.algorithm | String | 算法名称（fsrs） |
| prediction.next_review | DateTime | 下次复习时间 |
| prediction.fsrs_state_bytes | Array | FSRS 内存状态（Vec<u8>） |
| prediction.preset_used | String | 使用的参数预设名称 |

### 3.4 设计变更（v3 → v4）

| 变更 | v3 设计 | v4 实现 | 说明 |
|------|---------|---------|------|
| path 字段 | 未定义 | ✅ 已移除 | 文件路径已包含此信息，无需冗余 |
| source 字段 | 未定义 | ✅ 已移除 | 未定义用途，始终为 null |
| memory_quality | 中文 | 英文 Enum | 使用 Relearn/Hard/Good/Easy |
| timestamp 字段名 | timestamp | timestamp | 保持一致 |
| fsrs_state_bytes | base64 String | Vec<u8> | 直接存储字节数组 |

### 3.5 路径信息获取

路径从文件路径解析，不存储在卡片内部：

```
data/categories/knowledge/知识库/理则学/计算机/编程语言/Rust/所有权.json
→ path: knowledge/知识库/理则学/计算机/编程语言/Rust/所有权
```

---

## 4. 复习记录

### 4.1 设计原则

**独立性**：复习记录与计时器文件完全独立。

- 计时器数据链接到卡片后，复制到 review_records
- 复习记录不存储 timer_path 字段
- 计时器文件可安全删除，不影响学习卡片

### 4.2 结构

```rust
pub struct ReviewRecord {
    pub timestamp: DateTime<Utc>,
    pub duration_ms: i64,
    pub memory_quality: MemoryQuality,
}
```

### 4.3 MemoryQuality 映射

| 英文 | 中文显示 |
|------|----------|
| Relearn | 重学 |
| Hard | 困难 |
| Good | 好 |
| Easy | 简单 |

---

## 5. 计时器

### 5.1 文件结构

```json
{
  "started_at": "2026-06-12T10:30:00Z",
  "stopped_at": "2026-06-12T11:15:30Z",
  "duration_ms": 2730000
}
```

### 5.2 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| started_at | DateTime | 开始时间 |
| stopped_at | DateTime | 停止时间 |
| duration_ms | Number | 有效时长（毫秒） |

### 5.3 设计变更（v3 → v4）

| 变更 | v3 设计 | v4 实现 | 说明 |
|------|---------|---------|------|
| stopped_at | 必需 | ✅ 必需 | 不再是 Option |
| pause_records | 未定义 | ✅ 已移除 | 计时器只保存有效时间 |

### 5.4 计时器状态机

```rust
pub enum TimerState {
    Idle,
    Running {
        started_at: DateTime<Utc>,
        accumulated: i64,          // 累计时间（用于暂停恢复）
    },
    Paused {
        started_at: DateTime<Utc>,
        paused_at: DateTime<Utc>,
        accumulated: i64,          // 暂停时的累计时间
    },
    Stopped {
        started_at: DateTime<Utc>,
        stopped_at: DateTime<Utc>,
        duration_ms: i64,
    },
}
```

**暂停/恢复逻辑**：
- 暂停时：`accumulated += (pause_at - started_at)`
- 恢复时：`started_at = now`，保留 `accumulated`
- 停止时：`duration_ms = accumulated + (stop_at - started_at)`
- 暂停时间不计入有效时长

---

## 6. 参数预设

### 6.1 文件结构

```json
{
  "name": "vocabulary",
  "description": "词汇学习预设",
  "match_rules": [
    "knowledge/知识库/理则学/计算机/编程语言/Rust/**"
  ],
  "fsrs_parameters": [0.212, 1.2931, 2.3065, ...],
  "trained_at": "2026-06-12T10:00:00Z"
}
```

### 6.2 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| name | String | 预设名称 |
| description | String | 预设描述（可选） |
| match_rules | Array | 路径匹配模式列表（glob） |
| fsrs_parameters | Array | FSRS 参数（17 个，训练后生成） |
| trained_at | DateTime | 训练时间（可选） |

### 6.3 设计变更（v3 → v4）

| 变更 | v3 设计 | v4 实现 | 说明 |
|------|---------|---------|------|
| training_rules | training_rules[].path_pattern | match_rules: Array | 简化为字符串数组 |
| fsrs_state_bytes | base64 String | fsrs_parameters: Array | 直接存储 17 个参数值 |

### 6.4 路径匹配规则

使用 globset 库实现：

| 模式 | 含义 |
|------|------|
| `**` | 匹配任意层级路径 |
| `*` | 匹配单层路径 |
| `knowledge/知识库/**` | 匹配 knowledge 下所有层级 |

---

## 7. 待办

### 7.1 文件结构

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "期末考试复习",
  "created_at": "2026-06-12T10:00:00Z"
}
```

### 7.2 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 待办 ID |
| content | String | 待办内容 |
| created_at | DateTime | 创建时间 |

---

## 8. 日程

### 8.1 文件格式

使用标准 .ics 格式：

```ics
BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
DTSTART:20260612T140000Z
DTEND:20260612T150000Z
SUMMARY:程序设计工程实践期末作品提交
END:VEVENT
END:VCALENDAR
```

### 8.2 待办转日程

- 读取待办内容作为 SUMMARY
- 删除原待办文件
- 创建 .ics 日程文件

---

## 9. Obsidian 导入

### 9.1 过滤规则

默认过滤规则（.timeignore 不存在时使用）：

```
.obsidian/
.trash/
.git/
.templates/
attachments/
附件/
images/
*.tmp
*.bak
```

### 9.2 导入逻辑

- 只导入目录结构，不导入文件内容
- 跳过匹配过滤规则的路径
- 创建对应的目录结构到 `data/categories/<tree>/`

---

## 10. CLI 命令

### 10.1 命令列表（16 个）

| 命令 | 说明 |
|------|------|
| init | 初始化数据目录 |
| tree-create | 创建分类树（可选 Obsidian 导入） |
| tree-list | 列出分类树或卡片 |
| card-create | 创建学习卡片（可选指定预设） |
| card-list | 列出卡片 |
| card-predict | 显示卡片预测信息 |
| card-link | 链接计时器到卡片 |
| preset-create | 创建参数预设 |
| preset-list | 列出预设 |
| preset-train | 训练预设参数 |
| timer-start | 启动计时器 |
| timer-pause | 暂停计时器 |
| timer-stop | 停止计时器 |
| todo-create | 创建待办 |
| todo-list | 列出待办 |
| todo-to-schedule | 待办转日程 |
| schedule-create | 创建日程 |
| schedule-list | 列出日程 |
| review-list | 复习看板 |

---

## 11. 已修复的问题

### 11.1 数据结构问题

| 问题 | 修复 |
|------|------|
| ReviewRecord.timer_path | ✅ 移除，违反独立性原则 |
| ReviewRecord.state_bytes | ✅ 移除，状态应在 Prediction |
| Card.path | ✅ 移除，文件路径已包含 |
| Card.source | ✅ 移除，未定义用途 |
| Timer.pause_records | ✅ 移除，只需有效时间 |

### 11.2 计时器问题

| 问题 | 修复 |
|------|------|
| 暂停时间计入有效时长 | ✅ 修复状态机，accumulated 累计 |
| 无法从 Stopped 状态重新启动 | ✅ 添加 Stopped → Running 转换 |

### 11.3 待办转日程问题

| 问题 | 修复 |
|------|------|
| SUMMARY 显示 "Todo" | ✅ 读取待办内容作为 SUMMARY |

---

## 12. 测试覆盖

### 12.1 测试统计

- 单元测试：15 个
- 数据测试：17 个
- FSRS 测试：12 个
- 集成测试：6 个
- Obsidian 测试：12 个
- **总计：62 个测试**

### 12.2 测试内容

- 文件系统操作
- 计时器状态机转换
- 暂停/恢复时长计算
- FSRS 预测和训练
- Obsidian 导入和过滤
- 预设训练流程
- 待办转日程
- 日程文件格式

---

## 13. 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v4.0 | 2026-06-12 | 基于 v3 实现，移除冗余字段，修复计时器，完善测试 |
| v3.0 | 2026-05-31 | JSON 文件存储架构设计 |