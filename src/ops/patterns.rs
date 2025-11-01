//! Pattern detection operations for journal entries.
//!
//! This module provides functions for detecting patterns in journal writing behavior:
//! - Temporal patterns (day of week preferences, writing frequency, gaps)
//! - Topic clustering (coming soon)
//! - Sentiment trends (coming soon)
//! - Correlation discovery (coming soon)

use crate::ai::OllamaClient;
use crate::crypto::age::decrypt_with_passphrase;
use crate::crypto::SessionManager;
use crate::db::embeddings::search_similar_chunks;
use crate::db::patterns::{insert_pattern, PatternType};
use crate::db::Database;
use crate::errors::{AppResult, DatabaseError};
use bytemuck;
use chrono::{Datelike, NaiveDate, Weekday};
use rusqlite::params;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs;
use tracing::{debug, info, warn};

/// Result of temporal pattern detection.
#[derive(Debug, Clone)]
pub struct TemporalPatterns {
    /// Detected patterns with descriptions
    pub patterns: Vec<String>,
    /// Day of week distribution (0=Sunday, 6=Saturday)
    pub day_distribution: HashMap<Weekday, usize>,
    /// Average gap between entries in days
    pub avg_gap_days: f64,
    /// Longest gap between entries in days
    pub longest_gap_days: i64,
    /// Total number of entries analyzed
    pub total_entries: usize,
}

/// Detects temporal patterns in journal writing behavior.
///
/// Analyzes when and how frequently the user writes in their journal to identify
/// patterns such as preferred days of the week, writing streaks, and gaps.
///
/// # Flow
///
/// 1. Query all entries from database ordered by date
/// 2. Calculate day-of-week distribution
/// 3. Calculate gaps between entries
/// 4. Identify patterns (preferred days, average frequency, notable gaps)
/// 5. Store patterns in database
/// 6. Return pattern summary
///
/// # Arguments
///
/// * `db` - Database connection
///
/// # Returns
///
/// Returns `TemporalPatterns` with detected patterns and statistics.
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - Pattern storage fails
pub fn detect_temporal_patterns(db: &Database) -> AppResult<TemporalPatterns> {
    info!("Detecting temporal patterns in journal entries");

    let conn = db.get_conn()?;

    // Query all entries ordered by date
    let mut stmt = conn
        .prepare(
            r#"
        SELECT date, word_count, updated_at
        FROM entries
        ORDER BY date ASC
        "#,
        )
        .map_err(DatabaseError::Sqlite)?;

    let entries: Vec<(String, i64, String)> = stmt
        .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    let total_entries = entries.len();

    if total_entries == 0 {
        info!("No entries found for pattern detection");
        return Ok(TemporalPatterns {
            patterns: vec!["No entries found - cannot detect patterns".to_string()],
            day_distribution: HashMap::new(),
            avg_gap_days: 0.0,
            longest_gap_days: 0,
            total_entries: 0,
        });
    }

    debug!("Analyzing {} entries for temporal patterns", total_entries);

    // Convert date strings to NaiveDate
    let dates: Vec<NaiveDate> = entries
        .iter()
        .filter_map(|(date_str, _, _)| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
        .collect();

    // Calculate day-of-week distribution
    let mut day_distribution: HashMap<Weekday, usize> = HashMap::new();
    for date in &dates {
        *day_distribution.entry(date.weekday()).or_insert(0) += 1;
    }

    debug!("Day distribution: {:?}", day_distribution);

    // Calculate gaps between entries
    let mut gaps: Vec<i64> = Vec::new();
    for window in dates.windows(2) {
        let gap = window[1].signed_duration_since(window[0]).num_days();
        gaps.push(gap);
    }

    let avg_gap_days = if !gaps.is_empty() {
        gaps.iter().sum::<i64>() as f64 / gaps.len() as f64
    } else {
        0.0
    };

    let longest_gap_days = gaps.iter().max().copied().unwrap_or(0);

    debug!(
        "Average gap: {:.1} days, Longest gap: {} days",
        avg_gap_days, longest_gap_days
    );

    // Identify patterns
    let mut patterns = Vec::new();

    // Pattern 1: Preferred day of week
    if let Some((preferred_day, &count)) = day_distribution.iter().max_by_key(|(_, &v)| v) {
        let percentage = (count as f64 / total_entries as f64) * 100.0;
        if percentage > 30.0 {
            patterns.push(format!(
                "You write most often on {}s ({:.0}% of entries)",
                preferred_day, percentage
            ));

            // Store in database
            let metadata = json!({
                "day": preferred_day.to_string(),
                "count": count,
                "percentage": percentage
            });
            let first_date = dates.first().map(|d| d.to_string()).unwrap_or_default();
            let last_date = dates.last().map(|d| d.to_string()).unwrap_or_default();
            insert_pattern(
                &conn,
                PatternType::Temporal,
                &format!("Preferred day: {}", preferred_day),
                Some(&metadata.to_string()),
                Some(percentage / 100.0),
                &first_date,
                &last_date,
            )?;
        }
    }

    // Pattern 2: Writing frequency
    if avg_gap_days < 2.0 {
        patterns.push(format!(
            "You write frequently - average {:.1} days between entries",
            avg_gap_days
        ));
    } else if avg_gap_days > 7.0 {
        patterns.push(format!(
            "You write sporadically - average {:.1} days between entries",
            avg_gap_days
        ));
    } else {
        patterns.push(format!(
            "You write regularly - average {:.1} days between entries",
            avg_gap_days
        ));
    }

    // Store frequency pattern
    let freq_metadata = json!({
        "avg_gap_days": avg_gap_days,
        "longest_gap_days": longest_gap_days,
        "total_entries": total_entries
    });
    let first_date = dates.first().map(|d| d.to_string()).unwrap_or_default();
    let last_date = dates.last().map(|d| d.to_string()).unwrap_or_default();
    insert_pattern(
        &conn,
        PatternType::Temporal,
        &format!("Writing frequency: {:.1} day average gap", avg_gap_days),
        Some(&freq_metadata.to_string()),
        None,
        &first_date,
        &last_date,
    )?;

    // Pattern 3: Notable gaps
    if longest_gap_days > 30 {
        patterns.push(format!(
            "Longest break from journaling: {} days",
            longest_gap_days
        ));

        let gap_metadata = json!({
            "longest_gap_days": longest_gap_days
        });
        insert_pattern(
            &conn,
            PatternType::Temporal,
            &format!("Longest gap: {} days", longest_gap_days),
            Some(&gap_metadata.to_string()),
            None,
            &first_date,
            &last_date,
        )?;
    }

    // Pattern 4: Weekend vs weekday writing
    let weekend_count = day_distribution.get(&Weekday::Sat).unwrap_or(&0)
        + day_distribution.get(&Weekday::Sun).unwrap_or(&0);
    let weekday_count = total_entries - weekend_count;

    if weekend_count > weekday_count {
        let percentage = (weekend_count as f64 / total_entries as f64) * 100.0;
        patterns.push(format!(
            "Weekend writer - {:.0}% of entries on weekends",
            percentage
        ));
    } else if weekday_count > weekend_count * 2 {
        let percentage = (weekday_count as f64 / total_entries as f64) * 100.0;
        patterns.push(format!(
            "Weekday writer - {:.0}% of entries on weekdays",
            percentage
        ));
    }

    info!("Detected {} temporal patterns", patterns.len());

    Ok(TemporalPatterns {
        patterns,
        day_distribution,
        avg_gap_days,
        longest_gap_days,
        total_entries,
    })
}

/// Topic cluster result.
#[derive(Debug, Clone)]
pub struct TopicCluster {
    /// Main topic label
    pub topic: String,
    /// Entry dates in this cluster
    pub entry_dates: Vec<NaiveDate>,
    /// Confidence score
    pub confidence: f64,
}

/// Result of topic pattern detection.
#[derive(Debug, Clone)]
pub struct TopicPatterns {
    /// Detected topic clusters
    pub clusters: Vec<TopicCluster>,
    /// Total entries analyzed
    pub total_entries: usize,
}

/// Detects topic patterns by clustering similar journal entries.
///
/// Uses vector embeddings to group entries by semantic similarity, then uses AI
/// to extract and label the dominant topics in each cluster.
///
/// # Flow
///
/// 1. Query all entries with embeddings
/// 2. For each entry, find similar entries using embedding similarity
/// 3. Group entries into clusters based on similarity threshold
/// 4. For each cluster, decrypt entries and extract topics using AI
/// 5. Store topic patterns in database
/// 6. Return topic clusters
///
/// # Arguments
///
/// * `db` - Database connection
/// * `session` - Session manager for decryption
/// * `ai_client` - Ollama client for topic extraction
/// * `similarity_threshold` - Minimum similarity score to group entries (0.0-1.0)
/// * `min_cluster_size` - Minimum number of entries to form a cluster
///
/// # Returns
///
/// Returns `TopicPatterns` with detected clusters and their topics.
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - Decryption fails
/// - AI topic extraction fails
pub fn detect_topic_patterns(
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    similarity_threshold: f32,
    min_cluster_size: usize,
) -> AppResult<TopicPatterns> {
    info!("Detecting topic patterns via embedding clustering");

    let conn = db.get_conn()?;
    let passphrase = session.get_passphrase()?;

    // Get all entries that have embeddings
    let mut stmt = conn
        .prepare(
            r#"
        SELECT DISTINCT e.id, e.date, e.path
        FROM entries e
        INNER JOIN embeddings emb ON e.id = emb.entry_id
        ORDER BY e.date DESC
        "#,
        )
        .map_err(DatabaseError::Sqlite)?;

    let entries: Vec<(i64, String, String)> = stmt
        .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    let total_entries = entries.len();

    if total_entries == 0 {
        info!("No entries with embeddings found for topic clustering");
        return Ok(TopicPatterns {
            clusters: vec![],
            total_entries: 0,
        });
    }

    debug!("Analyzing {} entries for topic clustering", total_entries);

    // Simple clustering: for each entry, find similar entries
    // Group entries that are mutually similar
    let mut clusters: Vec<Vec<(i64, NaiveDate, String)>> = Vec::new();
    let mut processed: HashSet<i64> = HashSet::new();

    for (entry_id, date_str, path_str) in &entries {
        if processed.contains(entry_id) {
            continue;
        }

        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| DatabaseError::Custom(format!("Invalid date format: {}", e)))?;

        // Get this entry's first embedding as the representative
        let embedding_result: Result<Vec<u8>, _> = conn
            .query_row(
                "SELECT embedding FROM embeddings WHERE entry_id = ? LIMIT 1",
                params![entry_id],
                |row| row.get(0),
            )
            .map_err(DatabaseError::Sqlite);

        let embedding_bytes = match embedding_result {
            Ok(bytes) => bytes,
            Err(_) => {
                warn!("Could not get embedding for entry {}", entry_id);
                continue;
            }
        };

        // Convert bytes to f32 slice
        let embedding: Vec<f32> = bytemuck::cast_slice(&embedding_bytes).to_vec();

        // Find similar entries
        let similar = search_similar_chunks(&conn, &embedding, 50)?;

        // Group entries with similarity above threshold
        let mut cluster_members = vec![(*entry_id, date, path_str.clone())];
        processed.insert(*entry_id);

        for similar_chunk in similar {
            if similar_chunk.similarity >= similarity_threshold
                && !processed.contains(&similar_chunk.entry_id)
            {
                // Get the entry info
                if let Some((_, sim_date_str, sim_path_str)) = entries
                    .iter()
                    .find(|(id, _, _)| *id == similar_chunk.entry_id)
                {
                    if let Ok(sim_date) = NaiveDate::parse_from_str(sim_date_str, "%Y-%m-%d") {
                        cluster_members.push((
                            similar_chunk.entry_id,
                            sim_date,
                            sim_path_str.clone(),
                        ));
                        processed.insert(similar_chunk.entry_id);
                    }
                }
            }
        }

        if cluster_members.len() >= min_cluster_size {
            clusters.push(cluster_members);
        }
    }

    debug!("Found {} topic clusters", clusters.len());

    // Extract topics for each cluster using AI
    let mut topic_clusters = Vec::new();

    for (cluster_idx, cluster_members) in clusters.iter().enumerate() {
        debug!(
            "Processing cluster {} with {} entries",
            cluster_idx,
            cluster_members.len()
        );

        // Decrypt a sample of entries from the cluster (max 3) to extract topics
        let sample_size = cluster_members.len().min(3);
        let mut combined_text = String::new();

        for (_, _, path_str) in cluster_members.iter().take(sample_size) {
            match fs::read(path_str) {
                Ok(encrypted_content) => {
                    match decrypt_with_passphrase(&encrypted_content, passphrase) {
                        Ok(decrypted) => {
                            if let Ok(text) = String::from_utf8(decrypted) {
                                // Take first 500 chars to avoid overwhelming the AI
                                let excerpt = text.chars().take(500).collect::<String>();
                                combined_text.push_str(&excerpt);
                                combined_text.push_str("\n\n");
                            }
                        }
                        Err(e) => warn!("Failed to decrypt entry: {}", e),
                    }
                }
                Err(e) => warn!("Failed to read entry file: {}", e),
            }
        }

        if combined_text.is_empty() {
            continue;
        }

        // Extract topics using AI
        match ai_client.extract_topics(&combined_text) {
            Ok(topics) => {
                if !topics.is_empty() {
                    let main_topic = topics[0].clone();
                    let dates: Vec<NaiveDate> =
                        cluster_members.iter().map(|(_, date, _)| *date).collect();

                    // Store pattern in database
                    let metadata = json!({
                        "topics": topics,
                        "entry_count": cluster_members.len(),
                        "sample_size": sample_size
                    });

                    let first_date = dates
                        .iter()
                        .min()
                        .map(|d| d.to_string())
                        .unwrap_or_default();
                    let last_date = dates
                        .iter()
                        .max()
                        .map(|d| d.to_string())
                        .unwrap_or_default();

                    insert_pattern(
                        &conn,
                        PatternType::Topic,
                        &format!("Topic cluster: {}", main_topic),
                        Some(&metadata.to_string()),
                        Some(cluster_members.len() as f64 / total_entries as f64),
                        &first_date,
                        &last_date,
                    )?;

                    topic_clusters.push(TopicCluster {
                        topic: main_topic,
                        entry_dates: dates,
                        confidence: cluster_members.len() as f64 / total_entries as f64,
                    });
                }
            }
            Err(e) => warn!("Failed to extract topics for cluster: {}", e),
        }
    }

    info!("Detected {} topic clusters", topic_clusters.len());

    Ok(TopicPatterns {
        clusters: topic_clusters,
        total_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::entries::upsert_entry;
    use age::secrecy::SecretString;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_detect_temporal_patterns_no_entries() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        let result = detect_temporal_patterns(&db).unwrap();

        assert_eq!(result.total_entries, 0);
        assert_eq!(result.patterns.len(), 1);
        assert!(result.patterns[0].contains("No entries found"));
    }

    #[test]
    fn test_detect_temporal_patterns_with_entries() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Add some test entries
        let dates = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),  // Monday
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),  // Wednesday
            NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(),  // Monday
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), // Monday
        ];

        let conn = db.get_conn().unwrap();
        for date in dates {
            let path = PathBuf::from(format!("/tmp/{}.md", date));
            upsert_entry(&conn, &path, date, "abc123", 100).unwrap();
        }

        let result = detect_temporal_patterns(&db).unwrap();

        assert_eq!(result.total_entries, 4);
        assert!(!result.patterns.is_empty());
        assert!(result.avg_gap_days > 0.0);
    }

    #[test]
    fn test_detect_temporal_patterns_calculates_gaps() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());
        let db = Database::open(&db_path, &passphrase).unwrap();

        // Add entries with specific gaps
        let dates = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), // 1 day gap
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(), // 3 day gap
        ];

        let conn = db.get_conn().unwrap();
        for date in dates {
            let path = PathBuf::from(format!("/tmp/{}.md", date));
            upsert_entry(&conn, &path, date, "abc123", 100).unwrap();
        }

        let result = detect_temporal_patterns(&db).unwrap();

        assert_eq!(result.total_entries, 3);
        assert_eq!(result.avg_gap_days, 2.0); // (1 + 3) / 2
        assert_eq!(result.longest_gap_days, 3);
    }
}
