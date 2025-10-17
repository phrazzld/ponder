//! Generate AI reflections on journal entries.

use crate::ai::prompts::reflect_prompt;
use crate::ai::OllamaClient;
use crate::constants::DEFAULT_CHAT_MODEL;
use crate::crypto::temp::decrypt_to_temp;
use crate::crypto::SessionManager;
use crate::db::entries::get_entry_by_date;
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use chrono::NaiveDate;
use std::fs;
use tracing::{debug, info};

/// Generates a thoughtful reflection on a journal entry.
///
/// # Flow
///
/// 1. Look up entry metadata by date
/// 2. Decrypt the journal entry
/// 3. Send to LLM with reflection prompt
/// 4. Return the reflection text
///
/// Note: Currently returns reflection text only. Future versions may store
/// reflections in the database as encrypted reports.
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for encryption/decryption
/// * `ai_client` - Ollama client for chat completion
/// * `date` - Date of entry to reflect on
///
/// # Errors
///
/// Returns an error if:
/// - Entry for the date doesn't exist
/// - Session is locked
/// - Decryption fails
/// - Chat completion fails
pub fn reflect_on_entry(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    date: NaiveDate,
) -> AppResult<String> {
    info!("Generating reflection for entry: {}", date);

    // Ensure session is unlocked
    let passphrase = session.get_passphrase()?;

    // Get entry metadata
    let conn = db.get_conn()?;
    let entry = get_entry_by_date(&conn, date)?
        .ok_or_else(|| AppError::Journal(format!("No entry found for date: {}", date)))?;

    debug!("Found entry at: {:?}", entry.path);

    // Decrypt the entry
    let temp_path = decrypt_to_temp(&entry.path, passphrase)?;
    let content = fs::read_to_string(&temp_path)?;
    let _ = fs::remove_file(&temp_path); // Clean up temp file

    debug!("Decrypted entry ({} words)", entry.word_count);

    // Generate reflection using LLM
    let messages = reflect_prompt(&content);
    let reflection = ai_client.chat(DEFAULT_CHAT_MODEL, &messages)?;

    info!("Generated reflection for {}", date);
    Ok(reflection)
}

#[cfg(test)]
mod tests {
    // Integration tests in tests/ops_integration_tests.rs
}
