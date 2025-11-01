//! Integration tests for migration system
//!
//! Tests the complete v1.0 → v2.0 migration workflow including detection,
//! migration, verification, and cleanup operations.

use age::secrecy::SecretString;
use chrono::NaiveDate;
use ponder::config::Config;
use ponder::crypto::SessionManager;
use ponder::db::Database;
use ponder::errors::AppResult;
use ponder::ops::detection::V1Entry;
use ponder::ops::{
    detect_migration_state, migrate_all_entries, migrate_entry, scan_v1_entries, verify_migration,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Create a test config with temporary directories
fn create_test_config(journal_dir: PathBuf, db_path: PathBuf) -> Config {
    Config {
        journal_dir,
        editor: "echo".to_string(),
        db_path,
        session_timeout_minutes: 60,
        ollama_url: "http://localhost:11434".to_string(),
        ai_models: Default::default(),
    }
}

/// Create a test passphrase as SecretString
fn test_passphrase() -> SecretString {
    "test-passphrase".to_string().into()
}

/// Create a v1.0 entry file with given date and content
fn create_v1_entry(journal_dir: &Path, date: &str, content: &str) -> AppResult<PathBuf> {
    let filename = format!("{}.md", date.replace("-", ""));
    let path = journal_dir.join(&filename);
    fs::write(&path, content)?;
    Ok(path)
}

/// Test detecting v1.0 entries in various scenarios
#[test]
fn test_detect_v1_entries() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    // Create some v1.0 entries
    create_v1_entry(&journal_dir, "2024-01-15", "Entry 1")?;
    create_v1_entry(&journal_dir, "2024-01-16", "Entry 2")?;
    create_v1_entry(&journal_dir, "2024-02-20", "Entry 3")?;

    // Create some non-matching files (should be ignored)
    fs::write(journal_dir.join("README.md"), "Not a journal entry")?;
    fs::write(journal_dir.join("invalid.txt"), "Not markdown")?;
    fs::write(journal_dir.join("99999999.md"), "Invalid date")?;

    // Create v2.0 structure (should be ignored)
    let v2_dir = journal_dir.join("2024/01");
    fs::create_dir_all(&v2_dir)?;
    fs::write(v2_dir.join("17.md.age"), b"encrypted")?;

    // Scan for v1.0 entries
    let v1_entries = scan_v1_entries(&journal_dir)?;

    // Should find exactly 3 valid v1.0 entries
    assert_eq!(v1_entries.len(), 3);

    let dates: Vec<_> = v1_entries.iter().map(|e| e.date).collect();
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 1, 16).unwrap()));
    assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 2, 20).unwrap()));

    // Test detection state with empty database
    let db = Database::open(&db_path, &test_passphrase())?;
    let detection = detect_migration_state(&journal_dir, &db)?;

    assert_eq!(detection.total_v1, 3);
    assert_eq!(detection.already_migrated, 0);
    assert_eq!(detection.pending, 3);

    Ok(())
}

/// Test migrating a single entry successfully
#[test]
fn test_migrate_single_entry() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    let config = create_test_config(journal_dir.clone(), db_path.clone());
    let db = Database::open(&db_path, &test_passphrase())?;
    let mut session = SessionManager::new(60);
    session.unlock(test_passphrase());

    // Create a v1.0 entry
    let v1_path = create_v1_entry(
        &journal_dir,
        "2024-01-15",
        "# Test Entry\n\nThis is a test.",
    )?;
    let v1_entry = V1Entry {
        path: v1_path.clone(),
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
    };

    // Migrate without AI client (skip embeddings)
    let result = migrate_entry(&config, &db, &mut session, None, &v1_entry);

    // Check result
    assert!(result.success, "Migration should succeed");
    assert!(result.checksum_match, "Checksum should match");
    assert_eq!(result.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert!(result.error_message.is_none());

    // Verify v2.0 file exists
    let v2_path = journal_dir.join("2024/01/15.md.age");
    assert!(v2_path.exists(), "v2.0 encrypted file should exist");

    // Verify original v1.0 file still exists
    assert!(v1_path.exists(), "Original v1.0 file should still exist");

    // Check database records
    let migration_status = db
        .get_migration_status("20240115.md")?
        .expect("Migration record should exist");
    assert_eq!(migration_status.status, "verified");
    assert!(migration_status.checksum_match);

    Ok(())
}

/// Test migration verification process
#[test]
fn test_migration_verification() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    let config = create_test_config(journal_dir.clone(), db_path.clone());
    let db = Database::open(&db_path, &test_passphrase())?;
    let mut session = SessionManager::new(60);
    session.unlock(test_passphrase());

    // Create a v1.0 entry with known content
    let content = "# Verification Test\n\nThis content will be verified.";
    let v1_path = create_v1_entry(&journal_dir, "2024-03-10", content)?;
    let v1_entry = V1Entry {
        path: v1_path.clone(),
        date: NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
    };

    // Migrate the entry
    let result = migrate_entry(&config, &db, &mut session, None, &v1_entry);
    assert!(result.success);

    // Verify the migration manually
    let v2_path = journal_dir.join("2024/03/10.md.age");
    let passphrase = test_passphrase();

    // Compute expected checksum
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(content.as_bytes());
    let expected_checksum = hasher.finalize();

    // Verify checksum matches
    let matches = verify_migration(&v2_path, &passphrase, &expected_checksum)?;
    assert!(matches, "Decrypted content should match original checksum");

    // Try with wrong passphrase - should fail
    let wrong_passphrase = "wrong-passphrase".to_string().into();
    let verify_result = verify_migration(&v2_path, &wrong_passphrase, &expected_checksum);
    assert!(
        verify_result.is_err(),
        "Verification with wrong passphrase should fail"
    );

    Ok(())
}

/// Test migrating multiple entries in batch
#[test]
fn test_migrate_all_entries() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    let config = create_test_config(journal_dir.clone(), db_path.clone());
    let db = Database::open(&db_path, &test_passphrase())?;
    let mut session = SessionManager::new(60);
    session.unlock(test_passphrase());

    // Create multiple v1.0 entries
    create_v1_entry(&journal_dir, "2024-01-01", "Entry 1")?;
    create_v1_entry(&journal_dir, "2024-01-02", "Entry 2")?;
    create_v1_entry(&journal_dir, "2024-01-03", "Entry 3")?;
    create_v1_entry(&journal_dir, "2024-02-15", "Entry 4")?;
    create_v1_entry(&journal_dir, "2024-03-20", "Entry 5")?;

    // Scan for entries
    let v1_entries = scan_v1_entries(&journal_dir)?;
    assert_eq!(v1_entries.len(), 5);

    // Migrate all entries (migrate_all_entries will initialize migration state)
    let results = migrate_all_entries(
        &config,
        &db,
        &mut session,
        None, // No AI client - skip embeddings
        v1_entries,
        None, // No progress callback for this test
    )?;

    // All should succeed
    assert_eq!(results.len(), 5);

    for result in &results {
        assert!(result.success, "All migrations should succeed");
        assert!(result.checksum_match, "All checksums should match");
    }

    // Verify v2.0 files exist
    assert!(journal_dir.join("2024/01/01.md.age").exists());
    assert!(journal_dir.join("2024/01/02.md.age").exists());
    assert!(journal_dir.join("2024/01/03.md.age").exists());
    assert!(journal_dir.join("2024/02/15.md.age").exists());
    assert!(journal_dir.join("2024/03/20.md.age").exists());

    // Check migration state
    let state = db
        .get_migration_state()?
        .expect("Should have migration state");
    assert_eq!(state.total_entries, 5);
    assert_eq!(state.migrated_count, 5);
    assert_eq!(state.verified_count, 5);
    assert_eq!(state.failed_count, 0);

    Ok(())
}

/// Test migration resume capability after partial failure
#[test]
fn test_migration_resume() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    let config = create_test_config(journal_dir.clone(), db_path.clone());
    let db = Database::open(&db_path, &test_passphrase())?;
    let mut session = SessionManager::new(60);
    session.unlock(test_passphrase());

    // Create v1.0 entries
    create_v1_entry(&journal_dir, "2024-01-01", "Entry 1")?;
    create_v1_entry(&journal_dir, "2024-01-02", "Entry 2")?;
    create_v1_entry(&journal_dir, "2024-01-03", "Entry 3")?;

    // Manually migrate first entry
    let v1_entry = V1Entry {
        path: journal_dir.join("20240101.md"),
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    };
    let result = migrate_entry(&config, &db, &mut session, None, &v1_entry);
    assert!(result.success);

    // Scan should now show 2 pending
    let detection = detect_migration_state(&journal_dir, &db)?;
    assert_eq!(detection.total_v1, 3);
    assert_eq!(detection.already_migrated, 1);
    assert_eq!(detection.pending, 2);

    // Get only pending entries for migration
    let all_v1 = scan_v1_entries(&journal_dir)?;
    let pending: Vec<_> = all_v1
        .into_iter()
        .filter(|e| {
            let filename = e.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            db.get_migration_status(filename)
                .ok()
                .flatten()
                .map(|r| r.status != "verified")
                .unwrap_or(true)
        })
        .collect();

    assert_eq!(pending.len(), 2);

    // Migrate remaining entries (migrate_all_entries will initialize migration state)
    let results = migrate_all_entries(&config, &db, &mut session, None, pending, None)?;

    assert_eq!(results.len(), 2);
    for result in &results {
        assert!(result.success);
    }

    // Now all should be migrated
    let final_detection = detect_migration_state(&journal_dir, &db)?;
    assert_eq!(final_detection.pending, 0);
    assert_eq!(final_detection.already_migrated, 3);

    Ok(())
}

/// Test handling of invalid v1.0 entries
#[test]
fn test_migration_error_handling() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    let config = create_test_config(journal_dir.clone(), db_path.clone());
    let db = Database::open(&db_path, &test_passphrase())?;
    let mut session = SessionManager::new(60);
    session.unlock(test_passphrase());

    // Create a v1.0 entry
    let v1_path = create_v1_entry(&journal_dir, "2024-01-15", "Valid entry")?;

    // Create a V1Entry pointing to non-existent file
    let invalid_entry = V1Entry {
        path: journal_dir.join("nonexistent.md"),
        date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
    };

    // Create a valid entry
    let valid_entry = V1Entry {
        path: v1_path,
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
    };

    // Migrate both (one will fail, one will succeed) - migrate_all_entries will initialize state
    let results = migrate_all_entries(
        &config,
        &db,
        &mut session,
        None,
        vec![invalid_entry, valid_entry],
        None,
    )?;

    assert_eq!(results.len(), 2);

    // First should fail
    assert!(!results[0].success);
    assert!(results[0].error_message.is_some());

    // Second should succeed
    assert!(results[1].success);
    assert!(results[1].checksum_match);

    // Check migration state reflects partial success
    let state = db.get_migration_state()?.expect("Should have state");
    assert_eq!(state.total_entries, 2);
    assert_eq!(state.failed_count, 1);
    assert_eq!(state.verified_count, 1);

    Ok(())
}

/// Test cleanup safety - only delete verified entries
#[test]
fn test_cleanup_v1_safety() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    let config = create_test_config(journal_dir.clone(), db_path.clone());
    let db = Database::open(&db_path, &test_passphrase())?;
    let mut session = SessionManager::new(60);
    session.unlock(test_passphrase());

    // Create entries
    let v1_path_migrated = create_v1_entry(&journal_dir, "2024-01-01", "Migrated")?;
    let v1_path_not_migrated = create_v1_entry(&journal_dir, "2024-01-02", "Not migrated")?;
    let v1_path_failed = create_v1_entry(&journal_dir, "2024-01-03", "Failed migration")?;

    // Migrate first entry successfully
    let entry1 = V1Entry {
        path: v1_path_migrated.clone(),
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    };
    let result = migrate_entry(&config, &db, &mut session, None, &entry1);
    assert!(result.success);

    // Manually record a failed migration for third entry
    db.record_migration("20240103.md", "2024/01/03.md.age", "2024-01-03", "failed")?;

    // Find entries safe to delete (only verified ones)
    let all_v1 = scan_v1_entries(&journal_dir)?;
    let safe_to_delete: Vec<_> = all_v1
        .iter()
        .filter(|e| {
            let filename = e.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            db.get_migration_status(filename)
                .ok()
                .flatten()
                .map(|r| r.status == "verified" || r.status == "migrated")
                .unwrap_or(false)
        })
        .collect();

    // Should only find one safe to delete (the verified one)
    assert_eq!(safe_to_delete.len(), 1);
    assert_eq!(
        safe_to_delete[0].date,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
    );

    // Delete only the verified entry
    for entry in safe_to_delete {
        fs::remove_file(&entry.path)?;
    }

    // Verify deletion
    assert!(
        !v1_path_migrated.exists(),
        "Migrated entry should be deleted"
    );
    assert!(
        v1_path_not_migrated.exists(),
        "Unmigrated entry should remain"
    );
    assert!(
        v1_path_failed.exists(),
        "Failed migration entry should remain"
    );

    Ok(())
}

/// Test that migration succeeds without AI client (skips embeddings)
#[test]
fn test_skip_embeddings_flag() -> AppResult<()> {
    let temp_dir = TempDir::new()?;
    let journal_dir = temp_dir.path().to_path_buf();
    let db_dir = TempDir::new()?;
    let db_path = db_dir.path().join("test.db");

    let config = create_test_config(journal_dir.clone(), db_path.clone());
    let db = Database::open(&db_path, &test_passphrase())?;
    let mut session = SessionManager::new(60);
    session.unlock(test_passphrase());

    // Create a v1.0 entry
    let v1_path = create_v1_entry(&journal_dir, "2024-01-15", "Test content for embeddings")?;
    let v1_entry = V1Entry {
        path: v1_path,
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
    };

    // Migrate without AI client (None means skip embeddings)
    let result = migrate_entry(&config, &db, &mut session, None, &v1_entry);

    // Migration should still succeed even without embeddings
    assert!(result.success, "Migration should succeed without AI client");
    assert!(result.checksum_match, "Checksum verification should pass");
    assert!(result.error_message.is_none(), "Should have no errors");

    // Verify v2.0 encrypted file exists
    let v2_path = journal_dir.join("2024/01/15.md.age");
    assert!(v2_path.exists(), "Encrypted v2.0 file should exist");

    // Verify migration was recorded
    let migration_status = db
        .get_migration_status("20240115.md")?
        .expect("Migration record should exist");
    assert_eq!(migration_status.status, "verified");

    Ok(())
}
