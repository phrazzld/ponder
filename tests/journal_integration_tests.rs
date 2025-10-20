use serial_test::serial;
use std::fs;
use tempfile::tempdir;

// We need to import the actual library code
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use ponder::config::Config;
use ponder::errors::AppResult;
use ponder::journal_core::DateSpecifier;
use ponder::journal_io;

// Fixed test date for deterministic testing
// Using 2024-01-15 14:30:00 as our reference datetime
fn get_fixed_test_datetime() -> DateTime<Local> {
    use chrono::NaiveDate;
    // Create a fixed date and time
    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let time = date.and_hms_opt(14, 30, 0).unwrap();
    // Convert to local timezone
    Local.from_local_datetime(&time).single().unwrap()
}

// Helper function to set up a test environment
fn set_up_test_env() -> Result<(Config, tempfile::TempDir), Box<dyn std::error::Error>> {
    // Create a temporary directory for the journal files
    let temp_dir = tempdir()?;
    let dir_path = temp_dir.path().to_path_buf();

    // Create a Config instance pointing to the temp directory
    let config = Config {
        editor: "echo".to_string(),
        journal_dir: dir_path.clone(),
        db_path: dir_path.join("ponder.db"),
        ..Config::default()
    };

    Ok((config, temp_dir))
}

#[test]
#[serial]
fn test_journal_basic_flow() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env()
        .map_err(|e| ponder::errors::AppError::Journal(format!("Setup failed: {}", e)))?;

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Use fixed date/time for deterministic testing
    let current_datetime = get_fixed_test_datetime();
    let today = current_datetime.naive_local().date();

    // Test opening today's entry
    journal_io::open_journal_entries(&config, &[today], &current_datetime)?;

    // Verify that a journal file was created for today
    let dir_entries = fs::read_dir(&journal_dir).map_err(|e| {
        ponder::errors::AppError::Journal(format!("Failed to read journal directory: {}", e))
    })?;

    // Should be at least one entry (today's)
    let entry_count = dir_entries.count();
    assert!(
        entry_count > 0,
        "Journal directory should contain at least one entry after journal creation. \
        Fixed test date: 2024-01-15, Journal directory: {}",
        journal_dir.display()
    );

    Ok(())
}

#[test]
#[serial]
fn test_journal_specific_date() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env()
        .map_err(|e| ponder::errors::AppError::Journal(format!("Setup failed: {}", e)))?;

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Use fixed date/time for deterministic testing
    let current_datetime = get_fixed_test_datetime();

    // Test opening entry for a specific date
    let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).ok_or_else(|| {
        ponder::errors::AppError::Journal("Failed to create test date".to_string())
    })?;
    journal_io::open_journal_entries(&config, &[specific_date], &current_datetime)?;

    // Verify that a journal file was created for the specific date
    let expected_file = journal_dir.join("20230115.md");

    assert!(
        expected_file.exists(),
        "Expected journal file was not created for specific date. \
        Expected file: {}, Journal directory: {}",
        expected_file.display(),
        journal_dir.display()
    );

    Ok(())
}

#[test]
#[serial]
fn test_journal_retro() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env()
        .map_err(|e| ponder::errors::AppError::Journal(format!("Setup failed: {}", e)))?;

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Use fixed date/time for deterministic testing
    let current_datetime = get_fixed_test_datetime();
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
    let (config, _temp_dir) = set_up_test_env()
        .map_err(|e| ponder::errors::AppError::Journal(format!("Setup failed: {}", e)))?;

    // Create copies of the config values so we can check results
    let journal_dir = config.journal_dir.clone();

    // Ensure journal directory exists
    journal_io::ensure_journal_directory_exists(&journal_dir)?;

    // Use fixed date/time for deterministic testing
    let current_datetime = get_fixed_test_datetime();
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
