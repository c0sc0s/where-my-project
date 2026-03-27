use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use dirs::home_dir;

use crate::core::models::Config;

pub fn config_path() -> Result<PathBuf> {
    let home = home_dir().context("failed to determine home directory")?;
    Ok(home.join(".proj.json"))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file at {}", path.display()))?;

    if content.trim().is_empty() {
        return Ok(Config::default());
    }

    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse config file at {}", path.display()))
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;
    let content = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write config file at {}", path.display()))
}
