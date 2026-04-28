use std::path::Path;

use anyhow::{Result, anyhow};

use crate::{
    downloader,
    models::{DownloadPlan, PluginSummary},
    providers::{self, DEFAULT_LOADERS},
    tui::Tui,
};

use super::get::{progress_label, progress_ratio, progress_text, unique_game_versions};

pub async fn run(target_version: Option<String>, target_loader: Option<String>) -> Result<()> {
    let directory = Path::new(".");
    let mut manifest = downloader::load_manifest(directory).await?;
    if manifest.plugins.is_empty() {
        println!(
            "no {} found in current directory",
            downloader::MANIFEST_FILE
        );
        return Ok(());
    }

    let mut ui = Tui::new()?;

    let version = match target_version {
        Some(value) => value,
        None => {
            ask_version(
                &mut ui,
                &manifest.plugins[0].provider,
                &manifest.plugins[0].project_id,
            )
            .await?
        }
    };

    let loader = match target_loader {
        Some(value) => value,
        None => ask_loader(&mut ui)?,
    };

    let total_count = manifest.plugins.len();
    let mut complete = Vec::new();
    let mut warnings = Vec::new();

    for (index, installed) in manifest.plugins.clone().into_iter().enumerate() {
        ui.status(
            "Updating plugins",
            &[
                format!("Plugin {}/{}: {}", index + 1, total_count, installed.name),
                format!("Target: {} / {}", version, loader),
                "Looking for a compatible release...".into(),
            ],
            "Please wait",
        )?;
        let versions = providers::versions(installed.provider, &installed.project_id).await?;
        let Some(version_match) = versions.into_iter().find(|candidate| {
            candidate
                .game_versions
                .iter()
                .any(|value| value == &version)
                && candidate.loaders.iter().any(|value| value == &loader)
                && !candidate.files.is_empty()
        }) else {
            warnings.push(format!(
                "warning: {} does not support Minecraft {} / {}",
                installed.name, version, loader
            ));
            continue;
        };

        let file = version_match
            .files
            .iter()
            .find(|file| file.primary)
            .or_else(|| version_match.files.first())
            .cloned()
            .ok_or_else(|| anyhow!("no file for {}", installed.name))?;
        let plugin = PluginSummary {
            provider: installed.provider,
            project_id: installed.project_id.clone(),
            slug: installed.slug.clone(),
            name: installed.name.clone(),
            description: String::new(),
            downloads: 0,
        };
        let plan = DownloadPlan {
            plugin,
            version: version_match,
            file,
            game_version: version.clone(),
            loader: loader.clone(),
        };

        let old_path = directory.join(&installed.filename);
        let new_path = downloader::download_with_progress(&plan, directory, |progress| {
            let lines = vec![
                format!("Plugin {}/{}: {}", index + 1, total_count, plan.plugin.name),
                format!("Installed version: {}", installed.version_number),
                format!("New version: {}", plan.version.version_number),
                format!("Target: {} / {}", plan.game_version, plan.loader),
                progress_text(progress.downloaded, progress.total),
            ];
            ui.progress(
                "Updating plugins",
                &lines,
                progress_ratio(progress.downloaded, progress.total),
                &progress_label(progress.downloaded, progress.total),
                "Please wait",
            )
        })
        .await?;
        if old_path != new_path && old_path.exists() {
            let _ = tokio::fs::remove_file(old_path).await;
        }
        complete.push(format!("[ok] {} -> {}", installed.name, new_path.display()));
    }

    manifest = downloader::load_manifest(directory).await?;
    downloader::save_manifest(directory, &manifest).await?;
    ui.message(
        "Update complete",
        &summary_lines(&version, &loader, &complete, &warnings),
    )?;
    Ok(())
}

fn summary_lines(
    version: &str,
    loader: &str,
    complete: &[String],
    warnings: &[String],
) -> Vec<String> {
    let mut lines = vec![
        format!("Target: {} / {}", version, loader),
        format!("Updated: {}", complete.len()),
    ];

    if !complete.is_empty() {
        lines.push(String::new());
        lines.extend(complete.iter().cloned());
    }

    if !warnings.is_empty() {
        lines.push(String::new());
        lines.extend(warnings.iter().cloned());
    }

    lines
}

async fn ask_version(
    ui: &mut Tui,
    provider: &crate::models::ProviderKind,
    project_id: &str,
) -> Result<String> {
    let versions = providers::versions(*provider, project_id).await?;
    let game_versions = unique_game_versions(&versions);
    let items: Vec<String> = game_versions.iter().map(ToString::to_string).collect();
    let result = ui.select_one("Update target Minecraft version", &items)?;
    let Some(index) = result.indices.first().copied() else {
        return Err(anyhow!("no version selected"));
    };
    Ok(game_versions[index].clone())
}

fn ask_loader(ui: &mut Tui) -> Result<String> {
    let items: Vec<String> = DEFAULT_LOADERS
        .iter()
        .map(|value| value.to_string())
        .collect();
    let result = ui.select_one("Update target server software", &items)?;
    let Some(index) = result.indices.first().copied() else {
        return Err(anyhow!("no server software selected"));
    };
    Ok(items[index].clone())
}
