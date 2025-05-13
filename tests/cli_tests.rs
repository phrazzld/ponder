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
        .env("PONDER_EDITOR", "echo"); // Using 'echo' as a safe editor for testing
    cmd
}

#[test]
#[serial]
fn test_cli_no_args() {
    let mut cmd = set_up_command();

    // When running with no args, ponder should attempt to open today's entry
    // Since we're using "echo" as the editor, it should just echo the path and succeed
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(".md"));
}

#[test]
#[serial]
fn test_cli_retro_flag() {
    let mut cmd = set_up_command();

    // Test the --retro flag
    cmd.arg("--retro");

    // The behavior now is to create today's entry if no retro entries exist
    // So we simply verify that the command succeeds and outputs something
    cmd.assert().success();
}

#[test]
#[serial]
fn test_cli_reminisce_flag() {
    let mut cmd = set_up_command();

    // Test the --reminisce flag
    cmd.arg("--reminisce");

    // Since no reminisce entries will exist in the test directory,
    // And we're now using structured logging instead of println,
    // we just need to check that the command succeeds
    cmd.assert().success();
}

#[test]
#[serial]
fn test_cli_specific_date() {
    let mut cmd = set_up_command();

    // Test the --date flag with a specific date
    cmd.arg("--date").arg("2023-01-01");

    // Should succeed and call the editor with the specific date's file path
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("20230101.md"));
}

#[test]
#[serial]
fn test_cli_invalid_date() {
    let mut cmd = set_up_command();

    // Test an invalid date format
    cmd.arg("--date").arg("not-a-date");

    // Should fail with an error message about invalid date format
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid date format"));
}

#[test]
#[serial]
fn test_cli_verbose_flag() {
    let mut cmd = set_up_command();

    // Test with verbose flag
    cmd.arg("--verbose");

    // Should succeed and likely have more verbose output
    cmd.assert().success();
}

#[test]
#[serial]
fn test_cli_invalid_flags_combination() {
    let mut cmd = set_up_command();

    // Test incompatible flags
    cmd.arg("--retro").arg("--reminisce");

    // Should fail with an error about conflicting options
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}
