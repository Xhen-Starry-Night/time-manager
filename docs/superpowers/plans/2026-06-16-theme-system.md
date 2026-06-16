# Theme System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement platform-aware theme system (Light/Dark/Transparent) with settings UI.

**Architecture:** AppTheme enum mapped to iced's Theme via `to_iced_theme()`, persisted in Config, integrated via `application.theme()` closure. Colors migrated from hardcoded `from_rgb` to theme palette references.

**Tech Stack:** Rust, iced 0.14, serde, toml

---

### Task 1: AppTheme enum + to_iced_theme()

**Files:**
- Modify: `src/gui/styles/theme.rs`
- Modify: `src/gui/styles/mod.rs`

- [ ] **Step 1: Rewrite theme.rs with AppTheme enum**

Replace the existing dead `Theme` struct with:

```rust
use iced::{Theme, Color};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Light,
    Dark,
    #[cfg(target_os = "linux")]
    Transparent,
}

impl AppTheme {
    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => {
                #[cfg(target_os = "linux")]
                { Self::Transparent }
                #[cfg(not(target_os = "linux"))]
                { Self::Light }
            }
            #[cfg(target_os = "linux")]
            Self::Transparent => Self::Light,
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            Self::Light => "浅色",
            Self::Dark => "深色",
            #[cfg(target_os = "linux")]
            Self::Transparent => "透明",
        }
    }

    pub fn to_iced_theme(&self) -> Theme {
        match self {
            Self::Light => Theme::Light,
            Self::Dark => Theme::Dark,
            #[cfg(target_os = "linux")]
            Self::Transparent => Theme::custom_with_fn("transparent", |theme| {
                iced::theme::Palette {
                    background: Color::from_rgba(0.102, 0.106, 0.149, 0.85),
                    text: Color::from_rgb(0.753, 0.792, 0.961),
                    primary: Color::from_rgb(0.204, 0.596, 0.859),
                    success: Color::from_rgb(0.153, 0.682, 0.376),
                    danger: Color::from_rgb(0.906, 0.298, 0.235),
                    ..theme.palette()
                }
            }),
        }
    }

    pub fn default() -> Self {
        #[cfg(target_os = "windows")]
        { Self::Light }
        #[cfg(not(target_os = "windows"))]
        { Self::Transparent }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            #[cfg(target_os = "linux")]
            "transparent" => Self::Transparent,
            _ => Self::default(),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            #[cfg(target_os = "linux")]
            Self::Transparent => "transparent",
        }
    }
}
```

- [ ] **Step 2: Update mod.rs**

```rust
pub mod theme;
pub use theme::AppTheme;
```

- [ ] **Step 3: Build check**

Run: `cargo check`
Expected: OK (Dead Theme struct removal may cause unused warnings — the struct was never imported)

- [ ] **Step 4: Commit**

```bash
git add src/gui/styles/
git commit -m "feat: add AppTheme enum with to_iced_theme()"
```

---

### Task 2: Config persistence for theme

**Files:**
- Modify: `src/data/config.rs`
- Modify: `src/data/models.rs` (add Serialize/Deserialize derive for AppTheme? No, it's in gui/styles)

- [ ] **Step 1: Add theme field to Config**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub data_dir: Option<String>,
    pub default_preset: Option<String>,
    pub theme: Option<String>,
}
```

- [ ] **Step 2: Build check**

Run: `cargo check`
Expected: OK

- [ ] **Step 3: Commit**

```bash
git add src/data/config.rs
git commit -m "feat: add theme field to Config"
```

---

### Task 3: Theme state in App + Message + Update

**Files:**
- Modify: `src/gui/messages.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/settings_tab.rs`

- [ ] **Step 1: Add ThemeChanged message**

```rust
// In messages.rs
ThemeChanged(AppTheme),
```

- [ ] **Step 2: Add AppTheme import and theme field to SettingsTabState**

```rust
use crate::gui::styles::AppTheme;

pub struct SettingsTabState {
    pub config_dir: PathBuf,
    pub config_path: PathBuf,
    pub config: Config,
    pub form_data_dir: String,
    pub form_default_preset: String,
    pub form_theme: AppTheme,
    pub message: Option<String>,
    pub message_is_error: bool,
}
```

Update `new()` to accept `theme: AppTheme` and store it.

- [ ] **Step 3: Add theme field to App struct**

```rust
pub struct App {
    pub default_preset: String,
    pub theme: AppTheme,
    active_tab: TabId,
    ...
}
```

- [ ] **Step 4: Initialize theme in App::run()**

After loading config:
```rust
let theme = config.theme
    .as_deref()
    .map(AppTheme::from_str)
    .unwrap_or_else(AppTheme::default);
```

Pass `theme` to `SettingsTabState::new()` and store in App.

- [ ] **Step 5: Handle ThemeChanged in App::update()**

```rust
Message::ThemeChanged(theme) => {
    self.theme = theme;
    self.settings_tab.form_theme = theme;
    self.settings_tab.config.theme = Some(theme.as_str().to_string());
    let path = self.settings_tab.config_path.clone();
    let config = self.settings_tab.config.clone();
    let _ = config.save(&path);
    Task::none()
}
```

- [ ] **Step 6: Build check**

Run: `cargo check`
Expected: OK

- [ ] **Step 7: Commit**

```bash
git add src/gui/messages.rs src/app/mod.rs src/app/settings_tab.rs
git commit -m "feat: add theme state to App, Message, and SettingsTabState"
```

---

### Task 4: Theme picker in Settings tab

**Files:**
- Modify: `src/app/mod.rs` (settings_tab view section)
- Modify: `src/app/settings_tab.rs`

- [ ] **Step 1: Add theme selector to settings view**

In the update function, add the handler for when settings view needs a theme picker. The settings_tab currently renders in the `TabId::Settings` match arm of `App::view()`.

In the settings section of `App::view()` (line ~2397), add after the default_preset picker:

```rust
row![
    text("主题:").color(iced::Color::WHITE),
    iced::widget::pick_list(
        vec![AppTheme::Light, AppTheme::Dark]
            .into_iter()
            .chain({
                #[cfg(target_os = "linux")]
                Some(AppTheme::Transparent)
                #[cfg(not(target_os = "linux"))]
                None
            })
            .collect::<Vec<_>>(),
        Some(self.theme),
        Message::ThemeChanged,
    )
    .map(|s| Message::ThemeChanged(s)),
].spacing(8).padding(8),
```

Note: pick_list in iced 0.14 requires `From<AppTheme> for String` or implement the Display trait. Add Display impl to AppTheme:

```rust
impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display())
    }
}
```

- [ ] **Step 2: Rebuild app/mod.rs import to include AppTheme if missing**

The settings section already renders in `TabId::Settings` arm of view. Add AppTheme usage in the pick_list.

- [ ] **Step 3: Build check**

Run: `cargo check`
Expected: OK

- [ ] **Step 4: Commit**

```bash
git add src/app/mod.rs src/app/settings_tab.rs src/gui/styles/theme.rs
git commit -m "feat: add theme picker to settings tab"
```

---

### Task 5: main.rs theme integration

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Update main.rs with theme() and window() closures**

The current code calls `time_manager::app::App::run(data_dir)` which internally does `application(...).title(...).subscription(...).run()`.

Since `App::run()` handles the builder internally, we need to add the `.theme()` closure inside that builder chain in `App::run()`.

Current `app/mod.rs`:
```rust
application(|| { ... }, App::update, App::view)
    .title(App::title)
    .subscription(App::subscription)
    .run()
```

Change to:
```rust
let theme = state.theme;
application(|| { ... }, App::update, App::view)
    .title(App::title)
    .theme(move |app: &App| app.theme.to_iced_theme())
    .subscription(App::subscription)
    .run()
```

For window transparency, iced 0.14's `application()` builder doesn't support runtime toggling of `window.transparent`. The initial window settings are set before the state is created. We'll resolve this by:

- Always enabling `transparent` on Linux (since the Transparent palette's background alpha handles the actual transparency, and non-transparent themes just use opaque colors)
- On Windows, transparent is not available/composited, so no change needed

Actually, looking at iced 0.14 API more carefully, the `.window()` method is chained before `.run()` and does not have access to state. So we can't conditionally set it based on loaded theme. Best approach:

```rust
let window_settings = iced::window::Settings {
    #[cfg(target_os = "linux")]
    transparent: true,
    ..Default::default()
};

application(|| { ... }, App::update, App::view)
    .window(window_settings)
    .title(App::title)
    .theme(|app: &App| app.theme.to_iced_theme())
    .subscription(App::subscription)
    .run()
```

This means on Linux the window is always transparent-capable. When non-Transparent themes are active, the background color is fully opaque so there's no visual difference.

- [ ] **Step 2: Build check**

Run: `cargo check`
Expected: OK

- [ ] **Step 3: Commit**

```bash
git add src/main.rs src/app/mod.rs
git commit -m "feat: integrate theme with iced application builder"
```

---

### Task 6: Color migration — container/text base layer

**Files:**
- Modify: `src/app/mod.rs` (view functions)

Target: Remove hardcoded container background colors and text colors for the main layout containers — let iced's Theme handle them.

- [ ] **Step 1: Remove explicit container backgrounds in category/review/timer/todo/preset/settings panels**

Each panel currently has:
```rust
.style(|_: &iced::Theme| iced::widget::container::Style {
    background: Some(iced::Color::from_rgb(0.2, 0.2, 0.2).into()),
    ..Default::default()
})
```

Replace with:
```rust
.style(|theme: &iced::Theme| {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        ..Default::default()
    }
})
```

This applies to ~10 containers rendering tab content panels.

- [ ] **Step 2: Remove explicit text color WHITE on main content text**

Replace `text("xxx").color(iced::Color::WHITE)` with just `text("xxx")` for labels in the main content area (path labels, section headers). Iced's Theme provides appropriate text color automatically.

- [ ] **Step 3: Keep urgency colors and semantic button colors**

Urgency badges, button backgrounds for start/pause/stop — these are semantic (green=go, red=stop, yellow=pause). Keep them but reference from a helper:

```rust
fn semantic_color(kind: &str) -> Color {
    match kind {
        "start" => Color::from_rgb(0.2, 0.7, 0.4),
        "pause" => Color::from_rgb(0.95, 0.61, 0.07),
        "stop" => Color::from_rgb(0.91, 0.30, 0.24),
        _ => Color::from_rgb(0.5, 0.5, 0.5),
    }
}
```

- [ ] **Step 4: Build check**

Run: `cargo check`
Expected: OK

- [ ] **Step 5: Commit**

```bash
git add src/app/mod.rs
git commit -m "refactor: migrate container/text colors to theme palette"
```

---

### Task 7: Color migration — search input and list item styles

**Files:**
- Modify: `src/app/mod.rs`

Target: Search input style, list item backgrounds, filter buttons.

- [ ] **Step 1: Migrate search input theme**

Current:
```rust
.style(move |_: &iced::Theme, _| {
    iced::widget::text_input::Style {
        background: iced::Color::from_rgb(0.25, 0.25, 0.25).into(),
        border: iced::Border { color: iced::Color::from_rgb(0.3, 0.3, 0.3), ... },
        value: iced::Color::WHITE,
        ...
    }
})
```

Replace with theme-aware:
```rust
.style(move |theme: &iced::Theme, _| {
    let palette = theme.extended_palette();
    iced::widget::text_input::Style {
        background: palette.background.weak.color.into(),
        border: iced::Border { color: palette.background.strong.color, ... },
        value: palette.background.text,
        ...
    }
})
```

- [ ] **Step 2: Migrate search result item backgrounds**

Current uses `Color::from_rgb(0.3, 0.3, 0.3)` for item bg. Replace with `palette.background.weak.color` or `palette.background.base.color`.

- [ ] **Step 3: Migrate card/folder action button styling**

These buttons have custom backgrounds:
- "开始计时" button: `Color::from_rgb(0.3, 0.6, 0.4)` 
- Edit/delete buttons in todo/schedule lists

Keep semantic colors but use `palette` for text colors.

- [ ] **Step 4: Build check**

Run: `cargo check`
Expected: OK

- [ ] **Step 5: Commit**

```bash
git add src/app/mod.rs
git commit -m "refactor: migrate search/list styles to theme palette"
```

---

### Task 8: Color migration — component modules

**Files:**
- Modify: `src/gui/components/card_detail.rs`
- Modify: `src/gui/components/timer_display.rs`
- Modify: `src/gui/components/link_timer_modal.rs`
- Modify: `src/gui/components/new_todo_form.rs`
- Modify: `src/gui/components/new_node_modal.rs`
- Modify: `src/gui/components/tree_picker.rs`
- Modify: `src/gui/components/tree_view.rs`

- [ ] **Step 1: Review each component for hardcoded colors**

Each component file currently imports `Color` and uses `from_rgb`. The strategy:

- For `container` styles: use `|theme: &Theme| palette.background.weak.color` 
- For `text` colors: remove `.color()` call where default is fine
- For `button` backgrounds: keep semantic colors (start/pause/stop green/yellow/red)

- [ ] **Step 2: Build check**

Run: `cargo check`
Expected: OK

- [ ] **Step 3: Run tests**

Run: `cargo test`
Expected: 70/70 pass (no behavioral changes)

- [ ] **Step 4: Commit**

```bash
git add src/gui/components/
git commit -m "refactor: migrate component colors to theme palette"
```

---

### Task 9: Final verification

- [ ] **Step 1: Full build and test suite**

```bash
cargo clippy --all-targets -- -D warnings && cargo test
```
Expected: 0 warnings, 70/70 pass

- [ ] **Step 2: Windows target check**

```bash
cargo check --target x86_64-pc-windows-gnu
```
Expected: 0 warnings

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "chore: final theme system cleanup"
```
