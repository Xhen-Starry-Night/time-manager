# 分类树快速跳转搜索实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为庞大的分类树提供模糊搜索快速跳转功能，输入关键词后选择候选直接跳转并展开路径。

**Architecture:** 搜索框输入 → 实时匹配树节点路径 → 下拉显示候选 → 选择后展开路径、选中节点、滚动可见、显示详情。

**Tech Stack:** Rust, Iced 0.14, 现有 TreeView 组件

---

## 文件结构

**修改文件**：
- `src/app/category_tab.rs` - 添加搜索状态字段
- `src/gui/messages.rs` - 新增搜索相关消息
- `src/gui/components/tree_view.rs` - 添加 expand_to_path 和 is_expanded 方法
- `src/app/mod.rs` - 搜索 UI + 消息处理 + 跳转逻辑

**新增文件**：
- 无

---

## Task 1: 扩展 CategoryTabState 数据模型

**文件**: `src/app/category_tab.rs`

- [ ] **Step 1: 添加搜索状态字段**

在 `CategoryTabState` 结构体中添加：

```rust
use crate::data::models::Card;
use iced::widget::scrollable;

#[derive(Default)]
pub struct CategoryTabState {
    pub tree_nodes: Vec<crate::gui::components::tree_view::TreeNode>,
    pub tree_view: crate::gui::components::TreeView,
    pub selected_path: Option<String>,
    pub scroll_id: scrollable::Id,
    
    // 搜索跳转
    pub search_query: String,
    pub search_results: Vec<String>,
    pub search_dropdown_open: bool,
}
```

- [ ] **Step 2: 编译验证**

运行: `cargo build 2>&1 | tail -10`
预期: 编译通过（可能有未使用字段警告）

- [ ] **Step 3: 提交**

```bash
git add src/app/category_tab.rs
git commit -m "feat(gui): add search state fields to CategoryTabState"
```

---

## Task 2: 新增搜索消息类型

**文件**: `src/gui/messages.rs`

- [ ] **Step 1: 添加搜索消息**

在 `Message` 枚举中添加：

```rust
CategorySearchChanged(String),
CategorySearchResultSelected(String),
CategorySearchDropdownClose,
```

位置：在 `CategorySelected` 附近添加。

- [ ] **Step 2: 编译验证**

运行: `cargo build 2>&1 | tail -10`
预期: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/gui/messages.rs
git commit -m "feat(gui): add category search messages"
```

---

## Task 3: 扩展 TreeView 方法

**文件**: `src/gui/components/tree_view.rs`

- [ ] **Step 1: 添加 expand_to_path 方法**

在 `impl TreeView` 中添加：

```rust
pub fn expand_to_path(&mut self, target_path: &str) {
    let parts: Vec<&str> = target_path.split('/').collect();
    let mut current_path = String::new();
    
    for i in 0..parts.len().saturating_sub(1) {
        if i > 0 {
            current_path.push('/');
        }
        current_path.push_str(parts[i]);
        self.expanded.insert(current_path.clone(), true);
    }
}

pub fn is_expanded(&self, path: &str) -> bool {
    self.expanded.get(path).copied().unwrap_or(true)
}
```

- [ ] **Step 2: 编译验证**

运行: `cargo build 2>&1 | tail -10`
预期: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/gui/components/tree_view.rs
git commit -m "feat(gui): add expand_to_path and is_expanded methods to TreeView"
```

---

## Task 4: 实现搜索匹配逻辑

**文件**: `src/app/mod.rs`

- [ ] **Step 1: 添加搜索节点方法**

在 `impl App` 中添加：

```rust
fn search_category_nodes(&self, query: &str) -> Vec<String> {
    if query.is_empty() {
        return Vec::new();
    }
    
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    self.collect_matching_paths(&self.category_tab.tree_nodes, &query_lower, &mut results);
    results.truncate(10);
    results
}

fn collect_matching_paths(&self, nodes: &[crate::gui::components::tree_view::TreeNode], query: &str, results: &mut Vec<String>) {
    for node in nodes {
        if node.path.to_lowercase().contains(query) {
            results.push(node.path.clone());
        }
        self.collect_matching_paths(&node.children, query, results);
    }
}
```

- [ ] **Step 2: 添加计算滚动偏移方法**

在 `impl App` 中添加：

```rust
fn calculate_category_scroll_offset(&self, target_path: &str) -> f32 {
    let mut index = 0usize;
    self.count_visible_nodes_until(&self.category_tab.tree_nodes, target_path, &mut index);
    index as f32 * 28.0
}

fn count_visible_nodes_until(&self, nodes: &[crate::gui::components::tree_view::TreeNode], target: &str, index: &mut usize) -> bool {
    for node in nodes {
        if node.path == target {
            return true;
        }
        *index += 1;
        
        if !node.is_card && self.category_tab.tree_view.is_expanded(&node.path) {
            if self.count_visible_nodes_until(&node.children, target, index) {
                return true;
            }
        }
    }
    false
}
```

- [ ] **Step 3: 编译验证**

运行: `cargo build 2>&1 | tail -10`
预期: 编译通过（可能有未使用方法警告）

- [ ] **Step 4: 提交**

```bash
git add src/app/mod.rs
git commit -m "feat(gui): add category search and scroll calculation methods"
```

---

## Task 5: 实现搜索消息处理

**文件**: `src/app/mod.rs`

- [ ] **Step 1: 处理 CategorySearchChanged 消息**

在 `update` 方法的 `match message` 中添加（在 `CategorySelected` 附近）：

```rust
Message::CategorySearchChanged(query) => {
    self.category_tab.search_query = query.clone();
    
    if query.is_empty() {
        self.category_tab.search_results.clear();
        self.category_tab.search_dropdown_open = false;
    } else {
        self.category_tab.search_results = self.search_category_nodes(&query);
        self.category_tab.search_dropdown_open = !self.category_tab.search_results.is_empty();
    }
    
    Task::none()
}
```

- [ ] **Step 2: 处理 CategorySearchResultSelected 消息**

添加：

```rust
Message::CategorySearchResultSelected(path) => {
    self.category_tab.tree_view.expand_to_path(&path);
    self.category_tab.selected_path = Some(path.clone());
    self.category_tab.search_dropdown_open = false;
    self.category_tab.search_query.clear();
    self.category_tab.search_results.clear();
    
    let offset = self.calculate_category_scroll_offset(&path);
    
    let scroll_task = iced::widget::scrollable::scroll_to(
        self.category_tab.scroll_id.clone(),
        iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: offset }
    );
    
    scroll_task
}
```

- [ ] **Step 3: 处理 CategorySearchDropdownClose 消息**

添加：

```rust
Message::CategorySearchDropdownClose => {
    self.category_tab.search_dropdown_open = false;
    Task::none()
}
```

- [ ] **Step 4: 编译验证**

运行: `cargo build 2>&1 | tail -10`
预期: 编译通过

- [ ] **Step 5: 提交**

```bash
git add src/app/mod.rs
git commit -m "feat(gui): implement category search message handlers"
```

---

## Task 6: 实现搜索框 UI

**文件**: `src/app/mod.rs`

- [ ] **Step 1: 在分类树 Tab 添加搜索框**

找到 `TabId::Category =>` 分支，在树视图之前添加搜索框：

```rust
TabId::Category => {
    use iced::widget::scrollable;
    
    let search_input = text_input("搜索...", &self.category_tab.search_query)
        .on_input(Message::CategorySearchChanged)
        .width(Length::Fixed(300.0));
    
    let search_dropdown = if self.category_tab.search_dropdown_open {
        container(
            column(
                self.category_tab.search_results.iter().map(|path| {
                    button(text(path).color(iced::Color::WHITE))
                        .on_press(Message::CategorySearchResultSelected(path.clone()))
                        .width(Length::Fill)
                        .into()
                })
            )
            .spacing(4)
        )
        .width(Length::Fixed(300.0))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Color::from_rgb(0.3, 0.3, 0.3).into()),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
    } else {
        container(column![])
    };
    
    let tree_element = self.category_tab.tree_view.view(
        &self.category_tab.tree_nodes,
        self.category_tab.selected_path.as_deref(),
    ).map(Message::CategorySelected);
    
    let tree_scrollable = scrollable(tree_element)
        .id(self.category_tab.scroll_id.clone())
        .height(Length::Fill);
    
    // 后续是原有的左右分栏逻辑...
}
```

- [ ] **Step 2: 调整布局结构**

将原有的树视图 + 详情面板逻辑移到搜索框下方，整体结构为：

```rust
container(
    column![
        row![search_input].padding(8),
        search_dropdown,
        rule::horizontal(1.0),
        row![left_panel, right_panel].spacing(1),
    ]
    .spacing(4)
)
.width(Length::Fill)
.height(Length::Fill)
```

- [ ] **Step 3: 编译验证**

运行: `cargo build 2>&1 | tail -10`
预期: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src/app/mod.rs
git commit -m "feat(gui): add search input and dropdown UI to category tab"
```

---

## Task 7: 测试验证

- [ ] **Step 1: 运行单元测试**

运行: `cargo test 2>&1 | tail -20`
预期: 所有测试通过

- [ ] **Step 2: 编译发布版本**

运行: `cargo build --release 2>&1 | tail -5`
预期: 编译成功

- [ ] **Step 3: 手动测试**

创建测试数据：
```bash
TMD_DATA_DIR=/tmp/tmd-search-test cargo run --bin tmd -- tree-create 英语
TMD_DATA_DIR=/tmp/tmd-search-test cargo run --bin tmd -- card-create 英语/单词
TMD_DATA_DIR=/tmp/tmd-search-test cargo run --bin tmd -- card-create 英语/语法
TMD_DATA_DIR=/tmp/tmd-search-test cargo run --bin tmd -- tree-create 数学
TMD_DATA_DIR=/tmp/tmd-search-test cargo run --bin tmd -- card-create 数学/微积分
```

运行 GUI：
```bash
TMD_DATA_DIR=/tmp/tmd-search-test cargo run --bin tmd -- gui
```

验证清单：
- [ ] 搜索框显示在分类树顶部
- [ ] 输入 "英语" 显示匹配结果下拉
- [ ] 点击结果跳转并展开路径
- [ ] 跳转后节点选中高亮
- [ ] 跳转后节点可见（滚动到位）
- [ ] 跳转后右侧显示详情
- [ ] 无搜索时下拉不显示

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "feat(gui): implement category tree quick jump search"
```

---

## 验收标准

- [ ] 搜索框显示在分类树顶部，宽度 300px
- [ ] 输入关键词实时显示匹配结果下拉
- [ ] 下拉最多显示 10 条结果
- [ ] 匹配文件夹和卡片
- [ ] 大小写不敏感
- [ ] 点击结果跳转到目标节点
- [ ] 跳转后路径自动展开
- [ ] 跳转后节点选中高亮
- [ ] 跳转后节点滚动可见
- [ ] 跳转后右侧显示详情
- [ ] 无搜索时下拉不占用空间
- [ ] 选择后自动关闭下拉并清空搜索框
- [ ] 所有测试通过
