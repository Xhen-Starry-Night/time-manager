# 分类树与计时器模块 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现分类树数据结构管理、Obsidian导入、计时状态机、会话管理。

**Architecture:** CategoryNode 使用组合模式实现树结构，在内存中管理；TimerStateMachine 使用状态模式（Idle/Running/Paused），状态转换时记录时间戳；Obsidian导入用 walkdir 遍历目录。这些模块仅操作纯数据，不依赖 UI。

**Tech Stack:** Rust, walkdir 2, chrono 0.4

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/modules/mod.rs` | 模块入口 |
| `src/modules/learning/mod.rs` | learning模块入口 |
| `src/modules/learning/category/mod.rs` | 分类树子模块入口 |
| `src/modules/learning/category/tree.rs` | CategoryNode树结构、操作方法 |
| `src/modules/learning/category/obsidian/mod.rs` | Obsidian导入子模块入口 |
| `src/modules/learning/category/obsidian/vault_parser.rs` | Obsidian仓库解析 |
| `src/modules/learning/timer/mod.rs` | 计时子模块入口 |
| `src/modules/learning/timer/state_machine.rs` | 计时状态机 Idle/Running/Paused |
| `src/modules/learning/timer/session.rs` | 会话管理（创建、确认、参数合并） |
| `src/modules/learning/timer/params.rs` | 参数系统（默认值层级、预设） |

---

### Task 1: 添加 walkdir 依赖

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: 添加 walkdir**

在 `[dependencies]` 末尾追加 `walkdir = "2"`。

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add walkdir dependency"
```

---

### Task 2: 模块骨架

**Files:**
- Create: `src/modules/mod.rs`
- Create: `src/modules/learning/mod.rs`
- Create: `src/modules/learning/category/mod.rs`
- Create: `src/modules/learning/timer/mod.rs`

- [ ] **Step 1: 创建模块骨架文件**

`src/modules/mod.rs`:
```rust
pub mod learning;
```

`src/modules/learning/mod.rs`:
```rust
pub mod category;
pub mod timer;
```

`src/modules/learning/category/mod.rs`:
```rust
pub mod obsidian;
pub mod tree;
```

`src/modules/learning/timer/mod.rs`:
```rust
pub mod params;
pub mod session;
pub mod state_machine;
```

- [ ] **Step 2: 在 lib.rs 中注册模块**

在 `src/lib.rs` 中追加 `pub mod modules;`。

- [ ] **Step 3: 创建占位文件**

为所有叶子模块创建空文件。

- [ ] **Step 4: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/modules/
git commit -m "feat: add learning module skeleton"
```

---

### Task 3: CategoryNode 树结构

**Files:**
- Create: `src/modules/learning/category/tree.rs`

- [ ] **Step 1: 实现 CategoryNode**

```rust
use crate::data::models::{Category, CategoryInsert, Difficulty, Quality};

#[derive(Debug, Clone)]
pub struct CategoryNode {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub source: Option<String>,
    pub default_quality: Option<Quality>,
    pub default_understanding_difficulty: Option<Difficulty>,
    pub default_memory_difficulty: Option<Difficulty>,
    pub children: Vec<CategoryNode>,
}

impl CategoryNode {
    pub fn from_category(cat: &Category) -> Self {
        Self {
            id: cat.id,
            name: cat.name.clone(),
            path: cat.path.clone(),
            source: cat.source.clone(),
            default_quality: cat.default_quality.clone(),
            default_understanding_difficulty: cat.default_understanding_difficulty.clone(),
            default_memory_difficulty: cat.default_memory_difficulty.clone(),
            children: Vec::new(),
        }
    }

    pub fn find_mut(&mut self, id: i64) -> Option<&mut CategoryNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find(&self, id: i64) -> Option<&CategoryNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_by_path(&self, path: &str) -> Option<&CategoryNode> {
        if self.path == path {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_path(path) {
                return Some(found);
            }
        }
        None
    }

    pub fn collect_all(&self) -> Vec<&CategoryNode> {
        let mut result = vec![self];
        for child in &self.children {
            result.extend(child.collect_all());
        }
        result
    }

    pub fn effective_params(&self) -> (&Option<Quality>, &Option<Difficulty>, &Option<Difficulty>) {
        (&self.default_quality, &self.default_understanding_difficulty, &self.default_memory_difficulty)
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn leaf_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        if self.is_leaf() {
            result.push(self);
        } else {
            for child in &self.children {
                result.extend(child.leaf_nodes());
            }
        }
        result
    }
}

pub struct CategoryForest {
    pub roots: Vec<CategoryNode>,
}

impl CategoryForest {
    pub fn from_categories(categories: &[Category]) -> Self {
        let mut nodes: std::collections::HashMap<i64, CategoryNode> = categories
            .iter()
            .map(|c| (c.id, CategoryNode::from_category(c)))
            .collect();

        let mut roots = Vec::new();
        let mut children_map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();

        for cat in categories {
            if let Some(parent_id) = cat.parent_id {
                children_map.entry(parent_id).or_default().push(cat.id);
            } else {
                roots.push(cat.id);
            }
        }

        fn build_tree(
            root_id: i64,
            nodes: &mut std::collections::HashMap<i64, CategoryNode>,
            children_map: &std::collections::HashMap<i64, Vec<i64>>,
        ) -> Option<CategoryNode> {
            let mut node = nodes.remove(&root_id)?;
            if let Some(child_ids) = children_map.get(&root_id) {
                for &cid in child_ids {
                    if let Some(child) = build_tree(cid, nodes, children_map) {
                        node.children.push(child);
                    }
                }
            }
            Some(node)
        }

        let roots: Vec<CategoryNode> = roots
            .into_iter()
            .filter_map(|rid| build_tree(rid, &mut nodes, &children_map))
            .collect();

        Self { roots }
    }

    pub fn find(&self, id: i64) -> Option<&CategoryNode> {
        for root in &self.roots {
            if let Some(found) = root.find(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_mut(&mut self, id: i64) -> Option<&mut CategoryNode> {
        for root in &mut self.roots {
            if let Some(found) = root.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn all_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        for root in &self.roots {
            result.extend(root.collect_all());
        }
        result
    }

    pub fn leaf_nodes(&self) -> Vec<&CategoryNode> {
        let mut result = Vec::new();
        for root in &self.roots {
            result.extend(root.leaf_nodes());
        }
        result
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/modules/learning/category/tree.rs
git commit -m "feat: implement CategoryNode tree structure and CategoryForest"
```

---

### Task 4: Obsidian 导入解析

**Files:**
- Create: `src/modules/learning/category/obsidian/mod.rs`
- Create: `src/modules/learning/category/obsidian/vault_parser.rs`

- [ ] **Step 1: 实现 vault_parser**

`src/modules/learning/category/obsidian/mod.rs`:
```rust
pub mod vault_parser;
```

`src/modules/learning/category/obsidian/vault_parser.rs`:
```rust
use std::path::Path;
use walkdir::WalkDir;

pub struct VaultEntry {
    pub name: String,
    pub relative_path: String,
    pub is_dir: bool,
}

pub fn parse_vault(vault_path: &Path) -> Vec<VaultEntry> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(vault_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if name.starts_with('.') {
            continue;
        }

        let relative = path
            .strip_prefix(vault_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if relative.is_empty() {
            continue;
        }

        let is_dir = entry.file_type().is_dir();

        if is_dir {
            entries.push(VaultEntry {
                name,
                relative_path: relative,
                is_dir: true,
            });
        } else if is_markdown_file(&name) {
            let stem = name.trim_end_matches(".md");
            entries.push(VaultEntry {
                name: stem.to_string(),
                relative_path: relative.trim_end_matches(".md").to_string(),
                is_dir: false,
            });
        }
    }
    entries
}

fn is_markdown_file(name: &str) -> bool {
    name.to_lowercase().ends_with(".md")
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/modules/learning/category/obsidian/
git commit -m "feat: implement Obsidian vault parser"
```

---

### Task 5: 计时状态机

**Files:**
- Create: `src/modules/learning/timer/state_machine.rs`

- [ ] **Step 1: 实现计时状态机**

```rust
use chrono::{DateTime, Utc};
use crate::data::models::PauseRecord;

#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Idle,
    Running { start_time: DateTime<Utc>, pause_total_secs: i64, pauses: Vec<PauseRecord> },
    Paused { start_time: DateTime<Utc>, pause_total_secs: i64, pauses: Vec<PauseRecord>, pause_start: DateTime<Utc> },
}

#[derive(Debug, Clone)]
pub struct TimerStateMachine {
    pub state: TimerState,
    pub category_id: Option<i64>,
}

impl TimerStateMachine {
    pub fn new() -> Self {
        Self {
            state: TimerState::Idle,
            category_id: None,
        }
    }

    pub fn start(&mut self, category_id: i64) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Idle => {
                self.state = TimerState::Running {
                    start_time: Utc::now(),
                    pause_total_secs: 0,
                    pauses: Vec::new(),
                };
                self.category_id = Some(category_id);
                Ok(())
            }
            _ => Err("cannot start: timer is not idle"),
        }
    }

    pub fn pause(&mut self) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Running { .. } => {
                let now = Utc::now();
                if let TimerState::Running { start_time, pause_total_secs, pauses } = &self.state {
                    self.state = TimerState::Paused {
                        start_time: *start_time,
                        pause_total_secs: *pause_total_secs,
                        pauses: pauses.clone(),
                        pause_start: now,
                    };
                }
                Ok(())
            }
            _ => Err("cannot pause: timer is not running"),
        }
    }

    pub fn resume(&mut self) -> Result<(), &'static str> {
        match &self.state {
            TimerState::Paused { .. } => {
                let now = Utc::now();
                if let TimerState::Paused { start_time, pause_total_secs, mut pauses, pause_start } =
                    std::mem::replace(&mut self.state, TimerState::Idle)
                {
                    let pause_secs = (now - pause_start).num_seconds().max(0);
                    pauses.push(PauseRecord {
                        pause_start,
                        resume_time: now,
                    });
                    self.state = TimerState::Running {
                        start_time,
                        pause_total_secs: pause_total_secs + pause_secs,
                        pauses,
                    };
                }
                Ok(())
            }
            _ => Err("cannot resume: timer is not paused"),
        }
    }

    pub fn stop(&mut self) -> Result<StoppedSession, &'static str> {
        let now = Utc::now();
        match &self.state {
            TimerState::Running { start_time, pause_total_secs, pauses } => {
                let start = *start_time;
                let total = *pause_total_secs;
                let p = pauses.clone();
                self.state = TimerState::Idle;
                self.category_id = None;
                let elapsed = (now - start).num_seconds().max(0);
                let effective = (elapsed - total).max(0);
                Ok(StoppedSession {
                    category_id: self.category_id.unwrap_or(0),
                    start_time: start,
                    end_time: now,
                    duration_secs: effective,
                    pause_records: p,
                })
            }
            TimerState::Paused { start_time, pause_total_secs, pauses, pause_start } => {
                let start = *start_time;
                let p_total = *pause_total_secs;
                let mut p = pauses.clone();
                let ps = *pause_start;
                let pause_secs = (now - ps).num_seconds().max(0);
                p.push(PauseRecord {
                    pause_start: ps,
                    resume_time: now,
                });
                let total_pause = p_total + pause_secs;
                self.state = TimerState::Idle;
                let cat_id = self.category_id.unwrap_or(0);
                self.category_id = None;
                let elapsed = (now - start).num_seconds().max(0);
                let effective = (elapsed - total_pause).max(0);
                Ok(StoppedSession {
                    category_id: cat_id,
                    start_time: start,
                    end_time: now,
                    duration_secs: effective,
                    pause_records: p,
                })
            }
            TimerState::Idle => Err("cannot stop: timer is idle"),
        }
    }

    pub fn effective_secs(&self) -> i64 {
        match &self.state {
            TimerState::Idle => 0,
            TimerState::Running { start_time, pause_total_secs, .. } => {
                let elapsed = (Utc::now() - *start_time).num_seconds().max(0);
                (elapsed - pause_total_secs).max(0)
            }
            TimerState::Paused { start_time, pause_total_secs, pause_start, .. } => {
                let elapsed = (Utc::now() - *start_time).num_seconds().max(0);
                let current_pause = (Utc::now() - *pause_start).num_seconds().max(0);
                (elapsed - pause_total_secs - current_pause).max(0)
            }
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, TimerState::Running { .. } | TimerState::Paused { .. })
    }
}

pub struct StoppedSession {
    pub category_id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: i64,
    pub pause_records: Vec<PauseRecord>,
}

impl Default for TimerStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/modules/learning/timer/state_machine.rs
git commit -m "feat: implement timer state machine with Idle/Running/Paused states"
```

---

### Task 6: 参数系统

**Files:**
- Create: `src/modules/learning/timer/params.rs`

- [ ] **Step 1: 实现参数系统**

```rust
use crate::data::models::{Difficulty, Preset, Quality, SessionParams};

pub fn resolve_params(
    global_defaults: &SessionParams,
    node_overrides_quality: Option<&Quality>,
    node_overrides_ud: Option<&Difficulty>,
    node_overrides_md: Option<&Difficulty>,
) -> SessionParams {
    SessionParams {
        quality: node_overrides_quality.cloned().unwrap_or_else(|| global_defaults.quality.clone()),
        understanding_difficulty: node_overrides_ud.cloned().unwrap_or_else(|| global_defaults.understanding_difficulty.clone()),
        memory_difficulty: node_overrides_md.cloned().unwrap_or_else(|| global_defaults.memory_difficulty.clone()),
        completion_rate: global_defaults.completion_rate,
    }
}

pub fn apply_preset(preset: &Preset, params: &mut SessionParams) {
    params.quality = preset.quality.clone();
    params.understanding_difficulty = preset.understanding_difficulty.clone();
    params.memory_difficulty = preset.memory_difficulty.clone();
    params.completion_rate = preset.completion_rate;
}

pub fn presets_from_json(json: &str) -> Vec<Preset> {
    serde_json::from_str(json).unwrap_or_else(|_| Preset::built_in())
}

pub fn presets_to_json(presets: &[Preset]) -> String {
    serde_json::to_string(presets).unwrap_or_else(|_| "[]".into())
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/modules/learning/timer/params.rs
git commit -m "feat: implement parameter resolution and preset system"
```

---

### Task 7: 会话管理

**Files:**
- Create: `src/modules/learning/timer/session.rs`

- [ ] **Step 1: 实现会话管理**

```rust
use crate::data::models::SessionInsert;
use crate::modules::learning::timer::state_machine::StoppedSession;
use crate::data::models::SessionParams;

pub fn create_session_insert(stopped: StoppedSession, params: SessionParams, note: Option<String>) -> SessionInsert {
    SessionInsert {
        category_id: stopped.category_id,
        start_time: stopped.start_time,
        end_time: stopped.end_time,
        duration_secs: stopped.duration_secs,
        pause_records: stopped.pause_records,
        params,
        note,
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/modules/learning/timer/session.rs
git commit -m "feat: implement session creation from stopped timer"
```

---

### Task 8: 分类树与计时器单元测试

**Files:**
- Create: `tests/learning_tests.rs`

- [ ] **Step 1: 编写分类树测试**

```rust
use time_manager::data::models::{Category, CategoryInsert, Difficulty, Quality};
use time_manager::modules::learning::category::tree::{CategoryForest, CategoryNode};

fn sample_categories() -> Vec<Category> {
    vec![
        Category { id: 1, parent_id: None, name: "语言".into(), path: "语言".into(), source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 2, parent_id: Some(1), name: "英语".into(), path: "语言/英语".into(), source: None, default_quality: Some(Quality::High), default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 3, parent_id: Some(2), name: "六级".into(), path: "语言/英语/六级".into(), source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
        Category { id: 4, parent_id: None, name: "数学".into(), path: "数学".into(), source: None, default_quality: None, default_understanding_difficulty: None, default_memory_difficulty: None },
    ]
}

#[test]
fn test_forest_construction() {
    let forest = CategoryForest::from_categories(&sample_categories());
    assert_eq!(forest.roots.len(), 2);
    assert_eq!(forest.roots[0].name, "语言");
    assert_eq!(forest.roots[0].children.len(), 1);
    assert_eq!(forest.roots[0].children[0].name, "英语");
    assert_eq!(forest.roots[0].children[0].children[0].name, "六级");
}

#[test]
fn test_forest_find() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let node = forest.find(3).expect("find 六级");
    assert_eq!(node.name, "六级");
    assert!(forest.find(999).is_none());
}

#[test]
fn test_forest_leaf_nodes() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let leaves = forest.leaf_nodes();
    assert_eq!(leaves.len(), 2);
    let leaf_names: Vec<&str> = leaves.iter().map(|n| n.name.as_str()).collect();
    assert!(leaf_names.contains(&"六级"));
    assert!(leaf_names.contains(&"数学"));
}

#[test]
fn test_node_effective_params() {
    let forest = CategoryForest::from_categories(&sample_categories());
    let en = forest.find(2).expect("find 英语");
    assert_eq!(en.default_quality, Some(Quality::High));
}
```

- [ ] **Step 2: 编写计时器测试**

在 `tests/learning_tests.rs` 追加：

```rust
use time_manager::modules::learning::timer::state_machine::TimerStateMachine;

#[test]
fn test_timer_lifecycle() {
    let mut tm = TimerStateMachine::new();
    assert!(!tm.is_running());

    tm.start(42).expect("start");
    assert!(tm.is_running());

    let secs = tm.effective_secs();
    assert!(secs >= 0);

    tm.pause().expect("pause");
    assert!(tm.is_running());

    tm.resume().expect("resume");
    assert!(tm.is_running());

    let stopped = tm.stop().expect("stop");
    assert!(!tm.is_running());
    assert_eq!(stopped.category_id, 42);
    assert!(stopped.duration_secs >= 0);
}

#[test]
fn test_timer_invalid_transitions() {
    let mut tm = TimerStateMachine::new();
    assert!(tm.pause().is_err());
    assert!(tm.resume().is_err());
    assert!(tm.stop().is_err());

    tm.start(1).expect("start");
    assert!(tm.start(2).is_err());
    assert!(tm.resume().is_err());

    tm.pause().expect("pause");
    assert!(tm.pause().is_err());
}

#[test]
fn test_timer_pause_deducts_time() {
    let mut tm = TimerStateMachine::new();
    tm.start(1).expect("start");
    std::thread::sleep(std::time::Duration::from_millis(100));
    tm.pause().expect("pause");
    std::thread::sleep(std::time::Duration::from_millis(200));
    tm.resume().expect("resume");
    let stopped = tm.stop().expect("stop");
    assert!(stopped.duration_secs < 1);
}
```

- [ ] **Step 3: 运行测试**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add tests/learning_tests.rs
git commit -m "test: add category tree and timer state machine tests"
```

---

### Task 9: Obsidian 导入集成测试

**Files:**
- Modify: `tests/learning_tests.rs`

- [ ] **Step 1: 添加 Obsidian 导入测试**

追加到 `tests/learning_tests.rs`：

```rust
use std::fs;
use time_manager::modules::learning::category::obsidian::vault_parser;

#[test]
fn test_obsidian_vault_parse() {
    let vault_dir = std::env::temp_dir().join("tm-obsidian-test");
    let _ = fs::remove_dir_all(&vault_dir);
    fs::create_dir_all(vault_dir.join("语言/英语")).expect("create dirs");
    fs::write(vault_dir.join("语言/英语/六级.md"), "# 六级").expect("write");
    fs::write(vault_dir.join("语言/英语/语法.md"), "# 语法").expect("write");
    fs::create_dir_all(vault_dir.join(".obsidian")).expect("create hidden");
    fs::write(vault_dir.join(".obsidian/config"), "").expect("write hidden");

    let entries = vault_parser::parse_vault(&vault_dir);
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"英语"));
    assert!(names.contains(&"六级"));
    assert!(names.contains(&"语法"));
    assert!(!names.iter().any(|n| *n == ".obsidian" || *n == "config"));
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/learning_tests.rs
git commit -m "test: add Obsidian vault parser integration test"
```
