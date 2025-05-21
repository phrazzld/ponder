use serial_test::serial;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

use assert_cmd::Command;
use predicates::prelude::*;

/// A guard that ensures child processes are properly cleaned up
struct ChildProcessGuard {
    child: std::process::Child,
    name: String,
}

impl ChildProcessGuard {
    fn new(child: std::process::Child, name: String) -> Self {
        Self { child, name }
    }

    #[allow(dead_code)]
    fn wait_for_completion(mut self) -> std::io::Result<std::process::ExitStatus> {
        let status = self.child.wait()?;
        // Prevent Drop from trying to kill an already-waited-on process
        std::mem::forget(self);
        Ok(status)
    }
}

impl Drop for ChildProcessGuard {
    fn drop(&mut self) {
        // Check if the process has already exited
        match self.child.try_wait() {
            Ok(Some(status)) => {
                // Process already exited
                eprintln!("Process '{}' exited with status: {}", self.name, status);
                return;
            }
            Ok(None) => {
                // Process still running, try to kill it
                eprintln!("Process '{}' still running, attempting to kill.", self.name);
                if let Err(e) = self.child.kill() {
                    eprintln!("Failed to kill process '{}': {}", self.name, e);
                }
            }
            Err(e) => {
                eprintln!(
                    "Error checking process '{}' status: {}. Assuming it needs to be killed.",
                    self.name, e
                );
                if let Err(ke) = self.child.kill() {
                    eprintln!("Failed to kill process '{}' after error: {}", self.name, ke);
                }
            }
        }
        // Wait for the process to ensure it's reaped
        if let Err(e) = self.child.wait() {
            eprintln!(
                "Failed to wait for process '{}' after attempting kill: {}",
                self.name, e
            );
        } else {
            eprintln!(
                "Process '{}' successfully terminated and reaped.",
                self.name
            );
        }
    }
}

/// Wait for a file to reach a specific state (exist/not exist) with timeout
fn wait_for_file_state(
    file_path: &Path,
    should_exist: bool,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<(), String> {
    let start_time = Instant::now();
    while start_time.elapsed() < timeout {
        if file_path.exists() == should_exist {
            return Ok(());
        }
        thread::sleep(poll_interval);
    }
    Err(format!(
        "Timeout waiting for file '{}' to {} after {}ms. Current state: {}",
        file_path.display(),
        if should_exist { "exist" } else { "be removed" },
        timeout.as_millis(),
        if file_path.exists() {
            "exists"
        } else {
            "does not exist"
        }
    ))
}

/// Create a mock editor script that simulates a slow editor with configurable duration
#[cfg(unix)]
fn create_slow_editor_script(
    temp_dir: &Path,
    hold_duration_secs: u64,
    sentinel_file: &Path,
) -> String {
    let script_path = temp_dir.join("slow_editor.sh");
    let content = format!(
        "#!/bin/sh\n\
         SENTINEL_FILE_PATH=\"{}\"\n\
         HOLD_DURATION_SECS=\"{}\"\n\
         \n\
         # Ensure sentinel is cleaned up on exit/interrupt\n\
         trap 'rm -f \"$SENTINEL_FILE_PATH\"' EXIT HUP INT QUIT TERM\n\
         \n\
         # Create sentinel file to signal editor has started\n\
         touch \"$SENTINEL_FILE_PATH\"\n\
         echo \"Mock editor started, holding for $HOLD_DURATION_SECS seconds. Sentinel: $SENTINEL_FILE_PATH\" >&2\n\
         \n\
         sleep \"$HOLD_DURATION_SECS\"\n\
         \n\
         echo \"Mock editor finishing.\" >&2\n\
         # Trap will handle rm, but explicit rm is good practice for normal exit\n\
         rm -f \"$SENTINEL_FILE_PATH\"\n\
         exit 0\n",
        sentinel_file.display(),
        hold_duration_secs
    );
    fs::write(&script_path, content).unwrap();
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();
    script_path.to_str().unwrap().to_string()
}

#[cfg(windows)]
fn create_slow_editor_script(
    temp_dir: &Path,
    hold_duration_secs: u64,
    sentinel_file: &Path,
) -> String {
    let script_path = temp_dir.join("slow_editor.bat");
    let content = format!(
        "@echo off\r\n\
         SET SENTINEL_FILE_PATH={}\r\n\
         SET HOLD_DURATION_SECS={}\r\n\
         \r\n\
         REM Create sentinel file\r\n\
         echo. > \"%SENTINEL_FILE_PATH%\"\r\n\
         echo Mock editor started, holding for %HOLD_DURATION_SECS% seconds. Sentinel: %SENTINEL_FILE_PATH% >&2\r\n\
         \r\n\
         REM timeout command is used for delay\r\n\
         timeout /t %HOLD_DURATION_SECS% /nobreak > nul\r\n\
         \r\n\
         echo Mock editor finishing. >&2\r\n\
         REM Delete sentinel file\r\n\
         del \"%SENTINEL_FILE_PATH%\" > nul 2>&1\r\n\
         exit /b 0\r\n",
        sentinel_file.display(),
        hold_duration_secs
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

    // Define editor hold duration
    let editor_hold_duration = Duration::from_secs(3);
    let editor_hold_duration_secs = editor_hold_duration.as_secs();

    // Create the mock "slow" editor script
    let slow_editor_script =
        create_slow_editor_script(temp_dir.path(), editor_hold_duration_secs, &sentinel_file);

    // Scenario 1: First instance acquires lock
    let instance_a_child = std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--date")
        .arg(&today_date_str)
        .env("PONDER_EDITOR", &slow_editor_script)
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .spawn()
        .expect("Failed to spawn first ponder instance");

    let _instance_a_guard = ChildProcessGuard::new(instance_a_child, "PonderInstanceA".to_string());

    // Wait for the sentinel file to be created by the mock editor
    wait_for_file_state(
        &sentinel_file,
        true,
        Duration::from_secs(5),
        Duration::from_millis(100),
    )
    .expect("Sentinel file for instance A's editor was not created in time");

    // Scenario 2: While first instance has lock, second instance should fail
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
        .stderr(predicate::str::contains("Error: Lock(FileBusy"));

    // Wait for the sentinel file to be removed by the mock editor
    wait_for_file_state(
        &sentinel_file,
        false,
        editor_hold_duration + Duration::from_secs(5),
        Duration::from_millis(100),
    )
    .expect("Sentinel file for instance A's editor was not removed in time");

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

    // The ChildProcessGuard will ensure instance_a is cleaned up when it goes out of scope
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

    // Define editor hold duration
    let editor_hold_duration_secs = 2;

    // Create the mock "slow" editor script
    let slow_editor_script =
        create_slow_editor_script(temp_dir.path(), editor_hold_duration_secs, &sentinel_file);

    // Get dates for testing
    let now = chrono::Local::now();
    let today_date_str = now.format("%Y-%m-%d").to_string();
    let yesterday_date_str = (now - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    // Scenario 1: Lock today's file specifically
    let instance_a_child = std::process::Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--date")
        .arg(&today_date_str)
        .env("PONDER_EDITOR", &slow_editor_script)
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", journal_dir.to_str().unwrap())
        .spawn()
        .expect("Failed to spawn first ponder instance");

    let _instance_a_guard =
        ChildProcessGuard::new(instance_a_child, "PonderInstanceAPartialTest".to_string());

    // Wait for the sentinel file to be created by the mock editor
    wait_for_file_state(
        &sentinel_file,
        true,
        Duration::from_secs(5),
        Duration::from_millis(100),
    )
    .expect("Sentinel file for instance A's editor was not created in time");

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

    // Wait for the sentinel file to be removed by the mock editor
    wait_for_file_state(
        &sentinel_file,
        false,
        Duration::from_secs(editor_hold_duration_secs + 5),
        Duration::from_millis(100),
    )
    .expect("Sentinel file for instance A's editor was not removed in time");

    // The ChildProcessGuard will ensure instance_a is cleaned up when it goes out of scope
}
