use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::providers::bitwarden::BitwardenConfig;
use crate::providers::local_file::LocalFileConfig;
use crate::providers::onepassword::OnePasswordConfig;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CredentialSource {
    #[default]
    System,
    Bitwarden,
    #[serde(rename = "1password")]
    OnePassword,
    #[serde(rename = "local")]
    LocalFile,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub credential_source: CredentialSource,
    #[serde(default)]
    pub bitwarden: BitwardenConfig,
    #[serde(default)]
    pub onepassword: OnePasswordConfig,
    #[serde(default)]
    pub local_file: LocalFileConfig,
}

pub fn default_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    Ok(config_dir.join("sshproxy-rust").join("config.toml"))
}

pub fn load_config(path: Option<&PathBuf>) -> Result<Config> {
    let path = match path {
        Some(path) => path.clone(),
        None => default_config_path()?,
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file {}", path.display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_config_defaults_to_system_source() {
        let config = Config::default();

        assert_eq!(config.credential_source, CredentialSource::System);
    }
}
