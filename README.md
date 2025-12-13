# sshproxy-rust

A stripped-down Rust implementation of NERSC's SSH Proxy client that securely stores credentials in macOS Keychain and automatically generates TOTP codes.

## Features

- 🔐 **Secure credential storage** using macOS Keychain
- 🔑 **Automatic TOTP generation** from stored secret
- 🚀 **Async REST API** calls using reqwest
- 📦 **Minimal dependencies** and clean code
- ✅ **Proper file permissions** (600 for private keys)

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/sshproxy-rust`

## Usage

### First-time setup: Store credentials

```bash
./target/release/sshproxy-rust --update-password
```
This will prompt for your NERSC password

```bash
./target/release/sshproxy-rust --update-secret
```
This will prompt for your NERSC TOTP Secret


Credentials are stored securely in macOS Keychain under services `NERSC` (for password) and `NERSC_SECRET` (for TOTP secret)

### Get SSH key and certificate

```bash
# Use default settings (scope: default, output: ~/.ssh/nersc)
./target/release/sshproxy-rust

# Or specify username explicitly
./target/release/sshproxy-rust yourusername
```
## Command-line Options

```
Options:
  [USERNAME]                  NERSC username [default: USER environment variable]
  -p, --update-password       Update NERSC password in macOS Keychain
      --update-secret         Update NERSC TOTP secret in macOS Keychain
  -h, --help                  Print help
  -V, --version              Print version
```

## How it works

1. **Credential Retrieval**: Loads password and OTP secret from macOS Keychain for the current user
2. **TOTP Generation**: Generates current TOTP code (6-digit, 30-second interval) using SHA1 algorithm
3. **API Request**: POSTs to `https://sshproxy.nersc.gov/create_pair/default/` with HTTP Basic Auth (username:password+OTP)
4. **Key Processing**: Extracts private key and certificate from the combined response
5. **File Management**: 
   - Saves private key to `~/.ssh/nersc` with 600 permissions
   - Extracts and saves certificate to `~/.ssh/nersc-cert.pub`
   - Generates and saves public key to `~/.ssh/nersc.pub` using `ssh-keygen`
6. **Validation**: Displays certificate validity period using `ssh-keygen -L`

## Security Notes

- Credentials are stored in macOS Keychain (same security level as Safari passwords)
- Private keys are saved with 600 permissions (owner read/write only)
- TOTP codes are generated on-the-fly and never stored
- Password and OTP are combined and sent via HTTPS Basic Auth

## Comparison with Original Shell Script

**Original sshproxy.sh:**
- ✅ Interactive password+OTP prompt
- ✅ Multiple output formats (PuTTY support)
- ✅ ssh-agent integration
- ✅ SOCKS proxy support
- ❌ No credential storage
- ❌ Manual OTP entry each time

**This Rust implementation:**
- ✅ Secure credential storage
- ✅ Automatic TOTP generation
- ✅ Core functionality (key retrieval)
- ❌ No PuTTY format support
- ❌ No ssh-agent integration (yet)
- ❌ No SOCKS proxy support (yet)

## Dependencies

- `clap` - Command-line argument parsing
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `security-framework` - macOS Keychain access
- `totp-lite` - TOTP generation
- `data-encoding` - Base32 decoding
- `rpassword` - Secure password input
- `dirs` - Home directory detection

## Requirements

- macOS (for Keychain integration)
- Rust 1.70+ 
- `ssh-keygen` available in PATH (for generating public key and reading certificate info)

## License

BSD-3-Clause (matching original sshproxy)
