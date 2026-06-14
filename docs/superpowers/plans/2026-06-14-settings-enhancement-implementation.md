# Settings Tab Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Settings tab editable with configurable `data_dir` and `default_preset`, persisted via TOML config file.

**Architecture:** `Config` struct in `src/data/config.rs`, TOML file at `~/.config/time-manager/config.toml`, `SettingsTabState` with form fields, `default_preset` field on `App` propagated to all preset pickers.

**Tech Stack:** toml 0.8, serde, iced 0.14

---

### Task 1: Cargo.toml — add `toml` dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add toml dependency**

```toml
toml = "0.8"
```

Add it after `serde` in the dependencies section.

- [ ] **Step 2: Verify**

Run: `cargo check`
Expected: compiles (may have warnings about unused, that's fine)

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml && git commit -m "chore: add toml dependency"
```

---

### Task 2: Config struct + I/O

**Files:**
- Create: `src/data/config.rs`

- [ ] **Step 1: Create config module**

```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::data::{DataError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub data_dir: Option<String>,
    pub default_preset: Option<String>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("TMD_CONFIG_DIR") {
            return PathBuf::from(dir);
        }
        directories::ProjectDirs::from("com", "time-manager", "time-manager")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                PathBuf::from(home).join(".config/time-manager")
            })
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(|e| DataError::Io(e.to_string()))?;
        toml::from_str(&content).map_err(|e| DataError::ConfigParse(e.to_string()))
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DataError::Io(e.to_string()))?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| DataError::ConfigSerialize(e.to_string()))?;
        std::fs::write(path, content).map_err(|e| DataError::Io(e.to_string()))?;
        Ok(())
    }
}
```

Note: `DataError` needs two new variants: `ConfigParse(String)` and `ConfigSerialize(String)`. Add them to `src/data/mod.rs`:

```rust
pub enum DataError {
    // ... existing variants ...
    ConfigParse(String),
    ConfigSerialize(String),
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/data/config.rs src/data/mod.rs && git commit -m "feat: add Config struct and TOML load/save"
```

---

### Task 3: Register config module

**Files:**
- Modify: `src/data/mod.rs`

- [ ] **Step 1: Add config module declaration**

```rust
pub mod config;
```

Add alongside other `pub mod` declarations.

- [ ] **Step 2: Verify**

Run: `cargo check`
Expected: ok

- [ ] **Step 3: Commit**

```bash
git add src/data/mod.rs && git commit -m "feat: register config module"
```

---

### Task 4: Messages — settings form variants

**Files:**
- Modify: `src/gui/messages.rs`

- [ ] **Step 1: Add settings message variants**

Add before `Error(String)`:

```rust
    SettingsFormDataDirChanged(String),
    SettingsFormDefaultPresetChanged(String),
    SettingsFormSaveRequested,
    SettingsFormDismissMessage,
```

- [ ] **Step 2: Commit**

```bash
git add src/gui/messages.rs && git commit -m "feat: add settings form message variants"
```

---

### Task 5: SettingsTabState update

**Files:**
- Modify: `src/app/settings_tab.rs`

- [ ] **Step 1: Rewrite SettingsTabState**

Replace the entire file:

```rust
use std::path::PathBuf;
use crate::data::config::Config;

pub struct SettingsTabState {
    pub config_dir: PathBuf,
    pub config_path: PathBuf,
    pub config: Config,
    pub form_data_dir: String,
    pub form_default_preset: String,
    pub message: Option<String>,
    pub message_is_error: bool,
}

impl SettingsTabState {
    pub fn new(config_dir: PathBuf, config_path: PathBuf, config: Config) -> Self {
        let form_data_dir = config.data_dir.clone().unwrap_or_default();
        let form_default_preset = config.default_preset.clone().unwrap_or_else(|| "default".into());
        Self {
            config_dir,
            config_path,
            config,
            form_data_dir,
            form_default_preset,
            message: None,
            message_is_error: false,
        }
    }

    pub fn dismiss_message(&mut self) {
        self.message = None;
    }
}
```

- [ ] **Step 2: Fix App constructor usage**

In `src/app/mod.rs`, find where `SettingsTabState::new(data_dir.clone())` is called, update it:

```rust
settings_tab: SettingsTabState::new(
    Config::config_dir(),
    Config::config_path(),
    config.clone(),
),
```

Also add `use crate::data::config::Config;` import if needed.

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: ok

- [ ] **Step 4: Commit**

```bash
git add src/app/settings_tab.rs src/app/mod.rs && git commit -m "feat: update SettingsTabState with form fields"
```

---

### Task 6: App startup integration

**Files:**
- Modify: `src/app/mod.rs` (App::run)

- [ ] **Step 1: Add default_preset to App struct**

```rust
pub struct App {
    pub default_preset: String,
    // ... existing fields ...
}
```

- [ ] **Step 2: Update App::run to read config**

In `App::run`, after the existing data_dir resolution:

```rust
let config = Config::load(&Config::config_path()).unwrap_or_default();
let data_dir = config.data_dir.clone()
    .map(PathBuf::from)
    .unwrap_or(data_dir);
let default_preset = config.default_preset.clone()
    .unwrap_or_else(|| "default".into());
```

Pass `config` and `default_preset` when constructing App state.

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: ok

- [ ] **Step 4: Commit**

```bash
git add src/app/mod.rs && git commit -m "feat: integrate config loading at App startup"
```

---

### Task 7: Default preset propagation

**Files:**
- Modify: `src/app/mod.rs`, `src/app/category_tab.rs`, `src/app/timer_tab.rs`

- [ ] **Step 1: Update category_tab.rs — NewNodeForm + EditCardForm**

In `src/app/category_tab.rs`:
- `NewNodeForm` struct: add `pub preset: String` (already exists)
- `NewNodeForm::new(path, node_type)` → add `default_preset: &str` parameter, use it for preset field
- `EditCardForm::new(card, preset)` → already takes `preset` parameter

```rust
pub fn new(path: String, node_type: NodeType, default_preset: &str) -> Self {
    Self {
        path,
        node_type,
        name: String::new(),
        preset: default_preset.to_string(),
    }
}
```

- [ ] **Step 2: Update timer_tab.rs — TimerTabState**

In `src/app/timer_tab.rs`:
- Change `new_card_preset` initialization to take a parameter

```rust
impl TimerTabState {
    pub fn new(default_preset: &str) -> Self {
        Self {
            running: false,
            elapsed_seconds: 0,
            show_link_form: false,
            card_path_input: String::new(),
            selected_card: None,
            memory_quality: MemoryQuality::Good,
            new_card_name: String::new(),
            new_card_preset: default_preset.to_string(),
            new_card_node_type: NodeType::Card,
            history_records: Vec::new(),
            show_history: false,
            show_quick_timer: false,
        }
    }
}
```

- Update `TimerTabState::default()` calls to `TimerTabState::new()`.

- [ ] **Step 3: Update app/mod.rs — all hardcoded "default" preset usages**

Find and replace these patterns:

1. `NewNodeForm::new(path.clone(), node_type)` → `NewNodeForm::new(path.clone(), node_type, &self.default_preset)`
2. `EditCardForm::new(...)` → already takes preset, but find `"default".to_string()` in the caller and replace with `self.default_preset.clone()`
3. `TimerTabState::default()` → `TimerTabState::new(&self.default_preset)`
4. `preset_used: "default".to_string()` → `preset_used: self.default_preset.clone()` (multiple locations in handlers)

Specific locations in `src/app/mod.rs`:
- `NewNodeConfirm` handler: `let card = Card::new_with_preset(form.preset.clone());` — the form.preset comes from NewNodeForm which already uses default_preset
- `TimerLinkConfirm` handler: `preset_used: "default".to_string()`
- `RefreshPredictions` handler: `preset_used: "default".to_string()`
- `TimerCreateNewCard` handler (around line 870): `preset_used: "default".to_string()`
- `TimerNewCardPresetChanged` — this is a field change, already handled
- `EditCardPredict` / `EditCardConfirm` — check if "default" appears

- [ ] **Step 4: Verify compilation**

Run: `cargo check`
Expected: ok

- [ ] **Step 5: Commit**

```bash
git add src/app/mod.rs src/app/category_tab.rs src/app/timer_tab.rs && git commit -m "feat: propagate default_preset to all preset pickers"
```

---

### Task 8: App update handler — settings messages

**Files:**
- Modify: `src/app/mod.rs`

- [ ] **Step 1: Add settings message handlers**

Add after the last `Preset*` handler:

```rust
            Message::SettingsFormDataDirChanged(val) => {
                self.settings_tab.form_data_dir = val;
                Task::none()
            }

            Message::SettingsFormDefaultPresetChanged(val) => {
                self.settings_tab.form_default_preset = val;
                Task::none()
            }

            Message::SettingsFormSaveRequested => {
                let data_dir_val = self.settings_tab.form_data_dir.trim().to_string();
                let default_preset_val = self.settings_tab.form_default_preset.trim().to_string();

                let new_config = crate::data::config::Config {
                    data_dir: if data_dir_val.is_empty() { None } else { Some(data_dir_val.clone()) },
                    default_preset: if default_preset_val.is_empty() { None } else { Some(default_preset_val.clone()) },
                };

                let path = self.settings_tab.config_path.clone();
                let data_dir_changed = data_dir_val != self.settings_tab.config.data_dir.as_deref().unwrap_or("");

                match new_config.save(&path) {
                    Ok(()) => {
                        self.settings_tab.config = new_config;
                        // apply default_preset immediately
                        if !default_preset_val.is_empty() {
                            self.default_preset = default_preset_val;
                        }
                        let msg = if data_dir_changed {
                            "配置已保存，数据目录修改需重启生效".to_string()
                        } else {
                            "配置已保存".to_string()
                        };
                        self.settings_tab.message = Some(msg);
                        self.settings_tab.message_is_error = false;
                    }
                    Err(e) => {
                        self.settings_tab.message = Some(format!("保存失败: {}", e));
                        self.settings_tab.message_is_error = true;
                    }
                }
                Task::none()
            }

            Message::SettingsFormDismissMessage => {
                self.settings_tab.dismiss_message();
                Task::none()
            }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: ok

- [ ] **Step 3: Commit**

```bash
git add src/app/mod.rs && git commit -m "feat: add settings form update handlers"
```

---

### Task 9: App view — Settings tab UI

**Files:**
- Modify: `src/app/mod.rs` (view section)

- [ ] **Step 1: Update Settings tab view**

Replace the `TabId::Settings => { ... }` block with:

```rust
            TabId::Settings => {
                let mut col = column![
                    text("设置").size(20).color(iced::Color::WHITE),
                    rule::horizontal(1.0),
                    row![
                        text("配置目录:").color(iced::Color::WHITE),
                        text(self.settings_tab.config_dir.to_string_lossy().to_string())
                            .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                    ].spacing(8).padding(8),
                    row![
                        text("数据目录:").color(iced::Color::WHITE),
                        iced::widget::text_input("数据目录路径", &self.settings_tab.form_data_dir)
                            .on_input(Message::SettingsFormDataDirChanged),
                    ].spacing(8).padding(8),
                    row![
                        text("默认预设:").color(iced::Color::WHITE),
                        {
                            let presets: Vec<String> = self.preset_tab.presets.iter()
                                .map(|p| p.name.clone())
                                .collect();
                            iced::widget::pick_list(
                                presets,
                                Some(self.settings_tab.form_default_preset.clone()),
                                Message::SettingsFormDefaultPresetChanged,
                            )
                        },
                    ].spacing(8).padding(8),
                    button(text("保存")).on_press(Message::SettingsFormSaveRequested).padding(8),
                ]
                .spacing(4);

                if let Some(ref msg) = self.settings_tab.message {
                    let color = if self.settings_tab.message_is_error {
                        iced::Color::from_rgb(1.0, 0.3, 0.3)
                    } else {
                        iced::Color::from_rgb(0.3, 1.0, 0.3)
                    };
                    let msg_row = row![
                        text(msg.clone()).color(color),
                        button(text("×").size(12))
                            .style(iced::widget::button::text)
                            .on_press(Message::SettingsFormDismissMessage),
                    ].spacing(8).padding(8);
                    col = col.push(msg_row);
                }

                container(col)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                        ..Default::default()
                    })
                    .into()
            }
```

Note: `pick_list` might have type issues in the closure context. If there's a type inference error, wrap the result in `.into()` or wrap the pick_list in a function call.

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: may need minor fixes for iced 0.14 pick_list API

- [ ] **Step 3: Commit**

```bash
git add src/app/mod.rs && git commit -m "feat: update settings tab view with editable form"
```

---

### Task 10: Tests — config tests

**Files:**
- Modify: `tests/data_tests.rs` or add new test module

- [ ] **Step 1: Add config tests**

```rust
#[test]
fn test_config_load_save() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let config = time_manager::data::config::Config {
        data_dir: Some("/tmp/data".to_string()),
        default_preset: Some("english".to_string()),
    };
    config.save(&config_path).unwrap();

    let loaded = time_manager::data::config::Config::load(&config_path).unwrap();
    assert_eq!(loaded.data_dir, Some("/tmp/data".to_string()));
    assert_eq!(loaded.default_preset, Some("english".to_string()));
}

#[test]
fn test_config_partial() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    std::fs::write(&config_path, "default_preset = \"math\"").unwrap();
    let loaded = time_manager::data::config::Config::load(&config_path).unwrap();
    assert!(loaded.data_dir.is_none());
    assert_eq!(loaded.default_preset, Some("math".to_string()));
}

#[test]
fn test_config_not_found() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("nonexistent.toml");

    let loaded = time_manager::data::config::Config::load(&config_path).unwrap();
    assert!(loaded.data_dir.is_none());
    assert!(loaded.default_preset.is_none());
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -- test_config 2>&1`
Expected: 3 passed

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add tests/data_tests.rs && git commit -m "test: add config load/save/partial tests"
```

---

### Task 11: Full verification

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 2: Verify no new warnings**

Run: `cargo clippy --all-targets 2>&1 | grep "^warning" | grep -v "pre-existing" | head -5`
Expected: no new warnings from our changes
