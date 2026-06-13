# GUI 模块 Task 3 设计文档

> **日期**: 2026-06-13
> **状态**: 设计阶段
> **基于**: DESIGN_v3.md §4.3 + gui-remaining-development-roadmap.md Phase 3

---

## Task 3: 搜索功能

### 3.1 概述

**目标**: 为分类树和复习看板添加搜索功能，快速定位卡片。

**复杂度**: 低
**预计时间**: 1 小时

### 3.2 功能需求

#### 3.2.1 分类树搜索

**位置**: 分类树 Tab 顶部

**界面布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 分类树                        [搜索...] [新建] [导入]       │
├────────────────────────────────────────────────────────────┤
```

**功能**:
- 实时过滤：输入即过滤，无需按回车
- 路径匹配：匹配卡片完整路径（如 `语言/英语/六级/单词`）
- 大小写不敏感
- 空搜索框显示全部

#### 3.2.2 复习看板搜索

**位置**: 复习看板 Tab 顶部

**界面布局**:
```
┌────────────────────────────────────────────────────────────┐
│ 复习看板                    [搜索...] [紧急程度▼] [重新预测]│
├────────────────────────────────────────────────────────────┤
```

**功能**:
- 实时过滤卡片列表
- 路径匹配
- 与紧迫度筛选器协同工作（AND 逻辑）
- 空搜索框显示全部

---

### 3.3 数据模型

#### 3.3.1 状态扩展

**分类树状态** (`CategoryTabState`):
```rust
pub struct CategoryTabState {
    pub tree_nodes: Vec<TreeNode>,
    pub tree_view: TreeView,
    pub selected_path: Option<String>,
    pub search_query: String,  // 新增
}
```

**复习看板状态** (`ReviewTabState`):
```rust
pub struct ReviewTabState {
    pub cards: Vec<(String, Card)>,
    pub search_query: String,      // 已有
    pub urgency_filter: Option<u32>, // 已有
}
```

#### 3.3.2 消息复用

已有消息：
```rust
SearchChanged(String),  // 已存在
```

---

### 3.4 界面设计

#### 3.4.1 搜索框组件

**位置**: Tab 顶部工具栏左侧

**样式**:
- 宽度: `Length::Fixed(200.0)`
- 占位符: `"搜索..."`
- 深色背景: RGB(0.3, 0.3, 0.3)
- 白色文本

**组件代码**:
```rust
let search_input = text_input("搜索...", &self.category_tab.search_query)
    .on_input(Message::SearchChanged)
    .width(Length::Fixed(200.0));
```

#### 3.4.2 分类树界面修改

**修改前**:
```
┌────────────────────────────────────────────────────────────┐
│ 分类树                                                      │
├────────────────────────────────────────────────────────────┤
```

**修改后**:
```
┌────────────────────────────────────────────────────────────┐
│ 分类树                        [搜索...] [新建] [导入]       │
├────────────────────────────────────────────────────────────┤
```

**头部布局**:
```rust
let header = row![
    container(text("分类树").size(20).color(iced::Color::WHITE))
        .padding(8),
    Space::new().width(Length::Fill),
    search_input,
    new_button,
    import_button,
]
.spacing(8)
.padding(8);
```

#### 3.4.3 复习看板界面修改

**修改前**:
```
┌────────────────────────────────────────────────────────────┐
│ 复习看板                    [紧急程度▼]                    │
├────────────────────────────────────────────────────────────┤
```

**修改后**:
```
┌────────────────────────────────────────────────────────────┐
│ 复习看板                    [搜索...] [紧急程度▼] [重新预测]│
├────────────────────────────────────────────────────────────┤
```

**头部布局**:
```rust
let header = row![
    container(text("复习看板").size(20).color(iced::Color::WHITE))
        .padding(8),
    Space::new().width(Length::Fill),
    search_input,
    urgency_filter_buttons,
    repredict_button,
]
.spacing(8)
.padding(8);
```

---

### 3.5 过滤逻辑

#### 3.5.1 分类树过滤

**当前实现** (无搜索):
```rust
let tree_element = self.category_tab.tree_view.view(
    &self.category_tab.tree_nodes,
    self.category_tab.selected_path.as_deref(),
);
```

**修改后** (带搜索):
```rust
let filtered_nodes = if self.category_tab.search_query.is_empty() {
    self.category_tab.tree_nodes.clone()
} else {
    self.filter_tree_nodes(&self.category_tab.tree_nodes, &self.category_tab.search_query)
};

let tree_element = self.category_tab.tree_view.view(
    &filtered_nodes,
    self.category_tab.selected_path.as_deref(),
);
```

**过滤算法**:
```rust
fn filter_tree_nodes(&self, nodes: &[TreeNode], query: &str) -> Vec<TreeNode> {
    let query_lower = query.to_lowercase();
    
    nodes.iter()
        .filter_map(|node| {
            let path_matches = node.path.to_lowercase().contains(&query_lower);
            
            if node.is_card {
                if path_matches {
                    Some(node.clone())
                } else {
                    None
                }
            } else {
                let filtered_children = self.filter_tree_nodes(&node.children, query);
                if path_matches || !filtered_children.is_empty() {
                    let mut filtered = node.clone();
                    filtered.children = filtered_children;
                    Some(filtered)
                } else {
                    None
                }
            }
        })
        .collect()
}
```

#### 3.5.2 复习看板过滤

**当前实现**:
```rust
let cards_with_urgency: Vec<(String, i32, &Card)> = self.review_tab.cards.iter()
    .filter_map(|(path, card)| { ... })
    .filter(|(_, urgency, _)| {
        self.review_tab.urgency_filter
            .map_or(true, |filter| *urgency as u32 == filter)
    })
    .collect();
```

**修改后** (带搜索 + 紧迫度):
```rust
let query_lower = self.review_tab.search_query.to_lowercase();

let cards_with_urgency: Vec<(String, i32, &Card)> = self.review_tab.cards.iter()
    .filter_map(|(path, card)| { ... })
    .filter(|(path, urgency, _)| {
        let matches_search = query_lower.is_empty() 
            || path.to_lowercase().contains(&query_lower);
        let matches_urgency = self.review_tab.urgency_filter
            .map_or(true, |filter| *urgency as u32 == filter);
        
        matches_search && matches_urgency
    })
    .collect();
```

---

### 3.6 消息处理

#### 3.6.1 SearchChanged 消息

**当前实现**:
```rust
Message::SearchChanged(query) => {
    self.review_tab.search_query = query;
    Task::none()
}
```

**修改后** (支持分类树):
```rust
Message::SearchChanged(query) => {
    match self.active_tab {
        TabId::Category => {
            self.category_tab.search_query = query;
        }
        TabId::Review => {
            self.review_tab.search_query = query;
        }
        _ => {}
    }
    Task::none()
}
```

**问题**: 两个 Tab 共用一个消息会导致切换 Tab 时搜索内容混乱。

**解决方案**: 分离为两个独立消息。

#### 3.6.2 新增消息类型

```rust
pub enum Message {
    // ...
    CategorySearchChanged(String),
    ReviewSearchChanged(String),
    // ...
}
```

**消息处理**:
```rust
Message::CategorySearchChanged(query) => {
    self.category_tab.search_query = query;
    Task::none()
}

Message::ReviewSearchChanged(query) => {
    self.review_tab.search_query = query;
    Task::none()
}
```

**删除旧消息**:
```rust
// 删除: SearchChanged(String)
```

---

### 3.7 实现细节

#### Task 3.1: 分类树搜索框

**文件**: `src/app/mod.rs`, `src/gui/messages.rs`

**步骤**:
1. `CategoryTabState` 添加 `search_query: String` 字段
2. 新增 `CategorySearchChanged` 消息
3. 分类树 Tab 顶部添加搜索输入框
4. 实现 `filter_tree_nodes` 方法
5. 消息处理更新 `search_query`

#### Task 3.2: 复习看板搜索框

**文件**: `src/app/mod.rs`, `src/gui/messages.rs`

**步骤**:
1. 复习看板 Tab 顶部添加搜索输入框
2. 新增 `ReviewSearchChanged` 消息
3. 修改卡片过滤逻辑，加入搜索条件
4. 删除旧的 `SearchChanged` 消息

---

### 3.8 验收标准

#### Task 3.1: 分类树搜索

- [ ] 搜索框显示在顶部工具栏
- [ ] 输入文本实时过滤树节点
- [ ] 匹配路径的卡片显示
- [ ] 包含匹配子节点的文件夹显示
- [ ] 空搜索框显示全部节点
- [ ] 大小写不敏感
- [ ] 切换 Tab 后搜索内容保留

#### Task 3.2: 复习看板搜索

- [ ] 搜索框显示在顶部工具栏
- [ ] 输入文本实时过滤卡片列表
- [ ] 与紧迫度筛选器协同工作
- [ ] 空搜索框显示全部卡片
- [ ] 大小写不敏感
- [ ] 切换 Tab 后搜索内容保留

---

### 3.9 后续优化

**暂不实现**:
- 搜索结果高亮
- 搜索历史
- 正则表达式搜索
- 搜索结果计数

**理由**: 保持 MVP 简洁，后续根据用户反馈迭代。

---

## 4. 依赖关系

```
Task 3.1 (分类树搜索) ──> 无依赖
Task 3.2 (复习看板搜索) ──> 无依赖
```

---

## 5. 文件清单

### 修改文件
- `src/app/mod.rs` - 搜索框 UI + 过滤逻辑 + 消息处理
- `src/app/category_tab.rs` - 添加 search_query 字段
- `src/gui/messages.rs` - 新增消息类型

### 新增文件
- 无

---

## 6. 时间估计

| 任务 | 时间 |
|------|------|
| Task 3.1: 分类树搜索框 | 30 分钟 |
| Task 3.2: 复习看板搜索框 | 30 分钟 |
| **总计** | **约 1 小时** |
