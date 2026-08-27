use age::secrecy::SecretString;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Plaintext payload stored inside the encrypted local credentials file.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CredentialsBlob {
    pub password: String,
    pub otp_secret: String,
}

/// Encrypt a `CredentialsBlob` with a passphrase, returning raw (binary) age ciphertext bytes.
pub fn encrypt_blob(blob: &CredentialsBlob, passphrase: SecretString) -> Result<Vec<u8>> {
    encrypt_blob_with_recipient(blob, age::scrypt::Recipient::new(passphrase))
}

fn encrypt_blob_with_recipient(
    blob: &CredentialsBlob,
    recipient: age::scrypt::Recipient,
) -> Result<Vec<u8>> {
    let plaintext = toml::to_string(blob).context("Failed to serialize credentials")?;
    age::encrypt(&recipient, plaintext.as_bytes()).context("Failed to encrypt credentials")
}

/// Decrypt raw age ciphertext bytes with a passphrase, returning the parsed `CredentialsBlob`.
/// Fails if the passphrase is wrong (age's AEAD check) or the data is corrupt.
pub fn decrypt_blob(ciphertext: &[u8], passphrase: SecretString) -> Result<CredentialsBlob> {
    let identity = age::scrypt::Identity::new(passphrase);
    let plaintext = age::decrypt(&identity, ciphertext)
        .context("Failed to decrypt credentials file (wrong passphrase or corrupt file?)")?;
    let plaintext =
        String::from_utf8(plaintext).context("Decrypted credentials are not valid UTF-8")?;
    toml::from_str(&plaintext).context("Failed to parse decrypted credentials")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Low work factor keeps tests fast; production encryption uses age's ~1s auto-calibrated default.
    const TEST_WORK_FACTOR: u8 = 10;

    fn encrypt_for_test(blob: &CredentialsBlob, passphrase: &str) -> Vec<u8> {
        let mut recipient = age::scrypt::Recipient::new(SecretString::from(passphrase.to_string()));
        recipient.set_work_factor(TEST_WORK_FACTOR);
        encrypt_blob_with_recipient(blob, recipient).unwrap()
    }

    fn sample_blob() -> CredentialsBlob {
        CredentialsBlob {
            password: "hunter2".to_string(),
            otp_secret: "JBSWY3DPEHPK3PXP".to_string(),
        }
    }

    #[test]
    fn roundtrip_encrypts_and_decrypts() {
        let blob = sample_blob();
        let ciphertext = encrypt_for_test(&blob, "correct horse battery staple");

        let decrypted = decrypt_blob(
            &ciphertext,
            SecretString::from("correct horse battery staple".to_string()),
        )
        .unwrap();

        assert_eq!(blob, decrypted);
    }

    #[test]
    fn wrong_passphrase_fails_to_decrypt() {
        let blob = sample_blob();
        let ciphertext = encrypt_for_test(&blob, "correct horse battery staple");

        let result = decrypt_blob(
            &ciphertext,
            SecretString::from("wrong passphrase".to_string()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn corrupt_ciphertext_fails_to_decrypt() {
        let blob = sample_blob();
        let mut ciphertext = encrypt_for_test(&blob, "correct horse battery staple");
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;

        let result = decrypt_blob(
            &ciphertext,
            SecretString::from("correct horse battery staple".to_string()),
        );

        assert!(result.is_err());
    }
}
