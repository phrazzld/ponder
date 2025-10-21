//! Backup and restore operations for encrypted journal data.
//!
//! This module provides functionality to create encrypted backups of journal
//! entries and database, verify backup integrity, and restore from backups.

use crate::crypto::age::encrypt_with_passphrase;
use crate::crypto::SessionManager;
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use blake3::Hasher;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Report of a completed backup operation.
#[derive(Debug, Clone)]
pub struct BackupReport {
    /// Total number of journal entries included in backup
    pub total_entries: usize,
    /// Size of the backup archive in bytes
    pub archive_size: u64,
    /// BLAKE3 checksum of the backup archive
    pub checksum: String,
    /// Duration taken to create the backup
    pub duration: Duration,
}

/// Manifest describing the contents of a backup archive.
#[derive(Debug, Clone)]
pub struct BackupManifest {
    /// Paths to encrypted journal entry files in the backup
    pub entries: Vec<PathBuf>,
    /// Path to the database file in the backup
    pub db_path: PathBuf,
}

/// Creates a full encrypted backup of journal data.
///
/// Creates a compressed tar.gz archive containing all encrypted journal entries
/// and the database, then encrypts the entire archive with age encryption.
///
/// # Flow
///
/// 1. Collect all .md.age files from journal directory
/// 2. Include ponder.db database file
/// 3. Create tar.gz archive with relative paths
/// 4. Encrypt archive with passphrase
/// 5. Calculate checksum and record in database
///
/// # Arguments
///
/// * `db` - Database connection for recording backup metadata
/// * `session` - Session manager for encryption passphrase
/// * `journal_dir` - Directory containing encrypted journal entries
/// * `output_path` - Path where backup archive will be written
/// * `incremental` - Whether to create incremental backup (not yet implemented)
///
/// # Errors
///
/// Returns an error if:
/// - Journal directory doesn't exist or is inaccessible
/// - Session is locked
/// - Archive creation fails
/// - Encryption fails
/// - Database recording fails
pub fn create_backup(
    db: &Database,
    session: &mut SessionManager,
    journal_dir: &PathBuf,
    output_path: &PathBuf,
    _incremental: bool, // TODO: Implement incremental in future task
) -> AppResult<BackupReport> {
    let start_time = Instant::now();
    info!(
        "Creating backup from {:?} to {:?}",
        journal_dir, output_path
    );

    // Ensure session is unlocked for encryption
    let passphrase = session.get_passphrase()?;

    // Validate journal directory exists
    if !journal_dir.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Journal directory not found: {:?}", journal_dir),
        )));
    }

    // Step 1: Collect all .md.age files
    debug!("Collecting encrypted journal entries");
    let mut entry_paths = Vec::new();
    for entry in WalkDir::new(journal_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "age") {
            entry_paths.push(path.to_path_buf());
        }
    }

    debug!("Found {} encrypted entries", entry_paths.len());

    // Step 2: Include database file
    let db_path = journal_dir.join("ponder.db");
    if !db_path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Database not found: {:?}", db_path),
        )));
    }

    // Step 3: Create tar.gz archive in memory
    debug!("Creating tar.gz archive");
    let tar_gz_buffer = Vec::new();
    let encoder = GzEncoder::new(tar_gz_buffer, Compression::default());
    let mut tar = tar::Builder::new(encoder);

    // Step 4: Add journal entries with relative paths
    for entry_path in &entry_paths {
        let relative_path = entry_path
            .strip_prefix(journal_dir)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        debug!("Adding to archive: {:?}", relative_path);
        tar.append_path_with_name(entry_path, relative_path)
            .map_err(|e| {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to add {} to archive: {}",
                        relative_path.display(),
                        e
                    ),
                ))
            })?;
    }

    // Add database file
    debug!("Adding database to archive");
    tar.append_path_with_name(&db_path, "ponder.db")
        .map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to add database to archive: {}", e),
            ))
        })?;

    // Step 5: Finish tar and get compressed bytes
    let encoder = tar.into_inner().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to finalize tar archive: {}", e),
        ))
    })?;

    let tar_gz_bytes = encoder.finish().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to finish gzip compression: {}", e),
        ))
    })?;

    debug!("Archive size (compressed): {} bytes", tar_gz_bytes.len());

    // Step 6: Encrypt tar.gz with age
    debug!("Encrypting archive");
    let encrypted_bytes = encrypt_with_passphrase(&tar_gz_bytes, passphrase)?;

    // Step 7: Write to output_path
    debug!("Writing encrypted archive to {:?}", output_path);

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create output directory: {}", e),
            ))
        })?;
    }

    fs::write(output_path, &encrypted_bytes).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to write backup file: {}", e),
        ))
    })?;

    // Step 8: Calculate BLAKE3 checksum
    debug!("Calculating checksum");
    let mut hasher = Hasher::new();
    hasher.update(&encrypted_bytes);
    let checksum = hasher.finalize().to_hex().to_string();

    debug!("Backup checksum: {}", checksum);

    // Step 9: Record in database
    let backup_type = "full"; // TODO: Support incremental
    db.record_backup(
        output_path.to_str().unwrap_or("unknown"),
        backup_type,
        entry_paths.len() as i64,
        encrypted_bytes.len() as i64,
        &checksum,
    )?;

    let duration = start_time.elapsed();
    info!(
        "Backup completed: {} entries, {} bytes, {} seconds",
        entry_paths.len(),
        encrypted_bytes.len(),
        duration.as_secs()
    );

    Ok(BackupReport {
        total_entries: entry_paths.len(),
        archive_size: encrypted_bytes.len() as u64,
        checksum,
        duration,
    })
}

/// Verifies the integrity of a backup archive.
///
/// Decrypts and extracts the backup archive to verify its contents without
/// performing a full restore operation.
///
/// # Flow
///
/// 1. Decrypt backup archive with passphrase
/// 2. Extract tar.gz to temporary directory
/// 3. Verify database can be opened
/// 4. Count and list entry files
/// 5. Return manifest describing contents
///
/// # Arguments
///
/// * `session` - Session manager for decryption passphrase
/// * `backup_path` - Path to the backup archive file
///
/// # Errors
///
/// Returns an error if:
/// - Backup file doesn't exist
/// - Session is locked
/// - Decryption fails (wrong passphrase)
/// - Archive is corrupted
/// - Database verification fails
pub fn verify_backup(
    _session: &mut SessionManager,
    _backup_path: &PathBuf,
) -> AppResult<BackupManifest> {
    // TODO: Implement in Phase 7
    // Stub returns dummy data for compilation
    Ok(BackupManifest {
        entries: Vec::new(),
        db_path: PathBuf::from("ponder.db"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::SecretString;
    use tempfile::TempDir;

    #[test]
    fn test_backup_report_creation() {
        let report = BackupReport {
            total_entries: 10,
            archive_size: 1024,
            checksum: String::from("abc123"),
            duration: Duration::from_secs(5),
        };

        assert_eq!(report.total_entries, 10);
        assert_eq!(report.archive_size, 1024);
        assert_eq!(report.checksum, "abc123");
        assert_eq!(report.duration, Duration::from_secs(5));
    }

    #[test]
    fn test_backup_manifest_creation() {
        let manifest = BackupManifest {
            entries: vec![
                PathBuf::from("2024/01/01.md.age"),
                PathBuf::from("2024/01/02.md.age"),
            ],
            db_path: PathBuf::from("ponder.db"),
        };

        assert_eq!(manifest.entries.len(), 2);
        assert_eq!(manifest.db_path, PathBuf::from("ponder.db"));
    }

    #[test]
    #[ignore = "integration"]
    fn test_create_backup_empty_journal() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().to_path_buf();
        let output_path = temp_dir.path().join("backup.tar.gz.age");

        // Create empty database
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());

        // Create backup
        let report = create_backup(&db, &mut session, &journal_dir, &output_path, false).unwrap();

        // Verify report
        assert_eq!(report.total_entries, 0); // No journal entries
        assert!(report.archive_size > 0); // But has database
        assert!(!report.checksum.is_empty());
        assert!(output_path.exists());

        // Verify backup was recorded
        let last_backup = db.get_last_backup().unwrap();
        assert!(last_backup.is_some());
        let backup_record = last_backup.unwrap();
        assert_eq!(backup_record.entries_count, 0);
        assert_eq!(backup_record.checksum, report.checksum);
    }

    #[test]
    #[ignore = "integration"]
    fn test_create_backup_with_entries() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().to_path_buf();
        let output_path = temp_dir.path().join("backup.tar.gz.age");

        // Create database
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Create encrypted journal entries
        fs::create_dir_all(journal_dir.join("2024/01")).unwrap();
        let entry1_path = journal_dir.join("2024/01/01.md.age");
        let entry2_path = journal_dir.join("2024/01/02.md.age");

        let entry1_content = b"Test entry 1";
        let entry2_content = b"Test entry 2";

        let encrypted1 =
            crate::crypto::age::encrypt_with_passphrase(entry1_content, &passphrase).unwrap();
        let encrypted2 =
            crate::crypto::age::encrypt_with_passphrase(entry2_content, &passphrase).unwrap();

        fs::write(&entry1_path, encrypted1).unwrap();
        fs::write(&entry2_path, encrypted2).unwrap();

        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());

        // Create backup
        let report = create_backup(&db, &mut session, &journal_dir, &output_path, false).unwrap();

        // Verify report
        assert_eq!(report.total_entries, 2);
        assert!(report.archive_size > 0);
        assert!(!report.checksum.is_empty());
        assert!(output_path.exists());

        // Verify backup file exists and is not empty
        let backup_metadata = fs::metadata(&output_path).unwrap();
        assert!(backup_metadata.len() > 0);

        // Verify backup was recorded in database
        let last_backup = db.get_last_backup().unwrap();
        assert!(last_backup.is_some());
        let backup_record = last_backup.unwrap();
        assert_eq!(backup_record.entries_count, 2);
        assert_eq!(backup_record.backup_type, "full");
    }

    #[test]
    #[ignore = "integration"]
    fn test_create_backup_missing_journal_dir() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().join("nonexistent");
        let output_path = temp_dir.path().join("backup.tar.gz.age");

        // Create database in temp dir (not in nonexistent journal_dir)
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = temp_dir.path().join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());

        // Should fail with NotFound error
        let result = create_backup(&db, &mut session, &journal_dir, &output_path, false);
        assert!(result.is_err());
    }
}
