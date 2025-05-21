/*!
# Ponder - A Simple Journaling Tool

Ponder is a command-line tool for maintaining a journal of daily reflections.
It helps you create and manage markdown-formatted journal entries, with support
for creating entries for today, viewing past entries, and more.

This file contains the main application flow, coordinating the various components
to implement the journal functionality.

## Features

- Create and edit today's journal entry
- Review entries from the past week (retro mode)
- Review entries from significant past time intervals (reminisce mode)
- Open entries for specific dates
- Configurable editor and journal directory

## Usage

```
ponder [OPTIONS]

Options:
  -r, --retro                   Opens entries from the past week excluding today
  -m, --reminisce               Opens entries from significant past intervals (1 month ago, 3 months ago, etc.)
  -d, --date <DATE>             Opens an entry for a specific date (format: YYYY-MM-DD or YYYYMMDD)
  -v, --verbose                 Enable verbose output
  -h, --help                    Print help information
  -V, --version                 Print version information
```

## Configuration

The application can be configured with the following environment variables:
- `PONDER_EDITOR` or `EDITOR`: The editor to use for opening journal entries (defaults to "vim")
- `PONDER_DIR`: The directory to store journal entries (defaults to "~/Documents/rubberducks")
*/

use chrono::Local;
use clap::Parser;
use log::{debug, info};
use ponder::cli::CliArgs;
use ponder::config::Config;
use ponder::errors::{AppError, AppResult};
use ponder::journal_core::DateSpecifier;
use ponder::journal_io;

/// The main entry point for the ponder application.
///
/// This function coordinates the overall application flow:
/// 1. Initializes logging
/// 2. Parses command-line arguments
/// 3. Loads and validates configuration
/// 4. Ensures the journal directory exists
/// 5. Sets up the journal service with its dependencies
/// 6. Determines which entries to open based on CLI arguments
/// 7. Opens the appropriate journal entries
///
/// # Returns
///
/// A Result that is Ok(()) if the application ran successfully,
/// or an AppError if an error occurred at any point in the flow.
///
/// # Errors
///
/// This function can return various types of errors, including:
/// - Configuration errors (missing or invalid configuration)
/// - I/O errors (file not found, permission denied, etc.)
/// - Journal logic errors (invalid date format, etc.)
/// - Editor errors (failed to launch editor)
fn main() -> AppResult<()> {
    // Obtain current date/time once at the beginning
    let current_datetime = Local::now();
    let current_date = current_datetime.naive_local().date();

    // Initialize structured JSON logging
    env_logger::Builder::from_default_env()
        .format(move |buf, record| {
            use std::io::Write;

            // Use the timestamp obtained at the start
            let timestamp = current_datetime.to_rfc3339();
            writeln!(
                buf,
                "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"message\":\"{}\"}}",
                timestamp,
                record.level(),
                record.args()
            )
        })
        .init();

    info!("Starting ponder");

    // Parse command-line arguments
    let args = CliArgs::parse();
    debug!("CLI arguments: {:?}", args);

    // Set up verbose logging if requested
    if args.verbose {
        debug!("Verbose mode enabled");
    }

    // Load and validate configuration
    info!("Loading configuration");
    let config = Config::load()?;
    config.validate()?;

    // Ensure journal directory exists
    debug!("Journal directory: {:?}", config.journal_dir);
    journal_io::ensure_journal_directory_exists(&config.journal_dir)?;

    // Determine which entry type to open based on CLI arguments
    let date_spec = DateSpecifier::from_cli_args(args.retro, args.reminisce, args.date.as_deref())
        .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)))?;

    // Get the dates to open using the current date obtained earlier
    let dates_to_open = date_spec.resolve_dates(current_date);

    // Open the appropriate journal entries, passing the current date
    info!("Opening journal entries");
    journal_io::open_journal_entries(&config, &dates_to_open, &current_datetime)?;

    info!("Journal entries opened successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    // These tests have been moved to cli/mod.rs and are no longer needed here
}
