use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Helper function to set up a test Command instance with clean environment
fn set_up_command() -> Command {
    let mut cmd = Command::cargo_bin("ponder").unwrap();
    // Clear environment for test isolation
    cmd.env_clear();
    cmd
}

#[test]
#[serial]
fn test_reject_editor_with_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = set_up_command();

    cmd.env("PONDER_EDITOR", "vim --noplugin")
        .env("PONDER_DIR", temp_dir.path())
        .env("HOME", temp_dir.path()); // Set HOME to avoid any fallback issues

    // Test behavior: Editor commands with spaces should be rejected for security
    // This validates that the application prevents space-based command injection attacks
    // Use robust pattern matching: focus on essential security validation behavior
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Configuration error").and(
            predicate::str::contains("spaces").or(predicate::str::contains("metacharacters")),
        ));
}

#[test]
#[serial]
fn test_reject_editor_with_shell_metacharacters() {
    let temp_dir = TempDir::new().unwrap();
    // Use a specific path in our temp directory to check for file creation
    let pwned_file = temp_dir.path().join("pwned");

    let mut cmd = set_up_command();
    cmd.env(
        "PONDER_EDITOR",
        format!("echo hello > {}", pwned_file.display()),
    )
    .env("PONDER_DIR", temp_dir.path())
    .env("HOME", temp_dir.path());

    // Test behavior: Commands with shell metacharacters must be rejected for security
    // This is a critical security test preventing command injection attacks
    // Use robust pattern matching: focus on essential security validation behavior
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Configuration error").and(
            predicate::str::contains("metacharacters").or(predicate::str::contains("spaces")),
        )); // Might fail on space first

    // Verify the malicious file was never created - this confirms the security protection worked
    assert!(
        !pwned_file.exists(),
        "Malicious file should not have been created - security validation should prevent execution"
    );
}

#[test]
#[serial]
fn test_reject_editor_with_shell_invocation() {
    let temp_dir = TempDir::new().unwrap();
    let pwned_shell_file = temp_dir.path().join("pwned_shell");

    let mut cmd = set_up_command();
    cmd.env(
        "PONDER_EDITOR",
        format!("sh -c 'touch {}'", pwned_shell_file.display()),
    )
    .env("PONDER_DIR", temp_dir.path())
    .env("HOME", temp_dir.path());

    // Test behavior: Shell invocation commands must be rejected for security
    // This validates protection against sophisticated command injection attempts
    // Use robust pattern matching: focus on essential security validation behavior
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Configuration error").and(
            predicate::str::contains("metacharacters").or(predicate::str::contains("spaces")),
        ));

    // Verify the malicious file was never created - confirms complete prevention of shell execution
    assert!(
        !pwned_shell_file.exists(),
        "Malicious shell file should not have been created - security validation should prevent all shell execution"
    );
}

#[test]
#[serial]
fn test_launch_valid_editor_successfully() {
    let temp_dir = TempDir::new().unwrap();
    let journal_dir = temp_dir.path().join("journal");
    std::fs::create_dir_all(&journal_dir).unwrap();

    let sentinel_file = temp_dir.path().join("editor_was_launched.sentinel");
    let editor_script = temp_dir.path().join("test_editor.sh");

    // Create a simple script that creates the sentinel file
    let script_content = format!("#!/bin/sh\ntouch {}\nexit 0\n", sentinel_file.display());

    let mut script_file = File::create(&editor_script).unwrap();
    write!(script_file, "{}", script_content).unwrap();
    drop(script_file); // Ensure file is closed before setting permissions

    // Make the script executable on Unix
    #[cfg(unix)]
    {
        let mut perms = std::fs::metadata(&editor_script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&editor_script, perms).unwrap();
    }

    // On Windows, create a batch file instead
    #[cfg(windows)]
    {
        let editor_script = temp_dir.path().join("test_editor.bat");
        let script_content = format!("type nul > {}\nexit 0\n", sentinel_file.display());
        let mut script_file = File::create(&editor_script).unwrap();
        write!(script_file, "{}", script_content).unwrap();
    }

    assert!(
        !sentinel_file.exists(),
        "Sentinel file should not exist before test"
    );

    let mut cmd = set_up_command();
    cmd.env("PONDER_EDITOR", editor_script.to_str().unwrap())
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", temp_dir.path().to_str().unwrap());

    // Test behavior: Valid editor commands should execute successfully
    // This validates that legitimate editor commands pass security validation and work correctly
    cmd.assert().success();

    // Verify the editor was actually launched - confirms that valid commands are not blocked
    assert!(
        sentinel_file.exists(),
        "Editor script should have created sentinel file - validates that security checks don't block legitimate editors"
    );
}

#[test]
#[serial]
fn test_reject_empty_editor_string() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = set_up_command();

    cmd.env("PONDER_EDITOR", "")
        .env("PONDER_DIR", temp_dir.path())
        .env("HOME", temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be empty"));
}

// Additional test to verify we accept simple commands without arguments
#[test]
#[serial]
fn test_accept_simple_editor_command() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = set_up_command();

    // Using "true" as a safe editor command that always succeeds
    cmd.env("PONDER_EDITOR", "true")
        .env("PONDER_DIR", temp_dir.path())
        .env("HOME", temp_dir.path());

    cmd.assert().success();
}

// Test with absolute path to ensure that's accepted
#[test]
#[serial]
fn test_accept_absolute_path_editor() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = set_up_command();

    // Using "/usr/bin/true" as a safe editor command (Unix)
    // On macOS, true is typically at /usr/bin/true
    // On Linux, it's also typically at /usr/bin/true
    #[cfg(unix)]
    let editor_path = "/usr/bin/true";

    #[cfg(windows)]
    let editor_path = "C:\\Windows\\System32\\cmd.exe"; // A valid Windows path

    cmd.env("PONDER_EDITOR", editor_path)
        .env("PONDER_DIR", temp_dir.path())
        .env("HOME", temp_dir.path());

    // This might fail if the path doesn't exist on the system
    // In that case, we'll just check that it doesn't fail with validation error
    let result = cmd.assert();

    // If it fails, it should not be due to validation
    if !result.get_output().status.success() {
        // Make sure it's not failing due to our validation
        let stderr = std::str::from_utf8(&result.get_output().stderr).unwrap();
        assert!(!stderr.contains("cannot contain spaces"));
        assert!(!stderr.contains("cannot contain shell metacharacters"));
        assert!(!stderr.contains("cannot be empty"));
    }
}
