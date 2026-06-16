use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::data::{DataError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub data_dir: Option<String>,
    pub default_preset: Option<String>,
    pub theme: Option<String>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("TMD_CONFIG_DIR") {
            return PathBuf::from(dir);
        }
        directories::ProjectDirs::from("com", "time-manager", "time-manager")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(Self::default_config_dir)
    }

    fn default_config_dir() -> PathBuf {
        #[cfg(windows)]
        {
            if let Ok(profile) = std::env::var("USERPROFILE") {
                return PathBuf::from(profile).join("AppData/Roaming/time-manager");
            }
            PathBuf::from(r"C:\Users\Default\.config\time-manager")
        }
        #[cfg(not(windows))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".config/time-manager")
        }
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
