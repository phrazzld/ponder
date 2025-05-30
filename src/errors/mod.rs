//! Error handling utilities for the ponder application.
//!
//! This module provides the central error type `AppError` which represents all
//! possible error conditions that might occur in the application, as well as the
//! convenience type alias `AppResult` for functions that can return these errors.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Represents specific error cases that can occur when interacting with external editors.
///
/// This enum provides detailed, contextual error information for different failure modes
/// when launching or interacting with external text editors. Each variant captures
/// relevant information such as the editor command and underlying IO errors.
///
/// # Examples
///
/// Creating and formatting a command not found error:
///
/// ```
/// use ponder::errors::EditorError;
/// use std::io::{self, ErrorKind};
///
/// // Create a command not found error
/// let io_error = io::Error::new(ErrorKind::NotFound, "command not found");
/// let error = EditorError::CommandNotFound {
///     command: "vim".to_string(),
///     source: io_error,
/// };
///
/// // The error message should mention the command and the error
/// assert!(format!("{}", error).contains("not found"));
/// assert!(format!("{}", error).contains("vim"));
/// ```
///
/// Creating a permission denied error:
///
/// ```
/// use ponder::errors::EditorError;
/// use std::io::{self, ErrorKind};
///
/// // Create a permission denied error
/// let io_error = io::Error::new(ErrorKind::PermissionDenied, "permission denied");
/// let error = EditorError::PermissionDenied {
///     command: "vim".to_string(),
///     source: io_error,
/// };
///
/// assert!(format!("{}", error).contains("Permission denied"));
/// assert!(format!("{}", error).contains("vim"));
/// ```
///
/// Creating a non-zero exit code error:
///
/// ```
/// use ponder::errors::EditorError;
///
/// // Create a non-zero exit code error
/// let error = EditorError::NonZeroExit {
///     command: "vim".to_string(),
///     status_code: 1,
/// };
///
/// assert!(format!("{}", error).contains("non-zero status code"));
/// assert!(format!("{}", error).contains("vim"));
/// assert!(format!("{}", error).contains("1"));
/// ```
#[derive(Debug, Error)]
pub enum EditorError {
    /// Error when the specified editor command cannot be found.
    #[error("Editor command '{command}' not found: {source}. Please check that the editor is installed and available in your PATH.")]
    CommandNotFound {
        /// The editor command that was not found
        command: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Error when permission is denied to execute the editor command.
    #[error("Permission denied when trying to execute editor '{command}': {source}. Please check file permissions or try running with appropriate access rights.")]
    PermissionDenied {
        /// The editor command that had permission denied
        command: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Error when the editor command fails to execute due to other I/O errors.
    #[error("Failed to execute editor '{command}': {source}. Please check system resources, disk space, or editor installation.")]
    ExecutionFailed {
        /// The editor command that failed to execute
        command: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Error when the editor exits with a non-zero status code.
    #[error("Editor '{command}' exited with non-zero status code: {status_code}. This may indicate an issue with editor configuration or the file being edited.")]
    NonZeroExit {
        /// The editor command that exited with a non-zero status
        command: String,
        /// The exit status code
        status_code: i32,
    },

    /// A catch-all for unexpected editor-related errors.
    #[error("An unexpected issue occurred while trying to use editor '{command}': {message}")]
    Other {
        /// The editor command
        command: String,
        /// A description of the error
        message: String,
    },
}

/// Represents all possible errors that can occur in the ponder application.
///
/// This enum is the central error type used across the application, with variants
/// for different error categories. It uses `thiserror` for deriving the `Error` trait
/// implementation and formatted error messages.
///
/// Note: This type does not implement `Clone` to avoid losing error context when
/// cloning `std::io::Error` values.
///
/// # Examples
///
/// Creating a configuration error:
/// ```
/// use ponder::errors::AppError;
///
/// let error = AppError::Config("Missing journal directory".to_string());
/// assert_eq!(format!("{}", error), "Configuration error: Missing journal directory");
/// ```
///
/// Converting from an IO error:
/// ```
/// use ponder::errors::AppError;
/// use std::io::{self, ErrorKind};
///
/// let io_error = io::Error::new(ErrorKind::NotFound, "file not found");
/// let app_error: AppError = io_error.into();
///
/// match app_error {
///     AppError::Io(inner) => assert_eq!(inner.kind(), ErrorKind::NotFound),
///     _ => panic!("Expected Io variant"),
/// }
/// ```
/// Represents errors that can occur when attempting to lock journal files.
///
/// This enum provides detailed, contextual error information for different failure modes
/// when acquiring or managing file locks.
///
/// # Examples
///
/// Creating a file busy error:
///
/// ```
/// use ponder::errors::LockError;
/// use std::path::PathBuf;
///
/// let error = LockError::FileBusy {
///     path: PathBuf::from("/path/to/journal_file.md"),
/// };
///
/// assert!(format!("{}", error).contains("currently being edited"));
/// ```
///
/// Creating an acquisition failed error:
///
/// ```
/// use ponder::errors::LockError;
/// use std::path::PathBuf;
/// use std::io::{self, ErrorKind};
///
/// let io_error = io::Error::new(ErrorKind::PermissionDenied, "permission denied");
/// let error = LockError::AcquisitionFailed {
///     path: PathBuf::from("/path/to/journal_file.md"),
///     source: io_error,
/// };
///
/// assert!(format!("{}", error).contains("Failed to acquire lock"));
/// assert!(format!("{}", error).contains("permission denied"));
/// ```
#[derive(Debug, Error)]
pub enum LockError {
    /// Error when the file is already locked by another process.
    #[error("Journal file is currently being edited by another process: {path}. Please wait for the other editor to close or check for existing ponder processes.")]
    FileBusy {
        /// The path to the file that is locked
        path: PathBuf,
    },

    /// Error when acquiring the lock fails for a technical reason.
    #[error("Failed to acquire lock for journal file {path}: {source}. Please check file permissions and ensure the directory is accessible.")]
    AcquisitionFailed {
        /// The path to the file that couldn't be locked
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum AppError {
    /// Errors related to configuration loading or validation.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Input/output errors from filesystem operations.
    ///
    /// This variant automatically converts from `std::io::Error` through the `From` trait.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors in journal entry logic (e.g., invalid date formats).
    #[error("Journal logic error: {0}")]
    Journal(String),

    /// Errors when interacting with the text editor.
    ///
    /// This variant uses a dedicated EditorError type to provide detailed
    /// information about what went wrong with the editor interaction.
    #[error("Editor error: {0}")]
    Editor(#[from] EditorError),

    /// Errors related to file locking.
    ///
    /// This variant uses a dedicated LockError type to provide detailed
    /// information about what went wrong with file locking operations.
    #[error("File locking error: {0}")]
    Lock(#[from] LockError),
}

/// A type alias for `Result<T, AppError>` to simplify function signatures.
///
/// This type alias is used throughout the application to represent operations
/// that may fail with an `AppError`.
///
/// # Examples
///
/// ```
/// use ponder::errors::{AppResult, AppError};
///
/// fn might_fail() -> AppResult<String> {
///     // Operation that could fail
///     if false {
///         return Err(AppError::Journal("Something went wrong".to_string()));
///     }
///     Ok("Operation succeeded".to_string())
/// }
/// ```
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_app_error_from_io_error() {
        // Create an IO error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");

        // Convert to AppError
        let app_error: AppError = io_error.into();

        // Verify conversion
        match app_error {
            AppError::Io(inner) => {
                assert_eq!(inner.kind(), io::ErrorKind::NotFound);
            }
            _ => panic!("Expected AppError::Io variant"),
        }
    }

    #[test]
    fn test_app_error_display() {
        // Test Config error
        let config_error = AppError::Config("Invalid configuration".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Configuration error: Invalid configuration"
        );

        // Test Io error
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let app_io_error = AppError::Io(io_error);
        assert_eq!(format!("{}", app_io_error), "I/O error: permission denied");

        // Test Journal error
        let journal_error = AppError::Journal("Invalid date".to_string());
        assert_eq!(
            format!("{}", journal_error),
            "Journal logic error: Invalid date"
        );

        // Test Editor error with CommandNotFound variant
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let editor_error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };
        let app_error = AppError::Editor(editor_error);
        assert!(format!("{}", app_error).contains("Editor error"));
        assert!(format!("{}", app_error).contains("not found"));
        assert!(format!("{}", app_error).contains("vim"));

        // Test Lock error with FileBusy variant
        let lock_error = LockError::FileBusy {
            path: PathBuf::from("/path/to/journal.md"),
        };
        let app_error = AppError::Lock(lock_error);
        assert!(format!("{}", app_error).contains("File locking error"));
        assert!(format!("{}", app_error).contains("currently being edited"));
        assert!(format!("{}", app_error).contains("/path/to/journal.md"));
    }

    #[test]
    fn test_editor_error_variants() {
        // Test CommandNotFound variant
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };
        assert!(format!("{}", error).contains("not found"));
        assert!(format!("{}", error).contains("vim"));

        // Test PermissionDenied variant
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let error = EditorError::PermissionDenied {
            command: "vim".to_string(),
            source: io_error,
        };
        assert!(format!("{}", error).contains("Permission denied"));
        assert!(format!("{}", error).contains("vim"));

        // Test ExecutionFailed variant
        let io_error = io::Error::other("some other error");
        let error = EditorError::ExecutionFailed {
            command: "vim".to_string(),
            source: io_error,
        };
        assert!(format!("{}", error).contains("Failed to execute"));
        assert!(format!("{}", error).contains("vim"));

        // Test NonZeroExit variant
        let error = EditorError::NonZeroExit {
            command: "vim".to_string(),
            status_code: 1,
        };
        assert!(format!("{}", error).contains("non-zero status code"));
        assert!(format!("{}", error).contains("vim"));
        assert!(format!("{}", error).contains("1"));

        // Test Other variant
        let error = EditorError::Other {
            command: "vim".to_string(),
            message: "unexpected error".to_string(),
        };
        assert!(format!("{}", error).contains("unexpected issue"));
        assert!(format!("{}", error).contains("vim"));
        assert!(format!("{}", error).contains("unexpected error"));
    }

    #[test]
    fn test_lock_error_variants() {
        // Test FileBusy variant
        let error = LockError::FileBusy {
            path: PathBuf::from("/path/to/journal.md"),
        };
        assert!(format!("{}", error).contains("currently being edited"));
        assert!(format!("{}", error).contains("/path/to/journal.md"));

        // Test AcquisitionFailed variant
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let error = LockError::AcquisitionFailed {
            path: PathBuf::from("/path/to/journal.md"),
            source: io_error,
        };
        assert!(format!("{}", error).contains("Failed to acquire lock"));
        assert!(format!("{}", error).contains("/path/to/journal.md"));
        assert!(format!("{}", error).contains("permission denied"));
    }

    #[test]
    fn test_result_combinators() {
        // Test using map_err with AppResult
        let io_result: Result<(), io::Error> = Err(io::Error::other("test error"));
        let app_result: AppResult<()> = io_result.map_err(AppError::Io);

        assert!(app_result.is_err());
        match app_result {
            Err(AppError::Io(inner)) => {
                assert_eq!(inner.kind(), io::ErrorKind::Other);
            }
            _ => panic!("Expected AppError::Io variant"),
        }
    }

    #[test]
    fn test_editor_error_conversion_to_app_error() {
        // Create an EditorError
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let editor_error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };

        // Convert to AppError
        let app_error: AppError = editor_error.into();

        // Verify conversion
        match app_error {
            AppError::Editor(inner) => match inner {
                EditorError::CommandNotFound { command, .. } => {
                    assert_eq!(command, "vim");
                }
                _ => panic!("Expected EditorError::CommandNotFound variant"),
            },
            _ => panic!("Expected AppError::Editor variant"),
        }
    }

    #[test]
    fn test_lock_error_conversion_to_app_error() {
        // Create a LockError
        let lock_error = LockError::FileBusy {
            path: PathBuf::from("/path/to/journal.md"),
        };

        // Convert to AppError
        let app_error: AppError = lock_error.into();

        // Verify conversion
        match app_error {
            AppError::Lock(inner) => match inner {
                LockError::FileBusy { path } => {
                    assert_eq!(path, PathBuf::from("/path/to/journal.md"));
                }
                _ => panic!("Expected LockError::FileBusy variant"),
            },
            _ => panic!("Expected AppError::Lock variant"),
        }
    }

    #[test]
    fn test_enhanced_editor_error_display_with_resolution_hints() {
        // Test CommandNotFound variant includes resolution hints
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };
        let message = format!("{}", error);
        assert!(message.contains("not found"));
        assert!(message.contains("vim"));
        assert!(message.contains("install") || message.contains("PATH"));

        // Test PermissionDenied variant includes resolution hints
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let error = EditorError::PermissionDenied {
            command: "vim".to_string(),
            source: io_error,
        };
        let message = format!("{}", error);
        assert!(message.contains("Permission denied"));
        assert!(message.contains("vim"));
        assert!(message.contains("permission") || message.contains("access"));

        // Test NonZeroExit variant includes resolution hints
        let error = EditorError::NonZeroExit {
            command: "vim".to_string(),
            status_code: 1,
        };
        let message = format!("{}", error);
        assert!(message.contains("non-zero status code"));
        assert!(message.contains("vim"));
        assert!(message.contains("1"));
        assert!(message.contains("configuration") || message.contains("file"));

        // Test ExecutionFailed variant includes resolution hints
        let io_error = io::Error::other("disk full");
        let error = EditorError::ExecutionFailed {
            command: "vim".to_string(),
            source: io_error,
        };
        let message = format!("{}", error);
        assert!(message.contains("Failed to execute"));
        assert!(message.contains("vim"));
        assert!(
            message.contains("disk") || message.contains("space") || message.contains("resource")
        );
    }

    #[test]
    fn test_enhanced_lock_error_display_with_resolution_hints() {
        // Test FileBusy variant includes resolution hints
        let error = LockError::FileBusy {
            path: PathBuf::from("/path/to/journal.md"),
        };
        let message = format!("{}", error);
        assert!(message.contains("currently being edited"));
        assert!(message.contains("/path/to/journal.md"));
        assert!(
            message.contains("wait") || message.contains("process") || message.contains("close")
        );

        // Test AcquisitionFailed variant includes resolution hints
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let error = LockError::AcquisitionFailed {
            path: PathBuf::from("/path/to/journal.md"),
            source: io_error,
        };
        let message = format!("{}", error);
        assert!(message.contains("Failed to acquire lock"));
        assert!(message.contains("/path/to/journal.md"));
        assert!(message.contains("permission denied"));
        assert!(
            message.contains("permission")
                || message.contains("access")
                || message.contains("directory")
        );
    }

    #[test]
    fn test_enhanced_app_error_display_clarity() {
        // Test that AppError wrappers provide clear context
        let config_error = AppError::Config("Invalid log level configuration: unknown".to_string());
        let message = format!("{}", config_error);
        assert!(message.contains("Configuration"));
        assert!(message.contains("Invalid log level configuration"));
        // The wrapper should maintain clarity without being overly verbose

        let journal_error = AppError::Journal("Invalid date format: abc".to_string());
        let message = format!("{}", journal_error);
        assert!(message.contains("Journal"));
        assert!(message.contains("Invalid date format"));
        // The wrapper should maintain clarity without being overly verbose
    }
}
