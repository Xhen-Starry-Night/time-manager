use iced::{Theme, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Self::Transparent => Theme::custom_with_fn(
                "transparent",
                iced::theme::Palette {
                    background: Color::from_rgba(0.102, 0.106, 0.149, 0.85),
                    text: Color::from_rgb(0.753, 0.792, 0.961),
                    primary: Color::from_rgb(0.204, 0.596, 0.859),
                    success: Color::from_rgb(0.153, 0.682, 0.376),
                    warning: Color::from_rgb(0.953, 0.612, 0.071),
                    danger: Color::from_rgb(0.906, 0.298, 0.235),
                },
                iced::theme::palette::Extended::generate,
            ),
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
