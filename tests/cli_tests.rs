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
fn test_cli_retro_flag() {
    let mut cmd = set_up_command();

    // v2.0: Test the --retro flag with edit subcommand
    cmd.arg("edit").arg("--retro");

    // The behavior now is to create today's entry if no retro entries exist
    // So we simply verify that the command succeeds and outputs something
    cmd.assert().success();
}

#[test]
#[serial]
fn test_cli_reminisce_flag() {
    let mut cmd = set_up_command();

    // v2.0: Test the --reminisce flag with edit subcommand
    cmd.arg("edit").arg("--reminisce");

    // Since no reminisce entries will exist in the test directory,
    // And we're now using structured logging instead of println,
    // we just need to check that the command succeeds
    cmd.assert().success();
}

#[test]
#[serial]
fn test_cli_specific_date() {
    let mut cmd = set_up_command();

    // v2.0: Test the --date flag with edit subcommand
    cmd.arg("edit").arg("--date").arg("2023-01-01");

    // v2.0: Should succeed and call the editor with the encrypted file path (YYYY/MM/DD.md.age)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("2023/01/01.md.age"));
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
