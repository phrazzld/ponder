use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Debug helper to show directory contents with metadata
#[allow(dead_code)]
pub fn debug_directory_state(dir: &Path) -> String {
    if !dir.exists() {
        return format!("Directory does not exist: {}", dir.display());
    }

    let mut output = format!("Directory: {}\n", dir.display());

    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut count = 0;
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();

                        match entry.metadata() {
                            Ok(metadata) => {
                                let file_type = if metadata.is_dir() { "DIR " } else { "FILE" };
                                let size = metadata.len();
                                #[cfg(unix)]
                                let permissions = format!("{:o}", metadata.permissions().mode());
                                #[cfg(not(unix))]
                                let permissions = if metadata.permissions().readonly() {
                                    "ro"
                                } else {
                                    "rw"
                                }
                                .to_string();

                                output.push_str(&format!(
                                    "  {} {:>8} bytes  mode:{} {}\n",
                                    file_type,
                                    size,
                                    permissions,
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                ));
                            }
                            Err(e) => {
                                output.push_str(&format!(
                                    "  ERROR reading metadata for {}: {}\n",
                                    path.file_name().unwrap_or_default().to_string_lossy(),
                                    e
                                ));
                            }
                        }
                        count += 1;
                    }
                    Err(e) => {
                        output.push_str(&format!("  ERROR reading entry: {}\n", e));
                    }
                }
            }
            if count == 0 {
                output.push_str("  (empty directory)\n");
            }
        }
        Err(e) => {
            output.push_str(&format!("  ERROR reading directory: {}\n", e));
        }
    }

    output
}

/// Debug helper to show file information
#[allow(dead_code)]
pub fn debug_file_info(file: &Path) -> String {
    if !file.exists() {
        return format!("File does not exist: {}", file.display());
    }

    match fs::metadata(file) {
        Ok(metadata) => {
            let file_type = if metadata.is_dir() {
                "directory"
            } else {
                "file"
            };
            let size = metadata.len();
            #[cfg(unix)]
            let permissions = format!("{:o}", metadata.permissions().mode());
            #[cfg(not(unix))]
            let permissions = if metadata.permissions().readonly() {
                "readonly"
            } else {
                "read-write"
            }
            .to_string();
            let modified = metadata
                .modified()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|_| "unknown".to_string());

            format!(
                "File: {} ({})\n  Size: {} bytes\n  Permissions: {}\n  Modified: {}",
                file.display(),
                file_type,
                size,
                permissions,
                modified
            )
        }
        Err(e) => {
            format!(
                "File exists but cannot read metadata: {} (error: {})",
                file.display(),
                e
            )
        }
    }
}

/// Debug helper to show relevant environment variables
#[allow(dead_code)]
pub fn debug_environment_state() -> String {
    let vars = ["PONDER_DIR", "PONDER_EDITOR", "EDITOR", "HOME", "PWD"];
    let mut output = String::from("Environment variables:\n");

    for var in &vars {
        match env::var(var) {
            Ok(value) => output.push_str(&format!("  {}={}\n", var, value)),
            Err(_) => output.push_str(&format!("  {} (not set)\n", var)),
        }
    }

    output
}

/// Debug helper to show command execution context
#[allow(dead_code)]
pub fn debug_command_context(command: &str, args: &[&str], current_dir: Option<&Path>) -> String {
    let mut output = "Command execution context:\n".to_string();
    output.push_str(&format!("  Command: {}\n", command));

    if !args.is_empty() {
        output.push_str("  Arguments:\n");
        for (i, arg) in args.iter().enumerate() {
            output.push_str(&format!("    [{}]: {}\n", i, arg));
        }
    } else {
        output.push_str("  Arguments: (none)\n");
    }

    if let Some(dir) = current_dir {
        output.push_str(&format!("  Working directory: {}\n", dir.display()));
    }

    output.push_str(&debug_environment_state());
    output
}

/// Macro to enhance assertions with contextual debug information
#[macro_export]
macro_rules! assert_with_context {
    ($cond:expr, $context_fn:expr) => {
        assert!($cond, "Assertion failed with context:\n{}", $context_fn())
    };
    ($cond:expr, $msg:expr, $context_fn:expr) => {
        assert!($cond, "{}\nContext:\n{}", $msg, $context_fn())
    };
}

/// Debug helper to show comparison between expected and actual values
#[allow(dead_code)]
pub fn debug_comparison<T: std::fmt::Debug>(expected: &T, actual: &T, context: &str) -> String {
    format!(
        "{}\n  Expected: {:?}\n  Actual:   {:?}",
        context, expected, actual
    )
}
