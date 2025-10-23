//! Integration tests for high-level operations (ops module).
//!
//! These tests verify the full workflow of editing, searching, and querying
//! journal entries with encryption, embeddings, and AI operations.

use age::secrecy::SecretString;
use ponder::config::Config;
use ponder::crypto::temp::read_encrypted_string;
use ponder::crypto::SessionManager;
use ponder::db::Database;
use ponder::ops::{detect_migration_state, migrate_all_entries, scan_v1_entries};
use std::fs;
use tempfile::TempDir;

/// Test that edit workflow correctly handles word count calculation.
///
/// This is a regression test for the bug where word count was calculated
/// from the encrypted file instead of the plaintext temp file, causing
/// "stream did not contain valid UTF-8" errors.
#[test]
#[ignore = "integration"]
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
#[ignore = "integration"]
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
#[ignore = "integration"]
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

/// Test complete v1.0 → v2.0 migration workflow integration.
///
/// This test verifies the end-to-end migration workflow:
/// 1. Create v1.0 plaintext entries (YYYYMMDD.md format)
/// 2. Detect migration state
/// 3. Migrate all entries to v2.0 (encrypted YYYY/MM/DD.md.age format)
/// 4. Verify v2.0 files are encrypted and decryptable
/// 5. Verify migration tracking in database
/// 6. Verify v1.0 files still exist (cleanup is separate operation)
#[test]
#[ignore = "integration"]
fn test_full_migration_workflow() {
    // Setup environment
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir).expect("create journal dir");

    let db_path = temp_dir.path().join("test.db");
    let passphrase = SecretString::new("migration-test-passphrase".to_string());

    // Create config
    let config = Config {
        journal_dir: journal_dir.clone(),
        editor: "echo".to_string(),
        db_path: db_path.clone(),
        session_timeout_minutes: 60,
        ollama_url: "http://localhost:11434".to_string(),
    };

    // Create database
    let db = Database::open(&db_path, &passphrase).expect("open database");

    // Create session manager
    let mut session = SessionManager::new(60);
    session.unlock(passphrase.clone());

    // Create v1.0 entries (plaintext in root directory)
    let v1_entries = vec![
        (
            "2024-01-15",
            "# Journal Entry\n\nFirst day of the new system.",
        ),
        (
            "2024-01-16",
            "# Progress Report\n\nMaking good progress on the migration.",
        ),
        (
            "2024-02-10",
            "# Valentine's Week\n\nPlanning something special.",
        ),
        (
            "2024-03-01",
            "# March Goals\n\n1. Complete migration\n2. Test thoroughly\n3. Deploy",
        ),
    ];

    let mut v1_paths = Vec::new();
    for (date, content) in &v1_entries {
        let filename = format!("{}.md", date.replace("-", ""));
        let path = journal_dir.join(&filename);
        fs::write(&path, content).expect("write v1 entry");
        v1_paths.push(path);
    }

    // Verify v1 files exist and are plaintext
    for path in &v1_paths {
        assert!(path.exists(), "v1 entry should exist");
        let content = fs::read_to_string(path).expect("read v1 as plaintext");
        assert!(
            content.contains("# "),
            "v1 entry should be readable plaintext"
        );
    }

    // Scan for v1.0 entries
    let scanned_v1 = scan_v1_entries(&journal_dir).expect("scan v1 entries");
    assert_eq!(
        scanned_v1.len(),
        4,
        "Should find all 4 v1.0 entries by filename pattern"
    );

    // Detect migration state (before migration)
    let detection_before =
        detect_migration_state(&journal_dir, &db).expect("detect migration state");
    assert_eq!(detection_before.total_v1, 4);
    assert_eq!(detection_before.already_migrated, 0);
    assert_eq!(detection_before.pending, 4);

    // Migrate all entries (without AI client - skip embeddings for speed)
    let results = migrate_all_entries(&config, &db, &mut session, None, scanned_v1, None)
        .expect("migrate all entries");

    assert_eq!(results.len(), 4, "Should have results for all entries");

    // Verify all migrations succeeded
    for result in &results {
        assert!(
            result.success,
            "Migration should succeed for date: {}",
            result.date
        );
        assert!(
            result.checksum_match,
            "Checksum should match for date: {}",
            result.date
        );
        assert!(
            result.error_message.is_none(),
            "Should have no errors for date: {}",
            result.date
        );
    }

    // Verify v2.0 files exist and are encrypted
    let v2_paths = [
        journal_dir.join("2024/01/15.md.age"),
        journal_dir.join("2024/01/16.md.age"),
        journal_dir.join("2024/02/10.md.age"),
        journal_dir.join("2024/03/01.md.age"),
    ];

    for (v2_path, (_, expected_content)) in v2_paths.iter().zip(&v1_entries) {
        assert!(v2_path.exists(), "v2 entry should exist");

        // Verify file is encrypted (NOT valid UTF-8)
        let encrypted_bytes = fs::read(v2_path).expect("read v2 file");
        assert!(
            std::str::from_utf8(&encrypted_bytes).is_err(),
            "v2 entry should be encrypted (not valid UTF-8)"
        );

        // Verify we can decrypt it
        let decrypted = read_encrypted_string(v2_path, &passphrase).expect("decrypt v2 entry");
        assert_eq!(
            &decrypted, expected_content,
            "Decrypted content should match original"
        );
    }

    // Verify v1.0 files still exist (migration doesn't delete them)
    for path in &v1_paths {
        assert!(path.exists(), "v1 entry should still exist after migration");
    }

    // Detect migration state (after migration)
    let v1_entries_after = scan_v1_entries(&journal_dir).expect("scan v1 entries again");
    let detection_after =
        detect_migration_state(&journal_dir, &db).expect("detect after migration");
    assert_eq!(detection_after.total_v1, 4);
    assert_eq!(detection_after.already_migrated, 4);
    assert_eq!(detection_after.pending, 0);

    // Verify migration was tracked in database
    for v1_entry in v1_entries_after {
        let filename = v1_entry
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("get filename");
        let migration_status = db
            .get_migration_status(filename)
            .expect("get migration status")
            .expect("migration record should exist");

        assert!(
            migration_status.status == "verified" || migration_status.status == "migrated",
            "Migration status should be verified or migrated"
        );
        assert!(
            migration_status.checksum_match,
            "Checksum should match in database"
        );
    }

    // Verify migration state in database
    let migration_state = db
        .get_migration_state()
        .expect("get migration state")
        .expect("migration state should exist");

    assert_eq!(migration_state.total_entries, 4);
    assert_eq!(migration_state.migrated_count, 4);
    assert_eq!(migration_state.verified_count, 4);
    assert_eq!(migration_state.failed_count, 0);
    assert!(
        migration_state.completed_at.is_some(),
        "Migration should be marked complete"
    );

    // Cleanup
    drop(session);
    drop(db);
    temp_dir.close().expect("cleanup temp dir");
}
