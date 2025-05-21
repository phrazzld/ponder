use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// We need to import the actual library code
use chrono::{Local, NaiveDate};
use ponder::config::Config;
use ponder::errors::AppResult;
use ponder::journal_core::DateSpecifier;
use ponder::journal_io;

// Helper function to set up a test environment
fn set_up_test_env() -> (Config, tempfile::TempDir) {
    // Create a temporary directory for the journal files
    let temp_dir = tempdir().unwrap();
    let dir_path = temp_dir.path().to_string_lossy().to_string();

    // Use "echo" as a safe editor for testing
    let editor = "echo".to_string();

    // Create a Config instance pointing to the temp directory
    let config = Config {
        editor,
        journal_dir: PathBuf::from(&dir_path),
    };

    (config, temp_dir)
}

#[test]
#[serial]
fn test_journal_basic_flow() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Get the current date/time for reference
    let current_datetime = Local::now();
    let today = current_datetime.naive_local().date();

    // Test opening today's entry
    journal_io::open_journal_entries(&config, &[today], &current_datetime)?;

    // Verify that a journal file was created for today
    let dir_entries = fs::read_dir(&journal_dir).unwrap();

    // Should be at least one entry (today's)
    assert!(dir_entries.count() > 0);

    Ok(())
}

#[test]
#[serial]
fn test_journal_specific_date() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Get the current date/time for reference
    let current_datetime = Local::now();

    // Test opening entry for a specific date
    let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
    journal_io::open_journal_entries(&config, &[specific_date], &current_datetime)?;

    // Verify that a journal file was created for the specific date
    let expected_file = journal_dir.join("20230115.md");

    assert!(expected_file.exists());

    Ok(())
}

#[test]
#[serial]
fn test_journal_retro() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Get the current date/time for reference
    let current_datetime = Local::now();
    let reference_date = current_datetime.naive_local().date();

    // Since we're just creating the directory, there shouldn't be any retro entries
    let spec = DateSpecifier::Retro;
    let dates = spec.resolve_dates(reference_date);
    journal_io::open_journal_entries(&config, &dates, &current_datetime)?;

    // No assertion needed - we're just checking that it doesn't panic

    Ok(())
}

#[test]
#[serial]
fn test_journal_reminisce() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Get the current date/time for reference
    let current_datetime = Local::now();
    let reference_date = current_datetime.naive_local().date();

    // Since we're just creating the directory, there shouldn't be any reminisce entries
    let spec = DateSpecifier::Reminisce;
    let dates = spec.resolve_dates(reference_date);
    journal_io::open_journal_entries(&config, &dates, &current_datetime)?;

    // No assertion needed - we're just checking that it doesn't panic

    Ok(())
}

#[test]
#[serial]
fn test_relative_journal_path_rejected() {
    use ponder::errors::AppError;
    use std::path::Path;

    // Create a relative path to test
    let relative_path = Path::new("relative/path/to/journal");

    // Call ensure_journal_directory_exists with the relative path
    let result = journal_io::ensure_journal_directory_exists(relative_path);

    // The function should reject the relative path and return an error
    assert!(result.is_err());

    // Verify the error type and message
    match result {
        Err(AppError::Journal(msg)) => {
            assert!(msg.contains("must be absolute"));
            assert!(msg.contains("relative/path/to/journal"));
        }
        _ => panic!("Expected AppError::Journal variant"),
    }
}
