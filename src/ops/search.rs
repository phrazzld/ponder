//! Semantic search over journal entries.

use crate::ai::chunking::chunk_text;
use crate::ai::OllamaClient;
use crate::constants::{DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, DEFAULT_EMBED_MODEL};
use crate::crypto::temp::decrypt_to_temp;
use crate::crypto::SessionManager;
use crate::db::embeddings::search_similar_chunks;
use crate::db::Database;
use crate::errors::AppResult;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::fs;
use tracing::{debug, info};

/// Search result containing a matching journal entry excerpt.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Date of the journal entry
    pub date: NaiveDate,
    /// Decrypted excerpt from the entry
    pub excerpt: String,
    /// Similarity score (0.0-1.0, higher is better)
    pub score: f32,
}

/// Performs semantic search over journal entries.
///
/// # Flow
///
/// 1. Generate embedding for the search query
/// 2. Vector search for similar entry chunks
/// 3. Decrypt matching entry files
/// 4. Extract relevant chunks from decrypted content
/// 5. Format and return results with similarity scores and dates
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for decryption
/// * `ai_client` - Ollama client for embeddings
/// * `query` - Search query
/// * `limit` - Maximum number of results to return
/// * `time_window` - Optional date range to constrain search (not yet implemented)
///
/// # Errors
///
/// Returns an error if:
/// - Session is locked
/// - Embedding generation fails
/// - Database query fails
/// - Decryption fails
pub fn search_entries(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    query: &str,
    limit: usize,
    _time_window: Option<(NaiveDate, NaiveDate)>,
) -> AppResult<Vec<SearchResult>> {
    info!("Searching for: {}", query);

    // Ensure session is unlocked
    let passphrase = session.get_passphrase()?;

    // Generate embedding for the query
    debug!("Generating embedding for search query");
    let query_embedding = ai_client.embed(DEFAULT_EMBED_MODEL, query)?;

    // Search for similar chunks
    let conn = db.get_conn()?;
    let similar_chunks = search_similar_chunks(&conn, &query_embedding, limit)?;

    if similar_chunks.is_empty() {
        info!("No matching entries found");
        return Ok(Vec::new());
    }

    debug!("Found {} similar chunks", similar_chunks.len());

    // Group chunks by entry_id to minimize decryptions
    let mut chunks_by_entry: HashMap<i64, Vec<(usize, f32)>> = HashMap::new();
    for chunk in &similar_chunks {
        chunks_by_entry
            .entry(chunk.entry_id)
            .or_default()
            .push((chunk.chunk_idx, chunk.similarity));
    }

    // Decrypt entries and extract chunks
    let mut results = Vec::new();

    for (entry_id, chunk_data) in chunks_by_entry {
        // Get entry metadata
        let entry = conn
            .query_row(
                "SELECT id, path, date, checksum, word_count, updated_at, embedded_at FROM entries WHERE id = ?1",
                [entry_id],
                |row| {
                    Ok(crate::db::entries::Entry {
                        id: row.get(0)?,
                        path: std::path::PathBuf::from(row.get::<_, String>(1)?),
                        date: NaiveDate::parse_from_str(&row.get::<_, String>(2)?, "%Y-%m-%d")
                            .map_err(|e| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    2,
                                    rusqlite::types::Type::Text,
                                    Box::new(e),
                                )
                            })?,
                        checksum: row.get(3)?,
                        word_count: row.get::<_, i64>(4)? as usize,
                        updated_at: row.get(5)?,
                        embedded_at: row.get(6)?,
                    })
                },
            )
            .map_err(crate::errors::DatabaseError::Sqlite)?;

        // Decrypt the entry
        let temp_path = decrypt_to_temp(&entry.path, passphrase)?;
        let content = fs::read_to_string(&temp_path)?;
        let _ = fs::remove_file(&temp_path); // Clean up temp file

        // Chunk the content using same algorithm as embedding
        let chunks = chunk_text(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);

        // Extract requested chunks with their scores
        for (chunk_idx, score) in chunk_data {
            if let Some(chunk) = chunks.get(chunk_idx) {
                results.push(SearchResult {
                    date: entry.date,
                    excerpt: chunk.clone(),
                    score,
                });
            }
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    info!("Returning {} search results", results.len());
    Ok(results)
}

#[cfg(test)]
mod tests {
    // Integration tests in tests/ops_integration_tests.rs
}
