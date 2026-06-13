# GUI 分类树节点操作设计文档

> **日期**: 2026-06-13
> **状态**: 设计完成
> **基于**: DESIGN_v3.md §4.3.1 + brainstorming 探索

---

## 1. 功能概述

**目标**: 为分类树节点提供完整的 CRUD 操作能力，支持学习卡片和文件夹的管理。

**核心需求**:
- 学习卡片：查看详情、编辑、删除
- 文件夹：创建子节点（卡片/文件夹）、删除文件夹
- 操作入口：右侧详情面板按钮 + 右键菜单（Iced 限制，用操作按钮替代）

---

## 2. 界面布局

### 2.1 右侧详情面板（学习卡片）

```
┌──────────────────────────────────┐
│ 路径: 语言/英语/六级/单词        │
│                                  │
│ [编辑] [删除]        ← 操作按钮  │
│ ─────────────────────────       │
│ 复习记录: 5 次                    │
│ 下次复习: 2026-06-15             │
│ 预设: default                    │
│ 算法: fsrs                       │
└──────────────────────────────────┘
```

**操作按钮**:
- 编辑：打开编辑卡片 Modal
- 删除：打开删除确认 Modal

### 2.2 右侧详情面板（文件夹）

```
┌──────────────────────────────────┐
│ 路径: 语言/英语/六级              │
│                                  │
│ [新建卡片] [新建文件夹] [删除]    │
│ ─────────────────────────       │
│ 子项数量: 3 个                   │
│ (选中具体节点查看详情)            │
└──────────────────────────────────┘
```

**操作按钮**:
- 新建卡片：打开新建节点 Modal（类型预设为卡片）
- 新建文件夹：打开新建节点 Modal（类型预设为文件夹）
- 删除：打开删除确认 Modal

---

## 3. Modal 设计

### 3.1 新建节点 Modal

```
┌────────────────────────────────────┐
│ 新建节点                      [×]  │
├────────────────────────────────────┤
│ 在 "语言/英语/六级" 下创建         │
│                                    │
│ 类型: ○ 文件夹  ● 学习卡片        │
│                                    │
│ 名称: [________________]          │
│                                    │
│ 预设: [default          ▼]        │
│     (仅卡片类型时显示)             │
│                                    │
│              [取消]  [创建]        │
└────────────────────────────────────┘
```

**字段**:
- 路径预览：显示父节点路径
- 类型选择：单选按钮（文件夹 / 学习卡片）
- 名称输入：文本输入框
- 预设选择：下拉菜单（仅卡片类型时显示）

**默认值**:
- 类型：学习卡片
- 预设：用户配置的默认预设（无配置则为 "default"）

### 3.2 编辑卡片 Modal

```
┌────────────────────────────────────┐
│ 编辑卡片                      [×]  │
├────────────────────────────────────┤
│ ▼ 基本信息                         │
│   名称: [单词-Unit1_____]         │
│                                    │
│ ▼ 预测设置 ─────────────────────   │
│   预设: [default        ▼]         │
│   下次复习: [2026-06-15] [清除]    │
│   算法状态: ● 保留 ○ 清除          │
│                                    │
│ ▶ 复习记录 (5 条)                  │
│                                    │
│              [取消]  [保存]        │
└────────────────────────────────────┘
```

**展开复习记录**：

```
│ ▼ 复习记录 (5 条) ───────────────  │
│   ┌────────────────────────────┐  │
│   │ 2026-06-01 │ 45分 │ 好   [×]│  │
│   ├────────────────────────────┤  │
│   │ 2026-05-28 │ 30分 │ 困难[×]│  │
│   └────────────────────────────┘  │
│   [+ 添加记录]                     │
```

**分组说明**:

| 分组 | 可折叠 | 字段 |
|------|--------|------|
| 基本信息 | 是 | 名称（文件名） |
| 预测设置 | 是 | 预设、下次复习时间、算法状态 |
| 复习记录 | 是 | 记录列表（时间、时长、记忆表现） |

**复习记录操作**:
- 添加：点击 [+ 添加记录] 打开新记录表单
- 删除：点击记录右侧 [×] 按钮
- 新记录表单：时间（默认当前）、时长（分钟）、记忆表现（下拉）

**预设切换**:
- 切换预设时，保留现有预测状态
- 用户可手动清除预测状态（重新开始）

### 3.3 删除确认 Modal

```
┌────────────────────────────────────┐
│ 确认删除                      [×]  │
├────────────────────────────────────┤
│                                    │
│ 确定要删除 "单词-Unit1" 吗？       │
│                                    │
│ ⚠️ 此操作不可撤销                 │
│                                    │
│              [取消]  [删除]        │
└────────────────────────────────────┘
```

**删除文件夹**:

```
│ 确定要删除文件夹 "六级" 吗？       │
│                                    │
│ ⚠️ 文件夹包含 3 个子项，将一并删除│
│                                    │
│              [取消]  [删除]        │
```

---

## 4. 数据模型

### 4.1 Modal 状态

```rust
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
    NewNode { parent_path: String, default_type: NodeType },
    EditCard { path: String, card: Card },
    DeleteNode { path: String, is_folder: bool, children_count: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Folder,
    Card,
}
```

### 4.2 表单状态

```rust
pub struct NewNodeForm {
    pub parent_path: String,
    pub node_type: NodeType,
    pub name: String,
    pub preset: String,
}

pub struct EditCardForm {
    pub path: String,
    pub original_name: String,
    pub new_name: String,
    pub preset: String,
    pub next_review: Option<DateTime<Utc>>,
    pub clear_prediction: bool,
    pub review_records: Vec<ReviewRecord>,
    pub expanded_groups: HashSet<String>,
}

pub struct NewReviewRecordForm {
    pub timestamp: DateTime<Utc>,
    pub duration_minutes: i32,
    pub memory_quality: MemoryQuality,
}
```

---

## 5. 消息类型

```rust
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
    
    // 删除
    DeleteNodeOpen(String, bool),
    DeleteNodeConfirm,
}
```

---

## 6. 数据流

### 6.1 新建节点流程

```
用户点击 [新建卡片] 或 [新建文件夹]
    ↓
Message::NewNodeOpen(node_type)
    ↓
设置 modal = Some(Modal::NewNode { parent_path, default_type })
初始化 NewNodeForm
    ↓
用户填写表单
    ↓
Message::NewNodeConfirm
    ↓
调用 DataFs 方法：
  - 文件夹: create_tree(path)
  - 卡片: save_card(path, Card::new_with_preset(preset))
    ↓
重新加载数据（Task::future）
    ↓
关闭 Modal
```

### 6.2 编辑卡片流程

```
用户点击 [编辑]
    ↓
Message::EditCardOpen(path)
    ↓
加载卡片数据：get_card(path)
设置 modal = Some(Modal::EditCard { path, card })
初始化 EditCardForm
    ↓
用户修改表单
    ↓
Message::EditCardConfirm
    ↓
检查名称是否变更：
  - 变更: rename_card(old_path, new_path)
  - 不变: 跳过
    ↓
更新卡片数据：save_card(path, updated_card)
    ↓
重新加载数据
    ↓
关闭 Modal
```

### 6.3 删除节点流程

```
用户点击 [删除]
    ↓
Message::DeleteNodeOpen(path, is_folder)
    ↓
检查子项数量（如果是文件夹）
设置 modal = Some(Modal::DeleteNode { path, is_folder, children_count })
    ↓
用户确认
    ↓
Message::DeleteNodeConfirm
    ↓
调用 DataFs 方法：
  - 卡片: delete_card(path)
  - 文件夹: delete_folder(path)
    ↓
清除选中状态：selected_path = None
重新加载数据
    ↓
关闭 Modal
```

---

## 7. DataFs 扩展

### 7.1 新增方法

```rust
impl DataFs {
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
    
    /// 重命名卡片（移动文件）
    pub fn rename_card(&self, old_path: &str, new_name: &str) -> Result<String> {
        let (tree, old_card_path) = Self::parse_path(old_path)?;
        
        // 解析新路径
        let old_path_buf = std::path::Path::new(old_card_path);
        let parent = old_path_buf.parent()
            .ok_or_else(|| DataError::InvalidPath(old_path.into()))?;
        let new_card_path = parent.join(new_name);
        let new_card_path_str = new_card_path.to_string_lossy();
        let new_full_path = format!("{}/{}", tree, new_card_path_str);
        
        // 移动文件
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
        
        std::fs::rename(&old_file, &new_file)
            .map_err(|e| DataError::Io(e.to_string()))?;
        
        Ok(new_full_path)
    }
}
```

### 7.2 新增错误类型

```rust
pub enum DataError {
    // 现有...
    CardNotFound(String),
    FolderNotFound(String),
    
    // 新增
    InvalidNodeName(String),  // 名称包含非法字符
    NodeAlreadyExists(String), // 节点已存在
}
```

---

## 8. UI 组件

### 8.1 操作按钮区域

```rust
// 学习卡片操作按钮
let card_actions = row![
    button("编辑")
        .on_press(Message::EditCardOpen(path.clone())),
    button("删除")
        .on_press(Message::DeleteNodeOpen(path.clone(), false)),
].spacing(8);

// 文件夹操作按钮
let folder_actions = row![
    button("新建卡片")
        .on_press(Message::NewNodeOpen(NodeType::Card)),
    button("新建文件夹")
        .on_press(Message::NewNodeOpen(NodeType::Folder)),
    button("删除")
        .on_press(Message::DeleteNodeOpen(path.clone(), true)),
].spacing(8);
```

### 8.2 新建节点 Modal 组件

```rust
pub struct NewNodeModal;

impl NewNodeModal {
    pub fn view(form: &NewNodeForm, presets: &[String]) -> Element<Message> {
        let type_selector = row![
            radio("文件夹", NodeType::Folder, Some(form.node_type), 
                  Message::NewNodeTypeChanged),
            radio("学习卡片", NodeType::Card, Some(form.node_type),
                  Message::NewNodeTypeChanged),
        ].spacing(16);
        
        let preset_selector = if form.node_type == NodeType::Card {
            Some(pick_list(presets, Some(form.preset.clone()), 
                          Message::NewNodePresetChanged))
        } else {
            None
        };
        
        // ... 布局代码
    }
}
```

### 8.3 编辑卡片 Modal 组件

```rust
pub struct EditCardModal;

impl EditCardModal {
    pub fn view(form: &EditCardForm, presets: &[String]) -> Element<Message> {
        column![
            // 基本信息分组（可折叠）
            collapsible_group("基本信息", "basic", &form.expanded_groups, || {
                column![
                    text_input("名称", &form.new_name)
                        .on_input(Message::EditCardNameChanged),
                ]
            }),
            
            // 预测设置分组
            collapsible_group("预测设置", "prediction", &form.expanded_groups, || {
                column![
                    pick_list(presets, Some(form.preset.clone()),
                             Message::EditCardPresetChanged),
                    row![
                        text_input("下次复习", &form.next_review.map(|d| 
                            d.format("%Y-%m-%d").to_string()).unwrap_or_default()),
                        button("清除").on_press(Message::EditCardClearPrediction),
                    ],
                ]
            }),
            
            // 复习记录分组
            collapsible_group("复习记录", "records", &form.expanded_groups, || {
                column![
                    // 记录列表
                    column(form.review_records.iter().enumerate().map(|(i, r)| {
                        review_record_row(i, r)
                    })),
                    button("+ 添加记录").on_press(Message::EditCardAddReview),
                ]
            }),
        ].into()
    }
}
```

---

## 9. 实现清单

### Task 1: DataFs 扩展
- 新增 `delete_card()` 方法
- 新增 `delete_folder()` 方法
- 新增 `rename_card()` 方法
- 新增错误类型

### Task 2: 消息类型扩展
- 新增新建节点相关消息
- 新增编辑卡片相关消息
- 新增删除节点相关消息

### Task 3: 表单状态结构
- 新增 `NewNodeForm` 结构
- 新增 `EditCardForm` 结构
- 新增 `NodeType` 枚举

### Task 4: 右侧面板操作按钮
- 学习卡片显示 [编辑] [删除] 按钮
- 文件夹显示 [新建卡片] [新建文件夹] [删除] 按钮
- 按钮样式统一

### Task 5: 新建节点 Modal
- 路径预览显示
- 类型选择（单选按钮）
- 名称输入框
- 预设选择（下拉，仅卡片）
- 确认/取消按钮

### Task 6: 编辑卡片 Modal
- 分组折叠组件
- 基本信息组
- 预测设置组
- 复习记录组（列表 + 添加 + 删除）
- 保存/取消按钮

### Task 7: 删除确认 Modal
- 扩展现有 `ConfirmDelete` Modal
- 文件夹显示子项数量警告

### Task 8: 消息处理逻辑
- 新建节点确认逻辑
- 编辑卡片保存逻辑
- 删除节点确认逻辑
- 数据重新加载

---

## 10. 验收标准

- [ ] 学习卡片详情页显示 [编辑] [删除] 按钮
- [ ] 文件夹详情页显示 [新建卡片] [新建文件夹] [删除] 按钮
- [ ] 点击 [新建卡片] 打开新建节点 Modal（类型预设为卡片）
- [ ] 点击 [新建文件夹] 打开新建节点 Modal（类型预设为文件夹）
- [ ] 新建节点 Modal 显示父节点路径预览
- [ ] 新建节点 Modal 类型切换时，预设选择器显示/隐藏
- [ ] 新建节点成功后，左侧树自动显示新节点
- [ ] 点击 [编辑] 打开编辑卡片 Modal
- [ ] 编辑卡片 Modal 显示三个分组（基本信息、预测设置、复习记录）
- [ ] 分组可折叠/展开
- [ ] 修改卡片名称后保存，左侧树更新显示
- [ ] 修改预设、预测时间后保存，右侧详情更新
- [ ] 添加/删除复习记录后保存，数据更新
- [ ] 点击 [删除] 打开删除确认 Modal
- [ ] 删除文件夹时显示子项数量警告
- [ ] 删除成功后，清除选中状态，左侧树更新
- [ ] 所有删除操作均需确认

---

## 11. 时间估计

| 任务 | 时间 |
|------|------|
| Task 1: DataFs 扩展 | 20 分钟 |
| Task 2: 消息类型 | 10 分钟 |
| Task 3: 表单状态 | 15 分钟 |
| Task 4: 操作按钮 | 20 分钟 |
| Task 5: 新建节点 Modal | 40 分钟 |
| Task 6: 编辑卡片 Modal | 60 分钟 |
| Task 7: 删除确认 Modal | 15 分钟 |
| Task 8: 消息处理 | 40 分钟 |
| **总计** | **约 3.5 小时** |

---

## 12. 后续优化

**暂不实现**:
- 拖拽移动节点
- 批量操作
- 撤销/重做
- 节点搜索高亮

**理由**: 保持 MVP 简洁，后续根据用户反馈迭代。
