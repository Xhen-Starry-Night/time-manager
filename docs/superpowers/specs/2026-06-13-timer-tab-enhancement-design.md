# GUI Phase 5 设计文档 - 计时器界面完善

**日期**: 2026-06-13  
**状态**: 设计阶段  
**基于**: Phase 1-4 已完成实现

---

## 1. 概述

### 1.1 目标

完善计时器界面，实现完整的计时器功能：
- 计时器实时显示与控制
- 计时结束后的卡片链接流程
- 计时器历史记录查看
- 计时器与卡片学习的深度集成

### 1.2 当前状态

Phase 4 已完成：
- 计时器基础显示（TimerDisplay 组件）
- 开始/暂停/停止按钮
- 计时器 Subscription 实时更新
- 基础计时器状态管理

### 1.3 Phase 5 新增功能

1. **计时器显示优化**：时钟占窗口高度1/3，宽度2/3，从卡片跳转时显示路径（无"当前学习："前缀）
2. **计时结束流程**：计时停止后弹出链接模态框，支持手动输入、下拉选择和创建新卡片
3. **记忆质量评估**：记录本次学习的记忆质量
4. **计时记录管理**：完整的计时记录列表，支持增删改查和链接
5. **快捷计时**：从分类树/复习看板直接启动计时

---

## 2. 架构设计

### 2.1 模块结构

```
src/
├── app/
│   ├── mod.rs              # 主应用（新增计时器消息处理）
│   ├── timer_tab.rs        # 计时器 Tab 状态扩展
│   └── category_tab.rs     # 分类树（添加快捷计时入口）
├── gui/
│   ├── messages.rs          # 消息枚举（新增计时器消息）
│   └── components/
│       ├── timer_display.rs # 计时器显示（增强）
│       └── link_timer_modal.rs # 计时器链接模态框（新增）
└── timer/
    └── mod.rs              # TimerManager（扩展）
```

### 2.2 状态扩展

```rust
// src/app/timer_tab.rs
pub struct TimerTabState {
    pub state: TimerState,
    pub elapsed_ms: i64,
    pub link_mode: bool,              // 是否处于链接模式
    pub card_path_input: String,      // 卡片路径输入
    pub card_dropdown: Vec<String>,  // 已有卡片列表（下拉选择）
    pub selected_card: Option<String>, // 下拉选择的卡片
    pub memory_quality: MemoryQuality, // 记忆质量选择
    pub show_history: bool,          // 是否显示计时记录界面
    pub timer_history: Vec<TimerRecord>, // 计时器历史记录
    pub current_card: Option<String>, // 当前学习的卡片路径（快捷计时）
    pub show_create_new: bool,       // 是否显示创建新卡片界面
}

pub struct TimerRecord {
    pub start_time: DateTime<Utc>,
    pub duration_ms: i64,
    pub linked_card: Option<String>,
    pub memory_quality: Option<MemoryQuality>,
}
```

### 2.3 新增消息

```rust
// src/gui/messages.rs
pub enum Message {
    // ... 现有消息 ...
    
    // 计时器控制（已存在）
    TimerStarted,
    TimerPaused,
    TimerStopped(Result<PathBuf, String>),
    TimerTick(i64),
    
    // Phase 5 新增
    TimerLinkModeOpen,              // 打开链接模式
    TimerLinkModeClose,             // 关闭链接模式
    TimerCardPathChanged(String),   // 卡片路径输入变化
    TimerCardSelected(String),      // 下拉选择卡片
    TimerMemoryQualityChanged(MemoryQuality), // 记忆质量选择
    TimerLinkConfirm,               // 确认链接
    TimerCreateNewCard,             // 创建新卡片
    
    // 计时记录管理
    TimerHistoryLoad,               // 加载计时记录
    TimerHistoryLoaded(Vec<TimerRecord>),
    TimerHistoryEdit(Uuid),         // 编辑计时记录
    TimerHistoryDelete(Uuid),       // 删除计时记录
    TimerHistoryUpdate(TimerRecord),  // 更新计时记录
    TimerHistoryShow,               // 显示计时记录界面
    TimerHistoryHide,               // 隐藏计时记录界面
    
    // 快捷计时（从分类树/复习看板）
    QuickTimerStart(String),        // 为指定卡片启动计时
}
```

---

## 3. 界面设计

### 3.1 计时器主界面

**计时中状态（从卡片跳转）**:
```
┌────────────────────────────────────────────────────────────┐
│ 计时器                                          [计时记录]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│                                                            │
│                                                            │
│                     00:05:32                               │
│                                                            │
│                   [暂停]    [停止]                          │
│                                                            │
│  knowledge/数学/微积分                                     │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**计时中状态（直接进入）**:
```
┌────────────────────────────────────────────────────────────┐
│ 计时器                                          [计时记录]  │
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

**空闲状态**:
```
┌────────────────────────────────────────────────────────────┐
│ 计时器                                          [计时记录]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│                                                            │
│                                                            │
│                     00:00:00                               │
│                                                            │
│                   [开始计时]                                │
│                                                            │
│                                                            │
│                                                            │
│                                                            │
│                                                            │
│                                                            │
│                                                            │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### 3.2 计时器链接模态框

当计时停止时，弹出模态框：

```
┌────────────────────────────────────────────────────────────┐
│ 计时完成                                                    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ 学习时长: 5 分钟 32 秒                                     │
│                                                            │
│ ─────────────────────────────────────────────────────────  │
│                                                            │
│ 关联到卡片:                                                │
│ [knowledge/数学/微积分                    ] [浏览...]      │
│ 或                                                         │
│ [▼ 选择已有卡片...                        ]                  │
│                                                            │
│ 记忆质量:                                                  │
│   [重学] [困难] [好] [简单]                                │
│                                                            │
│                              [取消]  [保存并预测]           │
└────────────────────────────────────────────────────────────┘
```
┌────────────────────────────────────────────────────────────┐
│ 计时器                                          [历史记录]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│                                                            │
│                                                            │
│                     00:05:32                               │
│                                                            │
│                   [暂停]    [停止]                          │
│                                                            │
│  当前学习: knowledge/数学/微积分                           │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**已停止状态（链接模式）**:
```
┌────────────────────────────────────────────────────────────┐
│ 计时结束                                                    │
├────────────────────────────────────────────────────────────┤
│ 有效时长: 5 分钟 32 秒                                     │
│                                                            │
│ ─────────────────────────────────────────────────────────  │
│                                                            │
│ [创建新卡片]  [链接到现有卡片]  [跳过]                      │
│                                                            │
│ ─────────────────────────────────────────────────────────  │
│                                                            │
│ 卡片路径: [knowledge/数学/微积分    ]                      │
│                                                            │
│ 记忆表现:                                                  │
│   [重学] [困难] [好] [简单]                                │
│                                                            │
│                              [取消]  [保存]                 │
└────────────────────────────────────────────────────────────┘
```

**空闲状态（显示历史）**:
```
┌────────────────────────────────────────────────────────────┐
│ 计时器                                          [历史记录]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│                     00:00:00                               │
│                                                            │
│                   [开始计时]                                │
│                                                            │
│ ─────────────────────────────────────────────────────────  │
│                                                            │
│ 最近计时记录:                                              │
│ ┌───────────────────────────────────────────────────────┐  │
│ │ 2026-06-13 10:37  5m 32s  微积分  [好]                │  │
│ │ 2026-06-13 09:15  3m 12s  语法     [困难]             │  │
│ │ 2026-06-12 20:00  10m 5s  Rust    [简单]             │  │
│ └───────────────────────────────────────────────────────┘  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### 3.2 计时器链接模态框

当计时停止时，弹出模态框：

```
┌────────────────────────────────────────────────────────────┐
│ 计时完成                                                    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ 学习时长: 5 分钟 32 秒                                     │
│                                                            │
│ ─────────────────────────────────────────────────────────  │
│                                                            │
│ 关联到卡片:                                                │
│ [knowledge/数学/微积分                    ] [浏览...]    │
│                                                            │
│ 记忆质量:                                                  │
│   ○ 重学  ○ 困难  ● 好  ○ 简单                            │
│                                                            │
│                              [取消]  [保存并预测]           │
└────────────────────────────────────────────────────────────┘
```

### 3.3 计时记录界面

完整的计时记录管理界面：

```
┌────────────────────────────────────────────────────────────┐
│ 计时记录                                          [返回]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ ┌───────────────────────────────────────────────────────┐│
│ │ 2026-06-13 10:37  5m 32s  微积分  [好]  [编辑] [删除]  ││
│ │ 2026-06-13 09:15  3m 12s  语法     [困难] [编辑] [删除]││
│ │ 2026-06-12 20:00  10m 5s  --      --    [编辑] [删除] ││
│ └───────────────────────────────────────────────────────┘│
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**编辑计时记录模态框**:
```
┌────────────────────────────────────────────────────────────┐
│ 编辑计时记录                                                │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ 时间: [2026-06-13 10:37:31              ]                  │
│                                                            │
│ 时长: [5] 分钟 [32] 秒                                    │
│                                                            │
│ 关联卡片:                                                 │
│ [knowledge/数学/微积分                    ] [浏览...]     │
│ 或                                                        │
│ [▼ 选择已有卡片...                        ]                 │
│ 或                                                        │
│ [创建新卡片...]                                           │
│                                                            │
│ 记忆质量:                                                 │
│   [重学] [困难] [好] [简单]                               │
│                                                            │
│                              [取消]  [保存]                 │
└────────────────────────────────────────────────────────────┘
```

**创建新卡片模态框**（参考分类树创建界面）:
```
┌────────────────────────────────────────────────────────────┐
│ 创建新卡片                                                  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ 名称: [微积分复习                          ]                │
│                                                            │
│ 路径: [knowledge/数学/微积分复习            ]                │
│                                                            │
│ 预设: [default              ▼]                             │
│                                                            │
│                              [取消]  [创建]                 │
└────────────────────────────────────────────────────────────┘
```

---

## 4. 交互流程

### 4.1 完整计时流程

```
[开始计时] → [计时中...] → [停止计时] → [弹出链接模态框]
                                              ↓
[创建新卡片] ← [链接到现有卡片] ← [选择记忆质量] → [保存]
                                              ↓
                                        [保存到卡片复习记录]
                                              ↓
                                        [触发 FSRS 重新预测]
                                              ↓
                                        [返回计时器界面]
```

### 4.2 快捷计时流程

从分类树或复习看板点击"开始计时":
```
[点击开始计时] → [切换到计时器 Tab] → [自动开始计时]
                      ↓
              [记录当前卡片路径]
                      ↓
              [计时停止时自动填充路径]
```

### 4.3 计时记录管理流程

```
[点击计时记录] → [显示计时记录列表]
                      ↓
              [编辑] → [修改时长/卡片/质量/时间]
                      ↓
              [删除] → [确认删除] → [硬删除记录]
                      ↓
              [链接] → [选择或创建卡片]
```

### 4.3 消息处理流程

```rust
// 计时停止
Message::TimerStopped(result) => {
    match result {
        Ok(path) => {
            self.timer_manager.stop();
            self.timer_tab.state = TimerState::Idle;
            self.timer_tab.elapsed_ms = 0;
            
            // 如果有预设卡片路径，进入链接模式
            if let Some(card_path) = self.timer_tab.current_card.take() {
                self.timer_tab.link_mode = true;
                self.timer_tab.card_path_input = card_path;
            }
        }
        Err(e) => {
            self.error_message = Some(e);
        }
    }
    Task::none()
}

// 确认链接
Message::TimerLinkConfirm => {
    if let Some(form) = &self.timer_tab.edit_form {
        let record = ReviewRecord {
            timestamp: Utc::now(),
            duration_ms: form.duration_ms,
            memory_quality: form.memory_quality.clone(),
        };
        
        // 保存到卡片
        if let Ok(mut card) = self.data_fs.get_card(&form.card_path) {
            card.review_records.push(record);
            
            // 触发 FSRS 预测
            if let Ok(predictor) = FsrsPredictor::new() {
                if let Ok((next_review, new_state)) = predictor.predict_from_records(
                    &card.review_records,
                    None,
                    0.9
                ) {
                    card.prediction = Some(Prediction {
                        algorithm: "fsrs".to_string(),
                        next_review,
                        fsrs_state_bytes: FsrsPredictor::memory_state_to_bytes(&new_state),
                        preset_used: "default".to_string(),
                    });
                }
            }
            
            let _ = self.data_fs.save_card(&form.card_path, &card);
        }
        
        self.timer_tab.link_mode = false;
    }
    Task::none()
}
```

---

## 5. 数据模型

### 5.1 TimerRecord

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerRecord {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: i64,
    pub linked_card: Option<String>,
    pub memory_quality: Option<MemoryQuality>,
    pub created_at: DateTime<Utc>,
}
```

### 5.2 与 Card 的关联

计时记录保存为卡片的 ReviewRecord:

```rust
pub struct ReviewRecord {
    pub timestamp: DateTime<Utc>,   // 计时结束时间
    pub duration_ms: i64,           // 学习时长
    pub memory_quality: MemoryQuality, // 记忆质量评估
}
```

---

## 6. 实现细节

### 6.1 TimerManager 扩展

```rust
impl TimerManager {
    // 设置当前学习的卡片路径
    pub fn set_current_card(&mut self, path: Option<String>) {
        self.current_card = path;
    }
    
    // 获取当前卡片路径
    pub fn get_current_card(&self) -> Option<String> {
        self.current_card.clone()
    }
    
    // 保存计时记录到卡片
    pub fn save_timer_record(
        &self,
        card_path: &str,
        duration_ms: i64,
        quality: MemoryQuality,
    ) -> Result<(), DataError> {
        let mut card = self.data_fs.get_card(card_path)?;
        
        card.review_records.push(ReviewRecord {
            timestamp: Utc::now(),
            duration_ms,
            memory_quality: quality,
        });
        
        self.data_fs.save_card(card_path, &card)
    }
}
```

### 6.2 计时器历史存储

计时器历史存储在单独的文件中:

```
data/
└── timers/
    └── history.json        # 计时器历史记录
```

```rust
impl DataFs {
    pub fn save_timer_record(&self, record: &TimerRecord) -> Result<(), DataError> {
        let path = self.data_dir.join("timers").join("history.json");
        let mut records = self.load_timer_history()?;
        records.push(record.clone());
        
        let json = serde_json::to_string_pretty(&records)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load_timer_history(&self) -> Result<Vec<TimerRecord>, DataError> {
        let path = self.data_dir.join("timers").join("history.json");
        if !path.exists() {
            return Ok(vec![]);
        }
        
        let content = std::fs::read_to_string(path)?;
        let records: Vec<TimerRecord> = serde_json::from_str(&content)?;
        Ok(records)
    }
    
    pub fn delete_timer_record(&self, id: Uuid) -> Result<(), DataError> {
        let path = self.data_dir.join("timers").join("history.json");
        let mut records = self.load_timer_history()?;
        records.retain(|r| r.id != id);
        
        let json = serde_json::to_string_pretty(&records)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn update_timer_record(&self, record: &TimerRecord) -> Result<(), DataError> {
        let path = self.data_dir.join("timers").join("history.json");
        let mut records = self.load_timer_history()?;
        if let Some(idx) = records.iter().position(|r| r.id == record.id) {
            records[idx] = record.clone();
        }
        
        let json = serde_json::to_string_pretty(&records)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

---

## 7. 界面组件

### 7.1 TimerDisplay 增强

```rust
pub struct TimerDisplay;

impl TimerDisplay {
    pub fn view(state: &TimerState, elapsed_ms: i64, current_card: Option<&str>) -> Element<Message> {
        let time_text = format_elapsed(elapsed_ms);
        
        column![
            // 时间显示 - 大尺寸，占窗口高度1/3，宽度2/3
            container(
                text(time_text)
                    .size(72)  // 大字体
                    .color(Color::WHITE)
            )
            .width(Length::FillPortion(2))
            .height(Length::FillPortion(1))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
            
            // 当前学习卡片路径（如果有）
            if let Some(card) = current_card {
                container(
                    text(card)
                        .size(14)
                        .color(Color::from_rgb(0.7, 0.7, 0.7))
                )
                .width(Length::Fill)
                .center_x(Length::Fill)
            } else {
                container(Space::new().height(20))
                    .width(Length::Fill)
            },
            
            // 控制按钮
            row![
                start_button(state),
                pause_button(state),
                stop_button(state),
            ]
            .spacing(12)
            .width(Length::Fill)
            .center_x(Length::Fill),
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
```

### 7.2 LinkTimerModal 新增

```rust
pub struct LinkTimerModal {
    pub duration_ms: i64,
    pub card_path: String,
    pub card_dropdown: Vec<String>,  // 已有卡片列表
    pub selected_card: Option<String>,
    pub memory_quality: MemoryQuality,
    pub show_create_new: bool,       // 是否显示创建新卡片界面
}

impl LinkTimerModal {
    pub fn view(&self) -> Element<Message> {
        container(
            column![
                text("计时完成").size(20).color(Color::WHITE),
                
                text(format!("学习时长: {}", format_duration(self.duration_ms)))
                    .size(16)
                    .color(Color::WHITE),
                
                Space::new().height(16),
                
                // 卡片路径输入（手动输入）
                row![
                    text("关联到卡片:").color(Color::WHITE),
                    text_input("输入卡片路径...", &self.card_path)
                        .on_input(Message::TimerCardPathChanged),
                ],
                
                // 或下拉选择
                row![
                    text("或选择:").color(Color::WHITE),
                    pick_list(
                        self.card_dropdown.clone(),
                        self.selected_card.clone(),
                        Message::TimerCardSelected,
                    ),
                ],
                
                // 或创建新卡片
                button(text("创建新卡片..."))
                    .on_press(Message::TimerCreateNewCard),
                
                Space::new().height(16),
                
                // 记忆质量选择
                row![
                    text("记忆质量:").color(Color::WHITE),
                    MemoryQualitySelector::view(&self.memory_quality)
                        .map(Message::TimerMemoryQualityChanged),
                ],
                
                Space::new().height(24),
                
                // 按钮
                row![
                    button(text("取消"))
                        .on_press(Message::TimerLinkModeClose),
                    button(text("保存并预测"))
                        .on_press(Message::TimerLinkConfirm),
                ]
                .spacing(12),
            ]
            .spacing(8)
        )
        .padding(24)
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.2, 0.2, 0.2).into()),
            ..Default::default()
        })
        .into()
    }
}
```

---

## 8. 测试策略

### 8.1 单元测试

```rust
#[test]
fn test_timer_link_flow() {
    let mut app = App::default();
    
    // 开始计时
    app.update(Message::TimerStarted);
    assert!(matches!(app.timer_tab.state, TimerState::Running));
    
    // 设置当前卡片
    app.timer_manager.set_current_card(Some("knowledge/数学/微积分".to_string()));
    
    // 停止计时
    app.update(Message::TimerStopped(Ok(PathBuf::new())));
    assert!(app.timer_tab.link_mode);
    assert_eq!(app.timer_tab.card_path_input, "knowledge/数学/微积分");
}

#[test]
fn test_timer_save_record() {
    let data_fs = DataFs::init(temp_dir()).unwrap();
    let mut card = Card::new();
    
    // 保存计时记录
    let record = ReviewRecord {
        timestamp: Utc::now(),
        duration_ms: 300000, // 5分钟
        memory_quality: MemoryQuality::Good,
    };
    card.review_records.push(record);
    
    data_fs.save_card("knowledge/数学/微积分", &card).unwrap();
    
    // 验证保存成功
    let saved = data_fs.get_card("knowledge/数学/微积分").unwrap();
    assert_eq!(saved.review_records.len(), 1);
    assert_eq!(saved.review_records[0].duration_ms, 300000);
}
```

### 8.2 集成测试

- 完整计时流程：开始 → 停止 → 链接 → 保存 → 预测
- 快捷计时：从分类树启动 → 计时 → 自动填充路径
- 历史记录：多次计时 → 查看历史 → 验证排序

---

## 9. 风险与限制

### 9.1 技术风险

- **FSRS 预测精度**：首次复习的预测可能不够准确，需要用户反馈调整
- **计时器精度**：系统休眠时计时器可能暂停，需要处理
- **数据一致性**：计时记录与卡片复习记录需要保持同步

### 9.2 用户体验

- **中断处理**：计时中途退出应用，需要恢复计时状态
- **多卡片学习**：一次学习多个卡片，需要支持切换
- **后台计时**：应用最小化时计时器继续运行

---

## 10. 验收标准

- [ ] 计时器时钟显示占窗口高度1/3，宽度2/3
- [ ] 从卡片跳转时显示卡片路径（无"当前学习："前缀）
- [ ] 直接进入时不显示路径信息
- [ ] 计时停止后弹出链接模态框
- [ ] 模态框支持手动输入和下拉选择卡片路径
- [ ] 支持创建新卡片（参考分类树创建界面）
- [ ] 可以选择记忆质量（重学/困难/好/简单）
- [ ] 保存后自动触发 FSRS 重新预测
- [ ] 从分类树可以快捷启动计时
- [ ] 从复习看板可以快捷启动计时
- [ ] "计时记录"界面显示完整计时记录列表
- [ ] 计时记录支持编辑（时长、卡片、质量、时间）
- [ ] 计时记录支持硬删除
- [ ] 计时记录支持链接到卡片

---

## 11. 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-06-13 | 基于 Phase 1-4 实现创建 Phase 5 设计 |