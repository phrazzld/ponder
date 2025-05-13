//! Configuration management for the ponder application.
//!
//! This module handles loading and validating configuration settings from environment
//! variables, with sensible defaults. It supports configuring the journal directory
//! and the editor command used to open journal files.
//!
//! # Environment Variables
//!
//! - `PONDER_DIR`: Path to the journal directory (defaults to ~/Documents/rubberducks)
//! - `PONDER_EDITOR`: Editor to use for journal entries
//! - `EDITOR`: Fallback editor if PONDER_EDITOR is not set (defaults to "vim")
//! - `HOME`: Used for expanding the default journal directory path

use crate::errors::{AppError, AppResult};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Configuration for the ponder application.
///
/// This struct holds the configuration settings needed for the application, including
/// the editor command to use for opening journal entries and the directory where
/// journal entries are stored.
///
/// # Examples
///
/// Creating a configuration manually:
/// ```
/// use ponder::Config;
/// use std::path::PathBuf;
///
/// let config = Config {
///     editor: "nano".to_string(),
///     journal_dir: PathBuf::from("/path/to/journal"),
/// };
/// ```
///
/// Loading configuration from environment variables:
/// ```no_run
/// use ponder::Config;
/// use std::env;
///
/// // Set environment variables
/// env::set_var("PONDER_EDITOR", "code");
/// env::set_var("PONDER_DIR", "/custom/journal/path");
///
/// // Load configuration
/// let config = Config::load().expect("Failed to load configuration");
/// assert_eq!(config.editor, "code");
/// ```
pub struct Config {
    /// Editor command to use for opening journal entries.
    ///
    /// This is loaded from environment variables in the following order of precedence:
    /// 1. PONDER_EDITOR
    /// 2. EDITOR
    /// 3. Defaults to "vim" if neither is set
    pub editor: String,

    /// Directory where journal entries are stored.
    ///
    /// This is loaded from the PONDER_DIR environment variable with a fallback
    /// to ~/Documents/rubberducks if not specified.
    pub journal_dir: PathBuf,
}

impl Default for Config {
    /// Creates a new Config with default values.
    fn default() -> Self {
        Config {
            editor: "vim".to_string(),
            journal_dir: PathBuf::from(""),
        }
    }
}

impl Config {
    /// Creates a new Config with default values.
    ///
    /// This is primarily useful for testing or when you want to start with defaults
    /// and then modify specific fields.
    ///
    /// # Returns
    ///
    /// A new `Config` instance with "vim" as the editor and an empty path for the journal directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use ponder::Config;
    ///
    /// let config = Config::new();
    /// assert_eq!(config.editor, "vim");
    /// ```
    ///
    /// Note: For normal application usage, prefer `Config::load()` which populates
    /// the configuration from environment variables with sensible defaults.
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from environment variables with sensible defaults.
    ///
    /// This method reads configuration from environment variables, with fallbacks
    /// for missing values. It will expand the journal directory path using `shellexpand`
    /// to handle `~` and environment variable references.
    ///
    /// # Environment Variables
    ///
    /// - `PONDER_EDITOR`: Primary editor command to use
    /// - `EDITOR`: Fallback editor if PONDER_EDITOR is not set
    /// - `PONDER_DIR`: Journal directory path (defaults to ~/Documents/rubberducks)
    ///
    /// # Returns
    ///
    /// A Result containing either the loaded Config or an AppError if path expansion fails.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Config` if the journal directory path expansion fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ponder::Config;
    ///
    /// match Config::load() {
    ///     Ok(config) => println!("Loaded config with editor: {}", config.editor),
    ///     Err(err) => eprintln!("Failed to load config: {}", err),
    /// }
    /// ```
    pub fn load() -> AppResult<Self> {
        // Get editor from EDITOR or PONDER_EDITOR env vars, fallback to vim
        let editor = env::var("PONDER_EDITOR")
            .or_else(|_| env::var("EDITOR"))
            .unwrap_or_else(|_| "vim".to_string());

        // Get journal directory from PONDER_DIR env var, fallback to ~/Documents/rubberducks
        let journal_dir_str = env::var("PONDER_DIR").unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "".to_string());
            format!("{}/Documents/rubberducks", home)
        });

        // Expand the path (handles ~ and environment variables)
        let expanded_path = shellexpand::full(&journal_dir_str)
            .map_err(|e| AppError::Config(format!("Failed to expand path: {}", e)))?;

        let journal_dir = PathBuf::from(expanded_path.into_owned());

        // Validate the configuration
        if journal_dir.as_os_str().is_empty() {
            return Err(AppError::Config(
                "Journal directory path is empty".to_string(),
            ));
        }

        let config = Config {
            editor,
            journal_dir,
        };

        Ok(config)
    }

    /// Ensures the journal directory exists, creating it if necessary.
    ///
    /// This method checks if the configured journal directory exists and creates it
    /// (including any parent directories) if it doesn't.
    ///
    /// # Returns
    ///
    /// A Result that is Ok(()) if the directory exists or was successfully created,
    /// or an AppError if directory creation failed.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Config` if directory creation fails for any reason
    /// (e.g., insufficient permissions).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ponder::Config;
    /// use std::path::PathBuf;
    ///
    /// let config = Config {
    ///     editor: "vim".to_string(),
    ///     journal_dir: PathBuf::from("/tmp/my_journal"),
    /// };
    ///
    /// // Make sure the directory exists
    /// config.ensure_journal_dir().expect("Failed to create journal directory");
    /// ```
    pub fn ensure_journal_dir(&self) -> AppResult<()> {
        if !self.journal_dir.exists() {
            fs::create_dir_all(&self.journal_dir).map_err(|e| {
                AppError::Config(format!("Failed to create journal directory: {}", e))
            })?;
        }

        Ok(())
    }

    /// Validates that the configuration is usable.
    ///
    /// This method checks if the configuration meets the minimum requirements:
    /// - Editor command is not empty
    /// - Journal directory path is not empty
    /// - Journal directory path is absolute
    ///
    /// # Returns
    ///
    /// A Result that is Ok(()) if the configuration is valid,
    /// or an AppError with a description of what is invalid.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Config` with one of the following messages:
    /// - "Editor command is empty" if the editor is empty
    /// - "Journal directory path is empty" if the journal directory path is empty
    /// - "Journal directory must be an absolute path" if the path is relative
    ///
    /// # Examples
    ///
    /// ```
    /// use ponder::Config;
    /// use std::path::PathBuf;
    ///
    /// // Valid configuration
    /// let valid_config = Config {
    ///     editor: "vim".to_string(),
    ///     journal_dir: PathBuf::from("/absolute/path"),
    /// };
    /// assert!(valid_config.validate().is_ok());
    ///
    /// // Invalid configuration (empty editor)
    /// let invalid_config = Config {
    ///     editor: "".to_string(),
    ///     journal_dir: PathBuf::from("/absolute/path"),
    /// };
    /// assert!(invalid_config.validate().is_err());
    /// ```
    pub fn validate(&self) -> AppResult<()> {
        // Check that journal_dir is valid
        if self.journal_dir.as_os_str().is_empty() {
            return Err(AppError::Config(
                "Journal directory path is empty".to_string(),
            ));
        }

        // Check that editor is not empty
        if self.editor.is_empty() {
            return Err(AppError::Config("Editor command is empty".to_string()));
        }

        // Journal directory must be absolute
        if !self.journal_dir.is_absolute() {
            return Err(AppError::Config(
                "Journal directory must be an absolute path".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    fn setup() {
        // Clear relevant environment variables before each test
        env::remove_var("PONDER_EDITOR");
        env::remove_var("EDITOR");
        env::remove_var("PONDER_DIR");
    }

    #[test]
    fn test_new_config_defaults() {
        let config = Config::new();
        assert_eq!(config.editor, "vim");
        assert_eq!(config.journal_dir, PathBuf::from(""));
    }

    #[test]
    fn test_load_with_default_editor() {
        // Fully reset environment variables
        setup();

        // Store original environment variables to restore later
        let orig_editor = env::var("EDITOR").ok();
        let orig_ponder_editor = env::var("PONDER_EDITOR").ok();

        // Explicitly unset both variables
        env::remove_var("EDITOR");
        env::remove_var("PONDER_EDITOR");

        // Run the test
        let config = Config::load().unwrap();

        // Restore environment
        if let Some(val) = orig_editor {
            env::set_var("EDITOR", val);
        }
        if let Some(val) = orig_ponder_editor {
            env::set_var("PONDER_EDITOR", val);
        }

        assert_eq!(config.editor, "vim");
    }

    #[test]
    fn test_load_with_editor_env() {
        // Fully reset environment variables
        setup();

        // Store original environment variables to restore later
        let orig_editor = env::var("EDITOR").ok();
        let orig_ponder_editor = env::var("PONDER_EDITOR").ok();
        let orig_ponder_dir = env::var("PONDER_DIR").ok();

        // Explicitly unset all variables
        env::remove_var("PONDER_EDITOR");
        env::remove_var("EDITOR");
        env::remove_var("PONDER_DIR");

        // Test EDITOR environment variable
        env::set_var("EDITOR", "nano");
        let config = Config::load().unwrap();
        assert_eq!(config.editor, "nano");

        // Test PONDER_EDITOR taking precedence
        env::set_var("PONDER_EDITOR", "code");
        let config = Config::load().unwrap();
        assert_eq!(config.editor, "code");

        // Restore environment
        env::remove_var("EDITOR");
        env::remove_var("PONDER_EDITOR");

        if let Some(val) = orig_editor {
            env::set_var("EDITOR", val);
        }
        if let Some(val) = orig_ponder_editor {
            env::set_var("PONDER_EDITOR", val);
        }
        if let Some(val) = orig_ponder_dir {
            env::set_var("PONDER_DIR", val);
        }
    }

    #[test]
    fn test_load_with_custom_dir() {
        setup();

        // Create a temp directory to use as journal dir
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        env::set_var("PONDER_DIR", &dir_path);
        let config = Config::load().unwrap();

        assert_eq!(config.journal_dir, PathBuf::from(dir_path));
    }

    #[test]
    fn test_validate_valid_config() {
        setup();

        // Create a temp directory to use as journal dir
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        let config = Config {
            editor: "vim".to_string(),
            journal_dir: PathBuf::from(dir_path),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_editor() {
        let config = Config {
            editor: "".to_string(),
            journal_dir: PathBuf::from("/some/path"),
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(AppError::Config(message)) => {
                assert!(message.contains("Editor command is empty"));
            }
            _ => panic!("Expected Config error about empty editor"),
        }
    }

    #[test]
    fn test_validate_empty_journal_dir() {
        let config = Config {
            editor: "vim".to_string(),
            journal_dir: PathBuf::from(""),
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(AppError::Config(message)) => {
                assert!(message.contains("Journal directory path is empty"));
            }
            _ => panic!("Expected Config error about empty journal directory"),
        }
    }

    #[test]
    fn test_validate_relative_journal_dir() {
        let config = Config {
            editor: "vim".to_string(),
            journal_dir: PathBuf::from("relative/path"),
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(AppError::Config(message)) => {
                assert!(message.contains("must be an absolute path"));
            }
            _ => panic!("Expected Config error about relative path"),
        }
    }

    #[test]
    fn test_ensure_journal_dir() {
        // Create a temp directory
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().join("journal");

        let config = Config {
            editor: "vim".to_string(),
            journal_dir: dir_path.clone(),
        };

        // Directory shouldn't exist yet
        assert!(!dir_path.exists());

        // Should create the directory
        config.ensure_journal_dir().unwrap();

        // Now it should exist
        assert!(dir_path.exists());
    }
}
