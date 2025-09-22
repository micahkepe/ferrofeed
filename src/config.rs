use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

/// The default app config TOML file.
const APP_CONFIG_FILE: &str = "ferrofeed.toml";

/// The default app SQLite file.
const DEFAULT_DB_NAME: &str = "ferrofeed.db";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Path to the ferrofeed database file
    pub database_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let base_dirs = BaseDirs::new().expect("couldn't get base directories, HOME not set?");
        let data_dir = base_dirs.home_dir().join(".local/share/ferrofeed");
        Self {
            database_path: data_dir.join(DEFAULT_DB_NAME),
        }
    }
}

impl Config {
    /// Load and parse the user's configuration file, or the passed override path.
    pub fn load(config_path_override: Option<PathBuf>) -> Result<Self> {
        let default_config_path = BaseDirs::new()
            .expect("unable to determine base directories")
            .home_dir()
            .join(".config/ferrofeed")
            .join(APP_CONFIG_FILE);
        let path = config_path_override.unwrap_or(default_config_path);

        if path.exists() {
            // Load data
            let data = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config file at {}", path.display()))?;
            let cfg: Config = toml::from_str(&data)
                .with_context(|| format!("failed to parse TOML config at {}", path.display()))?;
            Ok(cfg)
        } else {
            // Create default
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?
            }
            let default = Config::default();
            std::fs::write(&path, toml::to_string_pretty(&default)?)?;
            Ok(Config::default())
        }
    }
}
