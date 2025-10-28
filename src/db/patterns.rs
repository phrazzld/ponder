//! Pattern CRUD operations.
//!
//! This module provides functions for creating, reading, updating, and querying
//! AI-detected patterns in journal entries.

use crate::errors::{AppResult, DatabaseError};
use rusqlite::{params, Connection, OptionalExtension};
use tracing::debug;

/// Pattern type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    Temporal,
    Topic,
    Sentiment,
    Correlation,
}

impl PatternType {
    /// Convert to database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            PatternType::Temporal => "temporal",
            PatternType::Topic => "topic",
            PatternType::Sentiment => "sentiment",
            PatternType::Correlation => "correlation",
        }
    }

    /// Parse from database string representation.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "temporal" => Some(PatternType::Temporal),
            "topic" => Some(PatternType::Topic),
            "sentiment" => Some(PatternType::Sentiment),
            "correlation" => Some(PatternType::Correlation),
            _ => None,
        }
    }
}

/// Represents a detected pattern in the database.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: i64,
    pub pattern_type: PatternType,
    pub description: String,
    pub metadata: Option<String>,
    pub confidence: Option<f64>,
    pub first_seen: String,
    pub last_seen: String,
    pub created_at: String,
}

/// Inserts a new pattern.
///
/// Returns the pattern ID.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `pattern_type` - Type of pattern detected
/// * `description` - Human-readable description of the pattern
/// * `metadata` - Optional JSON metadata with pattern details
/// * `confidence` - Optional confidence score (0.0 to 1.0)
/// * `first_seen` - First occurrence date (YYYY-MM-DD)
/// * `last_seen` - Last occurrence date (YYYY-MM-DD)
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn insert_pattern(
    conn: &Connection,
    pattern_type: PatternType,
    description: &str,
    metadata: Option<&str>,
    confidence: Option<f64>,
    first_seen: &str,
    last_seen: &str,
) -> AppResult<i64> {
    debug!(
        "Inserting {} pattern: {}",
        pattern_type.as_str(),
        description
    );

    conn.execute(
        r#"
        INSERT INTO patterns (pattern_type, description, metadata, confidence, first_seen, last_seen)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            pattern_type.as_str(),
            description,
            metadata,
            confidence,
            first_seen,
            last_seen
        ],
    )
    .map_err(DatabaseError::Sqlite)?;

    let pattern_id = conn.last_insert_rowid();
    debug!("Pattern inserted with id {}", pattern_id);
    Ok(pattern_id)
}

/// Retrieves a pattern by ID.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `id` - Pattern ID
///
/// # Errors
///
/// Returns an error if the database operation fails.
/// Returns `Ok(None)` if no pattern exists with the given ID.
pub fn get_pattern(conn: &Connection, id: i64) -> AppResult<Option<Pattern>> {
    debug!("Getting pattern with id {}", id);

    let result = conn
        .query_row(
            r#"
            SELECT id, pattern_type, description, metadata, confidence, first_seen, last_seen, created_at
            FROM patterns
            WHERE id = ?1
            "#,
            params![id],
            |row| {
                let pattern_type_str: String = row.get(1)?;
                Ok(Pattern {
                    id: row.get(0)?,
                    pattern_type: PatternType::from_str(&pattern_type_str).unwrap(),
                    description: row.get(2)?,
                    metadata: row.get(3)?,
                    confidence: row.get(4)?,
                    first_seen: row.get(5)?,
                    last_seen: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(DatabaseError::Sqlite)?;

    Ok(result)
}

/// Lists patterns filtered by type, ordered by confidence descending.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `pattern_type` - Optional pattern type to filter by
/// * `limit` - Maximum number of patterns to return
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn list_patterns(
    conn: &Connection,
    pattern_type: Option<PatternType>,
    limit: usize,
) -> AppResult<Vec<Pattern>> {
    debug!(
        "Listing patterns (type: {:?}, limit: {})",
        pattern_type, limit
    );

    let patterns = if let Some(pt) = pattern_type {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, pattern_type, description, metadata, confidence, first_seen, last_seen, created_at
                FROM patterns
                WHERE pattern_type = ?1
                ORDER BY confidence DESC, last_seen DESC
                LIMIT ?2
                "#,
            )
            .map_err(DatabaseError::Sqlite)?;

        let rows = stmt
            .query_map(params![pt.as_str(), limit as i64], |row| {
                let pattern_type_str: String = row.get(1)?;
                Ok(Pattern {
                    id: row.get(0)?,
                    pattern_type: PatternType::from_str(&pattern_type_str).unwrap(),
                    description: row.get(2)?,
                    metadata: row.get(3)?,
                    confidence: row.get(4)?,
                    first_seen: row.get(5)?,
                    last_seen: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(DatabaseError::Sqlite)?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::Sqlite)?
    } else {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, pattern_type, description, metadata, confidence, first_seen, last_seen, created_at
                FROM patterns
                ORDER BY confidence DESC, last_seen DESC
                LIMIT ?1
                "#,
            )
            .map_err(DatabaseError::Sqlite)?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let pattern_type_str: String = row.get(1)?;
                Ok(Pattern {
                    id: row.get(0)?,
                    pattern_type: PatternType::from_str(&pattern_type_str).unwrap(),
                    description: row.get(2)?,
                    metadata: row.get(3)?,
                    confidence: row.get(4)?,
                    first_seen: row.get(5)?,
                    last_seen: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(DatabaseError::Sqlite)?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(DatabaseError::Sqlite)?
    };

    debug!("Found {} patterns", patterns.len());
    Ok(patterns)
}

/// Updates an existing pattern's last_seen date and optionally confidence.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `id` - Pattern ID to update
/// * `last_seen` - New last seen date (YYYY-MM-DD)
/// * `confidence` - Optional new confidence score
///
/// # Errors
///
/// Returns an error if the database operation fails or pattern doesn't exist.
pub fn update_pattern(
    conn: &Connection,
    id: i64,
    last_seen: &str,
    confidence: Option<f64>,
) -> AppResult<()> {
    debug!("Updating pattern {} with last_seen: {}", id, last_seen);

    let rows_affected = if let Some(conf) = confidence {
        conn.execute(
            "UPDATE patterns SET last_seen = ?1, confidence = ?2 WHERE id = ?3",
            params![last_seen, conf, id],
        )
        .map_err(DatabaseError::Sqlite)?
    } else {
        conn.execute(
            "UPDATE patterns SET last_seen = ?1 WHERE id = ?2",
            params![last_seen, id],
        )
        .map_err(DatabaseError::Sqlite)?
    };

    if rows_affected == 0 {
        return Err(DatabaseError::Custom(format!("Pattern {} not found", id)).into());
    }

    debug!("Pattern {} updated", id);
    Ok(())
}

/// Deletes a pattern by ID.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `id` - Pattern ID to delete
///
/// # Errors
///
/// Returns an error if the database operation fails or pattern doesn't exist.
pub fn delete_pattern(conn: &Connection, id: i64) -> AppResult<()> {
    debug!("Deleting pattern {}", id);

    let rows_affected = conn
        .execute("DELETE FROM patterns WHERE id = ?1", params![id])
        .map_err(DatabaseError::Sqlite)?;

    if rows_affected == 0 {
        return Err(DatabaseError::Custom(format!("Pattern {} not found", id)).into());
    }

    debug!("Pattern {} deleted", id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::create_tables;
    use rusqlite::Connection;

    #[test]
    fn test_insert_pattern() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let id = insert_pattern(
            &conn,
            PatternType::Temporal,
            "You write most on Sunday evenings",
            Some(r#"{"day_of_week": "Sunday", "hour_range": [21, 23]}"#),
            Some(0.85),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();

        assert!(id > 0);

        // Verify it was inserted
        let pattern = get_pattern(&conn, id).unwrap().unwrap();
        assert_eq!(pattern.pattern_type, PatternType::Temporal);
        assert_eq!(pattern.description, "You write most on Sunday evenings");
        assert!(pattern.metadata.unwrap().contains("Sunday"));
        assert_eq!(pattern.confidence.unwrap(), 0.85);
        assert_eq!(pattern.first_seen, "2024-01-01");
        assert_eq!(pattern.last_seen, "2024-01-31");
    }

    #[test]
    fn test_get_pattern_not_found() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let result = get_pattern(&conn, 999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_patterns_by_type() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Insert patterns of different types
        insert_pattern(
            &conn,
            PatternType::Temporal,
            "Pattern 1",
            None,
            Some(0.9),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();
        insert_pattern(
            &conn,
            PatternType::Temporal,
            "Pattern 2",
            None,
            Some(0.7),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();
        insert_pattern(
            &conn,
            PatternType::Topic,
            "Pattern 3",
            None,
            Some(0.8),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();

        // List temporal patterns only
        let temporal = list_patterns(&conn, Some(PatternType::Temporal), 10).unwrap();
        assert_eq!(temporal.len(), 2);
        assert!(temporal
            .iter()
            .all(|p| p.pattern_type == PatternType::Temporal));

        // Should be ordered by confidence DESC
        assert_eq!(temporal[0].confidence.unwrap(), 0.9);
        assert_eq!(temporal[1].confidence.unwrap(), 0.7);

        // List topic patterns only
        let topics = list_patterns(&conn, Some(PatternType::Topic), 10).unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].pattern_type, PatternType::Topic);
    }

    #[test]
    fn test_list_all_patterns() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Insert 4 patterns
        insert_pattern(
            &conn,
            PatternType::Temporal,
            "Pattern 1",
            None,
            Some(0.6),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();
        insert_pattern(
            &conn,
            PatternType::Topic,
            "Pattern 2",
            None,
            Some(0.9),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();
        insert_pattern(
            &conn,
            PatternType::Sentiment,
            "Pattern 3",
            None,
            Some(0.7),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();
        insert_pattern(
            &conn,
            PatternType::Correlation,
            "Pattern 4",
            None,
            Some(0.8),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();

        // List all with limit
        let all_patterns = list_patterns(&conn, None, 2).unwrap();
        assert_eq!(all_patterns.len(), 2);

        // Should be ordered by confidence DESC
        assert_eq!(all_patterns[0].confidence.unwrap(), 0.9);
        assert_eq!(all_patterns[1].confidence.unwrap(), 0.8);
    }

    #[test]
    fn test_update_pattern() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let id = insert_pattern(
            &conn,
            PatternType::Temporal,
            "Test pattern",
            None,
            Some(0.5),
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();

        // Update last_seen and confidence
        update_pattern(&conn, id, "2024-02-28", Some(0.75)).unwrap();

        let pattern = get_pattern(&conn, id).unwrap().unwrap();
        assert_eq!(pattern.last_seen, "2024-02-28");
        assert_eq!(pattern.confidence.unwrap(), 0.75);

        // Update just last_seen
        update_pattern(&conn, id, "2024-03-15", None).unwrap();

        let pattern = get_pattern(&conn, id).unwrap().unwrap();
        assert_eq!(pattern.last_seen, "2024-03-15");
        assert_eq!(pattern.confidence.unwrap(), 0.75); // Should be unchanged
    }

    #[test]
    fn test_update_nonexistent_pattern() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let result = update_pattern(&conn, 999, "2024-01-01", Some(0.5));
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_pattern() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let id = insert_pattern(
            &conn,
            PatternType::Temporal,
            "Test pattern",
            None,
            None,
            "2024-01-01",
            "2024-01-31",
        )
        .unwrap();

        // Verify it exists
        assert!(get_pattern(&conn, id).unwrap().is_some());

        // Delete it
        delete_pattern(&conn, id).unwrap();

        // Verify it's gone
        assert!(get_pattern(&conn, id).unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_pattern() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let result = delete_pattern(&conn, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_with_json_metadata() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let metadata = r#"{
            "day_of_week": "Sunday",
            "hour_range": [21, 23],
            "average_word_count": 450,
            "consistency_score": 0.92
        }"#;

        let id = insert_pattern(
            &conn,
            PatternType::Temporal,
            "Peak writing time analysis",
            Some(metadata),
            Some(0.92),
            "2024-01-01",
            "2024-03-31",
        )
        .unwrap();

        let pattern = get_pattern(&conn, id).unwrap().unwrap();
        let stored_metadata = pattern.metadata.unwrap();
        assert!(stored_metadata.contains("day_of_week"));
        assert!(stored_metadata.contains("consistency_score"));
        assert!(stored_metadata.contains("0.92"));
    }

    #[test]
    fn test_pattern_type_conversion() {
        assert_eq!(PatternType::Temporal.as_str(), "temporal");
        assert_eq!(PatternType::Topic.as_str(), "topic");
        assert_eq!(PatternType::Sentiment.as_str(), "sentiment");
        assert_eq!(PatternType::Correlation.as_str(), "correlation");

        assert_eq!(
            PatternType::from_str("temporal"),
            Some(PatternType::Temporal)
        );
        assert_eq!(PatternType::from_str("topic"), Some(PatternType::Topic));
        assert_eq!(
            PatternType::from_str("sentiment"),
            Some(PatternType::Sentiment)
        );
        assert_eq!(
            PatternType::from_str("correlation"),
            Some(PatternType::Correlation)
        );
        assert_eq!(PatternType::from_str("invalid"), None);
    }
}
