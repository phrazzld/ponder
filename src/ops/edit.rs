//! Edit journal entries with encryption and embedding.

use crate::ai::chunking::chunk_text;
use crate::ai::OllamaClient;
use crate::config::Config;
use crate::constants::{DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, DEFAULT_EMBED_MODEL};
use crate::crypto::temp::{decrypt_to_temp, encrypt_from_temp};
use crate::crypto::SessionManager;
use crate::db::embeddings::insert_embedding;
use crate::db::entries::upsert_entry;
use crate::db::Database;
use crate::errors::{AppError, AppResult, EditorError};
use chrono::{DateTime, Local, NaiveDate};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

/// Edits a journal entry with full encryption and embedding pipeline.
///
/// # Flow
///
/// 1. Check if encrypted entry exists; if not, create new file
/// 2. Decrypt to temporary location (if exists)
/// 3. Initialize with date header if new
/// 4. Open in editor
/// 5. Re-encrypt with session key
/// 6. Calculate checksum and detect changes
/// 7. Generate and store embeddings if content changed
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `db` - Database connection
/// * `session` - Session manager for encryption keys
/// * `ai_client` - Ollama client for embeddings
/// * `date` - Date of entry to edit
/// * `reference_datetime` - Reference datetime for timestamps
///
/// # Errors
///
/// Returns an error if:
/// - Session is locked (passphrase needed)
/// - File decryption fails
/// - Editor fails to launch
/// - Encryption fails
/// - Database operations fail
/// - Ollama is offline or model not found
pub fn edit_entry(
    config: &Config,
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    date: NaiveDate,
    _reference_datetime: &DateTime<Local>,
) -> AppResult<()> {
    // Ensure session is unlocked
    let passphrase = session.get_passphrase()?;

    // Determine encrypted file path (YYYY/MM/DD.md.age)
    let encrypted_path = get_encrypted_entry_path(&config.journal_dir, date);

    // Ensure directory structure exists
    if let Some(parent) = encrypted_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let is_new = !encrypted_path.exists();

    // Decrypt to temp (or create new temp file)
    let temp_path = if is_new {
        let temp_dir = crate::crypto::temp::get_secure_temp_dir()?;
        let temp_path = temp_dir.join(format!("ponder-new-{}.md", uuid::Uuid::new_v4()));

        // Initialize with date header
        let header = format!("# {}\n\n", date.format("%Y-%m-%d"));
        fs::write(&temp_path, header)?;

        debug!("Created new temp file with header: {:?}", temp_path);
        temp_path
    } else {
        decrypt_to_temp(&encrypted_path, passphrase)?
    };

    // Get checksum before editing
    let original_checksum = calculate_checksum(&temp_path)?;

    // Launch editor
    launch_editor(config, &temp_path)?;

    // Get checksum after editing
    let new_checksum = calculate_checksum(&temp_path)?;
    let content_changed = original_checksum != new_checksum;

    // Re-encrypt
    encrypt_from_temp(&temp_path, &encrypted_path, passphrase)?;
    info!("Entry saved to {:?}", encrypted_path);

    // Update database
    let content = fs::read_to_string(&encrypted_path)?;
    let word_count = content.split_whitespace().count();

    let conn = db.get_conn()?;
    let entry_id = upsert_entry(&conn, &encrypted_path, date, &new_checksum, word_count)?;

    // Generate and store embeddings if content changed
    if content_changed {
        info!("Content changed, generating embeddings...");
        generate_and_store_embeddings(&conn, ai_client, entry_id, &encrypted_path, passphrase)?;
    } else {
        debug!("Content unchanged, skipping embedding generation");
    }

    Ok(())
}

/// Get the encrypted entry path for a given date.
///
/// Returns a path following the structure: `YYYY/MM/DD.md.age`
///
/// This deterministic structure:
/// - Groups entries by year and month for organization
/// - Uses zero-padded month and day (01-12, 01-31)
/// - Always uses `.md.age` extension for encrypted markdown
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
/// use chrono::NaiveDate;
///
/// let journal_dir = PathBuf::from("/journal");
/// let date = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
///
/// // Path will be: /journal/2024/03/05.md.age
/// ```
///
/// # Arguments
///
/// * `journal_dir` - Base journal directory
/// * `date` - Date of the entry
///
/// # Returns
///
/// PathBuf with structure: `journal_dir/YYYY/MM/DD.md.age`
fn get_encrypted_entry_path(journal_dir: &std::path::Path, date: NaiveDate) -> PathBuf {
    let year = date.format("%Y").to_string();
    let month = date.format("%m").to_string();
    let day = date.format("%d").to_string();

    journal_dir
        .join(year)
        .join(month)
        .join(format!("{}.md.age", day))
}

/// Calculate BLAKE3 checksum of a file.
fn calculate_checksum(path: &std::path::Path) -> AppResult<String> {
    let content = fs::read(path)?;
    let hash = blake3::hash(&content);
    Ok(hash.to_hex().to_string())
}

/// Launch the configured editor.
fn launch_editor(config: &Config, path: &std::path::Path) -> AppResult<()> {
    debug!("Launching editor: {} {:?}", config.editor, path);

    let status = Command::new(&config.editor).arg(path).status();

    match status {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::Editor(EditorError::CommandNotFound {
                command: config.editor.clone(),
                source: e,
            }));
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(AppError::Editor(EditorError::PermissionDenied {
                command: config.editor.clone(),
                source: e,
            }));
        }
        Err(e) => {
            return Err(AppError::Editor(EditorError::ExecutionFailed {
                command: config.editor.clone(),
                source: e,
            }));
        }
        Ok(status) if !status.success() => {
            return Err(AppError::Editor(EditorError::NonZeroExit {
                command: config.editor.clone(),
                status_code: status.code().unwrap_or(-1),
            }));
        }
        Ok(_) => {}
    }

    Ok(())
}

/// Generate embeddings for an entry and store them in the database.
fn generate_and_store_embeddings(
    conn: &rusqlite::Connection,
    ai_client: &OllamaClient,
    entry_id: i64,
    encrypted_path: &std::path::Path,
    passphrase: &age::secrecy::SecretString,
) -> AppResult<()> {
    // Decrypt to read content
    let temp_path = decrypt_to_temp(encrypted_path, passphrase)?;
    let content = fs::read_to_string(&temp_path)?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    // Chunk the text
    let chunks = chunk_text(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);
    debug!("Generated {} chunks for embedding", chunks.len());

    // Generate and store embeddings for each chunk
    for (idx, chunk) in chunks.iter().enumerate() {
        debug!(
            "Generating embedding for chunk {}/{}",
            idx + 1,
            chunks.len()
        );

        let embedding = ai_client.embed(DEFAULT_EMBED_MODEL, chunk)?;
        let chunk_hash = blake3::hash(chunk.as_bytes());
        let chunk_checksum = chunk_hash.to_hex().to_string();

        insert_embedding(conn, entry_id, idx, &embedding, &chunk_checksum)?;
    }

    info!("Stored {} embeddings for entry {}", chunks.len(), entry_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_encrypted_entry_path_structure() {
        let journal_dir = PathBuf::from("/journal");

        // Test single-digit month and day
        let date = NaiveDate::from_ymd_opt(2024, 3, 5).unwrap();
        let path = get_encrypted_entry_path(&journal_dir, date);
        assert_eq!(
            path,
            PathBuf::from("/journal/2024/03/05.md.age"),
            "Path should use zero-padded month and day"
        );

        // Test double-digit month and day
        let date = NaiveDate::from_ymd_opt(2024, 11, 25).unwrap();
        let path = get_encrypted_entry_path(&journal_dir, date);
        assert_eq!(
            path,
            PathBuf::from("/journal/2024/11/25.md.age"),
            "Path should maintain structure for double-digit values"
        );

        // Test January 1st (edge case)
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let path = get_encrypted_entry_path(&journal_dir, date);
        assert_eq!(
            path,
            PathBuf::from("/journal/2024/01/01.md.age"),
            "Path should handle January 1st correctly"
        );

        // Test December 31st (edge case)
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let path = get_encrypted_entry_path(&journal_dir, date);
        assert_eq!(
            path,
            PathBuf::from("/journal/2024/12/31.md.age"),
            "Path should handle December 31st correctly"
        );
    }

    #[test]
    fn test_encrypted_entry_path_always_has_age_extension() {
        let journal_dir = PathBuf::from("/journal");
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let path = get_encrypted_entry_path(&journal_dir, date);

        assert!(
            path.extension().is_some_and(|ext| ext == "age"),
            "Path must have .age extension"
        );
        assert!(
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".md.age")),
            "Filename must end with .md.age"
        );
    }

    #[test]
    fn test_encrypted_entry_path_directory_structure() {
        let journal_dir = PathBuf::from("/test");
        let date = NaiveDate::from_ymd_opt(2024, 7, 20).unwrap();
        let path = get_encrypted_entry_path(&journal_dir, date);

        // Verify components
        let components: Vec<_> = path.components().collect();
        assert!(
            components.len() >= 4,
            "Path should have at least 4 components"
        );

        // Check that path contains year/month/day structure
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("2024"), "Path should contain year");
        assert!(
            path_str.contains("07"),
            "Path should contain zero-padded month"
        );
        assert!(path_str.contains("20"), "Path should contain day");
    }
}
