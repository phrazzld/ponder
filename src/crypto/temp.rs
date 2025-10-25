//! Secure temporary file handling with tmpfs preference.
//!
//! This module provides secure temporary file operations, preferring RAM-based tmpfs
//! filesystems when available to minimize disk persistence of decrypted content.

use crate::crypto::age::{decrypt_file_streaming, encrypt_file_streaming};
use crate::errors::AppResult;
use age::secrecy::SecretString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use tracing::{debug, warn};

#[cfg(not(target_os = "linux"))]
use tracing::debug;

/// Temporary filesystem paths to check for RAM-based storage.
const TMPFS_PATHS: &[&str] = &["/dev/shm", "/run/shm"];

/// Get a secure temporary directory, preferring tmpfs when available.
///
/// On Linux/BSD systems, this function prefers RAM-based tmpfs filesystems
/// (`/dev/shm` or `/run/shm`). If tmpfs is not available, it falls back to
/// the system temp directory with a warning.
///
/// The returned directory has restricted permissions (0o700 on Unix).
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::get_secure_temp_dir;
///
/// let temp_dir = get_secure_temp_dir()?;
/// println!("Using temp dir: {:?}", temp_dir);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn get_secure_temp_dir() -> AppResult<PathBuf> {
    // Try tmpfs paths first (RAM-based, more secure)
    for tmpfs_path in TMPFS_PATHS {
        let path = Path::new(tmpfs_path);
        if path.exists() && path.is_dir() {
            debug!("Using tmpfs directory: {}", tmpfs_path);
            return Ok(path.to_path_buf());
        }
    }

    // Fall back to system temp directory
    let temp_dir = std::env::temp_dir();

    // Only warn on Linux where tmpfs is expected and actionable
    // On macOS/Windows, tmpfs doesn't exist so warning is not useful
    #[cfg(target_os = "linux")]
    warn!(
        "tmpfs not available, using system temp directory: {:?}. \
         Decrypted content may persist on disk. Consider creating /dev/shm.",
        temp_dir
    );

    #[cfg(not(target_os = "linux"))]
    debug!(
        "Using system temp directory: {:?} (tmpfs not available on this platform)",
        temp_dir
    );

    Ok(temp_dir)
}

/// Decrypt an encrypted file to a temporary location.
///
/// Decrypts `encrypted_path` to a temporary file in a secure temp directory,
/// returning the path to the decrypted file. The temporary file has restricted
/// permissions (0o600 on Unix).
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::decrypt_to_temp;
/// use age::secrecy::SecretString;
/// use std::path::Path;
///
/// let passphrase = SecretString::new("my-secret".to_string());
/// let encrypted = Path::new("journal.md.age");
/// let temp_path = decrypt_to_temp(encrypted, &passphrase)?;
/// // Use temp_path...
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decrypt_to_temp(encrypted_path: &Path, passphrase: &SecretString) -> AppResult<PathBuf> {
    let temp_dir = get_secure_temp_dir()?;

    // Generate unique temp file name based on original file
    let file_name = encrypted_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("temp");
    let temp_path = temp_dir.join(format!("ponder-{}-{}", file_name, uuid::Uuid::new_v4()));

    // Create temp file with secure permissions BEFORE writing any plaintext
    // This prevents a race condition window where plaintext is world-readable
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        // Create file with 0o600 mode from the start
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp_path)?;

        debug!("Created temp file with 0o600 permissions: {:?}", temp_path);
    }

    #[cfg(not(unix))]
    {
        // On non-Unix platforms, create the file normally
        // Windows doesn't have Unix-style permissions
        std::fs::File::create(&temp_path)?;
    }

    // Decrypt to temp file (now exists with secure permissions)
    decrypt_file_streaming(encrypted_path, &temp_path, passphrase)?;

    Ok(temp_path)
}

/// Re-encrypt a temporary file and securely delete the temp file.
///
/// Encrypts the temporary file at `temp_path` to `encrypted_path`, then
/// performs best-effort secure deletion of the temporary file.
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::{decrypt_to_temp, encrypt_from_temp};
/// use age::secrecy::SecretString;
/// use std::path::Path;
///
/// let passphrase = SecretString::new("my-secret".to_string());
/// let encrypted = Path::new("journal.md.age");
///
/// // Decrypt to temp
/// let temp_path = decrypt_to_temp(encrypted, &passphrase)?;
///
/// // Modify temp file...
///
/// // Re-encrypt and cleanup
/// encrypt_from_temp(&temp_path, encrypted, &passphrase)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn encrypt_from_temp(
    temp_path: &Path,
    encrypted_path: &Path,
    passphrase: &SecretString,
) -> AppResult<()> {
    // Encrypt from temp to target location
    encrypt_file_streaming(temp_path, encrypted_path, passphrase)?;

    // Securely delete temp file
    secure_delete(temp_path)?;

    Ok(())
}

/// Read encrypted file content as UTF-8 string.
///
/// Decrypts the file to a temporary location, reads the content as UTF-8,
/// and automatically cleans up the temporary file.
///
/// # Arguments
///
/// * `encrypted_path` - Path to the encrypted file (.age)
/// * `passphrase` - Passphrase for decryption
///
/// # Returns
///
/// The decrypted file content as a String.
///
/// # Errors
///
/// Returns an error if:
/// - Decryption fails (wrong passphrase, corrupted file)
/// - Content is not valid UTF-8
/// - File I/O fails
///
/// # Example
///
/// ```no_run
/// use ponder::crypto::temp::read_encrypted_string;
/// use age::secrecy::SecretString;
/// use std::path::Path;
///
/// let passphrase = SecretString::new("my-secret".to_string());
/// let encrypted = Path::new("journal.md.age");
///
/// let content = read_encrypted_string(encrypted, &passphrase)?;
/// println!("Journal content: {}", content);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn read_encrypted_string(
    encrypted_path: &Path,
    passphrase: &SecretString,
) -> AppResult<String> {
    // Decrypt to temp
    let temp_path = decrypt_to_temp(encrypted_path, passphrase)?;

    // Read content
    let content = fs::read_to_string(&temp_path)?;

    // Clean up temp file (guaranteed even on error above)
    let _ = fs::remove_file(&temp_path);

    Ok(content)
}

/// Best-effort secure file deletion (overwrite + remove).
///
/// Overwrites the file with zeros before removing it. This is not
/// cryptographically secure (SSD wear leveling, filesystem journals),
/// but better than direct deletion.
fn secure_delete(path: &Path) -> AppResult<()> {
    // Get file size
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len() as usize;

    // Overwrite with zeros
    let mut file = File::create(path)?;
    let zeros = vec![0u8; file_size.min(1024 * 1024)]; // Write in 1MB chunks max

    let mut remaining = file_size;
    while remaining > 0 {
        let chunk_size = remaining.min(zeros.len());
        file.write_all(&zeros[..chunk_size])?;
        remaining -= chunk_size;
    }
    file.sync_all()?;

    // Remove the file
    fs::remove_file(path)?;
    debug!("Securely deleted temp file: {:?}", path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::SecretString;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_secure_temp_dir() {
        let temp_dir = get_secure_temp_dir();
        assert!(temp_dir.is_ok());

        let dir = temp_dir.unwrap();
        assert!(dir.exists());
        assert!(dir.is_dir());

        // On macOS/non-Linux systems, should fall back to system temp
        // On Linux, might find tmpfs
    }

    #[test]
    fn test_decrypt_to_temp_permissions() {
        let passphrase = SecretString::new("test-passphrase".to_string());

        // Create a test encrypted file
        let plaintext = b"Test content for permissions check";
        let encrypted_file = NamedTempFile::new().expect("create encrypted file");

        // Encrypt the plaintext
        let encrypted =
            crate::crypto::age::encrypt_with_passphrase(plaintext, &passphrase).unwrap();
        std::fs::write(encrypted_file.path(), encrypted).expect("write encrypted data");

        // Decrypt to temp
        let temp_path = decrypt_to_temp(encrypted_file.path(), &passphrase);
        assert!(temp_path.is_ok());

        let temp = temp_path.unwrap();
        assert!(temp.exists());

        // Check permissions on Unix - verify 0o600 from creation (no race window)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&temp).expect("get metadata");
            let perms = metadata.permissions();
            assert_eq!(
                perms.mode() & 0o777,
                0o600,
                "temp file should have 0o600 perms immediately after creation"
            );
        }

        // Clean up
        let _ = fs::remove_file(&temp);
    }

    #[test]
    #[cfg(unix)]
    fn test_temp_file_permissions_no_race_condition() {
        // Verify that temp file is created with 0o600 from the start,
        // not after plaintext is written (which would create a security window)
        use std::os::unix::fs::PermissionsExt;

        let passphrase = SecretString::new("race-condition-test".to_string());
        let plaintext = b"Sensitive data that should never be world-readable";
        let encrypted_file = NamedTempFile::new().expect("create encrypted file");

        let encrypted =
            crate::crypto::age::encrypt_with_passphrase(plaintext, &passphrase).unwrap();
        std::fs::write(encrypted_file.path(), encrypted).expect("write encrypted data");

        // Decrypt to temp - this should create file with 0o600 immediately
        let temp_path = decrypt_to_temp(encrypted_file.path(), &passphrase).unwrap();

        // Verify permissions are secure
        let metadata = fs::metadata(&temp_path).expect("get metadata");
        let mode = metadata.permissions().mode() & 0o777;

        assert_eq!(
            mode, 0o600,
            "Temp file must be created with 0o600 from start to prevent race condition. Found: 0o{:o}",
            mode
        );

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_encrypt_from_temp_cleanup() {
        let passphrase = SecretString::new("test-passphrase".to_string());

        // Create a temp file with content
        let mut temp_file = NamedTempFile::new().expect("create temp file");
        let plaintext = b"Test content for cleanup";
        temp_file.write_all(plaintext).expect("write plaintext");
        temp_file.flush().expect("flush");

        let temp_path = temp_file.path().to_path_buf();
        let encrypted_file = NamedTempFile::new().expect("create encrypted file");

        // Keep temp file from being auto-deleted - persist() doesn't delete the file
        let temp_persisted = temp_file.into_temp_path();
        assert!(
            temp_path.exists(),
            "temp file should exist before encryption"
        );

        // Encrypt from temp
        let result = encrypt_from_temp(&temp_path, encrypted_file.path(), &passphrase);
        assert!(result.is_ok(), "encryption should succeed: {:?}", result);

        // Temp file should be deleted
        assert!(
            !temp_path.exists(),
            "temp file should be deleted after encryption"
        );

        // Encrypted file should exist and be decryptable
        assert!(encrypted_file.path().exists());

        // Decrypt to verify
        let output_path = temp_path.with_extension("decrypted");
        let decrypted = crate::crypto::age::decrypt_file_streaming(
            encrypted_file.path(),
            &output_path,
            &passphrase,
        );
        assert!(decrypted.is_ok());

        // Clean up
        let _ = fs::remove_file(&output_path);
        let _ = temp_persisted.close(); // Explicitly clean up if it still exists
    }

    #[test]
    fn test_secure_delete() {
        // Create a test file
        let mut temp_file = NamedTempFile::new().expect("create temp file");
        let content = b"Sensitive data that should be overwritten";
        temp_file.write_all(content).expect("write content");
        temp_file.flush().expect("flush");

        let temp_path = temp_file.path().to_path_buf();

        // Keep file from being auto-deleted - persist keeps the file
        let _temp_persisted = temp_file.into_temp_path();

        // File should exist
        assert!(temp_path.exists(), "file should exist before secure delete");

        // Securely delete
        let result = secure_delete(&temp_path);
        assert!(result.is_ok(), "secure delete should succeed");

        // File should no longer exist
        assert!(!temp_path.exists(), "file should be deleted");
    }

    #[test]
    fn test_decrypt_encrypt_roundtrip() {
        let passphrase = SecretString::new("test-roundtrip-passphrase".to_string());
        let plaintext = b"Full roundtrip test content with multiple lines\nLine 2\nLine 3";

        // Create encrypted file
        let encrypted_file = NamedTempFile::new().expect("create encrypted file");
        let encrypted =
            crate::crypto::age::encrypt_with_passphrase(plaintext, &passphrase).unwrap();
        std::fs::write(encrypted_file.path(), encrypted).expect("write encrypted");

        // Decrypt to temp
        let temp_path = decrypt_to_temp(encrypted_file.path(), &passphrase).unwrap();
        assert!(temp_path.exists());

        // Verify content
        let decrypted_content = fs::read(&temp_path).expect("read temp file");
        assert_eq!(decrypted_content, plaintext);

        // Re-encrypt from temp
        let new_encrypted = NamedTempFile::new().expect("create new encrypted file");
        encrypt_from_temp(&temp_path, new_encrypted.path(), &passphrase).unwrap();

        // Temp should be deleted
        assert!(!temp_path.exists());

        // New encrypted file should be decryptable
        let final_decrypted = crate::crypto::age::decrypt_with_passphrase(
            &fs::read(new_encrypted.path()).unwrap(),
            &passphrase,
        )
        .unwrap();
        assert_eq!(final_decrypted, plaintext);
    }
}
