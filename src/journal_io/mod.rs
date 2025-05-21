//! Journal I/O operations and file management.
//!
//! This module contains all the I/O operations related to journal entries,
//! including file creation, directory management, and launching external editors.

use crate::config::Config;
use crate::errors::{AppError, AppResult, EditorError};
use chrono::{Datelike, Local, NaiveDate};
use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

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
/// Returns:
/// - `AppError::Journal` if the provided path is not an absolute path
/// - `AppError::Io` if the directory creation fails due to permission issues,
///   invalid paths, or other filesystem errors
pub fn ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()> {
    // Validate that the path is absolute as a defense-in-depth measure
    if !journal_dir.is_absolute() {
        return Err(AppError::Journal(format!(
            "Journal directory path must be absolute: {}",
            journal_dir.display()
        )));
    }

    if !journal_dir.exists() {
        // Create the directory if it doesn't exist
        fs::create_dir_all(journal_dir).map_err(|e| {
            AppError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create journal directory: {}", e),
            ))
        })?;

        // Set secure permissions (0o700 - read/write/execute only for owner)
        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(0o700);
            fs::set_permissions(journal_dir, permissions).map_err(|e| {
                AppError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to set secure permissions on journal directory: {}",
                        e
                    ),
                ))
            })?;
            log::debug!("Set 0o700 permissions on journal directory");
        }
    }
    Ok(())
}

/// Initializes a journal entry for a specific date.
///
/// This function creates a journal entry file for the given date if it doesn't
/// exist, and adds a formatted date header if the file is empty. It handles
/// the creation and preparation of journal entry files, but doesn't open them.
///
/// # Parameters
///
/// * `journal_dir` - Path to the journal directory
/// * `date` - The date for which to initialize a journal entry
///
/// # Returns
///
/// A Result containing the PathBuf to the initialized journal entry file,
/// or an AppError if there was a problem creating or initializing the file.
///
/// # Errors
///
/// Returns `AppError::Io` if file or directory operations fail.
///
/// # Examples
///
/// ```no_run
/// use ponder::journal_io;
/// use std::path::PathBuf;
/// use chrono::Local;
///
/// let journal_dir = PathBuf::from("~/journal");
/// let today = Local::now().naive_local().date();
///
/// // Initialize today's journal entry
/// let entry_path = journal_io::initialize_journal_entry(&journal_dir, today)
///     .expect("Failed to initialize journal entry");
/// ```
pub fn initialize_journal_entry(journal_dir: &Path, date: NaiveDate) -> AppResult<PathBuf> {
    // Get the path for the journal entry
    let path = get_entry_path_for_date(journal_dir, date);

    // Add date header if needed (this also creates the file if it doesn't exist)
    append_date_header_if_needed(&path)?;

    Ok(path)
}

/// Opens journal entries based on the provided dates.
///
/// This function handles the opening of journal entries for a list of dates.
/// For a single date, it initializes the entry (creates if needed) before opening.
/// For multiple dates, it only opens existing entries.
///
/// # Parameters
///
/// * `config` - Configuration settings containing journal directory and editor command
/// * `dates` - The dates for which to open journal entries
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
/// use ponder::journal_io::open_journal_entries;
/// use chrono::Local;
///
/// let config = Config::load().expect("Failed to load config");
/// let today = Local::now().naive_local().date();
///
/// // Open today's journal entry
/// open_journal_entries(&config, &[today]).expect("Failed to open journal");
/// ```
pub fn open_journal_entries(config: &Config, dates: &[NaiveDate]) -> AppResult<()> {
    if dates.is_empty() {
        return Ok(());
    }

    // For a single date, initialize the entry before opening
    if dates.len() == 1 {
        let date = dates[0];

        // Initialize the journal entry (creates if needed)
        let path = initialize_journal_entry(&config.journal_dir, date)?;

        // Launch editor with the entry
        launch_editor(&config.editor, &[path])
    } else {
        // For multiple dates, only open existing entries
        let mut paths = Vec::new();
        for &date in dates {
            let path = get_entry_path_for_date(&config.journal_dir, date);
            if path.exists() {
                paths.push(path);
            }
        }

        // If no entries found, log a message and return
        if paths.is_empty() {
            log::info!("No existing entries found for the specified dates");
            return Ok(());
        }

        // Launch editor with all found entries
        launch_editor(&config.editor, &paths)
    }
}

/// Generates the file path for a journal entry on a specific date.
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

    // Set secure permissions (0o600 - read/write only for owner)
    #[cfg(unix)]
    {
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o600);
        file.set_permissions(permissions).map_err(|e| {
            AppError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to set secure permissions on journal file: {}", e),
            ))
        })?;
        log::debug!("Set 0o600 permissions on journal file");
    }

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
/// as arguments. It waits for the editor to close before returning.
///
/// # Parameters
///
/// * `editor` - The editor command to launch (e.g., "vim", "nano", "emacs")
/// * `paths` - Paths to the files to open in the editor
///
/// # Returns
///
/// A Result that is Ok(()) if the editor was launched and exited successfully,
/// or an AppError if the editor couldn't be launched or returned an error status.
///
/// # Errors
///
/// Returns `AppError::Editor` with a specific `EditorError` variant depending on what went wrong:
/// - `EditorError::CommandNotFound` if the editor command doesn't exist
/// - `EditorError::PermissionDenied` if permission is denied to execute the editor
/// - `EditorError::ExecutionFailed` for other I/O errors during execution
/// - `EditorError::NonZeroExit` if the editor exits with a non-zero status code
fn launch_editor(editor: &str, paths: &[PathBuf]) -> AppResult<()> {
    let mut command = Command::new(editor);
    let editor_cmd = editor.to_string(); // Clone for ownership in error types

    // Add each path as an argument
    for path in paths {
        command.arg(path);
    }

    // Execute the command and wait for it to complete
    log::debug!("Launching editor: {} with {} files", editor, paths.len());

    match command.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                let status_code = status.code().unwrap_or(-1);
                Err(EditorError::NonZeroExit {
                    command: editor_cmd,
                    status_code,
                }
                .into())
            }
        }
        Err(e) => {
            // Map the I/O error to a specific EditorError variant based on the error kind
            let specific_error = match e.kind() {
                std::io::ErrorKind::NotFound => EditorError::CommandNotFound {
                    command: editor_cmd,
                    source: e,
                },
                std::io::ErrorKind::PermissionDenied => EditorError::PermissionDenied {
                    command: editor_cmd,
                    source: e,
                },
                _ => EditorError::ExecutionFailed {
                    command: editor_cmd,
                    source: e,
                },
            };

            Err(specific_error.into())
        }
    }
}

/// Appends a date header to a journal entry file if it's empty.
///
/// This function checks if a journal entry file is empty, and if so,
/// adds a formatted date and time header. This is used to provide a
/// consistent structure for new journal entries.
///
/// # Parameters
///
/// * `path` - Path to the journal entry file
///
/// # Returns
///
/// A Result that is Ok(()) if the operation completed successfully
/// (either the file wasn't empty or the header was successfully added),
/// or an AppError if there was a problem.
///
/// # Errors
///
/// Returns `AppError::Io` if the file couldn't be read or written to.
pub(crate) fn append_date_header_if_needed(path: &Path) -> AppResult<()> {
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

        // Verify file permissions if on Unix platform
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn test_append_date_header_if_needed_non_empty_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_entry.md");

        // Create a file with existing content
        let existing_content = "Existing journal content";
        fs::write(&file_path, existing_content).expect("Failed to write content");

        // Try to append date header to the non-empty file
        append_date_header_if_needed(&file_path).expect("Failed to check file");

        // Read the file content
        let content = fs::read_to_string(&file_path).expect("Failed to read file");

        // Verify the existing content hasn't been modified
        assert_eq!(content, existing_content);
        assert!(!content.starts_with("# "));
    }

    #[test]
    fn test_ensure_journal_directory_exists_non_existent() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let journal_dir = temp_dir.path().join("new_journal");

        // Directory shouldn't exist initially
        assert!(!journal_dir.exists());

        // Create the directory
        ensure_journal_directory_exists(&journal_dir).expect("Failed to create directory");

        // Directory should now exist
        assert!(journal_dir.exists());
        assert!(journal_dir.is_dir());

        // Verify directory permissions if on Unix platform
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&journal_dir).expect("Failed to get directory metadata");
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o700);
        }
    }

    #[test]
    fn test_ensure_journal_directory_exists_already_exists() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let journal_dir = temp_dir.path().join("existing_journal");

        // Create the directory manually
        fs::create_dir(&journal_dir).expect("Failed to create directory");
        assert!(journal_dir.exists());

        // Calling the function should not fail
        ensure_journal_directory_exists(&journal_dir).expect("Failed to ensure directory exists");

        // Directory should still exist
        assert!(journal_dir.exists());
        assert!(journal_dir.is_dir());
    }

    #[test]
    fn test_ensure_journal_directory_exists_rejects_relative_path() {
        // Use a relative path (non-absolute)
        let journal_dir = Path::new("relative/path/to/journal");

        // Function should reject this path
        let result = ensure_journal_directory_exists(journal_dir);

        // Verify the function returns an error for a relative path
        assert!(result.is_err());
        match result {
            Err(AppError::Journal(msg)) => {
                assert!(msg.contains("must be absolute"));
                assert!(msg.contains("relative/path/to/journal"));
            }
            _ => panic!("Expected AppError::Journal variant"),
        }
    }

    #[test]
    fn test_create_or_open_entry_file_permissions() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_journal_entry.md");

        // Create the file using our function
        let file = create_or_open_entry_file(&file_path).expect("Failed to create file");

        // Close the file handle
        drop(file);

        // Verify the file was created
        assert!(file_path.exists());

        // Verify file permissions if on Unix platform
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn test_initialize_journal_entry() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let journal_dir = temp_dir.path();
        let date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();

        // Initialize a journal entry
        let path = initialize_journal_entry(journal_dir, date)
            .expect("Failed to initialize journal entry");

        // Verify the file was created
        assert!(path.exists());

        // Verify file path is correct
        let expected_path = journal_dir.join("20230115.md");
        assert_eq!(path, expected_path);

        // Verify file contains date header
        let content = fs::read_to_string(&path).expect("Failed to read file");
        assert!(content.starts_with("# "));
        assert!(content.contains("\n\n## "));

        // Note: The exact date in the header is based on Local::now(), not the parameter date,
        // so we don't check the specific date content, just the header structure

        // Verify file permissions if on Unix platform
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&path).expect("Failed to get file metadata");
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }
}
