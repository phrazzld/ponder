/*!
# Ponder

Ponder is a simple journaling tool for daily reflections, designed to help users maintain
a journal with minimal friction. It provides functionality for creating and viewing daily
journal entries, as well as reviewing past entries.

## Core Features

- Create and edit today's journal entry
- Review entries from the past week (retro mode)
- Review entries from significant past time intervals like 1 month ago, 3 months ago, etc. (reminisce mode)
- Open entries for specific dates
- Customizable editor and journal directory

## Architecture

The codebase follows a modular architecture with clear separation of concerns:

- `cli`: Command-line interface handling using clap
- `config`: Configuration loading and validation
- `errors`: Error handling infrastructure
- `journal_core`: Core journal logic without I/O operations
- `journal_io`: Journal I/O operations and file management

## Usage Example

```rust,no_run
use ponder::{Config, DateSpecifier, AppResult, journal_io};
use chrono::Local;

fn main() -> AppResult<()> {
    // Load configuration
    let config = Config::load()?;

    // Get the current date/time once
    let current_datetime = Local::now();
    let today = current_datetime.naive_local().date();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&config.journal_dir)?;

    // Option 1: Using DateSpecifier and edit_journal_entries (with file locking)
    let date_spec = DateSpecifier::Today;
    let dates = date_spec.resolve_dates(today);
    journal_io::edit_journal_entries(&config, &dates, &current_datetime)?;

    // Option 2: Initialize entry explicitly before opening
    let entry_path = journal_io::initialize_journal_entry(&config.journal_dir, today, &current_datetime)?;
    // Then open it with your preferred method...

    Ok(())
}
```
*/

/// AI operations for journal insights and semantic search
pub mod ai;
/// Command-line interface for parsing and handling user arguments
pub mod cli;
/// Configuration loading and management
pub mod config;
/// Centralized constants used throughout the application
pub mod constants;
/// Cryptographic operations for journal encryption and security
pub mod crypto;
/// Database operations for journal metadata and embeddings
pub mod db;
/// Error types and utilities for error handling
pub mod errors;
/// Core journal functionality
pub mod journal_core;
/// Journal I/O operations
pub mod journal_io;

// Re-export important types for convenience
pub use ai::OllamaClient;
pub use cli::CliArgs;
pub use config::Config;
pub use db::Database;
pub use errors::{AIError, AppError, AppResult, DatabaseError, LockError};
pub use journal_core::DateSpecifier;
