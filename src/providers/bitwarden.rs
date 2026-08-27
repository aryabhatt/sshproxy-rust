use anyhow::{Context, Result};
use serde::Deserialize;

use super::run_secret_command;

#[derive(Debug, Deserialize)]
pub struct BitwardenConfig {
    pub item: Option<String>,
    #[serde(default = "default_bitwarden_command")]
    pub command: String,
}

impl Default for BitwardenConfig {
    fn default() -> Self {
        Self {
            item: None,
            command: default_bitwarden_command(),
        }
    }
}

fn default_bitwarden_command() -> String {
    "bw".to_string()
}

pub fn bitwarden_args(kind: &str, item: &str) -> Vec<String> {
    vec!["get".to_string(), kind.to_string(), item.to_string()]
}

pub fn get_password_otp(config: &BitwardenConfig) -> Result<String> {
    let item = config
        .item
        .as_deref()
        .context("Bitwarden config requires `bitwarden.item`")?;

    let password_args = bitwarden_args("password", item);
    let totp_args = bitwarden_args("totp", item);

    let password = run_secret_command(&config.command, &password_args, "Bitwarden password")?;
    let totp_code = run_secret_command(&config.command, &totp_args, "Bitwarden TOTP")?;

    Ok(format!("{}{}", password, totp_code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, CredentialSource};

    #[test]
    fn parses_bitwarden_config() {
        let config: Config = toml::from_str(
            r#"
            credential_source = "bitwarden"

            [bitwarden]
            item = "NERSC"
            "#,
        )
        .unwrap();

        assert_eq!(config.credential_source, CredentialSource::Bitwarden);
        assert_eq!(config.bitwarden.item.as_deref(), Some("NERSC"));
        assert_eq!(config.bitwarden.command, "bw");
    }

    #[test]
    fn bitwarden_command_args_are_not_shell_strings() {
        assert_eq!(
            bitwarden_args("password", "NERSC"),
            vec!["get", "password", "NERSC"]
        );
        assert_eq!(
            bitwarden_args("totp", "NERSC"),
            vec!["get", "totp", "NERSC"]
        );
    }
}
