use anyhow::Result;
use reqwest::{Client, Url};
use serde::Deserialize;

use crate::{
    http,
    models::{PluginFile, PluginSummary, PluginVersion, ProviderKind},
};

const BASE_URL: &str = "https://api.modrinth.com/v2";

pub async fn search(query: &str) -> Result<Vec<PluginSummary>> {
    let client = client()?;
    let facets = r#"[["project_type:mod"],["server_side:required","server_side:optional"]]"#;
    let url = Url::parse_with_params(
        &format!("{BASE_URL}/search"),
        &[
            ("query", query),
            ("facets", facets),
            ("index", "downloads"),
            ("limit", "50"),
        ],
    )?;

    let body: SearchResponse =
        http::json_with_retry(client.get(url), "parse Modrinth search response").await?;

    Ok(body
        .hits
        .into_iter()
        .map(|hit| PluginSummary {
            provider: ProviderKind::Modrinth,
            project_id: hit.project_id,
            slug: hit.slug,
            name: hit.title,
            description: hit.description,
            downloads: hit.downloads,
        })
        .collect())
}

pub async fn versions(project_id: &str) -> Result<Vec<PluginVersion>> {
    let client = client()?;
    let url = format!("{BASE_URL}/project/{project_id}/version");
    let body: Vec<VersionResponse> =
        http::json_with_retry(client.get(url), "parse Modrinth versions response").await?;

    Ok(body
        .into_iter()
        .map(|version| PluginVersion {
            id: version.id,
            name: version.name,
            version_number: version.version_number,
            game_versions: version.game_versions,
            loaders: version.loaders,
            files: version
                .files
                .into_iter()
                .map(|file| PluginFile {
                    filename: file.filename,
                    url: file.url,
                    primary: file.primary,
                    size: file.size,
                })
                .collect(),
        })
        .collect())
}

fn client() -> Result<Client> {
    http::client()
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    downloads: u64,
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    id: String,
    name: String,
    version_number: String,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<FileResponse>,
}

#[derive(Debug, Deserialize)]
struct FileResponse {
    filename: String,
    url: String,
    primary: bool,
    size: Option<u64>,
}
