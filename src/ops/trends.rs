//! Comprehensive trend analysis operations.
//!
//! This module implements exhaustive trend analysis across all user journal entries.
//! Unlike semantic search (which finds similar content), trend analysis examines
//! EVERY entry to identify patterns, themes, and insights over time.
//!
//! # Pipeline Architecture
//!
//! 1. **Filter**: Identify user-created entries (exclude AI-generated reports/summaries)
//! 2. **Phase 1 - Discovery**: For each entry, ask LLM "Is this relevant to query?"
//! 3. **Phase 2 - Extraction**: From relevant entries, extract specific insights/excerpts
//! 4. **Phase 3 - Synthesis**: Generate comprehensive report with executive summary
//! 5. **Persistence**: Encrypt report, store in `reports/trends/`, record metadata
//!
//! # Usage
//!
//! ```no_run
//! use ponder::ops::analyze_trends;
//! use ponder::db::Database;
//! use ponder::crypto::SessionManager;
//! use ponder::ai::OllamaClient;
//!
//! let db = Database::open(&db_path, &passphrase)?;
//! let mut session = SessionManager::new(passphrase.clone(), 1800);
//! let ai_client = OllamaClient::new("http://127.0.0.1:11434")?;
//!
//! let report = analyze_trends(&db, &mut session, &ai_client, "productivity patterns", None)?;
//! println!("Report saved to: {}", report.path.display());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::ai::OllamaClient;
use crate::constants::DEFAULT_CHAT_MODEL;
use crate::crypto::age::encrypt_with_passphrase;
use crate::crypto::temp::decrypt_to_temp;
use crate::crypto::SessionManager;
use crate::db::entries::Entry;
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use chrono::NaiveDate;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Result of trend analysis with metadata about the generated report.
#[derive(Debug, Clone)]
pub struct TrendReport {
    /// Path to the encrypted report file
    pub path: PathBuf,
    /// The user's original query
    pub query: String,
    /// Number of entries analyzed
    pub total_entries: usize,
    /// Number of entries deemed relevant
    pub relevant_entries: usize,
    /// Database record ID for the report
    pub report_id: i64,
}

/// Insight extracted from a single journal entry.
#[derive(Debug, Clone)]
struct EntryInsight {
    /// Date of the entry
    date: NaiveDate,
    /// Relevant excerpt(s) from the entry
    excerpts: Vec<String>,
    /// Summary of the insight
    summary: String,
}

/// Checkpoint for resumable trend analysis operations.
///
/// Allows long-running trend analysis to be interrupted and resumed.
/// Stored as JSON in platform cache directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrendsCheckpoint {
    /// Original query for trend analysis
    query: String,
    /// ISO 8601 timestamp when analysis started
    started_at: String,
    /// Current phase of analysis
    phase: CheckpointPhase,
    /// Total number of entries being analyzed
    total_entries: usize,
    /// Entry IDs that have been processed (relevance checked)
    processed_entry_ids: HashSet<i64>,
    /// Entry IDs deemed relevant to the query
    relevant_entry_ids: Vec<i64>,
    /// Insights extracted from relevant entries
    extracted_insights: Vec<SerializableInsight>,
}

/// Phase of trend analysis for checkpointing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum CheckpointPhase {
    /// Phase 1: Checking entry relevance
    Discovery,
    /// Phase 2: Extracting insights from relevant entries
    Extraction,
}

/// Serializable version of EntryInsight (NaiveDate → String).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableInsight {
    /// Date in YYYY-MM-DD format
    date: String,
    /// Relevant excerpts from the entry
    excerpts: Vec<String>,
    /// Summary of the insight
    summary: String,
}

/// Progress bar wrapper for trend analysis phases.
///
/// Provides a consistent progress bar style across all phases with ETA,
/// throughput metrics, and smooth animation.
struct PhaseProgress {
    bar: ProgressBar,
    phase_name: String,
    start_time: Instant,
}

impl PhaseProgress {
    /// Creates a new progress bar for a phase.
    ///
    /// # Arguments
    ///
    /// * `total` - Total number of items to process
    /// * `phase_name` - Human-readable phase name (e.g., "Phase 1: Discovery")
    fn new(total: usize, phase_name: &str) -> Self {
        let bar = ProgressBar::new(total as u64);

        // Set style with smooth animation and comprehensive metrics
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{msg}\n[{bar:40.cyan/blue}] {pos}/{len} | {elapsed} | {per_sec} | ETA: {eta}",
                )
                .expect("Invalid progress bar template")
                .progress_chars("█▓▒░"),
        );

        bar.set_message(phase_name.to_string());

        Self {
            bar,
            phase_name: phase_name.to_string(),
            start_time: Instant::now(),
        }
    }

    /// Increments the progress bar by 1.
    ///
    /// Thread-safe - can be called from multiple threads.
    fn inc(&self) {
        self.bar.inc(1);
    }

    /// Completes the progress bar with a success message.
    fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        self.bar.finish_with_message(format!(
            "✓ {} complete ({:.1}s)",
            self.phase_name,
            elapsed.as_secs_f64()
        ));
    }
}

/// Returns the path to the checkpoint file for trend analysis.
///
/// Uses platform-appropriate cache directory (e.g., ~/.cache/ponder on Linux,
/// ~/Library/Caches/ponder on macOS).
///
/// # Returns
///
/// Returns the checkpoint file path, creating parent directory if needed.
///
/// # Errors
///
/// Returns an error if the cache directory cannot be determined or created.
fn get_checkpoint_path() -> AppResult<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| AppError::Journal("Could not determine cache directory".to_string()))?;

    let ponder_cache = cache_dir.join("ponder");
    fs::create_dir_all(&ponder_cache)?;

    Ok(ponder_cache.join("trends-checkpoint.json"))
}

/// Saves a checkpoint to disk with atomic writes.
///
/// Writes to a temporary file first, then renames to ensure atomicity.
/// This prevents corrupted checkpoints if the process is killed mid-write.
///
/// # Arguments
///
/// * `checkpoint` - The checkpoint to save
///
/// # Errors
///
/// Returns an error if serialization or file operations fail.
fn save_checkpoint(checkpoint: &TrendsCheckpoint) -> AppResult<()> {
    let checkpoint_path = get_checkpoint_path()?;
    let temp_path = checkpoint_path.with_extension("json.tmp");

    // Serialize to pretty JSON for human readability
    let json = serde_json::to_string_pretty(checkpoint)
        .map_err(|e| AppError::Journal(format!("Failed to serialize checkpoint: {}", e)))?;

    // Write to temp file
    fs::write(&temp_path, json)?;

    // Atomic rename
    fs::rename(&temp_path, &checkpoint_path)?;

    debug!("Checkpoint saved to {:?}", checkpoint_path);
    Ok(())
}

/// Loads a checkpoint from disk, validating it matches the query.
///
/// Returns `None` if:
/// - No checkpoint file exists
/// - Checkpoint file is corrupted (invalid JSON)
/// - Checkpoint query doesn't match the provided query
///
/// # Arguments
///
/// * `query` - The trend analysis query to match against
///
/// # Returns
///
/// Returns `Some(checkpoint)` if valid checkpoint exists, `None` otherwise.
///
/// # Errors
///
/// Returns an error only for unexpected I/O failures (not for missing/invalid checkpoints).
fn load_checkpoint(query: &str) -> AppResult<Option<TrendsCheckpoint>> {
    let checkpoint_path = get_checkpoint_path()?;

    // No checkpoint file = None (not an error)
    if !checkpoint_path.exists() {
        return Ok(None);
    }

    // Read and parse checkpoint
    let json = match fs::read_to_string(&checkpoint_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    let checkpoint: TrendsCheckpoint = match serde_json::from_str(&json) {
        Ok(cp) => cp,
        Err(e) => {
            warn!("Corrupted checkpoint file, ignoring: {}", e);
            return Ok(None);
        }
    };

    // Validate query matches
    if checkpoint.query != query {
        debug!(
            "Checkpoint query mismatch ('{}' != '{}'), ignoring",
            checkpoint.query, query
        );
        return Ok(None);
    }

    debug!(
        "Loaded checkpoint: {} phase, {} entries processed",
        if checkpoint.phase == CheckpointPhase::Discovery {
            "Discovery"
        } else {
            "Extraction"
        },
        checkpoint.processed_entry_ids.len()
    );

    Ok(Some(checkpoint))
}

/// Deletes the checkpoint file if it exists.
///
/// Ignores errors if the file doesn't exist (already deleted is success).
///
/// # Errors
///
/// Returns an error only for unexpected I/O failures.
fn delete_checkpoint() -> AppResult<()> {
    let checkpoint_path = get_checkpoint_path()?;

    match fs::remove_file(&checkpoint_path) {
        Ok(_) => {
            debug!("Checkpoint deleted");
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Already deleted = success
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Analyzes trends across all user journal entries.
///
/// This is the main entry point for comprehensive trend analysis. It orchestrates
/// the entire pipeline: filtering, relevance checking, insight extraction, synthesis,
/// and report persistence.
///
/// # Pipeline Flow
///
/// 1. Filter user entries (exclude AI-generated content)
/// 2. Check each entry for relevance to the query (Phase 1)
/// 3. Extract insights from relevant entries (Phase 2)
/// 4. Synthesize comprehensive report (Phase 3)
/// 5. Encrypt and save report with metadata
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for decryption
/// * `ai_client` - Ollama client for LLM operations
/// * `query` - Natural language query describing the trend (e.g., "productivity patterns")
/// * `output_path` - Optional custom save location (defaults to `reports/trends/trends-{timestamp}.md.age`)
///
/// # Returns
///
/// Returns `TrendReport` with metadata about the generated report.
///
/// # Errors
///
/// Returns an error if:
/// - Session is locked
/// - Database query fails
/// - LLM API fails
/// - File encryption fails
/// - Report persistence fails
pub fn analyze_trends(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    query: &str,
    output_path: Option<PathBuf>,
) -> AppResult<TrendReport> {
    info!("Starting comprehensive trend analysis for query: {}", query);

    // Ensure session is unlocked
    let _passphrase = session.get_passphrase()?;

    // Check for existing checkpoint
    let mut checkpoint = load_checkpoint(query)?;

    if let Some(ref cp) = checkpoint {
        println!("\n📂 Resuming from previous run:");
        println!(
            "   {} entries completed ({:.1}% of {})",
            cp.processed_entry_ids.len(),
            (cp.processed_entry_ids.len() as f64 / cp.total_entries as f64) * 100.0,
            cp.total_entries
        );
        println!(
            "   Phase: {}",
            if cp.phase == CheckpointPhase::Discovery {
                "Discovery (checking relevance)"
            } else {
                "Extraction (extracting insights)"
            }
        );
        println!();
    }

    // Phase 0: Filter user entries
    info!("Filtering user-created entries...");
    let mut user_entries = filter_user_entries(db)?;
    let total_entries = user_entries.len();

    if total_entries == 0 {
        return Err(AppError::Journal(
            "No user entries found. Have you created any journal entries?".to_string(),
        ));
    }

    // If resuming Discovery phase, filter out already-processed entries
    if let Some(ref cp) = checkpoint {
        if cp.phase == CheckpointPhase::Discovery {
            let original_count = user_entries.len();
            user_entries.retain(|e| !cp.processed_entry_ids.contains(&e.id));
            info!(
                "Resuming Discovery: {} entries remaining ({} already processed)",
                user_entries.len(),
                original_count - user_entries.len()
            );
        }
    }

    info!("Found {} user-created entries to analyze", total_entries);

    // Phase 1: Discover relevant entries
    info!("Phase 1: Checking relevance of each entry...");
    let mut relevant_entry_ids = discover_relevant_entries(
        &user_entries,
        query,
        ai_client,
        session,
        &mut checkpoint,
        total_entries,
    )?;

    // Merge with checkpoint's relevant IDs if resuming
    if let Some(ref cp) = checkpoint {
        if cp.phase == CheckpointPhase::Discovery {
            // Combine new discoveries with previously found relevant entries
            for id in &cp.relevant_entry_ids {
                if !relevant_entry_ids.contains(id) {
                    relevant_entry_ids.push(*id);
                }
            }
            info!(
                "Merged {} previously discovered relevant entries",
                cp.relevant_entry_ids.len()
            );
        }
    }

    let relevant_count = relevant_entry_ids.len();
    info!(
        "Found {}/{} entries relevant to query",
        relevant_count, total_entries
    );

    if relevant_count == 0 {
        warn!("No relevant entries found for query: {}", query);
        return Err(AppError::Journal(format!(
            "No entries found relevant to '{}'. Try a different query or broader theme.",
            query
        )));
    }

    // Get full entry objects for relevant IDs (need to query all user_entries, not filtered ones)
    let all_user_entries = filter_user_entries(db)?;
    let relevant_entries: Vec<&Entry> = all_user_entries
        .iter()
        .filter(|e| relevant_entry_ids.contains(&e.id))
        .collect();

    // Phase 2: Extract insights from relevant entries
    info!("Phase 2: Extracting insights from relevant entries...");

    // Check if resuming from Extraction phase
    let mut insights = if let Some(ref cp) = checkpoint {
        if cp.phase == CheckpointPhase::Extraction {
            // Convert SerializableInsight → EntryInsight
            let loaded_insights: Vec<EntryInsight> = cp
                .extracted_insights
                .iter()
                .map(|si| EntryInsight {
                    date: NaiveDate::parse_from_str(&si.date, "%Y-%m-%d")
                        .expect("Invalid date in checkpoint"),
                    excerpts: si.excerpts.clone(),
                    summary: si.summary.clone(),
                })
                .collect();

            info!(
                "Resuming Extraction: {} insights already extracted",
                loaded_insights.len()
            );
            loaded_insights
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Filter relevant_entries to skip those already extracted
    let already_extracted_dates: HashSet<NaiveDate> = insights.iter().map(|i| i.date).collect();
    let entries_to_extract: Vec<&Entry> = relevant_entries
        .into_iter()
        .filter(|e| !already_extracted_dates.contains(&e.date))
        .collect();

    if !entries_to_extract.is_empty() {
        // Update checkpoint to Extraction phase if in Discovery
        if let Some(ref mut cp) = checkpoint {
            if cp.phase == CheckpointPhase::Discovery {
                cp.phase = CheckpointPhase::Extraction;
                save_checkpoint(cp)?;
                info!("Checkpoint updated to Extraction phase");
            }
        }

        let new_insights = extract_insights(
            &entries_to_extract,
            query,
            ai_client,
            session,
            &mut checkpoint,
        )?;
        insights.extend(new_insights);
    }

    info!("Extracted {} insights total", insights.len());

    // Phase 3: Synthesize comprehensive report
    info!("Phase 3: Synthesizing comprehensive report...");
    let report_content = synthesize_report(&insights, query, ai_client)?;
    info!("Generated report ({} chars)", report_content.len());

    // Phase 4: Persist report
    info!("Persisting encrypted report...");
    let report_path = persist_report(
        &report_content,
        query,
        output_path,
        total_entries,
        relevant_count,
        session,
        db,
    )?;

    info!(
        "Trend analysis complete! Report saved to: {:?}",
        report_path
    );

    // Get the report ID from the database (it was just inserted by persist_report)
    let report_id = get_latest_report_id(db)?;

    // Delete checkpoint on successful completion
    delete_checkpoint()?;
    debug!("Checkpoint deleted after successful completion");

    Ok(TrendReport {
        path: report_path,
        query: query.to_string(),
        total_entries,
        relevant_entries: relevant_count,
        report_id,
    })
}

/// Filters database entries to only include user-created content.
///
/// Excludes AI-generated content like reports, summaries, and patterns by checking
/// path patterns. User entries follow the `YYYY/MM/DD.md.age` pattern, while AI
/// content lives in `reports/*` and `summaries/*` directories.
///
/// # Arguments
///
/// * `db` - Database connection
///
/// # Returns
///
/// Returns a vector of user-created entries.
///
/// # Errors
///
/// Returns an error if the database query fails.
fn filter_user_entries(db: &Database) -> AppResult<Vec<Entry>> {
    let conn = db.get_conn()?;

    // Query all entries
    let mut stmt = conn
        .prepare(
            r#"
        SELECT id, path, date, checksum, word_count, updated_at, embedded_at
        FROM entries
        ORDER BY date ASC
        "#,
        )
        .map_err(|e| crate::errors::DatabaseError::Sqlite(e))?;

    let entries = stmt
        .query_map([], |row| {
            Ok(Entry {
                id: row.get(0)?,
                path: PathBuf::from(row.get::<_, String>(1)?),
                date: NaiveDate::parse_from_str(&row.get::<_, String>(2)?, "%Y-%m-%d").map_err(
                    |e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    },
                )?,
                checksum: row.get(3)?,
                word_count: row.get::<_, i64>(4)? as usize,
                updated_at: row.get(5)?,
                embedded_at: row.get(6)?,
            })
        })
        .map_err(|e| crate::errors::DatabaseError::Sqlite(e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| crate::errors::DatabaseError::Sqlite(e))?;

    // Filter to only user entries (exclude AI-generated content)
    let user_entries: Vec<Entry> = entries
        .into_iter()
        .filter(|entry| {
            let path_str = entry.path.to_string_lossy();
            // Exclude paths containing "reports" or "summaries" as directory components
            // This works with absolute paths like /Users/user/.ponder/reports/...
            !path_str.contains("/reports/") && !path_str.contains("/summaries/")
        })
        .collect();

    debug!(
        "Filtered to {} user entries (excluded AI-generated content)",
        user_entries.len()
    );

    Ok(user_entries)
}

/// Discovers which entries are relevant to the user's query.
///
/// Phase 1 of the pipeline: For each entry, decrypt and ask a fast LLM:
/// "Does this entry contain information relevant to the query?"
///
/// This phase filters down the entry set to only those worth deep analysis.
/// Saves checkpoint every 100 entries to enable resumption.
///
/// # Arguments
///
/// * `entries` - All user entries to check
/// * `query` - User's trend query
/// * `ai_client` - Ollama client for LLM calls
/// * `session` - Session manager for decryption
/// * `checkpoint` - Optional checkpoint to update with progress
/// * `total_entries` - Total number of entries in the full dataset (for checkpoint)
///
/// # Returns
///
/// Returns a vector of entry IDs deemed relevant.
///
/// # Errors
///
/// Returns an error if decryption or LLM calls fail.
fn discover_relevant_entries(
    entries: &[Entry],
    query: &str,
    ai_client: &OllamaClient,
    session: &mut SessionManager,
    checkpoint: &mut Option<TrendsCheckpoint>,
    total_entries: usize,
) -> AppResult<Vec<i64>> {
    let passphrase = session.get_passphrase()?;
    let mut relevant_ids = Vec::new();

    // Initialize progress bar for Phase 1
    let progress = PhaseProgress::new(entries.len(), "Phase 1: Checking relevance");

    for (idx, entry) in entries.iter().enumerate() {
        // Decrypt entry
        let temp_path = decrypt_to_temp(&entry.path, passphrase)?;
        let content = fs::read_to_string(&temp_path)?;
        crate::crypto::temp::secure_delete(&temp_path)?;

        // Check relevance with LLM
        let is_relevant = check_relevance(&content, query, &entry.date, ai_client)?;

        if is_relevant {
            debug!("Entry {} ({}) is relevant", entry.id, entry.date);
            relevant_ids.push(entry.id);
        }

        // Update progress after each entry
        progress.inc();

        // Update checkpoint every 100 entries
        if let Some(ref mut cp) = checkpoint {
            cp.processed_entry_ids.insert(entry.id);
            if is_relevant {
                cp.relevant_entry_ids.push(entry.id);
            }

            if (idx + 1) % 100 == 0 {
                save_checkpoint(cp)?;
                debug!("Checkpoint saved at {} entries processed", idx + 1);
            }
        } else {
            // First run - create checkpoint after first entry
            if idx == 0 {
                let mut new_checkpoint = TrendsCheckpoint {
                    query: query.to_string(),
                    started_at: chrono::Local::now().to_rfc3339(),
                    phase: CheckpointPhase::Discovery,
                    total_entries,
                    processed_entry_ids: HashSet::new(),
                    relevant_entry_ids: Vec::new(),
                    extracted_insights: Vec::new(),
                };
                new_checkpoint.processed_entry_ids.insert(entry.id);
                if is_relevant {
                    new_checkpoint.relevant_entry_ids.push(entry.id);
                }
                *checkpoint = Some(new_checkpoint);
            }
        }
    }

    // Final checkpoint save
    if let Some(ref cp) = checkpoint {
        save_checkpoint(cp)?;
        debug!("Final Discovery checkpoint saved");
    }

    progress.finish();
    Ok(relevant_ids)
}

/// Checks if a single entry is relevant to the query.
///
/// Uses a fast LLM with a carefully crafted prompt to determine relevance.
/// The prompt asks for a yes/no answer with brief reasoning to ensure quality.
///
/// # Arguments
///
/// * `content` - Decrypted entry content
/// * `query` - User's trend query
/// * `date` - Entry date (for context)
/// * `ai_client` - Ollama client
///
/// # Returns
///
/// Returns `true` if the entry is relevant, `false` otherwise.
///
/// # Errors
///
/// Returns an error if the LLM call fails.
fn check_relevance(
    content: &str,
    query: &str,
    date: &NaiveDate,
    ai_client: &OllamaClient,
) -> AppResult<bool> {
    // Truncate very long entries to avoid token limits
    let truncated_content = if content.len() > 4000 {
        &content[..4000]
    } else {
        content
    };

    let prompt = format!(
        r#"You are analyzing a journal entry from {date} to determine relevance to a trend analysis query.

QUERY: "{query}"

ENTRY CONTENT:
{content}

TASK: Determine if this entry contains ANY information relevant to the query. Be inclusive - if there's even a tangential connection, answer YES.

Respond in this exact format:
RELEVANCE: YES or NO
REASON: (one sentence explaining your decision)

Your response:"#,
        date = date.format("%Y-%m-%d"),
        query = query,
        content = truncated_content
    );

    let messages = vec![crate::ai::Message {
        role: "user".to_string(),
        content: prompt,
    }];

    let response = ai_client.chat_with_retry(DEFAULT_CHAT_MODEL, &messages, 3)?;

    // Parse response - look for "YES" (case-insensitive, flexible)
    // Accept "RELEVANCE: YES", "YES", "Yes", "Relevant: Yes", etc.
    let upper_response = response.to_uppercase();
    let is_relevant = upper_response.contains("RELEVANCE: YES")
        || upper_response.contains("RELEVANT: YES")
        || (upper_response.contains("YES") && !upper_response.contains("NO"));

    if is_relevant {
        debug!("Entry from {} is relevant: {}", date, response.trim());
    }

    Ok(is_relevant)
}

/// Extracts insights from relevant entries.
///
/// Phase 2 of the pipeline: For each relevant entry, extract:
/// - Specific excerpts/quotes related to the query
/// - A summary of the insight
///
/// Saves checkpoint every 50 entries to enable resumption.
///
/// # Arguments
///
/// * `entries` - Relevant entries to analyze
/// * `query` - User's trend query
/// * `ai_client` - Ollama client
/// * `session` - Session manager for decryption
/// * `checkpoint` - Optional checkpoint to update with progress
///
/// # Returns
///
/// Returns a vector of `EntryInsight` structs.
///
/// # Errors
///
/// Returns an error if decryption or LLM calls fail.
fn extract_insights(
    entries: &[&Entry],
    query: &str,
    ai_client: &OllamaClient,
    session: &mut SessionManager,
    checkpoint: &mut Option<TrendsCheckpoint>,
) -> AppResult<Vec<EntryInsight>> {
    let passphrase = session.get_passphrase()?;
    let mut insights = Vec::new();

    // Initialize progress bar for Phase 2
    let progress = PhaseProgress::new(entries.len(), "Phase 2: Extracting insights");

    for (idx, entry) in entries.iter().enumerate() {
        // Decrypt entry
        let temp_path = decrypt_to_temp(&entry.path, passphrase)?;
        let content = fs::read_to_string(&temp_path)?;
        crate::crypto::temp::secure_delete(&temp_path)?;

        // Extract insight with LLM
        let insight = extract_single_insight(&content, query, &entry.date, ai_client)?;
        insights.push(insight.clone());

        // Update progress after each entry
        progress.inc();

        // Update checkpoint every 50 entries
        if let Some(ref mut cp) = checkpoint {
            // Convert EntryInsight → SerializableInsight
            let serializable_insight = SerializableInsight {
                date: insight.date.format("%Y-%m-%d").to_string(),
                excerpts: insight.excerpts.clone(),
                summary: insight.summary.clone(),
            };
            cp.extracted_insights.push(serializable_insight);

            if (idx + 1) % 50 == 0 {
                save_checkpoint(cp)?;
                debug!("Checkpoint saved at {} insights extracted", idx + 1);
            }
        }
    }

    // Final checkpoint save
    if let Some(ref cp) = checkpoint {
        save_checkpoint(cp)?;
        debug!("Final Extraction checkpoint saved");
    }

    progress.finish();
    Ok(insights)
}

/// Extracts insight from a single entry.
///
/// Uses LLM to identify relevant excerpts and summarize the key insight.
///
/// # Arguments
///
/// * `content` - Decrypted entry content
/// * `query` - User's trend query
/// * `date` - Entry date
/// * `ai_client` - Ollama client
///
/// # Returns
///
/// Returns an `EntryInsight` with excerpts and summary.
///
/// # Errors
///
/// Returns an error if the LLM call fails.
fn extract_single_insight(
    content: &str,
    query: &str,
    date: &NaiveDate,
    ai_client: &OllamaClient,
) -> AppResult<EntryInsight> {
    let prompt = format!(
        r#"You are extracting insights from a journal entry for trend analysis.

QUERY: "{query}"
ENTRY DATE: {date}

ENTRY CONTENT:
{content}

TASK: Extract 1-3 relevant excerpts (direct quotes) and provide a one-sentence summary of the key insight.

Respond in this exact format:
EXCERPT 1: "direct quote from entry"
EXCERPT 2: "another quote" (if applicable)
EXCERPT 3: "third quote" (if applicable)
INSIGHT: One-sentence summary of what this reveals about the trend

Your response:"#,
        query = query,
        date = date.format("%Y-%m-%d"),
        content = content
    );

    let messages = vec![crate::ai::Message {
        role: "user".to_string(),
        content: prompt,
    }];

    let response = ai_client.chat_with_retry(DEFAULT_CHAT_MODEL, &messages, 3)?;

    // Parse excerpts and insight from response
    let excerpts = parse_excerpts(&response);
    let summary = parse_insight(&response);

    Ok(EntryInsight {
        date: *date,
        excerpts,
        summary,
    })
}

/// Parses excerpts from LLM response.
fn parse_excerpts(response: &str) -> Vec<String> {
    response
        .lines()
        .filter(|line| line.starts_with("EXCERPT"))
        .map(|line| {
            // Extract text between quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return line[start + 1..start + 1 + end].to_string();
                }
            }
            line.to_string()
        })
        .collect()
}

/// Parses insight summary from LLM response.
fn parse_insight(response: &str) -> String {
    response
        .lines()
        .find(|line| line.starts_with("INSIGHT:"))
        .and_then(|line| line.strip_prefix("INSIGHT:"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "No insight extracted".to_string())
}

/// Synthesizes a comprehensive report from extracted insights.
///
/// Phase 3 of the pipeline: Takes all extracted insights and generates a
/// comprehensive trend analysis report with:
/// - Executive summary (3-5 key findings)
/// - Detailed analysis organized thematically or chronologically
/// - Specific citations with dates
///
/// # Arguments
///
/// * `insights` - All extracted insights from relevant entries
/// * `query` - User's trend query
/// * `ai_client` - Ollama client
///
/// # Returns
///
/// Returns the report content as a formatted Markdown string.
///
/// # Errors
///
/// Returns an error if the LLM call fails.
fn synthesize_report(
    insights: &[EntryInsight],
    query: &str,
    ai_client: &OllamaClient,
) -> AppResult<String> {
    // Limit insights to avoid context window overflow
    // gemma3:4b has ~8k token context, so limit to 50 insights
    const MAX_INSIGHTS: usize = 50;

    let limited_insights = if insights.len() > MAX_INSIGHTS {
        warn!(
            "Limiting synthesis to top {} insights (out of {}) to avoid context window overflow",
            MAX_INSIGHTS,
            insights.len()
        );
        &insights[..MAX_INSIGHTS]
    } else {
        insights
    };

    // Build context from insights
    let mut context = String::new();
    for insight in limited_insights {
        context.push_str(&format!("\n[{}]\n", insight.date.format("%Y-%m-%d")));
        for excerpt in &insight.excerpts {
            context.push_str(&format!("- \"{}\"\n", excerpt));
        }
        context.push_str(&format!("Insight: {}\n", insight.summary));
    }

    let note = if insights.len() > MAX_INSIGHTS {
        format!(
            "Note: Analyzing top {} most relevant entries out of {} total.",
            MAX_INSIGHTS,
            insights.len()
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are a professional trend analyst generating a comprehensive report from journal entry analysis.

QUERY: "{query}"

EXTRACTED INSIGHTS (from {count} entries):
{note}
{context}

TASK: Generate a comprehensive trend analysis report with the following structure:

# Trend Analysis: {query}

## Executive Summary
(3-5 bullet points highlighting the most important findings)

## Detailed Analysis
(Organize by themes or chronologically. Include specific dated citations.)

## Patterns Observed
(What recurring themes or patterns emerge?)

## Notable Shifts
(How did things change over time?)

## Conclusion
(Overall takeaway and implications)

Write in a clear, analytical style. Use specific dates and quotes to support your analysis.

Your report:"#,
        query = query,
        count = limited_insights.len(),
        note = note,
        context = context
    );

    let messages = vec![crate::ai::Message {
        role: "user".to_string(),
        content: prompt,
    }];

    // Create spinner for synthesis phase (single operation, unknown duration)
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{msg} {spinner}")
            .expect("Invalid spinner template"),
    );
    spinner.set_message("Phase 3: Synthesizing comprehensive report...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let report = ai_client.chat_with_retry(DEFAULT_CHAT_MODEL, &messages, 3)?;

    spinner.finish_with_message("✓ Phase 3: Synthesis complete");

    Ok(report)
}

/// Persists the trend report as an encrypted file and records metadata.
///
/// Encrypts the report content, saves to disk, and records metadata in the
/// `reports` table for future reference.
///
/// # Arguments
///
/// * `content` - Report content (Markdown)
/// * `query` - User's trend query
/// * `output_path` - Optional custom save location
/// * `total_entries` - Total entries analyzed
/// * `relevant_count` - Number of relevant entries
/// * `session` - Session manager (for passphrase)
/// * `db` - Database connection
///
/// # Returns
///
/// Returns the path to the saved encrypted report.
///
/// # Errors
///
/// Returns an error if encryption or database operations fail.
fn persist_report(
    content: &str,
    query: &str,
    output_path: Option<PathBuf>,
    total_entries: usize,
    relevant_count: usize,
    session: &mut SessionManager,
    db: &Database,
) -> AppResult<PathBuf> {
    let passphrase = session.get_passphrase()?;

    // Determine output path
    let report_path = if let Some(path) = output_path {
        path
    } else {
        // Default: reports/trends/trends-{timestamp}-{slug}.md.age
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let slug = query
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .take(30)
            .collect::<String>()
            .replace(' ', "-")
            .to_lowercase();
        PathBuf::from(format!(
            "reports/trends/trends-{}-{}.md.age",
            timestamp, slug
        ))
    };

    // Ensure parent directory exists
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Encrypt and save report
    let encrypted_bytes = encrypt_with_passphrase(content.as_bytes(), passphrase)?;
    fs::write(&report_path, encrypted_bytes)?;
    info!("Report encrypted and saved to: {:?}", report_path);

    // Record in database
    let conn = db.get_conn()?;
    conn.execute(
        r#"
        INSERT INTO reports (path, type, date_range)
        VALUES (?1, ?2, ?3)
        "#,
        rusqlite::params![
            report_path.to_string_lossy(),
            "trend_analysis",
            format!(
                "Analyzed {} entries ({} relevant) for: {}",
                total_entries, relevant_count, query
            )
        ],
    )
    .map_err(|e| crate::errors::DatabaseError::Sqlite(e))?;

    debug!("Report metadata recorded in database");

    Ok(report_path)
}

/// Gets the ID of the most recently inserted report.
fn get_latest_report_id(db: &Database) -> AppResult<i64> {
    let conn = db.get_conn()?;
    let id = conn
        .query_row(
            "SELECT id FROM reports ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| crate::errors::DatabaseError::Sqlite(e))?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_excerpts() {
        let response = r#"EXCERPT 1: "This is the first quote"
EXCERPT 2: "Second quote here"
INSIGHT: This reveals a pattern"#;

        let excerpts = parse_excerpts(response);
        assert_eq!(excerpts.len(), 2);
        assert_eq!(excerpts[0], "This is the first quote");
        assert_eq!(excerpts[1], "Second quote here");
    }

    #[test]
    fn test_parse_insight() {
        let response = r#"EXCERPT 1: "quote"
INSIGHT: This is the key finding"#;

        let insight = parse_insight(response);
        assert_eq!(insight, "This is the key finding");
    }

    #[test]
    fn test_parse_insight_missing() {
        let response = "EXCERPT 1: \"quote\"";
        let insight = parse_insight(response);
        assert_eq!(insight, "No insight extracted");
    }
}
