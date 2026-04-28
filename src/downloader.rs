use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::models::{DownloadPlan, InstalledManifest, InstalledPlugin};

pub const MANIFEST_FILE: &str = ".krate.json";

#[derive(Debug, Clone, Copy)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

pub async fn download_with_progress<F>(
    plan: &DownloadPlan,
    directory: &Path,
    mut on_progress: F,
) -> Result<PathBuf>
where
    F: FnMut(DownloadProgress) -> Result<()>,
{
    tokio::fs::create_dir_all(directory).await?;

    let filename = sanitize_filename(&plan.file.filename);
    let path = directory.join(&filename);
    let response = reqwest::get(&plan.file.url).await?.error_for_status()?;
    let total = response.content_length().or(plan.file.size);
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&path)
        .await
        .with_context(|| format!("create {}", path.display()))?;
    let mut downloaded = 0;

    on_progress(DownloadProgress { downloaded, total })?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(DownloadProgress { downloaded, total })?;
    }
    file.flush().await?;
    record_install(plan, &filename, directory).await?;
    Ok(path)
}

pub async fn load_manifest(directory: &Path) -> Result<InstalledManifest> {
    let path = directory.join(MANIFEST_FILE);
    if !path.exists() {
        return Ok(InstalledManifest::default());
    }
    let bytes = tokio::fs::read(&path).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub async fn save_manifest(directory: &Path, manifest: &InstalledManifest) -> Result<()> {
    let path = directory.join(MANIFEST_FILE);
    let bytes = serde_json::to_vec_pretty(manifest)?;
    tokio::fs::write(path, bytes).await?;
    Ok(())
}

async fn record_install(plan: &DownloadPlan, filename: &str, directory: &Path) -> Result<()> {
    let mut manifest = load_manifest(directory).await?;
    manifest
        .plugins
        .retain(|plugin| plugin.project_id != plan.plugin.project_id);
    manifest.plugins.push(InstalledPlugin {
        name: plan.plugin.name.clone(),
        provider: plan.plugin.provider,
        project_id: plan.plugin.project_id.clone(),
        slug: plan.plugin.slug.clone(),
        version_id: plan.version.id.clone(),
        version_number: plan.version.version_number.clone(),
        game_version: plan.game_version.clone(),
        loader: plan.loader.clone(),
        filename: filename.to_string(),
    });
    save_manifest(directory, &manifest).await
}

fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}
