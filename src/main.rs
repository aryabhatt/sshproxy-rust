use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::path::PathBuf;

mod config;
mod crypto;
mod providers;
mod ssh_key;

use config::load_config;

#[derive(Parser)]
#[command(
    author = "Dinesh Kumar",
    about = "Retrieve NERSC SSH keys using system credential storage",
    long_about = None,
    version = "2.1.0"
    )]
struct Args {
    /// Username, if not provided, taken from USER env variable
    // #[clap(long, env = "USER")]
    username: Option<String>,

    /// Update NERSC password in system credential storage
    #[clap(short = 'p', long)]
    update_password: bool,

    /// Update NERSC TOTP secret in system credential storage
    #[clap(long)]
    update_secret: bool,

    /// Path to config file
    #[clap(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    let config = load_config(args.config.as_ref())?;

    // get username
    let username = args.username.unwrap_or_else(|| {
        env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| whoami::username())
    });

    let can_update = providers::supports_direct_update(config.credential_source);

    // check if we need to update password
    if args.update_password && can_update {
        println!("Enter new password for user {}: ", username);
        let password = rpassword::read_password().context("Failed to read password")?;
        providers::update_password(&username, &password, &config)?;
        println!("Password updated successfully.");
        return Ok(());
    }

    // check if we need to update otp secret
    if args.update_secret && can_update {
        println!("Enter TOTP secret for user {}: ", username);
        let otp_secret = rpassword::read_password().context("Failed to read OTP secret")?;
        providers::update_secret(&username, &otp_secret, &config)?;
        println!("OTP secret updated successfully.");
        return Ok(());
    }

    // Determine output path
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let key_path = home.join(".ssh").join("nersc");

    let password_otp = providers::get_password_otp(&username, &config)?;

    println!("Requesting SSH key for user: {}", username);

    // Request key from API
    let key_content = ssh_key::request_ssh_key(&username, &password_otp).await?;

    // Extract certificate
    let cert_content = ssh_key::extract_certificate(&key_content)?;

    // Save files
    ssh_key::save_key_files(&key_path, &key_content, &cert_content)?;

    println!("Successfully obtained ssh key: {}", key_path.display());

    // Show validity
    let cert_path = format!("{}-cert.pub", key_path.display());
    if let Ok(validity) = ssh_key::get_cert_validity(&cert_path) {
        println!("Key is {}", validity.to_lowercase());
    }

    Ok(())
}
