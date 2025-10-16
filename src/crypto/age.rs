//! Age encryption implementation for journal files.
//!
//! This module provides passphrase-based encryption using the age crate,
//! supporting both in-memory and streaming file encryption/decryption.

use crate::errors::{AppResult, CryptoError};
use age::secrecy::SecretString;
use std::io::{Read, Write};
use std::path::Path;

/// Encrypt data using age with passphrase.
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::encrypt_with_passphrase;
/// use age::secrecy::SecretString;
///
/// let passphrase = SecretString::new("my-secret-passphrase".to_string());
/// let plaintext = b"Secret data";
/// let encrypted = encrypt_with_passphrase(plaintext, &passphrase)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn encrypt_with_passphrase(plaintext: &[u8], passphrase: &SecretString) -> AppResult<Vec<u8>> {
    let encryptor = age::Encryptor::with_user_passphrase(passphrase.clone());
    let mut encrypted = Vec::new();
    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .map_err(CryptoError::EncryptionFailed)?;
    writer.write_all(plaintext)?;
    writer.finish()?;
    Ok(encrypted)
}

/// Decrypt age-encrypted data with passphrase.
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::{encrypt_with_passphrase, decrypt_with_passphrase};
/// use age::secrecy::SecretString;
///
/// let passphrase = SecretString::new("my-secret-passphrase".to_string());
/// let plaintext = b"Secret data";
/// let encrypted = encrypt_with_passphrase(plaintext, &passphrase)?;
/// let decrypted = decrypt_with_passphrase(&encrypted, &passphrase)?;
/// assert_eq!(plaintext, decrypted.as_slice());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decrypt_with_passphrase(ciphertext: &[u8], passphrase: &SecretString) -> AppResult<Vec<u8>> {
    let decryptor = match age::Decryptor::new(ciphertext).map_err(CryptoError::DecryptionFailed)? {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err(CryptoError::UnsupportedFormat.into()),
    };

    let mut decrypted = Vec::new();
    let mut reader = decryptor
        .decrypt(passphrase, None)
        .map_err(CryptoError::InvalidPassphrase)?;
    reader.read_to_end(&mut decrypted)?;
    Ok(decrypted)
}

/// Streaming encryption for large files.
///
/// Encrypts a file from `input_path` to `output_path` using streaming to minimize memory usage.
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::encrypt_file_streaming;
/// use age::secrecy::SecretString;
/// use std::path::Path;
///
/// let passphrase = SecretString::new("my-secret-passphrase".to_string());
/// let input = Path::new("journal.md");
/// let output = Path::new("journal.md.age");
/// encrypt_file_streaming(input, output, &passphrase)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn encrypt_file_streaming(
    input_path: &Path,
    output_path: &Path,
    passphrase: &SecretString,
) -> AppResult<()> {
    let mut input = std::fs::File::open(input_path)?;
    let output = std::fs::File::create(output_path)?;

    let encryptor = age::Encryptor::with_user_passphrase(passphrase.clone());
    let mut writer = encryptor
        .wrap_output(output)
        .map_err(CryptoError::EncryptionFailed)?;
    std::io::copy(&mut input, &mut writer)?;
    writer.finish()?;
    Ok(())
}

/// Streaming decryption for large files.
///
/// Decrypts a file from `input_path` to `output_path` using streaming to minimize memory usage.
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::decrypt_file_streaming;
/// use age::secrecy::SecretString;
/// use std::path::Path;
///
/// let passphrase = SecretString::new("my-secret-passphrase".to_string());
/// let input = Path::new("journal.md.age");
/// let output = Path::new("journal.md");
/// decrypt_file_streaming(input, output, &passphrase)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decrypt_file_streaming(
    input_path: &Path,
    output_path: &Path,
    passphrase: &SecretString,
) -> AppResult<()> {
    let input = std::fs::File::open(input_path)?;
    let mut output = std::fs::File::create(output_path)?;

    let decryptor = match age::Decryptor::new(input).map_err(CryptoError::DecryptionFailed)? {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err(CryptoError::UnsupportedFormat.into()),
    };

    let mut reader = decryptor
        .decrypt(passphrase, None)
        .map_err(CryptoError::InvalidPassphrase)?;
    std::io::copy(&mut reader, &mut output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::SecretString;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let passphrase = SecretString::new("test-passphrase-12345".to_string());
        let plaintext = b"This is a secret journal entry with multiple lines.\nAnd some special chars: !@#$%^&*()";

        // Encrypt
        let encrypted =
            encrypt_with_passphrase(plaintext, &passphrase).expect("encryption should succeed");

        // Encrypted data should be different from plaintext
        assert_ne!(encrypted.as_slice(), plaintext);

        // Decrypt
        let decrypted =
            decrypt_with_passphrase(&encrypted, &passphrase).expect("decryption should succeed");

        // Decrypted should match original
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_file_streaming_roundtrip() {
        let passphrase = SecretString::new("test-file-passphrase".to_string());
        let plaintext = b"File-based encryption test\nWith multiple lines\nAnd some content";

        // Create temporary files
        let input_file = NamedTempFile::new().expect("create input file");
        let encrypted_file = NamedTempFile::new().expect("create encrypted file");
        let output_file = NamedTempFile::new().expect("create output file");

        // Write plaintext to input file
        std::fs::write(input_file.path(), plaintext).expect("write plaintext");

        // Encrypt
        encrypt_file_streaming(input_file.path(), encrypted_file.path(), &passphrase)
            .expect("file encryption should succeed");

        // Decrypt
        decrypt_file_streaming(encrypted_file.path(), output_file.path(), &passphrase)
            .expect("file decryption should succeed");

        // Verify decrypted content matches original
        let decrypted = std::fs::read(output_file.path()).expect("read decrypted file");
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let correct_passphrase = SecretString::new("correct-passphrase".to_string());
        let wrong_passphrase = SecretString::new("wrong-passphrase".to_string());
        let plaintext = b"Secret data";

        // Encrypt with correct passphrase
        let encrypted = encrypt_with_passphrase(plaintext, &correct_passphrase)
            .expect("encryption should succeed");

        // Try to decrypt with wrong passphrase
        let result = decrypt_with_passphrase(&encrypted, &wrong_passphrase);

        // Should fail
        assert!(
            result.is_err(),
            "decryption with wrong passphrase should fail"
        );
    }
}
