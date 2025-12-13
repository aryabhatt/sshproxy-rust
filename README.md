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
./target/release/sshproxy-rust --store-credentials -u <username>
```

This will prompt for:
- Your NERSC password
- Your OTP secret (base32 encoded, from your authenticator app setup)

Credentials are stored securely in macOS Keychain under service name `nersc-sshproxy`.

### Get SSH key and certificate

```bash
# Use default settings (scope: default, output: ~/.ssh/default)
./target/release/sshproxy-rust -u <username>

# Specify custom scope
./target/release/sshproxy-rust -u <username> -s perlmutter

# Specify output path
./target/release/sshproxy-rust -u <username> -o ~/.ssh/my-nersc-key

# Use collaboration account
./target/release/sshproxy-rust -u <username> -c <collab-account>

# Use alternate server URL (for testing)
./target/release/sshproxy-rust -u <username> -U https://test.sshproxy.nersc.gov
```

## Command-line Options

```
Options:
  -u, --user <USER>           NERSC username [default: current user]
  -o, --output <OUTPUT>       Output file path for private key
  -s, --scope <SCOPE>         Scope for the SSH key [default: default]
  -c, --collab <COLLAB>       Collaboration account (optional)
  -U, --url <URL>             URL for sshproxy server [default: https://sshproxy.nersc.gov]
      --store-credentials     Store credentials in keychain
  -h, --help                  Print help
```

## How it works

1. **Credential Retrieval**: Loads password and OTP secret from macOS Keychain
2. **TOTP Generation**: Generates current TOTP code using the stored secret
3. **API Request**: POSTs to `/create_pair/{scope}` with Basic Auth (username:password+OTP)
4. **Key Processing**: Extracts private key and certificate from response
5. **File Management**: 
   - Saves private key to specified path with 600 permissions
   - Extracts and saves certificate to `{path}-cert.pub`
   - Generates and saves public key to `{path}.pub`
6. **Validation**: Displays certificate validity period

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
- `ssh-keygen` available in PATH

## TODO / Future Enhancements

- [ ] Add ssh-agent integration with automatic expiration
- [ ] Support for other secure vaults (Linux Secret Service, Windows Credential Manager)
- [ ] PuTTY format output option
- [ ] SOCKS proxy support
- [ ] Configurable TOTP parameters (digits, interval)
- [ ] Credential rotation/update commands
- [ ] List stored credentials

## License

BSD-3-Clause (matching original sshproxy)
