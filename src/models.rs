use std::{fmt, str::FromStr};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    Modrinth,
    CurseForge,
}

impl ProviderKind {
    pub const ALL: [Self; 2] = [Self::Modrinth, Self::CurseForge];
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Modrinth => write!(f, "Modrinth"),
            Self::CurseForge => write!(f, "CurseForge"),
        }
    }
}

impl FromStr for ProviderKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "modrinth" => Ok(Self::Modrinth),
            "curseforge" | "curse-forge" => Ok(Self::CurseForge),
            _ => Err(anyhow!("unknown provider: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSummary {
    pub provider: ProviderKind,
    pub project_id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub downloads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<PluginFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginFile {
    pub filename: String,
    pub url: String,
    pub primary: bool,
    pub size: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DownloadPlan {
    pub plugin: PluginSummary,
    pub version: PluginVersion,
    pub file: PluginFile,
    pub game_version: String,
    pub loader: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstalledManifest {
    pub plugins: Vec<InstalledPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub name: String,
    pub provider: ProviderKind,
    pub project_id: String,
    pub slug: String,
    #[serde(default)]
    pub version_id: String,
    #[serde(default)]
    pub version_number: String,
    pub game_version: String,
    pub loader: String,
    pub filename: String,
}
