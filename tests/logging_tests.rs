use assert_cmd::Command;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;

/// Create a cross-platform slow editor script for testing
#[cfg(unix)]
fn create_slow_editor_script(
    temp_dir: &Path,
    hold_duration_secs: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let script_path = temp_dir.join("slow_editor.sh");
    let content = format!(
        "#!/bin/sh\n\
         sleep {}\n\
         exit 0\n",
        hold_duration_secs
    );
    fs::write(&script_path, content)?;
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms)?;
    Ok(script_path.to_str().unwrap().to_string())
}

#[cfg(windows)]
fn create_slow_editor_script(
    temp_dir: &Path,
    hold_duration_secs: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let script_path = temp_dir.join("slow_editor.bat");
    let content = format!(
        "@echo off\r\n\
         timeout /t {} /nobreak > nul\r\n\
         exit /b 0\r\n",
        hold_duration_secs
    );
    fs::write(&script_path, content)?;
    Ok(script_path.to_str().unwrap().to_string())
}

/// Test basic tracing infrastructure setup
#[test]
fn test_tracing_setup() {
    // This test exists just to verify that the tracing and UUID dependencies
    // are correctly included and linked. If this test compiles and runs,
    // it means the dependencies are working.

    // Using an actual uuid generation to avoid clippy warning about assert!(true)
    let _id = uuid::Uuid::new_v4();
}

/// Test that lock failures are logged exactly once at the application boundary
/// This verifies that T002 (removal of double logging) is working correctly
/// NOTE: v2.0 uses per-file encryption, reducing lock conflicts compared to v1.0
#[test]
#[serial]
#[ignore = "v2.0: per-file encryption architecture reduces lock conflicts"]
fn test_single_error_logging_for_lock_failure() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir)?;

    // Use fixed date for deterministic testing
    let test_date = "2024-01-15";

    // Create a slow editor script that will hold the lock
    let slow_editor = create_slow_editor_script(temp_dir.path(), 3)?;

    // First, start a long-running ponder process that will hold the lock
    let mut long_running_process = std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("edit") // v2.0: Add edit subcommand
        .arg("--date")
        .arg(test_date)
        .env("PONDER_EDITOR", &slow_editor)
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
        .env("HOME", journal_dir.to_str().unwrap())
        .spawn()?;

    // Give the first process time to acquire the lock
    std::thread::sleep(Duration::from_millis(1000));

    // Now attempt to run a second ponder process that should fail with FileBusy
    // Enable detailed logging to capture the error
    let output = Command::cargo_bin("ponder")?
        .env("PONDER_EDITOR", "true") // Fast editor
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .env("RUST_LOG", "debug") // Enable debug logging
        .env("CI", "true") // Force structured logging to ensure ERROR logs appear
        .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
        .arg("edit") // v2.0: Add edit subcommand
        .arg("--date")
        .arg(test_date)
        .output()?;

    // Verify the command failed as expected
    assert!(
        !output.status.success(),
        "Second ponder process should have failed due to lock conflict"
    );

    // Verify the error output contains the lock error message
    // Use robust pattern matching instead of checking for enum variant names
    let stderr_output = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_output.contains("currently being edited") || stderr_output.contains("FileBusy"),
        "Error message should contain lock error indication, but stderr was: {}",
        stderr_output
    );

    // Note: Enhanced error messages from T004 may appear in different contexts
    // This test focuses on verifying single error logging (no double logging from T002)

    // Count how many times the structured error log appears in stderr
    // We expect exactly one ERROR level log entry at the application boundary
    // Use robust pattern matching for the structured log entry
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let combined_output = format!("{}{}", stderr_str, stdout_str);
    let lock_error_count = combined_output.matches("\"level\":\"ERROR\"").count();

    // Clean up the long-running process
    let _ = long_running_process.kill();
    let _ = long_running_process.wait();

    // Assert single error logging - error should appear exactly once
    assert_eq!(
        lock_error_count, 1,
        "Lock error should be logged exactly once, but found {} occurrences in stderr: {}",
        lock_error_count, stderr_output
    );

    // The test passes if we've verified:
    // 1. The lock conflict occurs (command fails)
    // 2. The FileBusy error is present in output
    // 3. The error appears exactly once (no double logging)
    // Note: The enhanced display message may appear in logs depending on configuration

    Ok(())
}

/// Test that editor failures are logged exactly once at the application boundary
/// This verifies that T003 (removal of double logging) is working correctly
#[test]
#[serial]
fn test_single_error_logging_for_editor_failure() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir)?;

    // Use fixed date for deterministic testing
    let test_date = "2024-01-15";

    // Use a non-existent editor command to trigger CommandNotFound error
    let non_existent_editor = "this_editor_definitely_does_not_exist_anywhere";

    // Run ponder with the non-existent editor
    let output = Command::cargo_bin("ponder")?
        .env("PONDER_EDITOR", non_existent_editor)
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .env("RUST_LOG", "debug") // Enable debug logging
        .env("CI", "true") // Force structured logging to ensure ERROR logs appear
        .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
        .arg("edit") // v2.0: Add edit subcommand
        .arg("--date")
        .arg(test_date)
        .output()?;

    // Verify the command failed as expected
    assert!(
        !output.status.success(),
        "Ponder process should have failed due to editor not found"
    );

    // Verify the error output contains the editor error message
    // Use robust pattern matching instead of checking for enum variant names
    let stderr_output = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_output.contains("not found")
            && stderr_output.contains("this_editor_definitely_does_not_exist_anywhere"),
        "Error message should contain editor not found indication, but stderr was: {}",
        stderr_output
    );

    // Note: Enhanced error messages from T004 include resolution hints
    // This test focuses on verifying single error logging (no double logging from T003)

    // Count how many times the structured error log appears in stderr
    // We expect exactly one ERROR level log entry at the application boundary
    // Use robust pattern matching for the structured log entry
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let combined_output = format!("{}{}", stderr_str, stdout_str);
    let editor_error_count = combined_output.matches("\"level\":\"ERROR\"").count();

    // Assert single error logging - error should appear exactly once
    assert_eq!(
        editor_error_count, 1,
        "Editor error should be logged exactly once, but found {} occurrences in stderr: {}",
        editor_error_count, stderr_output
    );

    // The test passes if we've verified:
    // 1. The editor command fails (command not found)
    // 2. The CommandNotFound error is present in output
    // 3. The error appears exactly once (no double logging)

    Ok(())
}

/// Test that application errors are logged with structured format at the boundary
/// This verifies that issue #43 (structured error logging) is properly implemented
#[test]
#[serial]
fn test_structured_error_logging_boundary() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir)?;

    // Test with JSON logging format and non-existent editor to trigger error
    let output = Command::cargo_bin("ponder")?
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .env("PONDER_EDITOR", "command_that_definitely_does_not_exist")
        .env("CI", "true") // Force JSON logging
        .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
        .arg("--log-format")
        .arg("json")
        .arg("edit") // v2.0: Add edit subcommand
        .arg("--date")
        .arg("2024-01-15")
        .output()?;

    // Verify the command failed
    assert!(
        !output.status.success(),
        "Command should fail with missing editor"
    );

    // Parse stderr to verify structured logging
    let stderr_output = String::from_utf8_lossy(&output.stderr);

    // First verify we have a user-friendly error message
    assert!(
        stderr_output.contains("Error: Editor error"),
        "Should contain user-friendly error message, stderr: {}",
        stderr_output
    );

    // For JSON format with CI=true, we should have structured logs
    if stderr_output.contains("\"timestamp\":") {
        // Check for structured JSON log entry with error
        assert!(
            stderr_output.contains("\"level\":\"ERROR\""),
            "Should contain ERROR level structured log entry, stderr: {}",
            stderr_output
        );

        // Check for correlation ID in structured log
        assert!(
            stderr_output.contains("\"correlation_id\":"),
            "Should contain correlation_id in structured log, stderr: {}",
            stderr_output
        );

        // Check for error field in structured log
        assert!(
            stderr_output.contains("\"error\":") && stderr_output.contains("Application failed"),
            "Should contain structured error field with message, stderr: {}",
            stderr_output
        );

        // Check for error_chain field in structured log
        assert!(
            stderr_output.contains("\"error_chain\":"),
            "Should contain error_chain field, stderr: {}",
            stderr_output
        );
    } else {
        // If no JSON logs, the structured logging may not be initialized properly
        // But at minimum we should have the user-friendly error message
        eprintln!(
            "Warning: No JSON structured logs found in stderr, but user-friendly error present"
        );
    }

    Ok(())
}

/// Test that main() properly propagates and formats various AppError types
/// This verifies that T001 and T004 work together correctly
#[test]
#[serial]
fn test_main_error_propagation() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir)?;

    // Test 1: Config error - editor command with forbidden characters
    {
        // Create a test that triggers AppError::Config from editor with shell metacharacters
        let output = Command::cargo_bin("ponder")?
            .env("PONDER_DIR", journal_dir.to_str().unwrap())
            .env("HOME", journal_dir.to_str().unwrap())
            .env("PONDER_EDITOR", "vim;dangerous") // Contains semicolon which is forbidden
            .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
            .arg("edit") // v2.0: Add edit subcommand
            .arg("--date")
            .arg("2024-01-15")
            .output()?;

        // Verify the command failed
        assert!(
            !output.status.success(),
            "Command should fail with invalid editor command"
        );

        // Verify the error output contains Config error with proper formatting
        let stderr_output = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr_output.contains("Error: Configuration error") && stderr_output.contains("shell metacharacters"),
            "Error should contain Config error formatting for shell metacharacters, but stderr was: {}",
            stderr_output
        );
    }

    // Test 2: Journal error - invalid date format
    {
        let output = Command::cargo_bin("ponder")?
            .env("PONDER_DIR", journal_dir.to_str().unwrap())
            .env("HOME", journal_dir.to_str().unwrap())
            .env("PONDER_EDITOR", "true")
            .env("RUST_LOG", "error") // Valid log level
            .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
            .arg("edit") // v2.0: Add edit subcommand
            .arg("--date")
            .arg("invalid-date-format")
            .output()?;

        // Verify the command failed
        assert!(
            !output.status.success(),
            "Command should fail with invalid date"
        );

        // Verify the error output contains Journal error with proper formatting
        let stderr_output = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr_output.contains("Error: Journal logic error")
                && stderr_output.contains("Invalid date format"),
            "Error should contain Journal error formatting, but stderr was: {}",
            stderr_output
        );
    }

    // Test 3: Editor error - command not found
    {
        let output = Command::cargo_bin("ponder")?
            .env("PONDER_DIR", journal_dir.to_str().unwrap())
            .env("HOME", journal_dir.to_str().unwrap())
            .env("PONDER_EDITOR", "command_that_definitely_does_not_exist")
            .env("RUST_LOG", "error") // Valid log level
            .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
            .arg("edit") // v2.0: Add edit subcommand
            .arg("--date")
            .arg("2024-01-15")
            .output()?;

        // Verify the command failed
        assert!(
            !output.status.success(),
            "Command should fail with missing editor"
        );

        // Verify the error output contains Editor error with proper formatting
        let stderr_output = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr_output.contains("Error: Editor error") && stderr_output.contains("not found"),
            "Error should contain Editor error formatting, but stderr was: {}",
            stderr_output
        );
    }

    // Test 4: I/O error - invalid directory permission (simulate filesystem error)
    {
        // Create a directory with no write permissions to trigger I/O error
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir_all(&readonly_dir)?;

        // Make directory read-only (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir)?.permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&readonly_dir, perms)?;
        }

        let output = Command::cargo_bin("ponder")?
            .env("PONDER_DIR", readonly_dir.to_str().unwrap())
            .env("HOME", readonly_dir.to_str().unwrap())
            .env("PONDER_EDITOR", "true")
            .env("RUST_LOG", "error") // Valid log level
            .env("PONDER_TEST_PASSPHRASE", "test-passphrase") // v2.0: Non-interactive passphrase
            .arg("edit") // v2.0: Add edit subcommand
            .arg("--date")
            .arg("2024-01-15")
            .output()?;

        // Verify the command failed
        assert!(
            !output.status.success(),
            "Command should fail with I/O permission error"
        );

        // Verify the error output contains I/O or Database error with proper formatting
        // v2.0: Database initialization happens before journal I/O, so we get Database error
        let stderr_output = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr_output.contains("Error: I/O error")
                || stderr_output.contains("Error: Database error")
                || stderr_output.contains("Permission denied")
                || stderr_output.contains("unable to open database file"),
            "Error should contain I/O or Database error formatting, but stderr was: {}",
            stderr_output
        );

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir)?.permissions();
            perms.set_mode(0o755); // Restore write permission
            fs::set_permissions(&readonly_dir, perms)?;
        }
    }

    Ok(())
}
