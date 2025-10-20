use assert_cmd::Command;

pub const TEST_PASSPHRASE: &str = "test-passphrase";

/// Creates a `Command` for the `ponder` binary with a clean, non-interactive environment.
/// Additional environment variables or arguments can be configured by the caller.
pub fn base_ponder_command() -> Command {
    let mut cmd = Command::cargo_bin("ponder").expect("ponder binary not built");
    configure_ponder_command(&mut cmd);
    cmd
}

/// Applies the standard non-interactive environment to an existing `Command`.
pub fn configure_ponder_command(cmd: &mut Command) {
    #[cfg(target_os = "macos")]
    {
        cmd.env("PONDER_TEST_PASSPHRASE", TEST_PASSPHRASE);
    }
    #[cfg(not(target_os = "macos"))]
    {
        cmd.env_clear();
        if let Ok(path) = std::env::var("PATH") {
            cmd.env("PATH", path);
        }
        if let Ok(tmpdir) = std::env::var("TMPDIR") {
            cmd.env("TMPDIR", tmpdir);
        }
        cmd.env("PONDER_TEST_PASSPHRASE", TEST_PASSPHRASE);
    }
}
