use anyhow::{Context, Result};
use serde::Deserialize;

pub(crate) const DEFAULT_SERVICE_NAME: &str = "NERSC";

#[derive(Debug, Default, Deserialize)]
pub struct SystemConfig {
    /// Keychain/keyring service name. Defaults to "NERSC".
    pub service_name: Option<String>,
}

impl SystemConfig {
    fn service_name(&self) -> &str {
        self.service_name.as_deref().unwrap_or(DEFAULT_SERVICE_NAME)
    }
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as imp;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as imp;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as imp;

pub fn update_password(username: &str, password: &str, config: &SystemConfig) -> Result<()> {
    imp::update_password(username, password, config.service_name())
}

pub fn update_secret(username: &str, otp_secret: &str, config: &SystemConfig) -> Result<()> {
    imp::update_secret(username, otp_secret, config.service_name())
}

pub fn get_password_otp(username: &str, config: &SystemConfig) -> Result<String> {
    let service_name = config.service_name();

    let password = imp::get_password(username, service_name)
        .context("Failed to get password. Run with --update-password first")?;

    let otp_secret = imp::get_otp_secret(username, service_name)
        .context("Failed to get OTP secret. Run with --update-secret first")?;

    let totp_code = super::generate_totp(&otp_secret)?;

    Ok(format!("{}{}", password, totp_code))
}
