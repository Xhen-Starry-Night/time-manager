# GUI 分类树快速跳转搜索设计文档

> **日期**: 2026-06-13
> **状态**: 设计完成
> **基于**: DESIGN_v3.md §4.3.1 + brainstorming 探索

---

## 1. 功能概述

**目标**: 为庞大的分类树提供快速跳转功能，通过模糊搜索直接定位到目标节点。

**核心需求**:
- 输入关键词 → 显示候选列表（卡片 + 文件夹）
- 选择候选 → 展开路径 + 选中节点 + 滚动可见 + 显示详情

---

## 2. 界面布局

### 2.1 无搜索时

```
┌────────────────────────────────────────────────────────────┐
│ [分类树] [复习看板] [计时器] [日程] [待办] [预设] [设置]    │
├────────────────────────────────────────────────────────────┤
│ [搜索...]                                                   │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────┬──────────────────────────────────┐ │
│ │ 📁 语言             │   节点详情                        │ │
│ │   └ 📁 英语         │                                  │ │
│ │       └ 📄 单词     │   ...                             │ │
│ └─────────────────────┴──────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

### 2.2 有搜索时

```
┌────────────────────────────────────────────────────────────┐
│ [分类树] [复习看板] [计时器] [日程] [待办] [预设] [设置]    │
├────────────────────────────────────────────────────────────┤
│ [英语单词...]                                               │
├────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────┐   │
│ │ 语言/英语/六级/单词                                   │   │
│ │ 语言/英语/语法                                        │   │
│ └──────────────────────────────────────────────────────┘   │
├────────────────────────────────────────────────────────────┤
│ ┌─────────────────────┬──────────────────────────────────┐ │
│ │ 📁 语言             │   节点详情                        │ │
│ │   └ 📁 英语         │                                  │ │
│ │       └ 📄 单词     │   ...                             │ │
│ └─────────────────────┴──────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

**搜索结果下拉**:
- 显示位置：搜索框正下方
- 动态显示：仅在 `search_query` 不为空且有匹配结果时显示
- 不占用固定空间：无搜索时完全隐藏
- 最大显示：10 条结果
- 宽度：与搜索框对齐（约 300px）

---

## 3. 数据模型

### 3.1 CategoryTabState 扩展

```rust
pub struct CategoryTabState {
    pub tree_nodes: Vec<TreeNode>,
    pub tree_view: TreeView,
    pub selected_path: Option<String>,
    pub scroll_id: scrollable::Id,  // 滚动 ID
    
    // 搜索跳转
    pub search_query: String,
    pub search_results: Vec<String>,
    pub search_dropdown_open: bool,
}
```

### 3.2 TreeNode（已有）

```rust
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_card: bool,
    pub children: Vec<TreeNode>,
}
```

---

## 4. 消息类型

```rust
pub enum Message {
    // 搜索相关
    CategorySearchChanged(String),           // 搜索输入变化
    CategorySearchResultSelected(String),    // 选择搜索结果
    CategorySearchDropdownClose,            // 关闭下拉菜单
}
```

---

## 5. 搜索逻辑

### 5.1 匹配算法

```rust
fn search_nodes(&self, nodes: &[TreeNode], query: &str) -> Vec<String> {
    let query_lower = query.to_lowercase();
    self.collect_paths(nodes, &query_lower)
}

fn collect_paths(&self, nodes: &[TreeNode], query: &str) -> Vec<String> {
    let mut results = Vec::new();
    
    for node in nodes {
        // 匹配文件夹和卡片
        if node.path.to_lowercase().contains(query) {
            results.push(node.path.clone());
        }
        
        // 递归搜索子节点
        results.extend(self.collect_paths(&node.children, query));
    }
    
    // 最多返回 10 条
    results.truncate(10);
    results
}
```

### 5.2 触发时机

```rust
Message::CategorySearchChanged(query) => {
    self.category_tab.search_query = query.clone();
    
    if query.is_empty() {
        self.category_tab.search_results.clear();
        self.category_tab.search_dropdown_open = false;
    } else {
        self.category_tab.search_results = self.search_nodes(
            &self.category_tab.tree_nodes,
            &query
        );
        self.category_tab.search_dropdown_open = !self.category_tab.search_results.is_empty();
    }
    
    Task::none()
}
```

---

## 6. 跳转逻辑

### 6.1 展开路径

**TreeView 新增方法**:
```rust
pub fn expand_to_path(&mut self, target_path: &str) {
    let parts: Vec<&str> = target_path.split('/').collect();
    let mut current_path = String::new();
    
    // 展开所有父节点
    for i in 0..parts.len() - 1 {
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

### 6.2 计算滚动位置

```rust
fn calculate_scroll_offset(&self, target_path: &str) -> f32 {
    let index = self.find_visible_node_index(target_path);
    index as f32 * 28.0  // 每个节点约 28px 高度
}

fn find_visible_node_index(&self, target_path: &str) -> usize {
    let mut index = 0;
    self.count_visible_nodes(&self.category_tab.tree_nodes, target_path, &mut index)
}

fn count_visible_nodes(&self, nodes: &[TreeNode], target: &str, index: &mut usize) -> bool {
    for node in nodes {
        if node.path == target {
            return true;  // 找到目标
        }
        *index += 1;
        
        // 如果节点展开，继续遍历子节点
        if !node.is_card && self.category_tab.tree_view.is_expanded(&node.path) {
            if self.count_visible_nodes(&node.children, target, index) {
                return true;
            }
        }
    }
    false
}
```

### 6.3 完整跳转流程

```rust
Message::CategorySearchResultSelected(path) => {
    // 1. 展开路径到目标节点
    self.category_tab.tree_view.expand_to_path(&path);
    
    // 2. 选中目标节点
    self.category_tab.selected_path = Some(path.clone());
    
    // 3. 关闭搜索下拉
    self.category_tab.search_dropdown_open = false;
    self.category_tab.search_query.clear();
    self.category_tab.search_results.clear();
    
    // 4. 计算滚动位置并执行滚动
    let offset = self.calculate_scroll_offset(&path);
    
    scrollable::scroll_to(
        scrollable::Id::new("category_tree"),
        scrollable::AbsoluteOffset { x: 0.0, y: offset }
    )
}
```

---

## 7. UI 组件

### 7.1 搜索框

```rust
let search_input = text_input("搜索...", &self.category_tab.search_query)
    .on_input(Message::CategorySearchChanged)
    .width(Length::Fixed(300.0));
```

### 7.2 搜索结果下拉

```rust
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
        .padding(8)
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
    container(column![])  // 空容器，不占空间
};
```

### 7.3 绑定滚动 ID

```rust
let tree_scrollable = scrollable(
    self.category_tab.tree_view.view(
        &self.category_tab.tree_nodes,
        self.category_tab.selected_path.as_deref(),
    )
)
.id(scrollable::Id::new("category_tree"))
.height(Length::Fill);
```

---

## 8. 完整布局代码

```rust
TabId::Category => {
    // 搜索框
    let search_input = text_input("搜索...", &self.category_tab.search_query)
        .on_input(Message::CategorySearchChanged)
        .width(Length::Fixed(300.0));
    
    // 搜索下拉
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
            ..Default::default()
        })
    } else {
        container(column![])
    };
    
    // 树视图 + 详情面板
    let tree_element = scrollable(
        self.category_tab.tree_view.view(
            &self.category_tab.tree_nodes,
            self.category_tab.selected_path.as_deref(),
        )
    )
    .id(scrollable::Id::new("category_tree"))
    .height(Length::Fill);
    
    let left_panel = container(tree_element)
        .width(Length::FillPortion(2))
        .height(Length::Fill);
    
    let right_panel = self.render_category_detail();
    
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
    .style(|_| iced::widget::container::Style {
        background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
        ..Default::default()
    })
    .into()
}
```

---

## 9. 实现清单

### Task 1: 数据模型扩展
- `CategoryTabState` 添加搜索相关字段
- `CategoryTabState` 添加 `scroll_id`
- 新增消息类型

### Task 2: 搜索框 UI
- 分类树顶部添加搜索输入框
- 宽度 300px，左对齐

### Task 3: 搜索下拉 UI
- 动态显示搜索结果列表
- 点击触发跳转消息
- 样式：深色背景，圆角

### Task 4: TreeView 扩展方法
- `expand_to_path()` - 展开路径
- `is_expanded()` - 查询展开状态

### Task 5: 跳转逻辑
- 展开路径
- 选中节点
- 计算滚动偏移
- 执行滚动

---

## 10. 验收标准

- [ ] 搜索框显示在分类树顶部
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

---

## 11. 时间估计

| 任务 | 时间 |
|------|------|
| Task 1: 数据模型 | 10 分钟 |
| Task 2: 搜索框 UI | 15 分钟 |
| Task 3: 搜索下拉 UI | 20 分钟 |
| Task 4: TreeView 扩展 | 15 分钟 |
| Task 5: 跳转逻辑 | 30 分钟 |
| **总计** | **约 1.5 小时** |

---

## 12. 后续优化

**暂不实现**:
- 搜索结果高亮匹配字符
- 搜索历史
- 键盘上下键选择结果
- 正则表达式搜索

**理由**: 保持 MVP 简洁，后续根据用户反馈迭代。