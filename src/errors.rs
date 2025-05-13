//! Error handling utilities for the ponder application.
//!
//! This module provides the central error type `AppError` which represents all
//! possible error conditions that might occur in the application, as well as the
//! convenience type alias `AppResult` for functions that can return these errors.

use thiserror::Error;

/// Represents all possible errors that can occur in the ponder application.
///
/// This enum is the central error type used across the application, with variants
/// for different error categories. It uses `thiserror` for deriving the `Error` trait
/// implementation and formatted error messages.
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
    #[error("Editor interaction error: {0}")]
    Editor(String),
}

impl Clone for AppError {
    fn clone(&self) -> Self {
        match self {
            Self::Config(msg) => Self::Config(msg.clone()),
            Self::Io(err) => Self::Io(std::io::Error::new(err.kind(), err.to_string())),
            Self::Journal(msg) => Self::Journal(msg.clone()),
            Self::Editor(msg) => Self::Editor(msg.clone()),
        }
    }
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

        // Test Editor error
        let editor_error = AppError::Editor("Failed to launch".to_string());
        assert_eq!(
            format!("{}", editor_error),
            "Editor interaction error: Failed to launch"
        );
    }

    #[test]
    fn test_result_combinators() {
        // Test using map_err with AppResult
        let io_result: Result<(), io::Error> =
            Err(io::Error::new(io::ErrorKind::Other, "test error"));
        let app_result: AppResult<()> = io_result.map_err(AppError::Io);

        assert!(app_result.is_err());
        match app_result {
            Err(AppError::Io(inner)) => {
                assert_eq!(inner.kind(), io::ErrorKind::Other);
            }
            _ => panic!("Expected AppError::Io variant"),
        }
    }
}
