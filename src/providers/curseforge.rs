use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde::Deserialize;

use crate::{
    http,
    models::{PluginFile, PluginSummary, PluginVersion, ProviderKind},
};

const BASE_URL: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: &str = "432";
const BUKKIT_CLASS_ID: &str = "5";

pub async fn search(query: &str) -> Result<Vec<PluginSummary>> {
    let client = client()?;
    let url = Url::parse_with_params(
        &format!("{BASE_URL}/mods/search"),
        &[
            ("gameId", MINECRAFT_GAME_ID),
            ("classId", BUKKIT_CLASS_ID),
            ("searchFilter", query),
            ("sortField", "6"),
            ("sortOrder", "desc"),
            ("pageSize", "50"),
        ],
    )?;

    let body: SearchResponse =
        http::json_with_retry(client.get(url), "parse CurseForge search response").await?;

    Ok(body
        .data
        .into_iter()
        .map(|item| PluginSummary {
            provider: ProviderKind::CurseForge,
            project_id: item.id.to_string(),
            slug: item.slug,
            name: item.name,
            description: item.summary.unwrap_or_default(),
            downloads: item.download_count.unwrap_or(0.0) as u64,
        })
        .collect())
}

pub async fn versions(project_id: &str) -> Result<Vec<PluginVersion>> {
    let client = client()?;
    let url = Url::parse_with_params(
        &format!("{BASE_URL}/mods/{project_id}/files"),
        &[("pageSize", "50")],
    )?;

    let body: FilesResponse =
        http::json_with_retry(client.get(url), "parse CurseForge files response").await?;

    Ok(body
        .data
        .into_iter()
        .filter_map(|file| {
            let download_url = file.download_url?;
            let filename = file.file_name;
            Some(PluginVersion {
                id: file.id.to_string(),
                name: file.display_name.unwrap_or_else(|| filename.clone()),
                version_number: file.id.to_string(),
                game_versions: file.game_versions,
                loaders: vec!["bukkit".into(), "spigot".into(), "paper".into()],
                files: vec![PluginFile {
                    filename,
                    url: download_url,
                    primary: true,
                    size: file.file_length,
                }],
            })
        })
        .collect())
}

fn client() -> Result<Client> {
    let key = std::env::var("CURSEFORGE_API_KEY")
        .context("CurseForge requires CURSEFORGE_API_KEY in the environment")?;

    reqwest::Client::builder()
        .user_agent("mqverick/krate/0.1.0")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(60))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "x-api-key",
                key.parse()
                    .map_err(|_| anyhow::anyhow!("invalid CURSEFORGE_API_KEY header value"))?,
            );
            headers
        })
        .build()
        .context("build CurseForge client")
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    data: Vec<SearchMod>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchMod {
    id: u64,
    name: String,
    slug: String,
    summary: Option<String>,
    download_count: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct FilesResponse {
    data: Vec<FileData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileData {
    id: u64,
    display_name: Option<String>,
    file_name: String,
    download_url: Option<String>,
    game_versions: Vec<String>,
    file_length: Option<u64>,
}
