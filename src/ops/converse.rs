//! Conversational interface for interactive journal exploration.
//!
//! This module provides a conversational AI interface where users can explore
//! their journal entries through natural dialogue. The AI uses Chain-of-Thought
//! reasoning to analyze patterns, answer questions, and provide insights based
//! on the user's actual journal content.

use crate::ai::chunking::chunk_text;
use crate::ai::prompts::COT_SYSTEM_PROMPT;
use crate::ai::{Message, OllamaClient};
use crate::config::Config;
use crate::constants::{DEFAULT_CHUNK_OVERLAP, DEFAULT_CHUNK_SIZE, DEFAULT_EMBED_MODEL};
use crate::crypto::temp::decrypt_to_temp;
use crate::crypto::SessionManager;
use crate::db::embeddings::search_similar_chunks;
use crate::db::Database;
use crate::errors::AppResult;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use tracing::{debug, info};

/// Starts an interactive conversational session with the AI.
///
/// Users can ask questions about their journal, explore patterns, and have
/// natural conversations about their entries. The AI maintains conversation
/// history and assembles relevant context from the journal for each response.
///
/// # Flow
///
/// 1. Display welcome message and instructions
/// 2. Read user input in a loop
/// 3. For each question:
///    - Assemble relevant journal context via RAG
///    - Build prompt with CoT system message + conversation history + context
///    - Stream AI response token by token
///    - Add question and response to conversation history
/// 4. Exit on "quit", "exit", or empty input
///
/// # Arguments
///
/// * `db` - Database connection for vector search
/// * `session` - Session manager for decryption
/// * `ai_client` - Ollama client for embeddings and chat
/// * `_config` - Configuration (reserved for future use)
///
/// # Errors
///
/// Returns an error if:
/// - Session is locked
/// - Ollama is unreachable
/// - Database query fails
/// - Decryption fails
///
/// # Example
///
/// ```no_run
/// # use ponder::ops::converse::start_conversation;
/// # use ponder::{Config, Database, SessionManager, ai::OllamaClient};
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::load()?;
/// let mut session = SessionManager::new(std::time::Duration::from_secs(1800));
/// let db = Database::open(&config.database_path, session.get_passphrase()?)?;
/// let ai_client = OllamaClient::new(&config.ollama_url);
///
/// start_conversation(&db, &mut session, &ai_client, &config)?;
/// # Ok(())
/// # }
/// ```
pub fn start_conversation(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    _config: &Config,
) -> AppResult<()> {
    info!("Starting conversational interface");

    // Print welcome message
    println!("\n🤖 Ponder Conversational Assistant");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Ask me anything about your journal entries!");
    println!("I'll think through your questions step-by-step and cite specific entries.");
    println!();
    println!("Commands:");
    println!("  • Type your question and press Enter");
    println!("  • 'quit' or 'exit' to end the conversation");
    println!("  • Empty input to end");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize conversation history with system message
    let mut conversation_history: Vec<Message> = vec![Message::system(COT_SYSTEM_PROMPT)];

    // Main conversation loop
    loop {
        // Prompt for user input
        print!("You: ");
        io::stdout().flush()?;

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        // Check for exit conditions
        if user_input.is_empty()
            || user_input.eq_ignore_ascii_case("quit")
            || user_input.eq_ignore_ascii_case("exit")
        {
            println!("\n👋 Goodbye! Thanks for the conversation.");
            break;
        }

        // Assemble context from journal entries
        debug!("Assembling context for question: {}", user_input);
        let context_chunks =
            match assemble_conversation_context(db, session, ai_client, user_input, 10) {
                Ok(chunks) => chunks,
                Err(e) => {
                    eprintln!("\n❌ Error assembling context: {}", e);
                    eprintln!("   Continuing without journal context...\n");
                    Vec::new()
                }
            };

        // Build user message with context
        let user_message = if context_chunks.is_empty() {
            format!("Question: {}", user_input)
        } else {
            let context_str = context_chunks
                .iter()
                .map(|(date, excerpt)| format!("📝 Entry from {}:\n{}", date, excerpt))
                .collect::<Vec<_>>()
                .join("\n\n");

            format!(
                "Here are some relevant excerpts from my journal:\n\n{}\n\nQuestion: {}",
                context_str, user_input
            )
        };

        // Add user message to history
        conversation_history.push(Message::user(&user_message));

        // Get AI response (non-streaming for MVP - streaming can be added later)
        print!("\n🤖 Assistant: ");
        io::stdout().flush()?;

        let response = match ai_client.chat(&_config.ai_models.chat_model, &conversation_history) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("\n❌ Error getting AI response: {}", e);
                eprintln!("   Make sure Ollama is running: ollama serve");
                eprintln!(
                    "   And the model is available: ollama pull {}\n",
                    _config.ai_models.chat_model
                );
                conversation_history.pop(); // Remove user message since we got no response
                continue;
            }
        };

        println!("{}\n", response);

        // Add assistant response to history
        conversation_history.push(Message::assistant(&response));

        // Keep conversation history manageable (last 20 messages + system message)
        if conversation_history.len() > 21 {
            // Keep system message (index 0) and most recent 20 messages
            let system_msg = conversation_history[0].clone();
            let recent_messages: Vec<Message> = conversation_history
                .drain(conversation_history.len() - 20..)
                .collect();
            conversation_history = vec![system_msg];
            conversation_history.extend(recent_messages);
            debug!("Trimmed conversation history to 20 most recent messages");
        }
    }

    Ok(())
}

/// Assembles relevant context from journal entries for a conversational query.
///
/// Uses vector similarity search to find relevant journal excerpts, decrypts them,
/// and returns date-excerpt pairs suitable for inclusion in conversation prompts.
///
/// # Flow
///
/// 1. Generate embedding for the query
/// 2. Search for similar chunks via vector search
/// 3. Group chunks by entry to minimize decryptions
/// 4. Decrypt entries and extract matching chunks
/// 5. Return (date, excerpt) pairs
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for decryption
/// * `ai_client` - Ollama client for generating query embedding
/// * `query` - User's question or prompt
/// * `limit` - Maximum number of chunks to return
///
/// # Returns
///
/// Vector of (date, excerpt) pairs, where date is the entry date and excerpt
/// is the relevant text chunk from that entry.
///
/// # Errors
///
/// Returns an error if:
/// - Embedding generation fails
/// - Vector search fails
/// - Decryption fails
pub fn assemble_conversation_context(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    query: &str,
    limit: usize,
) -> AppResult<Vec<(NaiveDate, String)>> {
    // Ensure session is unlocked
    let passphrase = session.get_passphrase()?;

    // Generate embedding for the query
    debug!("Generating embedding for conversational query");
    let query_embedding = ai_client.embed_with_retry(DEFAULT_EMBED_MODEL, query, 3)?;

    // Search for similar chunks
    let conn = db.get_conn()?;
    let similar_chunks = search_similar_chunks(&conn, &query_embedding, limit)?;

    if similar_chunks.is_empty() {
        debug!("No relevant journal entries found");
        return Ok(Vec::new());
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
        // Get entry metadata to find the file path and date
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
        crate::crypto::temp::secure_delete(&temp_path)?;

        // Chunk the content using same algorithm as embedding
        let chunks = chunk_text(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);

        // Extract requested chunks
        for &chunk_idx in &chunk_indices {
            if let Some(chunk) = chunks.get(chunk_idx) {
                context_chunks.push((entry.date, chunk.to_string()));
            }
        }
    }

    Ok(context_chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entries::upsert_entry;
    use age::secrecy::SecretString;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_assemble_context_no_entries() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();
        let mut session = SessionManager::new(30); // 30 minutes
        session.unlock(passphrase.clone());

        let ollama_url =
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        let ai_client = OllamaClient::new(&ollama_url);

        let result = assemble_conversation_context(&db, &mut session, &ai_client, "test query", 5);

        // Should succeed but return empty context
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_assemble_context_limits_results() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Add a test entry
        let conn = db.get_conn().unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let path = PathBuf::from("/tmp/test.md");
        upsert_entry(&conn, &path, date, "abc123", 100).unwrap();

        let mut session = SessionManager::new(30); // 30 minutes
        session.unlock(passphrase);

        let ollama_url =
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        let ai_client = OllamaClient::new(&ollama_url);

        // Should not panic even with embeddings missing
        let result = assemble_conversation_context(&db, &mut session, &ai_client, "test query", 3);

        // Will fail to find similar chunks (no embeddings), but shouldn't panic
        assert!(result.is_ok());
    }
}
