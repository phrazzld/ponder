use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// We need to import the actual library code
use ponder::config::Config;
use ponder::editor::SystemEditor;
use ponder::errors::AppResult;
use ponder::journal::io::FileSystemIO;
use ponder::journal::JournalService;

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
fn test_journal_service_basic_flow() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Ensure the journal directory exists
    config.ensure_journal_dir()?;

    // Set up the required dependencies
    let io = Box::new(FileSystemIO {
        journal_dir: config.journal_dir.to_string_lossy().to_string(),
    });

    let editor = Box::new(SystemEditor {
        editor_cmd: config.editor.clone(),
    });

    // Create the journal service
    let journal_service = JournalService::new(config, io, editor);

    // Test opening today's entry
    journal_service.open_entry()?;

    // Verify that a journal file was created for today
    let journal_dir = journal_service.get_journal_dir();
    let dir_entries = fs::read_dir(journal_dir).unwrap();

    // Should be at least one entry (today's)
    assert!(dir_entries.count() > 0);

    Ok(())
}

#[test]
#[serial]
fn test_journal_service_specific_date() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Ensure the journal directory exists
    config.ensure_journal_dir()?;

    // Set up the required dependencies
    let io = Box::new(FileSystemIO {
        journal_dir: config.journal_dir.to_string_lossy().to_string(),
    });

    let editor = Box::new(SystemEditor {
        editor_cmd: config.editor.clone(),
    });

    // Create the journal service
    let journal_service = JournalService::new(config, io, editor);

    // Test opening entry for a specific date
    use chrono::NaiveDate;
    let specific_date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
    journal_service.open_specific_entry(specific_date)?;

    // Verify that a journal file was created for the specific date
    let journal_dir = journal_service.get_journal_dir();
    let expected_file = journal_dir.join("20230115.md");

    assert!(expected_file.exists());

    Ok(())
}

#[test]
#[serial]
fn test_journal_service_retro() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Ensure the journal directory exists
    config.ensure_journal_dir()?;

    // Set up the required dependencies
    let io = Box::new(FileSystemIO {
        journal_dir: config.journal_dir.to_string_lossy().to_string(),
    });

    let editor = Box::new(SystemEditor {
        editor_cmd: config.editor.clone(),
    });

    // Create the journal service
    let journal_service = JournalService::new(config, io, editor);

    // Since we're just creating the directory, there shouldn't be any retro entries
    journal_service.open_retro_entry()?;

    // No assertion needed - we're just checking that it doesn't panic

    Ok(())
}

#[test]
#[serial]
fn test_journal_service_reminisce() -> AppResult<()> {
    let (config, _temp_dir) = set_up_test_env();

    // Ensure the journal directory exists
    config.ensure_journal_dir()?;

    // Set up the required dependencies
    let io = Box::new(FileSystemIO {
        journal_dir: config.journal_dir.to_string_lossy().to_string(),
    });

    let editor = Box::new(SystemEditor {
        editor_cmd: config.editor.clone(),
    });

    // Create the journal service
    let journal_service = JournalService::new(config, io, editor);

    // Since we're just creating the directory, there shouldn't be any reminisce entries
    journal_service.open_reminisce_entry()?;

    // No assertion needed - we're just checking that it doesn't panic

    Ok(())
}
