//! Command-line interface for the ponder application.
//!
//! This module handles command-line argument parsing using the `clap` crate.
//! It defines the CLI structure and provides methods to parse and validate
//! command-line arguments.

use crate::constants;
use clap::{ArgGroup, Parser};
use std::fmt;

/// Command-line arguments for the ponder application.
///
/// This struct is automatically populated by clap from the command-line arguments.
/// It defines a variety of options for interacting with journal entries, including
/// viewing today's entry, past entries, or entries for specific dates.
///
/// The arguments form a mutual exclusion group, so only one of `--retro`, `--reminisce`,
/// or `--date` can be specified at a time.
///
/// # Examples
///
/// ```no_run
/// use ponder::cli::CliArgs;
/// use clap::Parser;
///
/// // Simulate parsing from command-line args
/// let args = CliArgs::parse_from(["ponder", "--retro"]);
/// assert!(args.retro);
/// assert!(!args.reminisce);
/// assert!(args.date.is_none());
/// ```
#[derive(Parser)]
#[clap(
    name = constants::APP_NAME,
    about = constants::APP_DESCRIPTION
)]
#[clap(author, version, long_about = None)]
#[clap(group(ArgGroup::new("entry_type").args(&["retro", "reminisce", "date"])))]
pub struct CliArgs {
    /// Opens entries from the past week excluding today.
    ///
    /// When this flag is specified, ponder will find and open journal entries
    /// from the past 7 days (excluding today).
    #[clap(short = 'r', long, conflicts_with_all = &["reminisce", "date"])]
    pub retro: bool,

    /// Opens entries from significant past intervals.
    ///
    /// This includes entries from 1 month ago, 3 months ago, 6 months ago,
    /// and yearly anniversaries (1 year ago, 2 years ago, etc.).
    /// This is useful for reflection on past writings.
    #[clap(short = 'm', long, conflicts_with_all = &["retro", "date"])]
    pub reminisce: bool,

    /// Opens an entry for a specific date.
    ///
    /// The date can be specified in either YYYY-MM-DD format (e.g., 2023-01-15)
    /// or YYYYMMDD format (e.g., 20230115).
    #[clap(short = 'd', long, conflicts_with_all = &["retro", "reminisce"])]
    pub date: Option<String>,

    /// Enables verbose output.
    ///
    /// When this flag is set, ponder will output more detailed information
    /// about what it's doing, which can be useful for debugging.
    #[clap(short = 'v', long)]
    pub verbose: bool,

    /// Sets the log output format.
    ///
    /// Supported values are:
    /// - "text" (default): Human-readable output format
    /// - "json": Structured JSON format, useful for parsing and analysis
    #[clap(long = "log-format", value_parser = [constants::LOG_FORMAT_TEXT, constants::LOG_FORMAT_JSON], default_value = constants::LOG_FORMAT_TEXT)]
    pub log_format: String,
}

impl fmt::Debug for CliArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliArgs")
            .field("retro", &self.retro)
            .field("reminisce", &self.reminisce)
            .field(
                "date",
                &self.date.as_ref().map(|_| constants::REDACTED_PLACEHOLDER),
            )
            .field("verbose", &self.verbose)
            .field("log_format", &self.log_format)
            .finish()
    }
}

impl CliArgs {
    // No methods needed anymore - all the necessary functionality is provided by:
    // 1. The clap::Parser implementation which provides the parse() method
    // 2. DateSpecifier::from_cli_args which handles date parsing and specifier creation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = CliArgs::parse_from(vec!["ponder"]);
        assert!(!args.retro);
        assert!(!args.reminisce);
        assert!(args.date.is_none());
        assert!(!args.verbose);
        assert_eq!(args.log_format, constants::LOG_FORMAT_TEXT);
    }

    #[test]
    fn test_retro_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "--retro"]);
        assert!(args.retro);
        assert!(!args.reminisce);
        assert!(args.date.is_none());

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-r"]);
        assert!(args.retro);
        assert!(!args.reminisce);
        assert!(args.date.is_none());
    }

    #[test]
    fn test_reminisce_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "--reminisce"]);
        assert!(!args.retro);
        assert!(args.reminisce);
        assert!(args.date.is_none());

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-m"]);
        assert!(!args.retro);
        assert!(args.reminisce);
        assert!(args.date.is_none());
    }

    #[test]
    fn test_date_option() {
        let args = CliArgs::parse_from(vec!["ponder", "--date", "2023-01-15"]);
        assert!(!args.retro);
        assert!(!args.reminisce);
        assert_eq!(args.date, Some("2023-01-15".to_string()));

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-d", "20230115"]);
        assert!(!args.retro);
        assert!(!args.reminisce);
        assert_eq!(args.date, Some("20230115".to_string()));
    }

    #[test]
    fn test_verbose_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "--verbose"]);
        assert!(args.verbose);

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-v"]);
        assert!(args.verbose);

        // Test with other flags
        let args = CliArgs::parse_from(vec!["ponder", "--retro", "--verbose"]);
        assert!(args.retro);
        assert!(args.verbose);
    }

    #[test]
    fn test_debug_impl_redacts_sensitive_info() {
        // Create args with sensitive date information
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2023-01-15".to_string()),
            verbose: true,
            log_format: "text".to_string(),
        };

        // Format it with debug
        let debug_output = format!("{:?}", args);

        // Verify flags are visible but date is redacted
        assert!(debug_output.contains("retro: false"));
        assert!(debug_output.contains("reminisce: false"));
        assert!(debug_output.contains("verbose: true"));

        // Verify sensitive date is redacted
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("2023-01-15"));
    }

    // parse_date test removed as parse_date method was removed
    // The date parsing functionality is now tested in DateSpecifier::from_cli_args tests

    #[test]
    fn test_log_format_option() {
        // Test with default value
        let args = CliArgs::parse_from(vec!["ponder"]);
        assert_eq!(args.log_format, constants::LOG_FORMAT_TEXT);

        // Test with explicit text value
        let args = CliArgs::parse_from(vec!["ponder", "--log-format", constants::LOG_FORMAT_TEXT]);
        assert_eq!(args.log_format, constants::LOG_FORMAT_TEXT);

        // Test with JSON value
        let args = CliArgs::parse_from(vec!["ponder", "--log-format", constants::LOG_FORMAT_JSON]);
        assert_eq!(args.log_format, constants::LOG_FORMAT_JSON);

        // Test with other flags
        let args = CliArgs::parse_from(vec![
            "ponder",
            "--retro",
            "--log-format",
            constants::LOG_FORMAT_JSON,
        ]);
        assert!(args.retro);
        assert_eq!(args.log_format, constants::LOG_FORMAT_JSON);
    }
}
