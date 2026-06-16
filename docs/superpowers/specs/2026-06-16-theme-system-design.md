# 主题系统设计

## 概述

解决 Windows/Linux 间颜色不统一问题，实现统一配色与主题切换。

## 架构

复用 charge-controller 的 `AppTheme` 模式：

```
Config (persist theme string)
  │
  ▼
App.theme: AppTheme
  │
  ├── main.rs: .theme(|s| s.theme.to_iced_theme())
  ├── main.rs: .window(transparent: bool)  // Linux only
  │
  └── view functions: use iced theme-aware styles
       + custom AppTheme custom_palette for Transparent
```

## AppTheme 枚举

`gui/styles/theme.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Light,
    Dark,
    #[cfg(target_os = "linux")]
    Transparent,
}
```

平台感知方法：
- `toggle(self) -> Self` — Linux: Light→Dark→Transparent→Light, Windows: Light↔Dark
- `display(self) -> &'static str` — `"浅色"`, `"深色"`, `"透明"`
- `to_iced_theme(self) -> iced::Theme` — Light/Dark 映射内置, Transparent 用 `Theme::custom()`
- `default() -> Self` — `#[cfg(windows)]` 返回 Light, `#[cfg(not(windows))]` 返回 Transparent

## 颜色系统

三层颜色来源：

| 层 | 来源 | 适用场景 |
|----|------|---------|
| **iced 内置主题色** | `Theme::Light/Dark` 的 `Palette` | 所有 widget 默认样式 |
| **自定义调色板** | Transparent 用 `Theme::custom()` 自定义 `Palette` | 覆盖背景/文字/主色 |
| **urgency 颜色** | 保留现有 `Theme.urgency_*` 字段 | 复习紧迫度标签 |

所有硬编码 `from_rgb` / `Color::WHITE` 的 style 闭包按以下规则处理：

1. **标准 widget**（button, text, container 等）使用 `iced::widget::*::Style` theme-aware 函数，让 iced 从 `Theme::Palette` 自动取色
2. **自定义背景/前景** 引用 `Theme::Palette` 中的颜色（`palette.background`, `palette.text`, `palette.primary` 等）
3. **urgency 颜色** 保留在 `gui::Theme` 自定义结构中

## 主题持久化

`Config` 新增字段：

```rust
pub struct Config {
    pub data_dir: Option<String>,
    pub default_preset: Option<String>,
    pub theme: Option<String>,  // "light" | "dark" | "transparent"
}
```

应用启动时，`Config.theme` → `AppTheme::from_str()` → `App.theme`。

## 窗口透明

`main.rs` 中根据平台和主题设置：

```rust
application(|| { ... }, update, view)
    .theme(|app: &App| app.theme.to_iced_theme())
    .window(if cfg!(target_os = "linux") {
        iced::window::Settings {
            transparent: matches!(app.theme, AppTheme::Transparent),
            ..Default::default()
        }
    } else {
        iced::window::Settings::default()
    })
```

但 `.window()` 在 `application()` 调用时静态设置，运行时无法修改。这意味着：
- 初始窗口根据默认主题配置透明属性
- 如果从 Transparent 切换到 Dark/Light，窗口透明属性保持不变（iced 目前不支持运行时更改 `transparent`）
- 这是已知限制，影响较小——Transparent 背景的 alpha 通道控制透明程度，切换到不透明主题只需背景完全不透明即可

## 设置界面

Settings tab 新增"主题"下拉选择器：

```
[主题: 浅色 ▼]  ← pick_list
```

选择后立即生效并保存到 Config。

## GUI 颜色迁移

当前 141 处 `from_rgb` + 154 处 `Color::WHITE` + 60 处 `style()` 的迁移策略：

**batch 1 — 高价值 (view 主体布局)**: container 背景色、text 颜色 —— 移除显式 style，依赖 iced 默认主题
**batch 2 — widget 样式**: button 背景色（开始/暂停/停止等操作按钮）—— 保留语义颜色但通过 theme palette 取色
**batch 3 — 次要元素**: 列表项边框、分隔线 —— 移除显式颜色，依赖默认

每个 batch 的目标：减少硬编码颜色数量，同时不破坏 UI 的可读性和信息层级。

## 文件改动清单

| 文件 | 改动 |
|------|------|
| `src/gui/styles/theme.rs` | 重写：`Theme` 结构 → `AppTheme` 枚举 + 调色板函数 |
| `src/gui/styles/mod.rs` | 重新导出 `AppTheme` |
| `src/gui/messages.rs` | 新增 `ThemeChanged(AppTheme)` 消息 |
| `src/data/config.rs` | `Config` 新增 `theme: Option<String>` |
| `src/app/mod.rs` | `App` 新增 `theme: AppTheme` 字段；update 处理 ThemeChanged |
| `src/app/settings_tab.rs` | 新增 `form_theme` 和 `theme_options` |
| `src/main.rs` | `.theme()` 和 `.window()` 集成 |
| `src/gui/components/*.rs` | 逐步迁移 style 闭包 |
| `src/app/*_tab.rs` | 逐步迁移 style 闭包（主要在 `app/mod.rs` 的 view 函数中） |
