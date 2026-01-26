use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::types::Config;

pub fn read_config() -> Result<Config> {
    let config_path = get_config_path()?;

    let content = fs::read_to_string(&config_path).context(format!(
        "Config file not found at {}. Create it with:\n  Line 1: Bitbucket base URL\n  Line 2: Access token\n  Line 3+: Excluded repository slugs (optional, one per line)",
        config_path.display()
    ))?;

    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 2 {
        anyhow::bail!(
            "Config file must have at least 2 lines:\n  Line 1: Bitbucket base URL\n  Line 2: Access token\n  Line 3+: Excluded repository slugs (optional)"
        );
    }

    let base_url = lines[0].trim().to_string();
    let token = lines[1].trim().to_string();

    if base_url.is_empty() || token.is_empty() {
        anyhow::bail!("Config file lines 1 and 2 cannot be empty");
    }

    // Parse excluded repos from line 3 onwards
    let excluded_repos: Vec<String> = lines
        .iter()
        .skip(2)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(Config {
        base_url,
        token,
        excluded_repos,
    })
}

fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join(".unsub-nojira"))
}
