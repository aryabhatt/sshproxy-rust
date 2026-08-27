pub mod bitwarden;
pub mod local_file;
pub mod onepassword;
pub mod system;

use anyhow::{Context, Result};
use std::process::Command;

use crate::config::{Config, CredentialSource};

/// Generate TOTP code from a base32-encoded secret. Used by the System and LocalFile
/// sources, which both store the raw secret; Bitwarden/1Password ask the CLI tool for
/// the current OTP code directly.
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

/// Run an external CLI tool, check its exit status, and return trimmed, non-empty stdout.
/// Used by the Bitwarden and 1Password sources to shell out to `bw`/`op`.
pub(crate) fn run_secret_command(
    command: &str,
    args: &[String],
    description: &str,
) -> Result<String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run {description} command `{command}`"))?;

    if !output.status.success() {
        anyhow::bail!(
            "{} command failed with status {}: {}",
            description,
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let value = String::from_utf8(output.stdout)
        .with_context(|| format!("{description} command returned invalid UTF-8"))?;
    let value = value.trim().to_string();
    if value.is_empty() {
        anyhow::bail!("{description} command returned empty output");
    }

    Ok(value)
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
        CredentialSource::System => system::get_password_otp(username),
        CredentialSource::Bitwarden => bitwarden::get_password_otp(&config.bitwarden),
        CredentialSource::OnePassword => onepassword::get_password_otp(&config.onepassword),
        CredentialSource::LocalFile => local_file::get_password_otp(username, &config.local_file),
    }
}

pub fn update_password(username: &str, password: &str, config: &Config) -> Result<()> {
    match config.credential_source {
        CredentialSource::System => system::update_password(username, password),
        CredentialSource::LocalFile => {
            local_file::update_password(username, password, &config.local_file)
        }
        CredentialSource::Bitwarden | CredentialSource::OnePassword => {
            anyhow::bail!("update_password is not supported for this credential_source")
        }
    }
}

pub fn update_secret(username: &str, otp_secret: &str, config: &Config) -> Result<()> {
    match config.credential_source {
        CredentialSource::System => system::update_secret(username, otp_secret),
        CredentialSource::LocalFile => {
            local_file::update_secret(username, otp_secret, &config.local_file)
        }
        CredentialSource::Bitwarden | CredentialSource::OnePassword => {
            anyhow::bail!("update_secret is not supported for this credential_source")
        }
    }
}
