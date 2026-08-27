use anyhow::{Context, Result};
use security_framework::passwords::{get_generic_password, set_generic_password};

use super::SERVICE_NAME;

/// NERSC passwords expire every year.
pub fn update_password(username: &str, password: &str) -> Result<()> {
    set_generic_password(SERVICE_NAME, username, password.as_bytes())
        .context("Failed to save password to keychain")?;
    Ok(())
}

/// usually totp secrets do not expire
pub fn update_secret(username: &str, otp_secret: &str) -> Result<()> {
    let service = format!("{}_SECRET", SERVICE_NAME);
    set_generic_password(&service, username, otp_secret.as_bytes())
        .context("Failed to save OTP secret to keychain")?;
    Ok(())
}

/// Retrieve password from macOS Keychain
pub fn get_password(username: &str) -> Result<String> {
    let password = get_generic_password(SERVICE_NAME, username)
        .context("Failed to retrieve password from keychain")?;
    Ok(String::from_utf8(password.to_vec())?)
}

/// Retrieve OTP secret from macOS Keychain
pub fn get_otp_secret(username: &str) -> Result<String> {
    let service = format!("{}_SECRET", SERVICE_NAME);
    let secret = get_generic_password(&service, username)
        .context("Failed to retrieve OTP secret from keychain")?;
    Ok(String::from_utf8(secret.to_vec())?)
}
