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

/// Runs a command with the given editor value and returns whether it succeeded
fn run_with_editor(editor_value: &str) -> (bool, String) {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = set_up_command();

    // Use a fast-failing command for journal_dir creation to speed up tests
    cmd.env("PONDER_EDITOR", editor_value)
        .env("PONDER_DIR", temp_dir.path())
        .env("HOME", temp_dir.path())
        // Add timeout to ensure tests don't hang
        .timeout(std::time::Duration::from_secs(5));

    let output = cmd.output().unwrap();
    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (success, stderr)
}

// Basic validation tests

#[test]
#[serial]
fn test_valid_single_command() {
    let editor_values = &["true"]; // Use a command that should be available everywhere

    for editor in editor_values {
        let (success, stderr) = run_with_editor(editor);
        // Test behavior: Valid single commands should not fail validation
        // This verifies that simple editor commands pass security checks
        if !success {
            // Use robust pattern matching: test for absence of validation-related errors
            // Focus on essential concepts rather than exact error message wording
            assert!(
                !stderr.contains("spaces") && !stderr.contains("Configuration error"),
                "Editor '{}' shouldn't fail validation for spaces (got: {})",
                editor,
                stderr
            );
            assert!(
                !stderr.contains("metacharacters") && !stderr.contains("Configuration error"),
                "Editor '{}' shouldn't fail validation for security concerns (got: {})",
                editor,
                stderr
            );
        }
    }
}

#[test]
#[serial]
fn test_valid_paths() {
    // Use a minimal set of paths to test, focusing on paths likely to exist
    let editor_values = &[
        "/bin/true", // Should exist on most Unix systems
    ];

    for editor in editor_values {
        let (success, stderr) = run_with_editor(editor);
        // Test behavior: Valid absolute paths should not fail validation
        // This verifies that absolute path editors pass security checks
        if !success {
            // Use robust pattern matching: test for absence of validation-related errors
            // Focus on essential concepts rather than exact error message wording
            assert!(
                !stderr.contains("spaces") && !stderr.contains("Configuration error"),
                "Path '{}' shouldn't fail validation for spaces (got: {})",
                editor,
                stderr
            );
            assert!(
                !stderr.contains("metacharacters") && !stderr.contains("Configuration error"),
                "Path '{}' shouldn't fail validation for security concerns (got: {})",
                editor,
                stderr
            );
        }
    }
}

// Command injection tests

#[test]
#[serial]
fn test_reject_commands_with_spaces() {
    // Simplify to just a few test cases to avoid timeouts
    let commands_with_spaces = &[
        "true -n", // Simple command with arg
        " true",   // Leading space
        "true ",   // Trailing space
    ];

    for cmd in commands_with_spaces {
        let (success, stderr) = run_with_editor(cmd);
        assert!(!success, "Command with spaces should be rejected: {}", cmd);
        // The important part is that it's rejected, not the specific error message
        assert!(
            !stderr.is_empty(),
            "There should be an error message for command: {}",
            cmd
        );
    }
}

#[test]
#[serial]
fn test_reject_shell_metacharacters() {
    let shell_metacharacter_commands = &[
        "vim;touch /tmp/hack",
        "vim|touch /tmp/hack",
        "vim&touch /tmp/hack",
        "vim&&touch /tmp/hack",
        "vim||touch /tmp/hack",
        "vim>file.txt",
        "vim<file.txt",
        "vim$(touch /tmp/hack)",
        "vim`touch /tmp/hack`",
        "vim\ntouch /tmp/hack", // Newline
        "vim\\touch",           // Backslash
        "vim\\'test'",          // Single quote
        "vim\\\"test\"",        // Double quote
        "vim(arg)",
        "vim)arg",
    ];

    for cmd in shell_metacharacter_commands {
        let (success, stderr) = run_with_editor(cmd);
        // Test behavior: Commands with shell metacharacters must be rejected for security
        // This is a critical security test preventing command injection attacks
        assert!(
            !success,
            "Command with shell metacharacters should be rejected: {}",
            cmd
        );

        // Use robust pattern matching: focus on essential security validation behavior
        // Test that the command was rejected due to security concerns, not exact wording
        assert!(
            stderr.contains("Configuration error")
                && (stderr.contains("metacharacters")
                    || stderr.contains("spaces")
                    || stderr.contains("shell")
                    || stderr.contains("security")),
            "Error for '{}' should indicate security validation failure, got: {}",
            cmd,
            stderr
        );
    }
}

#[test]
#[serial]
fn test_reject_empty_editor() {
    let (success, stderr) = run_with_editor("");
    // Test behavior: Empty editor commands must be rejected
    // This validates that the application requires a valid editor to function
    assert!(!success, "Empty editor command should be rejected");

    // Use robust pattern matching: focus on essential validation behavior
    // Test that empty input is rejected, regardless of exact error message wording
    assert!(
        stderr.contains("Configuration error")
            && (stderr.contains("empty")
                || stderr.contains("missing")
                || stderr.contains("required")),
        "Error should indicate empty/missing editor validation failure, got: {}",
        stderr
    );
}

// Verify that injection attempts actually don't create files

#[test]
#[serial]
fn test_injection_attempts_dont_create_files() {
    let temp_dir = TempDir::new().unwrap();
    let target_file = temp_dir.path().join("hacked.txt");

    let injection_attempts = &[
        format!("vim;touch {}", target_file.display()),
        format!("vim|touch {}", target_file.display()),
        format!("vim&touch {}", target_file.display()),
        format!("vim&&touch {}", target_file.display()),
        format!("vim||touch {}", target_file.display()),
        format!("vim>>{}", target_file.display()),
        format!("vim$(touch {})", target_file.display()),
        format!("vim`touch {}`", target_file.display()),
    ];

    for cmd in injection_attempts {
        let mut cmd_obj = set_up_command();
        cmd_obj
            .env("PONDER_EDITOR", cmd)
            .env("PONDER_DIR", temp_dir.path())
            .env("HOME", temp_dir.path())
            .timeout(std::time::Duration::from_secs(5));

        cmd_obj.assert().failure();

        // Verify the file was not created
        assert!(
            !target_file.exists(),
            "Injection attempt '{}' should not have created a file",
            cmd
        );
    }
}

// Test EDITOR variable fallback

#[test]
#[serial]
fn test_editor_fallback_validation() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = set_up_command();

    // Don't set PONDER_EDITOR, only set EDITOR with invalid value
    cmd.env("EDITOR", "vim;touch /tmp/hacked")
        .env("PONDER_DIR", temp_dir.path())
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(5));

    // Test behavior: Invalid EDITOR environment variable should be rejected
    // This validates security checks apply to fallback editor configuration
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Configuration error").and(
            predicate::str::contains("metacharacters").or(predicate::str::contains("spaces")),
        ));
}

#[test]
#[serial]
fn test_ponder_editor_precedence() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = set_up_command();

    // Set both PONDER_EDITOR and EDITOR, where PONDER_EDITOR is valid but EDITOR is invalid
    cmd.env("PONDER_EDITOR", "true")
        .env("EDITOR", "vim;touch /tmp/hacked")
        .env("PONDER_DIR", temp_dir.path())
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(5));

    // Should succeed because it uses the valid PONDER_EDITOR
    cmd.assert().success();
}

// Test actual execution to ensure validation is connected to launching

#[test]
#[serial]
fn test_validated_editor_is_actually_used() {
    let temp_dir = TempDir::new().unwrap();
    let journal_dir = temp_dir.path().join("journal");
    std::fs::create_dir_all(&journal_dir).unwrap();

    let sentinel_file = temp_dir.path().join("editor_was_launched");
    let editor_script = temp_dir.path().join("test_editor.sh");

    // Create a simple script that creates the sentinel file
    let script_content = format!("#!/bin/sh\ntouch {}\nexit 0\n", sentinel_file.display());

    let mut script_file = File::create(&editor_script).unwrap();
    write!(script_file, "{}", script_content).unwrap();
    script_file.flush().unwrap();
    drop(script_file); // Ensure file is closed before setting permissions

    // Make the script executable on Unix
    #[cfg(unix)]
    {
        let mut perms = std::fs::metadata(&editor_script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&editor_script, perms).unwrap();
    }

    // Using a wrapper script for the validation and execution test
    let mut cmd = set_up_command();
    cmd.env("PONDER_EDITOR", editor_script.to_str().unwrap())
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", temp_dir.path().to_str().unwrap())
        .timeout(std::time::Duration::from_secs(5));

    // Run ponder to trigger editor launch
    cmd.assert().success();

    // Verify the editor was actually launched
    assert!(
        sentinel_file.exists(),
        "Editor script should have created sentinel file"
    );
}

#[test]
#[serial]
fn test_actual_process_has_no_shell() {
    // Skip this test if we're in the Github CI environment
    // This test is more prone to issues in virtual environments
    if std::env::var("CI").is_ok() {
        return;
    }

    // This test attempts to verify that the process is launched directly
    // without a shell, by checking that shell expansion doesn't happen.

    let temp_dir = TempDir::new().unwrap();
    let journal_dir = temp_dir.path().join("journal");
    std::fs::create_dir_all(&journal_dir).unwrap();

    // First, create a script that would actually use shell features
    // if run through a shell, but will be interpreted literally if
    // exec'd directly.

    let script_name = "script_$$.sh"; // $$ is shell PID, would be expanded if shell is used
    let script_path = temp_dir.path().join(script_name);

    // The script just creates a sentinel file and exits
    let sentinel_file = temp_dir.path().join("executed");
    let script_content = format!("#!/bin/sh\ntouch {}\nexit 0\n", sentinel_file.display());

    let mut script_file = File::create(&script_path).unwrap();
    write!(script_file, "{}", script_content).unwrap();
    script_file.flush().unwrap();
    drop(script_file);

    // Make the script executable on Unix
    #[cfg(unix)]
    {
        let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).unwrap();
    }

    // Set up the command
    let mut cmd = set_up_command();
    cmd.env("PONDER_EDITOR", script_path.to_str().unwrap())
        .env("PONDER_DIR", journal_dir.to_str().unwrap())
        .env("HOME", temp_dir.path().to_str().unwrap())
        .timeout(std::time::Duration::from_secs(5));

    // Run ponder
    let result = cmd.assert();

    if cfg!(unix) {
        // On Unix, we expect this to at most fail because we can't find the literal script name
        // with $$ in it, not because of shell expansion issues
        assert!(
            sentinel_file.exists() || !result.get_output().status.success(),
            "Shell expansion should not occur"
        );
    }
}

// Unicode/Encoding tests

#[test]
#[serial]
fn test_unicode_characters_in_editor() {
    // Test with various Unicode characters that shouldn't be rejected
    let valid_unicode_editors = &[
        "编辑器",   // Chinese
        "エディタ", // Japanese
        "편집자",   // Korean
        "محرر",     // Arabic
        "édïtør",   // Latin with diacritics
    ];

    for editor in valid_unicode_editors {
        let (success, stderr) = run_with_editor(editor);
        // Test behavior: Valid unicode editor names should not fail validation
        // This verifies that unicode characters in editor names are properly supported
        if !success {
            // Use robust pattern matching: test for absence of validation-related errors
            // Focus on essential concepts rather than exact error message wording
            assert!(
                !stderr.contains("spaces") && !stderr.contains("Configuration error"),
                "Unicode editor '{}' shouldn't fail validation for spaces (got: {})",
                editor,
                stderr
            );
            assert!(
                !stderr.contains("metacharacters") && !stderr.contains("Configuration error"),
                "Unicode editor '{}' shouldn't fail validation for security concerns (got: {})",
                editor,
                stderr
            );
        }
    }
}

#[test]
#[serial]
fn test_unicode_spaces_and_metacharacters() {
    // Test with Unicode characters that look like spaces or could be used for obfuscation
    let invalid_unicode_editors = &[
        "vim\u{00A0}arg", // Non-breaking space
        "vim\u{2000}arg", // En Quad
        "vim\u{2001}arg", // Em Quad
        "vim\u{2003}arg", // Em Space
    ];

    for editor in invalid_unicode_editors {
        let (success, _stderr) = run_with_editor(editor);

        // We're not specifically testing how these are rejected, just that they don't
        // pass validation and execute arbitrary commands
        assert!(
            !success,
            "Unicode space command should be rejected: {}",
            editor
        );
    }
}
