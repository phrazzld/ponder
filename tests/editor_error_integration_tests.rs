// Integration tests for editor error handling
use serial_test::serial;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::tempdir;

// This function runs the ponder binary with a specific editor
// and returns a Result with whether the process succeeded and the stderr if it failed
fn run_with_editor(editor_command: &str) -> Result<(bool, String), Box<dyn std::error::Error>> {
    // Create a temporary environment for the test
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();

    // Set up the PONDER_EDITOR environment variable
    env::set_var("PONDER_EDITOR", editor_command);
    env::set_var("PONDER_DIR", temp_path);
    // v2.0: Set test passphrase for non-interactive testing
    env::set_var("PONDER_TEST_PASSPHRASE", "test-passphrase");

    // v2.0: Run with "edit" subcommand
    let output = std::process::Command::new("cargo")
        .args(["run", "--quiet", "--", "edit"])
        .output()?;

    // Clean up the environment
    env::remove_var("PONDER_EDITOR");
    env::remove_var("PONDER_DIR");
    env::remove_var("PONDER_TEST_PASSPHRASE");

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status.success(), stderr))
}

// Create a mock editor script for testing
fn create_mock_editor(
    path: &Path,
    content: &str,
    executable: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    // Make the file executable (or not) based on the parameter
    if executable {
        let metadata = file.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

// This test needs to be run serially to prevent environment variable conflicts
#[test]
#[serial]
fn test_editor_command_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let (success, stderr) = run_with_editor("nonexistent_editor_command")?;
    assert!(!success, "Command with nonexistent editor should fail");

    // Use robust pattern matching instead of checking for enum variant names
    // This safeguards against test brittleness when error message formats change
    assert!(
        stderr.contains("not found") && stderr.contains("nonexistent_editor_command"),
        "Error message should contain 'not found' and the command name. Stderr: {}",
        stderr
    );
    Ok(())
}

#[test]
#[serial]
fn test_editor_permission_denied() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our test script
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("non_executable_editor.sh");

    // Create a script file without execute permissions
    create_mock_editor(&script_path, "#!/bin/sh\necho 'This should not run'", false)?;

    // Set permissions explicitly to ensure it's not executable
    let metadata = fs::metadata(&script_path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o644); // rw-r--r--
    fs::set_permissions(&script_path, permissions)?;

    // Run the test with our non-executable script
    let script_path_str = script_path
        .to_str()
        .ok_or("Failed to convert script path to string")?;
    let (success, stderr) = run_with_editor(script_path_str)?;

    assert!(!success, "Command with non-executable editor should fail");
    // The exact error might be platform-dependent, but should mention permission
    // On some platforms it could be "permission denied"
    assert!(
        stderr.contains("PermissionDenied") || stderr.contains("permission"),
        "Error message should contain 'PermissionDenied' or 'permission'. Stderr: {}",
        stderr
    );
    Ok(())
}

#[test]
#[serial]
fn test_editor_non_zero_exit() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our test script
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("failing_editor.sh");

    // Create a script that exits with code 1
    create_mock_editor(&script_path, "#!/bin/sh\nexit 1", true)?;

    // Run the test with our failing script
    let script_path_str = script_path
        .to_str()
        .ok_or("Failed to convert script path to string")?;
    let (success, stderr) = run_with_editor(script_path_str)?;

    assert!(!success, "Command with failing editor should fail");
    // Use robust pattern matching for non-zero exit status
    // This safeguards against test brittleness when error message formats change
    assert!(
        stderr.contains("non-zero status code") || stderr.contains("exited with"),
        "Error message should indicate non-zero exit status, got: {}",
        stderr
    );
    assert!(
        stderr.contains("1"),
        "Error message should include the exit code 1, got: {}",
        stderr
    );
    Ok(())
}

#[test]
#[serial]
fn test_successful_editor() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our test script
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("success_editor.sh");

    // Create a script that exits successfully
    create_mock_editor(&script_path, "#!/bin/sh\nexit 0", true)?;

    // Run the test with our successful script
    let script_path_str = script_path
        .to_str()
        .ok_or("Failed to convert script path to string")?;
    let (success, stderr) = run_with_editor(script_path_str)?;

    assert!(
        success,
        "Command with successful editor should succeed: {}",
        stderr
    );
    assert!(
        !stderr.contains("Error") && !stderr.contains("error:"),
        "There should be no error output for successful editor, got: {}",
        stderr
    );
    Ok(())
}
