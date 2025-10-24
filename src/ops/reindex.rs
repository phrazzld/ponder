//! Reindex operation for regenerating embeddings.
//!
//! This module provides functionality to regenerate embeddings for journal entries
//! that are missing them or to reindex all entries (useful after model changes).

use crate::ai::chunking::chunk_text;
use crate::ai::OllamaClient;
use crate::constants::{DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, DEFAULT_EMBED_MODEL};
use crate::crypto::temp::decrypt_to_temp;
use crate::crypto::SessionManager;
use crate::db::embeddings::insert_embedding;
use crate::db::entries::{get_entries_without_embeddings, mark_embedded, Entry};
use crate::db::Database;
use crate::errors::AppResult;
use std::fs;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Report of a completed reindex operation.
#[derive(Debug, Clone)]
pub struct ReindexReport {
    /// Total number of entries processed
    pub total: usize,
    /// Number of successfully reindexed entries
    pub success: usize,
    /// Number of failed entries
    pub failed: usize,
    /// Duration of reindex operation
    pub duration: Duration,
}

/// Regenerates embeddings for entries missing them.
///
/// # Flow
///
/// 1. Query database for entries with `embedded_at IS NULL`
/// 2. For each entry:
///    - Decrypt entry content
///    - Chunk text
///    - Generate embeddings for each chunk
///    - Store in database
///    - Mark entry as embedded
/// 3. Return summary report
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for decryption passphrase
/// * `ai_client` - Ollama client for embedding generation
///
/// # Errors
///
/// Does not fail on individual entry errors - captures them and continues.
/// Only fails if database operations or passphrase retrieval fails.
pub fn reindex_entries(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
) -> AppResult<ReindexReport> {
    info!("Starting reindex operation");
    let start_time = Instant::now();

    let passphrase = session.get_passphrase()?;
    let conn = db.get_conn()?;

    // Get entries missing embeddings
    let entries = get_entries_without_embeddings(&conn)?;

    if entries.is_empty() {
        info!("No entries need reindexing");
        return Ok(ReindexReport {
            total: 0,
            success: 0,
            failed: 0,
            duration: start_time.elapsed(),
        });
    }

    eprintln!("🔄 Reindexing {} entries...", entries.len());

    let mut success_count = 0;
    let mut fail_count = 0;

    for (idx, entry) in entries.iter().enumerate() {
        eprintln!("[{}/{}] Processing: {}", idx + 1, entries.len(), entry.date);

        match regenerate_embeddings_for_entry(&conn, ai_client, entry, passphrase) {
            Ok(_) => {
                success_count += 1;
                debug!("Successfully reindexed entry {}", entry.date);
            }
            Err(e) => {
                fail_count += 1;
                warn!("Failed to reindex entry {}: {}", entry.date, e);
                eprintln!("  ⚠️  Failed: {}", e);
            }
        }
    }

    let duration = start_time.elapsed();
    info!(
        "Reindex complete: {}/{} successful in {:?}",
        success_count,
        entries.len(),
        duration
    );

    Ok(ReindexReport {
        total: entries.len(),
        success: success_count,
        failed: fail_count,
        duration,
    })
}

/// Regenerates embeddings for a single entry.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `ai_client` - Ollama client
/// * `entry` - Entry to reindex
/// * `passphrase` - Decryption passphrase
///
/// # Errors
///
/// Returns an error if:
/// - Decryption fails
/// - Embedding generation fails
/// - Database operations fail
fn regenerate_embeddings_for_entry(
    conn: &rusqlite::Connection,
    ai_client: &OllamaClient,
    entry: &Entry,
    passphrase: &age::secrecy::SecretString,
) -> AppResult<()> {
    debug!("Regenerating embeddings for entry {}", entry.id);

    // Decrypt to temp
    let encrypted_path = std::path::Path::new(&entry.path);
    let temp_path = decrypt_to_temp(encrypted_path, passphrase)?;

    // Read content
    let content = fs::read_to_string(&temp_path)?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    // Chunk the text
    let chunks = chunk_text(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);
    debug!("Generated {} chunks for entry {}", chunks.len(), entry.id);

    // Delete existing embeddings for this entry (if any)
    conn.execute(
        "DELETE FROM embeddings WHERE entry_id = ?",
        rusqlite::params![entry.id],
    )
    .map_err(crate::errors::DatabaseError::Sqlite)?;

    // Generate and store embeddings for each chunk
    for (idx, chunk) in chunks.iter().enumerate() {
        // Generate embedding
        let embedding_vec = ai_client.embed_with_retry(DEFAULT_EMBED_MODEL, chunk, 3)?;

        // Calculate chunk checksum
        let chunk_checksum = blake3::hash(chunk.as_bytes()).to_hex().to_string();

        // Store in database
        insert_embedding(conn, entry.id, idx, &embedding_vec, &chunk_checksum)?;
    }

    // Mark entry as embedded
    mark_embedded(conn, entry.id)?;

    debug!("Stored {} embeddings for entry {}", chunks.len(), entry.id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reindex_report_creation() {
        let report = ReindexReport {
            total: 10,
            success: 9,
            failed: 1,
            duration: Duration::from_secs(60),
        };

        assert_eq!(report.total, 10);
        assert_eq!(report.success, 9);
        assert_eq!(report.failed, 1);
    }
}
