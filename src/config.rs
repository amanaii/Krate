use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::models::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: ProviderKind,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Modrinth,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let bytes = tokio::fs::read(&path)
            .await
            .with_context(|| format!("read config {}", path.display()))?;
        serde_json::from_slice(&bytes).with_context(|| format!("parse config {}", path.display()))
    }

    pub async fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let bytes = serde_json::to_vec_pretty(self)?;
        tokio::fs::write(&path, bytes)
            .await
            .with_context(|| format!("write config {}", path.display()))
    }
}

fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .or_else(|| std::env::current_dir().ok())
        .context("locate config directory")?;
    Ok(dir.join("krate").join("config.json"))
}
