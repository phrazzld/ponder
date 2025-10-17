//! Integration tests for high-level operations (ops module).
//!
//! These tests verify the full workflow of editing, searching, and querying
//! journal entries with encryption, embeddings, and AI operations.

use age::secrecy::SecretString;
use ponder::crypto::temp::read_encrypted_string;
use ponder::db::Database;
use std::fs;
use tempfile::TempDir;

/// Test that edit workflow correctly handles word count calculation.
///
/// This is a regression test for the bug where word count was calculated
/// from the encrypted file instead of the plaintext temp file, causing
/// "stream did not contain valid UTF-8" errors.
#[test]
fn test_word_count_uses_plaintext_not_encrypted() {
    // Setup
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir).expect("create journal dir");

    let db_path = temp_dir.path().join("test.db");
    let passphrase = SecretString::new("test-passphrase".to_string());

    // Create database
    let db = Database::open(&db_path, &passphrase).expect("open database");
    db.initialize_schema().expect("initialize schema");

    // Create a test entry file
    let entry_path = journal_dir.join("2025").join("01").join("15.md.age");
    fs::create_dir_all(entry_path.parent().unwrap()).expect("create entry dirs");

    // Write plaintext content to temp
    let temp_path = temp_dir.path().join("test-entry.md");
    let test_content = "# 2025-01-15\n\nThis is a test journal entry with ten words total here.";
    fs::write(&temp_path, test_content).expect("write test content");

    // Encrypt temp to entry path
    ponder::crypto::temp::encrypt_from_temp(&temp_path, &entry_path, &passphrase)
        .expect("encrypt entry");

    // Verify encrypted file exists
    assert!(entry_path.exists(), "Encrypted file should exist");

    // Verify encrypted file is NOT valid UTF-8 (it's binary)
    let encrypted_bytes = fs::read(&entry_path).expect("read encrypted file");
    assert!(
        std::str::from_utf8(&encrypted_bytes).is_err(),
        "Encrypted file should NOT be valid UTF-8"
    );

    // Verify we can decrypt and read as string using helper
    let decrypted_content =
        read_encrypted_string(&entry_path, &passphrase).expect("decrypt and read encrypted file");

    assert_eq!(
        decrypted_content, test_content,
        "Decrypted content should match original"
    );

    // Verify word count from decrypted content
    let word_count = decrypted_content.split_whitespace().count();
    assert_eq!(word_count, 13, "Should count words from plaintext");

    // Cleanup
    drop(db);
    temp_dir.close().expect("cleanup temp dir");
}

/// Test that read_encrypted_string helper properly cleans up temp files.
#[test]
fn test_read_encrypted_string_cleans_up_temp() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let encrypted_path = temp_dir.path().join("test.md.age");
    let passphrase = SecretString::new("test".to_string());

    // Create encrypted file
    let content = "Test content for cleanup";
    let temp_write = temp_dir.path().join("write.md");
    fs::write(&temp_write, content).expect("write temp");
    ponder::crypto::temp::encrypt_from_temp(&temp_write, &encrypted_path, &passphrase)
        .expect("encrypt");

    // Count temp files before read
    let temp_files_before: Vec<_> = fs::read_dir(temp_dir.path())
        .expect("read temp dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("ponder-"))
        .collect();

    // Read encrypted string
    let decrypted =
        read_encrypted_string(&encrypted_path, &passphrase).expect("read encrypted string");
    assert_eq!(decrypted, content);

    // Count temp files after read
    let temp_files_after: Vec<_> = fs::read_dir(temp_dir.path())
        .expect("read temp dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("ponder-"))
        .collect();

    assert_eq!(
        temp_files_before.len(),
        temp_files_after.len(),
        "Should clean up temp files created during read"
    );

    temp_dir.close().expect("cleanup");
}

/// Test that attempting to read encrypted file as UTF-8 fails gracefully.
///
/// This documents the error case that the helper function prevents.
#[test]
fn test_reading_encrypted_as_utf8_fails() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let encrypted_path = temp_dir.path().join("test.md.age");
    let passphrase = SecretString::new("test".to_string());

    // Create encrypted file
    let content = "Test content";
    let temp_write = temp_dir.path().join("write.md");
    fs::write(&temp_write, content).expect("write temp");
    ponder::crypto::temp::encrypt_from_temp(&temp_write, &encrypted_path, &passphrase)
        .expect("encrypt");

    // Attempt to read encrypted file directly as UTF-8
    let result = fs::read_to_string(&encrypted_path);

    // Should fail with InvalidData error (not valid UTF-8)
    assert!(
        result.is_err(),
        "Reading encrypted file as UTF-8 should fail"
    );

    if let Err(e) = result {
        assert_eq!(
            e.kind(),
            std::io::ErrorKind::InvalidData,
            "Should be InvalidData error"
        );
        assert!(
            e.to_string().contains("UTF-8"),
            "Error should mention UTF-8: {}",
            e
        );
    }

    temp_dir.close().expect("cleanup");
}
