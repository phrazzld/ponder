//! RAG (Retrieval-Augmented Generation) query operations.

use crate::ai::chunking::chunk_text;
use crate::ai::prompts::ask_prompt;
use crate::ai::OllamaClient;
use crate::constants::{DEFAULT_CHAT_MODEL, DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, DEFAULT_EMBED_MODEL};
use crate::crypto::temp::decrypt_to_temp;
use crate::crypto::SessionManager;
use crate::db::embeddings::search_similar_chunks;
use crate::db::Database;
use crate::errors::AppResult;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::fs;
use tracing::{debug, info};

/// Answers a question using journal entries as context (RAG pipeline).
///
/// # Flow
///
/// 1. Generate embedding for the question
/// 2. Vector search for relevant entry chunks (top 5)
/// 3. Decrypt matching entry files
/// 4. Extract relevant chunks from decrypted content
/// 5. Send to LLM with context
/// 6. Return generated answer
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for decryption
/// * `ai_client` - Ollama client for embeddings and chat
/// * `question` - User's question
/// * `time_window` - Optional date range to constrain search (not yet implemented)
///
/// # Errors
///
/// Returns an error if:
/// - Session is locked
/// - Embedding generation fails
/// - Database query fails
/// - Decryption fails
/// - Chat completion fails
pub fn ask_question(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    question: &str,
    _time_window: Option<(NaiveDate, NaiveDate)>,
) -> AppResult<String> {
    info!("Answering question: {}", question);

    // Ensure session is unlocked
    let passphrase = session.get_passphrase()?;

    // Generate embedding for the question
    debug!("Generating embedding for question");
    let query_embedding = ai_client.embed(DEFAULT_EMBED_MODEL, question)?;

    // Search for similar chunks
    let conn = db.get_conn()?;
    let similar_chunks = search_similar_chunks(&conn, &query_embedding, 5)?;

    if similar_chunks.is_empty() {
        info!("No relevant entries found");
        let messages = ask_prompt(question, &[]);
        return ai_client.chat(DEFAULT_CHAT_MODEL, &messages);
    }

    debug!("Found {} similar chunks", similar_chunks.len());

    // Group chunks by entry_id to minimize decryptions
    let mut chunks_by_entry: HashMap<i64, Vec<usize>> = HashMap::new();
    for chunk in &similar_chunks {
        chunks_by_entry
            .entry(chunk.entry_id)
            .or_default()
            .push(chunk.chunk_idx);
    }

    // Decrypt entries and extract chunks
    let mut context_chunks = Vec::new();

    for (entry_id, chunk_indices) in chunks_by_entry {
        // Get entry metadata to find the file path
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

        // Extract requested chunks
        for &chunk_idx in &chunk_indices {
            if let Some(chunk) = chunks.get(chunk_idx) {
                context_chunks.push(format!(
                    "[Entry from {}]\n{}",
                    entry.date.format("%Y-%m-%d"),
                    chunk
                ));
            }
        }
    }

    debug!("Extracted {} context chunks", context_chunks.len());

    // Generate answer using LLM
    let messages = ask_prompt(question, &context_chunks);
    let answer = ai_client.chat(DEFAULT_CHAT_MODEL, &messages)?;

    info!("Generated answer");
    Ok(answer)
}

#[cfg(test)]
mod tests {
    // Integration tests in tests/ops_integration_tests.rs
}
