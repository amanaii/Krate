use anyhow::Result;

use crate::{config::Config, models::ProviderKind, tui};

pub async fn run() -> Result<()> {
    let current = Config::load().await?;
    let items: Vec<String> = ProviderKind::ALL.iter().map(ToString::to_string).collect();
    let initial = ProviderKind::ALL
        .iter()
        .position(|provider| provider == &current.provider);
    let mut ui = tui::Tui::new()?;
    let result = ui.select_one_sticky("Select provider server", &items, initial)?;
    let Some(index) = result.indices.first().copied() else {
        return Ok(());
    };

    let config = Config {
        provider: ProviderKind::ALL[index],
    };
    config.save().await?;
    Ok(())
}
