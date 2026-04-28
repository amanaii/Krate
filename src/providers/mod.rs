pub mod curseforge;
pub mod modrinth;

use anyhow::Result;

use crate::models::{PluginSummary, PluginVersion, ProviderKind};

pub const DEFAULT_LOADERS: &[&str] = &[
    "paper", "purpur", "spigot", "bukkit", "folia", "fabric", "forge", "neoforge",
];

pub async fn search(provider: ProviderKind, query: &str) -> Result<Vec<PluginSummary>> {
    match provider {
        ProviderKind::Modrinth => modrinth::search(query).await,
        ProviderKind::CurseForge => curseforge::search(query).await,
    }
}

pub async fn versions(provider: ProviderKind, project_id: &str) -> Result<Vec<PluginVersion>> {
    match provider {
        ProviderKind::Modrinth => modrinth::versions(project_id).await,
        ProviderKind::CurseForge => curseforge::versions(project_id).await,
    }
}
