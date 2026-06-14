# Preset CRUD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add create, edit (with rename), and delete functionality for presets in the Preset tab.

**Architecture:** Self-contained modal form managed by `PresetTabState`; new dedicated `Message` variants; DataFs gets `delete_preset` and `rename_preset`.

**Tech Stack:** iced 0.14, chrono, serde_json

---

### Task 1: DataFs — delete_preset + rename_preset

**Files:**
- Modify: `src/data/fs.rs` (after line 249)

- [ ] **Step 1: Add `delete_preset` method**

Insert after line 249 (`list_presets` closing brace):

```rust
    pub fn delete_preset(&self, name: &str) -> Result<()> {
        let file_path = self.data_dir.join("presets").join(format!("{name}.json"));
        if file_path.exists() {
            std::fs::remove_file(&file_path).map_err(|e| DataError::Io(e.to_string()))?;
        }
        Ok(())
    }

    pub fn rename_preset(&self, old_name: &str, new_name: &str) -> Result<()> {
        let old_path = self.data_dir.join("presets").join(format!("{old_name}.json"));
        let new_path = self.data_dir.join("presets").join(format!("{new_name}.json"));
        let mut preset: Preset = {
            let json = std::fs::read_to_string(&old_path)
                .map_err(|e| DataError::Io(e.to_string()))?;
            serde_json::from_str(&json).map_err(|e| DataError::Json(e.to_string()))?
        };
        preset.name = new_name.to_string();
        let json = serde_json::to_string_pretty(&preset).map_err(|e| DataError::Json(e.to_string()))?;
        std::fs::write(&new_path, json).map_err(|e| DataError::Io(e.to_string()))?;
        std::fs::remove_file(&old_path).map_err(|e| DataError::Io(e.to_string()))?;
        Ok(())
    }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add src/data/fs.rs
git commit -m "feat: add delete_preset and rename_preset to DataFs"
```

---

### Task 2: Messages — new preset form variants

**Files:**
- Modify: `src/gui/messages.rs`

- [ ] **Step 1: Add preset form message variants**

Add before line 119 (`Error(String)`):

```rust
    PresetCreateOpen,
    PresetEditOpen(String),
    PresetFormDismissed,
    PresetFormNameChanged(String),
    PresetFormDescriptionChanged(String),
    PresetFormMatchRulesChanged(String),
    PresetFormSaveRequested,
    PresetDeleteRequested(String),
    PresetDeleteConfirmed(String),
    PresetDeleteDismissed,
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: compile errors from unhandled messages in `app/mod.rs` — that's expected, handled in Task 4

- [ ] **Step 3: Commit**

```bash
git add src/gui/messages.rs
git commit -m "feat: add preset form message variants"
```

---

### Task 3: PresetTabState — form state + methods

**Files:**
- Modify: `src/app/preset_tab.rs`

- [ ] **Step 1: Rewrite PresetTabState**

Replace the entire file:

```rust
use crate::data::models::Preset;

#[derive(Default)]
pub struct PresetTabState {
    pub selected_preset: Option<String>,
    pub presets: Vec<Preset>,

    // form state
    pub show_form: bool,
    pub editing_name: Option<String>,
    pub form_name: String,
    pub form_description: String,
    pub form_match_rules: String,
    pub form_error: Option<String>,

    // delete confirm
    pub delete_target: Option<String>,
}

impl PresetTabState {
    pub fn reset_form(&mut self) {
        self.form_name.clear();
        self.form_description.clear();
        self.form_match_rules.clear();
        self.form_error = None;
        self.editing_name = None;
    }

    pub fn load_from_preset(&mut self, name: &str) {
        if let Some(preset) = self.presets.iter().find(|p| p.name == name) {
            self.form_name = preset.name.clone();
            self.form_description = preset.description.clone().unwrap_or_default();
            self.form_match_rules = preset.match_rules.join("\n");
            self.editing_name = Some(preset.name.clone());
            self.form_error = None;
        }
    }

    pub fn validate(&self) -> Result<Preset, String> {
        let name = self.form_name.trim();
        if name.is_empty() {
            return Err("名称不能为空".into());
        }
        // check duplicate name
        let duplicate = self.presets.iter().any(|p| {
            if let Some(ref editing) = self.editing_name {
                // editing: allow same name as self, reject if matches other preset
                p.name == name && p.name != *editing
            } else {
                // creating: reject any duplicate
                p.name == name
            }
        });
        if duplicate {
            return Err("名称已存在".into());
        }

        let description = if self.form_description.trim().is_empty() {
            None
        } else {
            Some(self.form_description.trim().to_string())
        };

        let match_rules: Vec<String> = self
            .form_match_rules
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(Preset {
            name: name.to_string(),
            description,
            match_rules,
            fsrs_parameters: None,
            trained_at: None,
        })
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: no errors (the unused fields don't cause issues)

- [ ] **Step 3: Commit**

```bash
git add src/app/preset_tab.rs
git commit -m "feat: extend PresetTabState with form and delete state"
```

---

### Task 4: App update — handle preset messages

**Files:**
- Modify: `src/app/mod.rs`

- [ ] **Step 1: Add preset message handling to update handler**

Find the `Message::PresetsLoaded` handler (line 213 area) and add handlers for new messages after it. Insert before the `Message::CardSelected` match arm:

```rust
            Message::PresetCreateOpen => {
                self.preset_tab.reset_form();
                self.preset_tab.show_form = true;
                Task::none()
            }

            Message::PresetEditOpen(name) => {
                self.preset_tab.load_from_preset(&name);
                self.preset_tab.show_form = true;
                Task::none()
            }

            Message::PresetFormDismissed => {
                self.preset_tab.show_form = false;
                self.preset_tab.reset_form();
                Task::none()
            }

            Message::PresetFormNameChanged(val) => {
                self.preset_tab.form_name = val;
                Task::none()
            }

            Message::PresetFormDescriptionChanged(val) => {
                self.preset_tab.form_description = val;
                Task::none()
            }

            Message::PresetFormMatchRulesChanged(val) => {
                self.preset_tab.form_match_rules = val;
                Task::none()
            }

            Message::PresetFormSaveRequested => {
                match self.preset_tab.validate() {
                    Ok(preset) => {
                        let data_fs = self.data_fs.clone();
                        let old_name = self.preset_tab.editing_name.clone();
                        let tasks = Task::batch(vec![
                            Task::perform(
                                async move {
                                    if let Some(ref old) = old_name {
                                        if old != &preset.name {
                                            data_fs.rename_preset(old, &preset.name).map_err(|e| e.to_string())?;
                                        } else {
                                            data_fs.save_preset(&preset).map_err(|e| e.to_string())?;
                                        }
                                    } else {
                                        data_fs.save_preset(&preset).map_err(|e| e.to_string())?;
                                    }
                                    let presets = data_fs.list_presets().map_err(|e| e.to_string())?;
                                    Ok(presets)
                                },
                                |result: Result<Vec<Preset>, String>| {
                                    match result {
                                        Ok(presets) => {
                                            // Store presets temporarily, use PresetsLoaded-like path
                                            // We'll send a custom message but for simplicity:
                                            // mark presets and hide form
                                            Message::PresetsLoaded(Ok(presets))
                                        }
                                        Err(e) => Message::Error(e),
                                    }
                                },
                            ),
                        ]);
                        // We need to also close the form after save
                        // The state will be cleared after PresetsLoaded sets the new presets
                        // But we also need to reset form — do it in the view or we can sequence
                        // Actually, PresetsLoaded will refresh presets but doesn't close form.
                        // Let's use a two-step: close form in the same frame.
                        self.preset_tab.show_form = false;
                        self.preset_tab.reset_form();
                        tasks
                    }
                    Err(e) => {
                        self.preset_tab.form_error = Some(e);
                        Task::none()
                    }
                }
            }

            Message::PresetDeleteRequested(name) => {
                self.preset_tab.delete_target = Some(name);
                Task::none()
            }

            Message::PresetDeleteConfirmed(name) => {
                self.preset_tab.delete_target = None;
                let data_fs = self.data_fs.clone();
                Task::perform(
                    async move {
                        data_fs.delete_preset(&name).map_err(|e| e.to_string())?;
                        data_fs.list_presets().map_err(|e| e.to_string())
                    },
                    |result: Result<Vec<Preset>, String>| {
                        match result {
                            Ok(presets) => Message::PresetsLoaded(Ok(presets)),
                            Err(e) => Message::Error(e),
                        }
                    },
                )
            }

            Message::PresetDeleteDismissed => {
                self.preset_tab.delete_target = None;
                Task::none()
            }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add src/app/mod.rs
git commit -m "feat: add preset CRUD update handlers"
```

---

### Task 5: App view — preset list with buttons + modals

**Files:**
- Modify: `src/app/mod.rs` (view section)

- [ ] **Step 1: Update preset tab view**

Replace the `TabId::Preset => { ... }` block (lines 1957-1998) with:

```rust
            TabId::Preset => {
                let preset_list: Vec<_> = self.preset_tab.presets.iter()
                    .map(|preset| {
                        let name = preset.name.clone();
                        let trained = if let Some(ts) = preset.trained_at {
                            ts.format("%Y-%m-%d %H:%M").to_string()
                        } else {
                            "未训练".to_string()
                        };
                        row![
                            text(preset.name.clone()).color(iced::Color::WHITE).width(Length::Fill),
                            text(trained).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                            button(row![
                                text("编辑").size(12),
                            ].spacing(4))
                            .style(iced::widget::button::text)
                            .on_press(Message::PresetEditOpen(name.clone())),
                            button(row![
                                text("删除").size(12),
                            ].spacing(4))
                            .style(iced::widget::button::text)
                            .on_press(Message::PresetDeleteRequested(name.clone())),
                        ]
                        .spacing(8)
                        .padding(8)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect();

                let list = if self.preset_tab.presets.is_empty() {
                    container(text("暂无预设").color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                } else {
                    container(scrollable(column(preset_list).spacing(4)))
                        .width(Length::Fill)
                        .height(Length::Fill)
                };

                let mut col = column![
                    row![
                        text("预设").size(20).color(iced::Color::WHITE),
                        iced::widget::horizontal_space(),
                        button(text("+ 新建"))
                            .on_press(Message::PresetCreateOpen),
                    ].padding(8),
                    rule::horizontal(1.0),
                    list,
                ];

                // form modal overlay
                if self.preset_tab.show_form {
                    let title = if self.preset_tab.editing_name.is_some() { "编辑预设" } else { "新建预设" };
                    let name_input = text_input("预设名称", &self.preset_tab.form_name)
                        .on_input(Message::PresetFormNameChanged);
                    let desc_input = text_input("描述（可选）", &self.preset_tab.form_description)
                        .on_input(Message::PresetFormDescriptionChanged);
                    let rules_input = text_input("匹配规则（一行一条）", &self.preset_tab.form_match_rules)
                        .on_input(Message::PresetFormMatchRulesChanged);

                    let mut form_col = column![
                        text(title).size(16).color(iced::Color::WHITE),
                        name_input,
                        desc_input,
                        rules_input,
                    ];

                    if let Some(ref err) = self.preset_tab.form_error {
                        form_col = form_col.push(text(err.clone()).color(iced::Color::from_rgb(1.0, 0.3, 0.3)));
                    }

                    form_col = form_col.push(
                        row![
                            iced::widget::horizontal_space(),
                            button(text("取消")).on_press(Message::PresetFormDismissed),
                            button(text("保存")).on_press(Message::PresetFormSaveRequested),
                        ].spacing(8)
                    );

                    col = col.push(
                        container(form_col.spacing(8).padding(16))
                            .width(Length::Fill)
                            .style(|_: &iced::Theme| iced::widget::container::Style {
                                background: Some(iced::Color::from_rgb(0.15, 0.15, 0.15).into()),
                                border: iced::Border::default().rounded(4),
                                ..Default::default()
                            })
                    );
                }

                // delete confirm overlay
                if let Some(ref target) = self.preset_tab.delete_target {
                    let confirm_col = column![
                        text(format!("确定删除预设「{target}」？")).color(iced::Color::WHITE),
                        row![
                            iced::widget::horizontal_space(),
                            button(text("取消")).on_press(Message::PresetDeleteDismissed),
                            button(text("删除")).on_press(Message::PresetDeleteConfirmed(target.clone())),
                        ].spacing(8),
                    ].spacing(8).padding(16);

                    col = col.push(
                        container(confirm_col)
                            .width(Length::Fill)
                            .style(|_: &iced::Theme| iced::widget::container::Style {
                                background: Some(iced::Color::from_rgb(0.15, 0.15, 0.15).into()),
                                border: iced::Border::default().rounded(4),
                                ..Default::default()
                            })
                    );
                }

                container(col.spacing(4))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
                        ..Default::default()
                    })
                    .into()
            }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: may need minor tweaks for iced API compatibility

- [ ] **Step 3: Commit**

```bash
git add src/app/mod.rs
git commit -m "feat: update preset tab view with CRUD UI"
```

---

### Task 6: Tests — preset CRUD tests

**Files:**
- Modify: `tests/data_tests.rs`

- [ ] **Step 1: Add delete_preset and rename_preset tests**

Add after `test_preset_not_found`:

```rust
#[test]
fn test_delete_preset() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let preset = Preset {
        name: "to_delete".to_string(),
        description: None,
        match_rules: vec![],
        fsrs_parameters: None,
        trained_at: None,
    };

    fs.save_preset(&preset).unwrap();
    assert!(fs.get_preset("to_delete").is_ok());

    fs.delete_preset("to_delete").unwrap();
    assert!(fs.get_preset("to_delete").is_err());
}

#[test]
fn test_rename_preset() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    let preset = Preset {
        name: "old_name".to_string(),
        description: Some("test".to_string()),
        match_rules: vec!["study/*".to_string()],
        fsrs_parameters: None,
        trained_at: None,
    };

    fs.save_preset(&preset).unwrap();
    fs.rename_preset("old_name", "new_name").unwrap();

    assert!(fs.get_preset("old_name").is_err());
    let renamed = fs.get_preset("new_name").unwrap();
    assert_eq!(renamed.name, "new_name");
    assert_eq!(renamed.description, Some("test".to_string()));
    assert_eq!(renamed.match_rules, vec!["study/*".to_string()]);
}

#[test]
fn test_preset_crud_workflow() {
    let dir = tempdir().unwrap();
    let fs = DataFs::init(dir.path().to_path_buf()).unwrap();

    // create
    let p1 = Preset {
        name: "english".to_string(),
        description: Some("英语高频词".to_string()),
        match_rules: vec!["main/english/**".to_string()],
        fsrs_parameters: None,
        trained_at: None,
    };
    fs.save_preset(&p1).unwrap();
    let presets = fs.list_presets().unwrap();
    assert!(presets.iter().any(|p| p.name == "english"));

    // rename
    fs.rename_preset("english", "english-v2").unwrap();
    let presets = fs.list_presets().unwrap();
    assert!(presets.iter().any(|p| p.name == "english-v2"));
    assert!(!presets.iter().any(|p| p.name == "english"));

    // delete
    fs.delete_preset("english-v2").unwrap();
    let presets = fs.list_presets().unwrap();
    assert!(!presets.iter().any(|p| p.name == "english-v2"));
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test data_tests test_delete_preset test_rename_preset test_preset_crud_workflow -- --nocapture`
Expected: 3 passed

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add tests/data_tests.rs
git commit -m "test: add preset CRUD tests"
```

---

### Task 7: Full verification and cleanup

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: all pass

- [ ] **Step 2: Check for clippy warnings**

Run: `cargo clippy --all-targets 2>&1 | head -30`

- [ ] **Step 3: Final commit if any fixes needed**

```bash
git add -A
git commit -m "chore: fix clippy warnings"
```
