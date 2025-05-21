use serial_test::serial;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

use assert_cmd::Command;
use predicates::prelude::*;

/// Create a mock editor script that simulates a slow editor by adding a delay
#[cfg(unix)]
fn create_slow_editor_script(temp_dir: &Path, sentinel_file: &Path) -> String {
    let script_path = temp_dir.join("slow_editor.sh");
    let content = format!(
        "#!/bin/sh\n\
         touch {}\n\
         sleep 2\n\
         rm {}\n\
         exit 0\n",
        sentinel_file.display(),
        sentinel_file.display()
    );
    fs::write(&script_path, content).unwrap();
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();
    script_path.to_str().unwrap().to_string()
}

#[cfg(windows)]
fn create_slow_editor_script(temp_dir: &Path, sentinel_file: &Path) -> String {
    let script_path = temp_dir.join("slow_editor.bat");
    let content = format!(
        "@echo off\r\n\
         echo.> {}\r\n\
         timeout /t 2 /nobreak >nul\r\n\
         del {}\r\n\
         exit /b 0\r\n",
        sentinel_file.display(),
        sentinel_file.display()
    );
    fs::write(&script_path, content).unwrap();
    script_path.to_str().unwrap().to_string()
}

/// Test that verifies our locking mechanism prevents concurrent access to the same file
#[test]
#[serial]
fn test_file_locking_prevents_concurrent_access() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir).unwrap();

    // Get today's date string for command arguments
    let today_date_str = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Create a sentinel file path to track when the editor is running
    let sentinel_file = temp_dir.path().join("editor_running");

    // Create the mock "slow" editor script
    let slow_editor = create_slow_editor_script(temp_dir.path(), &sentinel_file);

    // Scenario 1: First instance acquires lock
    // Use normal std::process::Command directly
    let mut instance_a = std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--date")
        .arg(&today_date_str)
        .env("PONDER_EDITOR", &slow_editor)
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .spawn()
        .expect("Failed to spawn first ponder instance");

    // Wait for the editor to start and create the sentinel file
    let mut attempts = 0;
    while !sentinel_file.exists() && attempts < 100 {
        thread::sleep(Duration::from_millis(100));
        attempts += 1;
    }

    if !sentinel_file.exists() {
        // If the sentinel file doesn't exist, kill the process to avoid hanging
        instance_a.kill().ok();
        instance_a.wait().ok();
        panic!("First instance failed to create sentinel file");
    }

    // Scenario 2: While first instance has lock, second instance should fail
    // Use assert_cmd for validation of output
    let assert_cmd = Command::cargo_bin("ponder")
        .unwrap()
        .env("PONDER_EDITOR", "true")
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .arg("--date")
        .arg(&today_date_str)
        .assert();

    // Second instance should fail with a lock error
    assert_cmd
        .failure()
        .stderr(predicate::str::contains("File locking error"))
        .stderr(predicate::str::contains(
            "currently being edited by another process",
        ));

    // Wait for first instance to finish
    instance_a
        .wait()
        .expect("First instance failed to complete");

    // Sentinel file should be gone now
    assert!(
        !sentinel_file.exists(),
        "Editor did not clean up sentinel file"
    );

    // Scenario 3: After first instance releases lock, third instance should succeed
    Command::cargo_bin("ponder")
        .unwrap()
        .env("PONDER_EDITOR", "true")
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .arg("--date")
        .arg(&today_date_str)
        .assert()
        .success();
}

/// Test to verify that attempting to access multiple files works correctly
/// when some files are locked and others are not
#[test]
#[serial]
fn test_partial_file_locking_with_multiple_files() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir).unwrap();

    // Create a sentinel file path
    let sentinel_file = temp_dir.path().join("editor_running");

    // Create the mock "slow" editor script
    let slow_editor = create_slow_editor_script(temp_dir.path(), &sentinel_file);

    // Get dates for testing
    let now = chrono::Local::now();
    let today_date_str = now.format("%Y-%m-%d").to_string();
    let yesterday_date_str = (now - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    // Scenario 1: Lock today's file specifically
    let mut instance_a = std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--date")
        .arg(&today_date_str)
        .env("PONDER_EDITOR", &slow_editor)
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .spawn()
        .expect("Failed to spawn first ponder instance");

    // Wait for the editor to start
    let mut attempts = 0;
    while !sentinel_file.exists() && attempts < 100 {
        thread::sleep(Duration::from_millis(100));
        attempts += 1;
    }

    if !sentinel_file.exists() {
        // If the sentinel file doesn't exist, kill the process to avoid hanging
        instance_a.kill().ok();
        instance_a.wait().ok();
        panic!("First instance failed to create sentinel file");
    }

    // Scenario 2: Try to access yesterday's file while today's is locked
    // This should succeed because it's a different file
    Command::cargo_bin("ponder")
        .unwrap()
        .env("PONDER_EDITOR", "true")
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .arg("--date")
        .arg(&yesterday_date_str)
        .assert()
        .success();

    // Ensure first instance finishes
    instance_a
        .wait()
        .expect("First instance failed to complete");
}
