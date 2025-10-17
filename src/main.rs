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

use chrono::Local;
use clap::Parser;
use ponder::cli::{CliArgs, EditArgs, PonderCommand};
use ponder::config::Config;
use ponder::constants::{self, DEFAULT_CHAT_MODEL, DEFAULT_EMBED_MODEL};
use ponder::crypto::SessionManager;
use ponder::db::Database;
use ponder::errors::{AppError, AppResult};
use ponder::journal_core::DateSpecifier;
use ponder::journal_io;
use ponder::ops;
use ponder::setup::ensure_model_available;
use ponder::OllamaClient;
use tracing::{debug, info, info_span};
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
        Some(PonderCommand::Search(search_args)) => cmd_search(&config, search_args),
        Some(PonderCommand::Lock) => cmd_lock(&config),
        None => {
            // Default: edit today's entry (v1.0 compatibility)
            cmd_edit(
                &config,
                EditArgs {
                    retro: false,
                    reminisce: false,
                    date: None,
                },
                current_date,
                &current_datetime,
            )
        }
    }
}

/// Ensures the embedding model is available, offering to install if missing.
///
/// This checks if the embedding model is installed and prompts the user
/// to install it if not found.
///
/// # Arguments
///
/// * `client` - Ollama client instance
///
/// # Returns
///
/// Returns `Ok(())` if model is available, `Err` if unavailable and declined.
fn ensure_embedding_available(client: &OllamaClient) -> AppResult<()> {
    ensure_model_available(client, DEFAULT_EMBED_MODEL, "Embedding")
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
    ensure_model_available(client, DEFAULT_CHAT_MODEL, "Chat")
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

    // Parse date specifier from edit args
    let date_spec = DateSpecifier::from_cli_args(
        edit_args.retro,
        edit_args.reminisce,
        edit_args.date.as_deref(),
    )
    .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)))?;

    let dates_to_open = date_spec.resolve_dates(current_date);

    // v2.0: Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = Database::open(&config.db_path, session.get_passphrase()?)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

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
    let db = Database::open(&config.db_path, session.get_passphrase()?)?;
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
    let db = Database::open(&config.db_path, session.get_passphrase()?)?;
    let ai_client = OllamaClient::new(&config.ollama_url);

    // Ensure chat model is available
    ensure_chat_available(&ai_client)?;

    // Generate reflection
    let reflection = ops::reflect_on_entry(&db, &mut session, &ai_client, date)?;

    // Output reflection
    println!("\n{}\n", reflection);

    Ok(())
}

/// Search command: Semantic search over journal entries.
fn cmd_search(config: &Config, search_args: ponder::cli::SearchArgs) -> AppResult<()> {
    info!("Command: search");

    // Parse date range if provided
    let time_window = parse_date_range(&search_args.from, &search_args.to)?;

    // Initialize session, database, and AI client
    let mut session = SessionManager::new(config.session_timeout_minutes);
    let db = Database::open(&config.db_path, session.get_passphrase()?)?;
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

/// Lock command: Clear session passphrase.
fn cmd_lock(config: &Config) -> AppResult<()> {
    info!("Command: lock");

    let mut session = SessionManager::new(config.session_timeout_minutes);
    session.lock();

    println!("Session locked successfully.");

    Ok(())
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

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(constants::DEFAULT_LOG_LEVEL))
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
