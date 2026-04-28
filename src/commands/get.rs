use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::{
    config::Config,
    downloader,
    models::{DownloadPlan, PluginSummary, PluginVersion},
    providers::{self, DEFAULT_LOADERS},
    tui::{SelectAction, Tui},
};

pub async fn run(query: &str) -> Result<()> {
    let config = Config::load().await?;
    let mut current_query = query.to_string();
    let mut plugins = providers::search(config.provider, &current_query).await?;
    if plugins.is_empty() {
        println!("no plugins found");
        return Ok(());
    }
    let mut plugin_rows = plugin_items(&plugins);

    let mut ui = Tui::new()?;
    let mut plans = Vec::new();

    loop {
        let markers = selected_markers(&plugins, &plans);
        let result = ui.select_many_marked(
            &format!("Search results: {current_query}"),
            &plugin_rows,
            &markers,
        )?;
        match result.action {
            SelectAction::Search => {
                let Some(next_query) = ui.input("New plugin search", &current_query)? else {
                    continue;
                };
                ui.status("Searching", &[format!("Looking for {next_query}...")], "")?;
                let next_plugins = providers::search(config.provider, &next_query).await?;
                if next_plugins.is_empty() {
                    ui.message(
                        "No results",
                        &[format!("No plugins found for {next_query}.")],
                    )?;
                    continue;
                }
                current_query = next_query;
                plugins = next_plugins;
                plugin_rows = plugin_items(&plugins);
                continue;
            }
            SelectAction::Install => {
                if plans.is_empty() {
                    ui.message(
                        "Nothing selected",
                        &["Select at least one plugin first.".into()],
                    )?;
                    continue;
                }
                break;
            }
            SelectAction::Confirm if !result.indices.is_empty() => {}
            _ => return Ok(()),
        }

        for index in result.indices {
            let plugin = plugins[index].clone();
            if let Some(plan) = choose_download_plan(&mut ui, plugin).await? {
                plans.retain(|existing: &DownloadPlan| {
                    existing.plugin.project_id != plan.plugin.project_id
                        || existing.plugin.provider != plan.plugin.provider
                });
                plans.push(plan);
            }
        }
    }

    if plans.is_empty() {
        return Ok(());
    }

    download_plans(&mut ui, &plans).await?;

    Ok(())
}

fn plugin_items(plugins: &[PluginSummary]) -> Vec<String> {
    plugins
        .iter()
        .map(|plugin| {
            format!(
                "{}  {} downloads  {}",
                plugin.name, plugin.downloads, plugin.description
            )
        })
        .collect()
}

fn selected_markers(plugins: &[PluginSummary], plans: &[DownloadPlan]) -> Vec<bool> {
    plugins
        .iter()
        .map(|plugin| {
            plans.iter().any(|plan| {
                plan.plugin.provider == plugin.provider
                    && plan.plugin.project_id == plugin.project_id
            })
        })
        .collect()
}

async fn download_plans(ui: &mut Tui, plans: &[DownloadPlan]) -> Result<()> {
    let mut complete = Vec::new();
    for (index, plan) in plans.iter().enumerate() {
        let total_count = plans.len();
        let path = downloader::download_with_progress(plan, Path::new("."), |progress| {
            let lines = vec![
                format!("Plugin {}/{}: {}", index + 1, total_count, plan.plugin.name),
                format!("Plugin version: {}", plan.version.version_number),
                format!("Target: {} / {}", plan.game_version, plan.loader),
                progress_text(progress.downloaded, progress.total),
            ];
            ui.progress(
                "Downloading plugins",
                &lines,
                progress_ratio(progress.downloaded, progress.total),
                &progress_label(progress.downloaded, progress.total),
                "Please wait",
            )
        })
        .await?;
        complete.push(format!("[ok] {} -> {}", plan.plugin.name, path.display()));
    }

    ui.message("Download complete", &complete)?;
    Ok(())
}

pub(super) fn progress_text(downloaded: u64, total: Option<u64>) -> String {
    match total {
        Some(total) if total > 0 => format!("{} / {}", bytes(downloaded), bytes(total)),
        _ => format!("{} downloaded", bytes(downloaded)),
    }
}

pub(super) fn progress_ratio(downloaded: u64, total: Option<u64>) -> Option<f64> {
    let total = total?;
    if total == 0 {
        return None;
    }
    Some(downloaded as f64 / total as f64)
}

pub(super) fn progress_label(downloaded: u64, total: Option<u64>) -> String {
    match progress_ratio(downloaded, total) {
        Some(ratio) => format!("{:.0}%", (ratio * 100.0).clamp(0.0, 100.0)),
        None => bytes(downloaded),
    }
}

fn bytes(value: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    let value = value as f64;
    if value >= MIB {
        format!("{:.1} MiB", value / MIB)
    } else if value >= KIB {
        format!("{:.1} KiB", value / KIB)
    } else {
        format!("{} B", value as u64)
    }
}

async fn choose_download_plan(ui: &mut Tui, plugin: PluginSummary) -> Result<Option<DownloadPlan>> {
    let versions = providers::versions(plugin.provider, &plugin.project_id)
        .await
        .with_context(|| format!("load versions for {}", plugin.name))?;
    if versions.is_empty() {
        ui.message(
            "No versions",
            &[format!("{} has no downloadable versions.", plugin.name)],
        )?;
        return Ok(None);
    }

    let game_versions = unique_game_versions(&versions);
    let version_items: Vec<String> = game_versions.iter().map(ToString::to_string).collect();
    let game_result = ui.select_one(
        &format!("{}: Minecraft version", plugin.name),
        &version_items,
    )?;
    let Some(game_index) = game_result.indices.first().copied() else {
        return Ok(None);
    };
    let game_version = game_versions[game_index].clone();

    let loaders = loaders_for_version(&versions, &game_version);
    let loader_items: Vec<String> = loaders.iter().map(ToString::to_string).collect();
    let loader_result =
        ui.select_one(&format!("{}: Server software", plugin.name), &loader_items)?;
    let Some(loader_index) = loader_result.indices.first().copied() else {
        return Ok(None);
    };
    let loader = loaders[loader_index].clone();

    let version = versions
        .into_iter()
        .find(|version| {
            version
                .game_versions
                .iter()
                .any(|value| value == &game_version)
                && version.loaders.iter().any(|value| value == &loader)
                && !version.files.is_empty()
        })
        .ok_or_else(|| {
            anyhow!(
                "no matching version for {} {game_version} {loader}",
                plugin.name
            )
        })?;
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .cloned()
        .ok_or_else(|| anyhow!("no file for {}", plugin.name))?;

    Ok(Some(DownloadPlan {
        plugin,
        version,
        file,
        game_version,
        loader,
    }))
}

pub fn unique_game_versions(versions: &[PluginVersion]) -> Vec<String> {
    let mut values = Vec::new();
    for version in versions {
        for game_version in &version.game_versions {
            if !values.contains(game_version) {
                values.push(game_version.clone());
            }
        }
    }
    values.sort_by(|left, right| compare_versions(right, left));
    values
}

pub fn loaders_for_version(versions: &[PluginVersion], game_version: &str) -> Vec<String> {
    let mut values = Vec::new();
    for preferred in DEFAULT_LOADERS {
        if versions.iter().any(|version| {
            version
                .game_versions
                .iter()
                .any(|value| value == game_version)
                && version.loaders.iter().any(|loader| loader == preferred)
        }) {
            values.push((*preferred).to_string());
        }
    }

    for version in versions {
        if !version
            .game_versions
            .iter()
            .any(|value| value == game_version)
        {
            continue;
        }
        for loader in &version.loaders {
            if !values.contains(loader) {
                values.push(loader.clone());
            }
        }
    }

    values
}

fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts = version_parts(left);
    let right_parts = version_parts(right);
    left_parts.cmp(&right_parts).then_with(|| left.cmp(right))
}

fn version_parts(value: &str) -> Vec<u32> {
    value
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{compare_versions, unique_game_versions};
    use crate::models::PluginVersion;

    #[test]
    fn sorts_minecraft_versions_numerically_descending() {
        let mut versions = ["1.8.9", "1.21.11", "1.21.10", "26.1"]
            .map(String::from)
            .to_vec();
        versions.sort_by(|left, right| compare_versions(right, left));

        assert_eq!(versions, ["26.1", "1.21.11", "1.21.10", "1.8.9"]);
    }

    #[test]
    fn unique_game_versions_are_sorted() {
        let versions = vec![PluginVersion {
            id: "id".into(),
            name: "name".into(),
            version_number: "version".into(),
            game_versions: vec!["1.8.9".into(), "1.21.11".into(), "1.21.10".into()],
            loaders: Vec::new(),
            files: Vec::new(),
        }];

        assert_eq!(
            unique_game_versions(&versions),
            ["1.21.11", "1.21.10", "1.8.9"]
        );
    }
}
