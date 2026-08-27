use anyhow::{Context, Result};

pub(crate) const SERVICE_NAME: &str = "NERSC";

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

pub fn update_password(username: &str, password: &str) -> Result<()> {
    imp::update_password(username, password)
}

pub fn update_secret(username: &str, otp_secret: &str) -> Result<()> {
    imp::update_secret(username, otp_secret)
}

pub fn get_password_otp(username: &str) -> Result<String> {
    let password = imp::get_password(username)
        .context("Failed to get password. Run with --update-password first")?;

    let otp_secret = imp::get_otp_secret(username)
        .context("Failed to get OTP secret. Run with --update-secret first")?;

    let totp_code = super::generate_totp(&otp_secret)?;

    Ok(format!("{}{}", password, totp_code))
}
