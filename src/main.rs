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

mod cli;
mod config;
mod editor;
mod errors;
mod journal;

use cli::CliArgs;
use config::Config;
use editor::SystemEditor;
use errors::AppResult;
use journal::io::FileSystemIO;
use journal::{DateSpecifier, JournalService};
use log::{debug, error, info};

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
    // Initialize logging
    env_logger::init();
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
    let config = Config::load().map_err(|e| {
        error!("Configuration error: {}", e);
        e
    })?;

    config.validate().map_err(|e| {
        error!("Invalid configuration: {}", e);
        e
    })?;

    // Ensure journal directory exists
    debug!("Journal directory: {:?}", config.journal_dir);
    config.ensure_journal_dir().map_err(|e| {
        error!("Failed to create journal directory: {}", e);
        e
    })?;

    // Initialize I/O, editor, and journal service
    info!("Initializing journal service");
    let io = Box::new(FileSystemIO {
        journal_dir: config.journal_dir.to_string_lossy().to_string(),
    });

    let editor = Box::new(SystemEditor {
        editor_cmd: config.editor.clone(),
    });

    let journal_service = JournalService::new(config, io, editor);

    // Determine which entry type to open based on CLI arguments
    let date_spec = get_date_specifier_from_args(&args)?;

    // Open the appropriate journal entries
    info!("Opening journal entries");
    journal_service.open_entries(&date_spec).map_err(|e| {
        error!("Failed to open journal entries: {}", e);
        e
    })?;

    info!("Journal entries opened successfully");
    Ok(())
}

/// Converts CLI arguments to a DateSpecifier.
///
/// This helper function examines the CLI arguments and determines the
/// appropriate DateSpecifier to use for journal entry selection.
///
/// # Parameters
///
/// * `args` - The parsed command-line arguments
///
/// # Returns
///
/// A Result containing either the appropriate DateSpecifier or an AppError
/// if a date string couldn't be parsed.
///
/// # Errors
///
/// Returns an error if the date string (from `--date` option) is invalid or
/// in an unsupported format.
///
/// # Examples
///
/// ```
/// use ponder::cli::CliArgs;
/// use ponder::journal::DateSpecifier;
///
/// // No flags specified - defaults to today
/// let args = CliArgs {
///     retro: false,
///     reminisce: false,
///     date: None,
///     verbose: false,
/// };
/// let date_spec = get_date_specifier_from_args(&args).unwrap();
/// assert_eq!(date_spec, DateSpecifier::Today);
/// ```
fn get_date_specifier_from_args(args: &CliArgs) -> AppResult<DateSpecifier> {
    if args.retro {
        Ok(DateSpecifier::Retro)
    } else if args.reminisce {
        Ok(DateSpecifier::Reminisce)
    } else if let Some(date_str) = &args.date {
        // Parse the date string
        match DateSpecifier::from_args(false, false, Some(date_str)) {
            Ok(date_spec) => Ok(date_spec),
            Err(e) => {
                error!("Invalid date format: {}", e);
                Err(e)
            }
        }
    } else {
        // Default to today if no options are specified
        Ok(DateSpecifier::Today)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use errors::AppError;

    #[test]
    fn test_get_date_specifier_from_retro_args() {
        let args = CliArgs {
            retro: true,
            reminisce: false,
            date: None,
            verbose: false,
        };

        let date_spec = get_date_specifier_from_args(&args).unwrap();
        assert_eq!(date_spec, DateSpecifier::Retro);
    }

    #[test]
    fn test_get_date_specifier_from_reminisce_args() {
        let args = CliArgs {
            retro: false,
            reminisce: true,
            date: None,
            verbose: false,
        };

        let date_spec = get_date_specifier_from_args(&args).unwrap();
        assert_eq!(date_spec, DateSpecifier::Reminisce);
    }

    #[test]
    fn test_get_date_specifier_from_date_args() {
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2023-01-15".to_string()),
            verbose: false,
        };

        let date_spec = get_date_specifier_from_args(&args).unwrap();
        if let DateSpecifier::Specific(date) = date_spec {
            assert_eq!(date.year(), 2023);
            assert_eq!(date.month(), 1);
            assert_eq!(date.day(), 15);
        } else {
            panic!("Expected DateSpecifier::Specific");
        }
    }

    #[test]
    fn test_get_date_specifier_from_invalid_date_args() {
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("invalid-date".to_string()),
            verbose: false,
        };

        let result = get_date_specifier_from_args(&args);
        assert!(result.is_err());

        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("Invalid date format"));
            }
            _ => panic!("Expected Journal error"),
        }
    }

    #[test]
    fn test_get_date_specifier_from_default_args() {
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: None,
            verbose: false,
        };

        let date_spec = get_date_specifier_from_args(&args).unwrap();
        assert_eq!(date_spec, DateSpecifier::Today);
    }
}
