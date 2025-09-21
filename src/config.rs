use directories::ProjectDirs;
use serde::Deserialize;
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

/// The default app config TOML file.
const APP_CONFIG_FILE: &str = "ferrofeed.toml";

/// The default app SQLite file.
const DEFAULT_DB_NAME: &str = "ferrofeed.db";

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Path to the ferrofeed database file
    pub database_path: PathBuf,
}

/// Get the location of the cache, data, and config files as [`ProjectDirs`].
fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("org", "ferrofeed", "ferrofeed")
        .expect("Unable to determine config directory")
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = get_project_dirs();
        let data_dir = project_dirs.data_dir();
        Self {
            database_path: data_dir.join(DEFAULT_DB_NAME),
        }
    }
}

impl Config {
    pub fn load(config_path_override: Option<PathBuf>) -> Result<Self> {
        let path = if let Some(p) = config_path_override {
            p
        } else {
            let project_dirs = get_project_dirs();
            project_dirs.config_dir().join(APP_CONFIG_FILE)
        };

        if path.exists() {
            // Load data
            let data = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config file at {}", path.display()))?;
            let cfg: Config = toml::from_str(&data)
                .with_context(|| format!("failed to parse TOML config at {}", path.display()))?;
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }
}
