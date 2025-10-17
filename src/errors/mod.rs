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

/// Represents specific error cases that can occur during cryptographic operations.
///
/// This enum provides detailed, contextual error information for different failure modes
/// when performing encryption, decryption, or key management operations.
///
/// # Examples
///
/// ```
/// use ponder::errors::CryptoError;
///
/// let error = CryptoError::VaultLocked;
/// let message = format!("{}", error);
/// assert!(message.contains("locked"));
/// assert!(message.contains("passphrase"));
/// ```
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Vault is locked and needs to be unlocked before operations can proceed.
    #[error("Session locked. Run command again to unlock with your passphrase.\n\nNote: Sessions automatically lock after inactivity (configurable via PONDER_SESSION_TIMEOUT).")]
    VaultLocked,

    /// Incorrect passphrase provided for decryption.
    #[error("Incorrect passphrase. Please try again with the correct passphrase used to encrypt your journal.")]
    InvalidPassphrase(#[source] age::DecryptError),

    /// Encrypted data uses unsupported encryption format.
    #[error("Unsupported encryption format")]
    UnsupportedFormat,

    /// Invalid file path provided for encryption operation.
    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    /// Error during encryption operation.
    #[error("Encryption failed: {0}")]
    EncryptionFailed(#[source] age::EncryptError),

    /// Error during decryption operation.
    #[error("Decryption failed: {0}")]
    DecryptionFailed(#[source] age::DecryptError),
}

/// Represents specific error cases that can occur during database operations.
///
/// This enum provides detailed, contextual error information for different failure modes
/// when interacting with the encrypted SQLite database.
///
/// # Examples
///
/// ```
/// use ponder::errors::DatabaseError;
///
/// let error = DatabaseError::NotFound("Entry with id 123 not found".to_string());
/// assert!(format!("{}", error).contains("not found"));
/// ```
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// SQLite database error.
    #[error("Database error: {0}\n\nIf you're seeing 'file is not a database' or cipher errors, this may indicate:\n- Wrong passphrase (the database is encrypted with SQLCipher)\n- Corrupted database file\n- Incompatible database format")]
    Sqlite(#[from] rusqlite::Error),

    /// Connection pool error.
    #[error("Failed to get connection from pool: {0}\n\nThis may indicate database connection issues. Try closing other ponder instances.")]
    Pool(#[from] r2d2::Error),

    /// Requested entry not found in database.
    #[error("Entry not found: {0}")]
    NotFound(String),

    /// Custom database error with detailed message.
    #[error("Database error: {0}")]
    Custom(String),
}

/// Represents specific error cases that can occur during AI operations.
///
/// This enum provides detailed, contextual error information for different failure modes
/// when interacting with the Ollama API or performing AI operations.
///
/// # Examples
///
/// ```
/// use ponder::errors::AIError;
///
/// let error = AIError::ModelNotFound("llama3.2:3b".to_string());
/// assert!(format!("{}", error).contains("llama3.2:3b"));
/// ```
#[derive(Debug, Error)]
pub enum AIError {
    /// Ollama API is not reachable.
    #[error("Ollama API error: {0}. Is Ollama running? Try: ollama serve")]
    OllamaOffline(#[source] reqwest::Error),

    /// Requested model not found in Ollama.
    #[error("Model not found: {0}. Try: ollama pull {0}")]
    ModelNotFound(String),

    /// Invalid or unexpected response from Ollama API.
    #[error("Invalid response from Ollama: {0}")]
    InvalidResponse(String),
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

    /// Errors related to cryptographic operations.
    ///
    /// This variant uses a dedicated CryptoError type to provide detailed
    /// information about what went wrong with encryption, decryption, or key management.
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    /// Errors related to database operations.
    ///
    /// This variant uses a dedicated DatabaseError type to provide detailed
    /// information about what went wrong with database operations.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// Errors related to AI operations.
    ///
    /// This variant uses a dedicated AIError type to provide detailed
    /// information about what went wrong with Ollama API interactions.
    #[error("AI error: {0}")]
    AI(#[from] AIError),
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

    /// Test error source chaining for EditorError variants that have #[source] attributes
    #[test]
    fn test_editor_error_source_chaining() {
        use std::error::Error;

        // Test CommandNotFound source chaining
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let original_io_kind = io_error.kind();
        let editor_error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };

        // Test that source() returns the underlying io::Error
        let source = editor_error
            .source()
            .expect("EditorError::CommandNotFound should have a source");
        let source_io_error = source
            .downcast_ref::<io::Error>()
            .expect("Source should be an io::Error");
        assert_eq!(source_io_error.kind(), original_io_kind);
        assert_eq!(source_io_error.to_string(), "command not found");

        // Test PermissionDenied source chaining
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let editor_error = EditorError::PermissionDenied {
            command: "vim".to_string(),
            source: io_error,
        };

        let source = editor_error
            .source()
            .expect("EditorError::PermissionDenied should have a source");
        let source_io_error = source
            .downcast_ref::<io::Error>()
            .expect("Source should be an io::Error");
        assert_eq!(source_io_error.kind(), io::ErrorKind::PermissionDenied);

        // Test ExecutionFailed source chaining
        let io_error = io::Error::other("execution failed");
        let editor_error = EditorError::ExecutionFailed {
            command: "vim".to_string(),
            source: io_error,
        };

        let source = editor_error
            .source()
            .expect("EditorError::ExecutionFailed should have a source");
        let source_io_error = source
            .downcast_ref::<io::Error>()
            .expect("Source should be an io::Error");
        assert_eq!(source_io_error.to_string(), "execution failed");

        // Test that NonZeroExit has no source (it doesn't have #[source])
        let editor_error = EditorError::NonZeroExit {
            command: "vim".to_string(),
            status_code: 1,
        };
        assert!(
            editor_error.source().is_none(),
            "EditorError::NonZeroExit should not have a source"
        );

        // Test that Other has no source (it doesn't have #[source])
        let editor_error = EditorError::Other {
            command: "vim".to_string(),
            message: "unexpected error".to_string(),
        };
        assert!(
            editor_error.source().is_none(),
            "EditorError::Other should not have a source"
        );
    }

    /// Test error source chaining for LockError variants that have #[source] attributes
    #[test]
    fn test_lock_error_source_chaining() {
        use std::error::Error;

        // Test AcquisitionFailed source chaining
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let original_io_kind = io_error.kind();
        let lock_error = LockError::AcquisitionFailed {
            path: PathBuf::from("/path/to/journal.md"),
            source: io_error,
        };

        // Test that source() returns the underlying io::Error
        let source = lock_error
            .source()
            .expect("LockError::AcquisitionFailed should have a source");
        let source_io_error = source
            .downcast_ref::<io::Error>()
            .expect("Source should be an io::Error");
        assert_eq!(source_io_error.kind(), original_io_kind);
        assert_eq!(source_io_error.to_string(), "permission denied");

        // Test that FileBusy has no source (it doesn't have #[source])
        let lock_error = LockError::FileBusy {
            path: PathBuf::from("/path/to/journal.md"),
        };
        assert!(
            lock_error.source().is_none(),
            "LockError::FileBusy should not have a source"
        );
    }

    /// Test error source chaining for AppError variants, including nested chaining
    #[test]
    fn test_app_error_source_chaining() {
        use std::error::Error;

        // Test AppError::Editor with source chaining through to io::Error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let editor_error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };
        let app_error = AppError::Editor(editor_error);

        // Test first level: AppError -> EditorError
        let first_source = app_error
            .source()
            .expect("AppError::Editor should have a source");
        let editor_source = first_source
            .downcast_ref::<EditorError>()
            .expect("First source should be EditorError");

        // Test second level: EditorError -> io::Error
        let second_source = editor_source
            .source()
            .expect("EditorError should have a source");
        let io_source = second_source
            .downcast_ref::<io::Error>()
            .expect("Second source should be io::Error");
        assert_eq!(io_source.kind(), io::ErrorKind::NotFound);

        // Test AppError::Lock with source chaining
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let lock_error = LockError::AcquisitionFailed {
            path: PathBuf::from("/path/to/journal.md"),
            source: io_error,
        };
        let app_error = AppError::Lock(lock_error);

        // Test first level: AppError -> LockError
        let first_source = app_error
            .source()
            .expect("AppError::Lock should have a source");
        let lock_source = first_source
            .downcast_ref::<LockError>()
            .expect("First source should be LockError");

        // Test second level: LockError -> io::Error
        let second_source = lock_source
            .source()
            .expect("LockError should have a source");
        let io_source = second_source
            .downcast_ref::<io::Error>()
            .expect("Second source should be io::Error");
        assert_eq!(io_source.kind(), io::ErrorKind::PermissionDenied);

        // Test AppError::Io source chaining (direct io::Error)
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_error = AppError::Io(io_error);

        let source = app_error
            .source()
            .expect("AppError::Io should have a source");
        let io_source = source
            .downcast_ref::<io::Error>()
            .expect("Source should be io::Error");
        assert_eq!(io_source.kind(), io::ErrorKind::NotFound);

        // Test AppError variants without sources
        let config_error = AppError::Config("Invalid configuration".to_string());
        assert!(
            config_error.source().is_none(),
            "AppError::Config should not have a source"
        );

        let journal_error = AppError::Journal("Invalid date".to_string());
        assert!(
            journal_error.source().is_none(),
            "AppError::Journal should not have a source"
        );
    }

    /// Test full error chain traversal to ensure complete source chains work correctly
    #[test]
    fn test_full_error_chain_traversal() {
        use std::error::Error;

        // Create a deep error chain: AppError -> EditorError -> io::Error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "vim: command not found");
        let editor_error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };
        let app_error = AppError::Editor(editor_error);

        // Collect all errors in the chain
        let mut error_chain = Vec::new();
        let mut current_error: &dyn Error = &app_error;

        loop {
            error_chain.push(current_error.to_string());
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        // Verify the chain has the expected depth and content
        assert_eq!(
            error_chain.len(),
            3,
            "Error chain should have 3 levels: AppError -> EditorError -> io::Error"
        );
        assert!(
            error_chain[0].contains("Editor error"),
            "First error should be AppError::Editor"
        );
        assert!(
            error_chain[1].contains("not found") && error_chain[1].contains("vim"),
            "Second error should be EditorError::CommandNotFound"
        );
        assert!(
            error_chain[2].contains("vim: command not found"),
            "Third error should be the original io::Error"
        );

        // Test a shorter chain: AppError -> io::Error (direct conversion)
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let app_error = AppError::Io(io_error);

        let mut error_chain = Vec::new();
        let mut current_error: &dyn Error = &app_error;

        loop {
            error_chain.push(current_error.to_string());
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        assert_eq!(
            error_chain.len(),
            2,
            "Direct io::Error chain should have 2 levels: AppError -> io::Error"
        );
        assert!(
            error_chain[0].contains("I/O error"),
            "First error should be AppError::Io"
        );
        assert!(
            error_chain[1].contains("permission denied"),
            "Second error should be the original io::Error"
        );
    }

    /// Test error construction with proper context preservation
    #[test]
    fn test_error_construction_context_preservation() {
        // Test that constructed errors preserve all expected context

        // EditorError construction
        let io_error = io::Error::new(io::ErrorKind::NotFound, "vim: command not found");
        let editor_error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };

        // Verify all context is preserved
        match editor_error {
            EditorError::CommandNotFound { command, source } => {
                assert_eq!(command, "vim");
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
                assert_eq!(source.to_string(), "vim: command not found");
            }
            _ => panic!("Expected CommandNotFound variant"),
        }

        // LockError construction
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let lock_error = LockError::AcquisitionFailed {
            path: PathBuf::from("/test/journal.md"),
            source: io_error,
        };

        // Verify all context is preserved
        match lock_error {
            LockError::AcquisitionFailed { path, source } => {
                assert_eq!(path, PathBuf::from("/test/journal.md"));
                assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
                assert_eq!(source.to_string(), "permission denied");
            }
            _ => panic!("Expected AcquisitionFailed variant"),
        }

        // AppError construction with proper wrapping
        let config_error = AppError::Config("Invalid log level: unknown".to_string());
        match config_error {
            AppError::Config(message) => {
                assert_eq!(message, "Invalid log level: unknown");
            }
            _ => panic!("Expected Config variant"),
        }
    }

    /// Comprehensive tests for error Display implementations covering all variants and edge cases
    #[test]
    fn test_comprehensive_error_display_formatting() {
        // Test AppError::Config Display with various message types
        let long_message = "X".repeat(1000);
        let config_errors = vec![
            ("Empty config", "".to_string()),
            ("Simple message", "Invalid setting".to_string()),
            ("Unicode message", "配置错误: invalid 设置".to_string()),
            ("Long message", long_message),
            (
                "Special chars",
                "Config with\nnewlines\tand\ttabs".to_string(),
            ),
        ];

        for (desc, message) in config_errors {
            let error = AppError::Config(message.clone());
            let display = format!("{}", error);
            assert!(
                display.starts_with("Configuration error: "),
                "Config error should start with proper prefix for {}",
                desc
            );
            assert!(
                display.contains(&message),
                "Config error should contain original message for {}",
                desc
            );
        }

        // Test AppError::Journal Display with various message types
        let long_date_error = format!("Very long date error: {}", "x".repeat(500));
        let journal_errors = vec![
            (
                "Date parsing",
                "Invalid date format: 2024-13-45".to_string(),
            ),
            ("Empty date", "Date cannot be empty".to_string()),
            ("Unicode date", "无效日期: 2024年13月".to_string()),
            ("Long date error", long_date_error),
        ];

        for (desc, message) in journal_errors {
            let error = AppError::Journal(message.clone());
            let display = format!("{}", error);
            assert!(
                display.starts_with("Journal logic error: "),
                "Journal error should start with proper prefix for {}",
                desc
            );
            assert!(
                display.contains(&message),
                "Journal error should contain original message for {}",
                desc
            );
        }
    }

    /// Test EditorError Display implementations for comprehensive coverage
    #[test]
    fn test_comprehensive_editor_error_display() {
        // Test CommandNotFound with various command names
        let long_command = "x".repeat(100);
        let commands = vec![
            ("simple", "vim".to_string()),
            ("with-dashes", "text-editor".to_string()),
            ("with_underscores", "my_editor".to_string()),
            ("unicode", "编辑器".to_string()),
            ("long", long_command),
            ("absolute path", "/usr/bin/vim".to_string()),
            ("relative path", "./vim".to_string()),
        ];

        for (desc, command) in commands {
            let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
            let error = EditorError::CommandNotFound {
                command: command.clone(),
                source: io_error,
            };
            let display = format!("{}", error);

            assert!(
                display.contains(&command),
                "CommandNotFound should contain command name for {}",
                desc
            );
            assert!(
                display.contains("not found"),
                "CommandNotFound should indicate not found for {}",
                desc
            );
            assert!(
                display.contains("PATH") || display.contains("install"),
                "CommandNotFound should provide resolution hint for {}",
                desc
            );
        }

        // Test PermissionDenied with various scenarios
        let permission_scenarios = vec![
            (
                "No permission",
                io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"),
            ),
            (
                "Access denied",
                io::Error::new(io::ErrorKind::PermissionDenied, "access denied"),
            ),
            (
                "Read-only",
                io::Error::new(io::ErrorKind::PermissionDenied, "read-only file system"),
            ),
        ];

        for (desc, io_error) in permission_scenarios {
            let error = EditorError::PermissionDenied {
                command: "vim".to_string(),
                source: io_error,
            };
            let display = format!("{}", error);

            assert!(
                display.contains("vim"),
                "PermissionDenied should contain command for {}",
                desc
            );
            assert!(
                display.contains("Permission denied") || display.contains("denied"),
                "PermissionDenied should indicate permission issue for {}",
                desc
            );
        }

        // Test ExecutionFailed with various failure reasons
        let execution_failures = vec![
            ("Resource", io::Error::other("insufficient resources")),
            ("Disk full", io::Error::other("no space left on device")),
            ("Memory", io::Error::other("out of memory")),
            ("Signal", io::Error::other("killed by signal")),
        ];

        for (desc, io_error) in execution_failures {
            let error = EditorError::ExecutionFailed {
                command: "vim".to_string(),
                source: io_error,
            };
            let display = format!("{}", error);

            assert!(
                display.contains("vim"),
                "ExecutionFailed should contain command for {}",
                desc
            );
            assert!(
                display.contains("Failed to execute"),
                "ExecutionFailed should indicate execution failure for {}",
                desc
            );
        }

        // Test NonZeroExit with various status codes
        let exit_codes = vec![1, 127, 130, 255];
        for code in exit_codes {
            let error = EditorError::NonZeroExit {
                command: "vim".to_string(),
                status_code: code,
            };
            let display = format!("{}", error);

            assert!(
                display.contains("vim"),
                "NonZeroExit should contain command for code {}",
                code
            );
            assert!(
                display.contains(&code.to_string()),
                "NonZeroExit should contain status code {}",
                code
            );
            assert!(
                display.contains("non-zero"),
                "NonZeroExit should indicate non-zero exit for code {}",
                code
            );
        }

        // Test Other variant with various messages
        let long_message = "x".repeat(500);
        let other_messages = vec![
            ("Timeout", "editor operation timed out".to_string()),
            ("Corruption", "editor config file corrupted".to_string()),
            ("Network", "editor requires network access".to_string()),
            ("Unicode", "编辑器错误信息".to_string()),
            ("Empty", "".to_string()),
            ("Long", long_message),
        ];

        for (desc, message) in other_messages {
            let error = EditorError::Other {
                command: "vim".to_string(),
                message: message.clone(),
            };
            let display = format!("{}", error);

            assert!(
                display.contains("vim"),
                "Other variant should contain command for {}",
                desc
            );
            assert!(
                display.contains("unexpected issue") || display.contains("issue"),
                "Other variant should indicate unexpected issue for {}",
                desc
            );
            if !message.is_empty() {
                assert!(
                    display.contains(&message),
                    "Other variant should contain custom message for {}",
                    desc
                );
            }
        }
    }

    /// Test LockError Display implementations for comprehensive coverage
    #[test]
    fn test_comprehensive_lock_error_display() {
        use std::path::PathBuf;

        // Test FileBusy with various path types
        let long_path = format!("/{}", "very_long_path_name/".repeat(20))
            .trim_end_matches('/')
            .to_string();
        let paths = vec![
            ("simple", "/journal.md".to_string()),
            (
                "nested",
                "/home/user/documents/journal/2024/01/15.md".to_string(),
            ),
            ("windows", r"C:\Users\User\Documents\journal.md".to_string()),
            ("unicode", "/用户/文档/日记.md".to_string()),
            ("spaces", "/path with spaces/journal entry.md".to_string()),
            ("long", long_path),
            ("relative", "./journal.md".to_string()),
        ];

        for (desc, path) in paths {
            let error = LockError::FileBusy {
                path: PathBuf::from(&path),
            };
            let display = format!("{}", error);

            assert!(
                display.contains(&path),
                "FileBusy should contain file path for {}",
                desc
            );
            assert!(
                display.contains("currently being edited") || display.contains("being edited"),
                "FileBusy should indicate file is being edited for {}",
                desc
            );
            assert!(
                display.contains("wait")
                    || display.contains("process")
                    || display.contains("close"),
                "FileBusy should provide resolution hint for {}",
                desc
            );
        }

        // Test AcquisitionFailed with various I/O error types
        let io_errors = vec![
            (
                "Permission",
                io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"),
            ),
            (
                "Not found",
                io::Error::new(io::ErrorKind::NotFound, "directory not found"),
            ),
            (
                "Read only",
                io::Error::new(io::ErrorKind::PermissionDenied, "read-only file system"),
            ),
            (
                "Disk full",
                io::Error::new(io::ErrorKind::WriteZero, "no space left on device"),
            ),
            (
                "Network",
                io::Error::new(io::ErrorKind::NetworkUnreachable, "network is unreachable"),
            ),
        ];

        for (desc, io_error) in io_errors {
            let path = "/test/journal.md";
            let error = LockError::AcquisitionFailed {
                path: PathBuf::from(path),
                source: io_error,
            };
            let display = format!("{}", error);

            assert!(
                display.contains(path),
                "AcquisitionFailed should contain file path for {}",
                desc
            );
            assert!(
                display.contains("Failed to acquire lock"),
                "AcquisitionFailed should indicate lock failure for {}",
                desc
            );
            assert!(
                display.contains("permission")
                    || display.contains("access")
                    || display.contains("directory"),
                "AcquisitionFailed should provide resolution hint for {}",
                desc
            );
        }
    }

    /// Test AppError Display wrapping behavior
    #[test]
    fn test_app_error_display_wrapping_behavior() {
        // Test that AppError properly wraps and formats nested errors

        // Test Editor error wrapping preserves all information
        let io_error = io::Error::new(io::ErrorKind::NotFound, "command not found");
        let editor_error = EditorError::CommandNotFound {
            command: "vim".to_string(),
            source: io_error,
        };
        let app_error = AppError::Editor(editor_error);
        let display = format!("{}", app_error);

        assert!(
            display.starts_with("Editor error: "),
            "AppError::Editor should start with proper prefix"
        );
        assert!(
            display.contains("vim"),
            "AppError::Editor should preserve command from underlying error"
        );
        assert!(
            display.contains("not found"),
            "AppError::Editor should preserve details from underlying error"
        );

        // Test Lock error wrapping preserves all information
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let lock_error = LockError::AcquisitionFailed {
            path: PathBuf::from("/test/journal.md"),
            source: io_error,
        };
        let app_error = AppError::Lock(lock_error);
        let display = format!("{}", app_error);

        assert!(
            display.starts_with("File locking error: "),
            "AppError::Lock should start with proper prefix"
        );
        assert!(
            display.contains("/test/journal.md"),
            "AppError::Lock should preserve path from underlying error"
        );
        assert!(
            display.contains("permission denied"),
            "AppError::Lock should preserve I/O error details"
        );

        // Test I/O error wrapping preserves error kind and message
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_error = AppError::Io(io_error);
        let display = format!("{}", app_error);

        assert!(
            display.starts_with("I/O error: "),
            "AppError::Io should start with proper prefix"
        );
        assert!(
            display.contains("file not found"),
            "AppError::Io should preserve original I/O error message"
        );
    }

    /// Test error Display consistency and formatting standards
    #[test]
    fn test_error_display_consistency_and_standards() {
        // Test that all AppError variants follow consistent formatting patterns

        // All AppError variants should have a descriptive prefix followed by ": "
        let errors = vec![
            (
                AppError::Config("test".to_string()),
                "Configuration error: ",
            ),
            (
                AppError::Journal("test".to_string()),
                "Journal logic error: ",
            ),
            (AppError::Io(io::Error::other("test")), "I/O error: "),
        ];

        for (error, expected_prefix) in errors {
            let display = format!("{}", error);
            assert!(
                display.starts_with(expected_prefix),
                "Error display should start with consistent prefix: {}",
                expected_prefix
            );
            assert!(
                !display.ends_with('\n'),
                "Error display should not end with newline"
            );
            assert!(
                !display.starts_with(' '),
                "Error display should not start with whitespace"
            );
            assert!(
                !display.ends_with(' '),
                "Error display should not end with whitespace"
            );
        }

        // Test that error messages are properly formatted and readable
        let long_message = "This is a very long error message that should still be properly formatted and readable even when it exceeds typical line lengths and contains various types of information";
        let config_error = AppError::Config(long_message.to_string());
        let display = format!("{}", config_error);

        assert!(
            display.contains(long_message),
            "Long error messages should be preserved completely"
        );
        assert!(
            display.len() > long_message.len(),
            "Error display should include both prefix and original message"
        );

        // Test special character handling
        let special_chars = "Message with\ttabs\nand newlines\rand\0null bytes";
        let journal_error = AppError::Journal(special_chars.to_string());
        let display = format!("{}", journal_error);

        assert!(
            display.contains("tabs"),
            "Error display should preserve readable parts of special character messages"
        );
        assert!(
            display.starts_with("Journal logic error: "),
            "Error display should maintain proper formatting even with special characters"
        );
    }
}
