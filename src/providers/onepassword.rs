use anyhow::{Context, Result};
use serde::Deserialize;

use super::run_secret_command;

#[derive(Debug, Deserialize)]
pub struct OnePasswordConfig {
    pub item: Option<String>,
    pub vault: Option<String>,
    pub account: Option<String>,
    #[serde(default = "default_onepassword_command")]
    pub command: String,
    #[serde(default = "default_onepassword_password_field")]
    pub password_field: String,
}

impl Default for OnePasswordConfig {
    fn default() -> Self {
        Self {
            item: None,
            vault: None,
            account: None,
            command: default_onepassword_command(),
            password_field: default_onepassword_password_field(),
        }
    }
}

fn default_onepassword_command() -> String {
    "op".to_string()
}

fn default_onepassword_password_field() -> String {
    "password".to_string()
}

pub fn onepassword_base_args(config: &OnePasswordConfig, item: &str) -> Vec<String> {
    let mut args = vec!["item".to_string(), "get".to_string(), item.to_string()];
    if let Some(vault) = &config.vault {
        args.push("--vault".to_string());
        args.push(vault.clone());
    }
    if let Some(account) = &config.account {
        args.push("--account".to_string());
        args.push(account.clone());
    }
    args
}

pub fn get_password_otp(config: &OnePasswordConfig) -> Result<String> {
    let item = config
        .item
        .as_deref()
        .context("1Password config requires `onepassword.item`")?;

    let mut password_args = onepassword_base_args(config, item);
    password_args.push("--fields".to_string());
    password_args.push(format!("label={}", config.password_field));
    password_args.push("--reveal".to_string());

    let mut otp_args = onepassword_base_args(config, item);
    otp_args.push("--otp".to_string());

    let password = run_secret_command(&config.command, &password_args, "1Password password")?;
    let totp_code = run_secret_command(&config.command, &otp_args, "1Password TOTP")?;

    Ok(format!("{}{}", password, totp_code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, CredentialSource};

    #[test]
    fn parses_onepassword_config_with_defaults() {
        let config: Config = toml::from_str(
            r#"
            credential_source = "1password"

            [onepassword]
            item = "NERSC"
            "#,
        )
        .unwrap();

        assert_eq!(config.credential_source, CredentialSource::OnePassword);
        assert_eq!(config.onepassword.item.as_deref(), Some("NERSC"));
        assert_eq!(config.onepassword.command, "op");
        assert_eq!(config.onepassword.password_field, "password");
    }

    #[test]
    fn onepassword_command_args_include_optional_scope() {
        let config = OnePasswordConfig {
            item: Some("NERSC".to_string()),
            vault: Some("Private".to_string()),
            account: Some("work".to_string()),
            command: "op".to_string(),
            password_field: "password".to_string(),
        };

        assert_eq!(
            onepassword_base_args(&config, "NERSC"),
            vec![
                "item",
                "get",
                "NERSC",
                "--vault",
                "Private",
                "--account",
                "work"
            ]
        );
    }
}
