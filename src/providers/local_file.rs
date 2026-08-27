use age::secrecy::{ExposeSecret, SecretString};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::crypto::age_store::{self, CredentialsBlob};

#[derive(Debug, Default, Deserialize)]
pub struct LocalFileConfig {
    /// Path to the age-encrypted credentials file. Defaults to
    /// ~/.config/sshproxy-rust/credentials.age.
    pub path: Option<PathBuf>,
}

pub fn resolve_path(config: &LocalFileConfig) -> Result<PathBuf> {
    match &config.path {
        Some(path) => Ok(path.clone()),
        None => default_local_file_path(),
    }
}

fn default_local_file_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    Ok(config_dir.join("sshproxy-rust").join("credentials.age"))
}

fn prompt_passphrase(prompt: &str) -> Result<SecretString> {
    println!("{prompt}");
    let raw = rpassword::read_password().context("Failed to read passphrase")?;
    Ok(SecretString::from(raw))
}

fn read_blob_with_passphrase(path: &PathBuf, passphrase: SecretString) -> Result<CredentialsBlob> {
    let ciphertext = fs::read(path).with_context(|| {
        format!(
            "Failed to read credentials file {}. Run with --update-password first",
            path.display()
        )
    })?;
    age_store::decrypt_blob(&ciphertext, passphrase)
}

fn write_blob_with_passphrase(
    path: &PathBuf,
    blob: &CredentialsBlob,
    passphrase: SecretString,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let ciphertext = age_store::encrypt_blob(blob, passphrase)?;
    fs::write(path, &ciphertext)
        .with_context(|| format!("Failed to write credentials file {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
    }

    Ok(())
}

/// Read + decrypt the existing file, prompting once for the current passphrase, and hand
/// back that same passphrase so callers can re-encrypt without prompting a second time.
fn read_for_update(path: &PathBuf) -> Result<(CredentialsBlob, SecretString)> {
    let passphrase = prompt_passphrase("Enter current passphrase for local credentials file: ")?;
    let passphrase_for_rewrite = SecretString::from(passphrase.expose_secret().to_string());
    let blob = read_blob_with_passphrase(path, passphrase)?;
    Ok((blob, passphrase_for_rewrite))
}

/// Prompt for and confirm a new passphrase, used only when creating the file for the first time.
fn prompt_new_passphrase() -> Result<SecretString> {
    let passphrase =
        prompt_passphrase("Enter a new passphrase to encrypt the local credentials file: ")?;
    let confirm = prompt_passphrase("Confirm passphrase: ")?;
    if passphrase.expose_secret() != confirm.expose_secret() {
        anyhow::bail!("Passphrases did not match");
    }
    Ok(passphrase)
}

fn merge_field(path: &PathBuf, apply: impl FnOnce(&mut CredentialsBlob)) -> Result<()> {
    let (mut blob, passphrase) = if path.exists() {
        read_for_update(path)?
    } else {
        (
            CredentialsBlob {
                password: String::new(),
                otp_secret: String::new(),
            },
            prompt_new_passphrase()?,
        )
    };
    apply(&mut blob);
    write_blob_with_passphrase(path, &blob, passphrase)
}

pub fn get_password_otp(_username: &str, config: &LocalFileConfig) -> Result<String> {
    let path = resolve_path(config)?;
    let passphrase = prompt_passphrase("Enter passphrase for local credentials file: ")?;
    let blob = read_blob_with_passphrase(&path, passphrase)?;
    let totp_code = super::generate_totp(&blob.otp_secret)?;
    Ok(format!("{}{}", blob.password, totp_code))
}

pub fn update_password(_username: &str, password: &str, config: &LocalFileConfig) -> Result<()> {
    let path = resolve_path(config)?;
    merge_field(&path, |blob| blob.password = password.to_string())
}

pub fn update_secret(_username: &str, otp_secret: &str, config: &LocalFileConfig) -> Result<()> {
    let path = resolve_path(config)?;
    merge_field(&path, |blob| blob.otp_secret = otp_secret.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_configured_path_override() {
        let config = LocalFileConfig {
            path: Some(PathBuf::from("/custom/path/creds.age")),
        };

        assert_eq!(
            resolve_path(&config).unwrap(),
            PathBuf::from("/custom/path/creds.age")
        );
    }

    #[test]
    fn resolves_default_path_when_unset() {
        let config = LocalFileConfig::default();

        let path = resolve_path(&config).unwrap();

        assert!(path.ends_with("sshproxy-rust/credentials.age"));
        assert!(path.is_absolute());
    }

    #[cfg(unix)]
    #[test]
    fn file_permissions_are_0600_after_write() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("credentials.age");
        let blob = CredentialsBlob {
            password: "hunter2".to_string(),
            otp_secret: "SECRET".to_string(),
        };

        write_blob_with_passphrase(&path, &blob, SecretString::from("passphrase".to_string()))
            .unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn write_then_read_roundtrips_via_passphrase_helpers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("credentials.age");
        let blob = CredentialsBlob {
            password: "hunter2".to_string(),
            otp_secret: "SECRET".to_string(),
        };

        write_blob_with_passphrase(&path, &blob, SecretString::from("passphrase".to_string()))
            .unwrap();
        let read_back =
            read_blob_with_passphrase(&path, SecretString::from("passphrase".to_string())).unwrap();

        assert_eq!(blob, read_back);
    }
}
