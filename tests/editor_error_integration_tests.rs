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
fn run_with_editor(editor_command: &str) -> (bool, String) {
    // Create a temporary environment for the test
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path();

    // Set up the PONDER_EDITOR environment variable
    env::set_var("PONDER_EDITOR", editor_command);
    env::set_var("PONDER_DIR", temp_path);

    // Run the command and capture its output
    let output = std::process::Command::new("cargo")
        .args(["run", "--quiet"])
        .output()
        .expect("Failed to execute command");

    // Clean up the environment
    env::remove_var("PONDER_EDITOR");
    env::remove_var("PONDER_DIR");

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stderr)
}

// Create a mock editor script for testing
fn create_mock_editor(path: &Path, content: &str, executable: bool) {
    let mut file = File::create(path).expect("Failed to create mock editor script");
    file.write_all(content.as_bytes())
        .expect("Failed to write to mock editor script");

    // Make the file executable (or not) based on the parameter
    if executable {
        let metadata = file.metadata().expect("Failed to get file metadata");
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(path, permissions).expect("Failed to set permissions");
    }
}

// This test needs to be run serially to prevent environment variable conflicts
#[test]
#[serial]
fn test_editor_command_not_found() {
    let (success, stderr) = run_with_editor("nonexistent_editor_command");
    assert!(!success, "Command with nonexistent editor should fail");
    assert!(
        stderr.contains("CommandNotFound"),
        "Error message should indicate the command was not found, got: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_editor_permission_denied() {
    // Create a temporary directory for our test script
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let script_path = temp_dir.path().join("non_executable_editor.sh");

    // Create a script file without execute permissions
    create_mock_editor(&script_path, "#!/bin/sh\necho 'This should not run'", false);

    // Set permissions explicitly to ensure it's not executable
    let metadata = fs::metadata(&script_path).expect("Failed to get metadata");
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o644); // rw-r--r--
    fs::set_permissions(&script_path, permissions).expect("Failed to set permissions");

    // Run the test with our non-executable script
    let (success, stderr) = run_with_editor(script_path.to_str().unwrap());

    assert!(!success, "Command with non-executable editor should fail");
    // The exact error might be platform-dependent, but should mention permission
    // On some platforms it could be "permission denied"
    assert!(
        stderr.contains("PermissionDenied") || stderr.contains("permission"),
        "Error message should indicate permission issues, got: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_editor_non_zero_exit() {
    // Create a temporary directory for our test script
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let script_path = temp_dir.path().join("failing_editor.sh");

    // Create a script that exits with code 1
    create_mock_editor(&script_path, "#!/bin/sh\nexit 1", true);

    // Run the test with our failing script
    let (success, stderr) = run_with_editor(script_path.to_str().unwrap());

    assert!(!success, "Command with failing editor should fail");
    assert!(
        stderr.contains("NonZeroExit"),
        "Error message should indicate non-zero exit status, got: {}",
        stderr
    );
    assert!(
        stderr.contains("1"),
        "Error message should include the exit code 1, got: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_successful_editor() {
    // Create a temporary directory for our test script
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let script_path = temp_dir.path().join("success_editor.sh");

    // Create a script that exits successfully
    create_mock_editor(&script_path, "#!/bin/sh\nexit 0", true);

    // Run the test with our successful script
    let (success, stderr) = run_with_editor(script_path.to_str().unwrap());

    assert!(
        success,
        "Command with successful editor should succeed: {}",
        stderr
    );
    assert!(
        stderr.is_empty(),
        "There should be no stderr output for successful editor, got: {}",
        stderr
    );
}
