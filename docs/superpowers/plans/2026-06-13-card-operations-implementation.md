# 分类树节点操作实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为分类树节点提供完整的 CRUD 操作能力，支持学习卡片和文件夹的管理。

**Architecture:** 
- DataFs 层新增 delete_card、delete_folder、rename_card 方法
- Modal 组件独立封装（NewNodeModal、EditCardModal、DeleteConfirmModal）
- App 状态扩展表单字段和消息处理

**Tech Stack:** Rust, Iced 0.14, serde, chrono

---

## 文件结构

### 新建文件
- `src/gui/components/new_node_modal.rs` - 新建节点 Modal 组件
- `src/gui/components/edit_card_modal.rs` - 编辑卡片 Modal 组件

### 修改文件
- `src/data/fs.rs` - 新增 delete_card、delete_folder、rename_card 方法
- `src/data/error.rs` - 新增 FolderNotFound、InvalidNodeName 错误类型
- `src/gui/messages.rs` - 新增节点操作相关消息
- `src/gui/mod.rs` - 导出新组件
- `src/gui/components/mod.rs` - 注册新组件
- `src/app/mod.rs` - 新增表单状态、消息处理、右侧面板操作按钮
- `src/app/category_tab.rs` - 新增 NewNodeForm、EditCardForm 状态

---

## Task 1: DataFs 扩展 - 删除方法

**Files:**
- Modify: `src/data/fs.rs`
- Modify: `src/data/error.rs`

### Step 1: 新增错误类型

```rust
// src/data/error.rs

#[derive(Debug, Clone, thiserror::Error)]
pub enum DataError {
    // 现有...
    
    #[error("Folder not found: {0}")]
    FolderNotFound(String),
    
    #[error("Invalid node name: {0}")]
    InvalidNodeName(String),
    
    #[error("Node already exists: {0}")]
    NodeAlreadyExists(String),
}
```

### Step 2: 实现 delete_card 方法

```rust
// src/data/fs.rs

impl DataFs {
    // ... 现有方法 ...
    
    /// 删除学习卡片文件
    pub fn delete_card(&self, path: &str) -> Result<()> {
        let (tree, card_path) = Self::parse_path(path)?;
        let file_path = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", card_path));
        
        if !file_path.exists() {
            return Err(DataError::CardNotFound(path.into()));
        }
        
        std::fs::remove_file(&file_path)
            .map_err(|e| DataError::Io(e.to_string()))?;
        
        Ok(())
    }
}
```

### Step 3: 实现 delete_folder 方法

```rust
// src/data/fs.rs

impl DataFs {
    /// 删除文件夹（递归删除目录及内容）
    pub fn delete_folder(&self, path: &str) -> Result<()> {
        let (tree, folder_path) = Self::parse_path(path)?;
        let dir_path = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(folder_path);
        
        if !dir_path.exists() {
            return Err(DataError::FolderNotFound(path.into()));
        }
        
        std::fs::remove_dir_all(&dir_path)
            .map_err(|e| DataError::Io(e.to_string()))?;
        
        Ok(())
    }
}
```

### Step 4: 实现 rename_card 方法

```rust
// src/data/fs.rs

impl DataFs {
    /// 重命名卡片（移动文件）
    pub fn rename_card(&self, old_path: &str, new_name: &str) -> Result<String> {
        let (tree, old_card_path) = Self::parse_path(old_path)?;
        
        let old_path_buf = std::path::Path::new(old_card_path);
        let parent = old_path_buf.parent()
            .ok_or_else(|| DataError::InvalidPath(old_path.into()))?;
        let new_card_path = parent.join(new_name);
        let new_card_path_str = new_card_path.to_string_lossy();
        let new_full_path = format!("{}/{}", tree, new_card_path_str);
        
        let old_file = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", old_card_path));
        let new_file = self
            .data_dir
            .join("categories")
            .join(&tree)
            .join(format!("{}.json", new_card_path_str));
        
        if new_file.exists() {
            return Err(DataError::NodeAlreadyExists(new_full_path));
        }
        
        std::fs::rename(&old_file, &new_file)
            .map_err(|e| DataError::Io(e.to_string()))?;
        
        Ok(new_full_path)
    }
}
```

### Step 5: 验证编译

```bash
cargo build --release
```

Expected: 编译成功，无错误

---

## Task 2: 消息类型扩展

**Files:**
- Modify: `src/gui/messages.rs`

### Step 1: 新增 NodeType 枚举

```rust
// src/gui/messages.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Folder,
    Card,
}
```

### Step 2: 新增消息类型

```rust
// src/gui/messages.rs

#[derive(Debug, Clone)]
pub enum Message {
    // 现有消息...
    
    // 新建节点
    NewNodeOpen(NodeType),
    NewNodeNameChanged(String),
    NewNodeTypeChanged(NodeType),
    NewNodePresetChanged(String),
    NewNodeConfirm,
    
    // 编辑卡片
    EditCardOpen(String),
    EditCardNameChanged(String),
    EditCardPresetChanged(String),
    EditCardNextReviewChanged(String),
    EditCardClearPrediction,
    EditCardToggleGroup(String),
    EditCardAddReview,
    EditCardRemoveReview(usize),
    EditCardNewReviewDurationChanged(String),
    EditCardNewReviewQualityChanged(MemoryQuality),
    EditCardConfirm,
    
    // 删除节点
    DeleteNodeOpen(String, bool),
    DeleteNodeConfirm,
}
```

### Step 3: 扩展 Modal 枚举

```rust
// src/gui/messages.rs

#[derive(Debug, Clone)]
pub enum Modal {
    // 现有...
    NewCard { path: String },
    LinkTimer { timer_path: PathBuf, duration_ms: i64 },
    NewTodo,
    EditTodo { id: Uuid },
    NewSchedule,
    PresetDetail { name: String },
    ConfirmDelete { item: String },
    Error { message: String },
    
    // 新增
    NewNode,
    EditCard { path: String },
}
```

### Step 3: 导入依赖

```rust
// src/gui/messages.rs

use crate::data::models::{Card, MemoryQuality};
```

### Step 4: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 3: 表单状态结构

**Files:**
- Modify: `src/app/category_tab.rs`

### Step 1: 新增 NewNodeForm 结构

```rust
// src/app/category_tab.rs

use crate::gui::NodeType;

#[derive(Debug, Clone)]
pub struct NewNodeForm {
    pub parent_path: String,
    pub node_type: NodeType,
    pub name: String,
    pub preset: String,
}

impl NewNodeForm {
    pub fn new(parent_path: String, default_type: NodeType) -> Self {
        Self {
            parent_path,
            node_type: default_type,
            name: String::new(),
            preset: "default".to_string(),
        }
    }
}
```

### Step 2: 新增 EditCardForm 结构

```rust
// src/app/category_tab.rs

use std::collections::HashSet;
use chrono::{DateTime, Utc};
use crate::data::models::{Card, ReviewRecord, MemoryQuality};

#[derive(Debug, Clone)]
pub struct EditCardForm {
    pub path: String,
    pub original_name: String,
    pub new_name: String,
    pub preset: String,
    pub next_review: Option<DateTime<Utc>>,
    pub clear_prediction: bool,
    pub review_records: Vec<ReviewRecord>,
    pub expanded_groups: HashSet<String>,
    pub new_review_duration: String,
    pub new_review_quality: MemoryQuality,
}

impl EditCardForm {
    pub fn new(path: String, card: &Card) -> Self {
        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
        let preset = card.prediction.as_ref()
            .map(|p| p.preset_used.clone())
            .unwrap_or_else(|| "default".to_string());
        
        let mut expanded_groups = HashSet::new();
        expanded_groups.insert("basic".to_string());
        expanded_groups.insert("prediction".to_string());
        
        Self {
            path: path.clone(),
            original_name: name.clone(),
            new_name: name,
            preset,
            next_review: card.prediction.as_ref().map(|p| p.next_review),
            clear_prediction: false,
            review_records: card.review_records.clone(),
            expanded_groups,
            new_review_duration: String::new(),
            new_review_quality: MemoryQuality::Good,
        }
    }
}
```

### Step 3: 扩展 CategoryTabState

```rust
// src/app/category_tab.rs

pub struct CategoryTabState {
    pub tree_name: String,
    pub selected_path: Option<String>,
    pub search_query: String,
    pub tree_nodes: Vec<TreeNode>,
    pub tree_view: TreeView,
    pub search_results: Vec<SearchResult>,
    pub search_selected_index: Option<usize>,
    pub search_focused: bool,
    
    // 新增
    pub new_node_form: Option<NewNodeForm>,
    pub edit_card_form: Option<EditCardForm>,
    pub delete_target: Option<(String, bool)>, // (path, is_folder)
}
```

### Step 4: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 4: 右侧面板操作按钮

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 新增操作按钮组件函数

```rust
// src/app/mod.rs

fn card_action_buttons(path: &str) -> Element<Message> {
    let path = path.to_string();
    row![
        button(
            text("编辑").color(iced::Color::WHITE)
        )
        .on_press(Message::EditCardOpen(path.clone()))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.3, 0.5, 0.7).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        button(
            text("删除").color(iced::Color::WHITE)
        )
        .on_press(Message::DeleteNodeOpen(path.clone(), false))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.7, 0.3, 0.3).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .spacing(8)
    .into()
}

fn folder_action_buttons(path: &str) -> Element<Message> {
    let path = path.to_string();
    row![
        button(
            text("新建卡片").color(iced::Color::WHITE)
        )
        .on_press(Message::NewNodeOpen(NodeType::Card))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.6, 0.4).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        button(
            text("新建文件夹").color(iced::Color::WHITE)
        )
        .on_press(Message::NewNodeOpen(NodeType::Folder))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.4, 0.5, 0.6).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        button(
            text("删除").color(iced::Color::WHITE)
        )
        .on_press(Message::DeleteNodeOpen(path.clone(), true))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.7, 0.3, 0.3).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .spacing(8)
    .into()
}
```

### Step 2: 修改右侧面板布局

找到 `TabId::Category` 分支中的右侧面板渲染代码，修改为：

```rust
// 判断选中节点是否为卡片
let is_card = self.category_tab.selected_path.as_ref()
    .and_then(|p| self.review_tab.cards.iter().find(|(path, _)| path == p))
    .is_some();

let right_panel = if let Some(ref selected_path) = self.category_tab.selected_path {
    if is_card {
        // 学习卡片详情
        let card_data = self.review_tab.cards.iter()
            .find(|(path, _)| path == selected_path);
        
        if let Some((path, card)) = card_data {
            container(
                column![
                    row![
                        text("路径: ").color(iced::Color::WHITE),
                        text(path).color(iced::Color::WHITE),
                    ],
                    Space::new().height(12),
                    card_action_buttons(path),
                    Space::new().height(12),
                    rule::horizontal(1.0),
                    Space::new().height(12),
                    crate::gui::components::CardDetail::view(path, card)
                        .map(|_| Message::ClearError),
                ]
                .padding(16)
            )
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                ..Default::default()
            })
        } else {
            container(text("选择卡片查看详情").size(16))
                .width(Length::FillPortion(3))
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
        }
    } else {
        // 文件夹详情
        let children_count = self.category_tab.tree_nodes.iter()
            .filter(|n| n.path.starts_with(selected_path) && n.path != *selected_path)
            .count();
        
        container(
            column![
                row![
                    text("路径: ").color(iced::Color::WHITE),
                    text(selected_path).color(iced::Color::WHITE),
                ],
                Space::new().height(12),
                folder_action_buttons(selected_path),
                Space::new().height(12),
                rule::horizontal(1.0),
                Space::new().height(12),
                row![
                    text("子项数量: ").color(iced::Color::WHITE),
                    text(format!("{} 个", children_count)).color(iced::Color::WHITE),
                ],
                Space::new().height(8),
                text("(选中具体节点查看详情)")
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                    .size(14),
            ]
            .padding(16)
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
            ..Default::default()
        })
    }
} else {
    container(text("选择节点查看详情").size(16))
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
            ..Default::default()
        })
};
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 5: 新建节点 Modal 组件

**Files:**
- Create: `src/gui/components/new_node_modal.rs`
- Modify: `src/gui/components/mod.rs`

### Step 1: 创建组件文件

```rust
// src/gui/components/new_node_modal.rs

use iced::widget::{button, column, row, text, text_input, radio, pick_list, Space};
use iced::{Element, Length};
use crate::gui::{Message, NodeType};
use crate::app::category_tab::NewNodeForm;

pub struct NewNodeModal;

impl NewNodeModal {
    pub fn view(form: &NewNodeForm, presets: &[String]) -> Element<Message> {
        let path_preview = row![
            text("在 ").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            text(format!("\"{}\"", form.parent_path)).color(iced::Color::from_rgb(0.3, 0.7, 0.9)),
            text(" 下创建").color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
        ];
        
        let type_selector = row![
            radio(
                "文件夹",
                NodeType::Folder,
                Some(form.node_type),
                Message::NewNodeTypeChanged,
            ),
            radio(
                "学习卡片",
                NodeType::Card,
                Some(form.node_type),
                Message::NewNodeTypeChanged,
            ),
        ]
        .spacing(16);
        
        let name_input = text_input("名称", &form.name)
            .on_input(Message::NewNodeNameChanged)
            .width(Length::Fill);
        
        let preset_selector = if form.node_type == NodeType::Card {
            Some(
                column![
                    text("预设:").color(iced::Color::WHITE),
                    pick_list(presets, Some(form.preset.clone()), Message::NewNodePresetChanged)
                        .width(Length::Fill),
                ]
                .spacing(4)
            )
        } else {
            None
        };
        
        let buttons = row![
            button(text("取消").color(iced::Color::WHITE))
                .on_press(Message::ModalClose)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.4, 0.4, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            button(text("创建").color(iced::Color::WHITE))
                .on_press(Message::NewNodeConfirm)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.3, 0.6, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(8)
        .width(Length::Fill)
        .push(Space::new().width(Length::Fill));
        
        let mut content = column![
            path_preview,
            Space::new().height(16),
            text("类型:").color(iced::Color::WHITE),
            type_selector,
            Space::new().height(12),
            text("名称:").color(iced::Color::WHITE),
            name_input,
        ]
        .spacing(4);
        
        if let Some(preset_ui) = preset_selector {
            content = content
                .push(Space::new().height(12))
                .push(preset_ui);
        }
        
        content
            .push(Space::new().height(24))
            .push(buttons)
            .padding(16)
            .into()
    }
}
```

### Step 2: 注册组件

```rust
// src/gui/components/mod.rs

mod new_node_modal;

pub use new_node_modal::NewNodeModal;
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 6: 编辑卡片 Modal 组件

**Files:**
- Create: `src/gui/components/edit_card_modal.rs`
- Modify: `src/gui/components/mod.rs`

### Step 1: 创建组件文件

```rust
// src/gui/components/edit_card_modal.rs

use iced::widget::{button, column, row, text, text_input, pick_list, checkbox, Space};
use iced::{Element, Length};
use crate::gui::Message;
use crate::app::category_tab::EditCardForm;
use crate::data::models::MemoryQuality;

pub struct EditCardModal;

impl EditCardModal {
    pub fn view(form: &EditCardForm, presets: &[String]) -> Element<Message> {
        let basic_expanded = form.expanded_groups.contains("basic");
        let prediction_expanded = form.expanded_groups.contains("prediction");
        let records_expanded = form.expanded_groups.contains("records");
        
        let basic_group = collapsible_section(
            "基本信息",
            "basic",
            basic_expanded,
            column![
                text("名称:").color(iced::Color::WHITE),
                text_input("名称", &form.new_name)
                    .on_input(Message::EditCardNameChanged)
                    .width(Length::Fill),
            ]
            .spacing(4),
        );
        
        let prediction_group = collapsible_section(
            "预测设置",
            "prediction",
            prediction_expanded,
            column![
                row![
                    text("预设:").color(iced::Color::WHITE),
                    pick_list(presets, Some(form.preset.clone()), Message::EditCardPresetChanged)
                        .width(Length::Fill),
                ]
                .spacing(8),
                Space::new().height(8),
                row![
                    text("下次复习:").color(iced::Color::WHITE),
                    text_input(
                        "YYYY-MM-DD",
                        &form.next_review.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
                    )
                    .on_input(Message::EditCardNextReviewChanged)
                    .width(Length::Fixed(150.0)),
                    button(text("清除").color(iced::Color::WHITE).size(12))
                        .on_press(Message::EditCardClearPrediction)
                        .style(|_, _| iced::widget::button::Style {
                            background: Some(iced::Color::from_rgb(0.5, 0.3, 0.3).into()),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(8),
                Space::new().height(8),
                checkbox("清除预测状态", form.clear_prediction)
                    .on_toggle(|_| Message::EditCardClearPrediction),
            ]
            .spacing(4),
        );
        
        let records_group = collapsible_section(
            &format!("复习记录 ({} 条)", form.review_records.len()),
            "records",
            records_expanded,
            column![
                column(form.review_records.iter().enumerate().map(|(i, r)| {
                    review_record_row(i, r)
                }))
                .spacing(4),
                Space::new().height(8),
                button(text("+ 添加记录").color(iced::Color::WHITE))
                    .on_press(Message::EditCardAddReview)
                    .style(|_, _| iced::widget::button::Style {
                        background: Some(iced::Color::from_rgb(0.3, 0.4, 0.5).into()),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(4),
        );
        
        let buttons = row![
            button(text("取消").color(iced::Color::WHITE))
                .on_press(Message::ModalClose)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.4, 0.4, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            button(text("保存").color(iced::Color::WHITE))
                .on_press(Message::EditCardConfirm)
                .style(|_, _| iced::widget::button::Style {
                    background: Some(iced::Color::from_rgb(0.3, 0.6, 0.4).into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(8)
        .width(Length::Fill)
        .push(Space::new().width(Length::Fill));
        
        column![
            basic_group,
            Space::new().height(8),
            prediction_group,
            Space::new().height(8),
            records_group,
            Space::new().height(24),
            buttons,
        ]
        .padding(16)
        .into()
    }
}

fn collapsible_section(
    title: &str,
    group_id: &str,
    expanded: bool,
    content: Element<Message>,
) -> Element<Message> {
    let icon = if expanded { "▼" } else { "▶" };
    let group_id_toggle = group_id.to_string();
    
    column![
        button(
            row![
                text(icon).color(iced::Color::WHITE),
                text(title).color(iced::Color::WHITE),
            ]
            .spacing(8)
        )
        .on_press(Message::EditCardToggleGroup(group_id_toggle))
        .style(|_, _| iced::widget::button::Style {
            background: Some(iced::Color::from_rgb(0.25, 0.25, 0.25).into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .width(Length::Fill),
        if expanded {
            column![content].padding(8)
        } else {
            column![]
        },
    ]
    .spacing(4)
    .into()
}

fn review_record_row(index: usize, record: &crate::data::models::ReviewRecord) -> Element<Message> {
    let timestamp = record.timestamp.format("%Y-%m-%d").to_string();
    let duration = record.duration_ms / 60000;
    let quality = record.memory_quality.as_str();
    
    row![
        text(timestamp).color(iced::Color::WHITE).width(Length::Fixed(100.0)),
        text(format!("{}分", duration)).color(iced::Color::WHITE).width(Length::Fixed(60.0)),
        text(quality).color(iced::Color::WHITE).width(Length::Fixed(60.0)),
        button(text("×").color(iced::Color::WHITE))
            .on_press(Message::EditCardRemoveReview(index))
            .style(|_, _| iced::widget::button::Style {
                background: Some(iced::Color::from_rgb(0.5, 0.3, 0.3).into()),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .spacing(8)
    .into()
}
```

### Step 2: 注册组件

```rust
// src/gui/components/mod.rs

mod edit_card_modal;

pub use edit_card_modal::EditCardModal;
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 7: 消息处理逻辑 - 新建节点

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 处理 NewNodeOpen 消息

```rust
// src/app/mod.rs - update 方法中

Message::NewNodeOpen(default_type) => {
    if let Some(ref path) = self.category_tab.selected_path {
        let form = NewNodeForm::new(path.clone(), default_type);
        self.category_tab.new_node_form = Some(form);
        self.modal = Some(Modal::NewNode);
    }
    Task::none()
}

Message::NewNodeNameChanged(name) => {
    if let Some(ref mut form) = self.category_tab.new_node_form {
        form.name = name;
    }
    Task::none()
}

Message::NewNodeTypeChanged(node_type) => {
    if let Some(ref mut form) = self.category_tab.new_node_form {
        form.node_type = node_type;
    }
    Task::none()
}

Message::NewNodePresetChanged(preset) => {
    if let Some(ref mut form) = self.category_tab.new_node_form {
        form.preset = preset;
    }
    Task::none()
}
```

### Step 2: 处理 NewNodeConfirm 消息

```rust
// src/app/mod.rs - update 方法中

Message::NewNodeConfirm => {
    if let Some(ref form) = self.category_tab.new_node_form {
        if form.name.is_empty() {
            self.error_message = Some("名称不能为空".to_string());
            return Task::none();
        }
        
        let new_path = format!("{}/{}", form.parent_path, form.name);
        
        let result = match form.node_type {
            NodeType::Folder => {
                self.data_fs.create_tree(&new_path)
            }
            NodeType::Card => {
                let card = Card::new_with_preset(form.preset.clone());
                self.data_fs.save_card(&new_path, &card)
            }
        };
        
        match result {
            Ok(()) => {
                self.modal = None;
                self.category_tab.new_node_form = None;
                
                // 重新加载数据
                let data_fs = Arc::new(self.data_fs.clone());
                return Task::future(async move {
                    let trees = data_fs.list_trees().unwrap_or_default();
                    let mut all_cards = Vec::new();
                    for tree in &trees {
                        let cards = data_fs.list_cards(tree).unwrap_or_default();
                        all_cards.extend(cards);
                    }
                    Message::DataLoaded(Ok(DataSnapshot {
                        trees,
                        cards: all_cards,
                        presets: Vec::new(),
                        todos: Vec::new(),
                    }))
                });
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }
    Task::none()
}
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 8: 消息处理逻辑 - 编辑卡片

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 处理 EditCardOpen 消息

```rust
// src/app/mod.rs - update 方法中

Message::EditCardOpen(path) => {
    match self.data_fs.get_card(&path) {
        Ok(card) => {
            let form = EditCardForm::new(path, &card);
            self.category_tab.edit_card_form = Some(form);
            self.modal = Some(Modal::EditCard);
        }
        Err(e) => {
            self.error_message = Some(e.to_string());
        }
    }
    Task::none()
}
```

### Step 2: 处理编辑表单消息

```rust
// src/app/mod.rs - update 方法中

Message::EditCardNameChanged(name) => {
    if let Some(ref mut form) = self.category_tab.edit_card_form {
        form.new_name = name;
    }
    Task::none()
}

Message::EditCardPresetChanged(preset) => {
    if let Some(ref mut form) = self.category_tab.edit_card_form {
        form.preset = preset;
    }
    Task::none()
}

Message::EditCardNextReviewChanged(date_str) => {
    if let Some(ref mut form) = self.category_tab.edit_card_form {
        form.next_review = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, Utc));
    }
    Task::none()
}

Message::EditCardClearPrediction => {
    if let Some(ref mut form) = self.category_tab.edit_card_form {
        form.clear_prediction = !form.clear_prediction;
        if form.clear_prediction {
            form.next_review = None;
        }
    }
    Task::none()
}

Message::EditCardToggleGroup(group) => {
    if let Some(ref mut form) = self.category_tab.edit_card_form {
        if form.expanded_groups.contains(&group) {
            form.expanded_groups.remove(&group);
        } else {
            form.expanded_groups.insert(group);
        }
    }
    Task::none()
}

Message::EditCardAddReview => {
    if let Some(ref mut form) = self.category_tab.edit_card_form {
        let duration: i64 = form.new_review_duration.parse().unwrap_or(0);
        if duration > 0 {
            let record = ReviewRecord {
                timestamp: Utc::now(),
                duration_ms: duration * 60000,
                memory_quality: form.new_review_quality.clone(),
            };
            form.review_records.push(record);
            form.new_review_duration.clear();
        }
    }
    Task::none()
}

Message::EditCardRemoveReview(index) => {
    if let Some(ref mut form) = self.category_tab.edit_card_form {
        if index < form.review_records.len() {
            form.review_records.remove(index);
        }
    }
    Task::none()
}
```

### Step 3: 处理 EditCardConfirm 消息

```rust
// src/app/mod.rs - update 方法中

Message::EditCardConfirm => {
    if let Some(ref form) = self.category_tab.edit_card_form {
        let mut card = Card::new();
        card.review_records = form.review_records.clone();
        
        if form.clear_prediction {
            card.prediction = None;
        } else if let Some(next_review) = form.next_review {
            if let Some(ref mut pred) = card.prediction {
                pred.next_review = next_review;
                pred.preset_used = form.preset.clone();
            } else {
                card.prediction = Some(Prediction {
                    algorithm: "fsrs".to_string(),
                    next_review,
                    fsrs_state_bytes: vec![],
                    preset_used: form.preset.clone(),
                });
            }
        }
        
        // 检查是否需要重命名
        if form.new_name != form.original_name {
            match self.data_fs.rename_card(&form.path, &form.new_name) {
                Ok(new_path) => {
                    if let Err(e) = self.data_fs.save_card(&new_path, &card) {
                        self.error_message = Some(e.to_string());
                        return Task::none();
                    }
                }
                Err(e) => {
                    self.error_message = Some(e.to_string());
                    return Task::none();
                }
            }
        } else {
            if let Err(e) = self.data_fs.save_card(&form.path, &card) {
                self.error_message = Some(e.to_string());
                return Task::none();
            }
        }
        
        self.modal = None;
        self.category_tab.edit_card_form = None;
        
        // 重新加载数据
        let data_fs = Arc::new(self.data_fs.clone());
        return Task::future(async move {
            let trees = data_fs.list_trees().unwrap_or_default();
            let mut all_cards = Vec::new();
            for tree in &trees {
                let cards = data_fs.list_cards(tree).unwrap_or_default();
                all_cards.extend(cards);
            }
            Message::DataLoaded(Ok(DataSnapshot {
                trees,
                cards: all_cards,
                presets: Vec::new(),
                todos: Vec::new(),
            }))
        });
    }
    Task::none()
}
```

### Step 4: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 9: 消息处理逻辑 - 删除节点

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 处理 DeleteNodeOpen 消息

```rust
// src/app/mod.rs - update 方法中

Message::DeleteNodeOpen(path, is_folder) => {
    self.category_tab.delete_target = Some((path.clone(), is_folder));
    self.modal = Some(Modal::ConfirmDelete { item: path });
    Task::none()
}
```

### Step 2: 处理 DeleteNodeConfirm 消息

```rust
// src/app/mod.rs - update 方法中

Message::DeleteNodeConfirm => {
    if let Some((path, is_folder)) = self.category_tab.delete_target.take() {
        let result = if is_folder {
            self.data_fs.delete_folder(&path)
        } else {
            self.data_fs.delete_card(&path)
        };
        
        match result {
            Ok(()) => {
                self.modal = None;
                self.category_tab.selected_path = None;
                
                // 重新加载数据
                let data_fs = Arc::new(self.data_fs.clone());
                return Task::future(async move {
                    let trees = data_fs.list_trees().unwrap_or_default();
                    let mut all_cards = Vec::new();
                    for tree in &trees {
                        let cards = data_fs.list_cards(tree).unwrap_or_default();
                        all_cards.extend(cards);
                    }
                    Message::DataLoaded(Ok(DataSnapshot {
                        trees,
                        cards: all_cards,
                        presets: Vec::new(),
                        todos: Vec::new(),
                    }))
                });
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }
    Task::none()
}
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 10: Modal 渲染集成

**Files:**
- Modify: `src/app/mod.rs`

### Step 1: 修改 view 方法中的 Modal 渲染

找到现有的 Modal 渲染代码，添加新的 Modal 类型：

```rust
// src/app/mod.rs - view 方法中

let modal_layer = if let Some(ref modal) = self.modal {
    let modal_content = match modal {
        Modal::NewCard { path } => {
            // 现有代码...
        }
        Modal::LinkTimer { timer_path, duration_ms } => {
            // 现有代码...
        }
        Modal::NewTodo => {
            // 现有代码...
        }
        Modal::EditTodo { id } => {
            // 现有代码...
        }
        Modal::NewSchedule => {
            // 现有代码...
        }
        Modal::PresetDetail { name } => {
            // 现有代码...
        }
        Modal::ConfirmDelete { item } => {
            // 现有删除确认...
            if self.category_tab.delete_target.is_some() {
                // 节点删除确认
                let (_, is_folder) = self.category_tab.delete_target.as_ref().unwrap();
                
                let warning = if *is_folder {
                    "⚠️ 文件夹及其所有内容将被删除"
                } else {
                    "⚠️ 此操作不可撤销"
                };
                
                column![
                    text(format!("确定要删除 \"{}\" 吗？", item))
                        .color(iced::Color::WHITE)
                        .size(16),
                    Space::new().height(12),
                    text(warning)
                        .color(iced::Color::from_rgb(0.9, 0.5, 0.3))
                        .size(14),
                    Space::new().height(24),
                    row![
                        button(text("取消").color(iced::Color::WHITE))
                            .on_press(Message::ModalClose)
                            .style(|_, _| iced::widget::button::Style {
                                background: Some(iced::Color::from_rgb(0.4, 0.4, 0.4).into()),
                                text_color: iced::Color::WHITE,
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                        button(text("删除").color(iced::Color::WHITE))
                            .on_press(Message::DeleteNodeConfirm)
                            .style(|_, _| iced::widget::button::Style {
                                background: Some(iced::Color::from_rgb(0.7, 0.3, 0.3).into()),
                                text_color: iced::Color::WHITE,
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    ]
                    .spacing(8),
                ]
                .padding(24)
                .into()
            } else {
                // 现有的待办删除确认代码...
                column![].into()
            }
        }
        Modal::Error { message } => {
            // 现有代码...
        }
        
        // 新增 Modal 类型
        Modal::NewNode => {
            if let Some(ref form) = self.category_tab.new_node_form {
                let presets: Vec<String> = self.preset_tab.presets.iter()
                    .map(|p| p.name.clone())
                    .collect();
                NewNodeModal::view(form, &presets)
            } else {
                column![].into()
            }
        }
        
        // 注意：EditCard 需要在 Modal 枚举中添加
        _ => column![].into(),
    };
    
    Some(ModalView::view(modal_content, Message::ModalClose))
} else {
    None
};
```

### Step 2: 扩展 Modal 枚举

```rust
// src/gui/messages.rs

#[derive(Debug, Clone)]
pub enum Modal {
    // 现有...
    NewCard { path: String },
    LinkTimer { timer_path: PathBuf, duration_ms: i64 },
    NewTodo,
    EditTodo { id: Uuid },
    NewSchedule,
    PresetDetail { name: String },
    ConfirmDelete { item: String },
    Error { message: String },
    
    // 新增
    NewNode,
    EditCard { path: String },
}
```

### Step 3: 验证编译

```bash
cargo build --release
```

Expected: 编译成功

---

## Task 11: 测试验证

### Step 1: 构建测试

```bash
cargo build --release
```

Expected: 编译成功，无错误

### Step 2: 运行 GUI 测试

```bash
TMD_DATA_DIR=/tmp/tmd-test-data cargo run --bin tmd-gui
```

Manual Test Checklist:
- [ ] 选中文件夹，右侧显示 [新建卡片] [新建文件夹] [删除] 按钮
- [ ] 选中学习卡片，右侧显示 [编辑] [删除] 按钮
- [ ] 点击 [新建卡片] 打开新建节点 Modal，类型预设为卡片
- [ ] 点击 [新建文件夹] 打开新建节点 Modal，类型预设为文件夹
- [ ] 新建节点 Modal 显示父节点路径
- [ ] 类型切换时，预设选择器显示/隐藏
- [ ] 创建成功后，左侧树显示新节点
- [ ] 点击 [编辑] 打开编辑卡片 Modal
- [ ] 编辑卡片 Modal 三个分组可折叠/展开
- [ ] 修改名称后保存，左侧树更新
- [ ] 添加/删除复习记录后保存，数据更新
- [ ] 点击 [删除] 打开删除确认 Modal
- [ ] 删除文件夹显示警告信息
- [ ] 删除成功后，清除选中状态，左侧树更新

---

## 验收标准

- [ ] DataFs 新增 delete_card、delete_folder、rename_card 方法
- [ ] 消息类型支持新建、编辑、删除操作
- [ ] 右侧面板显示操作按钮（学习卡片和文件夹不同）
- [ ] 新建节点 Modal 功能完整
- [ ] 编辑卡片 Modal 分组折叠功能正常
- [ ] 删除确认 Modal 显示警告
- [ ] 所有操作后数据正确更新

---

## 时间估计

| 任务 | 时间 |
|------|------|
| Task 1: DataFs 扩展 | 20 分钟 |
| Task 2: 消息类型扩展 | 15 分钟 |
| Task 3: 表单状态结构 | 20 分钟 |
| Task 4: 右侧面板操作按钮 | 30 分钟 |
| Task 5: 新建节点 Modal | 40 分钟 |
| Task 6: 编辑卡片 Modal | 60 分钟 |
| Task 7: 新建节点消息处理 | 30 分钟 |
| Task 8: 编辑卡片消息处理 | 45 分钟 |
| Task 9: 删除节点消息处理 | 20 分钟 |
| Task 10: Modal 渲染集成 | 30 分钟 |
| Task 11: 测试验证 | 30 分钟 |
| **总计** | **约 5.5 小时** |
