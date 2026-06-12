use std::path::PathBuf;

pub struct SettingsTabState {
    pub data_dir: PathBuf,
}

impl SettingsTabState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }
}