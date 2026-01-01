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
cargo install --git https://github.com/aryabhatt/sshproxy-rust --tag v2.0.0
```

### First-Time Setup

```bash
# Store your NERSC password
sshproxy-rust --update-password

# Store your NERSC TOTP secret
sshproxy-rust --update-secret
```

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
