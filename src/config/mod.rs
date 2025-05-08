use crate::errors::{AppError, AppResult};
use std::env;
use std::path::PathBuf;
use std::fs;

/// Configuration for the ponder application
pub struct Config {
    /// Editor command to use for opening journal entries (from EDITOR env var or default)
    pub editor: String,
    
    /// Directory where journal entries are stored
    pub journal_dir: PathBuf,
}

impl Config {
    /// Creates a new Config with default values
    pub fn new() -> Self {
        Config {
            editor: "vim".to_string(),
            journal_dir: PathBuf::from(""),
        }
    }
    
    /// Loads configuration from environment variables with sensible defaults
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
            return Err(AppError::Config("Journal directory path is empty".to_string()));
        }
        
        let config = Config {
            editor,
            journal_dir,
        };
        
        Ok(config)
    }
    
    /// Ensures the journal directory exists, creating it if necessary
    pub fn ensure_journal_dir(&self) -> AppResult<()> {
        if !self.journal_dir.exists() {
            fs::create_dir_all(&self.journal_dir)
                .map_err(|e| AppError::Config(
                    format!("Failed to create journal directory: {}", e)
                ))?;
        }
        
        Ok(())
    }
    
    /// Validates that the configuration is usable
    pub fn validate(&self) -> AppResult<()> {
        // Check that journal_dir is valid
        if self.journal_dir.as_os_str().is_empty() {
            return Err(AppError::Config("Journal directory path is empty".to_string()));
        }
        
        // Check that editor is not empty
        if self.editor.is_empty() {
            return Err(AppError::Config("Editor command is empty".to_string()));
        }
        
        // Journal directory must be absolute
        if !self.journal_dir.is_absolute() {
            return Err(AppError::Config("Journal directory must be an absolute path".to_string()));
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
        setup();
        
        // Neither PONDER_EDITOR nor EDITOR is set
        let config = Config::load().unwrap();
        assert_eq!(config.editor, "vim");
    }

    #[test]
    fn test_load_with_editor_env() {
        setup();
        
        // Set EDITOR
        env::set_var("EDITOR", "nano");
        let config = Config::load().unwrap();
        assert_eq!(config.editor, "nano");
        
        // PONDER_EDITOR should take precedence
        env::set_var("PONDER_EDITOR", "code");
        let config = Config::load().unwrap();
        assert_eq!(config.editor, "code");
        
        // Clean up environment variables
        env::remove_var("EDITOR");
        env::remove_var("PONDER_EDITOR");
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
            },
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
            },
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
            },
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