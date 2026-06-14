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
    pub fn new(config_dir: PathBuf, config_path: PathBuf, config: Config, actual_data_dir: &str) -> Self {
        let form_data_dir = config.data_dir.clone().unwrap_or_else(|| actual_data_dir.to_string());
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