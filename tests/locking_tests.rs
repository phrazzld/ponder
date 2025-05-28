use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

use assert_cmd::Command;
use predicates::prelude::*;

// Fixed test date for deterministic testing
const FIXED_TEST_DATE: &str = "2024-01-15";

/// Execute a test closure with retry logic for transient failures
fn retry_test<F>(test_name: &str, mut test_fn: F) -> Result<(), String>
where
    F: FnMut() -> Result<(), String>,
{
    const MAX_ATTEMPTS: u32 = 3;
    const RETRY_DELAY: Duration = Duration::from_secs(1);

    for attempt in 1..=MAX_ATTEMPTS {
        eprintln!(
            "[RETRY] Executing test '{}' - Attempt {}/{}",
            test_name, attempt, MAX_ATTEMPTS
        );

        match test_fn() {
            Ok(()) => {
                eprintln!("[RETRY] Test '{}' passed on attempt {}", test_name, attempt);
                return Ok(());
            }
            Err(e) => {
                eprintln!(
                    "[RETRY] Test '{}' failed on attempt {}: {}",
                    test_name, attempt, e
                );
                if attempt < MAX_ATTEMPTS {
                    eprintln!(
                        "[RETRY] Waiting {} seconds before retry...",
                        RETRY_DELAY.as_secs()
                    );
                    thread::sleep(RETRY_DELAY);
                }
            }
        }
    }

    Err(format!(
        "Test '{}' failed after {} attempts",
        test_name, MAX_ATTEMPTS
    ))
}

/// A guard that ensures child processes are properly cleaned up
struct ChildProcessGuard {
    child: std::process::Child,
    name: String,
}

/// A guard that ensures sentinel files are cleaned up if left behind
struct SentinelFileGuard {
    path: PathBuf,
}

impl SentinelFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for SentinelFileGuard {
    fn drop(&mut self) {
        // Clean up sentinel file if it still exists (safety net)
        if self.path.exists() {
            eprintln!(
                "[CLEANUP] Removing leftover sentinel file: {}",
                self.path.display()
            );
            if let Err(e) = std::fs::remove_file(&self.path) {
                eprintln!("[CLEANUP] Failed to remove sentinel file: {}", e);
            } else {
                eprintln!("[CLEANUP] Successfully removed sentinel file");
            }
        } else {
            eprintln!(
                "[CLEANUP] Sentinel file already cleaned up: {}",
                self.path.display()
            );
        }
    }
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
        eprintln!("[CLEANUP] Starting cleanup for process '{}'", self.name);

        // Check if the process has already exited
        match self.child.try_wait() {
            Ok(Some(status)) => {
                // Process already exited
                eprintln!(
                    "[CLEANUP] Process '{}' already exited with status: {}",
                    self.name, status
                );
                return;
            }
            Ok(None) => {
                // Process still running, try to kill it
                eprintln!(
                    "[CLEANUP] Process '{}' still running, attempting to kill",
                    self.name
                );
                if let Err(e) = self.child.kill() {
                    eprintln!("[CLEANUP] Failed to kill process '{}': {}", self.name, e);
                } else {
                    eprintln!(
                        "[CLEANUP] Successfully sent kill signal to process '{}'",
                        self.name
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "[CLEANUP] Error checking process '{}' status: {}. Attempting to kill anyway",
                    self.name, e
                );
                if let Err(ke) = self.child.kill() {
                    eprintln!(
                        "[CLEANUP] Failed to kill process '{}' after error: {}",
                        self.name, ke
                    );
                } else {
                    eprintln!(
                        "[CLEANUP] Successfully sent kill signal to process '{}'",
                        self.name
                    );
                }
            }
        }

        // Always wait for the process to ensure it's reaped and doesn't become a zombie
        eprintln!("[CLEANUP] Waiting for process '{}' to be reaped", self.name);
        match self.child.wait() {
            Ok(status) => {
                eprintln!(
                    "[CLEANUP] Process '{}' successfully terminated and reaped with status: {}",
                    self.name, status
                );
            }
            Err(e) => {
                eprintln!(
                    "[CLEANUP] Failed to wait for process '{}' (this may indicate it was already reaped): {}",
                    self.name, e
                );
            }
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
/// Script handles its own file cleanup, parent process provides additional RAII safety net
#[cfg(unix)]
fn create_slow_editor_script(
    temp_dir: &Path,
    hold_duration_secs: u64,
    sentinel_file: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let script_path = temp_dir.join("slow_editor.sh");
    let content = format!(
        "#!/bin/sh\n\
         SENTINEL_FILE_PATH=\"{}\"\n\
         HOLD_DURATION_SECS=\"{}\"\n\
         \n\
         # Ensure sentinel is cleaned up on exit/interrupt (script-level cleanup)\n\
         trap 'rm -f \"$SENTINEL_FILE_PATH\"' EXIT HUP INT QUIT TERM\n\
         \n\
         # Create sentinel file to signal editor has started\n\
         touch \"$SENTINEL_FILE_PATH\"\n\
         echo \"[MOCK EDITOR] Started, holding for $HOLD_DURATION_SECS seconds. Sentinel: $SENTINEL_FILE_PATH\" >&2\n\
         \n\
         sleep \"$HOLD_DURATION_SECS\"\n\
         \n\
         echo \"[MOCK EDITOR] Finishing normally\" >&2\n\
         # Trap will handle rm, but explicit rm is good practice for normal exit\n\
         rm -f \"$SENTINEL_FILE_PATH\"\n\
         exit 0\n",
        sentinel_file.display(),
        hold_duration_secs
    );
    fs::write(&script_path, content)?;
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms)?;
    let script_str = script_path
        .to_str()
        .ok_or("Failed to convert script path to string")?;
    Ok(script_str.to_string())
}

#[cfg(windows)]
fn create_slow_editor_script(
    temp_dir: &Path,
    hold_duration_secs: u64,
    sentinel_file: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let script_path = temp_dir.join("slow_editor.bat");
    let content = format!(
        "@echo off\r\n\
         SET SENTINEL_FILE_PATH={}\r\n\
         SET HOLD_DURATION_SECS={}\r\n\
         \r\n\
         REM Create sentinel file to signal editor has started\r\n\
         echo. > \"%SENTINEL_FILE_PATH%\"\r\n\
         echo [MOCK EDITOR] Started, holding for %HOLD_DURATION_SECS% seconds. Sentinel: %SENTINEL_FILE_PATH% >&2\r\n\
         \r\n\
         REM timeout command is used for delay\r\n\
         timeout /t %HOLD_DURATION_SECS% /nobreak > nul\r\n\
         \r\n\
         echo [MOCK EDITOR] Finishing normally >&2\r\n\
         REM Delete sentinel file (script-level cleanup)\r\n\
         del \"%SENTINEL_FILE_PATH%\" > nul 2>&1\r\n\
         exit /b 0\r\n",
        sentinel_file.display(),
        hold_duration_secs
    );
    fs::write(&script_path, content)?;
    let script_str = script_path
        .to_str()
        .ok_or("Failed to convert script path to string")?;
    Ok(script_str.to_string())
}

/// Test that verifies our locking mechanism prevents concurrent access to the same file
#[test]
#[serial]
fn test_file_locking_prevents_concurrent_access() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[TEST] Starting file locking test with RAII cleanup");

    // Create a temporary directory for testing - this will be automatically cleaned up
    let temp_dir = tempdir()?;
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir)?;
    eprintln!(
        "[TEST] Created temporary directories: {}",
        temp_dir.path().display()
    );

    // Use fixed date string for deterministic testing
    let today_date_str = FIXED_TEST_DATE.to_string();

    // Create a sentinel file path and guard for RAII cleanup
    let sentinel_path = temp_dir.path().join("editor_running");
    let _sentinel_guard = SentinelFileGuard::new(sentinel_path.clone());
    eprintln!(
        "[TEST] Created sentinel file path with RAII guard: {}",
        sentinel_path.display()
    );

    // Define editor hold duration
    let editor_hold_duration = Duration::from_secs(3);
    let editor_hold_duration_secs = editor_hold_duration.as_secs();

    // Create the mock "slow" editor script
    let slow_editor_script =
        create_slow_editor_script(temp_dir.path(), editor_hold_duration_secs, &sentinel_path)?;
    eprintln!("[TEST] Created mock editor script: {}", slow_editor_script);

    // Execute the core test logic with retry mechanism
    let test_result = retry_test("file_locking_prevents_concurrent_access", || {
        // Scenario 1: First instance acquires lock
        let instance_a_child = std::process::Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--date")
            .arg(&today_date_str)
            .env("PONDER_EDITOR", &slow_editor_script)
            .env(
                "PONDER_DIR",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .env(
                "HOME",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .spawn()
            .map_err(|e| format!("Failed to spawn first ponder instance: {}", e))?;

        let _instance_a_guard =
            ChildProcessGuard::new(instance_a_child, "PonderInstanceA".to_string());
        eprintln!("[TEST] Spawned first ponder instance with ChildProcessGuard");

        // Wait for the sentinel file to be created by the mock editor
        eprintln!("[TEST] Waiting for sentinel file to be created...");
        wait_for_file_state(
            &sentinel_path,
            true,
            Duration::from_secs(5),
            Duration::from_millis(100),
        )?;
        eprintln!("[TEST] Sentinel file created, first instance is running");

        // Scenario 2: While first instance has lock, second instance should fail
        eprintln!("[TEST] Second instance should fail due to lock");
        let second_result = Command::cargo_bin("ponder")
            .map_err(|e| format!("Failed to create command: {}", e))?
            .env("PONDER_EDITOR", "true")
            .env(
                "PONDER_DIR",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .env(
                "HOME",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .arg("--date")
            .arg(&today_date_str)
            .assert()
            .try_failure();

        // Verify it failed with the expected lock error
        match second_result {
            Ok(_) => {
                // Now verify the error message
                Command::cargo_bin("ponder")
                    .map_err(|e| format!("Failed to create command: {}", e))?
                    .env("PONDER_EDITOR", "true")
                    .env(
                        "PONDER_DIR",
                        journal_dir
                            .to_str()
                            .ok_or("Failed to convert journal_dir to string")?,
                    )
                    .env(
                        "HOME",
                        journal_dir
                            .to_str()
                            .ok_or("Failed to convert journal_dir to string")?,
                    )
                    .arg("--date")
                    .arg(&today_date_str)
                    .assert()
                    .failure()
                    .stderr(predicate::str::contains("Error: Lock(FileBusy"));
                eprintln!("[TEST] Second instance correctly failed with lock error");
            }
            Err(_) => {
                return Err("Second instance did not fail as expected".to_string());
            }
        }

        // Wait for the sentinel file to be removed by the mock editor
        eprintln!("[TEST] Waiting for first instance to complete...");
        wait_for_file_state(
            &sentinel_path,
            false,
            editor_hold_duration + Duration::from_secs(5),
            Duration::from_millis(100),
        )?;
        eprintln!("[TEST] First instance completed, lock should be released");

        // Scenario 3: After first instance releases lock, third instance should succeed
        eprintln!("[TEST] Third instance should succeed now that lock is released");
        let third_result = Command::cargo_bin("ponder")
            .map_err(|e| format!("Failed to create command: {}", e))?
            .env("PONDER_EDITOR", "true")
            .env(
                "PONDER_DIR",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .env(
                "HOME",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .arg("--date")
            .arg(&today_date_str)
            .assert();

        if third_result.try_success().is_err() {
            return Err("Third instance did not succeed as expected".to_string());
        }
        eprintln!("[TEST] Third instance succeeded as expected");

        Ok(())
    });

    // Propagate any test failures
    test_result.map_err(|e| format!("Test failed after maximum retry attempts: {}", e))?;

    eprintln!("[TEST] Test completed successfully");
    // Note: RAII cleanup will handle:
    // - ChildProcessGuard will clean up any remaining child processes
    // - SentinelFileGuard will clean up the sentinel file if left behind
    // - TempDir will clean up the entire temporary directory tree
    Ok(())
}

/// Test to verify that attempting to access multiple files works correctly
/// when some files are locked and others are not
#[test]
#[serial]
fn test_partial_file_locking_with_multiple_files() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[TEST] Starting partial file locking test with RAII cleanup");

    // Create a temporary directory for testing - this will be automatically cleaned up
    let temp_dir = tempdir()?;
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir)?;
    eprintln!(
        "[TEST] Created temporary directories: {}",
        temp_dir.path().display()
    );

    // Create a sentinel file path and guard for RAII cleanup
    let sentinel_path = temp_dir.path().join("editor_running_partial");
    let _sentinel_guard = SentinelFileGuard::new(sentinel_path.clone());
    eprintln!(
        "[TEST] Created sentinel file path with RAII guard: {}",
        sentinel_path.display()
    );

    // Define editor hold duration
    let editor_hold_duration_secs = 2;

    // Create the mock "slow" editor script
    let slow_editor_script =
        create_slow_editor_script(temp_dir.path(), editor_hold_duration_secs, &sentinel_path)?;
    eprintln!("[TEST] Created mock editor script: {}", slow_editor_script);

    // Use fixed dates for deterministic testing
    let today_date_str = FIXED_TEST_DATE.to_string();
    let yesterday_date_str = "2024-01-14".to_string();

    // Execute the core test logic with retry mechanism
    let test_result = retry_test("partial_file_locking_with_multiple_files", || {
        // Scenario 1: Lock today's file specifically
        let instance_a_child = std::process::Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--date")
            .arg(&today_date_str)
            .env("PONDER_EDITOR", &slow_editor_script)
            .env(
                "PONDER_DIR",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .env(
                "HOME",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .spawn()
            .map_err(|e| format!("Failed to spawn first ponder instance: {}", e))?;

        let _instance_a_guard =
            ChildProcessGuard::new(instance_a_child, "PonderInstanceAPartialTest".to_string());
        eprintln!("[TEST] Spawned first ponder instance with ChildProcessGuard");

        // Wait for the sentinel file to be created by the mock editor
        eprintln!("[TEST] Waiting for sentinel file to be created...");
        wait_for_file_state(
            &sentinel_path,
            true,
            Duration::from_secs(5),
            Duration::from_millis(100),
        )?;
        eprintln!("[TEST] Sentinel file created, first instance is running");

        // Scenario 2: Try to access yesterday's file while today's is locked
        // This should succeed because it's a different file
        eprintln!("[TEST] Attempting to access yesterday's file while today's is locked");
        let yesterday_result = Command::cargo_bin("ponder")
            .map_err(|e| format!("Failed to create command: {}", e))?
            .env("PONDER_EDITOR", "true")
            .env(
                "PONDER_DIR",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .env(
                "HOME",
                journal_dir
                    .to_str()
                    .ok_or("Failed to convert journal_dir to string")?,
            )
            .arg("--date")
            .arg(&yesterday_date_str)
            .assert();

        if yesterday_result.try_success().is_err() {
            return Err("Failed to access yesterday's file while today's is locked".to_string());
        }
        eprintln!("[TEST] Successfully accessed yesterday's file while today's is locked");

        // Wait for the sentinel file to be removed by the mock editor
        eprintln!("[TEST] Waiting for first instance to complete...");
        wait_for_file_state(
            &sentinel_path,
            false,
            Duration::from_secs(editor_hold_duration_secs + 5),
            Duration::from_millis(100),
        )?;
        eprintln!("[TEST] First instance completed");

        Ok(())
    });

    // Propagate any test failures
    test_result.map_err(|e| format!("Test failed after maximum retry attempts: {}", e))?;

    eprintln!("[TEST] Partial file locking test completed successfully");
    // Note: RAII cleanup will handle:
    // - ChildProcessGuard will clean up any remaining child processes
    // - SentinelFileGuard will clean up the sentinel file if left behind
    // - TempDir will clean up the entire temporary directory tree
    Ok(())
}
