use anyhow::{Context, Result};
use reqwest::Client;
use std::path::PathBuf;

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const URL: &str = "https://sshproxy.nersc.gov";
const SCOPE: &str = "default";

/// Request SSH key and certificate from sshproxy API
pub async fn request_ssh_key(username: &str, password_otp: &str) -> Result<String> {
    let endpoint = format!("{}/create_pair/{}/", URL, SCOPE);

    let client = Client::builder()
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let request = client
        .post(&endpoint)
        .basic_auth(username, Some(password_otp));

    let response = request
        .send()
        .await
        .context("Failed to send request to sshproxy server")?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        anyhow::bail!("Server returned error: {} - {}", status, body);
    }

    // Check for authentication failure
    if body.contains("Authentication failed") {
        anyhow::bail!("Authentication failed. Check your password and OTP");
    }

    // Check for valid RSA private key
    if !body.contains("-----BEGIN RSA PRIVATE KEY-----")
        && !body.contains("-----BEGIN OPENSSH PRIVATE KEY-----")
    {
        anyhow::bail!(
            "Response does not contain a valid SSH private key:\n{}",
            body
        );
    }

    Ok(body)
}

/// Extract certificate from combined key file
pub fn extract_certificate(key_content: &str) -> Result<String> {
    for line in key_content.lines() {
        if line.contains("ssh-rsa") || line.contains("ssh-ed25519") {
            return Ok(line.to_string());
        }
    }
    anyhow::bail!("No certificate found in key file")
}

/// Save key files to disk with proper permissions
pub fn save_key_files(key_path: &PathBuf, key_content: &str, cert_content: &str) -> Result<()> {
    // Save private key
    fs::write(key_path, key_content).context("Failed to write private key")?;

    // Set permissions to 600 (Unix only)
    #[cfg(unix)]
    {
        let metadata = fs::metadata(key_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(key_path, permissions)?;
    }

    // Save certificate
    let cert_path = key_path
        .with_extension("")
        .with_extension("pub")
        .with_extension("");
    let cert_path = format!("{}-cert.pub", cert_path.display());
    fs::write(&cert_path, cert_content).context("Failed to write certificate")?;

    // Generate and save public key using ssh-keygen
    let output = std::process::Command::new("ssh-keygen")
        .arg("-y")
        .arg("-f")
        .arg(key_path)
        .output()
        .context("Failed to generate public key with ssh-keygen")?;

    if !output.status.success() {
        anyhow::bail!(
            "ssh-keygen failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let pub_path = key_path.with_extension("pub");
    fs::write(&pub_path, output.stdout).context("Failed to write public key")?;

    Ok(())
}

/// Get certificate validity information
pub fn get_cert_validity(cert_path: &str) -> Result<String> {
    let output = std::process::Command::new("ssh-keygen")
        .arg("-L")
        .arg("-f")
        .arg(cert_path)
        .output()
        .context("Failed to read certificate with ssh-keygen")?;

    if !output.status.success() {
        anyhow::bail!("ssh-keygen -L failed");
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.trim().starts_with("Valid:") {
            return Ok(line.trim().to_string());
        }
    }

    Ok("Valid: unknown".to_string())
}
