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
#[test]
#[serial]
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
        .arg("--date")
        .arg(test_date)
        .env("PONDER_EDITOR", &slow_editor)
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
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
        .arg("--date")
        .arg(test_date)
        .output()?;

    // Verify the command failed as expected
    assert!(
        !output.status.success(),
        "Second ponder process should have failed due to lock conflict"
    );

    // Verify the error output contains the lock error message
    let stderr_output = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_output.contains("Lock(FileBusy"),
        "Error message should contain FileBusy lock error, but stderr was: {}",
        stderr_output
    );

    // Note: Enhanced error messages from T004 may appear in different contexts
    // This test focuses on verifying single error logging (no double logging from T002)

    // Count how many times the lock error appears in stderr
    // We expect it to appear exactly once (single error logging)
    let lock_error_count = stderr_output.matches("Lock(FileBusy").count();

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
