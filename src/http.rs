use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;

const USER_AGENT: &str = "mqverick/krate/0.1.0";

pub fn client() -> Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .build()
        .context("build HTTP client")
}

pub async fn json_with_retry<T>(request: RequestBuilder, context: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut last_error = None;

    for attempt in 0..3 {
        let Some(request) = request.try_clone() else {
            break;
        };

        match request.send().await {
            Ok(response) => match response.error_for_status() {
                Ok(response) => return response.json().await.with_context(|| context.to_string()),
                Err(error) => return Err(error).with_context(|| context.to_string()),
            },
            Err(error) => {
                last_error = Some(error);
                let delay_ms = 250 * (attempt + 1) * (attempt + 1);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    Err(last_error.context("request failed")?).with_context(|| context.to_string())
}
