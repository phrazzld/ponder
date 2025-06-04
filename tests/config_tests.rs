use serial_test::serial;
use std::env;
use std::path::PathBuf;
use tempfile::tempdir;

use ponder::config::Config;
use ponder::errors::{AppError, AppResult};
use ponder::journal_io;

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
    // Test behavior: Empty editor configuration should be rejected
    // This validates that the application requires a valid editor to function
    match result {
        Err(AppError::Config(msg)) => {
            // Use robust pattern matching: focus on essential validation concepts
            // Test for empty/missing editor validation, not exact wording
            let msg_lower = msg.to_lowercase();
            assert!(
                msg_lower.contains("editor")
                    && (msg_lower.contains("empty")
                        || msg_lower.contains("missing")
                        || msg_lower.contains("required")),
                "Config error should indicate empty/missing editor issue, got: {}",
                msg
            );
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
    // Test behavior: Relative path configuration should be rejected
    // This validates that journal directory must be an absolute path for security/consistency
    match result {
        Err(AppError::Config(msg)) => {
            // Use robust pattern matching: focus on essential path validation concepts
            // Test for path requirement validation, not exact wording
            let msg_lower = msg.to_lowercase();
            assert!(
                (msg_lower.contains("path") || msg_lower.contains("directory"))
                    && (msg_lower.contains("absolute")
                        || msg_lower.contains("relative")
                        || msg_lower.contains("must")),
                "Config error should indicate path validation issue, got: {}",
                msg
            );
        }
        _ => panic!("Expected Config error about relative path"),
    }

    Ok(())
}

#[test]
#[serial]
fn test_ensure_journal_directory_exists() -> AppResult<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path().to_string_lossy().to_string();
    let journal_dir = PathBuf::from(&base_path).join("journals");

    // We no longer need to create the Config object since we're using
    // journal_io::ensure_journal_directory_exists directly

    // Directory shouldn't exist yet
    assert!(!journal_dir.exists());

    // Call ensure_journal_directory_exists to create it
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Now the directory should exist
    assert!(journal_dir.exists());

    Ok(())
}
