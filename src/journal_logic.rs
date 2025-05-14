//! Core journal functionality for the ponder application.
//!
//! This module contains the logic for handling journal entries,
//! including creating, opening, and managing entries based on different
//! date specifications. It provides the `DateSpecifier` enum for different
//! types of date selections.
//!
//! This module uses direct filesystem operations and process spawning
//! instead of trait abstractions for simplicity.

use crate::config::Config;
use crate::errors::{AppError, AppResult};
use chrono::{Duration, Local, Months, NaiveDate};
use std::fs;
use std::path::{Path, PathBuf};

// Constants for date calculations
const REMINISCE_ONE_MONTH_AGO: u32 = 1;
const REMINISCE_THREE_MONTHS_AGO: u32 = 3;
const REMINISCE_SIX_MONTHS_AGO: u32 = 6;
const MONTHS_PER_YEAR: u32 = 12;
const MAX_REMINISCE_YEARS_AGO: u32 = 100;

/// Represents different ways to specify a date or set of dates for journal entries.
///
/// This enum is used to represent the different modes of selecting journal entries:
/// - Today's entry
/// - Entries from the past week (retro)
/// - Entries from significant past intervals (reminisce)
/// - An entry for a specific date
///
/// # Examples
///
/// ```
/// use ponder::journal_logic::DateSpecifier;
/// use chrono::NaiveDate;
///
/// // Create a DateSpecifier for today's entry
/// let today = DateSpecifier::Today;
///
/// // Create a DateSpecifier for the past week
/// let retro = DateSpecifier::Retro;
///
/// // Create a DateSpecifier for significant past intervals
/// let reminisce = DateSpecifier::Reminisce;
///
/// // Create a DateSpecifier for a specific date
/// let date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
/// let specific = DateSpecifier::Specific(date);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum DateSpecifier {
    /// Represents today's journal entry.
    Today,

    /// Represents entries from the past week (excluding today).
    ///
    /// When this variant is used, the journal service will attempt to
    /// open all existing entries from the 7 days before today.
    Retro,

    /// Represents entries from significant past intervals.
    ///
    /// This includes entries from:
    /// - 1 month ago
    /// - 3 months ago
    /// - 6 months ago
    /// - Yearly anniversaries (1 year ago, 2 years ago, etc.)
    Reminisce,

    /// Represents a specific date's journal entry.
    ///
    /// This variant holds a `NaiveDate` value representing the specific
    /// date for which to open or create a journal entry.
    Specific(NaiveDate),
}

impl DateSpecifier {
    /// Creates a DateSpecifier from command-line arguments.
    ///
    /// This method creates a DateSpecifier based on the values of the retro,
    /// reminisce, and date_str parameters, which typically come from
    /// command-line arguments.
    ///
    /// # Parameters
    ///
    /// * `retro` - Flag indicating whether to open entries from the past week
    /// * `reminisce` - Flag indicating whether to open entries from significant past intervals
    /// * `date_str` - Optional string containing a specific date (in YYYY-MM-DD or YYYYMMDD format)
    ///
    /// # Returns
    ///
    /// A Result containing either the appropriate DateSpecifier or an AppError
    /// if the date string couldn't be parsed.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Journal` if the date string is invalid or in an unsupported format.
    ///
    /// # Examples
    ///
    /// ```
    /// use ponder::journal_logic::DateSpecifier;
    ///
    /// // Create a DateSpecifier for today (default)
    /// let today = DateSpecifier::from_args(false, false, None).unwrap();
    /// assert_eq!(today, DateSpecifier::Today);
    ///
    /// // Create a DateSpecifier for retro mode
    /// let retro = DateSpecifier::from_args(true, false, None).unwrap();
    /// assert_eq!(retro, DateSpecifier::Retro);
    ///
    /// // Create a DateSpecifier for reminisce mode
    /// let reminisce = DateSpecifier::from_args(false, true, None).unwrap();
    /// assert_eq!(reminisce, DateSpecifier::Reminisce);
    ///
    /// // Create a DateSpecifier for a specific date
    /// let specific = DateSpecifier::from_args(false, false, Some("2023-01-15")).unwrap();
    /// match specific {
    ///     DateSpecifier::Specific(date) => assert_eq!(date.to_string(), "2023-01-15"),
    ///     _ => panic!("Expected Specific variant"),
    /// }
    /// ```
    pub fn from_args(retro: bool, reminisce: bool, date_str: Option<&str>) -> AppResult<Self> {
        // If a specific date is provided, it takes precedence
        if let Some(date_str) = date_str {
            return Self::parse_date_string(date_str)
                .map(DateSpecifier::Specific)
                .map_err(|e| AppError::Journal(format!("Invalid date format: {}", e)));
        }

        // Otherwise, use flags
        if reminisce {
            Ok(DateSpecifier::Reminisce)
        } else if retro {
            Ok(DateSpecifier::Retro)
        } else {
            Ok(DateSpecifier::Today)
        }
    }

    /// Parse a date string in YYYY-MM-DD or YYYYMMDD format
    fn parse_date_string(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
        // Try parsing in YYYY-MM-DD format first
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(date_str, "%Y%m%d"))
    }

    /// Gets the relevant dates for this date specifier
    ///
    /// This method calculates and returns the dates corresponding to the date specifier:
    /// - For Today: returns just today's date
    /// - For Retro: returns dates from the past 7 days
    /// - For Reminisce: returns dates from 1 month ago, 3 months ago, 6 months ago, and yearly anniversaries
    /// - For Specific: returns the specified date
    pub fn get_dates(&self) -> Vec<NaiveDate> {
        match self {
            DateSpecifier::Today => {
                vec![Local::now().naive_local().date()]
            }
            DateSpecifier::Retro => {
                let now = Local::now().naive_local().date();
                (1..=7).map(|days| now - Duration::days(days)).collect()
            }
            DateSpecifier::Reminisce => {
                let now = Local::now();
                let today = now.naive_local().date();
                let mut dates = Vec::new();

                // Add specific month intervals
                if let Some(date) = today.checked_sub_months(Months::new(REMINISCE_ONE_MONTH_AGO)) {
                    dates.push(date);
                }
                if let Some(date) =
                    today.checked_sub_months(Months::new(REMINISCE_THREE_MONTHS_AGO))
                {
                    dates.push(date);
                }
                if let Some(date) = today.checked_sub_months(Months::new(REMINISCE_SIX_MONTHS_AGO))
                {
                    dates.push(date);
                }

                // Add every year ago for the past MAX_REMINISCE_YEARS_AGO years
                for year in 1..=MAX_REMINISCE_YEARS_AGO {
                    if let Some(date) =
                        today.checked_sub_months(Months::new(MONTHS_PER_YEAR * year))
                    {
                        dates.push(date);
                    }
                }

                // Remove duplicates and sort the dates
                dates.sort();
                dates.dedup();
                dates.reverse();

                dates
            }
            DateSpecifier::Specific(date) => {
                vec![*date]
            }
        }
    }
}

/// Opens journal entries based on the provided date specifier.
///
/// This function handles the opening of journal entries based on different date specifications:
/// - `DateSpecifier::Today`: Opens today's entry, creating it if it doesn't exist.
/// - `DateSpecifier::Retro`: Opens entries from the past week (if they exist).
/// - `DateSpecifier::Reminisce`: Opens entries from significant past dates (if they exist).
/// - `DateSpecifier::Specific(date)`: Opens an entry for a specific date, creating it if needed.
///
/// # Parameters
///
/// * `config` - Configuration settings containing journal directory and editor command
/// * `date_spec` - The date specifier determining which journal entries to open
///
/// # Returns
///
/// A Result that is Ok(()) if the operation completed successfully, or an AppError if there was a problem.
///
/// # Errors
///
/// May return the following errors:
/// - `AppError::Io` for file system operation failures
/// - `AppError::Editor` for editor launch failures
/// - `AppError::Journal` for journal-specific logic errors
///
/// # Examples
///
/// ```no_run
/// use ponder::config::Config;
/// use ponder::journal_logic::DateSpecifier;
/// use ponder::journal_logic::open_journal_entries;
///
/// let config = Config::load().expect("Failed to load config");
/// 
/// // Open today's journal entry
/// open_journal_entries(&config, &DateSpecifier::Today).expect("Failed to open journal");
/// ```
pub fn open_journal_entries(config: &Config, date_spec: &DateSpecifier) -> AppResult<()> {
    // This is a stub implementation that will be completed in a future task
    Ok(())
}

/// Ensures the journal directory exists, creating it if necessary.
///
/// This function checks if the specified directory exists and creates it
/// (including all parent directories) if it doesn't exist yet.
///
/// # Parameters
///
/// * `journal_dir` - Path to the journal directory
///
/// # Returns
///
/// A Result that is Ok(()) if the directory exists or was successfully created,
/// or an AppError if directory creation failed.
///
/// # Errors
///
/// Returns `AppError::Io` if the directory creation fails due to permission issues,
/// invalid paths, or other filesystem errors.
///
/// # Examples
///
/// ```no_run
/// use ponder::journal_logic::ensure_journal_directory_exists;
/// use std::path::PathBuf;
///
/// let journal_dir = PathBuf::from("/path/to/journal");
/// ensure_journal_directory_exists(&journal_dir).expect("Failed to create journal directory");
/// ```
pub fn ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()> {
    if !journal_dir.exists() {
        fs::create_dir_all(journal_dir)?;
    }
    
    Ok(())
}