//! Error handling utilities for the ponder application.
//!
//! This module provides the central error type `AppError` which represents all
//! possible error conditions that might occur in the application, as well as the
//! convenience type alias `AppResult` for functions that can return these errors.

use std::io;
use thiserror::Error;

/// Represents specific error cases that can occur when interacting with external editors.
///
/// This enum provides detailed, contextual error information for different failure modes
/// when launching or interacting with external text editors. Each variant captures
/// relevant information such as the editor command and underlying IO errors.
///
/// # Examples
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
/// assert!(format!("{}", error).contains("not found"));
/// assert!(format!("{}", error).contains("vim"));
/// ```
#[derive(Debug, Error)]
pub enum EditorError {
    /// Error when the specified editor command cannot be found.
    #[error("Editor command '{command}' not found: {source}")]
    CommandNotFound {
        /// The editor command that was not found
        command: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Error when permission is denied to execute the editor command.
    #[error("Permission denied when trying to execute editor '{command}': {source}")]
    PermissionDenied {
        /// The editor command that had permission denied
        command: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Error when the editor command fails to execute due to other I/O errors.
    #[error("Failed to execute editor '{command}': {source}")]
    ExecutionFailed {
        /// The editor command that failed to execute
        command: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Error when the editor exits with a non-zero status code.
    #[error("Editor '{command}' exited with non-zero status code: {status_code}")]
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
        let io_error = io::Error::new(io::ErrorKind::Other, "some other error");
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
}
