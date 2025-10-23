//! Integration tests for backup and restore operations.
//!
//! These tests verify the full workflow of creating encrypted backups,
//! verifying backup integrity, and restoring from backups.

use age::secrecy::SecretString;
use ponder::crypto::SessionManager;
use ponder::db::Database;
use ponder::ops;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Helper to create a test journal structure with encrypted entries.
fn setup_test_journal(journal_dir: &Path, db_path: &Path, passphrase: &SecretString) -> Database {
    // Create journal directory structure
    fs::create_dir_all(journal_dir).expect("create journal dir");

    // Initialize database
    let db = Database::open(db_path, passphrase).expect("open database");
    db.initialize_schema().expect("initialize schema");

    // Create some test entries
    let entries = vec![
        ("2025/01/15.md.age", "# 2025-01-15\n\nFirst test entry."),
        ("2025/01/16.md.age", "# 2025-01-16\n\nSecond test entry."),
        ("2025/01/17.md.age", "# 2025-01-17\n\nThird test entry."),
    ];

    for (rel_path, content) in entries {
        let entry_path = journal_dir.join(rel_path);
        fs::create_dir_all(entry_path.parent().unwrap()).expect("create entry dirs");

        // Write to temp file, then encrypt
        let temp_path = journal_dir.join(format!("{}.tmp", rel_path.replace('/', "_")));
        fs::write(&temp_path, content).expect("write temp");
        ponder::crypto::temp::encrypt_from_temp(&temp_path, &entry_path, passphrase)
            .expect("encrypt entry");
        // Note: encrypt_from_temp deletes the temp file automatically
    }

    db
}

#[test]
fn test_create_full_backup() {
    // Setup
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    let db_path = journal_dir.join("ponder.db");
    let passphrase = SecretString::new("test-passphrase".to_string());

    let db = setup_test_journal(&journal_dir, &db_path, &passphrase);
    let mut session = SessionManager::new(30);
    session.unlock(passphrase.clone());

    // Create backup
    let backup_path = temp_dir.path().join("backup.tar.gz.age");
    let report =
        ops::create_backup(&db, &mut session, &journal_dir, &backup_path).expect("create backup");

    // Verify report
    assert_eq!(report.total_entries, 3, "Should backup 3 entries");
    assert!(report.archive_size > 0, "Archive should have size");
    assert!(!report.checksum.is_empty(), "Should have checksum");
    assert!(report.duration.as_secs() < 10, "Should complete quickly");

    // Verify backup file exists
    assert!(backup_path.exists(), "Backup file should exist");

    // Verify backup file is encrypted (not plain tar.gz)
    let backup_bytes = fs::read(&backup_path).expect("read backup");
    assert!(
        !backup_bytes.starts_with(b"\x1f\x8b"),
        "Backup should be encrypted, not plain gzip"
    );
}

#[test]
fn test_verify_backup() {
    // Setup
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    let db_path = journal_dir.join("ponder.db");
    let passphrase = SecretString::new("test-passphrase".to_string());

    let db = setup_test_journal(&journal_dir, &db_path, &passphrase);
    let mut session = SessionManager::new(30);
    session.unlock(passphrase.clone());

    // Create backup
    let backup_path = temp_dir.path().join("backup.tar.gz.age");
    ops::create_backup(&db, &mut session, &journal_dir, &backup_path).expect("create backup");

    // Verify backup
    let manifest = ops::verify_backup(&mut session, &backup_path).expect("verify backup");

    // Check manifest
    assert_eq!(manifest.entries.len(), 3, "Should find 3 entries");
    assert_eq!(
        manifest.db_path,
        PathBuf::from("ponder.db"),
        "Should find database"
    );

    // Verify entry paths
    let entry_paths: Vec<String> = manifest
        .entries
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        entry_paths.contains(&"2025/01/15.md.age".to_string()),
        "Should contain first entry"
    );
    assert!(
        entry_paths.contains(&"2025/01/16.md.age".to_string()),
        "Should contain second entry"
    );
    assert!(
        entry_paths.contains(&"2025/01/17.md.age".to_string()),
        "Should contain third entry"
    );
}

#[test]
fn test_restore_backup() {
    // Setup
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    let db_path = journal_dir.join("ponder.db");
    let passphrase = SecretString::new("test-passphrase".to_string());

    let db = setup_test_journal(&journal_dir, &db_path, &passphrase);
    let mut session = SessionManager::new(30);
    session.unlock(passphrase.clone());

    // Create backup
    let backup_path = temp_dir.path().join("backup.tar.gz.age");
    let backup_report =
        ops::create_backup(&db, &mut session, &journal_dir, &backup_path).expect("create backup");

    // Delete original journal
    fs::remove_dir_all(&journal_dir).expect("delete journal");
    assert!(!journal_dir.exists(), "Journal should be deleted");

    // Restore from backup
    let restore_dir = temp_dir.path().join("restored");
    let restore_report = ops::restore_backup(&mut session, &backup_path, &restore_dir, false)
        .expect("restore backup");

    // Verify restore report
    assert_eq!(
        restore_report.entries_restored, 3,
        "Should restore 3 entries"
    );
    assert!(restore_report.db_size > 0, "Database should have size");
    assert_eq!(
        restore_report.checksum, backup_report.checksum,
        "Checksums should match"
    );

    // Verify restored files exist
    assert!(
        restore_dir.join("2025/01/15.md.age").exists(),
        "First entry should exist"
    );
    assert!(
        restore_dir.join("2025/01/16.md.age").exists(),
        "Second entry should exist"
    );
    assert!(
        restore_dir.join("2025/01/17.md.age").exists(),
        "Third entry should exist"
    );
    assert!(
        restore_dir.join("ponder.db").exists(),
        "Database should exist"
    );

    // Verify restored database can be opened
    let restored_db = Database::open(&restore_dir.join("ponder.db"), &passphrase)
        .expect("open restored database");
    drop(restored_db); // Ensure it's valid

    // Verify restored entry content matches original
    let restored_entry_path = restore_dir.join("2025/01/15.md.age");
    let temp_path = ponder::crypto::temp::decrypt_to_temp(&restored_entry_path, &passphrase)
        .expect("decrypt restored entry");
    let restored_content = fs::read_to_string(&temp_path).expect("read restored content");
    assert_eq!(
        restored_content, "# 2025-01-15\n\nFirst test entry.",
        "Content should match original"
    );
}

#[test]
fn test_backup_wrong_passphrase() {
    // Setup
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    let db_path = journal_dir.join("ponder.db");
    let passphrase1 = SecretString::new("correct-passphrase".to_string());
    let passphrase2 = SecretString::new("wrong-passphrase".to_string());

    let db = setup_test_journal(&journal_dir, &db_path, &passphrase1);
    let mut session1 = SessionManager::new(30);
    session1.unlock(passphrase1.clone());

    // Create backup with correct passphrase
    let backup_path = temp_dir.path().join("backup.tar.gz.age");
    ops::create_backup(&db, &mut session1, &journal_dir, &backup_path).expect("create backup");

    // Try to verify with wrong passphrase
    let mut session2 = SessionManager::new(30);
    session2.unlock(passphrase2);

    let result = ops::verify_backup(&mut session2, &backup_path);
    assert!(result.is_err(), "Verify should fail with wrong passphrase");

    // Verify error message mentions decryption
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    assert!(
        error_msg.contains("decrypt") || error_msg.contains("Crypto"),
        "Error should mention decryption issue: {}",
        error_msg
    );
}

#[test]
fn test_restore_force_overwrite() {
    // Setup
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    let db_path = journal_dir.join("ponder.db");
    let passphrase = SecretString::new("test-passphrase".to_string());

    let db = setup_test_journal(&journal_dir, &db_path, &passphrase);
    let mut session = SessionManager::new(30);
    session.unlock(passphrase.clone());

    // Create backup
    let backup_path = temp_dir.path().join("backup.tar.gz.age");
    ops::create_backup(&db, &mut session, &journal_dir, &backup_path).expect("create backup");

    // Create target directory with existing file
    let restore_dir = temp_dir.path().join("existing");
    fs::create_dir_all(&restore_dir).expect("create restore dir");
    fs::write(restore_dir.join("existing.txt"), "existing file").expect("write existing file");

    // Try restore without force (should fail)
    let result = ops::restore_backup(&mut session, &backup_path, &restore_dir, false);
    assert!(
        result.is_err(),
        "Restore should fail when target exists without force"
    );
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("exists") || error_msg.contains("force"),
        "Error should mention existing directory: {}",
        error_msg
    );

    // Restore with force (should succeed)
    let restore_report = ops::restore_backup(&mut session, &backup_path, &restore_dir, true)
        .expect("restore with force");
    assert_eq!(
        restore_report.entries_restored, 3,
        "Should restore 3 entries with force"
    );

    // Verify restored files exist
    assert!(
        restore_dir.join("2025/01/15.md.age").exists(),
        "Entry should exist after force restore"
    );
}

#[test]
fn test_backup_empty_journal() {
    // Setup
    let temp_dir = TempDir::new().expect("create temp dir");
    let journal_dir = temp_dir.path().join("journal");
    let db_path = journal_dir.join("ponder.db");
    let passphrase = SecretString::new("test-passphrase".to_string());

    // Create empty journal (just database, no entries)
    fs::create_dir_all(&journal_dir).expect("create journal dir");
    let db = Database::open(&db_path, &passphrase).expect("open database");
    db.initialize_schema().expect("initialize schema");

    let mut session = SessionManager::new(30);
    session.unlock(passphrase.clone());

    // Create backup of empty journal
    let backup_path = temp_dir.path().join("empty_backup.tar.gz.age");
    let report = ops::create_backup(&db, &mut session, &journal_dir, &backup_path)
        .expect("create empty backup");

    // Verify report
    assert_eq!(report.total_entries, 0, "Should backup 0 entries");
    assert!(
        report.archive_size > 0,
        "Archive should still have size (database)"
    );

    // Verify backup
    let manifest = ops::verify_backup(&mut session, &backup_path).expect("verify empty backup");
    assert_eq!(manifest.entries.len(), 0, "Should have no entries");
    assert_eq!(
        manifest.db_path,
        PathBuf::from("ponder.db"),
        "Should still have database"
    );
}
