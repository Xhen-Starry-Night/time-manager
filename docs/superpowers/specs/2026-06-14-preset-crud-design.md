# Preset CRUD 设计文档

## 目标

支持预设标签页中参数预设的创建、编辑、删除功能。编辑内容包括预设名称、描述、匹配规则。

## 数据层

### DataFs 新增方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `delete_preset` | `(&self, name: &str) -> Result<()>` | 删除 `presets/{name}.json` |
| `rename_preset` | `(&self, old: &str, new: &str) -> Result<()>` | 读取旧文件 → 修改 name → 写入新文件 → 删除旧文件 |

现有 `save_preset` / `get_preset` / `list_presets` 保持不变。

### 删除预设时的行为

删除预设只删除文件系统上的预设文件。所有引用了该预设的卡片保留 `preset_used` 的文本引用不变，不做自动回退。

### 重命名预设时的行为

`rename_preset` 内部仅执行文件重命名操作，不更新卡片的 `preset_used` 字段（卡片侧保持旧名称引用）。名称冲突校验在业务层完成。

## 交互流程

### 预设标签页布局

```
┌─────────────────────────────────────────┐
│  预设                          [+ 新建] │
│  ─────────────────────────────────────── │
│  default         未训练       [编辑][删除]│
│  英语高频词       2026-06-10  [编辑][删除]│
│  ...                                    │
└─────────────────────────────────────────┘
```

- 标题行文字 "预设" + 右侧 "新建预设" 按钮
- 分隔线
- 预设列表（可滚动）
- 每行：预设名称（左） | 训练时间标签（`trained_at` 为 `None` 显示 "未训练"，否则显示格式化时间如 "2026-06-14 15:30"） | [编辑] [删除] 图标按钮（右）
- 状态标签逻辑变更：原代码使用 `fsrs_parameters` 判断 "默认参数"/"已训练"，统一改为显示 `trained_at` 时间
- 无预设时显示空状态文字

### 创建

1. 点击 [+ 新建] → 打开新建 Modal
2. 表单字段：名称（TextInput）、描述（TextInput）、匹配规则（TextArea 多行，一行一条 glob）
3. 校验：名称非空 + 不与现有预设名称重复
4. [保存] → `DataFs::save_preset` → 刷新列表 → 关闭 Modal
5. [取消] → 关闭 Modal，不保存

### 编辑

1. 点击 [编辑] → 打开编辑 Modal，预填当前值
2. 修改字段（名称修改 = 重命名）
3. 校验：名称非空；若名称变更，不得与其它预设重复（允许与自身同名）
4. [保存] → 若名称变更 → `DataFs::rename_preset`；否则 `DataFs::save_preset` → 刷新列表 → 关闭 Modal
5. [取消] → 关闭 Modal，不保存

### 删除

1. 点击 [删除] → 弹出确认对话框："确定删除预设「xxx」？"
2. [确认删除] → `DataFs::delete_preset` → 刷新列表 → 关闭确认框
3. [取消] → 关闭确认框

## 组件结构

### PresetTabState 扩展

```rust
pub struct PresetTabState {
    pub selected_preset: Option<String>,
    pub presets: Vec<Preset>,

    // 表单状态
    pub show_form: bool,
    pub editing_name: Option<String>,   // Some(name) = 编辑模式, None = 新建
    pub form_name: String,
    pub form_description: String,
    pub form_match_rules: String,       // 多行文本，一行一条规则
    pub form_error: Option<String>,

    // 删除确认
    pub delete_target: Option<String>,
}
```

### 新增方法

- `validate(&self, presets: &[Preset]) -> Result<Preset, String>` — 校验表单并返回 Preset
- `load_from_preset(name: &str, presets: &[Preset])` — 加载预设到表单
- `reset_form()` — 清空表单

### 消息

| 消息 | 载荷 | 说明 |
|------|------|------|
| `PresetCreateRequested` | — | 打开新建表单 |
| `PresetEditRequested` | `String` | 打开编辑表单，参数为预设名称 |
| `PresetFormDismissed` | — | 关闭表单 |
| `PresetFormNameChanged` | `String` | 表单名称字段变更 |
| `PresetFormDescriptionChanged` | `String` | 表单描述字段变更 |
| `PresetFormMatchRulesChanged` | `String` | 表单匹配规则字段变更 |
| `PresetFormSaveRequested` | — | 提交保存 |
| `PresetDeleteRequested` | `String` | 请求删除确认 |
| `PresetDeleteConfirmed` | `String` | 确认删除 |
| `PresetDeleteDismissed` | — | 取消删除 |

### 已有消息（复用）

- `ButtonId::NewPreset` — 新建按钮点击（已声明未使用）
- `ButtonId::Delete` — 删除按钮点击（已声明未使用）

## 校验规则

1. 名称不能为空
2. 新建时：名称不能与 `presets` 列表中任何已有预设重复
3. 编辑时：若名称未变更，允许；若名称变更，不能与 `presets` 列表中其它预设重复（排除自身）
4. 匹配规则：自由文本，无格式校验

## 错误处理

- 表单校验失败 → `form_error` 设置错误文本，保留表单不关闭
- `DataFs` 操作失败（IO 错误） → 设置 `form_error`，显示错误消息
- 删除确认后 IO 错误 → 仅记录日志，不阻塞 UI

## 测试

- `test_preset_crud` — 创建、编辑（含重命名）、删除完整流程
- `test_delete_preset` — 删除预设文件验证
- `test_rename_preset` — 重命名后新旧文件验证
- `test_preset_validate` — 校验规则测试（空名称、重复名称）
