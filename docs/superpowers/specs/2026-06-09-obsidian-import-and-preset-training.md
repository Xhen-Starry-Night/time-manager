# Obsidian 导入与参数预设训练设计

**日期**: 2026-06-09  
**状态**: 设计阶段

---

## 1. 概述

### 1.1 功能目标

**Obsidian 导入**：
- 从 Obsidian vault 导入目录结构到学习分类树
- 使用 `.timeignore` 文件过滤不需要的目录
- 只导入目录结构，不导入 Markdown 文件内容

**参数预设训练**：
- 不同类型学习内容使用不同的 FSRS 参数
- 根据历史复习数据优化参数
- 提高预测准确性

### 1.2 当前实现差距

| 功能 | 设计要求 | 当前状态 |
|------|----------|----------|
| Card 结构 | 包含 `prediction` 和 `preset_used` | ❌ 缺失 |
| Preset 结构 | 包含 `match_rules`、`fsrs_parameters` | ⚠️ 部分 |
| Obsidian 导入 | 目录结构导入 | ❌ 未实现 |
| 预设训练 | FSRS 参数优化 | ❌ 未实现 |

---

## 2. 数据结构调整

### 2.1 学习卡片结构调整

**当前实现**：
```json
{
  "path": "main/语言/英语",
  "source": null,
  "review_records": [...]
}
```

**目标结构**：
```json
{
  "path": "main/语言/英语",
  "source": null,
  "review_records": [...],
  "prediction": {
    "algorithm": "fsrs",
    "next_review": "2026-06-10T10:00:00Z",
    "fsrs_state_bytes": [1, 2, 3, ...],
    "preset_used": "vocabulary"
  }
}
```

**字段说明**：
- `prediction.algorithm`: 预测算法名称
- `prediction.next_review`: 下次复习时间
- `prediction.fsrs_state_bytes`: FSRS 内存状态（8 字节）
- `prediction.preset_used`: 使用的参数预设名称

### 2.2 参数预设结构调整

**当前实现**：
```json
{
  "name": "default",
  "description": "默认预设",
  "match_rules": []
}
```

**目标结构**：
```json
{
  "name": "vocabulary",
  "description": "单词背诵预设",
  "match_rules": [
    "main/语言/**/单词/**",
    "main/语言/**/词汇/**"
  ],
  "fsrs_parameters": [1.0, 2.0, 3.0, ...],
  "trained_at": "2026-06-09T10:00:00Z"
}
```

**字段说明**：
- `match_rules`: 路径匹配模式列表（glob 模式）
- `fsrs_parameters`: 17 个 FSRS 权重参数（训练后生成）
- `trained_at`: 训练时间

### 2.3 全局配置调整

**当前实现**：
```json
{
  "default_preset": "default",
  "default_algorithm": "fsrs"
}
```

保持不变。

---

## 3. Obsidian 导入设计

### 3.1 过滤规则文件 `.timeignore`

**位置**: `data/.timeignore`

**格式**: 类似 `.gitignore`

```
# 系统目录
.obsidian/
.trash/
.git/
.templates/

# 媒体文件
附件/
attachments/
images/

# 临时文件
*.tmp
*.bak
*.swp

# 其他
_drafts/
_archive/
```

### 3.2 导入流程

```
1. 用户调用: tree-create <tree-name> --import-path <vault-path>
2. 读取 data/.timeignore（如不存在，使用默认规则）
3. 遍历 vault 目录
4. 过滤匹配的路径
5. 在 data/categories/<tree-name>/ 下创建相同目录结构
6. 不创建文件，只创建目录
```

### 3.3 CLI 命令

```bash
# 从 Obsidian vault 导入
tmd tree-create english --import-path ~/obsidian/vault --description "英语学习"

# 创建空分类树
tmd tree-create math --description "数学学习"
```

### 3.4 实现细节

**依赖**：
- `walkdir`: 遍历目录
- `globset`: 路径匹配（已在 Cargo.toml）

**关键函数**：
```rust
pub fn import_from_obsidian(
    vault_path: &Path,
    tree_name: &str,
    data_dir: &Path,
    ignore_rules: &[String],
) -> Result<Vec<String>>
```

**返回**: 创建的目录路径列表

---

## 4. 参数预设训练设计

### 4.1 训练流程

```
1. 用户调用: preset-train <preset-name>
2. 读取预设文件，解析 match_rules
3. 遍历所有卡片，匹配路径
4. 收集匹配卡片的 review_records
5. 转换为 FSRS 训练数据格式
6. 调用 FSRS 优化器
7. 获取优化后的参数
8. 保存到预设文件
```

### 4.2 FSRS 训练数据格式

FSRS 需要 `FSRSItem` 结构：

```rust
pub struct FSRSItem {
    pub reviews: Vec<FSRSReview>,
}

pub struct FSRSReview {
    pub rating: Rating,  // Again/Hard/Good/Easy
    pub delta_t: u32,    // 距上次复习的天数
}
```

**转换逻辑**：
```rust
fn convert_review_records(records: &[ReviewRecord]) -> Vec<FSRSItem> {
    // 每个卡片的所有复习记录转换为一个 FSRSItem
}
```

### 4.3 FSRS 训练 API

FSRS 库提供的训练函数：

```rust
// 准备训练数据
pub fn prepare_training_data(items: Vec<FSRSItem>) -> (Vec<FSRSItem>, Vec<FSRSItem>)

// 计算平均回忆率
pub fn calculate_average_recall(items: &[FSRSItem]) -> f32
```

**注意**: FSRS 5.2.0 的完整训练 API 需要进一步研究。可能需要：
1. 使用 burn 框架（FSRS 依赖）
2. 或直接使用默认参数 + 微调

### 4.4 CLI 命令

```bash
# 训练预设
tmd preset-train vocabulary

# 查看训练状态
tmd preset-list vocabulary
```

### 4.5 最小训练数据要求

建议至少需要：
- **卡片数量**: ≥ 10 张
- **总复习次数**: ≥ 30 次

低于此要求时，输出警告并使用默认参数。

---

## 5. 预测流程调整

### 5.1 当前流程

```rust
// 使用全局默认参数
let predictor = FsrsPredictor::new()?;
let (interval, state) = predictor.predict_next_review(...)?;
```

### 5.2 调整后流程

```rust
// 1. 读取卡片的 preset_used
let preset_name = card.prediction.as_ref()
    .map(|p| &p.preset_used)
    .unwrap_or(&config.default_preset);

// 2. 加载预设参数
let preset = fs.get_preset(preset_name)?;

// 3. 使用预设参数创建预测器
let predictor = FsrsPredictor::with_parameters(&preset.fsrs_parameters)?;

// 4. 预测
let (interval, state) = predictor.predict_next_review(...)?;
```

### 5.3 FsrsPredictor 扩展

**当前实现**：
```rust
pub fn new() -> Result<Self, String>  // 使用默认参数
```

**扩展后**：
```rust
pub fn new() -> Result<Self, String>  // 使用默认参数
pub fn with_parameters(params: &[f32]) -> Result<Self, String>  // 使用自定义参数
pub fn get_default_parameters() -> &'static [f32]  // 获取默认参数
```

---

## 6. 创建卡片时指定预设

### 6.1 CLI 命令

```bash
# 使用默认预设
tmd card-create main/语言/英语/单词-Unit1

# 指定预设
tmd card-create main/语言/英语/单词-Unit1 --preset vocabulary
```

### 6.2 逻辑

```
1. 检查 --preset 参数
2. 如果指定，使用指定的预设
3. 如果未指定，使用 config.default_preset
4. 创建卡片时，设置 prediction.preset_used
```

---

## 7. 文件变更清单

### 7.1 新建文件

```
src/obsidian/mod.rs         # Obsidian 导入模块
src/obsidian/import.rs      # 导入逻辑
src/training/mod.rs         # 训练模块
```

### 7.2 修改文件

```
src/data/models.rs          # Card 和 Preset 结构调整
src/data/fs.rs              # 新增方法
src/fsrs/mod.rs             # 支持自定义参数
src/cli/mod.rs              # 新增命令参数
src/bin/tmd.rs              # 命令实现
```

### 7.3 新增依赖

无需新增（globset 和 walkdir 已存在）

---

## 8. 测试策略

### 8.1 单元测试

**Obsidian 导入**：
- 测试 `.timeignore` 规则解析
- 测试路径过滤逻辑
- 测试目录结构创建

**参数预设训练**：
- 测试路径匹配（globset）
- 测试复习记录转换
- 测试参数保存和加载

### 8.2 集成测试

- 完整导入流程（创建临时 vault，导入，验证结构）
- 完整训练流程（创建预设，训练，验证参数）
- 预测流程（使用训练后的参数预测）

---

## 9. 实施优先级

### Phase 1: 数据结构调整（必需）
1. Card 结构添加 `prediction` 字段
2. Preset 结构添加 `fsrs_parameters` 字段
3. 更新现有代码适配新结构

### Phase 2: Obsidian 导入（核心功能）
1. 实现 `.timeignore` 解析
2. 实现目录遍历和过滤
3. 实现目录结构创建
4. 更新 `tree-create` 命令

### Phase 3: 参数预设训练（高级功能）
1. 实现路径匹配逻辑
2. 实现 FSRS 训练数据转换
3. 调用 FSRS 优化 API
4. 更新 `preset-train` 命令

### Phase 4: 预测流程调整（必需）
1. 扩展 FsrsPredictor 支持自定义参数
2. 更新预测逻辑使用预设参数
3. 更新 `card-create` 命令

---

## 10. 风险与限制

### 10.1 FSRS 训练 API

**风险**: FSRS 5.2.0 的训练 API 可能不够完善

**缓解**: 
- 先使用默认参数
- 后续版本优化训练逻辑
- 或考虑手动调参

### 10.2 最小数据要求

**风险**: 新用户没有足够历史数据训练

**缓解**:
- 提供内置默认预设
- 训练前检查数据量
- 数据不足时给出警告

### 10.3 兼容性

**风险**: 旧版本卡片文件缺少 `prediction` 字段

**缓解**:
- 向后兼容，缺少时使用默认预设
- 首次访问时自动添加字段

---

## 11. 后续改进

1. **GUI 支持**: 在界面中显示和管理预设
2. **预设分享**: 导出/导入预设配置
3. **自动匹配**: 根据卡片路径自动推荐预设
4. **训练进度**: 显示训练进度和预估时间