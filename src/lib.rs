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
- `editor`: Editor abstraction for opening journal files
- `errors`: Error handling infrastructure
- `journal`: Core journal functionality with dependency injection

## Usage Example

```rust
use ponder::{Config, JournalService, DateSpecifier};
use ponder::journal::io::FileSystemIO;
use ponder::editor::SystemEditor;

fn main() -> ponder::AppResult<()> {
    // Load configuration
    let config = Config::load()?;

    // Create components
    let io = Box::new(FileSystemIO {
        journal_dir: config.journal_dir.clone(),
    });
    let editor = Box::new(SystemEditor {
        editor_cmd: config.editor.clone(),
    });

    // Create service and open today's journal entry
    let journal_service = JournalService::new(config, io, editor)?;
    journal_service.open_entries(&DateSpecifier::Today)
}
```
*/

/// Command-line interface for parsing and handling user arguments
pub mod cli;
/// Configuration loading and management
pub mod config;
/// Editor abstraction for opening journal files
pub mod editor;
/// Error types and utilities for error handling
pub mod errors;
/// Core journal functionality
pub mod journal;

// Re-export important types for convenience
pub use cli::CliArgs;
pub use config::Config;
pub use editor::{Editor, SystemEditor};
pub use errors::{AppError, AppResult};
pub use journal::{DateSpecifier, JournalService};
