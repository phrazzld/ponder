//! Vector embedding operations.
//!
//! This module provides functions for storing and searching vector embeddings
//! for semantic similarity search.

use crate::constants::EMBEDDING_DIMENSIONS;
use crate::errors::{AppResult, DatabaseError};
use bytemuck::{cast_slice, cast_slice_mut};
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use tracing::debug;

/// Result of a similarity search.
#[derive(Debug, Clone)]
pub struct SimilarChunk {
    pub entry_id: i64,
    pub chunk_idx: usize,
    pub similarity: f32,
}

/// Configuration for temporal-aware similarity search.
#[derive(Debug, Clone)]
pub struct TemporalSearchConfig {
    /// Optional date range filter (start_date, end_date)
    pub date_range: Option<(NaiveDate, NaiveDate)>,

    /// Whether the temporal constraint was explicitly provided by the user.
    /// If true, window expansion will be disabled to respect user intent.
    /// If false, the constraint is implicit/default and expansion is allowed.
    pub explicit_temporal_constraint: bool,

    /// Recency decay factor in days (smaller = faster decay)
    /// Boost formula: exp(-days_old / decay_factor)
    /// Default: 30 days
    pub recency_decay_days: f32,

    /// Minimum results before expanding date window
    /// If filtered results < threshold, expand window and retry
    /// Only applies when explicit_temporal_constraint is false
    /// Default: 3
    pub min_results_threshold: usize,

    /// Today's date for recency calculation (typically Utc::now().date_naive())
    pub reference_date: NaiveDate,
}

/// Inserts an embedding vector for a journal entry chunk.
///
/// Accepts either a database connection or transaction (Transaction derefs to Connection).
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entry_id` - ID of the journal entry
/// * `chunk_idx` - Index of the chunk within the entry
/// * `embedding` - 768-dimensional embedding vector
/// * `checksum` - Content checksum for the chunk
///
/// # Errors
///
/// Returns an error if:
/// - Embedding dimensions don't match expected size
/// - Database operation fails
pub fn insert_embedding(
    conn: &Connection,
    entry_id: i64,
    chunk_idx: usize,
    embedding: &[f32],
    checksum: &str,
) -> AppResult<()> {
    if embedding.len() != EMBEDDING_DIMENSIONS {
        return Err(DatabaseError::Custom(format!(
            "Invalid embedding dimensions: expected {}, got {}",
            EMBEDDING_DIMENSIONS,
            embedding.len()
        ))
        .into());
    }

    debug!(
        "Inserting embedding for entry {} chunk {}",
        entry_id, chunk_idx
    );

    // Convert f32 slice to bytes
    let bytes = cast_slice::<f32, u8>(embedding);

    conn.execute(
        r#"
        INSERT INTO embeddings (entry_id, chunk_idx, embedding, checksum)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(entry_id, chunk_idx) DO UPDATE SET
            embedding = excluded.embedding,
            checksum = excluded.checksum,
            created_at = CURRENT_TIMESTAMP
        "#,
        params![entry_id, chunk_idx as i64, bytes, checksum],
    )
    .map_err(DatabaseError::Sqlite)?;

    Ok(())
}

/// Retrieves all embeddings for a journal entry.
///
/// Returns a vector of (chunk_idx, embedding) tuples.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entry_id` - ID of the journal entry
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn get_entry_embeddings(conn: &Connection, entry_id: i64) -> AppResult<Vec<(usize, Vec<f32>)>> {
    debug!("Getting embeddings for entry {}", entry_id);

    let mut stmt = conn
        .prepare(
            r#"
        SELECT chunk_idx, embedding
        FROM embeddings
        WHERE entry_id = ?1
        ORDER BY chunk_idx
        "#,
        )
        .map_err(DatabaseError::Sqlite)?;

    let embeddings = stmt
        .query_map(params![entry_id], |row| {
            let chunk_idx: i64 = row.get(0)?;
            let bytes: Vec<u8> = row.get(1)?;

            // Convert bytes to f32 slice
            let mut values = vec![0.0f32; EMBEDDING_DIMENSIONS];
            cast_slice_mut::<f32, u8>(&mut values).copy_from_slice(&bytes);

            Ok((chunk_idx as usize, values))
        })
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    Ok(embeddings)
}

/// Searches for chunks similar to the query embedding with temporal awareness.
///
/// This function extends basic similarity search with:
/// 1. Optional date range filtering
/// 2. Recency boosting (recent entries get higher scores)
/// 3. Fallback expansion if too few results in date range
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `query_embedding` - Query embedding vector
/// * `limit` - Maximum number of results to return
/// * `config` - Optional temporal search configuration
///
/// # Errors
///
/// Returns an error if:
/// - Query embedding dimensions don't match expected size
/// - Database operation fails
pub fn search_similar_chunks_with_temporal(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
    config: Option<&TemporalSearchConfig>,
) -> AppResult<Vec<SimilarChunk>> {
    if query_embedding.len() != EMBEDDING_DIMENSIONS {
        return Err(DatabaseError::Custom(format!(
            "Invalid query embedding dimensions: expected {}, got {}",
            EMBEDDING_DIMENSIONS,
            query_embedding.len()
        ))
        .into());
    }

    let config = match config {
        Some(c) => c,
        None => {
            // No temporal config - use basic search
            return search_similar_chunks_basic(conn, query_embedding, limit);
        }
    };

    debug!(
        "Temporal search with date range: {:?}, decay: {} days",
        config.date_range, config.recency_decay_days
    );

    // Try with date filter first
    let mut results = search_with_date_filter(
        conn,
        query_embedding,
        limit * 2, // Fetch 2x limit before recency boosting
        config.date_range.as_ref(),
    )?;

    // Apply recency boosting if we have a date range
    if config.date_range.is_some() {
        apply_recency_boost(conn, &mut results, config)?;
    }

    // Check if we need to expand the date window
    // Only expand if constraint is implicit (not explicitly provided by user)
    if results.len() < config.min_results_threshold
        && config.date_range.is_some()
        && !config.explicit_temporal_constraint
    {
        debug!(
            "Only {} results with date filter (implicit constraint), expanding window",
            results.len()
        );

        // Retry without date filter, but keep recency boosting
        results = search_with_date_filter(conn, query_embedding, limit * 2, None)?;
        apply_recency_boost(conn, &mut results, config)?;
    } else if results.len() < config.min_results_threshold
        && config.date_range.is_some()
        && config.explicit_temporal_constraint
    {
        debug!(
            "Only {} results with explicit date filter - respecting user constraint, not expanding",
            results.len()
        );
    }

    // Sort by final score and take top N
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    results.truncate(limit);

    debug!(
        "Found {} similar chunks after temporal filtering",
        results.len()
    );
    Ok(results)
}

/// Searches with optional date range filter.
fn search_with_date_filter(
    conn: &Connection,
    query_embedding: &[f32],
    _limit: usize,
    date_range: Option<&(NaiveDate, NaiveDate)>,
) -> AppResult<Vec<SimilarChunk>> {
    let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = match date_range {
        Some((start, end)) => (
            r#"
            SELECT e.entry_id, e.chunk_idx, e.embedding
            FROM embeddings e
            INNER JOIN entries ent ON e.entry_id = ent.id
            WHERE ent.date >= ?1 AND ent.date <= ?2
            "#
            .to_string(),
            vec![
                Box::new(start.format("%Y-%m-%d").to_string()),
                Box::new(end.format("%Y-%m-%d").to_string()),
            ],
        ),
        None => (
            "SELECT entry_id, chunk_idx, embedding FROM embeddings".to_string(),
            vec![],
        ),
    };

    let mut stmt = conn.prepare(&sql).map_err(DatabaseError::Sqlite)?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let results: Vec<SimilarChunk> = stmt
        .query_map(params_refs.as_slice(), |row| {
            let entry_id: i64 = row.get(0)?;
            let chunk_idx: i64 = row.get(1)?;
            let bytes: Vec<u8> = row.get(2)?;

            // Convert bytes to f32 slice
            let mut values = vec![0.0f32; EMBEDDING_DIMENSIONS];
            cast_slice_mut::<f32, u8>(&mut values).copy_from_slice(&bytes);

            // Calculate cosine similarity
            let similarity = cosine_similarity(query_embedding, &values);

            Ok(SimilarChunk {
                entry_id,
                chunk_idx: chunk_idx as usize,
                similarity,
            })
        })
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    Ok(results)
}

/// Applies recency boost to similarity scores based on entry dates.
fn apply_recency_boost(
    conn: &Connection,
    results: &mut [SimilarChunk],
    config: &TemporalSearchConfig,
) -> AppResult<()> {
    if results.is_empty() {
        return Ok(());
    }

    // Get dates for all entries
    let entry_ids: Vec<String> = results
        .iter()
        .map(|r| r.entry_id.to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let placeholders = entry_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, date FROM entries WHERE id IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&sql).map_err(DatabaseError::Sqlite)?;

    let params: Vec<&dyn rusqlite::ToSql> = entry_ids
        .iter()
        .map(|id| id as &dyn rusqlite::ToSql)
        .collect();

    let date_map: std::collections::HashMap<i64, NaiveDate> = stmt
        .query_map(params.as_slice(), |row| {
            let id: i64 = row.get(0)?;
            let date_str: String = row.get(1)?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok((id, date))
        })
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<_, _>>()
        .map_err(DatabaseError::Sqlite)?;

    // Apply recency boost
    for chunk in results.iter_mut() {
        if let Some(entry_date) = date_map.get(&chunk.entry_id) {
            let days_old = (config.reference_date - *entry_date).num_days() as f32;
            let recency_weight = (-days_old / config.recency_decay_days).exp();

            // Boost similarity score
            chunk.similarity *= recency_weight;

            debug!(
                "Entry {} ({} days old): recency_weight={:.3}, boosted_score={:.3}",
                chunk.entry_id, days_old, recency_weight, chunk.similarity
            );
        }
    }

    Ok(())
}

/// Searches for chunks similar to the query embedding (basic version without temporal awareness).
///
/// Uses cosine similarity to rank results. Returns the top N most similar chunks.
/// This is the original implementation, kept for backward compatibility.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `query_embedding` - Query embedding vector
/// * `limit` - Maximum number of results to return
///
/// # Errors
///
/// Returns an error if:
/// - Query embedding dimensions don't match expected size
/// - Database operation fails
pub fn search_similar_chunks(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> AppResult<Vec<SimilarChunk>> {
    search_similar_chunks_with_temporal(conn, query_embedding, limit, None)
}

/// Basic search without temporal awareness (internal helper).
fn search_similar_chunks_basic(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> AppResult<Vec<SimilarChunk>> {
    debug!("Searching for similar chunks (limit: {})", limit);

    let mut stmt = conn
        .prepare("SELECT entry_id, chunk_idx, embedding FROM embeddings")
        .map_err(DatabaseError::Sqlite)?;

    let mut results: Vec<SimilarChunk> = stmt
        .query_map([], |row| {
            let entry_id: i64 = row.get(0)?;
            let chunk_idx: i64 = row.get(1)?;
            let bytes: Vec<u8> = row.get(2)?;

            // Convert bytes to f32 slice
            let mut values = vec![0.0f32; EMBEDDING_DIMENSIONS];
            cast_slice_mut::<f32, u8>(&mut values).copy_from_slice(&bytes);

            // Calculate cosine similarity
            let similarity = cosine_similarity(query_embedding, &values);

            Ok(SimilarChunk {
                entry_id,
                chunk_idx: chunk_idx as usize,
                similarity,
            })
        })
        .map_err(DatabaseError::Sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::Sqlite)?;

    // Sort by similarity descending and take top N
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    results.truncate(limit);

    debug!("Found {} similar chunks", results.len());
    Ok(results)
}

/// Calculates cosine similarity between two vectors.
///
/// Returns a value between -1.0 (opposite) and 1.0 (identical).
///
/// # Panics
///
/// Panics if vectors have different lengths.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector lengths must match");

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

/// Gets the total number of embeddings (chunks) in the database.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn count_total_embeddings(conn: &Connection) -> AppResult<usize> {
    debug!("Counting total embeddings");

    let count: usize = conn
        .query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))
        .map_err(DatabaseError::Sqlite)?;

    debug!("Total embeddings: {}", count);
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();

        // Insert a test entry
        conn.execute(
            "INSERT INTO entries (path, date, checksum, word_count) VALUES (?, ?, ?, ?)",
            params!["/tmp/test.md", "2024-01-01", "abc123", 100],
        )
        .unwrap();

        conn
    }

    fn create_test_embedding(seed: f32) -> Vec<f32> {
        (0..EMBEDDING_DIMENSIONS)
            .map(|i| (i as f32 * seed).sin())
            .collect()
    }

    #[test]
    fn test_insert_embedding() {
        let conn = setup_test_db();
        let embedding = create_test_embedding(1.0);

        insert_embedding(&conn, 1, 0, &embedding, "abc123").unwrap();

        // Verify insertion
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embeddings WHERE entry_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_embedding_wrong_dimensions() {
        let conn = setup_test_db();
        let embedding = vec![0.0; 100]; // Wrong size

        let result = insert_embedding(&conn, 1, 0, &embedding, "abc123");
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_embedding_upsert() {
        let conn = setup_test_db();
        let embedding1 = create_test_embedding(1.0);
        let embedding2 = create_test_embedding(2.0);

        // Insert
        insert_embedding(&conn, 1, 0, &embedding1, "abc123").unwrap();

        // Update
        insert_embedding(&conn, 1, 0, &embedding2, "def456").unwrap();

        // Should still be 1 row
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embeddings WHERE entry_id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_entry_embeddings() {
        let conn = setup_test_db();
        let embedding1 = create_test_embedding(1.0);
        let embedding2 = create_test_embedding(2.0);

        insert_embedding(&conn, 1, 0, &embedding1, "abc123").unwrap();
        insert_embedding(&conn, 1, 1, &embedding2, "def456").unwrap();

        let embeddings = get_entry_embeddings(&conn, 1).unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].0, 0);
        assert_eq!(embeddings[1].0, 1);
    }

    #[test]
    fn test_get_entry_embeddings_empty() {
        let conn = setup_test_db();
        let embeddings = get_entry_embeddings(&conn, 1).unwrap();
        assert_eq!(embeddings.len(), 0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let vec = create_test_embedding(1.0);
        let similarity = cosine_similarity(&vec, &vec);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!(similarity.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![-1.0, 0.0, 0.0];
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_search_similar_chunks() {
        let conn = setup_test_db();
        let query = create_test_embedding(1.0);
        let similar = create_test_embedding(1.1); // Very similar
        let different = create_test_embedding(5.0); // Different

        insert_embedding(&conn, 1, 0, &similar, "abc123").unwrap();
        insert_embedding(&conn, 1, 1, &different, "def456").unwrap();

        let results = search_similar_chunks(&conn, &query, 2).unwrap();
        assert_eq!(results.len(), 2);

        // First result should be more similar
        assert!(results[0].similarity > results[1].similarity);
        assert_eq!(results[0].chunk_idx, 0);
    }

    #[test]
    fn test_search_similar_chunks_limit() {
        let conn = setup_test_db();
        let query = create_test_embedding(1.0);

        // Insert 5 embeddings
        for i in 0..5 {
            let embedding = create_test_embedding(i as f32);
            insert_embedding(&conn, 1, i, &embedding, "abc123").unwrap();
        }

        // Request only 3
        let results = search_similar_chunks(&conn, &query, 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_similar_chunks_wrong_dimensions() {
        let conn = setup_test_db();
        let query = vec![0.0; 100]; // Wrong size

        let result = search_similar_chunks(&conn, &query, 10);
        assert!(result.is_err());
    }
}
