//! Journal I/O operations and file management.
//!
//! This module contains all the I/O operations related to journal entries,
//! including file creation, directory management, and launching external editors.

use crate::config::Config;
use crate::constants;
use crate::errors::{AppError, AppResult, EditorError, LockError};
use chrono::{Datelike, Local, NaiveDate};
use fs2::FileExt;
use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info};

// Constants from the centralized constants module are used in this file

/// Ensures the journal directory exists, creating it if necessary.
///
/// This function checks if the specified directory exists and creates it
/// (including all parent directories) if it doesn't exist yet. On Unix-like
/// systems, it also sets secure permissions (0o700) on the directory.
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
///
/// # Examples
///
/// ```no_run
/// use ponder::journal_io;
/// use std::path::PathBuf;
///
/// let journal_dir = PathBuf::from("/tmp/my_journal");
///
/// // Create the journal directory if it doesn't exist
/// match journal_io::ensure_journal_directory_exists(&journal_dir) {
///     Ok(()) => println!("Journal directory is ready"),
///     Err(e) => eprintln!("Failed to ensure journal directory: {}", e),
/// }
/// ```
///
/// With a relative path (will fail):
///
/// ```
/// use ponder::journal_io;
/// use ponder::errors::AppError;
/// use std::path::PathBuf;
///
/// // Using a relative path (which will be rejected)
/// let result = journal_io::ensure_journal_directory_exists(&PathBuf::from("relative/path"));
/// assert!(result.is_err());
///
/// if let Err(err) = result {
///     match err {
///         AppError::Journal(msg) => assert!(msg.contains("must be absolute")),
///         _ => panic!("Expected a Journal error"),
///     }
/// }
/// ```
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

        // Set secure permissions (read/write/execute only for owner)
        #[cfg(unix)]
        {
            let permissions = Permissions::from_mode(constants::DEFAULT_DIR_PERMISSIONS);
            fs::set_permissions(journal_dir, permissions).map_err(|e| {
                AppError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to set secure permissions on journal directory: {}",
                        e
                    ),
                ))
            })?;
            debug!("Set 0o700 permissions on journal directory");
        }
    }
    Ok(())
}

/// Initializes a journal entry for a specific date.
///
/// This function creates a journal entry file for the given date if it doesn't
/// exist, and adds a formatted date header if the file is empty. It handles
/// the creation and preparation of journal entry files, but doesn't open them.
/// On Unix-like systems, it also sets secure permissions (0o600) on the file.
///
/// # Parameters
///
/// * `journal_dir` - Path to the journal directory
/// * `date` - The date for which to initialize a journal entry
/// * `reference_datetime` - The reference date/time to use for journal headers
///
/// # Returns
///
/// A Result containing the PathBuf to the initialized journal entry file,
/// or an AppError if there was a problem creating or initializing the file.
///
/// # Errors
///
/// Returns:
/// - `AppError::Journal` if the journal directory path is not absolute
/// - `AppError::Io` if file or directory operations fail
///
/// # Examples
///
/// ```no_run
/// use ponder::journal_io;
/// use std::path::PathBuf;
/// use chrono::{Local, NaiveDate};
///
/// // Using an absolute path
/// let journal_dir = PathBuf::from("/tmp/journal");
///
/// // Create the directory first
/// journal_io::ensure_journal_directory_exists(&journal_dir).expect("Failed to create directory");
///
/// // Get current date/time for reference
/// let current_datetime = Local::now();
///
/// // Initialize a journal entry for a specific date
/// let date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
/// let entry_path = journal_io::initialize_journal_entry(&journal_dir, date, &current_datetime)
///     .expect("Failed to initialize journal entry");
///
/// // The function returns the path to the journal entry
/// assert_eq!(entry_path, journal_dir.join("20230615.md"));
/// ```
///
/// You can also verify the entry exists after initialization:
///
/// ```no_run
/// use ponder::journal_io;
/// use std::path::PathBuf;
/// use std::fs;
/// use chrono::Local;
///
/// let journal_dir = PathBuf::from("/tmp/journal");
/// journal_io::ensure_journal_directory_exists(&journal_dir).expect("Failed to create directory");
///
/// // Get current date/time
/// let current_datetime = Local::now();
/// let today = current_datetime.naive_local().date();
///
/// // Initialize today's journal entry
/// let entry_path = journal_io::initialize_journal_entry(&journal_dir, today, &current_datetime)
///     .expect("Failed to initialize journal entry");
///
/// // Verify the entry exists
/// assert!(entry_path.exists());
///
/// // You can read the file contents to see the date header
/// let content = fs::read_to_string(&entry_path).expect("Failed to read entry");
/// assert!(content.starts_with("# "));  // Header starts with '# '
/// ```
pub fn initialize_journal_entry(
    journal_dir: &Path,
    date: NaiveDate,
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<PathBuf> {
    // Get the path for the journal entry
    let path = get_entry_path_for_date(journal_dir, date);

    // Add date header if needed (this also creates the file if it doesn't exist)
    append_date_header_if_needed(&path, reference_datetime)?;

    Ok(path)
}

/// Edits journal entries with file locking to prevent concurrent access.
///
/// This function handles the initialization, locking, and opening of journal entries for a list of dates.
/// For a single date, it initializes the entry (creates if needed) before opening.
/// For multiple dates, it only opens existing entries.
/// All files are exclusively locked while the editor is running to prevent concurrent modifications.
///
/// # Parameters
///
/// * `config` - Configuration settings containing journal directory and editor command
/// * `dates` - The dates for which to open journal entries
/// * `reference_datetime` - The current date and time to use for journal headers
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
/// - `AppError::Lock` if files could not be locked (e.g., already being edited by another process)
///
/// # Examples
///
/// Editing today's journal entry:
///
/// ```no_run
/// use ponder::config::Config;
/// use ponder::journal_io::edit_journal_entries;
/// use chrono::Local;
///
/// // Load configuration
/// let config = Config::load().expect("Failed to load config");
///
/// // Get current date and time
/// let current_datetime = Local::now();
/// let today = current_datetime.naive_local().date();
///
/// // Edit today's journal entry
/// edit_journal_entries(&config, &[today], &current_datetime).expect("Failed to edit journal");
/// ```
///
/// # File Locking
///
/// This function uses exclusive advisory locks to prevent multiple processes from
/// editing the same journal file simultaneously. If a file is already being edited
/// by another process, this function will return an error. The locks are automatically
/// released when the editor exits, allowing subsequent edits.
///
pub fn edit_journal_entries(
    config: &Config,
    dates: &[NaiveDate],
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    // Short-circuit if no dates provided
    if dates.is_empty() {
        return Ok(());
    }

    let mut paths_to_open: Vec<PathBuf> = Vec::new();
    let mut locked_files: Vec<File> = Vec::new();

    // 1. Determine paths and perform initialization (if needed)
    for (i, &date) in dates.iter().enumerate() {
        let path = get_entry_path_for_date(&config.journal_dir, date);

        // For the first date (likely the primary entry), ensure it has a timestamp header
        if i == 0 {
            debug!(
                "Adding timestamp header for primary journal entry {} at {}",
                date,
                path.display()
            );
            // Always append a timestamp header for the primary entry
            append_timestamp_header(&path, reference_datetime)?;
        }

        // Only add path if the file exists (relevant for retro/reminisce mode)
        if path.exists() {
            paths_to_open.push(path);
        } else if i > 0 {
            // Log that the file was skipped (only for non-primary dates)
            debug!("Skipping non-existent journal entry for {}", date);
        }
    }

    // Return early if no files to open
    if paths_to_open.is_empty() {
        info!("No existing entries found for the specified dates");
        return Ok(());
    }

    // 2. Acquire exclusive locks on all files before launching editor
    debug!(
        "Attempting to acquire locks on {} files",
        paths_to_open.len()
    );
    for path in &paths_to_open {
        // Open the file with read/write permissions needed for locking
        let file = OpenOptions::new()
            .read(true)
            .write(true) // Need write permissions for exclusive lock
            .open(path)
            .map_err(|e| LockError::AcquisitionFailed {
                path: path.clone(),
                source: e,
            })?;

        // Attempt to acquire a non-blocking exclusive lock
        match file.try_lock_exclusive() {
            Ok(()) => {
                // Lock acquired successfully, store the file handle
                debug!("Acquired exclusive lock on {}", path.display());
                locked_files.push(file);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Lock is held by another process (WouldBlock is typically returned for busy locks)
                error!("Failed to acquire lock on {}: File is busy", path.display());
                // Release any locks we've already acquired
                drop(locked_files);
                return Err(LockError::FileBusy { path: path.clone() }.into());
            }
            Err(e) => {
                // Other error occurred
                error!("Error acquiring lock on {}: {}", path.display(), e);
                // Release any locks we've already acquired
                drop(locked_files);
                return Err(LockError::AcquisitionFailed {
                    path: path.clone(),
                    source: e,
                }
                .into());
            }
        }
    }

    // 3. Launch editor with all paths
    info!(
        "Acquired locks on {} files, launching editor",
        paths_to_open.len()
    );
    let result = launch_editor(&config.editor, &paths_to_open);

    // 4. Locks are automatically released when locked_files goes out of scope
    debug!("Editor session finished, releasing locks");

    result
}

/// Opens journal entries based on the provided dates (without locking).
///
/// This is a wrapper around edit_journal_entries for backward compatibility.
/// New code should use edit_journal_entries which provides file locking.
///
/// # Parameters
///
/// * `config` - Configuration settings containing journal directory and editor command
/// * `dates` - The dates for which to open journal entries
/// * `reference_datetime` - The current date and time to use for journal headers
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
/// - `AppError::Lock` if files could not be locked
///
/// # Examples
///
/// Opening a single date (today):
///
/// ```no_run
/// use ponder::config::Config;
/// use ponder::journal_io::open_journal_entries;
/// use chrono::Local;
///
/// // Load configuration
/// let config = Config::load().expect("Failed to load config");
///
/// // Get current date and time
/// let current_datetime = Local::now();
/// let today = current_datetime.naive_local().date();
///
/// // Open today's journal entry
/// open_journal_entries(&config, &[today], &current_datetime).expect("Failed to open journal");
/// ```
///
pub fn open_journal_entries(
    config: &Config,
    dates: &[NaiveDate],
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    // Call the new function that handles locking
    edit_journal_entries(config, dates, reference_datetime)
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
    let filename = format!(
        "{:04}{:02}{:02}{}",
        date.year(),
        date.month(),
        date.day(),
        constants::JOURNAL_FILE_EXTENSION
    );
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

    // Set secure permissions (read/write only for owner)
    #[cfg(unix)]
    {
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(constants::DEFAULT_FILE_PERMISSIONS);
        file.set_permissions(permissions).map_err(|e| {
            AppError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to set secure permissions on journal file: {}", e),
            ))
        })?;
        debug!("Set 0o600 permissions on journal file");
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
    // Clone for ownership in error types and logging
    let editor_cmd = editor.to_string();

    // Add each path as an argument
    for path in paths {
        command.arg(path);
    }

    // Execute the command and wait for it to complete
    debug!("Launching editor: {} with {} files", editor, paths.len());

    match command.status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                let status_code = status.code().unwrap_or(-1);

                // Log the error with the editor command and status code
                error!(
                    error.type = "EditorError::NonZeroExit",
                    error.command = %editor_cmd,
                    error.status_code = status_code,
                    "Editor exited with non-zero status code"
                );

                // Create and return the EditorError
                let err = EditorError::NonZeroExit {
                    command: editor_cmd,
                    status_code,
                };
                Err(err.into())
            }
        }
        Err(e) => {
            // Map the I/O error to a specific EditorError variant based on the error kind
            let specific_error = match e.kind() {
                std::io::ErrorKind::NotFound => {
                    let error_string = e.to_string();

                    // Log the error with details
                    error!(
                        error.type = "EditorError::CommandNotFound",
                        error.command = %editor_cmd,
                        error.message = %error_string,
                        "Editor command not found"
                    );

                    // Create the EditorError
                    EditorError::CommandNotFound {
                        command: editor_cmd,
                        source: e,
                    }
                }
                std::io::ErrorKind::PermissionDenied => {
                    let error_string = e.to_string();

                    // Log the error with details
                    error!(
                        error.type = "EditorError::PermissionDenied",
                        error.command = %editor_cmd,
                        error.message = %error_string,
                        "Permission denied when trying to execute editor"
                    );

                    // Create the EditorError
                    EditorError::PermissionDenied {
                        command: editor_cmd,
                        source: e,
                    }
                }
                _ => {
                    let error_string = e.to_string();

                    // Log the error with details
                    error!(
                        error.type = "EditorError::ExecutionFailed",
                        error.command = %editor_cmd,
                        error.message = %error_string,
                        "Failed to execute editor command"
                    );

                    // Create the EditorError
                    EditorError::ExecutionFailed {
                        command: editor_cmd,
                        source: e,
                    }
                }
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
/// * `reference_datetime` - The reference date/time to use for the header
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
pub(crate) fn append_date_header_if_needed(
    path: &Path,
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    // Create or open the file
    let mut file = create_or_open_entry_file(path)?;

    // Read the file content to check if it's empty
    let content = read_file_content(path)?;

    // Only append a header if the file is empty
    if content.is_empty() {
        // Format the date and time headers using the provided reference datetime
        let entry = format!(
            "# {}\n\n## {}\n\n",
            reference_datetime.format(constants::JOURNAL_HEADER_DATE_FORMAT), // Example: "January 15, 2023: Sunday"
            reference_datetime.format(constants::JOURNAL_HEADER_TIME_FORMAT)  // Example: "14:30:45"
        );

        // Append the formatted header to the file
        append_to_file(&mut file, &entry)?;
    }

    Ok(())
}

/// Appends a timestamp header to a journal entry file unconditionally.
///
/// This function always appends a new timestamp header to the journal file,
/// regardless of whether the file is empty or already contains content.
/// If the file is empty, it adds both date and time headers. If the file
/// already has content, it only adds a time header.
///
/// # Parameters
///
/// * `path` - Path to the journal entry file
/// * `reference_datetime` - The reference date/time to use for the timestamp
///
/// # Returns
///
/// A Result indicating success or failure.
///
/// # Errors
///
/// Returns `AppError::Io` if file operations fail.
pub(crate) fn append_timestamp_header(
    path: &Path,
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    // Create or open the file
    let mut file = create_or_open_entry_file(path)?;

    // Read the file content to check if it's empty
    let content = read_file_content(path)?;

    if content.is_empty() {
        // File is empty: add both date and time headers
        let entry = format!(
            "# {}\n\n## {}\n\n",
            reference_datetime.format(constants::JOURNAL_HEADER_DATE_FORMAT),
            reference_datetime.format(constants::JOURNAL_HEADER_TIME_FORMAT)
        );
        append_to_file(&mut file, &entry)?;
    } else {
        // File has content: only add time header
        let entry = format!(
            "## {}\n\n",
            reference_datetime.format(constants::JOURNAL_HEADER_TIME_FORMAT)
        );
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

        // Create a reference datetime for the test
        let reference_datetime = Local::now();

        // Append date header to the empty file
        append_date_header_if_needed(&file_path, &reference_datetime)
            .expect("Failed to append date header");

        // Read the file content
        let content = fs::read_to_string(&file_path).expect("Failed to read file");

        // Verify that the file now contains a formatted header
        assert!(content.starts_with("# "));
        assert!(content.contains("\n\n## "));

        // Verify specific format elements (month, year, time format)
        let expected_year = reference_datetime.format("%Y").to_string();
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

        // Create a reference datetime for the test
        let reference_datetime = Local::now();

        // Try to append date header to the non-empty file
        append_date_header_if_needed(&file_path, &reference_datetime)
            .expect("Failed to check file");

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

        // Create a reference datetime for the test
        let reference_datetime = Local::now();

        // Initialize a journal entry
        let path = initialize_journal_entry(journal_dir, date, &reference_datetime)
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

        // Verify the header contains the timestamp from our reference_datetime
        let expected_year = reference_datetime.format("%Y").to_string();
        assert!(content.contains(&expected_year));

        // Verify file permissions if on Unix platform
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&path).expect("Failed to get file metadata");
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn test_append_timestamp_header_empty_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_entry.md");

        // Create empty file by opening and closing it
        File::create(&file_path).expect("Failed to create test file");

        // Create a reference datetime for the test
        let reference_datetime = Local::now();

        // Append timestamp header to the empty file
        append_timestamp_header(&file_path, &reference_datetime)
            .expect("Failed to append timestamp header");

        // Read the file content
        let content = fs::read_to_string(&file_path).expect("Failed to read file");

        // Verify both date and time headers were added
        assert!(content.contains(&format!(
            "# {}",
            reference_datetime.format(constants::JOURNAL_HEADER_DATE_FORMAT)
        )));
        assert!(content.contains(&format!(
            "## {}",
            reference_datetime.format(constants::JOURNAL_HEADER_TIME_FORMAT)
        )));
    }

    #[test]
    fn test_append_timestamp_header_non_empty_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_entry.md");

        // Create a file with existing content (simulating an existing journal entry)
        let existing_content =
            "# May 29, 2025: Thursday\n\n## 07:26:38\n\nExisting journal content";
        fs::write(&file_path, existing_content).expect("Failed to write content");

        // Create a reference datetime for the test
        let reference_datetime = Local::now();

        // Append timestamp header to the non-empty file
        append_timestamp_header(&file_path, &reference_datetime)
            .expect("Failed to append timestamp header");

        // Read the file content
        let content = fs::read_to_string(&file_path).expect("Failed to read file");

        // Verify the existing content is still there
        assert!(content.contains("Existing journal content"));

        // Verify a new time header was added (should appear twice now)
        let time_header = format!(
            "## {}",
            reference_datetime.format(constants::JOURNAL_HEADER_TIME_FORMAT)
        );
        assert!(content.contains(&time_header));

        // The original date header should still be there (only once)
        let date_headers: Vec<&str> = content.matches("# May 29, 2025: Thursday").collect();
        assert_eq!(date_headers.len(), 1, "Should have exactly one date header");
    }
}
