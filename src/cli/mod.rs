//! Command-line interface for the ponder application.
//!
//! This module handles command-line argument parsing using the `clap` crate.
//! It defines the CLI structure and provides methods to parse and validate
//! command-line arguments.

use clap::{ArgGroup, Parser};

#[cfg(test)]
use chrono::NaiveDate;
#[cfg(test)]
use std::str::FromStr;

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
#[derive(Parser, Debug)]
#[clap(
    name = "ponder",
    about = "A simple journaling tool for daily reflections"
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
}

impl CliArgs {
    /// Parses command-line arguments from the current process.
    ///
    /// This is a convenience wrapper around `clap::Parser::parse()` that
    /// uses the current process's command-line arguments.
    ///
    /// # Returns
    ///
    /// A new `CliArgs` instance populated from command-line arguments.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ponder::cli::CliArgs;
    ///
    /// let args = CliArgs::parse();
    /// // Use the parsed arguments
    /// ```
    pub fn parse() -> Self {
        CliArgs::parse_from(std::env::args())
    }

    /// Parses the date string from command-line arguments into a NaiveDate.
    ///
    /// This method attempts to parse the date specified with the `--date` option
    /// into a `chrono::NaiveDate`. It supports two date formats:
    /// - YYYY-MM-DD (e.g., 2023-01-15)
    /// - YYYYMMDD (e.g., 20230115)
    ///
    /// # Returns
    ///
    /// - `None` if no date was specified in the command-line arguments
    /// - `Some(Ok(date))` if the date was successfully parsed
    /// - `Some(Err(error))` if the date string could not be parsed
    ///
    /// # Examples
    ///
    /// ```
    /// use ponder::cli::CliArgs;
    /// use clap::Parser;
    ///
    /// // Valid date in YYYY-MM-DD format
    /// let args = CliArgs::parse_from(["ponder", "--date", "2023-01-15"]);
    /// let date = args.parse_date().unwrap().unwrap();
    /// assert_eq!(date.to_string(), "2023-01-15");
    ///
    /// // Valid date in YYYYMMDD format
    /// let args = CliArgs::parse_from(["ponder", "--date", "20230115"]);
    /// let date = args.parse_date().unwrap().unwrap();
    /// assert_eq!(date.to_string(), "2023-01-15");
    ///
    /// // No date specified
    /// let args = CliArgs::parse_from(["ponder"]);
    /// assert!(args.parse_date().is_none());
    ///
    /// // Invalid date format
    /// let args = CliArgs::parse_from(["ponder", "--date", "invalid"]);
    /// assert!(args.parse_date().unwrap().is_err());
    /// ```
    ///
    /// Note: This method is primarily used for testing purposes and may be useful
    /// for applications that need to perform custom date parsing.
    #[cfg(test)]
    pub fn parse_date(&self) -> Option<Result<NaiveDate, chrono::ParseError>> {
        self.date.as_ref().map(|date_str| {
            // Try parsing in YYYY-MM-DD format first
            NaiveDate::from_str(date_str).or_else(|_| {
                // Try parsing in YYYYMMDD format if the first format failed
                NaiveDate::parse_from_str(date_str, "%Y%m%d")
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_default_args() {
        let args = CliArgs::parse_from(vec!["ponder"]);
        assert!(!args.retro);
        assert!(!args.reminisce);
        assert!(args.date.is_none());
        assert!(!args.verbose);
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
    fn test_parse_date() {
        // Test ISO format
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("2023-01-15".to_string()),
            verbose: false,
        };

        let parsed_date = args.parse_date().unwrap().unwrap();
        assert_eq!(parsed_date.year(), 2023);
        assert_eq!(parsed_date.month(), 1);
        assert_eq!(parsed_date.day(), 15);

        // Test compact format
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("20230115".to_string()),
            verbose: false,
        };

        let parsed_date = args.parse_date().unwrap().unwrap();
        assert_eq!(parsed_date.year(), 2023);
        assert_eq!(parsed_date.month(), 1);
        assert_eq!(parsed_date.day(), 15);

        // Test None case
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: None,
            verbose: false,
        };

        assert!(args.parse_date().is_none());

        // Test invalid date
        let args = CliArgs {
            retro: false,
            reminisce: false,
            date: Some("invalid-date".to_string()),
            verbose: false,
        };

        assert!(args.parse_date().unwrap().is_err());
    }
}
