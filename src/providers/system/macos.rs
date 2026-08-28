use anyhow::{Context, Result};
use security_framework::passwords::{get_generic_password, set_generic_password};

/// NERSC passwords expire every year.
pub fn update_password(username: &str, password: &str, service_name: &str) -> Result<()> {
    set_generic_password(service_name, username, password.as_bytes())
        .context("Failed to save password to keychain")?;
    Ok(())
}

/// usually totp secrets do not expire
pub fn update_secret(username: &str, otp_secret: &str, service_name: &str) -> Result<()> {
    let service = format!("{}_SECRET", service_name);
    set_generic_password(&service, username, otp_secret.as_bytes())
        .context("Failed to save OTP secret to keychain")?;
    Ok(())
}

/// Retrieve password from macOS Keychain
pub fn get_password(username: &str, service_name: &str) -> Result<String> {
    let password = get_generic_password(service_name, username)
        .context("Failed to retrieve password from keychain")?;
    Ok(String::from_utf8(password.to_vec())?)
}

/// Retrieve OTP secret from macOS Keychain
pub fn get_otp_secret(username: &str, service_name: &str) -> Result<String> {
    let service = format!("{}_SECRET", service_name);
    let secret = get_generic_password(&service, username)
        .context("Failed to retrieve OTP secret from keychain")?;
    Ok(String::from_utf8(secret.to_vec())?)
}
