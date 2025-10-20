//! Entry CRUD operations.
//!
//! This module provides functions for creating, reading, updating, and querying
//! journal entry metadata in the database.

use crate::errors::{AppResult, DatabaseError};
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tracing::debug;

/// Represents a journal entry in the database.
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i64,
    pub path: PathBuf,
    pub date: NaiveDate,
    pub checksum: String,
    pub word_count: usize,
    pub updated_at: String,
    pub embedded_at: Option<String>,
}

/// Inserts or updates a journal entry.
///
/// If an entry with the same date already exists, it will be updated.
/// Returns the entry ID.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `path` - Path to the journal file
/// * `date` - Date of the entry
/// * `checksum` - Content checksum for change detection
/// * `word_count` - Number of words in the entry
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn upsert_entry(
    conn: &Connection,
    path: &Path,
    date: NaiveDate,
    checksum: &str,
    word_count: usize,
) -> AppResult<i64> {
    debug!(
        "Upserting entry for date {} with checksum {}",
        date, checksum
    );

    let path_str = path.to_string_lossy();

    conn.execute(
        r#"
        INSERT INTO entries (path, date, checksum, word_count, updated_at)
        VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
        ON CONFLICT(date) DO UPDATE SET
            path = excluded.path,
            checksum = excluded.checksum,
            word_count = excluded.word_count,
            updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            path_str.as_ref(),
            date.to_string(),
            checksum,
            word_count as i64
        ],
    )
    .map_err(DatabaseError::Sqlite)?;

    // Get the entry ID
    let entry_id: i64 = conn
        .query_row(
            "SELECT id FROM entries WHERE date = ?1",
            params![date.to_string()],
            |row| row.get(0),
        )
        .map_err(DatabaseError::Sqlite)?;

    debug!("Entry upserted with id {}", entry_id);
    Ok(entry_id)
}

/// Retrieves an entry by date.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `date` - Date of the entry to retrieve
///
/// # Errors
///
/// Returns an error if the database operation fails.
/// Returns `Ok(None)` if no entry exists for the given date.
pub fn get_entry_by_date(conn: &Connection, date: NaiveDate) -> AppResult<Option<Entry>> {
    debug!("Getting entry for date {}", date);

    let result = conn.query_row(
        r#"
        SELECT id, path, date, checksum, word_count, updated_at, embedded_at
        FROM entries
        WHERE date = ?1
        "#,
        params![date.to_string()],
        |row| {
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
        },
    );

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e).into()),
    }
}

/// Gets the file path for an entry by ID.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entry_id` - Entry ID
///
/// # Errors
///
/// Returns an error if the entry doesn't exist or the database operation fails.
pub fn get_entry_path(conn: &Connection, entry_id: i64) -> AppResult<PathBuf> {
    debug!("Getting path for entry id {}", entry_id);

    let path_str: String = conn
        .query_row(
            "SELECT path FROM entries WHERE id = ?1",
            params![entry_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DatabaseError::NotFound(format!("Entry with id {} not found", entry_id))
            }
            _ => DatabaseError::Sqlite(e),
        })?;

    Ok(PathBuf::from(path_str))
}

/// Gets the checksum for an entry by date.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `date` - Date of the entry
///
/// # Errors
///
/// Returns an error if the database operation fails.
/// Returns `Ok(None)` if no entry exists for the given date.
pub fn get_entry_checksum(conn: &Connection, date: NaiveDate) -> AppResult<Option<String>> {
    debug!("Getting checksum for entry date {}", date);

    let result = conn.query_row(
        "SELECT checksum FROM entries WHERE date = ?1",
        params![date.to_string()],
        |row| row.get(0),
    );

    match result {
        Ok(checksum) => Ok(Some(checksum)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e).into()),
    }
}

/// Checks if an entry needs embedding update based on checksum.
///
/// Returns `true` if:
/// - The entry has never been embedded (embedded_at is NULL)
/// - The checksum has changed since last embedding
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entry_id` - Entry ID to check
/// * `current_checksum` - Current content checksum
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn needs_embedding_update(
    conn: &Connection,
    entry_id: i64,
    current_checksum: &str,
) -> AppResult<bool> {
    debug!("Checking if entry {} needs embedding update", entry_id);

    let result: (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT embedded_at, checksum FROM entries WHERE id = ?1",
            params![entry_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DatabaseError::NotFound(format!("Entry with id {} not found", entry_id))
            }
            _ => DatabaseError::Sqlite(e),
        })?;

    let (embedded_at, stored_checksum) = result;

    // Need update if never embedded or checksum changed
    let needs_update =
        embedded_at.is_none() || stored_checksum.as_deref() != Some(current_checksum);

    debug!(
        "Entry {} needs embedding update: {}",
        entry_id, needs_update
    );
    Ok(needs_update)
}

/// Marks an entry as having been embedded.
///
/// Updates the `embedded_at` timestamp to the current time.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `entry_id` - Entry ID to mark
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn mark_embedded(conn: &Connection, entry_id: i64) -> AppResult<()> {
    debug!("Marking entry {} as embedded", entry_id);

    let rows_affected = conn
        .execute(
            "UPDATE entries SET embedded_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![entry_id],
        )
        .map_err(DatabaseError::Sqlite)?;

    if rows_affected == 0 {
        return Err(
            DatabaseError::NotFound(format!("Entry with id {} not found", entry_id)).into(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_upsert_entry_insert() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let entry_id = upsert_entry(&conn, &path, date, "abc123", 100).unwrap();
        assert!(entry_id > 0);

        // Verify entry was inserted
        let entry = get_entry_by_date(&conn, date).unwrap().unwrap();
        assert_eq!(entry.id, entry_id);
        assert_eq!(entry.path, path);
        assert_eq!(entry.checksum, "abc123");
        assert_eq!(entry.word_count, 100);
    }

    #[test]
    fn test_upsert_entry_update() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // Insert
        let entry_id1 = upsert_entry(&conn, &path, date, "abc123", 100).unwrap();

        // Update
        let entry_id2 = upsert_entry(&conn, &path, date, "def456", 200).unwrap();

        // Same ID
        assert_eq!(entry_id1, entry_id2);

        // Verify update
        let entry = get_entry_by_date(&conn, date).unwrap().unwrap();
        assert_eq!(entry.checksum, "def456");
        assert_eq!(entry.word_count, 200);
    }

    #[test]
    fn test_get_entry_by_date_not_found() {
        let conn = setup_test_db();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let result = get_entry_by_date(&conn, date).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_entry_path() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let entry_id = upsert_entry(&conn, &path, date, "abc123", 100).unwrap();
        let retrieved_path = get_entry_path(&conn, entry_id).unwrap();

        assert_eq!(retrieved_path, path);
    }

    #[test]
    fn test_get_entry_path_not_found() {
        let conn = setup_test_db();
        let result = get_entry_path(&conn, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_needs_embedding_update_never_embedded() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let entry_id = upsert_entry(&conn, &path, date, "abc123", 100).unwrap();
        let needs_update = needs_embedding_update(&conn, entry_id, "abc123").unwrap();

        assert!(needs_update); // Never embedded
    }

    #[test]
    fn test_needs_embedding_update_checksum_changed() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let entry_id = upsert_entry(&conn, &path, date, "abc123", 100).unwrap();
        mark_embedded(&conn, entry_id).unwrap();

        let needs_update = needs_embedding_update(&conn, entry_id, "def456").unwrap();
        assert!(needs_update); // Checksum changed
    }

    #[test]
    fn test_needs_embedding_update_no_change() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let entry_id = upsert_entry(&conn, &path, date, "abc123", 100).unwrap();
        mark_embedded(&conn, entry_id).unwrap();

        let needs_update = needs_embedding_update(&conn, entry_id, "abc123").unwrap();
        assert!(!needs_update); // No change
    }

    #[test]
    fn test_mark_embedded() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let entry_id = upsert_entry(&conn, &path, date, "abc123", 100).unwrap();

        // Initially not embedded
        let entry = get_entry_by_date(&conn, date).unwrap().unwrap();
        assert!(entry.embedded_at.is_none());

        // Mark as embedded
        mark_embedded(&conn, entry_id).unwrap();

        // Now embedded
        let entry = get_entry_by_date(&conn, date).unwrap().unwrap();
        assert!(entry.embedded_at.is_some());
    }

    #[test]
    fn test_mark_embedded_not_found() {
        let conn = setup_test_db();
        let result = mark_embedded(&conn, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry_checksum() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // No entry yet
        let checksum = get_entry_checksum(&conn, date).unwrap();
        assert!(checksum.is_none());

        // Create entry
        upsert_entry(&conn, &path, date, "abc123", 100).unwrap();

        // Retrieve checksum
        let checksum = get_entry_checksum(&conn, date).unwrap();
        assert_eq!(checksum, Some("abc123".to_string()));

        // Update checksum
        upsert_entry(&conn, &path, date, "def456", 100).unwrap();
        let checksum = get_entry_checksum(&conn, date).unwrap();
        assert_eq!(checksum, Some("def456".to_string()));
    }

    #[test]
    fn test_conflict_detection_scenario() {
        let conn = setup_test_db();
        let path = PathBuf::from("/tmp/20240101.md");
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // Simulate: User starts editing with checksum A
        let original_checksum = "checksum_a";
        upsert_entry(&conn, &path, date, original_checksum, 100).unwrap();

        // Simulate: External process modifies entry (checksum B)
        let external_checksum = "checksum_b";
        upsert_entry(&conn, &path, date, external_checksum, 150).unwrap();

        // Simulate: User saves their changes (detect conflict)
        let db_checksum = get_entry_checksum(&conn, date).unwrap().unwrap();
        assert_ne!(
            db_checksum, original_checksum,
            "Conflict should be detected: DB checksum changed"
        );
        assert_eq!(
            db_checksum, external_checksum,
            "DB should have external checksum"
        );

        // User's save proceeds (last-write-wins)
        let user_checksum = "checksum_user";
        upsert_entry(&conn, &path, date, user_checksum, 200).unwrap();

        // Verify user's checksum is now in DB
        let final_checksum = get_entry_checksum(&conn, date).unwrap().unwrap();
        assert_eq!(final_checksum, user_checksum, "Last write should win");
    }
}
