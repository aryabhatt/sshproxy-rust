use anyhow::{Context, Result};
use keyring::Entry;

use super::SERVICE_NAME;

/// NERSC passwords expire every year.
pub fn update_password(username: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, username).context("Failed to create keyring entry")?;
    entry
        .set_password(password)
        .context("Failed to save password to credential storage")?;
    Ok(())
}

/// usually totp secrets do not expire
pub fn update_secret(username: &str, otp_secret: &str) -> Result<()> {
    let service = format!("{}_SECRET", SERVICE_NAME);
    let entry = Entry::new(&service, username).context("Failed to create keyring entry")?;
    entry
        .set_password(otp_secret)
        .context("Failed to save OTP secret to credential storage")?;
    Ok(())
}

/// Retrieve password from credential storage
pub fn get_password(username: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, username).context("Failed to create keyring entry")?;
    entry
        .get_password()
        .context("Failed to retrieve password from credential storage")
}

/// Retrieve OTP secret from credential storage
pub fn get_otp_secret(username: &str) -> Result<String> {
    let service = format!("{}_SECRET", SERVICE_NAME);
    let entry = Entry::new(&service, username).context("Failed to create keyring entry")?;
    entry
        .get_password()
        .context("Failed to retrieve OTP secret from credential storage")
}
