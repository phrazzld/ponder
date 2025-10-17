//! Edit journal entries with encryption and embedding.

use crate::ai::chunking::chunk_text;
use crate::ai::OllamaClient;
use crate::config::Config;
use crate::constants::{DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, DEFAULT_EMBED_MODEL};
use crate::crypto::temp::{decrypt_to_temp, encrypt_from_temp};
use crate::crypto::SessionManager;
use crate::db::entries::upsert_entry;
use crate::db::embeddings::insert_embedding;
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
    let entry_id = upsert_entry(
        &conn,
        &encrypted_path,
        date,
        &new_checksum,
        word_count,
    )?;

    // Generate and store embeddings if content changed
    if content_changed {
        info!("Content changed, generating embeddings...");
        generate_and_store_embeddings(&conn, ai_client, entry_id, &encrypted_path, passphrase)?;
    } else {
        debug!("Content unchanged, skipping embedding generation");
    }

    Ok(())
}

/// Get the encrypted entry path for a given date (YYYY/MM/DD.md.age).
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
        debug!("Generating embedding for chunk {}/{}", idx + 1, chunks.len());

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
    // Integration tests in tests/ops_integration_tests.rs
}
