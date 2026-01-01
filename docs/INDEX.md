# sshproxy-rust - Complete Documentation

[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://aryabhatt.github.io/sshproxy-rust/sshproxy_rust/)
[![Build Status](https://github.com/aryabhatt/sshproxy-rust/actions/workflows/docs.yml/badge.svg)](https://github.com/aryabhatt/sshproxy-rust/actions)

A stripped-down Rust implementation of NERSC's SSH Proxy client that securely stores credentials in system credential storage and automatically generates TOTP codes.

---

## Table of Contents

1. [Features](#features)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Configuration](#configuration)
5. [Usage](#usage)
6. [How It Works](#how-it-works)
7. [Security](#security)
8. [Troubleshooting](#troubleshooting)
9. [Development](#development)
10. [Comparison with Original](#comparison-with-original)
11. [Contributing](#contributing)
12. [License](#license)

---

## Features

- **Secure credential storage** using macOS Keychain or Linux kernel keyring
- **Automatic TOTP generation** from stored secret
- **Async REST API** calls using reqwest
- **Minimal dependencies** and clean code
- **Proper file permissions** (600 for private keys)
- **Cross-platform** support for macOS and Linux
- **Certificate validation** and automatic expiry display

---

## Installation

### Prerequisites

- **Operating System**: macOS or Linux
- **Rust**: Latest stable version recommended (minimum 1.70+)
- **SSH Tools**: `ssh-keygen` must be available in your PATH

### From Source

```bash
# Clone the repository
git clone https://github.com/aryabhatt/sshproxy-rust.git
cd sshproxy-rust

# Build and install
cargo build --release
cargo install --path .
```

The binary will be installed to `$HOME/.cargo/bin/sshproxy-rust`

### From GitHub Release

Install a specific version directly from GitHub:

```bash
# Install latest release
cargo install --git https://github.com/aryabhatt/sshproxy-rust

# Install specific version
cargo install --git https://github.com/aryabhatt/sshproxy-rust --tag v2.0.0
```

---

## Quick Start

### First-Time Setup

**Step 1: Store your NERSC password**

```bash
sshproxy-rust --update-password
```

You will be prompted to enter your NERSC password securely.

**Step 2: Store your NERSC TOTP secret**

```bash
sshproxy-rust --update-secret
```

You will be prompted to enter your TOTP secret. This not the 6-digit code from authenticator app. Visit [NERSC](https://docs.nersc.gov/connect/mfa/) is generate a MFA Secret.

**Note:** Both credentials must be set before you can generate SSH keys.

**Step 3: Generate SSH certificate**

```bash
sshproxy-rust
```

This will generate your SSH key pair and certificate.

### Example Output

```bash
$ sshproxy-rust
Generating TOTP code...
Requesting certificate from NERSC...
✓ Certificate saved to ~/.ssh/nersc-cert.pub
✓ Private key saved to ~/.ssh/nersc
✓ Public key saved to ~/.ssh/nersc.pub
```

---

## Configuration

### Default File Locations

The tool saves SSH keys and certificates to the following locations:

- **Private key**: `~/.ssh/nersc`
- **Certificate**: `~/.ssh/nersc-cert.pub`
- **Public key**: `~/.ssh/nersc.pub`

All keys are automatically set to `600` permissions (owner read/write only).

### Credential Storage

Credentials are stored securely in system-native credential storage:

- **macOS**: Keychain
  - Service: `NERSC` (password)
  - Service: `NERSC_SECRET` (TOTP secret)
  - Security level: Same as Safari passwords
  
- **Linux**: Kernel keyring
  - Session keyring: Persists until logout
  - For persistent storage across reboots, consider using `user` keyring or a password manager
  - Service names: `NERSC` and `NERSC_SECRET`

### Using with SSH

Add the following to your `~/.ssh/config` file:

```ssh-config
Host perlmutter
    User <your_nersc_username>
    HostName perlmutter-p1.nersc.gov
    IdentityFile ~/.ssh/nersc
```

Then connect with simple commands:

```bash
ssh perlmutter
```

---

## Usage

### Command-Line Options

```
sshproxy-rust [OPTIONS] [USERNAME]

Arguments:
  [USERNAME]                  NERSC username [default: $USER environment variable]

Options:
  -p, --update-password       Update NERSC password in credential storage
      --update-secret         Update NERSC TOTP secret in credential storage
  -h, --help                  Print help
  -V, --version              Print version
```

### Common Use Cases

#### Generate SSH certificate (default user)

```bash
sshproxy-rust
```

Uses the `$USER` environment variable as the NERSC username.

#### Generate SSH certificate (specific user)

```bash
sshproxy-rust yourusername
```

#### Update stored password

```bash
sshproxy-rust --update-password
```

#### Update stored TOTP secret

```bash
sshproxy-rust --update-secret
```

#### Check version

```bash
sshproxy-rust --version
```

---

## How It Works

The tool follows a six-step process to generate SSH credentials:

1. **Credential Retrieval**: Loads password and OTP secret from system credential storage for the current user

2. **TOTP Generation**: Generates current TOTP code (6-digit, 30-second interval) using SHA1 algorithm

3. **API Request**: POSTs to `https://sshproxy.nersc.gov/create_pair/default/` with HTTP Basic Auth (username:password+OTP)

4. **Key Processing**: Extracts private key and certificate from the combined response

5. **File Management**: 
   - Saves private key to `~/.ssh/nersc` with 600 permissions
   - Extracts and saves certificate to `~/.ssh/nersc-cert.pub`
   - Generates and saves public key to `~/.ssh/nersc.pub` using `ssh-keygen`

6. **Validation**: Displays certificate validity period (typically 24 hours) using `ssh-keygen -L`

### Certificate Lifecycle

- **Validity Period**: NERSC certificates are typically valid for **24 hours**
- **Renewal**: Simply re-run `sshproxy-rust` to generate a new certificate when the old one expires
- **Automatic Check**: The tool displays the validity period after generation

---

## Security

### Security Features

- ✅ **No plaintext storage**: Credentials stored in OS-native secure storage
- ✅ **TOTP on-the-fly**: TOTP codes generated dynamically, never stored
- ✅ **Secure file permissions**: Private keys automatically set to 600
- ✅ **HTTPS-only**: All API communication encrypted via TLS
- ✅ **No credential logging**: Passwords and secrets never logged

### Security Considerations

- **macOS Keychain**: Credentials protected by Keychain encryption, same security as Safari passwords
- **Linux Kernel Keyring**: Session-based storage, cleared on logout
- **HTTPS Basic Auth**: Password and OTP combined and sent via HTTPS Basic Authentication
- **Private Key Protection**: Files created with restrictive permissions from the start

### Best Practices

1. **Keep your TOTP secret secure**: Treat it like a password
2. **Rotate certificates regularly**: Generate new certificates every 24 hours
3. **Use SSH config**: Avoid typing credentials manually
4. **Monitor access**: Check NERSC logs for unauthorized access

---

## Troubleshooting

### "Failed to retrieve password from keyring"

**Cause**: Password not stored in system credential storage.

**Solution**: 
```bash
sshproxy-rust --update-password
```

### "Failed to retrieve TOTP secret from keyring"

**Cause**: TOTP secret not stored in system credential storage.

**Solution**: 
```bash
sshproxy-rust --update-secret
```

### "ssh-keygen not found"

**Cause**: OpenSSH tools not installed or not in PATH.

**Solution**: 
- **macOS**: OpenSSH is pre-installed. Check your PATH.
- **Linux**: Install OpenSSH client:
  ```bash
  # Debian/Ubuntu
  sudo apt-get install openssh-client
  
  # RHEL/CentOS/Fedora
  sudo dnf install openssh-clients
  ```

### "Permission denied" errors

**Cause**: Incorrect file permissions or missing `~/.ssh/` directory.

**Solution**: 
```bash
# Create .ssh directory if it doesn't exist
mkdir -p ~/.ssh
chmod 700 ~/.ssh

# Re-run sshproxy-rust (it will set correct permissions)
sshproxy-rust
```

### Certificate expired

**Cause**: NERSC certificates expire after 24 hours.

**Solution**: 
```bash
# Generate a new certificate
sshproxy-rust
```

### "Authentication failed"

**Cause**: Incorrect password or TOTP secret.

**Solution**: 
```bash
# Update password
sshproxy-rust --update-password

# Update TOTP secret
sshproxy-rust --update-secret
```

### macOS Keychain access denied

**Cause**: Application doesn't have Keychain access permission.

**Solution**: 
- Open **System Preferences** → **Security & Privacy** → **Privacy** → **Keychain**
- Ensure Terminal or your terminal emulator has access

---

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/aryabhatt/sshproxy-rust.git
cd sshproxy-rust

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Run directly
cargo run -- --help
```


### Code Documentation

```bash
# Build and open documentation
cargo doc --open

# Build documentation for all dependencies
cargo doc --open --document-private-items
```

---

## Comparison with Original

### Original sshproxy.sh

**Features:**
- Interactive password+OTP prompt
- Multiple output formats (PuTTY support)
- ssh-agent integration
- SOCKS proxy support
- No credential storage
- Manual OTP entry each time

**Pros:**
- More features
- Shell script (easy to read/modify)

**Cons:**
- No credential storage
- Manual OTP entry every time
- Less secure (credentials in memory/history)

### This Rust Implementation

**Features:**
- Secure credential storage
- Automatic TOTP generation
- Core functionality (key retrieval)
- Cross-platform binary
- Modern async architecture

**Pros:**
- Automated TOTP
- Secure credential storage
- Fast and lightweight
- Type-safe Rust implementation

**Cons:**
- No PuTTY format support (yet)
- No ssh-agent integration (yet)
- No SOCKS proxy support (yet)

**Future Enhancements:**
- [ ] ssh-agent integration
- [ ] PuTTY format export
- [ ] SOCKS proxy support
- [ ] Multiple scope support
- [ ] Custom output paths

---

## Dependencies

Core dependencies:

- **clap** - Command-line argument parsing
- **tokio** - Async runtime
- **reqwest** - HTTP client for API requests
- **security-framework** - macOS Keychain access (macOS only)
- **keyring** - Linux kernel keyring access (Linux only)
- **totp-lite** - TOTP code generation
- **data-encoding** - Base32 decoding for TOTP secrets
- **rpassword** - Secure password input (no echo)
- **dirs** - Cross-platform home directory detection

---

## Contributing

Contributions are welcome! Here's how you can help:

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Open a new issue with:
   - Description of the problem
   - Steps to reproduce
   - Expected vs actual behavior
   - System information (OS, Rust version)

### Proposing Features

1. Open an issue to discuss the feature
2. Explain the use case and benefits
3. Consider implementation complexity

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linters:
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Code Style

- Follow Rust standard conventions
- Run `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Add tests for new functionality
- Update documentation as needed

---

## License

**BSD-3-Clause License** (matching original sshproxy)

This project maintains the same license as the original NERSC sshproxy to ensure compatibility and proper attribution.

---

## Additional Resources

- **NERSC Documentation**: [https://docs.nersc.gov/connect/mfa/](https://docs.nersc.gov/connect/mfa/)
- **API Documentation**: [https://aryabhatt.github.io/sshproxy-rust/sshproxy_rust/](https://aryabhatt.github.io/sshproxy-rust/sshproxy_rust/)
- **Original sshproxy**: [https://github.com/NERSC/sshproxy](https://github.com/NERSC/sshproxy)

---

## FAQ

### Q: How often do I need to regenerate certificates?

**A:** NERSC certificates expire after 24 hours. You'll need to regenerate daily.

### Q: Can I use this on Windows?

**A:** Not currently. The tool supports macOS and Linux only. Windows support may be added in the future.

### Q: Where can I find my TOTP secret?

**A:** Your TOTP secret is provided when you first set up MFA at NERSC. Check your authenticator app settings or contact NERSC support.

### Q: Can I customize the output file location?

**A:** Not currently. The tool uses the standard `~/.ssh/nersc` location. Custom paths may be added in a future release.

### Q: Is this tool officially supported by NERSC?

**A:** No, this is an independent implementation. For official tools, see [NERSC's documentation](https://docs.nersc.gov/).

---

**Version**: 2.0.0  
**Last Updated**: 2024-01-15  
**Maintainer**: [@aryabhatt](https://github.com/aryabhatt)
