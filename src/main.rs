use anyhow::{Context, Result};
//use bitwarden::secrets_manager;
use reqwest::Client;
use security_framework::passwords::get_generic_password;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{env, fs};

const SERVICE_NAME: &str = "NERSC";
const URL: &str = "https://sshproxy.nersc.gov";
const SCOPE: &str = "default";

/// Retrieve password from macOS Keychain
fn get_password(username: &str) -> Result<String> {
    let password = get_generic_password(SERVICE_NAME, username)
        .context("Failed to retrieve password from keychain")?;
    Ok(String::from_utf8(password.to_vec())?)
}

/// Retrieve OTP secret from macOS Keychain
fn get_otp_secret(username: &str) -> Result<String> {
    let service = format!("{}_SECRET", SERVICE_NAME);
    let secret = get_generic_password(&service, username)
        .context("Failed to retrieve OTP secret from keychain")?;
    Ok(String::from_utf8(secret.to_vec())?)
}

/// Generate TOTP code from secret
fn generate_totp(secret: &str) -> Result<String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    use totp_lite::{totp_custom, Sha1};

    // Decode base32 secret
    let secret_bytes = data_encoding::BASE32_NOPAD
        .decode(secret.to_uppercase().as_bytes())
        .context("Failed to decode base32 OTP secret")?;

    // Get current Unix timestamp
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // Generate TOTP (30 second interval, 6 digits)
    let totp = totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp);

    Ok(format!("{:06}", totp))
}

/// Request SSH key and certificate from sshproxy API
async fn request_ssh_key(username: &str, password_otp: &str) -> Result<String> {
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
fn extract_certificate(key_content: &str) -> Result<String> {
    for line in key_content.lines() {
        if line.contains("ssh-rsa") || line.contains("ssh-ed25519") {
            return Ok(line.to_string());
        }
    }
    anyhow::bail!("No certificate found in key file")
}

/// Save key files to disk with proper permissions
fn save_key_files(key_path: &PathBuf, key_content: &str, cert_content: &str) -> Result<()> {
    // Save private key
    fs::write(key_path, key_content).context("Failed to write private key")?;

    // Set permissions to 600
    let metadata = fs::metadata(key_path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(key_path, permissions)?;

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
fn get_cert_validity(cert_path: &str) -> Result<String> {
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

#[tokio::main]
async fn main() -> Result<()> {
    // Determine output path
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let key_path = home.join(".ssh").join("nersc");

    // get username from env
    let user = env::var("USER").context("Could not determine username from environment")?;

    // Retrieve credentials from keychain
    let password = get_password(&user)
        .context("Failed to get password. Run with --store-credentials first")?;

    let otp_secret = get_otp_secret(&user)
        .context("Failed to get OTP secret. Run with --store-credentials first")?;

    // Generate TOTP code
    let totp_code = generate_totp(&otp_secret)?;

    // Combine password and OTP
    let password_otp = format!("{}{}", password, totp_code);

    println!("Requesting SSH key for user: {}", user);

    // Request key from API
    let key_content = request_ssh_key(&user, &password_otp).await?;

    // Extract certificate
    let cert_content = extract_certificate(&key_content)?;

    // Save files
    save_key_files(&key_path, &key_content, &cert_content)?;

    println!("Successfully obtained ssh key: {}", key_path.display());

    // Show validity
    let cert_path = format!("{}-cert.pub", key_path.display());
    if let Ok(validity) = get_cert_validity(&cert_path) {
        println!("Key is {}", validity.to_lowercase());
    }

    Ok(())
}
