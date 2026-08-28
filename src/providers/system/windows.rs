use anyhow::{Context, Result};
use keyring::Entry;

/// NERSC passwords expire every year.
pub fn update_password(username: &str, password: &str, service_name: &str) -> Result<()> {
    let entry = Entry::new(service_name, username).context("Failed to create keyring entry")?;
    entry
        .set_password(password)
        .context("Failed to save password to credential storage")?;
    Ok(())
}

/// usually totp secrets do not expire
pub fn update_secret(username: &str, otp_secret: &str, service_name: &str) -> Result<()> {
    let service = format!("{}_SECRET", service_name);
    let entry = Entry::new(&service, username).context("Failed to create keyring entry")?;
    entry
        .set_password(otp_secret)
        .context("Failed to save OTP secret to credential storage")?;
    Ok(())
}

/// Retrieve password from credential storage
pub fn get_password(username: &str, service_name: &str) -> Result<String> {
    let entry = Entry::new(service_name, username).context("Failed to create keyring entry")?;
    entry
        .get_password()
        .context("Failed to retrieve password from credential storage")
}

/// Retrieve OTP secret from credential storage
pub fn get_otp_secret(username: &str, service_name: &str) -> Result<String> {
    let service = format!("{}_SECRET", service_name);
    let entry = Entry::new(&service, username).context("Failed to create keyring entry")?;
    entry
        .get_password()
        .context("Failed to retrieve OTP secret from credential storage")
}
