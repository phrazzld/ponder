//! Input/Output operations for journal entries.
//!
//! This module provides the IO abstraction layer for the journal system,
//! allowing journal entry files to be created, read, and modified.
//! The `JournalIO` trait defines the interface for file operations,
//! and `FileSystemIO` provides a concrete implementation using the
//! standard filesystem.

use crate::errors::AppResult;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

/// Interface for journal file operations.
///
/// This trait defines the operations needed to interact with journal entries,
/// abstracting the underlying storage mechanism. This allows for different
/// implementations (e.g., for testing or to support different storage backends).
///
/// # Examples
///
/// Using the trait with a concrete implementation:
///
/// ```no_run
/// use ponder::journal::io::{JournalIO, FileSystemIO};
/// use chrono::Local;
/// use std::path::PathBuf;
///
/// # fn main() -> ponder::errors::AppResult<()> {
/// let io = FileSystemIO {
///     journal_dir: PathBuf::from("/path/to/journal"),
/// };
///
/// // Create the journal directory if it doesn't exist
/// io.ensure_journal_dir()?;
///
/// // Generate a path for today's entry
/// let path_buf = io.generate_path_for_date(Local::now())?;
///
/// // Check if the file exists
/// if !io.file_exists(&path_buf) {
///     // Create a new file and write to it
///     let mut file = io.create_or_open_file(&path_buf)?;
///     io.append_to_file(&mut file, "# New Journal Entry\n\n")?;
/// }
/// # Ok(())
/// # }
/// ```
pub trait JournalIO {
    /// Ensures the journal directory exists, creating it if necessary.
    ///
    /// # Returns
    ///
    /// A Result that is Ok(()) if the directory exists or was successfully created,
    /// or an AppError if directory creation failed.
    #[allow(dead_code)]
    fn ensure_journal_dir(&self) -> AppResult<()>;

    /// Generates a file path for a journal entry on the specified date.
    ///
    /// # Parameters
    ///
    /// * `date` - The date for which to generate a path
    ///
    /// # Returns
    ///
    /// A Result containing either the generated path as a PathBuf
    /// or an AppError if path generation failed.
    fn generate_path_for_date(&self, date: DateTime<Local>) -> AppResult<PathBuf>;

    /// Generates a file path for a journal entry from a NaiveDate.
    ///
    /// # Parameters
    ///
    /// * `date` - The date for which to generate a path
    ///
    /// # Returns
    ///
    /// A Result containing either the generated path as a PathBuf
    /// or an AppError if path generation failed.
    fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<PathBuf>;

    /// Checks if a file exists at the specified path.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise.
    fn file_exists(&self, path: &Path) -> bool;

    /// Creates a new file or opens an existing file at the specified path.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to the file to create or open
    ///
    /// # Returns
    ///
    /// A Result containing either the opened File or an AppError
    /// if file creation/opening failed.
    fn create_or_open_file(&self, path: &Path) -> AppResult<File>;

    /// Reads the content of a file as a string.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to the file to read
    ///
    /// # Returns
    ///
    /// A Result containing either the file content as a String
    /// or an AppError if file reading failed.
    fn read_file_content(&self, path: &Path) -> AppResult<String>;

    /// Appends content to a file.
    ///
    /// # Parameters
    ///
    /// * `file` - The file to append to
    /// * `content` - The content to append
    ///
    /// # Returns
    ///
    /// A Result that is Ok(()) if the content was appended successfully,
    /// or an AppError if the append operation failed.
    fn append_to_file(&self, file: &mut File, content: &str) -> AppResult<()>;
}

/// Filesystem implementation of the JournalIO trait.
///
/// This struct implements the JournalIO trait using the local filesystem
/// for storage. It stores journal entries as Markdown files (.md) in a
/// specified directory, with filenames based on dates (YYYYMMDD.md).
///
/// # Examples
///
/// ```no_run
/// use ponder::journal::io::{JournalIO, FileSystemIO};
/// use std::path::PathBuf;
///
/// let io = FileSystemIO {
///     journal_dir: PathBuf::from("/path/to/journal"),
/// };
///
/// // Ensure the journal directory exists
/// io.ensure_journal_dir().expect("Failed to create journal directory");
/// ```
pub struct FileSystemIO {
    /// The directory where journal entries are stored.
    pub journal_dir: PathBuf,
}

impl JournalIO for FileSystemIO {
    fn ensure_journal_dir(&self) -> AppResult<()> {
        if !self.journal_dir.exists() {
            std::fs::create_dir_all(&self.journal_dir)?;
        }
        Ok(())
    }

    fn generate_path_for_date(&self, date: DateTime<Local>) -> AppResult<PathBuf> {
        let filename = format!("{}.md", date.format("%Y%m%d"));
        let filepath = self.journal_dir.join(filename);
        Ok(filepath)
    }

    fn generate_path_for_naive_date(&self, date: NaiveDate) -> AppResult<PathBuf> {
        let filename = format!("{:04}{:02}{:02}.md", date.year(), date.month(), date.day());
        let filepath = self.journal_dir.join(filename);
        Ok(filepath)
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_or_open_file(&self, path: &Path) -> AppResult<File> {
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)?;
        Ok(file)
    }

    fn read_file_content(&self, path: &Path) -> AppResult<String> {
        let mut content = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    fn append_to_file(&self, file: &mut File, content: &str) -> AppResult<()> {
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
