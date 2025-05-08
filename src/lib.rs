pub mod cli;
pub mod config;
pub mod editor;
pub mod errors;
pub mod journal;

// Re-export important types for convenience
pub use cli::CliArgs;
pub use config::Config;
pub use editor::{Editor, SystemEditor};
pub use errors::{AppError, AppResult};
pub use journal::{DateSpecifier, JournalService};