//! Detection and analysis of v1.0 journal entries for migration.

use crate::db::Database;
use crate::errors::AppResult;
use chrono::NaiveDate;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Result of scanning for v1.0 entries.
#[derive(Debug, Clone, PartialEq)]
pub struct V1Entry {
    /// Path to the v1.0 entry (e.g., "20240115.md")
    pub path: PathBuf,
    /// Parsed date
    pub date: NaiveDate,
}

/// Overall migration state analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDetectionResult {
    /// V1.0 entries found (not yet migrated)
    pub v1_entries: Vec<V1Entry>,
    /// V1.0 entries already migrated (present in migration_log)
    pub migrated_entries: Vec<PathBuf>,
    /// Total v1.0 entries discovered
    pub total_v1: usize,
    /// Number already migrated
    pub already_migrated: usize,
    /// Number pending migration
    pub pending: usize,
}

/// Scans for v1.0 entries in the journal directory.
///
/// V1.0 entries are Markdown files in the root directory matching the pattern
/// `YYYYMMDD.md` (e.g., "20240115.md").
///
/// # Arguments
///
/// * `journal_dir` - Root journal directory to scan
///
/// # Returns
///
/// Returns a vector of discovered v1.0 entries with their parsed dates.
///
/// # Errors
///
/// Returns an error if:
/// - Directory cannot be read
/// - File names cannot be parsed
pub fn scan_v1_entries(journal_dir: &Path) -> AppResult<Vec<V1Entry>> {
    debug!("Scanning for v1.0 entries in: {:?}", journal_dir);

    let mut entries = Vec::new();

    // Only scan root directory (non-recursive)
    let read_dir = match std::fs::read_dir(journal_dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Journal directory doesn't exist yet - no entries to migrate
            debug!("Journal directory not found, no v1.0 entries");
            return Ok(entries);
        }
        Err(e) => return Err(e.into()),
    };

    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();

        // Skip directories (v2.0 uses YYYY/MM/DD.md.age structure)
        if path.is_dir() {
            continue;
        }

        // Check if it's a .md file (v1.0 format)
        if let Some(extension) = path.extension() {
            if extension != "md" {
                continue;
            }
        } else {
            continue;
        }

        // Parse filename as YYYYMMDD.md
        if let Some(filename) = path.file_stem() {
            if let Some(filename_str) = filename.to_str() {
                // Must be exactly 8 digits (YYYYMMDD)
                if filename_str.len() == 8 && filename_str.chars().all(|c| c.is_ascii_digit()) {
                    // Parse date
                    match NaiveDate::parse_from_str(filename_str, "%Y%m%d") {
                        Ok(date) => {
                            debug!("Found v1.0 entry: {:?} ({})", path, date);
                            entries.push(V1Entry {
                                path: path.clone(),
                                date,
                            });
                        }
                        Err(e) => {
                            debug!("Failed to parse date from {}: {}", filename_str, e);
                            continue;
                        }
                    }
                }
            }
        }
    }

    info!("Found {} v1.0 entries", entries.len());
    Ok(entries)
}

/// Checks if a v1.0 entry has been migrated.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `v1_path` - Path to the v1.0 entry (can be absolute or just filename)
///
/// # Returns
///
/// Returns `true` if the entry has been migrated (status is "migrated" or "verified"),
/// `false` otherwise.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub fn is_migrated(db: &Database, v1_path: &Path) -> AppResult<bool> {
    // Use only the filename for database lookup (migration_log stores just filenames)
    let filename = v1_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if let Some(record) = db.get_migration_status(filename)? {
        Ok(record.status == "migrated" || record.status == "verified")
    } else {
        Ok(false)
    }
}

/// Detects and analyzes migration state.
///
/// Performs a comprehensive analysis of v1.0 entries and their migration status:
/// - Scans for all v1.0 entries in journal directory
/// - Queries migration_log to determine which are already migrated
/// - Returns detailed breakdown for migration planning
///
/// # Arguments
///
/// * `journal_dir` - Root journal directory
/// * `db` - Database connection
///
/// # Returns
///
/// Returns a detailed migration detection result with counts and file lists.
///
/// # Errors
///
/// Returns an error if:
/// - Directory scan fails
/// - Database queries fail
pub fn detect_migration_state(
    journal_dir: &Path,
    db: &Database,
) -> AppResult<MigrationDetectionResult> {
    debug!("Detecting migration state in: {:?}", journal_dir);

    // Scan for all v1.0 entries
    let all_v1_entries = scan_v1_entries(journal_dir)?;

    // Separate into migrated and pending
    let mut v1_entries = Vec::new();
    let mut migrated_entries = Vec::new();

    for entry in all_v1_entries {
        if is_migrated(db, &entry.path)? {
            migrated_entries.push(entry.path);
        } else {
            v1_entries.push(entry);
        }
    }

    let result = MigrationDetectionResult {
        total_v1: v1_entries.len() + migrated_entries.len(),
        already_migrated: migrated_entries.len(),
        pending: v1_entries.len(),
        v1_entries,
        migrated_entries,
    };

    info!(
        "Migration detection: {} total, {} already migrated, {} pending",
        result.total_v1, result.already_migrated, result.pending
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::SecretString;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_v1_entries_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();

        let entries = scan_v1_entries(journal_dir).unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_scan_v1_entries_no_journal_dir() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");

        // Should not error, just return empty
        let entries = scan_v1_entries(&nonexistent).unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_scan_v1_entries_finds_valid_entries() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();

        // Create valid v1.0 entries
        fs::write(journal_dir.join("20240115.md"), "Entry 1").unwrap();
        fs::write(journal_dir.join("20240220.md"), "Entry 2").unwrap();
        fs::write(journal_dir.join("20231231.md"), "Entry 3").unwrap();

        let entries = scan_v1_entries(journal_dir).unwrap();
        assert_eq!(entries.len(), 3);

        // Verify dates are parsed correctly
        let dates: Vec<NaiveDate> = entries.iter().map(|e| e.date).collect();
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2024, 2, 20).unwrap()));
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()));
    }

    #[test]
    fn test_scan_v1_entries_ignores_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();

        // Create various files that should be ignored
        fs::write(journal_dir.join("20240115.md"), "Valid").unwrap();
        fs::write(journal_dir.join("invalid.md"), "Invalid name").unwrap();
        fs::write(journal_dir.join("2024011.md"), "Too short").unwrap();
        fs::write(journal_dir.join("202401155.md"), "Too long").unwrap();
        fs::write(journal_dir.join("20240115.txt"), "Wrong extension").unwrap();
        fs::write(journal_dir.join("20240115.md.age"), "V2.0 format").unwrap();
        fs::write(journal_dir.join("abcdefgh.md"), "Not digits").unwrap();

        // Create v2.0 directory structure (should be ignored)
        fs::create_dir_all(journal_dir.join("2024/01")).unwrap();
        fs::write(
            journal_dir.join("2024/01/15.md.age"),
            "V2.0 encrypted entry",
        )
        .unwrap();

        let entries = scan_v1_entries(journal_dir).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].date,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );
    }

    #[test]
    fn test_is_migrated_not_in_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        let v1_path = PathBuf::from("20240115.md");
        let result = is_migrated(&db, &v1_path).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_is_migrated_pending_status() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Record as pending
        db.record_migration("20240115.md", "2024/01/15.md.age", "2024-01-15", "pending")
            .unwrap();

        let v1_path = PathBuf::from("20240115.md");
        let result = is_migrated(&db, &v1_path).unwrap();
        assert!(!result); // Pending is not considered migrated
    }

    #[test]
    fn test_is_migrated_completed_status() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Record as migrated
        db.record_migration("20240115.md", "2024/01/15.md.age", "2024-01-15", "pending")
            .unwrap();
        db.update_migration_status("20240115.md", "migrated", false, None)
            .unwrap();

        let v1_path = PathBuf::from("20240115.md");
        let result = is_migrated(&db, &v1_path).unwrap();
        assert!(result);
    }

    #[test]
    fn test_is_migrated_verified_status() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Record as verified
        db.record_migration("20240115.md", "2024/01/15.md.age", "2024-01-15", "pending")
            .unwrap();
        db.update_migration_status("20240115.md", "verified", true, None)
            .unwrap();

        let v1_path = PathBuf::from("20240115.md");
        let result = is_migrated(&db, &v1_path).unwrap();
        assert!(result);
    }

    #[test]
    fn test_detect_migration_state_no_entries() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        let result = detect_migration_state(journal_dir, &db).unwrap();
        assert_eq!(result.total_v1, 0);
        assert_eq!(result.already_migrated, 0);
        assert_eq!(result.pending, 0);
    }

    #[test]
    fn test_detect_migration_state_all_pending() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Create v1.0 entries
        fs::write(journal_dir.join("20240115.md"), "Entry 1").unwrap();
        fs::write(journal_dir.join("20240220.md"), "Entry 2").unwrap();

        let result = detect_migration_state(journal_dir, &db).unwrap();
        assert_eq!(result.total_v1, 2);
        assert_eq!(result.already_migrated, 0);
        assert_eq!(result.pending, 2);
        assert_eq!(result.v1_entries.len(), 2);
        assert_eq!(result.migrated_entries.len(), 0);
    }

    #[test]
    fn test_detect_migration_state_mixed() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Create v1.0 entries
        fs::write(journal_dir.join("20240115.md"), "Entry 1").unwrap();
        fs::write(journal_dir.join("20240220.md"), "Entry 2").unwrap();
        fs::write(journal_dir.join("20240330.md"), "Entry 3").unwrap();

        // Mark first as migrated
        db.record_migration("20240115.md", "2024/01/15.md.age", "2024-01-15", "pending")
            .unwrap();
        db.update_migration_status("20240115.md", "verified", true, None)
            .unwrap();

        let result = detect_migration_state(journal_dir, &db).unwrap();
        assert_eq!(result.total_v1, 3);
        assert_eq!(result.already_migrated, 1);
        assert_eq!(result.pending, 2);
        assert_eq!(result.v1_entries.len(), 2);
        assert_eq!(result.migrated_entries.len(), 1);
    }
}
