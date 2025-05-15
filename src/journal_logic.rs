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
use chrono::{Datelike, Duration, Local, Months, NaiveDate};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_append_date_header_if_needed_empty_file() {
        // Create a temporary directory that will be deleted when the test finishes
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_entry.md");

        // Create empty file by opening and closing it
        File::create(&file_path).expect("Failed to create test file");

        // Append date header to the empty file
        append_date_header_if_needed(&file_path).expect("Failed to append date header");

        // Read the file content
        let content = fs::read_to_string(&file_path).expect("Failed to read file");

        // Verify that the file now contains a formatted header
        assert!(content.starts_with("# "));
        assert!(content.contains("\n\n## "));

        // Verify specific format elements (month, year, time format)
        let now = Local::now();
        let expected_year = now.format("%Y").to_string();
        assert!(content.contains(&expected_year));

        // File should also contain a time header with HH:MM:SS format
        assert!(content.contains(":"));
    }

    #[test]
    fn test_append_date_header_if_needed_nonempty_file() {
        // Create a temporary directory that will be deleted when the test finishes
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_entry.md");

        // Create file with content
        fs::write(&file_path, "Existing content").expect("Failed to create test file");

        // Append date header to the non-empty file (should not change the file)
        append_date_header_if_needed(&file_path).expect("Failed to append date header");

        // Read the file content
        let content = fs::read_to_string(&file_path).expect("Failed to read file");

        // Verify that the file still contains only the original content
        assert_eq!(content, "Existing content");
        assert!(!content.contains("# "));
        assert!(!content.contains("## "));
    }

    #[test]
    fn test_open_journal_entries_today() {
        // This test uses the MockCommand Pattern
        // Instead of actually launching an editor, we'll check if the function
        // correctly prepares to launch the editor with the right file

        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let journal_dir = temp_dir.path().to_path_buf();

        // Create a test config that we can verify against
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: journal_dir.clone(),
        };

        // Call the function with Today date specifier
        // Note: Since we can't easily mock the editor launch,
        // this test will fail at the actual Command execution,
        // but we can check that all the preparation is correct
        let result = open_journal_entries(&config, &DateSpecifier::Today);

        // We expect the function to fail when trying to launch the non-existent editor
        assert!(result.is_err());

        // Verify a file was created for today
        let today = Local::now().naive_local().date();
        let expected_path = get_entry_path_for_date(&journal_dir, today);
        assert!(expected_path.exists());

        // Verify the file has a header
        let content = fs::read_to_string(expected_path).expect("Failed to read file");
        assert!(content.starts_with("# "));
        assert!(content.contains("\n\n## "));
    }

    #[test]
    fn test_open_journal_entries_specific_date() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let journal_dir = temp_dir.path().to_path_buf();

        // Create a test config
        let config = Config {
            editor: "test-editor".to_string(),
            journal_dir: journal_dir.clone(),
        };

        // Specific date for testing
        let test_date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();

        // Call the function with Specific date specifier
        let result = open_journal_entries(&config, &DateSpecifier::Specific(test_date));

        // We expect the function to fail when trying to launch the non-existent editor
        assert!(result.is_err());

        // Verify a file was created for the specific date
        let expected_path = get_entry_path_for_date(&journal_dir, test_date);
        assert!(expected_path.exists());

        // Verify the file has a header
        let content = fs::read_to_string(expected_path).expect("Failed to read file");
        assert!(content.starts_with("# "));
        assert!(content.contains("\n\n## "));
    }
}

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
    match date_spec {
        DateSpecifier::Today => {
            // Get today's date and generate path
            let today = Local::now().naive_local().date();
            let path = get_entry_path_for_date(&config.journal_dir, today);

            // Add date header if needed (this also creates the file if it doesn't exist)
            append_date_header_if_needed(&path)?;

            // Launch editor with today's entry
            launch_editor(&config.editor, &[path])
        }

        DateSpecifier::Retro => {
            // Get dates from past week
            let dates = date_spec.get_dates();

            // Find existing entries
            let mut paths = Vec::new();
            for date in dates {
                let path = get_entry_path_for_date(&config.journal_dir, date);
                if file_exists(&path) {
                    paths.push(path);
                }
            }

            // If no entries found, log a message and return
            if paths.is_empty() {
                log::info!("No entries found for the past week");
                return Ok(());
            }

            // Launch editor with all found entries
            launch_editor(&config.editor, &paths)
        }

        DateSpecifier::Reminisce => {
            // Get reminisce dates (1 month ago, 3 months ago, 6 months ago, yearly anniversaries)
            let dates = date_spec.get_dates();

            // Find existing entries
            let mut paths = Vec::new();
            for date in dates {
                let path = get_entry_path_for_date(&config.journal_dir, date);
                if file_exists(&path) {
                    paths.push(path);
                }
            }

            // If no entries found, log a message and return
            if paths.is_empty() {
                log::info!("No entries found for reminisce intervals");
                return Ok(());
            }

            // Launch editor with all found entries
            launch_editor(&config.editor, &paths)
        }

        DateSpecifier::Specific(date) => {
            // Generate path for specific date
            let path = get_entry_path_for_date(&config.journal_dir, *date);

            // Add date header if needed (also creates the file if it doesn't exist)
            append_date_header_if_needed(&path)?;

            // Launch editor with specific entry
            launch_editor(&config.editor, &[path])
        }
    }
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

/// Generates a file path for a journal entry for the specified date.
///
/// This function creates a PathBuf for a journal entry file by combining
/// the journal directory path with a filename derived from the date.
/// The filename format is YYYYMMDD.md.
///
/// # Parameters
///
/// * `journal_dir` - Path to the journal directory
/// * `date` - The date for which to generate the path
///
/// # Returns
///
/// A PathBuf containing the full path to the journal entry file.
fn get_entry_path_for_date(journal_dir: &Path, date: NaiveDate) -> PathBuf {
    let filename = format!("{:04}{:02}{:02}.md", date.year(), date.month(), date.day());
    journal_dir.join(filename)
}

/// Checks if a file exists at the specified path.
///
/// # Parameters
///
/// * `path` - The path to check
///
/// # Returns
///
/// `true` if the file exists, `false` otherwise.
fn file_exists(path: &Path) -> bool {
    path.exists()
}

/// Creates a new file or opens an existing file at the specified path.
///
/// This function opens a file with read and append permissions, creating
/// it if it doesn't exist yet. This is used to open journal entry files
/// for reading or appending content.
///
/// # Parameters
///
/// * `path` - The path to the file to create or open
///
/// # Returns
///
/// A Result containing either the opened File or an AppError
/// if file creation/opening failed.
///
/// # Errors
///
/// Returns `AppError::Io` if the file couldn't be created or opened due
/// to permission issues, invalid paths, or other filesystem errors.
fn create_or_open_entry_file(path: &Path) -> AppResult<File> {
    let file = OpenOptions::new()
        .read(true)
        .create(true)
        .append(true)
        .open(path)?;
    Ok(file)
}

/// Reads the content of a file as a string.
///
/// This function opens the file at the specified path and reads its
/// entire content into a String.
///
/// # Parameters
///
/// * `path` - The path to the file to read
///
/// # Returns
///
/// A Result containing either the file content as a String
/// or an AppError if file reading failed.
///
/// # Errors
///
/// Returns `AppError::Io` if the file couldn't be opened or read due
/// to permission issues, invalid paths, or other filesystem errors.
fn read_file_content(path: &Path) -> AppResult<String> {
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Appends content to a file.
///
/// This function appends the specified content to an already opened file.
/// It's typically used to add text to journal entry files.
///
/// # Parameters
///
/// * `file` - The file to append to (must be opened with write permissions)
/// * `content` - The content to append (as a string)
///
/// # Returns
///
/// A Result that is Ok(()) if the content was appended successfully,
/// or an AppError if the append operation failed.
///
/// # Errors
///
/// Returns `AppError::Io` if the file couldn't be written to due
/// to permission issues, disk space, or other filesystem errors.
fn append_to_file(file: &mut File, content: &str) -> AppResult<()> {
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Launches an external editor to open files.
///
/// This function launches the specified editor command, passing the file paths
/// as arguments to the command. It's used to open journal entry files for editing.
///
/// # Parameters
///
/// * `editor_cmd` - The editor command to execute (e.g., "vim", "code", "nano")
/// * `files_to_open` - A slice of file paths to open in the editor
///
/// # Returns
///
/// A Result that is Ok(()) if the editor was launched successfully,
/// or an AppError if there was a problem launching the editor.
///
/// # Errors
///
/// Returns `AppError::Editor` if the editor command failed to execute.
/// This could happen if:
/// - The editor command is not found
/// - The process cannot be spawned
/// - One of the file paths is invalid
fn launch_editor(editor_cmd: &str, files_to_open: &[PathBuf]) -> AppResult<()> {
    if files_to_open.is_empty() {
        return Ok(());
    }

    Command::new(editor_cmd)
        .args(files_to_open)
        .status()
        .map_err(|e| AppError::Editor(format!("Failed to launch editor: {}", e)))?;

    Ok(())
}

/// Appends a date/time header to a journal file if it's empty.
///
/// This function checks if a journal file is empty and, if it is, adds a
/// formatted date-time header. The header consists of a primary header with
/// the full date and day of the week, followed by a secondary header with the time.
///
/// # Parameters
///
/// * `path` - The path to the journal file to check and modify
///
/// # Returns
///
/// A Result that is Ok(()) if the operation completed successfully, or an AppError if there was a problem.
///
/// # Errors
///
/// Returns `AppError::Io` if the file couldn't be created, opened, read, or written to
/// due to permission issues, invalid paths, or other filesystem errors.
///
/// # Examples
///
/// ```no_run
/// use ponder::journal_logic::append_date_header_if_needed;
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("/path/to/journal/20230115.md");
/// append_date_header_if_needed(&path).expect("Failed to append date header");
/// ```
pub fn append_date_header_if_needed(path: &Path) -> AppResult<()> {
    // Create or open the file
    let mut file = create_or_open_entry_file(path)?;

    // Read the file content to check if it's empty
    let content = read_file_content(path)?;

    // Only append a header if the file is empty
    if content.is_empty() {
        let now = Local::now();

        // Format the date and time headers
        let entry = format!(
            "# {}\n\n## {}\n\n",
            now.format("%B %d, %Y: %A"), // Example: "January 15, 2023: Sunday"
            now.format("%H:%M:%S")       // Example: "14:30:45"
        );

        // Append the formatted header to the file
        append_to_file(&mut file, &entry)?;
    }

    Ok(())
}
