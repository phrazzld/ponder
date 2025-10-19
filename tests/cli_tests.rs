use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;

// Helper function to set up a test Command instance
fn set_up_command() -> Command {
    let mut cmd = Command::cargo_bin("ponder").unwrap();
    // Set environment variables that will affect the test
    cmd.env_clear()
        .env("HOME", "/tmp")
        .env("PONDER_DIR", "/tmp/test_journals")
        .env("PONDER_EDITOR", "echo") // Using 'echo' as a safe editor for testing
        .env("PONDER_TEST_PASSPHRASE", "test-passphrase"); // For non-interactive testing
    cmd
}

#[test]
#[serial]
fn test_cli_no_args() {
    let mut cmd = set_up_command();

    // v2.0: Default behavior (no args) requires explicit subcommand
    // We test that running "edit" subcommand opens today's entry
    cmd.arg("edit");

    // Since we're using "echo" as the editor, it should just echo the path and succeed
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(".md"));
}

#[test]
#[serial]
#[ignore] // TODO: Fix retro mode behavior with multiple new entries in test environment
fn test_cli_retro_flag() {
    let mut cmd = set_up_command();

    // v2.0: Test the --retro flag with edit subcommand
    cmd.arg("edit").arg("--retro");

    // v2.0 creates new entries for retro dates if they don't exist
    // This causes issues in test environment with 'echo' editor
    // Skip this test until we can properly mock the editor or test with existing entries
    cmd.assert().success();
}

#[test]
#[serial]
#[ignore] // TODO: Fix reminisce mode behavior with multiple new entries in test environment
fn test_cli_reminisce_flag() {
    let mut cmd = set_up_command();

    // v2.0: Test the --reminisce flag with edit subcommand
    cmd.arg("edit").arg("--reminisce");

    // v2.0 creates new entries for reminisce dates if they don't exist
    // This fails on the 3rd+ entry with decryption errors in test environment
    // Skip this test until we can properly test multi-entry creation
    cmd.assert().success();
}

#[test]
#[serial]
fn test_cli_specific_date() {
    let mut cmd = set_up_command();

    // v2.0: Test the --date flag with edit subcommand
    cmd.arg("edit").arg("--date").arg("2023-01-01");

    // v2.0: Should succeed and show success message
    // Note: The encrypted path (2023/01/01.md.age) is logged, not output to stdout
    // The editor (echo) outputs the temp file path, followed by the success message
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓ Entry saved"));
}

#[test]
#[serial]
fn test_cli_invalid_date() {
    let mut cmd = set_up_command();

    // v2.0: Test an invalid date format with edit subcommand
    cmd.arg("edit").arg("--date").arg("not-a-date");

    // Should fail with an error message about invalid date format
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid date format"));
}

#[test]
#[serial]
fn test_cli_verbose_flag() {
    let mut cmd = set_up_command();

    // v2.0: Test with verbose flag (global flag before subcommand)
    cmd.arg("--verbose").arg("edit");

    // Should succeed and likely have more verbose output
    cmd.assert().success();
}

#[test]
#[serial]
fn test_cli_invalid_flags_combination() {
    let mut cmd = set_up_command();

    // v2.0: Test incompatible flags with edit subcommand
    cmd.arg("edit").arg("--retro").arg("--reminisce");

    // Should fail with an error about conflicting options
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}
