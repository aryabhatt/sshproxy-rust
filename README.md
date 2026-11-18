# sshproxy-rust

[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://aryabhatt.github.io/sshproxy-rust/)

A stripped-down Rust implementation of NERSC's SSH Proxy client that securely stores credentials in system credential storage and automatically generates TOTP codes.

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

The TOTP secret is the base32 key shown when you set up NERSC MFA, not the 6-digit code from your authenticator app. You can paste it as shown: dashes, spaces, lowercase letters and trailing `=` are all accepted.

Credentials are stored securely in system credential storage:
- **macOS**: Keychain under services `NERSC` (for password) and `NERSC_SECRET` (for TOTP secret)
- **Linux**: Kernel keyring under the same service names
- **Windows**: Credential Manager under the same service names

### Generate SSH Certificate

```bash
# Use default username ($USER, or %USERNAME% on Windows)
sshproxy-rust

# Or specify username
sshproxy-rust yourusername
```

Certificates are saved to:
- Private key: `~/.ssh/nersc`
- Certificate: `~/.ssh/nersc-cert.pub`
- Public key: `~/.ssh/nersc.pub`

On Windows, `~` is your user profile folder (`%USERPROFILE%`), and `ssh-keygen` comes from the built-in Windows OpenSSH Client.

## Documentation

📚 **[Complete Documentation](docs/INDEX.md)** - Installation, configuration, troubleshooting, and more

🔧 **[API Documentation](https://aryabhatt.github.io/sshproxy-rust/sshproxy_rust/)** - Full API docs

## Acknowledgments

Documentation created with the help of Claude Sonnet 4.5 via https://chat.cborg.lbl.gov

## License

BSD-3-Clause (matching original sshproxy)
