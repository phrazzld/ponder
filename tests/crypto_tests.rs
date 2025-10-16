//! Integration tests for cryptographic operations.
//!
//! These tests verify the complete encryption/decryption workflows across
//! the crypto module, including file operations, session management, and
//! temporary file handling.

use age::secrecy::SecretString;
use ponder::crypto::{
    decrypt_file_streaming, decrypt_to_temp, encrypt_file_streaming, encrypt_from_temp,
    encrypt_with_passphrase, get_secure_temp_dir, SessionManager,
};
use std::fs;
use std::io::Write;
use std::thread;
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn test_full_file_encryption_roundtrip() {
    let passphrase = SecretString::new("integration-test-passphrase".to_string());

    // Create test content
    let plaintext = b"Integration test content\nWith multiple lines\nAnd special chars: !@#$%^&*()";

    // Create temp directory for test files
    let temp_dir = tempdir().expect("create temp dir");
    let input_file = temp_dir.path().join("plaintext.md");
    let encrypted_file = temp_dir.path().join("encrypted.md.age");
    let output_file = temp_dir.path().join("decrypted.md");

    // Write plaintext
    fs::write(&input_file, plaintext).expect("write plaintext");

    // Encrypt
    encrypt_file_streaming(&input_file, &encrypted_file, &passphrase)
        .expect("encryption should succeed");

    // Verify encrypted file exists and is different from plaintext
    assert!(encrypted_file.exists());
    let encrypted_content = fs::read(&encrypted_file).expect("read encrypted file");
    assert_ne!(encrypted_content.as_slice(), plaintext);

    // Decrypt
    decrypt_file_streaming(&encrypted_file, &output_file, &passphrase)
        .expect("decryption should succeed");

    // Verify decrypted content matches original
    let decrypted_content = fs::read(&output_file).expect("read decrypted file");
    assert_eq!(decrypted_content.as_slice(), plaintext);
}

#[test]
fn test_streaming_large_file() {
    let passphrase = SecretString::new("large-file-test-passphrase".to_string());

    // Create a large file (>10MB)
    let large_content = vec![b'A'; 10 * 1024 * 1024 + 1]; // 10MB + 1 byte

    let temp_dir = tempdir().expect("create temp dir");
    let input_file = temp_dir.path().join("large.txt");
    let encrypted_file = temp_dir.path().join("large.txt.age");
    let output_file = temp_dir.path().join("large_decrypted.txt");

    // Write large file
    fs::write(&input_file, &large_content).expect("write large file");

    // Encrypt (streaming should handle this without loading entire file into memory)
    encrypt_file_streaming(&input_file, &encrypted_file, &passphrase)
        .expect("encryption should succeed");

    // Decrypt
    decrypt_file_streaming(&encrypted_file, &output_file, &passphrase)
        .expect("decryption should succeed");

    // Verify size matches
    let decrypted_content = fs::read(&output_file).expect("read decrypted file");
    assert_eq!(decrypted_content.len(), large_content.len());
    assert_eq!(decrypted_content, large_content);
}

#[test]
fn test_session_manager_timeout() {
    // Note: 0-minute timeout means 0 seconds, which makes elapsed >= timeout immediately true
    let mut session = SessionManager::new(0); // Will create Duration::from_secs(0 * 60) = 0 seconds

    // Actually, let's use a fraction of a minute for testing
    // We'll test by creating a session that times out quickly
    // Instead, let's just verify the timeout mechanism works with explicit timing

    let passphrase = SecretString::new("session-test-passphrase".to_string());

    // Initially locked
    assert!(session.is_locked());

    // For 0-minute timeout, we need to test the actual timeout behavior differently
    // Let's just verify lock/unlock behavior instead, since 0 timeout means instant expiry
    session.unlock(passphrase.clone());

    // With 0 timeout, session might expire immediately or very quickly
    // Let's wait and verify it expires
    thread::sleep(Duration::from_millis(50));

    // Should be locked after waiting (with 0 timeout, this happens very fast)
    assert!(session.is_locked());

    // Getting passphrase should fail
    let result = session.get_passphrase();
    assert!(result.is_err());

    // Can unlock again
    session.unlock(passphrase);

    // And then lock explicitly works
    session.lock();
    assert!(session.is_locked());
}

#[test]
fn test_session_manager_within_timeout() {
    // Use longer timeout
    let mut session = SessionManager::new(30);

    let passphrase = SecretString::new("session-test-passphrase".to_string());

    // Unlock
    session.unlock(passphrase);
    assert!(!session.is_locked());

    // Multiple accesses within timeout should succeed
    for _ in 0..10 {
        let result = session.get_passphrase();
        assert!(result.is_ok());
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn test_temp_file_cleanup_on_success() {
    let passphrase = SecretString::new("temp-cleanup-test".to_string());

    // Create encrypted file
    let plaintext = b"Temp file cleanup test";
    let encrypted_content = encrypt_with_passphrase(plaintext, &passphrase).unwrap();

    let mut encrypted_file = NamedTempFile::new().expect("create encrypted file");
    encrypted_file
        .write_all(&encrypted_content)
        .expect("write encrypted");
    encrypted_file.flush().expect("flush");

    // Decrypt to temp
    let temp_path = decrypt_to_temp(encrypted_file.path(), &passphrase).unwrap();
    assert!(temp_path.exists());

    // Re-encrypt from temp (should delete temp file)
    let new_encrypted = NamedTempFile::new().expect("create new encrypted file");
    encrypt_from_temp(&temp_path, new_encrypted.path(), &passphrase).unwrap();

    // Temp file should be deleted
    assert!(!temp_path.exists());
}

#[test]
fn test_temp_file_permissions() {
    let passphrase = SecretString::new("permissions-test".to_string());

    // Create encrypted file
    let plaintext = b"Permission test content";
    let encrypted_content = encrypt_with_passphrase(plaintext, &passphrase).unwrap();

    let mut encrypted_file = NamedTempFile::new().expect("create encrypted file");
    encrypted_file
        .write_all(&encrypted_content)
        .expect("write encrypted");
    encrypted_file.flush().expect("flush");

    // Decrypt to temp
    let temp_path = decrypt_to_temp(encrypted_file.path(), &passphrase).unwrap();
    assert!(temp_path.exists());

    // Check permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&temp_path).expect("get metadata");
        let perms = metadata.permissions();
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "temp file should have 0o600 permissions"
        );
    }

    // Clean up
    let _ = fs::remove_file(&temp_path);
}

#[test]
fn test_secure_temp_dir_exists() {
    let temp_dir = get_secure_temp_dir();
    assert!(temp_dir.is_ok());

    let dir = temp_dir.unwrap();
    assert!(dir.exists());
    assert!(dir.is_dir());
}

#[test]
fn test_decrypt_to_temp_unique_names() {
    let passphrase = SecretString::new("unique-names-test".to_string());

    // Create encrypted file
    let plaintext = b"Unique naming test";
    let encrypted_content = encrypt_with_passphrase(plaintext, &passphrase).unwrap();

    let mut encrypted_file = NamedTempFile::new().expect("create encrypted file");
    encrypted_file
        .write_all(&encrypted_content)
        .expect("write encrypted");
    encrypted_file.flush().expect("flush");

    // Decrypt to temp multiple times
    let temp_path1 = decrypt_to_temp(encrypted_file.path(), &passphrase).unwrap();
    let temp_path2 = decrypt_to_temp(encrypted_file.path(), &passphrase).unwrap();

    // Paths should be different (UUID-based)
    assert_ne!(temp_path1, temp_path2);

    // Both should exist
    assert!(temp_path1.exists());
    assert!(temp_path2.exists());

    // Clean up
    let _ = fs::remove_file(&temp_path1);
    let _ = fs::remove_file(&temp_path2);
}

#[test]
fn test_wrong_passphrase_fails() {
    let correct_passphrase = SecretString::new("correct-passphrase".to_string());
    let wrong_passphrase = SecretString::new("wrong-passphrase".to_string());

    // Create test content
    let plaintext = b"Secret content";

    let temp_dir = tempdir().expect("create temp dir");
    let input_file = temp_dir.path().join("plaintext.md");
    let encrypted_file = temp_dir.path().join("encrypted.md.age");
    let output_file = temp_dir.path().join("decrypted.md");

    // Write and encrypt with correct passphrase
    fs::write(&input_file, plaintext).expect("write plaintext");
    encrypt_file_streaming(&input_file, &encrypted_file, &correct_passphrase)
        .expect("encryption should succeed");

    // Try to decrypt with wrong passphrase
    let result = decrypt_file_streaming(&encrypted_file, &output_file, &wrong_passphrase);
    assert!(
        result.is_err(),
        "decryption with wrong passphrase should fail"
    );
}

#[test]
fn test_session_explicit_lock() {
    let mut session = SessionManager::new(30);
    let passphrase = SecretString::new("lock-test".to_string());

    // Unlock
    session.unlock(passphrase);
    assert!(!session.is_locked());

    // Can get passphrase
    assert!(session.get_passphrase().is_ok());

    // Explicitly lock
    session.lock();
    assert!(session.is_locked());

    // Cannot get passphrase anymore
    assert!(session.get_passphrase().is_err());
}

#[test]
fn test_multiple_session_unlock_calls() {
    let mut session = SessionManager::new(30);

    let passphrase1 = SecretString::new("first-passphrase".to_string());
    let passphrase2 = SecretString::new("second-passphrase".to_string());

    // First unlock
    session.unlock(passphrase1);
    assert!(!session.is_locked());

    // Second unlock with different passphrase (should replace)
    session.unlock(passphrase2);
    assert!(!session.is_locked());

    // Session should remain unlocked
    assert!(session.get_passphrase().is_ok());
}
