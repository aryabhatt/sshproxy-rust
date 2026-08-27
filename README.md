# sshproxy-rust

[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://aryabhatt.github.io/sshproxy-rust/)

A stripped-down Rust implementation of NERSC's SSH Proxy client that securely stores credentials in system credential storage, Bitwarden, 1Password, or an age-encrypted local file, and automatically generates SSH certificates.

## Quick Start

### Installation

```bash
# From source
cargo build --release
cargo install --path .

# Or from GitHub
cargo install --git https://github.com/aryabhatt/sshproxy-rust --tag v2.1.0
```

### First-Time Setup

```bash
# Store your NERSC password
sshproxy-rust --update-password

# Store your NERSC TOTP secret
sshproxy-rust --update-secret
```

Credentials are stored securely in system credential storage:
- **macOS**: Keychain under services `NERSC` (for password) and `NERSC_SECRET` (for TOTP secret)
- **Linux**: Kernel keyring under the same service names
- **Windows**: Credential Manager under the same service names

You can also use Bitwarden or 1Password by creating `~/.config/sshproxy-rust/config.toml`:

```toml
credential_source = "bitwarden"

[bitwarden]
item = "NERSC"
```

```toml
credential_source = "1password"

[onepassword]
item = "NERSC"
vault = "Private"
```

Or store credentials in an age-encrypted local file, protected by a passphrase you enter interactively (never stored):

```toml
credential_source = "local"

[local_file]
# path = "/custom/path/credentials.age"   # optional, defaults to
                                            # ~/.config/sshproxy-rust/credentials.age
```

`--update-password` and `--update-secret` work for `system` and `local` sources (for `local`, you'll be prompted for a passphrase, and asked to confirm it the first time the file is created). They are ignored for Bitwarden/1Password — update those credentials directly in the password manager.

### Generate SSH Certificate

```bash
# Use default username ($USER)
sshproxy-rust

# Or specify username
sshproxy-rust yourusername
```

Certificates are saved to:
- Private key: `~/.ssh/nersc`
- Certificate: `~/.ssh/nersc-cert.pub`
- Public key: `~/.ssh/nersc.pub`

## Documentation

📚 **[Complete Documentation](docs/INDEX.md)** - Installation, configuration, troubleshooting, and more

🔧 **[API Documentation](https://aryabhatt.github.io/sshproxy-rust/sshproxy_rust/)** - Full API docs

## Acknowledgments

Documentation created with the help of Claude Sonnet 4.5 via https://chat.cborg.lbl.gov

## License

BSD-3-Clause (matching original sshproxy)
