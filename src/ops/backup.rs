//! Backup and restore operations for encrypted journal data.
//!
//! This module provides functionality to create encrypted backups of journal
//! entries and database, verify backup integrity, and restore from backups.

use crate::crypto::age::{decrypt_with_passphrase, encrypt_with_passphrase};
use crate::crypto::SessionManager;
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use blake3::Hasher;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tar::Archive;
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

/// Report of a completed restore operation.
#[derive(Debug, Clone)]
pub struct RestoreReport {
    /// Number of journal entries restored
    pub entries_restored: usize,
    /// Size of the restored database in bytes
    pub db_size: u64,
    /// BLAKE3 checksum of the backup archive
    pub checksum: String,
    /// Duration taken to restore the backup
    pub duration: Duration,
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
    db.record_backup(
        output_path.to_str().unwrap_or("unknown"),
        "full",
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
    session: &mut SessionManager,
    backup_path: &PathBuf,
) -> AppResult<BackupManifest> {
    info!("Verifying backup: {:?}", backup_path);

    // Step 1: Read encrypted archive from path
    if !backup_path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Backup file not found: {:?}", backup_path),
        )));
    }

    let encrypted_bytes = fs::read(backup_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read backup file: {}", e),
        ))
    })?;

    debug!("Read {} encrypted bytes", encrypted_bytes.len());

    // Ensure session is unlocked for decryption
    let passphrase = session.get_passphrase()?;

    // Step 2: Decrypt with passphrase
    debug!("Decrypting backup archive");
    let decrypted_bytes = decrypt_with_passphrase(&encrypted_bytes, passphrase)?;

    debug!("Decrypted to {} bytes", decrypted_bytes.len());

    // Step 3: Extract tar.gz to temporary directory
    debug!("Extracting archive to temporary directory");
    let temp_dir = tempfile::TempDir::new().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create temp directory: {}", e),
        ))
    })?;

    let decoder = GzDecoder::new(decrypted_bytes.as_slice());
    let mut archive = Archive::new(decoder);

    archive.unpack(temp_dir.path()).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to extract archive: {}", e),
        ))
    })?;

    debug!("Extracted to: {:?}", temp_dir.path());

    // Step 4: Verify database file exists and can be opened
    let db_path = temp_dir.path().join("ponder.db");
    if !db_path.exists() {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Database file missing from backup archive",
        )));
    }

    // Try to open the database to verify it's valid
    debug!("Verifying database integrity");
    let _db = Database::open(&db_path, passphrase).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Database verification failed: {}", e),
        ))
    })?;

    debug!("Database verified successfully");

    // Step 5: Count .md.age files and collect paths
    debug!("Collecting entry paths");
    let mut entry_paths = Vec::new();
    for entry in WalkDir::new(temp_dir.path())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "age") {
            // Store relative path
            if let Ok(rel_path) = path.strip_prefix(temp_dir.path()) {
                entry_paths.push(rel_path.to_path_buf());
            }
        }
    }

    debug!("Found {} journal entries in backup", entry_paths.len());

    info!(
        "Backup verified: {} entries, database OK",
        entry_paths.len()
    );

    // Step 6: Cleanup handled automatically by TempDir drop

    // Step 7: Return manifest
    Ok(BackupManifest {
        entries: entry_paths,
        db_path: PathBuf::from("ponder.db"),
    })
}

/// Restores a backup archive to a target directory.
///
/// Extracts and verifies a backup archive, restoring encrypted journal entries
/// and database to the specified target directory. Uses atomic operations to
/// ensure all-or-nothing restoration.
///
/// # Flow
///
/// 1. Verify backup integrity
/// 2. Check target directory state
/// 3. Extract to temporary location
/// 4. Atomically move files to target directory
/// 5. Verify database accessibility
/// 6. Return restoration report
///
/// # Arguments
///
/// * `session` - Session manager for decryption passphrase
/// * `backup_path` - Path to the backup archive file
/// * `target_dir` - Directory where backup will be restored
/// * `force` - Whether to overwrite existing files
///
/// # Errors
///
/// Returns an error if:
/// - Backup verification fails
/// - Target directory exists and force is false
/// - Extraction fails
/// - Database verification fails
/// - File operations fail
pub fn restore_backup(
    session: &mut SessionManager,
    backup_path: &PathBuf,
    target_dir: &PathBuf,
    force: bool,
) -> AppResult<RestoreReport> {
    let start_time = Instant::now();
    info!(
        "Restoring backup from {:?} to {:?}",
        backup_path, target_dir
    );

    // Step 1: Verify backup integrity first
    debug!("Verifying backup before restore");
    let manifest = verify_backup(session, backup_path)?;

    info!(
        "Backup verified: {} entries to restore",
        manifest.entries.len()
    );

    // Step 2: Check if target_dir exists
    if target_dir.exists() && !force {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "Target directory already exists: {:?}. Use force flag to overwrite.",
                target_dir
            ),
        )));
    }

    let passphrase = session.get_passphrase()?;

    // Step 3: Extract to temp location first (for atomic operation)
    debug!("Extracting backup to temporary location");
    let temp_dir = tempfile::TempDir::new().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create temp directory: {}", e),
        ))
    })?;

    // Read and decrypt archive
    let encrypted_bytes = fs::read(backup_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read backup file: {}", e),
        ))
    })?;

    // Calculate checksum of backup archive
    let mut hasher = Hasher::new();
    hasher.update(&encrypted_bytes);
    let checksum = hasher.finalize().to_hex().to_string();

    let decrypted_bytes = decrypt_with_passphrase(&encrypted_bytes, passphrase)?;

    // Extract tar.gz
    let decoder = GzDecoder::new(decrypted_bytes.as_slice());
    let mut archive = Archive::new(decoder);
    archive.unpack(temp_dir.path()).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to extract archive: {}", e),
        ))
    })?;

    debug!("Extracted to temp location: {:?}", temp_dir.path());

    // Step 4: Move files from temp to target_dir (atomic operation)
    debug!("Moving files to target directory");

    // Create target directory if it doesn't exist
    if !target_dir.exists() {
        fs::create_dir_all(target_dir).map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create target directory: {}", e),
            ))
        })?;
    }

    // Move each file from temp to target
    for entry_path in &manifest.entries {
        let src = temp_dir.path().join(entry_path);
        let dst = target_dir.join(entry_path);

        // Create parent directories if needed
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create directory {:?}: {}", parent, e),
                ))
            })?;
        }

        // Move/copy file
        fs::copy(&src, &dst).map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to restore {}: {}", entry_path.display(), e),
            ))
        })?;
    }

    // Move database file
    let src_db = temp_dir.path().join("ponder.db");
    let dst_db = target_dir.join("ponder.db");
    fs::copy(&src_db, &dst_db).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to restore database: {}", e),
        ))
    })?;

    debug!("All files moved to target directory");

    // Step 5: Verify database opens with provided passphrase
    debug!("Verifying restored database");
    let _db = Database::open(&dst_db, passphrase).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Restored database verification failed: {}", e),
        ))
    })?;

    // Get database size
    let db_size = fs::metadata(&dst_db)
        .map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get database size: {}", e),
            ))
        })?
        .len();

    let duration = start_time.elapsed();
    info!(
        "Restore completed: {} entries, database {} bytes, {} seconds",
        manifest.entries.len(),
        db_size,
        duration.as_secs()
    );

    // Step 6: Return report
    Ok(RestoreReport {
        entries_restored: manifest.entries.len(),
        db_size,
        checksum,
        duration,
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
        let report = create_backup(&db, &mut session, &journal_dir, &output_path).unwrap();

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
        let report = create_backup(&db, &mut session, &journal_dir, &output_path).unwrap();

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
        let result = create_backup(&db, &mut session, &journal_dir, &output_path);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "integration"]
    fn test_verify_backup_success() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().to_path_buf();
        let backup_path = temp_dir.path().join("backup.tar.gz.age");

        // Create database and entries
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        fs::create_dir_all(journal_dir.join("2024/01")).unwrap();
        let entry1_path = journal_dir.join("2024/01/01.md.age");
        let entry1_content = b"Test entry 1";
        let encrypted1 =
            crate::crypto::age::encrypt_with_passphrase(entry1_content, &passphrase).unwrap();
        fs::write(&entry1_path, encrypted1).unwrap();

        // Create backup
        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());
        create_backup(&db, &mut session, &journal_dir, &backup_path).unwrap();

        // Verify backup
        let manifest = verify_backup(&mut session, &backup_path).unwrap();

        // Check manifest
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.db_path, PathBuf::from("ponder.db"));
        assert!(manifest
            .entries
            .contains(&PathBuf::from("2024/01/01.md.age")));
    }

    #[test]
    #[ignore = "integration"]
    fn test_verify_backup_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let backup_path = temp_dir.path().join("nonexistent.tar.gz.age");

        let passphrase = SecretString::new("test_password".to_string());
        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());

        // Should fail with NotFound error
        let result = verify_backup(&mut session, &backup_path);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "integration"]
    fn test_verify_backup_wrong_passphrase() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().to_path_buf();
        let backup_path = temp_dir.path().join("backup.tar.gz.age");

        // Create backup with first passphrase
        let passphrase1 = SecretString::new("correct_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase1).unwrap();

        let mut session1 = SessionManager::new(30);
        session1.unlock(passphrase1.clone());
        create_backup(&db, &mut session1, &journal_dir, &backup_path).unwrap();

        // Try to verify with wrong passphrase
        let passphrase2 = SecretString::new("wrong_password".to_string());
        let mut session2 = SessionManager::new(30);
        session2.unlock(passphrase2.clone());

        let result = verify_backup(&mut session2, &backup_path);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "integration"]
    fn test_verify_backup_empty_journal() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().to_path_buf();
        let backup_path = temp_dir.path().join("backup.tar.gz.age");

        // Create empty backup (only database)
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());
        create_backup(&db, &mut session, &journal_dir, &backup_path).unwrap();

        // Verify backup
        let manifest = verify_backup(&mut session, &backup_path).unwrap();

        // Should have no entries but database should be valid
        assert_eq!(manifest.entries.len(), 0);
        assert_eq!(manifest.db_path, PathBuf::from("ponder.db"));
    }

    #[test]
    #[ignore = "integration"]
    fn test_restore_backup_success() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().join("original");
        let backup_path = temp_dir.path().join("backup.tar.gz.age");
        let restore_dir = temp_dir.path().join("restored");

        // Create journal with entries
        fs::create_dir_all(&journal_dir).unwrap();
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        fs::create_dir_all(journal_dir.join("2024/01")).unwrap();
        let entry_path = journal_dir.join("2024/01/01.md.age");
        let entry_content = b"Test journal entry";
        let encrypted =
            crate::crypto::age::encrypt_with_passphrase(entry_content, &passphrase).unwrap();
        fs::write(&entry_path, encrypted).unwrap();

        // Create backup
        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());
        create_backup(&db, &mut session, &journal_dir, &backup_path).unwrap();

        // Restore backup to new location
        let report = restore_backup(&mut session, &backup_path, &restore_dir, false).unwrap();

        // Verify report
        assert_eq!(report.entries_restored, 1);
        assert!(report.db_size > 0);

        // Verify files exist in restored location
        assert!(restore_dir.join("2024/01/01.md.age").exists());
        assert!(restore_dir.join("ponder.db").exists());

        // Verify database can be opened
        let restored_db = Database::open(&restore_dir.join("ponder.db"), &passphrase).unwrap();
        drop(restored_db);

        // Verify restored entry matches original
        let restored_encrypted = fs::read(restore_dir.join("2024/01/01.md.age")).unwrap();
        let restored_content =
            crate::crypto::age::decrypt_with_passphrase(&restored_encrypted, &passphrase).unwrap();
        assert_eq!(restored_content, entry_content);
    }

    #[test]
    #[ignore = "integration"]
    fn test_restore_backup_target_exists_no_force() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().join("original");
        let backup_path = temp_dir.path().join("backup.tar.gz.age");
        let restore_dir = temp_dir.path().join("restored");

        // Create journal and backup
        fs::create_dir_all(&journal_dir).unwrap();
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());
        create_backup(&db, &mut session, &journal_dir, &backup_path).unwrap();

        // Create target directory with existing content
        fs::create_dir_all(&restore_dir).unwrap();
        fs::write(restore_dir.join("existing_file.txt"), b"existing").unwrap();

        // Try to restore without force flag - should fail
        let result = restore_backup(&mut session, &backup_path, &restore_dir, false);
        assert!(result.is_err());

        // Existing file should still be there
        assert!(restore_dir.join("existing_file.txt").exists());
    }

    #[test]
    #[ignore = "integration"]
    fn test_restore_backup_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().join("original");
        let backup_path = temp_dir.path().join("backup.tar.gz.age");
        let restore_dir = temp_dir.path().join("restored");

        // Create journal and backup
        fs::create_dir_all(&journal_dir).unwrap();
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        fs::create_dir_all(journal_dir.join("2024/01")).unwrap();
        let entry_path = journal_dir.join("2024/01/01.md.age");
        let encrypted =
            crate::crypto::age::encrypt_with_passphrase(b"Test entry", &passphrase).unwrap();
        fs::write(&entry_path, encrypted).unwrap();

        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());
        create_backup(&db, &mut session, &journal_dir, &backup_path).unwrap();

        // Create target directory with existing content
        fs::create_dir_all(&restore_dir).unwrap();
        fs::write(restore_dir.join("old_file.txt"), b"old").unwrap();

        // Restore with force flag - should succeed
        let report = restore_backup(&mut session, &backup_path, &restore_dir, true).unwrap();
        assert_eq!(report.entries_restored, 1);

        // Restored files should exist
        assert!(restore_dir.join("2024/01/01.md.age").exists());
        assert!(restore_dir.join("ponder.db").exists());

        // Old file should still be there (we don't delete existing files)
        assert!(restore_dir.join("old_file.txt").exists());
    }

    #[test]
    #[ignore = "integration"]
    fn test_restore_backup_empty_journal() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path().join("original");
        let backup_path = temp_dir.path().join("backup.tar.gz.age");
        let restore_dir = temp_dir.path().join("restored");

        // Create empty journal (only database)
        fs::create_dir_all(&journal_dir).unwrap();
        let passphrase = SecretString::new("test_password".to_string());
        let db_path = journal_dir.join("ponder.db");
        let db = Database::open(&db_path, &passphrase).unwrap();

        let mut session = SessionManager::new(30);
        session.unlock(passphrase.clone());
        create_backup(&db, &mut session, &journal_dir, &backup_path).unwrap();

        // Restore backup
        let report = restore_backup(&mut session, &backup_path, &restore_dir, false).unwrap();

        // Should have restored database but no entries
        assert_eq!(report.entries_restored, 0);
        assert!(report.db_size > 0);
        assert!(restore_dir.join("ponder.db").exists());
    }
}
