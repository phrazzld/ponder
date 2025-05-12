//! Core journal functionality for the ponder application.
//!
//! This module contains the core logic for handling journal entries,
//! including creating, opening, and managing entries based on different
//! date specifications. It provides the `DateSpecifier` enum for different
//! types of date selections and the `JournalService` which implements the
//! main journal operations.
//!
//! The module follows a dependency injection pattern, allowing for flexible
//! configuration and easier testing.

pub mod io;

#[cfg(test)]
mod tests;

use crate::config::Config;
use crate::editor::Editor;
use crate::errors::{AppError, AppResult};
use chrono::{Duration, Local, Months, NaiveDate};
use io::JournalIO;
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
/// use ponder::journal::DateSpecifier;
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
    /// use ponder::journal::DateSpecifier;
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

/// Service for journal operations that follows dependency injection pattern.
///
/// This struct is the main entry point for journal operations. It implements
/// the core functionality for creating, opening, and managing journal entries.
/// It uses dependency injection to allow for flexible configuration and easier testing.
///
/// # Examples
///
/// ```no_run
/// use ponder::{Config, JournalService, DateSpecifier};
/// use ponder::journal::io::FileSystemIO;
/// use ponder::editor::SystemEditor;
/// use std::path::PathBuf;
///
/// // Create configuration
/// let config = Config {
///     editor: "vim".to_string(),
///     journal_dir: PathBuf::from("/path/to/journal"),
/// };
///
/// // Create dependencies
/// let io = Box::new(FileSystemIO {
///     journal_dir: config.journal_dir.clone(),
/// });
/// let editor = Box::new(SystemEditor {
///     editor_cmd: config.editor.clone(),
/// });
///
/// // Create the journal service
/// let journal_service = JournalService::new(config, io, editor);
///
/// // Use the service to open today's entry
/// journal_service.open_entry().expect("Failed to open today's entry");
/// ```
pub struct JournalService {
    /// Configuration settings for the journal service
    #[allow(dead_code)] // Used only in test-only methods
    config: Config,

    /// I/O abstraction for file operations
    io: Box<dyn JournalIO>,

    /// Editor abstraction for opening files
    editor: Box<dyn Editor>,
}

impl JournalService {
    /// Creates a new JournalService with the given dependencies.
    ///
    /// This constructor takes ownership of the provided dependencies, allowing
    /// for maximum flexibility in how they are implemented.
    ///
    /// # Parameters
    ///
    /// * `config` - Configuration settings for the journal service
    /// * `io` - I/O abstraction for file operations
    /// * `editor` - Editor abstraction for opening files
    ///
    /// # Returns
    ///
    /// A new JournalService instance with the given dependencies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ponder::{Config, JournalService};
    /// use ponder::journal::io::FileSystemIO;
    /// use ponder::editor::SystemEditor;
    /// use std::path::PathBuf;
    ///
    /// let config = Config {
    ///     editor: "vim".to_string(),
    ///     journal_dir: PathBuf::from("/path/to/journal"),
    /// };
    ///
    /// let io = Box::new(FileSystemIO {
    ///     journal_dir: config.journal_dir.clone(),
    /// });
    ///
    /// let editor = Box::new(SystemEditor {
    ///     editor_cmd: config.editor.clone(),
    /// });
    ///
    /// let journal_service = JournalService::new(config, io, editor);
    /// ```
    pub fn new(config: Config, io: Box<dyn JournalIO>, editor: Box<dyn Editor>) -> AppResult<Self> {
        // Ensure journal directory exists
        io.ensure_journal_dir()?;
        
        Ok(JournalService { config, io, editor })
    }

    /// Gets the editor command from the configuration.
    ///
    /// # Returns
    ///
    /// A string slice containing the editor command.
    ///
    /// Note: This method is used in tests to verify JournalService construction.
    #[cfg(test)]
    pub fn get_editor_cmd(&self) -> &str {
        &self.config.editor
    }

    /// Gets the journal directory from the configuration.
    ///
    /// # Returns
    ///
    /// A reference to the PathBuf containing the journal directory path.
    ///
    /// Note: This method is used in tests and integration tests to access the journal directory.
    #[cfg(test)]
    pub fn get_journal_dir(&self) -> &PathBuf {
        &self.config.journal_dir
    }

    /// Appends a date/time header to the specified journal file.
    ///
    /// This method appends a formatted header to a journal file. If the file is empty,
    /// it adds a primary header with the date and a secondary header with the time.
    /// If the file already has content, it adds only a secondary header with the time.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to the journal file
    ///
    /// # Returns
    ///
    /// A Result that is Ok(()) if the header was appended successfully,
    /// or an AppError if there was a problem with file operations.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file couldn't be created or opened
    /// - The file content couldn't be read
    /// - The header couldn't be appended
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ponder::{Config, JournalService};
    /// use ponder::journal::io::FileSystemIO;
    /// use ponder::editor::SystemEditor;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> ponder::errors::AppResult<()> {
    /// let config = Config {
    ///     editor: "vim".to_string(),
    ///     journal_dir: PathBuf::from("/path/to/journal"),
    /// };
    ///
    /// let io = Box::new(FileSystemIO {
    ///     journal_dir: config.journal_dir.clone(),
    /// });
    ///
    /// let editor = Box::new(SystemEditor {
    ///     editor_cmd: config.editor.clone(),
    /// });
    ///
    /// let journal_service = JournalService::new(config, io, editor);
    /// journal_service.append_date_time(&PathBuf::from("/path/to/journal/20230115.md"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn append_date_time(&self, path: &Path) -> AppResult<()> {
        let mut file = self.io.create_or_open_file(path)?;
        let now = Local::now();

        let content = self.io.read_file_content(path)?;

        let entry = if content.is_empty() {
            format!(
                "# {}\n\n## {}\n\n",
                now.format("%B %d, %Y: %A"),
                now.format("%H:%M:%S")
            )
        } else {
            format!("\n\n## {}\n\n", now.format("%H:%M:%S"))
        };

        self.io.append_to_file(&mut file, &entry)?;
        Ok(())
    }

    /// Gets the path for today's journal entry.
    ///
    /// This method generates the file path for today's journal entry
    /// using the current date.
    ///
    /// # Returns
    ///
    /// A Result containing either the path as a PathBuf or an AppError
    /// if path generation failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the path couldn't be generated (which depends on
    /// the implementation of the JournalIO trait).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ponder::{Config, JournalService};
    /// use ponder::journal::io::FileSystemIO;
    /// use ponder::editor::SystemEditor;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> ponder::errors::AppResult<()> {
    /// let journal_service = JournalService::new(
    ///     Config {
    ///         editor: "vim".to_string(),
    ///         journal_dir: PathBuf::from("/path/to/journal"),
    ///     },
    ///     Box::new(FileSystemIO {
    ///         journal_dir: PathBuf::from("/path/to/journal"),
    ///     }),
    ///     Box::new(SystemEditor {
    ///         editor_cmd: "vim".to_string(),
    ///     }),
    /// );
    ///
    /// let path = journal_service.get_todays_entry_path()?;
    /// println!("Today's entry path: {:?}", path);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_todays_entry_path(&self) -> AppResult<PathBuf> {
        let now = Local::now();
        self.io.generate_path_for_date(now)
    }

    /// Gets paths to entries from the past week (excluding today).
    ///
    /// This method searches for existing journal entries from the past 7 days
    /// (excluding today) and returns their paths.
    ///
    /// # Returns
    ///
    /// A Result containing either a vector of paths as PathBufs or an AppError
    /// if path generation or file checking failed.
    ///
    /// # Errors
    ///
    /// Returns an error if paths couldn't be generated (which depends on
    /// the implementation of the JournalIO trait).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ponder::{Config, JournalService};
    /// use ponder::journal::io::FileSystemIO;
    /// use ponder::editor::SystemEditor;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> ponder::errors::AppResult<()> {
    /// let journal_service = JournalService::new(
    ///     Config {
    ///         editor: "vim".to_string(),
    ///         journal_dir: PathBuf::from("/path/to/journal"),
    ///     },
    ///     Box::new(FileSystemIO {
    ///         journal_dir: PathBuf::from("/path/to/journal"),
    ///     }),
    ///     Box::new(SystemEditor {
    ///         editor_cmd: "vim".to_string(),
    ///     }),
    /// );
    ///
    /// let paths = journal_service.get_retro_entries()?;
    /// println!("Found {} entries from the past week", paths.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_retro_entries(&self) -> AppResult<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Use DateSpecifier to get dates from the past 7 days
        let dates = DateSpecifier::Retro.get_dates();

        // Check for existing entries on each date
        for date in dates {
            let path = self.io.generate_path_for_naive_date(date)?;

            if self.io.file_exists(&path) {
                paths.push(path);
            }
        }

        Ok(paths)
    }

    /// Gets paths to entries from significant past dates (1 month ago, 3 months ago, yearly)
    pub fn get_reminisce_entries(&self) -> AppResult<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Use DateSpecifier to calculate significant past dates
        let dates = DateSpecifier::Reminisce.get_dates();

        // Check for existing entries on each date
        for date in dates {
            let path = self.io.generate_path_for_naive_date(date)?;
            if self.io.file_exists(&path) {
                paths.push(path);
            }
        }

        Ok(paths)
    }

    /// Opens journal entries based on the provided date specifier
    pub fn open_entries(&self, date_spec: &DateSpecifier) -> AppResult<()> {
        match date_spec {
            DateSpecifier::Today => {
                let path = self.get_todays_entry_path()?;
                self.append_date_time(&path)?;
                self.editor.open_files(&[&path])
            }
            DateSpecifier::Retro => {
                let paths = self.get_retro_entries()?;
                if paths.is_empty() {
                    log::info!("No entries found for the past week");
                    return Ok(());
                }
                // Convert Vec<PathBuf> to Vec<&Path>
                let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
                self.editor.open_files(&path_refs)
            }
            DateSpecifier::Reminisce => {
                let paths = self.get_reminisce_entries()?;
                if paths.is_empty() {
                    log::info!("No entries found for reminisce intervals");
                    return Ok(());
                }
                // Convert Vec<PathBuf> to Vec<&Path>
                let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
                self.editor.open_files(&path_refs)
            }
            DateSpecifier::Specific(date) => {
                let path = self.io.generate_path_for_naive_date(*date)?;
                if !self.io.file_exists(&path) {
                    // Create a new file for the specific date
                    self.append_date_time(&path)?;
                }
                self.editor.open_files(&[&path])
            }
        }
    }

    /// Opens today's journal entry, creating it if it doesn't exist
    ///
    /// Note: This is a convenience method used in tests and integration tests.
    #[cfg(test)]
    pub fn open_entry(&self) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Today)
    }

    /// Opens entries from the past week (excluding today)
    ///
    /// Note: This is a convenience method used in tests and integration tests.
    #[cfg(test)]
    pub fn open_retro_entry(&self) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Retro)
    }

    /// Opens entries from significant past dates (1 month ago, 3 months ago, yearly anniversaries)
    ///
    /// Note: This is a convenience method used in tests and integration tests.
    #[cfg(test)]
    pub fn open_reminisce_entry(&self) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Reminisce)
    }

    /// Opens a journal entry for a specific date
    ///
    /// Note: This is a convenience method used in tests and integration tests.
    #[cfg(test)]
    pub fn open_specific_entry(&self, date: NaiveDate) -> AppResult<()> {
        self.open_entries(&DateSpecifier::Specific(date))
    }
}
