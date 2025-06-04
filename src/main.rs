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
use ponder::cli::CliArgs;
use ponder::config::Config;
use ponder::constants;
use ponder::errors::{AppError, AppResult};
use ponder::journal_core::DateSpecifier;
use ponder::journal_io;
use tracing::{debug, info, info_span};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use uuid::Uuid;

/// Runs the core application logic with the given correlation ID and CLI arguments.
///
/// This function contains the main application flow:
/// 1. Initializes logging based on CLI arguments
/// 2. Loads and validates configuration
/// 3. Ensures the journal directory exists
/// 4. Determines which entries to open based on CLI arguments
/// 5. Opens the appropriate journal entries
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
///
/// # Errors
///
/// This function can return various types of errors, including:
/// - Configuration errors (missing or invalid configuration)
/// - I/O errors (file not found, permission denied, etc.)
/// - Journal logic errors (invalid date format, etc.)
/// - Editor errors (failed to launch editor)
fn run_application(
    correlation_id: &str,
    args: CliArgs,
    current_datetime: chrono::DateTime<Local>,
) -> AppResult<()> {
    let current_date = current_datetime.naive_local().date();

    // Determine log format based on CLI args
    let use_json_logging = args.log_format == constants::LOG_FORMAT_JSON
        || std::env::var(constants::ENV_VAR_CI).is_ok();

    // Configure tracing subscriber with appropriate filter
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(constants::DEFAULT_LOG_LEVEL))
        .map_err(|e| AppError::Config(format!("Invalid log level configuration: {}", e)))?;

    // Create the subscriber builder with the filter
    let subscriber_builder = tracing_subscriber::registry().with(filter_layer);

    // Add the appropriate formatter based on the log format
    // Use try_init to avoid panics in test environments where subscriber may already be set
    let _init_result = if use_json_logging {
        // JSON logging for CI or when explicitly requested
        let json_layer = fmt::layer()
            .json()
            .with_timer(fmt::time::ChronoUtc::default()) // Use UTC time with RFC 3339 format
            .with_current_span(true) // Include current span info
            .with_span_list(true) // Include span hierarchy
            .flatten_event(true); // Flatten event fields into the JSON object
        subscriber_builder.with(json_layer).try_init()
    } else {
        // Human-readable logging for development
        let pretty_layer = fmt::layer().pretty().with_writer(std::io::stderr);
        subscriber_builder.with(pretty_layer).try_init()
    };

    // Create and enter the root span with correlation ID
    let root_span = info_span!(
        constants::TRACING_ROOT_SPAN_NAME,
        service_name = constants::TRACING_SERVICE_NAME,
        correlation_id = %correlation_id
    );
    let _guard = root_span.enter();

    // Log the application start with correlation ID
    info!("Starting ponder");
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

    // Edit the appropriate journal entries, passing the current date
    // This includes file locking to prevent concurrent access
    info!("Opening journal entries (with file locking)");
    journal_io::edit_journal_entries(&config, &dates_to_open, &current_datetime)?;

    info!("Journal entries opened successfully");
    Ok(())
}

/// The main entry point for the ponder application.
///
/// This function handles the application startup and error boundary:
/// 1. Obtains current date/time
/// 2. Parses command-line arguments
/// 3. Generates correlation ID for tracing
/// 4. Runs the core application logic
/// 5. Handles any errors with structured logging and user-friendly messages
///
/// The main function implements structured error logging at the application boundary,
/// ensuring that all errors are properly logged with correlation IDs for monitoring
/// while also providing user-friendly error messages to stderr.
fn main() {
    // Obtain current date/time once at the beginning
    let current_datetime = Local::now();

    // Parse command-line arguments first (needed for log format)
    let args = CliArgs::parse();

    // Generate a correlation ID for this application invocation
    let correlation_id = Uuid::new_v4().to_string();

    // Run the core application logic and handle any errors at the boundary
    match run_application(&correlation_id, args, current_datetime) {
        Ok(()) => {
            // Application completed successfully
            std::process::exit(0);
        }
        Err(error) => {
            // Structured logging for monitoring/alerting with full error context
            tracing::error!(
                error = %error,
                error_chain = ?error,
                correlation_id = %correlation_id,
                "Application failed"
            );

            // User-friendly output for CLI users
            eprintln!("Error: {}", error);

            // Exit with failure code
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
#[allow(dead_code)] // Unit tests disabled due to concurrency issues with temp files
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Test that run_application succeeds with valid configuration
    #[test]
    fn test_run_application_success() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().join("journal");
        fs::create_dir_all(&journal_dir)?;

        // Create CLI args for a specific date with a safe editor
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2024-01-15".to_string()),
            verbose: false,
            log_format: "text".to_string(),
        };

        // Generate a test correlation ID
        let correlation_id = "test-correlation-123";

        // Set up environment for the test
        std::env::set_var("PONDER_DIR", journal_dir.to_str().unwrap());
        std::env::set_var("PONDER_EDITOR", "true"); // Use 'true' command as safe test editor

        // Get current datetime for test
        let current_datetime = Local::now();

        // Run the application logic
        let result = run_application(correlation_id, args, current_datetime);

        // Clean up environment
        std::env::remove_var("PONDER_DIR");
        std::env::remove_var("PONDER_EDITOR");

        // Verify success
        assert!(
            result.is_ok(),
            "run_application should succeed with valid configuration"
        );

        Ok(())
    }

    /// Test that run_application propagates Config errors correctly
    #[test]
    fn test_run_application_config_error() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for testing to ensure path resolution works
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().join("journal");
        fs::create_dir_all(&journal_dir)?;

        // Create CLI args with an invalid editor (shell metacharacters)
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2024-01-15".to_string()),
            verbose: false,
            log_format: "text".to_string(),
        };

        let correlation_id = "test-correlation-config-error";
        let current_datetime = Local::now();

        // Set up environment with invalid editor command and valid journal dir
        std::env::set_var("PONDER_DIR", journal_dir.to_str().unwrap());
        std::env::set_var("PONDER_EDITOR", "vim;dangerous"); // Contains forbidden semicolon

        // Run the application logic
        let result = run_application(correlation_id, args, current_datetime);

        // Clean up environment
        std::env::remove_var("PONDER_DIR");
        std::env::remove_var("PONDER_EDITOR");

        // Verify that we get a Config error
        assert!(
            result.is_err(),
            "run_application should fail with invalid editor"
        );
        let error = result.unwrap_err();
        match error {
            AppError::Config(_) => {
                // Expected - should be a Config error for invalid editor
            }
            other => panic!("Expected Config error, got: {:?}", other),
        }

        Ok(())
    }

    /// Test that run_application propagates Journal errors correctly
    #[test]
    fn test_run_application_journal_error() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().join("journal");
        fs::create_dir_all(&journal_dir)?;

        // Create CLI args with an invalid date format
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("invalid-date-format".to_string()), // Invalid date format
            verbose: false,
            log_format: "text".to_string(),
        };

        let correlation_id = "test-correlation-journal-error";
        let current_datetime = Local::now();

        // Set up environment
        std::env::set_var("PONDER_DIR", journal_dir.to_str().unwrap());
        std::env::set_var("PONDER_EDITOR", "true");

        // Run the application logic
        let result = run_application(correlation_id, args, current_datetime);

        // Clean up environment
        std::env::remove_var("PONDER_DIR");
        std::env::remove_var("PONDER_EDITOR");

        // Verify that we get a Journal error
        assert!(
            result.is_err(),
            "run_application should fail with invalid date"
        );
        let error = result.unwrap_err();
        match error {
            AppError::Journal(_) => {
                // Expected - should be a Journal error for invalid date
            }
            other => panic!("Expected Journal error, got: {:?}", other),
        }

        Ok(())
    }

    /// Test that run_application propagates Editor errors correctly
    #[test]
    fn test_run_application_editor_error() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().join("journal");
        fs::create_dir_all(&journal_dir)?;

        // Create CLI args with valid settings
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2024-01-15".to_string()),
            verbose: false,
            log_format: "text".to_string(),
        };

        let correlation_id = "test-correlation-editor-error";
        let current_datetime = Local::now();

        // Set up environment with non-existent editor
        std::env::set_var("PONDER_DIR", journal_dir.to_str().unwrap());
        std::env::set_var("PONDER_EDITOR", "command_that_definitely_does_not_exist");

        // Run the application logic
        let result = run_application(correlation_id, args, current_datetime);

        // Clean up environment
        std::env::remove_var("PONDER_DIR");
        std::env::remove_var("PONDER_EDITOR");

        // Verify that we get an Editor error
        assert!(
            result.is_err(),
            "run_application should fail with missing editor"
        );
        let error = result.unwrap_err();
        match error {
            AppError::Editor(_) => {
                // Expected - should be an Editor error for missing command
            }
            other => panic!("Expected Editor error, got: {:?}", other),
        }

        Ok(())
    }

    /// Test that run_application propagates IO errors correctly
    #[test]
    fn test_run_application_io_error() -> Result<(), Box<dyn std::error::Error>> {
        // Create CLI args with valid settings
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2024-01-15".to_string()),
            verbose: false,
            log_format: "text".to_string(),
        };

        let correlation_id = "test-correlation-io-error";
        let current_datetime = Local::now();

        // Set up environment with a path that cannot be created (no permission to root)
        // Using /proc/1/root or similar won't work cross-platform, so we'll use a file as directory
        let temp_file = tempfile::NamedTempFile::new()?;
        let file_path = temp_file.path().to_str().unwrap();

        // Try to use a file path as a directory path (will fail when trying to create subdirs)
        let invalid_journal_path = format!("{}/journal", file_path);
        std::env::set_var("PONDER_DIR", &invalid_journal_path);
        std::env::set_var("PONDER_EDITOR", "true");

        // Run the application logic
        let result = run_application(correlation_id, args, current_datetime);

        // Clean up environment
        std::env::remove_var("PONDER_DIR");
        std::env::remove_var("PONDER_EDITOR");

        // Verify that we get an I/O error
        assert!(
            result.is_err(),
            "run_application should fail with invalid directory"
        );
        let error = result.unwrap_err();

        // Should be either I/O error or Config error depending on validation order
        match error {
            AppError::Io(_) | AppError::Config(_) => {
                // Either is acceptable - depends on where the validation fails
            }
            other => panic!("Expected I/O or Config error, got: {:?}", other),
        }

        Ok(())
    }

    /// Test that error propagation preserves error context through function boundary
    #[test]
    fn test_error_propagation_preserves_context() -> Result<(), Box<dyn std::error::Error>> {
        use std::error::Error;

        // Create CLI args that will cause a specific error
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2024-01-15".to_string()),
            verbose: false,
            log_format: "text".to_string(),
        };

        let correlation_id = "test-correlation-context";
        let current_datetime = Local::now();

        // Set up environment to cause an Editor error with a specific command
        let temp_dir = tempdir()?;
        let journal_dir = temp_dir.path().join("journal");
        fs::create_dir_all(&journal_dir)?;

        std::env::set_var("PONDER_DIR", journal_dir.to_str().unwrap());
        std::env::set_var("PONDER_EDITOR", "test_nonexistent_editor_for_context_test");

        // Run the application logic
        let result = run_application(correlation_id, args, current_datetime);

        // Clean up environment
        std::env::remove_var("PONDER_DIR");
        std::env::remove_var("PONDER_EDITOR");

        // Verify error and context preservation
        assert!(result.is_err(), "Should get an error");
        let app_error = result.unwrap_err();

        // Verify error chain is preserved
        let mut error_count = 0;
        let mut current_error: &dyn Error = &app_error;

        loop {
            error_count += 1;
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        // Should have at least 2 levels: AppError -> EditorError (or deeper)
        assert!(
            error_count >= 2,
            "Error chain should have multiple levels, got: {}",
            error_count
        );

        // Verify the error message contains context about the issue
        let error_string = format!("{}", app_error);
        assert!(
            error_string.contains("editor") || error_string.contains("Editor"),
            "Error message should contain the editor command: {}",
            error_string
        );

        Ok(())
    }
}
