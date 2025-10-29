//! Command-line interface for the ponder application.
//!
//! This module handles command-line argument parsing using the `clap` crate.
//! It defines the CLI structure and provides methods to parse and validate
//! command-line arguments.

use crate::constants;
use clap::{ArgGroup, Parser, Subcommand};
use std::fmt;

/// Command-line arguments for the ponder application.
///
/// Ponder v2.0 uses a subcommand architecture:
/// - `edit`: Edit journal entries with encryption
/// - `ask`: Query journal with RAG (Retrieval-Augmented Generation)
/// - `reflect`: Generate AI reflection on an entry
/// - `search`: Semantic search over journal entries
/// - `lock`: Lock the encrypted session
/// - `backup`: Create encrypted backup of journal and database
/// - `restore`: Restore from encrypted backup
///
/// # Examples
///
/// ```no_run
/// use ponder::cli::{CliArgs, PonderCommand, EditArgs};
/// use clap::Parser;
///
/// // Edit today's entry
/// let args = CliArgs::parse_from(["ponder", "edit"]);
///
/// // Ask a question
/// let args = CliArgs::parse_from(["ponder", "ask", "What did I write about goals?"]);
///
/// // Search entries
/// let args = CliArgs::parse_from(["ponder", "search", "anxiety", "--limit", "5"]);
/// ```
#[derive(Parser)]
#[clap(
    name = constants::APP_NAME,
    about = constants::APP_DESCRIPTION
)]
#[clap(author, version, long_about = None)]
pub struct CliArgs {
    /// Subcommand to execute
    #[clap(subcommand)]
    pub command: Option<PonderCommand>,

    /// Enables verbose output (applies to all commands)
    #[clap(short = 'v', long, global = true)]
    pub verbose: bool,

    /// Sets the log output format (applies to all commands)
    #[clap(long = "log-format", value_parser = [constants::LOG_FORMAT_TEXT, constants::LOG_FORMAT_JSON], default_value = constants::LOG_FORMAT_TEXT, global = true)]
    pub log_format: String,
}

/// Subcommands for ponder operations.
#[derive(Subcommand)]
pub enum PonderCommand {
    /// Edit a journal entry with encryption
    Edit(EditArgs),

    /// Query journal entries using AI (RAG)
    Ask(AskArgs),

    /// Generate AI reflection on a journal entry
    Reflect(ReflectArgs),

    /// Generate AI summaries (daily, weekly, monthly)
    Summarize(SummarizeArgs),

    /// Semantic search over journal entries
    Search(SearchArgs),

    /// Lock the encrypted session (clear passphrase from memory)
    Lock,

    /// Create encrypted backup of journal entries and database
    Backup(BackupArgs),

    /// Restore from encrypted backup archive
    Restore(RestoreArgs),

    /// Regenerate embeddings for journal entries
    Reindex,

    /// Clean up v1.0 entries after successful migration (deletes only verified entries)
    CleanupV1(CleanupV1Args),

    /// Show journal database health and statistics
    Status,
}

/// Arguments for the `edit` subcommand.
#[derive(Parser)]
#[clap(group(ArgGroup::new("entry_type").args(&["retro", "reminisce", "date"])))]
pub struct EditArgs {
    /// Opens entries from the past week excluding today
    #[clap(short = 'r', long, conflicts_with_all = &["reminisce", "date"])]
    pub retro: bool,

    /// Opens entries from significant past intervals
    #[clap(short = 'm', long, conflicts_with_all = &["retro", "date"])]
    pub reminisce: bool,

    /// Opens an entry for a specific date (YYYY-MM-DD or YYYYMMDD)
    #[clap(short = 'd', long, conflicts_with_all = &["retro", "reminisce"])]
    pub date: Option<String>,

    /// Migrate v1.0 plaintext entries to v2.0 encrypted format
    #[clap(long)]
    pub migrate: bool,
}

/// Arguments for the `ask` subcommand.
#[derive(Parser)]
pub struct AskArgs {
    /// Question to ask about your journal entries
    pub question: String,

    /// Optional date range start (YYYY-MM-DD)
    #[clap(long)]
    pub from: Option<String>,

    /// Optional date range end (YYYY-MM-DD)
    #[clap(long)]
    pub to: Option<String>,
}

/// Arguments for the `reflect` subcommand.
#[derive(Parser)]
pub struct ReflectArgs {
    /// Date of entry to reflect on (YYYY-MM-DD or YYYYMMDD, defaults to today)
    #[clap(short = 'd', long)]
    pub date: Option<String>,
}

/// Summary period granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryPeriod {
    Daily,
    Weekly,
    Monthly,
}

impl std::str::FromStr for SummaryPeriod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" | "day" => Ok(SummaryPeriod::Daily),
            "weekly" | "week" => Ok(SummaryPeriod::Weekly),
            "monthly" | "month" => Ok(SummaryPeriod::Monthly),
            _ => Err(format!(
                "Invalid summary period: '{}'. Valid options: daily, weekly, monthly",
                s
            )),
        }
    }
}

impl std::fmt::Display for SummaryPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SummaryPeriod::Daily => write!(f, "daily"),
            SummaryPeriod::Weekly => write!(f, "weekly"),
            SummaryPeriod::Monthly => write!(f, "monthly"),
        }
    }
}

/// Arguments for the `summarize` subcommand.
#[derive(Parser)]
pub struct SummarizeArgs {
    /// Summary period (daily, weekly, or monthly)
    #[clap(value_parser = clap::value_parser!(SummaryPeriod))]
    pub period: SummaryPeriod,

    /// Date for the summary (YYYY-MM-DD or YYYYMMDD, defaults to today for daily, last Sunday for weekly, current month for monthly)
    #[clap(short = 'd', long)]
    pub date: Option<String>,
}

/// Arguments for the `search` subcommand.
#[derive(Parser)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Maximum number of results to return
    #[clap(short = 'l', long, default_value = "5")]
    pub limit: usize,

    /// Optional date range start (YYYY-MM-DD)
    #[clap(long)]
    pub from: Option<String>,

    /// Optional date range end (YYYY-MM-DD)
    #[clap(long)]
    pub to: Option<String>,
}

/// Arguments for the `backup` subcommand.
#[derive(Parser)]
pub struct BackupArgs {
    /// Output path for the backup archive (will be encrypted with .age extension)
    pub output: std::path::PathBuf,

    /// Verify backup integrity after creation
    #[clap(long)]
    pub verify: bool,
}

/// Arguments for the `restore` subcommand.
#[derive(Parser)]
pub struct RestoreArgs {
    /// Path to the encrypted backup archive
    pub backup: std::path::PathBuf,

    /// Force overwrite if target directory exists
    #[clap(short = 'f', long)]
    pub force: bool,
}

/// Arguments for the `cleanup-v1` subcommand.
#[derive(Parser)]
pub struct CleanupV1Args {
    /// Skip confirmation prompt (dangerous - deletes files)
    #[clap(short = 'y', long)]
    pub yes: bool,
}

impl fmt::Debug for CliArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliArgs")
            .field("command", &"[REDACTED]") // Redact to avoid exposing sensitive journal queries
            .field("verbose", &self.verbose)
            .field("log_format", &self.log_format)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_no_command() {
        let args = CliArgs::parse_from(vec!["ponder"]);
        assert!(args.command.is_none());
        assert!(!args.verbose);
        assert_eq!(args.log_format, constants::LOG_FORMAT_TEXT);
    }

    #[test]
    fn test_edit_today() {
        let args = CliArgs::parse_from(vec!["ponder", "edit"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(!edit_args.retro);
                assert!(!edit_args.reminisce);
                assert!(edit_args.date.is_none());
            }
            _ => panic!("Expected Edit command"),
        }
    }

    #[test]
    fn test_edit_retro() {
        let args = CliArgs::parse_from(vec!["ponder", "edit", "--retro"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(edit_args.retro);
                assert!(!edit_args.reminisce);
                assert!(edit_args.date.is_none());
            }
            _ => panic!("Expected Edit command"),
        }

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "edit", "-r"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(edit_args.retro);
            }
            _ => panic!("Expected Edit command"),
        }
    }

    #[test]
    fn test_edit_reminisce() {
        let args = CliArgs::parse_from(vec!["ponder", "edit", "--reminisce"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(!edit_args.retro);
                assert!(edit_args.reminisce);
                assert!(edit_args.date.is_none());
            }
            _ => panic!("Expected Edit command"),
        }

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "edit", "-m"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(edit_args.reminisce);
            }
            _ => panic!("Expected Edit command"),
        }
    }

    #[test]
    fn test_edit_specific_date() {
        let args = CliArgs::parse_from(vec!["ponder", "edit", "--date", "2023-01-15"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(!edit_args.retro);
                assert!(!edit_args.reminisce);
                assert_eq!(edit_args.date, Some("2023-01-15".to_string()));
            }
            _ => panic!("Expected Edit command"),
        }

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "edit", "-d", "20230115"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert_eq!(edit_args.date, Some("20230115".to_string()));
            }
            _ => panic!("Expected Edit command"),
        }
    }

    #[test]
    fn test_edit_migrate_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "edit", "--migrate"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(edit_args.migrate);
                assert!(!edit_args.retro);
                assert!(!edit_args.reminisce);
                assert!(edit_args.date.is_none());
            }
            _ => panic!("Expected Edit command"),
        }

        // Test migrate with today (implicit)
        let args = CliArgs::parse_from(vec!["ponder", "edit"]);
        match args.command {
            Some(PonderCommand::Edit(edit_args)) => {
                assert!(!edit_args.migrate);
            }
            _ => panic!("Expected Edit command"),
        }
    }

    #[test]
    fn test_ask_command() {
        let args = CliArgs::parse_from(vec!["ponder", "ask", "What did I write about goals?"]);
        match args.command {
            Some(PonderCommand::Ask(ask_args)) => {
                assert_eq!(ask_args.question, "What did I write about goals?");
                assert!(ask_args.from.is_none());
                assert!(ask_args.to.is_none());
            }
            _ => panic!("Expected Ask command"),
        }

        // Test with date range
        let args = CliArgs::parse_from(vec![
            "ponder",
            "ask",
            "goals",
            "--from",
            "2024-01-01",
            "--to",
            "2024-06-30",
        ]);
        match args.command {
            Some(PonderCommand::Ask(ask_args)) => {
                assert_eq!(ask_args.question, "goals");
                assert_eq!(ask_args.from, Some("2024-01-01".to_string()));
                assert_eq!(ask_args.to, Some("2024-06-30".to_string()));
            }
            _ => panic!("Expected Ask command"),
        }
    }

    #[test]
    fn test_reflect_command() {
        // Default (today)
        let args = CliArgs::parse_from(vec!["ponder", "reflect"]);
        match args.command {
            Some(PonderCommand::Reflect(reflect_args)) => {
                assert!(reflect_args.date.is_none());
            }
            _ => panic!("Expected Reflect command"),
        }

        // Specific date
        let args = CliArgs::parse_from(vec!["ponder", "reflect", "--date", "2024-01-15"]);
        match args.command {
            Some(PonderCommand::Reflect(reflect_args)) => {
                assert_eq!(reflect_args.date, Some("2024-01-15".to_string()));
            }
            _ => panic!("Expected Reflect command"),
        }
    }

    #[test]
    fn test_search_command() {
        let args = CliArgs::parse_from(vec!["ponder", "search", "anxiety"]);
        match args.command {
            Some(PonderCommand::Search(search_args)) => {
                assert_eq!(search_args.query, "anxiety");
                assert_eq!(search_args.limit, 5); // Default
                assert!(search_args.from.is_none());
                assert!(search_args.to.is_none());
            }
            _ => panic!("Expected Search command"),
        }

        // Test with limit and date range
        let args = CliArgs::parse_from(vec![
            "ponder",
            "search",
            "project-x",
            "--limit",
            "10",
            "--from",
            "2024-01-01",
            "--to",
            "2024-12-31",
        ]);
        match args.command {
            Some(PonderCommand::Search(search_args)) => {
                assert_eq!(search_args.query, "project-x");
                assert_eq!(search_args.limit, 10);
                assert_eq!(search_args.from, Some("2024-01-01".to_string()));
                assert_eq!(search_args.to, Some("2024-12-31".to_string()));
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_lock_command() {
        let args = CliArgs::parse_from(vec!["ponder", "lock"]);
        match args.command {
            Some(PonderCommand::Lock) => {} // Success
            _ => panic!("Expected Lock command"),
        }
    }

    #[test]
    fn test_cleanup_v1_command() {
        let args = CliArgs::parse_from(vec!["ponder", "cleanup-v1"]);
        match args.command {
            Some(PonderCommand::CleanupV1(cleanup_args)) => {
                assert!(!cleanup_args.yes);
            }
            _ => panic!("Expected CleanupV1 command"),
        }

        // Test with --yes flag
        let args = CliArgs::parse_from(vec!["ponder", "cleanup-v1", "--yes"]);
        match args.command {
            Some(PonderCommand::CleanupV1(cleanup_args)) => {
                assert!(cleanup_args.yes);
            }
            _ => panic!("Expected CleanupV1 command"),
        }

        // Test with -y short form
        let args = CliArgs::parse_from(vec!["ponder", "cleanup-v1", "-y"]);
        match args.command {
            Some(PonderCommand::CleanupV1(cleanup_args)) => {
                assert!(cleanup_args.yes);
            }
            _ => panic!("Expected CleanupV1 command"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        let args = CliArgs::parse_from(vec!["ponder", "--verbose", "edit"]);
        assert!(args.verbose);

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "-v", "edit"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_log_format_option() {
        // Default
        let args = CliArgs::parse_from(vec!["ponder", "edit"]);
        assert_eq!(args.log_format, constants::LOG_FORMAT_TEXT);

        // Explicit text
        let args = CliArgs::parse_from(vec![
            "ponder",
            "--log-format",
            constants::LOG_FORMAT_TEXT,
            "edit",
        ]);
        assert_eq!(args.log_format, constants::LOG_FORMAT_TEXT);

        // JSON format
        let args = CliArgs::parse_from(vec![
            "ponder",
            "--log-format",
            constants::LOG_FORMAT_JSON,
            "edit",
        ]);
        assert_eq!(args.log_format, constants::LOG_FORMAT_JSON);
    }

    #[test]
    fn test_debug_impl_redacts_sensitive_info() {
        let args = CliArgs {
            command: Some(PonderCommand::Ask(AskArgs {
                question: "What did I write about anxiety?".to_string(),
                from: None,
                to: None,
            })),
            verbose: true,
            log_format: "text".to_string(),
        };

        let debug_output = format!("{:?}", args);

        // Verify flags are visible but command is redacted
        assert!(debug_output.contains("verbose: true"));
        assert!(debug_output.contains("[REDACTED]"));
        // Sensitive query should not appear
        assert!(!debug_output.contains("anxiety"));
    }

    #[test]
    fn test_backup_command() {
        let args = CliArgs::parse_from(vec!["ponder", "backup", "backup.tar.gz.age"]);
        match args.command {
            Some(PonderCommand::Backup(backup_args)) => {
                assert_eq!(
                    backup_args.output,
                    std::path::PathBuf::from("backup.tar.gz.age")
                );
                assert!(!backup_args.verify);
            }
            _ => panic!("Expected Backup command"),
        }

        // Test with verify flag
        let args = CliArgs::parse_from(vec!["ponder", "backup", "output.age", "--verify"]);
        match args.command {
            Some(PonderCommand::Backup(backup_args)) => {
                assert!(backup_args.verify);
            }
            _ => panic!("Expected Backup command"),
        }
    }

    #[test]
    fn test_restore_command() {
        let args = CliArgs::parse_from(vec!["ponder", "restore", "backup.tar.gz.age"]);
        match args.command {
            Some(PonderCommand::Restore(restore_args)) => {
                assert_eq!(
                    restore_args.backup,
                    std::path::PathBuf::from("backup.tar.gz.age")
                );
                assert!(!restore_args.force);
            }
            _ => panic!("Expected Restore command"),
        }

        // Test with force flag
        let args = CliArgs::parse_from(vec!["ponder", "restore", "backup.age", "--force"]);
        match args.command {
            Some(PonderCommand::Restore(restore_args)) => {
                assert!(restore_args.force);
            }
            _ => panic!("Expected Restore command"),
        }

        // Test short form
        let args = CliArgs::parse_from(vec!["ponder", "restore", "backup.age", "-f"]);
        match args.command {
            Some(PonderCommand::Restore(restore_args)) => {
                assert!(restore_args.force);
            }
            _ => panic!("Expected Restore command"),
        }
    }

    #[test]
    fn test_summarize_daily() {
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "daily"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Daily);
                assert!(summarize_args.date.is_none());
            }
            _ => panic!("Expected Summarize command"),
        }

        // Test with date
        let args =
            CliArgs::parse_from(vec!["ponder", "summarize", "daily", "--date", "2024-01-15"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Daily);
                assert_eq!(summarize_args.date, Some("2024-01-15".to_string()));
            }
            _ => panic!("Expected Summarize command"),
        }

        // Test with short date flag
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "daily", "-d", "20240115"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Daily);
                assert_eq!(summarize_args.date, Some("20240115".to_string()));
            }
            _ => panic!("Expected Summarize command"),
        }
    }

    #[test]
    fn test_summarize_weekly() {
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "weekly"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Weekly);
                assert!(summarize_args.date.is_none());
            }
            _ => panic!("Expected Summarize command"),
        }

        // Test with date
        let args = CliArgs::parse_from(vec![
            "ponder",
            "summarize",
            "weekly",
            "--date",
            "2024-01-07",
        ]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Weekly);
                assert_eq!(summarize_args.date, Some("2024-01-07".to_string()));
            }
            _ => panic!("Expected Summarize command"),
        }
    }

    #[test]
    fn test_summarize_monthly() {
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "monthly"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Monthly);
                assert!(summarize_args.date.is_none());
            }
            _ => panic!("Expected Summarize command"),
        }

        // Test with date
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "monthly", "--date", "2024-01"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Monthly);
                assert_eq!(summarize_args.date, Some("2024-01".to_string()));
            }
            _ => panic!("Expected Summarize command"),
        }
    }

    #[test]
    fn test_summary_period_aliases() {
        // Test "day" alias
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "day"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Daily);
            }
            _ => panic!("Expected Summarize command"),
        }

        // Test "week" alias
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "week"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Weekly);
            }
            _ => panic!("Expected Summarize command"),
        }

        // Test "month" alias
        let args = CliArgs::parse_from(vec!["ponder", "summarize", "month"]);
        match args.command {
            Some(PonderCommand::Summarize(summarize_args)) => {
                assert_eq!(summarize_args.period, SummaryPeriod::Monthly);
            }
            _ => panic!("Expected Summarize command"),
        }
    }
}
