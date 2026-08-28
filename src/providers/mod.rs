pub mod local_file;
pub mod system;

use anyhow::{Context, Result};

use crate::config::{Config, CredentialSource};

/// Generate TOTP code from a base32-encoded secret. Used by the System and LocalFile
/// sources, which both store the raw secret.
pub(crate) fn generate_totp(secret: &str) -> Result<String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    use totp_lite::{totp_custom, Sha1};

    let secret_bytes = data_encoding::BASE32_NOPAD
        .decode(secret.to_uppercase().as_bytes())
        .context("Failed to decode base32 OTP secret")?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let totp = totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp);

    Ok(format!("{:06}", totp))
}

/// Whether `--update-password` / `--update-secret` are meaningful for this source.
pub fn supports_direct_update(source: CredentialSource) -> bool {
    matches!(
        source,
        CredentialSource::System | CredentialSource::LocalFile
    )
}

pub fn get_password_otp(username: &str, config: &Config) -> Result<String> {
    match config.credential_source {
        CredentialSource::System => system::get_password_otp(username, &config.system),
        CredentialSource::LocalFile => local_file::get_password_otp(username, &config.local_file),
    }
}

pub fn update_password(username: &str, password: &str, config: &Config) -> Result<()> {
    match config.credential_source {
        CredentialSource::System => system::update_password(username, password, &config.system),
        CredentialSource::LocalFile => {
            local_file::update_password(username, password, &config.local_file)
        }
    }
}

pub fn update_secret(username: &str, otp_secret: &str, config: &Config) -> Result<()> {
    match config.credential_source {
        CredentialSource::System => system::update_secret(username, otp_secret, &config.system),
        CredentialSource::LocalFile => {
            local_file::update_secret(username, otp_secret, &config.local_file)
        }
    }
}
