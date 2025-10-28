//! Summary generation operations for journal entries.
//!
//! This module provides functions for generating AI-powered summaries at different
//! granularities (daily, weekly, monthly). Summaries are encrypted and stored in the
//! database for later retrieval.

use crate::ai::prompts::daily_summary_prompt;
use crate::ai::OllamaClient;
use crate::constants::DEFAULT_CHAT_MODEL;
use crate::crypto::age::{decrypt_with_passphrase, encrypt_with_passphrase};
use crate::crypto::SessionManager;
use crate::db::entries::get_entry_by_date;
use crate::db::summaries::{upsert_summary, SummaryLevel};
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use chrono::NaiveDate;
use std::fs;
use tracing::{debug, info};

/// Generates a daily summary for a journal entry.
///
/// # Flow
///
/// 1. Retrieve entry metadata from database
/// 2. Decrypt entry file to read content
/// 3. Generate summary using AI
/// 4. Extract topics and sentiment analysis
/// 5. Encrypt summary
/// 6. Store encrypted summary in database
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for encryption/decryption
/// * `ai_client` - Ollama client for summary generation
/// * `date` - Date of the entry to summarize
///
/// # Returns
///
/// The ID of the stored summary record.
///
/// # Errors
///
/// Returns an error if:
/// - Session is locked
/// - Entry doesn't exist for the date
/// - Entry file can't be read
/// - Summary generation fails
/// - Encryption fails
/// - Database operation fails
pub fn generate_daily_summary(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    date: NaiveDate,
) -> AppResult<i64> {
    info!("Generating daily summary for {}", date);

    // Ensure session is unlocked
    let passphrase = session.get_passphrase()?;

    // Get entry metadata
    let conn = db.get_conn()?;
    let entry = get_entry_by_date(&conn, date)?
        .ok_or_else(|| AppError::Journal(format!("No entry found for date {}", date)))?;

    debug!("Found entry at path: {:?}", entry.path);

    // Read and decrypt entry content
    let encrypted_content = fs::read(&entry.path)?;
    let decrypted_content = decrypt_with_passphrase(&encrypted_content, passphrase)?;
    let content = String::from_utf8(decrypted_content)
        .map_err(|e| AppError::Journal(format!("Invalid UTF-8 in entry content: {}", e)))?;

    debug!("Decrypted entry content ({} chars)", content.len());

    // Generate summary using AI
    let messages = daily_summary_prompt(&content);
    let summary_text = ai_client.chat(DEFAULT_CHAT_MODEL, &messages)?;

    info!("Generated summary ({} chars)", summary_text.len());

    // Extract topics
    let topics = ai_client.extract_topics(&content)?;
    let topics_json = if topics.is_empty() {
        None
    } else {
        Some(
            serde_json::to_string(&topics)
                .map_err(|e| AppError::Journal(format!("Failed to serialize topics: {}", e)))?,
        )
    };

    debug!("Extracted {} topics", topics.len());

    // Analyze sentiment
    let sentiment = ai_client.analyze_sentiment(&content)?;
    debug!("Sentiment score: {}", sentiment);

    // Encrypt summary
    let encrypted_summary = encrypt_with_passphrase(summary_text.as_bytes(), passphrase)?;

    // Store in database
    let word_count = summary_text.split_whitespace().count() as i64;
    let summary_id = upsert_summary(
        &conn,
        &date.to_string(),
        SummaryLevel::Daily,
        &encrypted_summary,
        topics_json.as_deref(),
        Some(sentiment as f64),
        Some(word_count),
    )?;

    info!("Daily summary stored with id {}", summary_id);
    Ok(summary_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entries::upsert_entry;
    use age::secrecy::SecretString;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_generate_daily_summary_signature() {
        // Unit test verifying function signature
        // Integration tests with actual Ollama in tests/ops_integration_tests.rs
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test".to_string());

        let db = Database::open(&db_path, &passphrase).unwrap();
        let mut session = SessionManager::new(60);
        session.unlock(passphrase.clone());
        let client = OllamaClient::new("http://localhost:11434");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // Just verify the function exists with correct signature
        let _result: Result<i64, _> = generate_daily_summary(&db, &mut session, &client, date);
    }
}
