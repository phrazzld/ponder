//! Core journal functionality without I/O operations.
//!
//! This module contains pure logic for date specifications and journal
//! date calculations. It provides the `DateSpecifier` enum for different
//! types of date selections without any filesystem or I/O operations.

use crate::constants;
use chrono::{Duration, Months, NaiveDate};

// Constants for date calculations (re-exported from constants module)
pub use constants::MAX_REMINISCE_YEARS_AGO;
pub use constants::MONTHS_PER_YEAR;
pub use constants::REMINISCE_ONE_MONTH_AGO;
pub use constants::REMINISCE_SIX_MONTHS_AGO;
pub use constants::REMINISCE_THREE_MONTHS_AGO;
pub use constants::RETRO_DAYS;

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
/// use ponder::journal_core::DateSpecifier;
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
    /// A Result containing either the appropriate DateSpecifier or a chrono::ParseError
    /// if the date string couldn't be parsed.
    ///
    /// # Errors
    ///
    /// Returns `chrono::ParseError` if the date string is invalid or in an unsupported format.
    ///
    /// # Examples
    ///
    /// ```
    /// use ponder::journal_core::DateSpecifier;
    ///
    /// // Create a DateSpecifier for today (default)
    /// let today = DateSpecifier::from_cli_args(false, false, None).unwrap();
    /// assert_eq!(today, DateSpecifier::Today);
    ///
    /// // Create a DateSpecifier for retro mode
    /// let retro = DateSpecifier::from_cli_args(true, false, None).unwrap();
    /// assert_eq!(retro, DateSpecifier::Retro);
    ///
    /// // Create a DateSpecifier for reminisce mode
    /// let reminisce = DateSpecifier::from_cli_args(false, true, None).unwrap();
    /// assert_eq!(reminisce, DateSpecifier::Reminisce);
    ///
    /// // Create a DateSpecifier for a specific date
    /// let specific = DateSpecifier::from_cli_args(false, false, Some("2023-01-15")).unwrap();
    /// match specific {
    ///     DateSpecifier::Specific(date) => assert_eq!(date.to_string(), "2023-01-15"),
    ///     _ => panic!("Expected Specific variant"),
    /// }
    /// ```
    pub fn from_cli_args(
        retro: bool,
        reminisce: bool,
        date_str: Option<&str>,
    ) -> Result<Self, chrono::ParseError> {
        // If a specific date is provided, it takes precedence
        if let Some(date_str) = date_str {
            return Self::parse_date_string(date_str).map(DateSpecifier::Specific);
        }

        // Check flags in order of precedence
        if retro {
            Ok(DateSpecifier::Retro)
        } else if reminisce {
            Ok(DateSpecifier::Reminisce)
        } else {
            // Default to today if no flags are set
            Ok(DateSpecifier::Today)
        }
    }

    /// Parse a date string in YYYY-MM-DD or YYYYMMDD format
    fn parse_date_string(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
        // Try parsing in YYYY-MM-DD format first
        NaiveDate::parse_from_str(date_str, constants::DATE_FORMAT_ISO)
            .or_else(|_| NaiveDate::parse_from_str(date_str, constants::DATE_FORMAT_COMPACT))
    }

    /// Gets the relevant dates for this date specifier
    ///
    /// This method calculates and returns the dates corresponding to the date specifier:
    /// - For Today: returns just the reference date
    /// - For Retro: returns dates from the past 7 days relative to the reference date
    /// - For Reminisce: returns dates from 1 month ago, 3 months ago, 6 months ago, and yearly anniversaries
    /// - For Specific: returns the specified date
    ///
    /// # Examples
    ///
    /// ```
    /// use ponder::journal_core::DateSpecifier;
    /// use chrono::NaiveDate;
    ///
    /// // Get today's date
    /// let today = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
    ///
    /// // Resolve today's entry
    /// let today_spec = DateSpecifier::Today;
    /// let today_dates = today_spec.resolve_dates(today);
    /// assert_eq!(today_dates, vec![today]);
    ///
    /// // Resolve retro entries (past 7 days)
    /// let retro_spec = DateSpecifier::Retro;
    /// let retro_dates = retro_spec.resolve_dates(today);
    /// assert_eq!(retro_dates.len(), 7);
    /// assert_eq!(retro_dates[0], today - chrono::Duration::days(1)); // Yesterday
    ///
    /// // Resolve a specific date
    /// let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
    /// let specific_spec = DateSpecifier::Specific(specific_date);
    /// let specific_dates = specific_spec.resolve_dates(today);
    /// assert_eq!(specific_dates, vec![specific_date]);
    /// ```
    pub fn resolve_dates(&self, reference_date: NaiveDate) -> Vec<NaiveDate> {
        match self {
            DateSpecifier::Today => {
                vec![reference_date]
            }
            DateSpecifier::Retro => (1..=RETRO_DAYS)
                .map(|days| reference_date - Duration::days(days))
                .collect(),
            DateSpecifier::Reminisce => {
                let mut dates = Vec::new();

                // Add specific month intervals
                if let Some(date) =
                    reference_date.checked_sub_months(Months::new(REMINISCE_ONE_MONTH_AGO))
                {
                    dates.push(date);
                }
                if let Some(date) =
                    reference_date.checked_sub_months(Months::new(REMINISCE_THREE_MONTHS_AGO))
                {
                    dates.push(date);
                }
                if let Some(date) =
                    reference_date.checked_sub_months(Months::new(REMINISCE_SIX_MONTHS_AGO))
                {
                    dates.push(date);
                }

                // Add every year ago for the past MAX_REMINISCE_YEARS_AGO years
                for year in 1..=MAX_REMINISCE_YEARS_AGO {
                    if let Some(date) =
                        reference_date.checked_sub_months(Months::new(MONTHS_PER_YEAR * year))
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_from_cli_args_today() {
        let spec = DateSpecifier::from_cli_args(false, false, None).unwrap();
        assert_eq!(spec, DateSpecifier::Today);
    }

    #[test]
    fn test_from_cli_args_retro() {
        let spec = DateSpecifier::from_cli_args(true, false, None).unwrap();
        assert_eq!(spec, DateSpecifier::Retro);
    }

    #[test]
    fn test_from_cli_args_reminisce() {
        let spec = DateSpecifier::from_cli_args(false, true, None).unwrap();
        assert_eq!(spec, DateSpecifier::Reminisce);
    }

    #[test]
    fn test_from_cli_args_specific_date() {
        let spec = DateSpecifier::from_cli_args(false, false, Some("2023-01-15")).unwrap();
        match spec {
            DateSpecifier::Specific(date) => {
                assert_eq!(date.year(), 2023);
                assert_eq!(date.month(), 1);
                assert_eq!(date.day(), 15);
            }
            _ => panic!("Expected Specific variant"),
        }
    }

    #[test]
    fn test_from_cli_args_specific_date_compact() {
        let spec = DateSpecifier::from_cli_args(false, false, Some("20230115")).unwrap();
        match spec {
            DateSpecifier::Specific(date) => {
                assert_eq!(date.year(), 2023);
                assert_eq!(date.month(), 1);
                assert_eq!(date.day(), 15);
            }
            _ => panic!("Expected Specific variant"),
        }
    }

    #[test]
    fn test_from_cli_args_invalid_date() {
        let result = DateSpecifier::from_cli_args(false, false, Some("invalid-date"));

        // Verify it returns a chrono::ParseError
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("input contains invalid characters"));
            }
            _ => panic!("Expected chrono::ParseError"),
        }
    }

    #[test]
    fn test_resolve_dates_today() {
        let spec = DateSpecifier::Today;
        let reference_date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
        let dates = spec.resolve_dates(reference_date);
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], reference_date);
    }

    #[test]
    fn test_resolve_dates_retro() {
        let spec = DateSpecifier::Retro;
        let reference_date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
        let dates = spec.resolve_dates(reference_date);
        assert_eq!(dates.len(), 7);
        // Retro gives us the past 7 days, excluding today
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2023, 6, 14).unwrap());
        assert_eq!(dates[6], NaiveDate::from_ymd_opt(2023, 6, 8).unwrap());
    }

    #[test]
    fn test_resolve_dates_reminisce() {
        let spec = DateSpecifier::Reminisce;
        let reference_date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
        let dates = spec.resolve_dates(reference_date);

        // Check some expected dates
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2023, 5, 15).unwrap())); // 1 month ago
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2023, 3, 15).unwrap())); // 3 months ago
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2022, 12, 15).unwrap())); // 6 months ago
        assert!(dates.contains(&NaiveDate::from_ymd_opt(2022, 6, 15).unwrap()));
        // 1 year ago
    }

    #[test]
    fn test_resolve_dates_specific() {
        let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
        let spec = DateSpecifier::Specific(specific_date);
        let reference_date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
        let dates = spec.resolve_dates(reference_date);
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], specific_date);
    }
}
