//! Constants used throughout the application.
//!
//! This module contains all constants used in the Ponder application, organized
//! into logical groups. Having constants centralized makes them easier to find,
//! modify, and reference consistently.

// Application Metadata
/// The name of the application.
pub const APP_NAME: &str = "ponder";
/// The description of the application used in CLI help text.
pub const APP_DESCRIPTION: &str = "A simple journaling tool for daily reflections";

// CLI Arguments & Defaults
/// Default command for the editor if not specified otherwise.
pub const DEFAULT_EDITOR_COMMAND: &str = "vim";
/// Log format identifier for plain text.
pub const LOG_FORMAT_TEXT: &str = "text";
/// Log format identifier for JSON.
pub const LOG_FORMAT_JSON: &str = "json";
/// Default log level.
pub const DEFAULT_LOG_LEVEL: &str = "info";

// Configuration Keys & Environment Variables
/// Environment variable for specifying the Ponder journal directory.
pub const ENV_VAR_PONDER_DIR: &str = "PONDER_DIR";
/// Environment variable for specifying the preferred Ponder editor.
pub const ENV_VAR_PONDER_EDITOR: &str = "PONDER_EDITOR";
/// Standard environment variable for specifying the default editor.
pub const ENV_VAR_EDITOR: &str = "EDITOR";
/// Standard environment variable for the user's home directory.
pub const ENV_VAR_HOME: &str = "HOME";
/// Environment variable often used to indicate a Continuous Integration environment.
pub const ENV_VAR_CI: &str = "CI";
/// Default sub-directory name for journal entries within the user's home directory.
pub const DEFAULT_JOURNAL_SUBDIR: &str = "Documents/rubberducks";

// Validation
/// Characters forbidden in editor commands for security reasons.
pub const EDITOR_FORBIDDEN_CHARS: &[char] =
    &['|', '&', ';', '$', '(', ')', '`', '\\', '<', '>', '\'', '"'];
/// Placeholder string for redacted information in debug output.
pub const REDACTED_PLACEHOLDER: &str = "[REDACTED]";

// File System Parameters
/// File extension for journal entries.
pub const JOURNAL_FILE_EXTENSION: &str = ".md";
/// Default POSIX permissions for newly created directories (owner read/write/execute).
#[cfg(unix)]
pub const DEFAULT_DIR_PERMISSIONS: u32 = 0o700;
/// Default POSIX permissions for newly created files (owner read/write).
#[cfg(unix)]
pub const DEFAULT_FILE_PERMISSIONS: u32 = 0o600;

// Date/Time Logic
/// Date format string for ISO date format (YYYY-MM-DD).
pub const DATE_FORMAT_ISO: &str = "%Y-%m-%d";
/// Date format string for compact date format (YYYYMMDD).
pub const DATE_FORMAT_COMPACT: &str = "%Y%m%d";
/// Format string for journal filenames.
pub const FILENAME_FORMAT: &str = "{:04}{:02}{:02}{}";
/// Format string for the journal entry header.
pub const JOURNAL_HEADER_TEMPLATE: &str = "# {}\n\n## {}\n\n";
/// Date format used in journal headers.
pub const JOURNAL_HEADER_DATE_FORMAT: &str = "%B %d, %Y: %A";
/// Time format used in journal headers.
pub const JOURNAL_HEADER_TIME_FORMAT: &str = "%H:%M:%S";
/// Number of months ago for the 'one month' reminisce option.
pub const REMINISCE_ONE_MONTH_AGO: u32 = 1;
/// Number of months ago for the 'three months' reminisce option.
pub const REMINISCE_THREE_MONTHS_AGO: u32 = 3;
/// Number of months ago for the 'six months' reminisce option.
pub const REMINISCE_SIX_MONTHS_AGO: u32 = 6;
/// Number of months in a year.
pub const MONTHS_PER_YEAR: u32 = 12;
/// Maximum number of years ago for reminisce.
pub const MAX_REMINISCE_YEARS_AGO: u32 = 100;
/// Number of days ago for retro.
pub const RETRO_DAYS: i64 = 7;

// Logging Configuration
/// Service name used in tracing spans and structured logs.
pub const TRACING_SERVICE_NAME: &str = "ponder";
/// Name for the root tracing span covering an application invocation.
pub const TRACING_ROOT_SPAN_NAME: &str = "app_invocation";
// Test comment
