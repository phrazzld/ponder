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
- `journal_logic`: Core journal functionality

## Usage Example

```rust,no_run
use ponder::{Config, DateSpecifier};
use ponder::journal_logic;

fn main() -> ponder::AppResult<()> {
    // Load configuration
    let config = Config::load()?;

    // Ensure journal directory exists
    journal_logic::ensure_journal_directory_exists(&config.journal_dir)?;

    // Open today's journal entry
    journal_logic::open_journal_entries(&config, &DateSpecifier::Today)
}
```
*/

/// Command-line interface for parsing and handling user arguments
pub mod cli;
/// Configuration loading and management
pub mod config;
/// Error types and utilities for error handling
pub mod errors;
/// Core journal functionality (deprecated, use journal_logic instead)
pub mod journal;
/// Journal functionality (new implementation)
pub mod journal_logic;

// Re-export important types for convenience
pub use cli::CliArgs;
pub use config::Config;
pub use errors::{AppError, AppResult};
pub use journal_logic::DateSpecifier;
