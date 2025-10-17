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
/// Environment variable for specifying the database path.
pub const ENV_VAR_PONDER_DB: &str = "PONDER_DB";
/// Environment variable for specifying session timeout in minutes.
pub const ENV_VAR_PONDER_SESSION_TIMEOUT: &str = "PONDER_SESSION_TIMEOUT";
/// Environment variable for specifying Ollama API URL.
pub const ENV_VAR_OLLAMA_URL: &str = "OLLAMA_URL";
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

// Cryptography Configuration
/// Default session timeout in minutes for passphrase caching.
///
/// After this period of inactivity, the user will need to re-enter their passphrase.
/// This balances security (shorter timeout = less exposure) with usability
/// (longer timeout = fewer prompts).
pub const DEFAULT_SESSION_TIMEOUT_MINUTES: u64 = 30;

/// File extension for encrypted files using age encryption.
///
/// Encrypted journal entries will have this extension appended (e.g., "20240615.md.age").
pub const ENCRYPTED_FILE_EXTENSION: &str = ".age";

/// RAM-based temporary filesystem paths to check for secure temp file storage.
///
/// These tmpfs filesystems store files in RAM, minimizing disk persistence of
/// decrypted content. Checked in order; falls back to system temp if none found.
pub const TMPFS_PATHS: &[&str] = &["/dev/shm", "/run/shm"];

// Database Configuration
/// Default database filename within the journal directory.
///
/// The encrypted database is stored alongside journal entries.
pub const DEFAULT_DB_FILENAME: &str = "ponder.db";

/// Embedding vector dimensions for the nomic-embed-text model.
///
/// nomic-embed-text produces 768-dimensional embeddings. This constant
/// ensures consistency across embedding storage and retrieval operations.
pub const EMBEDDING_DIMENSIONS: usize = 768;

/// Default limit for vector similarity search results.
///
/// Controls how many similar chunks to return when performing semantic search.
/// This balances result relevance with processing time and context window constraints.
pub const VECTOR_SEARCH_LIMIT: usize = 12;

// AI Configuration
/// Default URL for Ollama API.
///
/// The local Ollama server typically runs on this address.
pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";

/// Default embedding model for semantic search.
///
/// embeddinggemma is a 300M parameter embedding model from Google optimized for retrieval tasks.
/// It produces 768-dimensional embeddings and is the smallest high-quality embedding model available.
pub const DEFAULT_EMBED_MODEL: &str = "embeddinggemma";

/// Default chat model for insights and reflections.
///
/// gemma3:4b is a high-quality model from Google suitable for local inference.
/// It provides excellent performance for chat and reflection tasks.
pub const DEFAULT_CHAT_MODEL: &str = "gemma3:4b";

/// Default chunk size in words for text chunking.
///
/// Approximately 700 words provides good balance between context and granularity
/// for embedding generation. This is roughly equivalent to 2-3 paragraphs.
pub const DEFAULT_CHUNK_SIZE: usize = 700;

/// Default overlap in words between consecutive chunks.
///
/// 100 words of overlap helps maintain context continuity at chunk boundaries
/// and improves retrieval quality for queries that span chunk edges.
pub const DEFAULT_CHUNK_OVERLAP: usize = 100;
