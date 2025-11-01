/*!
# Ponder v2.0 - AI-Powered Encrypted Journaling

Ponder v2.0 is a command-line tool for maintaining encrypted journal entries
with AI-powered insights, semantic search, and RAG query capabilities.

## Features

- **Edit**: Create and edit encrypted journal entries with automatic embedding generation
- **Ask**: Query your journal using RAG (Retrieval-Augmented Generation)
- **Reflect**: Generate AI reflections on specific journal entries
- **Search**: Semantic search over your encrypted journal entries
- **Lock**: Secure session management with passphrase protection

## Usage

```
ponder <COMMAND>

Commands:
  edit      Edit a journal entry with encryption
  ask       Query journal entries using AI (RAG)
  reflect   Generate AI reflection on a journal entry
  search    Semantic search over journal entries
  lock      Lock the encrypted session

Options:
  -v, --verbose           Enable verbose output
  --log-format <FORMAT>   Log output format [text|json]
  -h, --help              Print help
  -V, --version           Print version
```

## Configuration

Environment variables:
- `PONDER_DIR`: Journal directory (default: ~/Documents/rubberducks)
- `PONDER_EDITOR`: Editor command (default: vim)
- `PONDER_DB`: Database path (default: $PONDER_DIR/ponder.db)
- `PONDER_SESSION_TIMEOUT`: Session timeout in minutes (default: 30)
- `OLLAMA_URL`: Ollama API URL (default: http://127.0.0.1:11434)
*/

use chrono::{Datelike, Local};
use clap::Parser;
use ponder::cli::{CliArgs, ConverseArgs, EditArgs, PonderCommand};
use ponder::config::Config;
use ponder::constants::{self, DEFAULT_CHAT_MODEL, DEFAULT_EMBED_MODEL};
use ponder::crypto::SessionManager;
use ponder::db::Database;
use ponder::errors::{AppError, AppResult, DatabaseError};
use ponder::journal_core::DateSpecifier;
use ponder::journal_io;
use ponder::ops;
use ponder::setup::{ensure_model_available, ModelType};
use ponder::OllamaClient;
use tracing::{debug, info, info_span, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use uuid::Uuid;

/// Runs the core application logic with the given correlation ID and CLI arguments.
///
/// This function dispatches to the appropriate operation based on the subcommand:
/// - `edit`: Edit encrypted journal entries (v1.0 compatibility + v2.0 encryption)
/// - `ask`: RAG query over journal entries
/// - `reflect`: Generate AI reflection on an entry
/// - `search`: Semantic search over entries
/// - `lock`: Clear session passphrase
/// - None: Default to `edit today` for v1.0 compatibility
///
/// # Arguments
///
/// * `correlation_id` - The correlation ID for this application invocation
/// * `args` - The parsed CLI arguments
/// * `current_datetime` - The current date and time when the application started
///
/// # Returns
///
/// A Result that is Ok(()) if the application ran successfully,
/// or an AppError if an error occurred at any point in the flow.
fn run_application(
    correlation_id: &str,
    args: CliArgs,
    current_datetime: chrono::DateTime<Local>,
) -> AppResult<()> {
    let current_date = current_datetime.naive_local().date();

    // Create and enter the root span with correlation ID
    let root_span = info_span!(
        constants::TRACING_ROOT_SPAN_NAME,
        service_name = constants::TRACING_SERVICE_NAME,
        correlation_id = %correlation_id
    );
    let _guard = root_span.enter();

    info!("Starting ponder");
    debug!("CLI arguments: {:?}", args);

    if args.verbose {
        debug!("Verbose mode enabled");
    }

    // Load and validate configuration
    info!("Loading configuration");
    let config = Config::load()?;
    config.validate()?;

    // Dispatch based on command
    match args.command {
        Some(PonderCommand::Edit(edit_args)) => {
            cmd_edit(&config, edit_args, current_date, &current_datetime)
        }
        Some(PonderCommand::Ask(ask_args)) => cmd_ask(&config, ask_args),
        Some(PonderCommand::Reflect(reflect_args)) => {
            cmd_reflect(&config, reflect_args, current_date)
        }
        Some(PonderCommand::Summarize(summarize_args)) => {
            cmd_summarize(&config, summarize_args, current_date)
        }
        Some(PonderCommand::Summaries(summaries_args)) => cmd_summaries(&config, summaries_args),
        Some(PonderCommand::Search(search_args)) => cmd_search(&config, search_args),
        Some(PonderCommand::Converse(converse_args)) => cmd_converse(&config, &converse_args),
        Some(PonderCommand::Lock) => cmd_lock(&config),
        Some(PonderCommand::Backup(backup_args)) => cmd_backup(&config, backup_args),
        Some(PonderCommand::Restore(restore_args)) => cmd_restore(&config, restore_args),
        Some(PonderCommand::Reindex) => cmd_reindex(&config),
        Some(PonderCommand::CleanupV1(cleanup_args)) => cmd_cleanup_v1(&config, cleanup_args),
        Some(PonderCommand::Status) => cmd_status(&config),
        None => {
            // Default: edit today's entry (v1.0 compatibility)
            cmd_edit(
                &config,
                EditArgs {
                    retro: false,
                    reminisce: false,
                    date: None,
                    migrate: false,
                },
                current_date,
                &current_datetime,
            )
        }
    }
}

/// Opens database with passphrase retry logic.
///
/// Attempts to open the database, retrying up to MAX_PASSPHRASE_ATTEMPTS times
/// if the wrong passphrase is provided.
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `session` - Session manager
///
/// # Returns
///
/// Returns the opened database or error after max retries.
fn open_database_with_retry(config: &Config, session: &mut SessionManager) -> AppResult<Database> {
    const MAX_PASSPHRASE_ATTEMPTS: u32 = 3;

    // Detect first-run vs existing database
    let db_exists = config.db_path.exists();

    for attempt in 1..=MAX_PASSPHRASE_ATTEMPTS {
        // Get passphrase (prompts if locked)
        let passphrase = session.get_passphrase_or_prompt(db_exists)?;

        // Try to open database
        match Database::open(&config.db_path, passphrase) {
            Ok(db) => {
                info!("Database opened successfully");
                return Ok(db);
            }
            Err(AppError::Database(DatabaseError::WrongPassphrase)) => {
                // Wrong passphrase - lock session to force re-prompt
                session.lock();

                if attempt < MAX_PASSPHRASE_ATTEMPTS {
                    warn!(
                        "Incorrect passphrase, attempt {}/{}",
                        attempt, MAX_PASSPHRASE_ATTEMPTS
                    );
                    println!(
                        "\nIncorrect passphrase. Please try again (attempt {}/{}).\n",
                        attempt + 1,
                        MAX_PASSPHRASE_ATTEMPTS
                    );
                } else {
                    return Err(ponder::errors::CryptoError::MaxRetriesExceeded.into());
                }
            }
            Err(e) => {
                // Other error - propagate immediately
                return Err(e);
            }
        }
    }

    unreachable!("Loop should always return or error before reaching here")
}

/// Truncates text to max length with ellipsis if needed.
fn truncate_with_ellipsis(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        // Find a good break point (space) before max_len
        let truncated = &text[..max_len];
        let last_space = truncated.rfind(' ').unwrap_or(max_len);
        format!("{}...", truncated[..last_space].trim())
    }
}

/// Parses flexible date formats for summary viewing.
///
/// Supports:
/// - YYYY-MM-DD (full date)
/// - YYYY-MM (monthly, converts to YYYY-MM-01)
fn parse_flexible_date(date_str: &str) -> AppResult<String> {
    use chrono::NaiveDate;

    // Try full date format first
    if NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok() {
        return Ok(date_str.to_string());
    }

    // Try monthly format (YYYY-MM) - convert to YYYY-MM-01
    if date_str.len() == 7 && date_str.chars().nth(4) == Some('-') {
        let full_date = format!("{}-01", date_str);
        if NaiveDate::parse_from_str(&full_date, "%Y-%m-%d").is_ok() {
            return Ok(full_date);
        }
    }

    Err(AppError::Journal(format!(
        "Invalid date format: '{}'. Use YYYY-MM-DD or YYYY-MM",
        date_str
    )))
}

/// Displays a full summary with formatted output.
fn display_summary(
    db: &Database,
    session: &mut SessionManager,
    date_str: &str,
    level: ponder::db::summaries::SummaryLevel,
) -> AppResult<()> {
    use ponder::crypto::age::decrypt_with_passphrase;
    use ponder::db::summaries::{format_summary_date, get_summary};

    let conn = db.get_conn()?;
    let summary = get_summary(&conn, date_str, level)?
        .ok_or_else(|| AppError::Journal("Failed to fetch generated summary".to_string()))?;

    let passphrase = session.get_passphrase()?;
    let decrypted = decrypt_with_passphrase(&summary.summary_encrypted, passphrase)?;
    let content = String::from_utf8(decrypted)
        .map_err(|e| AppError::Journal(format!("Invalid UTF-8 in summary: {}", e)))?;

    // Display with nice formatting
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "{} {} Summary: {}",
        level.icon(),
        level.as_str().to_uppercase(),
        format_summary_date(date_str, level)
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("{}\n", content);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Metadata
    if let Some(sentiment) = summary.sentiment {
        print!("Sentiment: {:.2}", sentiment);
    }
    if let Some(word_count) = summary.word_count {
        print!("  |  {} words", word_count);
    }
    println!();
    if let Some(topics) = &summary.topics {
        if let Ok(topics_vec) = serde_json::from_str::<Vec<String>>(topics) {
            if !topics_vec.is_empty() {
                println!("Topics: {}", topics_vec.join(", "));
            }
        }
    }
    println!("Saved with ID: {}\n", summary.id);

    Ok(())
}

fn ensure_embedding_available(client: &OllamaClient) -> AppResult<()> {
    ensure_model_available(client, DEFAULT_EMBED_MODEL, ModelType::Embed, "Embedding")
}

/// Ensures the chat model is available, offering to install if missing.
///
/// This checks if the chat model is installed and prompts the user
/// to install it if not found.
///
/// # Arguments
///
/// * `client` - Ollama client instance
///
/// # Returns
///
/// Returns `Ok(())` if model is available, `Err` if unavailable and declined.
fn ensure_chat_available(client: &OllamaClient) -> AppResult<()> {
    ensure_model_available(client, DEFAULT_CHAT_MODEL, ModelType::Chat, "Chat")
}

/// Edit command: Edit journal entries with encryption and embedding.
fn cmd_edit(
    config: &Config,
    edit_args: EditArgs,
    current_date: chrono::NaiveDate,
    current_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    info!("Command: edit");

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&config.journal_dir)?;

    // v2.0: Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

    // Handle --migrate flag
    if edit_args.migrate {
        return cmd_migrate(config, &db, &mut session, Some(&ai_client));
    }

    // Detect v1.0 entries and auto-prompt for migration (one-time)
    let detection_result = ops::detect_migration_state(&config.journal_dir, &db)?;
    if detection_result.pending > 0 && !has_migration_been_prompted(&db)? {
        if prompt_migration(&detection_result)? {
            return cmd_migrate(config, &db, &mut session, Some(&ai_client));
        } else {
            // Mark that we've prompted (don't ask again)
            mark_migration_prompted(&db)?;
        }
    }

    // Parse date specifier from edit args
    let date_spec = DateSpecifier::from_cli_args(
        edit_args.retro,
        edit_args.reminisce,
        edit_args.date.as_deref(),
    )
    .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)))?;

    let dates_to_open = date_spec.resolve_dates(current_date);

    // Ensure embedding model is available (for automatic embedding generation)
    ensure_embedding_available(&ai_client)?;

    // Edit each entry
    for date in dates_to_open {
        info!("Editing entry for: {}", date);
        ops::edit_entry(
            config,
            &db,
            &mut session,
            &ai_client,
            date,
            current_datetime,
        )?;
    }

    info!("Entries edited successfully");
    Ok(())
}

/// Ask command: Query journal entries using RAG.
fn cmd_ask(config: &Config, ask_args: ponder::cli::AskArgs) -> AppResult<()> {
    info!("Command: ask");

    // Parse date range if provided
    let time_window = parse_date_range(&ask_args.from, &ask_args.to)?;

    // Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

    // Ensure models are available (embedding for search, chat for LLM)
    ensure_embedding_available(&ai_client)?;
    ensure_chat_available(&ai_client)?;

    // Query
    let answer = ops::ask_question(
        &db,
        &mut session,
        &ai_client,
        &ask_args.question,
        time_window,
    )?;

    // Output answer
    println!("{}", answer);

    Ok(())
}

/// Reflect command: Generate AI reflection on a journal entry.
fn cmd_reflect(
    config: &Config,
    reflect_args: ponder::cli::ReflectArgs,
    current_date: chrono::NaiveDate,
) -> AppResult<()> {
    info!("Command: reflect");

    // Parse date (default to today)
    let date = if let Some(date_str) = reflect_args.date {
        DateSpecifier::from_cli_args(false, false, Some(&date_str))
            .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)))?
            .resolve_dates(current_date)
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Journal("Failed to resolve date".to_string()))?
    } else {
        current_date
    };

    // Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

    // Ensure chat model is available
    ensure_chat_available(&ai_client)?;

    // Generate reflection
    let reflection = ops::reflect_on_entry(&db, &mut session, &ai_client, date)?;

    // Output reflection
    println!("\n{}\n", reflection);

    Ok(())
}

/// Summarize command: Generate AI-powered summaries (daily, weekly, monthly).
fn cmd_summarize(
    config: &Config,
    summarize_args: ponder::cli::SummarizeArgs,
    current_date: chrono::NaiveDate,
) -> AppResult<()> {
    use ponder::cli::SummaryPeriod;
    use ponder::db::summaries::SummaryLevel;

    info!("Command: summarize");

    // Parse date based on period
    let (date, year, month) = match summarize_args.period {
        SummaryPeriod::Daily => {
            let date = if let Some(date_str) = summarize_args.date {
                DateSpecifier::from_cli_args(false, false, Some(&date_str))
                    .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)))?
                    .resolve_dates(current_date)
                    .into_iter()
                    .next()
                    .ok_or_else(|| AppError::Journal("Failed to resolve date".to_string()))?
            } else {
                current_date
            };
            (Some(date), None, None)
        }
        SummaryPeriod::Weekly => {
            let end_date = if let Some(date_str) = summarize_args.date {
                DateSpecifier::from_cli_args(false, false, Some(&date_str))
                    .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)))?
                    .resolve_dates(current_date)
                    .into_iter()
                    .next()
                    .ok_or_else(|| AppError::Journal("Failed to resolve date".to_string()))?
            } else {
                // Default to last Sunday
                let days_since_sunday = current_date.weekday().num_days_from_sunday();
                current_date - chrono::Duration::days(days_since_sunday as i64)
            };
            (Some(end_date), None, None)
        }
        SummaryPeriod::Monthly => {
            let (year, month) = if let Some(date_str) = summarize_args.date {
                let date = DateSpecifier::from_cli_args(false, false, Some(&date_str))
                    .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)))?
                    .resolve_dates(current_date)
                    .into_iter()
                    .next()
                    .ok_or_else(|| AppError::Journal("Failed to resolve date".to_string()))?;
                (date.year(), date.month())
            } else {
                (current_date.year(), current_date.month())
            };
            (None, Some(year), Some(month))
        }
    };

    // Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

    // Ensure chat model is available
    ensure_chat_available(&ai_client)?;

    // Generate summary based on period
    let _summary_id = match summarize_args.period {
        SummaryPeriod::Daily => {
            let date = date.unwrap();
            println!("Generating daily summary for {}...", date);
            let id = ops::generate_daily_summary(
                &db,
                &mut session,
                &ai_client,
                &config.ai_models.summary_model,
                date,
            )?;
            println!(
                "✓ Daily summary for {} generated successfully (ID: {})",
                date, id
            );

            // Display the full summary immediately
            display_summary(&db, &mut session, &date.to_string(), SummaryLevel::Daily)?;
            id
        }
        SummaryPeriod::Weekly => {
            let end_date = date.unwrap();
            let start_date = end_date - chrono::Duration::days(6);
            println!(
                "Generating weekly summary for {} to {}...",
                start_date, end_date
            );
            let id = ops::generate_weekly_summary(
                &db,
                &mut session,
                &ai_client,
                &config.ai_models.summary_model,
                end_date,
            )?;
            println!(
                "✓ Weekly summary for week ending {} generated successfully (ID: {})",
                end_date, id
            );

            // Display the full summary immediately
            display_summary(
                &db,
                &mut session,
                &end_date.to_string(),
                SummaryLevel::Weekly,
            )?;
            id
        }
        SummaryPeriod::Monthly => {
            let year = year.unwrap();
            let month = month.unwrap();
            println!("Generating monthly summary for {}-{:02}...", year, month);
            let id = ops::generate_monthly_summary(
                &db,
                &mut session,
                &ai_client,
                &config.ai_models.summary_model,
                year,
                month,
            )?;
            println!(
                "✓ Monthly summary for {}-{:02} generated successfully (ID: {})",
                year, month, id
            );

            // Display the full summary immediately
            let date_str = format!("{}-{:02}-01", year, month);
            display_summary(&db, &mut session, &date_str, SummaryLevel::Monthly)?;
            id
        }
    };

    Ok(())
}

/// Summaries command: Browse and view past AI-generated summaries.
fn cmd_summaries(config: &Config, summaries_args: ponder::cli::SummariesArgs) -> AppResult<()> {
    use ponder::cli::SummaryPeriod;
    use ponder::crypto::age::decrypt_with_passphrase;
    use ponder::db::summaries::{get_summary, list_summaries, SummaryLevel};

    info!("Command: summaries");

    // Initialize session and database
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;

    // Check if showing a specific summary or listing summaries
    if let Some(date_str) = summaries_args.date {
        // Parse flexible date format (YYYY-MM-DD or YYYY-MM)
        let parsed_date = parse_flexible_date(&date_str)?;

        // Show specific summary - need to determine level
        // Try each level until we find a match
        let conn = db.get_conn()?;
        let passphrase = session.get_passphrase()?;

        let mut found = false;
        for level in [
            SummaryLevel::Daily,
            SummaryLevel::Weekly,
            SummaryLevel::Monthly,
        ] {
            if let Some(summary) = get_summary(&conn, &parsed_date, level)? {
                // Decrypt summary content
                let decrypted = decrypt_with_passphrase(&summary.summary_encrypted, passphrase)?;
                let content = String::from_utf8(decrypted).map_err(|e| {
                    AppError::Journal(format!("Invalid UTF-8 in summary content: {}", e))
                })?;

                // Display summary
                println!(
                    "\n{} Summary for {}",
                    level.as_str().to_uppercase(),
                    summary.date
                );
                println!("Created: {}", summary.created_at);
                if let Some(word_count) = summary.word_count {
                    println!("Word count: {}", word_count);
                }
                if let Some(sentiment) = summary.sentiment {
                    println!("Sentiment: {:.2}", sentiment);
                }
                if let Some(topics) = &summary.topics {
                    if let Ok(topics_vec) = serde_json::from_str::<Vec<String>>(topics) {
                        if !topics_vec.is_empty() {
                            println!("Topics: {}", topics_vec.join(", "));
                        }
                    }
                }
                println!("\n{}\n", content);

                found = true;
                break;
            }
        }

        if !found {
            println!("No summary found for date: {}", date_str);
        }
    } else {
        // List summaries
        let conn = db.get_conn()?;
        let summaries = if let Some(period) = summaries_args.period {
            let level = match period {
                SummaryPeriod::Daily => SummaryLevel::Daily,
                SummaryPeriod::Weekly => SummaryLevel::Weekly,
                SummaryPeriod::Monthly => SummaryLevel::Monthly,
            };
            list_summaries(&conn, level, summaries_args.limit)?
        } else {
            // Use smart hierarchical query by default
            use ponder::db::summaries::get_recent_summaries_hierarchical;
            get_recent_summaries_hierarchical(&conn, 1, 3, 10)?
        };

        if summaries.is_empty() {
            println!("No summaries found.");
        } else {
            // Group summaries by level for hierarchical display
            use std::collections::HashMap;
            let mut grouped: HashMap<SummaryLevel, Vec<_>> = HashMap::new();
            for summary in summaries {
                grouped
                    .entry(summary.level)
                    .or_insert_with(Vec::new)
                    .push(summary);
            }

            println!(
                "\nFound {} summaries:\n",
                grouped.values().map(|v| v.len()).sum::<usize>()
            );

            // Get passphrase for decrypting previews
            let passphrase = session.get_passphrase()?;

            // Display in hierarchical order: Monthly → Weekly → Daily
            for level in [
                SummaryLevel::Monthly,
                SummaryLevel::Weekly,
                SummaryLevel::Daily,
            ] {
                if let Some(summaries) = grouped.get(&level) {
                    println!(
                        "{} {} ({})",
                        level.icon(),
                        level.label_plural(),
                        summaries.len()
                    );
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                    for summary in summaries {
                        println!(
                            "{}",
                            ponder::db::summaries::format_summary_date(&summary.date, level)
                        );
                        if let Some(word_count) = summary.word_count {
                            print!("  Word count: {}", word_count);
                        }
                        if let Some(sentiment) = summary.sentiment {
                            print!("  Sentiment: {:.2}", sentiment);
                        }
                        println!();
                        if let Some(topics) = &summary.topics {
                            if let Ok(topics_vec) = serde_json::from_str::<Vec<String>>(topics) {
                                if !topics_vec.is_empty() {
                                    println!("  Topics: {}", topics_vec.join(", "));
                                }
                            }
                        }

                        // Decrypt and show content preview
                        if let Ok(decrypted) =
                            decrypt_with_passphrase(&summary.summary_encrypted, passphrase)
                        {
                            if let Ok(content) = String::from_utf8(decrypted) {
                                let preview = truncate_with_ellipsis(&content, 150);
                                println!("  Preview: {}", preview);
                            }
                        }

                        println!("  Created: {}", summary.created_at);
                        println!();
                    }
                }
            }

            // Footer with helpful hints
            println!("Filter by level: --period {{monthly|weekly|daily}}");
            println!("Show full summary: --date YYYY-MM-DD");
        }
    }

    Ok(())
}

/// Search command: Semantic search over journal entries.
fn cmd_search(config: &Config, search_args: ponder::cli::SearchArgs) -> AppResult<()> {
    info!("Command: search");

    // Parse date range if provided
    let time_window = parse_date_range(&search_args.from, &search_args.to)?;

    // Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

    // Ensure embedding model is available
    ensure_embedding_available(&ai_client)?;

    // Search
    let results = ops::search_entries(
        &db,
        &mut session,
        &ai_client,
        &search_args.query,
        search_args.limit,
        time_window,
    )?;

    // Output results
    if results.is_empty() {
        println!("No results found.");
    } else {
        println!("\nFound {} results:\n", results.len());
        for result in results {
            println!("Date: {}", result.date);
            println!("Score: {:.4}", result.score);
            println!("Excerpt: {}\n", result.excerpt);
            println!("---\n");
        }
    }

    Ok(())
}

/// Converse command: Interactive conversational AI for journal exploration.
fn cmd_converse(config: &Config, converse_args: &ConverseArgs) -> AppResult<()> {
    info!("Command: converse");

    // Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

    // Ensure models are available (embedding for context search, chat for conversation)
    ensure_embedding_available(&ai_client)?;
    ensure_chat_available(&ai_client)?;

    // Start conversation with context visibility (inverted from --no-context flag)
    let show_context = !converse_args.no_context;
    ops::start_conversation(&db, &mut session, &ai_client, config, show_context)?;

    Ok(())
}

/// Lock command: Clear session passphrase.
fn cmd_lock(config: &Config) -> AppResult<()> {
    info!("Command: lock");

    let mut session = SessionManager::new(config.session_timeout_minutes);
    session.lock();

    println!("Session locked successfully.");

    Ok(())
}

/// Backup command: Create encrypted backup of journal entries and database.
fn cmd_backup(config: &Config, backup_args: ponder::cli::BackupArgs) -> AppResult<()> {
    info!("Command: backup");

    // Initialize session and database
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;

    // Create backup (respects PONDER_DB config via config.db_path)
    let report = ops::create_backup(
        &db,
        &mut session,
        &config.journal_dir,
        &config.db_path,
        &backup_args.output,
    )?;

    println!("✓ Backup created successfully");
    println!("  Entries: {}", report.total_entries);
    println!("  Size: {} bytes", report.archive_size);
    println!("  Checksum: {}", report.checksum);
    println!("  Duration: {:.2}s", report.duration.as_secs_f64());
    println!("  Output: {:?}", backup_args.output);

    // Verify if requested
    if backup_args.verify {
        println!("\nVerifying backup...");
        let manifest = ops::verify_backup(&mut session, &backup_args.output)?;
        println!("✓ Backup verification passed");
        println!("  Verified entries: {}", manifest.entries.len());
        println!("  Database: {:?}", manifest.db_path);
    }

    Ok(())
}

/// Restore command: Restore from encrypted backup archive.
fn cmd_restore(config: &Config, restore_args: ponder::cli::RestoreArgs) -> AppResult<()> {
    info!("Command: restore");

    // Initialize session
    let mut session = SessionManager::new(config.session_timeout_minutes);

    // Restore backup (respects PONDER_DB config via config.db_path)
    let report = ops::restore_backup(
        &mut session,
        &restore_args.backup,
        &config.journal_dir,
        &config.db_path,
        restore_args.force,
    )?;

    println!("✓ Restore completed successfully");
    println!("  Entries restored: {}", report.entries_restored);
    println!("  Database size: {} bytes", report.db_size);
    println!("  Checksum: {}", report.checksum);
    println!("  Duration: {:.2}s", report.duration.as_secs_f64());
    println!("  Target: {:?}", config.journal_dir);

    Ok(())
}

/// Migration command: Migrate v1.0 plaintext entries to v2.0 encrypted format.
fn cmd_migrate(
    config: &Config,
    db: &Database,
    session: &mut SessionManager,
    ai_client: Option<&OllamaClient>,
) -> AppResult<()> {
    info!("Command: migrate");

    // Scan for v1.0 entries
    let v1_entries = ops::scan_v1_entries(&config.journal_dir)?;

    if v1_entries.is_empty() {
        println!("No v1.0 entries found to migrate.");
        return Ok(());
    }

    println!("Found {} v1.0 entries to migrate", v1_entries.len());
    println!();

    // Migrate all entries with progress
    let results = ops::migrate_all_entries(
        config,
        db,
        session,
        ai_client,
        v1_entries,
        Some(Box::new(print_progress)),
    )?;

    // Summary
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    println!();
    println!("✓ Migration completed");
    println!("  Successful: {}", successful);
    if failed > 0 {
        println!("  Failed: {}", failed);
    }

    Ok(())
}

/// Cleanup v1.0 command: Delete verified-migrated v1.0 entries.
fn cmd_cleanup_v1(config: &Config, cleanup_args: ponder::cli::CleanupV1Args) -> AppResult<()> {
    info!("Command: cleanup-v1");

    // Initialize session and database
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;

    // Scan for v1.0 entries
    let detection_result = ops::detect_migration_state(&config.journal_dir, &db)?;

    if detection_result.migrated_entries.is_empty() {
        println!("No migrated v1.0 entries found to clean up.");
        return Ok(());
    }

    println!(
        "Found {} verified-migrated v1.0 entries",
        detection_result.migrated_entries.len()
    );
    println!();

    // Confirm deletion unless --yes flag provided
    if !cleanup_args.yes {
        println!("This will permanently delete the following files:");
        for path in &detection_result.migrated_entries {
            println!("  {}", path.display());
        }
        println!();
        print!("Continue? [y/N]: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Delete files
    let mut deleted = 0;
    let mut failed = 0;

    for path in &detection_result.migrated_entries {
        match std::fs::remove_file(path) {
            Ok(()) => {
                deleted += 1;
                info!("Deleted: {:?}", path);
            }
            Err(e) => {
                failed += 1;
                warn!("Failed to delete {:?}: {}", path, e);
            }
        }
    }

    println!();
    println!("✓ Cleanup completed");
    println!("  Deleted: {}", deleted);
    if failed > 0 {
        println!("  Failed: {}", failed);
    }

    Ok(())
}

/// Regenerates embeddings for entries missing them.
fn cmd_reindex(config: &Config) -> AppResult<()> {
    info!("Command: reindex");

    // Initialize session and database
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;

    // Initialize Ollama client
    let ai_client = OllamaClient::new(&config.ollama_url);

    println!("🔍 Checking for entries missing embeddings...");
    println!();

    // Run reindex operation
    let report = ops::reindex_entries(&db, &mut session, &ai_client)?;

    if report.total == 0 {
        println!("✅ All entries already have embeddings");
        return Ok(());
    }

    println!();
    println!("✅ Reindex Complete");
    println!("  Total: {} entries", report.total);
    println!("  Success: {} entries", report.success);
    if report.failed > 0 {
        println!("  Failed: {} entries", report.failed);
    }
    println!("  Duration: {:.2}s", report.duration.as_secs_f64());

    Ok(())
}

/// Shows journal database health and statistics.
fn cmd_status(config: &Config) -> AppResult<()> {
    info!("Command: status");

    // Initialize session and database
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = open_database_with_retry(config, &mut session)?;

    // Create temporary status operation (we'll implement this next)
    // For now, just show basic stats
    let conn = db.get_conn()?;

    // Get stats
    let (embedded, total_entries) = ponder::db::entries::get_embedding_stats(&conn)?;
    let total_embeddings = ponder::db::embeddings::count_total_embeddings(&conn)?;
    let (oldest, newest) = ponder::db::entries::get_entry_date_range(&conn)?;
    let (migration_verified, migration_total) = db.get_migration_stats()?;

    // Calculate DB size
    let db_path = config.db_path.clone();
    let db_size_mb = std::fs::metadata(&db_path)?.len() as f64 / 1_048_576.0;

    // Display status
    println!("📊 Journal Status");
    println!();
    println!("Entries:");
    println!("  Total: {} entries", total_entries);
    println!(
        "  With embeddings: {} ({:.1}%)",
        embedded,
        if total_entries > 0 {
            (embedded as f64 / total_entries as f64) * 100.0
        } else {
            0.0
        }
    );
    if total_entries > embedded {
        println!(
            "  Without embeddings: {} ({:.1}%) ⚠️",
            total_entries - embedded,
            if total_entries > 0 {
                ((total_entries - embedded) as f64 / total_entries as f64) * 100.0
            } else {
                0.0
            }
        );
    }
    if let (Some(oldest_date), Some(newest_date)) = (oldest, newest) {
        println!("  Date range: {} to {}", oldest_date, newest_date);
    }
    println!();

    println!("Embeddings:");
    println!("  Total vectors: {} chunks", total_embeddings);
    if total_entries > 0 {
        println!(
            "  Average: {:.1} chunks/entry",
            total_embeddings as f64 / total_entries as f64
        );
    }
    println!();

    if migration_total > 0 {
        println!("Migration:");
        println!("  Completed: {} entries", migration_verified);
        println!(
            "  Pending: {} entries",
            migration_total - migration_verified
        );
        println!();
    }

    println!("Database:");
    println!("  Size: {:.1} MB", db_size_mb);
    println!("  Location: {}", db_path.display());
    println!();

    if total_entries > embedded {
        println!("⚠️  Run 'ponder reindex' to generate missing embeddings");
    }

    Ok(())
}

/// Prompt user for migration with yes/no choice.
fn prompt_migration(detection_result: &ops::MigrationDetectionResult) -> AppResult<bool> {
    println!();
    println!("📦 v1.0 Journal Entries Detected");
    println!();
    println!(
        "Found {} plaintext v1.0 entries that can be migrated to v2.0 encrypted format.",
        detection_result.pending
    );
    println!();
    println!("Migration will:");
    println!("  • Encrypt entries using age encryption");
    println!("  • Preserve all content (verified with checksums)");
    println!("  • Generate embeddings for AI features");
    println!("  • Keep original files (delete with 'ponder cleanup-v1' after verification)");
    println!();
    print!("Migrate now? [y/N]: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Print progress for migration.
fn print_progress(current: usize, total: usize, result: &ops::MigrationResult) {
    if result.success {
        println!("[{}/{}] ✓ Migrated: {}", current, total, result.date);
    } else {
        println!(
            "[{}/{}] ✗ Failed: {} - {}",
            current,
            total,
            result.date,
            result.error_message.as_deref().unwrap_or("Unknown error")
        );
    }
}

/// Check if migration prompt has been shown (to avoid repeated prompts).
fn has_migration_been_prompted(db: &Database) -> AppResult<bool> {
    // Check if migration_state exists (indicates we've prompted or started migration)
    Ok(db.get_migration_state()?.is_some())
}

/// Mark that migration prompt has been shown.
fn mark_migration_prompted(db: &Database) -> AppResult<()> {
    // Initialize migration state with 0 entries to mark prompt as shown
    // This prevents repeated prompts while user decides not to migrate yet
    match db.init_migration_state(0) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Already exists, that's fine
            Ok(())
        }
    }
}

/// Parse date range from optional string arguments.
fn parse_date_range(
    from: &Option<String>,
    to: &Option<String>,
) -> AppResult<Option<(chrono::NaiveDate, chrono::NaiveDate)>> {
    match (from, to) {
        (Some(from_str), Some(to_str)) => {
            let from_date = chrono::NaiveDate::parse_from_str(from_str, constants::DATE_FORMAT_ISO)
                .map_err(|e| AppError::Journal(format!("Invalid 'from' date: {}", e)))?;
            let to_date = chrono::NaiveDate::parse_from_str(to_str, constants::DATE_FORMAT_ISO)
                .map_err(|e| AppError::Journal(format!("Invalid 'to' date: {}", e)))?;
            Ok(Some((from_date, to_date)))
        }
        (Some(_), None) | (None, Some(_)) => Err(AppError::Journal(
            "Both --from and --to must be specified together".to_string(),
        )),
        (None, None) => Ok(None),
    }
}

/// The main entry point for the ponder application.
///
/// This function handles the application startup and error boundary:
/// 1. Obtains current date/time
/// 2. Parses command-line arguments
/// 3. Initializes logging/tracing
/// 4. Generates correlation ID for tracing
/// 5. Runs the core application logic
/// 6. Handles any errors with structured logging and user-friendly messages
fn main() {
    // Obtain current date/time once at the beginning
    let current_datetime = Local::now();

    // Parse command-line arguments first (needed for log format)
    let args = CliArgs::parse();

    // Initialize tracing/logging
    let use_json_logging = args.log_format == constants::LOG_FORMAT_JSON
        || std::env::var(constants::ENV_VAR_CI).is_ok();

    // Default to "warn" level for clean UX, "info" with --verbose flag
    let default_level = if args.verbose {
        constants::DEFAULT_LOG_LEVEL // "info" - show diagnostics
    } else {
        "warn" // Only warnings and errors on happy path
    };

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_level))
        .unwrap_or_else(|e| {
            eprintln!("Error: Invalid log level configuration: {}", e);
            std::process::exit(1);
        });

    let subscriber_builder = tracing_subscriber::registry().with(filter_layer);

    let _init_result = if use_json_logging {
        let json_layer = fmt::layer()
            .json()
            .with_timer(fmt::time::ChronoUtc::default())
            .with_current_span(true)
            .with_span_list(true)
            .flatten_event(true);
        subscriber_builder.with(json_layer).try_init()
    } else {
        let pretty_layer = fmt::layer().pretty().with_writer(std::io::stderr);
        subscriber_builder.with(pretty_layer).try_init()
    };

    // Generate correlation ID
    let correlation_id = Uuid::new_v4().to_string();

    // Run application
    match run_application(&correlation_id, args, current_datetime) {
        Ok(()) => {
            std::process::exit(0);
        }
        Err(error) => {
            tracing::error!(
                error = %error,
                error_chain = ?error,
                correlation_id = %correlation_id,
                "Application failed"
            );

            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }
}
