# Settings Tab Enhancement Design

## Goal

将目前只读的设置标签页改造为可编辑，支持 `data_dir` 和 `default_preset` 的配置持久化与运行时管理。

## Config 文件

### 文件位置

```rust
// 优先级:
// 1. 环境变量 TMD_CONFIG_DIR + "/config.toml"
// 2. 平台默认配置目录 + "/config.toml" (Linux: ~/.config/time-manager/config.toml)
```

配置文件位置由固定规则确定，不存储在配置文件自身中（避免循环依赖）。

### 格式 (TOML)

```toml
data_dir = "/home/user/.local/share/time-manager"
default_preset = "english"
```

两个字段均为 optional，支持部分配置。

## Config 结构体与 I/O

### 位置

新建 `src/data/config.rs`。

### 结构体

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub data_dir: Option<String>,
    pub default_preset: Option<String>,
}
```

### 方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `config_dir()` | `() -> PathBuf` | 返回配置目录路径（环境变量优先） |
| `config_path()` | `() -> PathBuf` | `config_dir() / "config.toml"` |
| `Config::load()` | `(path: &Path) -> Result<Config>` | 读取 TOML 文件，文件不存在返回 Default |
| `Config::save()` | `(&self, path: &Path) -> Result<()>` | 写入 TOML 文件，自动创建目录 |

### 依赖

需要 `toml` crate 用于序列化/反序列化。在 `Cargo.toml` 中添加 `toml = "0.8"`。

## SettingsTabState

### 位置

修改 `src/app/settings_tab.rs`。

```rust
pub struct SettingsTabState {
    pub config_dir: PathBuf,
    pub config_path: PathBuf,
    pub config: Config,
    pub form_data_dir: String,
    pub form_default_preset: String,
    pub message: Option<String>,
    pub message_is_error: bool,
}
```

### 方法

- `new(config_dir: PathBuf, config: Config)` — 初始化，依据 config 填充表单
- `dismiss_message()` — 清除提示消息

## Messages

在 `src/gui/messages.rs` 中新增：

| 消息 | 载荷 | 说明 |
|------|------|------|
| `SettingsFormDataDirChanged` | `String` | 数据目录输入变更 |
| `SettingsFormDefaultPresetChanged` | `String` | 默认预设选择变更 |
| `SettingsFormSaveRequested` | — | 保存配置 |
| `SettingsFormDismissMessage` | — | 清除提示消息 |

## App 启动集成

修改 `src/app/mod.rs` 中 `App::run()` 启动流程：

1. 按现有逻辑确定 `data_dir`（env/CLI/平台默认）
2. 计算 `config_path`（`config_dir() / "config.toml"`）
3. 尝试读取 `Config::load(&config_path)`
4. 若 config 中存在 `data_dir`，覆盖已确定的 `data_dir`
5. 确定 `default_preset`：config 中有值则用它，否则 `"default"`

```rust
// App 新增字段
pub default_preset: String,

// App::run() 中
let config_path = Config::config_path();
let config = Config::load(&config_path).ok().unwrap_or_default();
let data_dir = config.data_dir.clone()
    .map(PathBuf::from)
    .unwrap_or(data_dir);
let default_preset = config.default_preset.clone()
    .unwrap_or_else(|| "default".into());

let settings_tab = SettingsTabState::new(
    Config::config_dir(),
    config_path,
    config,
);

App {
    default_preset,
    settings_tab,
    ...
}
```

## Default Preset 传播

`App` 新增 `default_preset: String` 字段后，替换所有硬编码 `"default"` 为 `self.default_preset`。

### 涉及位置

| 文件 | 位置 | 替换方式 |
|------|------|----------|
| `src/app/mod.rs` | `RefreshPredictions` handler | `preset_used: "default".to_string()` → `preset_used: self.default_preset.clone()` |
| `src/app/mod.rs` | `TimerLinkConfirm` handler | 同上 |
| `src/app/mod.rs` | `TimerCreateNewCard` handler | 同上 |
| `src/app/category_tab.rs` | `NewNodeForm::new()` | 增加 `default_preset: String` 参数 |
| `src/app/category_tab.rs` | `EditCardForm::new()` | 增加 `default_preset: &str` 参数 |
| `src/app/timer_tab.rs` | `TimerTabState::default()` | 增加 `default_preset: String` 字段，构造时传入 |

所有替换后，当 config 中不指定 `default_preset` 时仍 fallback 到 `"default"`。

### 保存时即时生效

当 `SettingsFormSaveRequested` 处理时：
1. `Config::save()` 写入文件
2. `self.default_preset = config.default_preset.clone().unwrap_or_else(|| "default".into())`
3. 设置提示消息 `"配置已保存"`

## Settings 标签页视图

```
┌──────────────────────────────────────┐
│  设置                                │
│  ──────────────────────────────────── │
│  配置目录: /path/to/config            │
│                                      │
│  数据目录: [____________________]    │
│  默认预设: [▼ pick_list 选择预设]     │
│                                      │
│  [保存]                               │
│                                      │
│  (如有消息，显示在此处)               │
└──────────────────────────────────────┘
```

- 配置目录只读显示
- 数据目录：`text_input`，可编辑
- 默认预设：`pick_list`，从 `preset_tab.presets` 中选择
- 保存按钮：执行 `SettingsFormSaveRequested`
- 消息区域：绿色提示 / 红色错误

## 错误处理

| 场景 | 行为 |
|------|------|
| config.toml 不存在 | `Config::load()` 返回 Default（空 Config） |
| config.toml 格式错误 | `Config::load()` 返回 Default，设置 `message` 警告 |
| 保存时 IO 错误 | `message_is_error = true`，显示错误信息 |
| data_dir 为空 | 保存时校验，不保存并提示 |
| default_preset 为空 | 保存时 fallback 到 `"default"` |

## 测试

- `test_config_load_save` — 写入后读取验证 roundtrip
- `test_config_partial` — 部分字段的 TOML 正确解析
- `test_config_not_found` — 文件不存在时返回 Default
- `test_config_path` — 环境变量 / 默认路径正确
