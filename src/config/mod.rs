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
use std::fmt;
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

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("editor", &"[REDACTED_COMMAND]")
            .field("journal_dir", &"[REDACTED_PATH]")
            .finish()
    }
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

    /// Validates an editor command string for security.
    ///
    /// This function checks that the editor command:
    /// - Is not empty
    /// - Contains no shell metacharacters
    /// - Contains no spaces
    ///
    /// # Arguments
    ///
    /// * `editor_cmd` - The editor command string to validate
    ///
    /// # Returns
    ///
    /// A Result containing the validated editor command or an AppError
    fn validate_editor_command(editor_cmd: &str) -> AppResult<&str> {
        // Check for empty string
        if editor_cmd.is_empty() {
            return Err(AppError::Config(
                "Editor command cannot be empty".to_string(),
            ));
        }

        // Check for spaces
        if editor_cmd.contains(' ') {
            return Err(AppError::Config(
                "Editor command cannot contain spaces. Use a wrapper script or shell alias for editors requiring arguments".to_string(),
            ));
        }

        // Check for shell metacharacters
        const FORBIDDEN_CHARS: &[char] =
            &['|', '&', ';', '$', '(', ')', '`', '\\', '<', '>', '\'', '"'];

        for &ch in FORBIDDEN_CHARS.iter() {
            if editor_cmd.contains(ch) {
                return Err(AppError::Config(format!(
                    "Editor command cannot contain shell metacharacters: '{}'. Use a wrapper script or shell alias instead",
                    ch
                )));
            }
        }

        Ok(editor_cmd)
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
    /// Returns `AppError::Config` if:
    /// - The journal directory path expansion fails
    /// - The editor command fails validation (empty, contains spaces or shell metacharacters)
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
        let editor_raw = env::var("PONDER_EDITOR")
            .or_else(|_| env::var("EDITOR"))
            .unwrap_or_else(|_| "vim".to_string());

        // Validate the editor command
        let editor = Config::validate_editor_command(&editor_raw)?;

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
            editor: editor.to_string(),
            journal_dir,
        };

        Ok(config)
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
    /// - "Journal directory path is empty" if the journal directory path is empty
    /// - "Editor command is empty" if the editor is empty
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
    use crate::journal_io;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_debug_impl_redacts_sensitive_info() {
        // Create config with sensitive information
        let config = Config {
            editor: "vim".to_string(),
            journal_dir: PathBuf::from("/home/username/private/journal"),
        };

        // Format it with debug
        let debug_output = format!("{:?}", config);

        // Verify sensitive fields are redacted
        assert!(debug_output.contains("[REDACTED_COMMAND]"));
        assert!(debug_output.contains("[REDACTED_PATH]"));

        // Verify actual values are not present
        assert!(!debug_output.contains("vim"));
        assert!(!debug_output.contains("/home/username/private/journal"));
    }

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
    #[serial]
    fn test_load_with_default_editor() {
        // Fully reset environment variables
        setup();

        // Store original environment variables to restore later
        let orig_editor = env::var("EDITOR").ok();
        let orig_ponder_editor = env::var("PONDER_EDITOR").ok();

        // Explicitly unset both variables
        env::remove_var("EDITOR");
        env::remove_var("PONDER_EDITOR");

        // Set EDITOR to nano for this test
        env::set_var("EDITOR", "nano");

        // Run the test
        let config = Config::load().unwrap();

        // Restore environment
        env::remove_var("EDITOR");
        if let Some(val) = orig_editor {
            env::set_var("EDITOR", val);
        }
        if let Some(val) = orig_ponder_editor {
            env::set_var("PONDER_EDITOR", val);
        }

        // If EDITOR is set to nano, we expect the config to use nano
        assert_eq!(config.editor, "nano");
    }

    #[test]
    #[serial]
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
    #[serial]
    fn test_load_with_custom_dir() {
        setup();

        // Store original environment variable to restore later
        let orig_ponder_dir = env::var("PONDER_DIR").ok();

        // Create a temp directory to use as journal dir
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        env::set_var("PONDER_DIR", &dir_path);
        let config = Config::load().unwrap();

        // Restore environment
        if let Some(val) = orig_ponder_dir {
            env::set_var("PONDER_DIR", val);
        } else {
            env::remove_var("PONDER_DIR");
        }

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
        journal_io::ensure_journal_directory_exists(&config.journal_dir).unwrap();

        // Now it should exist
        assert!(dir_path.exists());
    }

    #[test]
    fn test_validate_editor_command_valid() {
        assert_eq!(Config::validate_editor_command("vim").unwrap(), "vim");
        assert_eq!(Config::validate_editor_command("nano").unwrap(), "nano");
        assert_eq!(
            Config::validate_editor_command("/usr/bin/code").unwrap(),
            "/usr/bin/code"
        );
        assert_eq!(
            Config::validate_editor_command("./my-editor").unwrap(),
            "./my-editor"
        );
    }

    #[test]
    fn test_validate_editor_command_empty() {
        let result = Config::validate_editor_command("");
        assert!(result.is_err());
        match result {
            Err(AppError::Config(msg)) => assert!(msg.contains("cannot be empty")),
            _ => panic!("Expected Config error for empty command"),
        }
    }

    #[test]
    fn test_validate_editor_command_with_spaces() {
        let result = Config::validate_editor_command("vim --noplugin");
        assert!(result.is_err());
        match result {
            Err(AppError::Config(msg)) => assert!(msg.contains("cannot contain spaces")),
            _ => panic!("Expected Config error for command with spaces"),
        }
    }

    #[test]
    fn test_validate_editor_command_with_metacharacters() {
        // Test commands without spaces (so they fail on metacharacters, not spaces)
        let test_cases = [
            ("echo>/tmp/file", '>'),
            ("echo|cat", '|'),
            ("sh&echo", '&'),
            ("vim;echo", ';'),
            ("$(echo)", '$'),
            ("`echo`", '`'),
            ("vim&", '&'),
            ("vim'~/test'", '\''),
            ("vim\"test\"", '"'),
            ("vim(test)", '('),
            ("vim)test", ')'),
            ("vim\\test", '\\'),
            ("vim<file", '<'),
        ];

        for (cmd, char) in test_cases.iter() {
            let result = Config::validate_editor_command(cmd);
            assert!(result.is_err());
            match result {
                Err(AppError::Config(msg)) => {
                    assert!(msg.contains("Editor command cannot contain shell metacharacters"));
                    assert!(msg.contains(&char.to_string()));
                }
                _ => panic!("Expected Config error for metacharacter '{}'", char),
            }
        }
    }

    #[test]
    #[serial]
    fn test_load_config_with_invalid_editor() {
        // Store original values
        let orig_ponder_editor = env::var("PONDER_EDITOR").ok();
        let orig_editor = env::var("EDITOR").ok();
        let orig_ponder_dir = env::var("PONDER_DIR").ok();

        // Clear environment first
        env::remove_var("PONDER_EDITOR");
        env::remove_var("EDITOR");

        // Set invalid editor
        env::set_var("PONDER_EDITOR", "vim --noplugin");
        env::set_var("PONDER_DIR", "/tmp");

        let result = Config::load();
        assert!(result.is_err());
        match result {
            Err(AppError::Config(msg)) => assert!(msg.contains("cannot contain spaces")),
            _ => panic!("Expected Config error for invalid editor"),
        }

        // Test with metacharacters
        env::set_var("PONDER_EDITOR", "echo>/tmp/pwned");
        let result = Config::load();
        assert!(result.is_err());
        match result {
            Err(AppError::Config(msg)) => {
                assert!(msg.contains("Editor command cannot contain shell metacharacters"))
            }
            _ => panic!("Expected Config error for metacharacters"),
        }

        // Restore original values
        if let Some(val) = orig_ponder_editor {
            env::set_var("PONDER_EDITOR", val);
        } else {
            env::remove_var("PONDER_EDITOR");
        }
        if let Some(val) = orig_editor {
            env::set_var("EDITOR", val);
        } else {
            env::remove_var("EDITOR");
        }
        if let Some(val) = orig_ponder_dir {
            env::set_var("PONDER_DIR", val);
        } else {
            env::remove_var("PONDER_DIR");
        }
    }
}
