use serial_test::serial;
use std::env;
use std::path::PathBuf;
use tempfile::tempdir;

use ponder::config::Config;
use ponder::errors::{AppError, AppResult};

#[test]
#[serial]
fn test_config_load_with_environment_vars() {
    // Save the original environment variables
    let original_ponder_dir = env::var("PONDER_DIR").ok();
    let original_ponder_editor = env::var("PONDER_EDITOR").ok();
    let original_editor = env::var("EDITOR").ok();

    // Set environment variables for the test
    let temp_dir = tempdir().unwrap();
    let dir_path = temp_dir.path().to_string_lossy().to_string();

    env::set_var("PONDER_DIR", &dir_path);
    env::set_var("PONDER_EDITOR", "test-editor");

    // Load the configuration
    let config = Config::load().unwrap();

    // Verify the config values match the environment variables
    assert_eq!(config.editor, "test-editor");
    assert_eq!(config.journal_dir, PathBuf::from(&dir_path));

    // Restore the original environment variables
    match original_ponder_dir {
        Some(val) => env::set_var("PONDER_DIR", val),
        None => env::remove_var("PONDER_DIR"),
    }

    match original_ponder_editor {
        Some(val) => env::set_var("PONDER_EDITOR", val),
        None => env::remove_var("PONDER_EDITOR"),
    }

    match original_editor {
        Some(val) => env::set_var("EDITOR", val),
        None => env::remove_var("EDITOR"),
    }
}

#[test]
#[serial]
fn test_config_load_with_fallbacks() {
    // Save the original environment variables
    let original_ponder_dir = env::var("PONDER_DIR").ok();
    let original_ponder_editor = env::var("PONDER_EDITOR").ok();
    let original_editor = env::var("EDITOR").ok();
    let original_home = env::var("HOME").ok();

    // Remove environment variables to test fallbacks
    env::remove_var("PONDER_DIR");
    env::remove_var("PONDER_EDITOR");
    env::remove_var("EDITOR");

    // Set HOME for predictable fallback path
    let temp_dir = tempdir().unwrap();
    let home_path = temp_dir.path().to_string_lossy().to_string();
    env::set_var("HOME", &home_path);

    // Load the configuration
    let config = Config::load().unwrap();

    // Verify fallback values
    assert_eq!(config.editor, "vim");

    // Expected fallback path is ~/Documents/rubberducks
    let expected_journal_dir = PathBuf::from(&home_path)
        .join("Documents")
        .join("rubberducks");
    assert_eq!(config.journal_dir, expected_journal_dir);

    // Restore the original environment variables
    match original_ponder_dir {
        Some(val) => env::set_var("PONDER_DIR", val),
        None => env::remove_var("PONDER_DIR"),
    }

    match original_ponder_editor {
        Some(val) => env::set_var("PONDER_EDITOR", val),
        None => env::remove_var("PONDER_EDITOR"),
    }

    match original_editor {
        Some(val) => env::set_var("EDITOR", val),
        None => env::remove_var("EDITOR"),
    }

    match original_home {
        Some(val) => env::set_var("HOME", val),
        None => env::remove_var("HOME"),
    }
}

#[test]
#[serial]
fn test_config_validation() -> AppResult<()> {
    // Test valid configuration
    let valid_config = Config {
        editor: "vim".to_string(),
        journal_dir: PathBuf::from("/absolute/path"),
    };
    valid_config.validate()?;

    // Test empty editor
    let invalid_editor_config = Config {
        editor: "".to_string(),
        journal_dir: PathBuf::from("/absolute/path"),
    };
    let result = invalid_editor_config.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::Config(msg)) => {
            assert!(msg.contains("Editor command is empty"));
        }
        _ => panic!("Expected Config error about empty editor"),
    }

    // Test relative path
    let relative_path_config = Config {
        editor: "vim".to_string(),
        journal_dir: PathBuf::from("relative/path"),
    };
    let result = relative_path_config.validate();
    assert!(result.is_err());
    match result {
        Err(AppError::Config(msg)) => {
            assert!(msg.contains("must be an absolute path"));
        }
        _ => panic!("Expected Config error about relative path"),
    }

    Ok(())
}

#[test]
#[serial]
fn test_ensure_journal_dir() -> AppResult<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path().to_string_lossy().to_string();
    let journal_dir = PathBuf::from(&base_path).join("journals");

    // Create a config with the temporary journal directory
    let config = Config {
        editor: "vim".to_string(),
        journal_dir: journal_dir.clone(),
    };

    // Directory shouldn't exist yet
    assert!(!journal_dir.exists());

    // Call ensure_journal_dir to create it
    config.ensure_journal_dir()?;

    // Now the directory should exist
    assert!(journal_dir.exists());

    Ok(())
}
