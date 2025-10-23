//! Migration engine for v1.0 to v2.0 journal entries.
//!
//! This module handles the migration of plaintext v1.0 journal entries
//! (YYYYMMDD.md) to encrypted v2.0 format (YYYY/MM/DD.md.age) with
//! automatic embedding generation.

use crate::ai::chunking::chunk_text;
use crate::ai::OllamaClient;
use crate::config::Config;
use crate::constants::{DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, DEFAULT_EMBED_MODEL};
use crate::crypto::temp::{decrypt_to_temp, encrypt_from_temp};
use crate::crypto::SessionManager;
use crate::db::embeddings::insert_embedding;
use crate::db::entries::upsert_entry;
use crate::db::Database;
use crate::errors::AppResult;
use crate::ops::detection::V1Entry;
use blake3::Hash;
use chrono::NaiveDate;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Result of a single entry migration.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationResult {
    /// Path to the v1.0 entry
    pub v1_path: PathBuf,
    /// Path to the v2.0 encrypted entry
    pub v2_path: PathBuf,
    /// Date of the entry
    pub date: NaiveDate,
    /// Whether the migration succeeded
    pub success: bool,
    /// Whether checksums matched during verification
    pub checksum_match: bool,
    /// Error message if migration failed
    pub error_message: Option<String>,
}

/// Progress callback for batch migration.
///
/// Called after each entry is processed with:
/// - Current entry number (1-indexed)
/// - Total number of entries
/// - Result of the migration
pub type ProgressCallback = Box<dyn Fn(usize, usize, &MigrationResult)>;

/// Migrates a single v1.0 entry to v2.0 format.
///
/// # Flow
///
/// 1. Read v1.0 plaintext entry
/// 2. Calculate v1 checksum for verification
/// 3. Encrypt to v2.0 format (YYYY/MM/DD.md.age)
/// 4. Record migration in database
/// 5. Verify migration by decrypting and comparing checksums
/// 6. Generate and store embeddings if Ollama available
/// 7. Update migration status to "verified" if successful
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `db` - Database connection
/// * `session` - Session manager for encryption keys
/// * `ai_client` - Optional Ollama client for embeddings (None skips embedding generation)
/// * `v1_entry` - V1.0 entry to migrate
///
/// # Returns
///
/// Returns a `MigrationResult` with success status and details.
///
/// # Errors
///
/// Does not return errors - captures them in `MigrationResult.error_message`.
/// This allows batch migration to continue even if individual entries fail.
pub fn migrate_entry(
    config: &Config,
    db: &Database,
    session: &mut SessionManager,
    ai_client: Option<&OllamaClient>,
    v1_entry: &V1Entry,
) -> MigrationResult {
    let v1_path = &v1_entry.path;
    let date = v1_entry.date;

    info!("Migrating entry: {:?} ({})", v1_path, date);

    // Get passphrase (should already be unlocked)
    let passphrase = match session.get_passphrase() {
        Ok(p) => p,
        Err(e) => {
            return MigrationResult {
                v1_path: v1_path.clone(),
                v2_path: PathBuf::new(),
                date,
                success: false,
                checksum_match: false,
                error_message: Some(format!("Failed to get passphrase: {}", e)),
            };
        }
    };

    // Determine v2.0 encrypted path
    let v2_path = get_v2_encrypted_path(&config.journal_dir, date);

    // Ensure directory structure exists
    if let Some(parent) = v2_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return MigrationResult {
                v1_path: v1_path.clone(),
                v2_path,
                date,
                success: false,
                checksum_match: false,
                error_message: Some(format!("Failed to create directory: {}", e)),
            };
        }
    }

    // Check if v2 already exists (shouldn't happen, but be defensive)
    if v2_path.exists() {
        warn!("V2.0 entry already exists at {:?}, skipping", v2_path);
        return MigrationResult {
            v1_path: v1_path.clone(),
            v2_path,
            date,
            success: false,
            checksum_match: false,
            error_message: Some("V2.0 entry already exists".to_string()),
        };
    }

    // Read v1.0 plaintext content
    let v1_content = match fs::read_to_string(v1_path) {
        Ok(c) => c,
        Err(e) => {
            return MigrationResult {
                v1_path: v1_path.clone(),
                v2_path,
                date,
                success: false,
                checksum_match: false,
                error_message: Some(format!("Failed to read v1.0 entry: {}", e)),
            };
        }
    };

    // Calculate v1 checksum for verification
    let v1_checksum = blake3::hash(v1_content.as_bytes());

    // Create temp file with v1 content
    let temp_dir = match crate::crypto::temp::get_secure_temp_dir() {
        Ok(d) => d,
        Err(e) => {
            return MigrationResult {
                v1_path: v1_path.clone(),
                v2_path,
                date,
                success: false,
                checksum_match: false,
                error_message: Some(format!("Failed to create temp dir: {}", e)),
            };
        }
    };

    let temp_path = temp_dir.join(format!("ponder-migrate-{}.md", uuid::Uuid::new_v4()));
    if let Err(e) = fs::write(&temp_path, &v1_content) {
        return MigrationResult {
            v1_path: v1_path.clone(),
            v2_path,
            date,
            success: false,
            checksum_match: false,
            error_message: Some(format!("Failed to write temp file: {}", e)),
        };
    }

    // Encrypt to v2.0 format
    if let Err(e) = encrypt_from_temp(&temp_path, &v2_path, passphrase) {
        return MigrationResult {
            v1_path: v1_path.clone(),
            v2_path,
            date,
            success: false,
            checksum_match: false,
            error_message: Some(format!("Encryption failed: {}", e)),
        };
    }

    debug!("Encrypted v1.0 entry to v2.0: {:?}", v2_path);

    // Record migration in database
    let v1_filename = v1_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let v2_relative = v2_path
        .strip_prefix(&config.journal_dir)
        .unwrap_or(&v2_path)
        .to_string_lossy();

    if let Err(e) = db.record_migration(
        v1_filename,
        &v2_relative,
        &date.format("%Y-%m-%d").to_string(),
        "pending",
    ) {
        // Non-fatal - log but continue
        warn!("Failed to record migration in database: {}", e);
    }

    // Verify migration by decrypting and comparing checksums
    let checksum_match = match verify_migration(&v2_path, passphrase, &v1_checksum) {
        Ok(matches) => matches,
        Err(e) => {
            warn!("Verification failed: {}", e);
            // Update status to failed
            let _ = db.update_migration_status(
                v1_filename,
                "failed",
                false,
                Some(&format!("Verification failed: {}", e)),
            );
            return MigrationResult {
                v1_path: v1_path.clone(),
                v2_path,
                date,
                success: false,
                checksum_match: false,
                error_message: Some(format!("Verification failed: {}", e)),
            };
        }
    };

    if !checksum_match {
        warn!("Checksum mismatch after migration");
        let _ = db.update_migration_status(v1_filename, "failed", false, Some("Checksum mismatch"));
        return MigrationResult {
            v1_path: v1_path.clone(),
            v2_path,
            date,
            success: false,
            checksum_match: false,
            error_message: Some("Checksum mismatch after encryption/decryption".to_string()),
        };
    }

    // Calculate checksum for v2 (use v1 checksum since content is identical)
    let v2_checksum_str = v1_checksum.to_hex().to_string();

    // Calculate word count
    let word_count = v1_content.split_whitespace().count();

    // Update database with entry metadata
    let conn = match db.get_conn() {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to get database connection: {}", e);
            return MigrationResult {
                v1_path: v1_path.clone(),
                v2_path,
                date,
                success: false,
                checksum_match: true,
                error_message: Some(format!("Database connection failed: {}", e)),
            };
        }
    };

    let entry_id = match upsert_entry(&conn, &v2_path, date, &v2_checksum_str, word_count) {
        Ok(id) => id,
        Err(e) => {
            warn!("Failed to upsert entry: {}", e);
            return MigrationResult {
                v1_path: v1_path.clone(),
                v2_path,
                date,
                success: false,
                checksum_match: true,
                error_message: Some(format!("Failed to upsert entry: {}", e)),
            };
        }
    };

    // Generate embeddings if Ollama available
    if let Some(ai_client) = ai_client {
        debug!("Generating embeddings for migrated entry");
        match generate_embeddings_for_entry(&conn, ai_client, entry_id, &v2_path, passphrase) {
            Ok(_) => debug!("Embeddings generated successfully"),
            Err(e) => {
                // Non-fatal - embeddings can be regenerated later
                warn!("Failed to generate embeddings: {}", e);
            }
        }
    } else {
        debug!("Skipping embedding generation (no AI client)");
    }

    // Update migration status to verified
    if let Err(e) = db.update_migration_status(v1_filename, "verified", true, None) {
        warn!("Failed to update migration status: {}", e);
    }

    info!("Successfully migrated: {:?} → {:?}", v1_path, v2_path);

    MigrationResult {
        v1_path: v1_path.clone(),
        v2_path,
        date,
        success: true,
        checksum_match: true,
        error_message: None,
    }
}

/// Verifies a migration by decrypting and comparing checksums.
///
/// # Arguments
///
/// * `v2_path` - Path to the encrypted v2.0 entry
/// * `passphrase` - Encryption passphrase
/// * `expected_checksum` - Expected BLAKE3 checksum from v1.0 entry
///
/// # Returns
///
/// Returns `true` if checksums match, `false` otherwise.
///
/// # Errors
///
/// Returns an error if decryption fails.
pub fn verify_migration(
    v2_path: &Path,
    passphrase: &age::secrecy::SecretString,
    expected_checksum: &Hash,
) -> AppResult<bool> {
    debug!("Verifying migration for: {:?}", v2_path);

    // Decrypt to temp
    let temp_path = decrypt_to_temp(v2_path, passphrase)?;

    // Read decrypted content
    let decrypted_content = fs::read_to_string(&temp_path)?;

    // Calculate checksum
    let actual_checksum = blake3::hash(decrypted_content.as_bytes());

    // Temp file is automatically cleaned up
    let matches = actual_checksum == *expected_checksum;

    if matches {
        debug!("Verification successful: checksums match");
    } else {
        warn!(
            "Verification failed: expected {}, got {}",
            expected_checksum.to_hex(),
            actual_checksum.to_hex()
        );
    }

    Ok(matches)
}

/// Migrates all v1.0 entries to v2.0 format.
///
/// Processes entries sequentially with progress callbacks. Each entry is:
/// 1. Migrated to encrypted v2.0 format
/// 2. Verified for data integrity
/// 3. Embedded if Ollama is available
/// 4. Tracked in migration_log and migration_state
///
/// If a migration fails, processing continues with remaining entries.
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `db` - Database connection
/// * `session` - Session manager for encryption keys
/// * `ai_client` - Optional Ollama client (None skips embeddings)
/// * `v1_entries` - Vector of v1.0 entries to migrate
/// * `progress_callback` - Optional callback for progress updates
///
/// # Returns
///
/// Returns a vector of `MigrationResult` for all entries.
///
/// # Errors
///
/// Returns an error if:
/// - Session is locked
/// - Database operations fail
pub fn migrate_all_entries(
    config: &Config,
    db: &Database,
    session: &mut SessionManager,
    ai_client: Option<&OllamaClient>,
    v1_entries: Vec<V1Entry>,
    progress_callback: Option<ProgressCallback>,
) -> AppResult<Vec<MigrationResult>> {
    let total = v1_entries.len();
    info!("Starting migration of {} entries", total);

    // Initialize migration state
    db.init_migration_state(total as i64)?;

    let mut results = Vec::new();
    let mut migrated_count = 0;
    let mut verified_count = 0;
    let mut failed_count = 0;

    for (idx, v1_entry) in v1_entries.into_iter().enumerate() {
        let current = idx + 1;

        // Migrate entry
        let result = migrate_entry(config, db, session, ai_client, &v1_entry);

        // Update counters
        if result.success {
            migrated_count += 1;
            if result.checksum_match {
                verified_count += 1;
            }
        } else {
            failed_count += 1;
        }

        // Update progress in database
        db.update_migration_progress(migrated_count, verified_count, failed_count)?;

        // Call progress callback
        if let Some(ref callback) = progress_callback {
            callback(current, total, &result);
        }

        results.push(result);
    }

    // Mark migration as completed
    db.complete_migration()?;

    info!(
        "Migration completed: {} migrated, {} verified, {} failed",
        migrated_count, verified_count, failed_count
    );

    Ok(results)
}

/// Gets the v2.0 encrypted entry path for a given date.
///
/// Returns a path following the structure: `YYYY/MM/DD.md.age`
///
/// # Arguments
///
/// * `journal_dir` - Base journal directory
/// * `date` - Date of the entry
///
/// # Returns
///
/// PathBuf with structure: `journal_dir/YYYY/MM/DD.md.age`
fn get_v2_encrypted_path(journal_dir: &Path, date: NaiveDate) -> PathBuf {
    let year = date.format("%Y").to_string();
    let month = date.format("%m").to_string();
    let day = date.format("%d").to_string();

    journal_dir
        .join(year)
        .join(month)
        .join(format!("{}.md.age", day))
}

/// Generates and stores embeddings for a migrated entry.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `ai_client` - Ollama client
/// * `entry_id` - Database entry ID
/// * `encrypted_path` - Path to encrypted entry
/// * `passphrase` - Decryption passphrase
///
/// # Errors
///
/// Returns an error if:
/// - Decryption fails
/// - Ollama is unavailable
/// - Database operations fail
fn generate_embeddings_for_entry(
    conn: &rusqlite::Connection,
    ai_client: &OllamaClient,
    entry_id: i64,
    encrypted_path: &Path,
    passphrase: &age::secrecy::SecretString,
) -> AppResult<()> {
    // Decrypt to temp
    let temp_path = decrypt_to_temp(encrypted_path, passphrase)?;

    // Read content
    let content = fs::read_to_string(&temp_path)?;

    // Chunk the text
    let chunks = chunk_text(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);
    debug!("Generated {} chunks for entry {}", chunks.len(), entry_id);

    // Generate and store embeddings
    for (idx, chunk) in chunks.iter().enumerate() {
        // Generate embedding
        let embedding_vec = ai_client.embed(DEFAULT_EMBED_MODEL, chunk)?;

        // Calculate chunk checksum
        let chunk_checksum = blake3::hash(chunk.as_bytes()).to_hex().to_string();

        // Store in database
        insert_embedding(conn, entry_id, idx, &embedding_vec, &chunk_checksum)?;
    }

    // Mark entry as embedded
    conn.execute(
        "UPDATE entries SET embedded_at = CURRENT_TIMESTAMP WHERE id = ?",
        [entry_id],
    )
    .map_err(crate::errors::DatabaseError::Sqlite)?;

    debug!("Stored {} embeddings for entry {}", chunks.len(), entry_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::SecretString;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_v2_encrypted_path() {
        let journal_dir = PathBuf::from("/journal");
        let date = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();

        let path = get_v2_encrypted_path(&journal_dir, date);
        assert_eq!(path, PathBuf::from("/journal/2024/03/05.md.age"));
    }

    #[test]
    fn test_get_v2_encrypted_path_padding() {
        let journal_dir = PathBuf::from("/journal");
        let date = NaiveDate::from_ymd_opt(2024, 1, 9).unwrap();

        let path = get_v2_encrypted_path(&journal_dir, date);
        assert_eq!(path, PathBuf::from("/journal/2024/01/09.md.age"));
    }

    #[test]
    fn test_migrate_entry_success() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();
        let db_path = temp_dir.path().join("test.db");

        // Setup
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();
        let mut session = SessionManager::new(60); // 60 minutes
        session.unlock(passphrase.clone());

        let config = Config {
            journal_dir: journal_dir.to_path_buf(),
            editor: "echo".to_string(),
            db_path: db_path.clone(),
            session_timeout_minutes: 60,
            ollama_url: "http://localhost:11434".to_string(),
        };

        // Create v1.0 entry
        let v1_path = journal_dir.join("20240115.md");
        let v1_content = "# 2024-01-15\n\nTest entry content.";
        fs::write(&v1_path, v1_content).unwrap();

        let v1_entry = V1Entry {
            path: v1_path.clone(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };

        // Migrate (without AI client)
        let result = migrate_entry(&config, &db, &mut session, None, &v1_entry);

        // Verify result
        assert!(result.success);
        assert!(result.checksum_match);
        assert_eq!(result.v1_path, v1_path);
        assert_eq!(result.v2_path, journal_dir.join("2024/01/15.md.age"));

        // Verify v2.0 file exists
        assert!(result.v2_path.exists());

        // Verify can decrypt
        let temp_path = decrypt_to_temp(&result.v2_path, &passphrase).unwrap();
        let decrypted = fs::read_to_string(&temp_path).unwrap();
        assert_eq!(decrypted, v1_content);
    }

    #[test]
    fn test_verify_migration_success() {
        let temp_dir = TempDir::new().unwrap();
        let passphrase = SecretString::new("test_password".to_string());

        // Create and encrypt content
        let original_content = "Test content for verification";
        let expected_checksum = blake3::hash(original_content.as_bytes());

        let temp_path = temp_dir
            .path()
            .join(format!("temp-{}.md", uuid::Uuid::new_v4()));
        fs::write(&temp_path, original_content).unwrap();

        let encrypted_path = temp_dir.path().join("encrypted.md.age");
        encrypt_from_temp(&temp_path, &encrypted_path, &passphrase).unwrap();

        // Verify
        let matches = verify_migration(&encrypted_path, &passphrase, &expected_checksum).unwrap();
        assert!(matches);
    }

    #[test]
    fn test_verify_migration_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let passphrase = SecretString::new("test_password".to_string());

        // Create and encrypt content
        let original_content = "Test content";
        let wrong_checksum = blake3::hash(b"Different content");

        let temp_path = temp_dir
            .path()
            .join(format!("temp-{}.md", uuid::Uuid::new_v4()));
        fs::write(&temp_path, original_content).unwrap();

        let encrypted_path = temp_dir.path().join("encrypted.md.age");
        encrypt_from_temp(&temp_path, &encrypted_path, &passphrase).unwrap();

        // Verify with wrong checksum
        let matches = verify_migration(&encrypted_path, &passphrase, &wrong_checksum).unwrap();
        assert!(!matches);
    }

    #[test]
    fn test_migrate_all_entries() {
        let temp_dir = TempDir::new().unwrap();
        let journal_dir = temp_dir.path();
        let db_path = temp_dir.path().join("test.db");

        // Setup
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();
        let mut session = SessionManager::new(60); // 60 minutes
        session.unlock(passphrase.clone());

        let config = Config {
            journal_dir: journal_dir.to_path_buf(),
            editor: "echo".to_string(),
            db_path: db_path.clone(),
            session_timeout_minutes: 60,
            ollama_url: "http://localhost:11434".to_string(),
        };

        // Create v1.0 entries
        let v1_entries = vec![
            V1Entry {
                path: journal_dir.join("20240115.md"),
                date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            },
            V1Entry {
                path: journal_dir.join("20240116.md"),
                date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            },
        ];

        for entry in &v1_entries {
            fs::write(&entry.path, format!("Content for {}", entry.date)).unwrap();
        }

        // Migrate all
        let results =
            migrate_all_entries(&config, &db, &mut session, None, v1_entries, None).unwrap();

        // Verify results
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
        assert!(results[0].v2_path.exists());
        assert!(results[1].v2_path.exists());

        // Verify migration state
        let state = db.get_migration_state().unwrap().unwrap();
        assert_eq!(state.total_entries, 2);
        assert_eq!(state.migrated_count, 2);
        assert_eq!(state.verified_count, 2);
        assert_eq!(state.failed_count, 0);
        assert!(state.completed_at.is_some());
    }
}
