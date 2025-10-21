//! Database schema definitions and initialization.
//!
//! This module defines the SQLite schema for journal entries, embeddings,
//! and related metadata. All tables are created with proper indexes and
//! foreign key constraints.

use crate::errors::{AppResult, DatabaseError};
use rusqlite::Connection;
use tracing::{debug, info};

/// Current schema version.
///
/// Increment this whenever schema changes are made to support future migrations.
pub const SCHEMA_VERSION: i32 = 1;

/// Creates all database tables and indexes.
///
/// This function is idempotent - it uses `CREATE TABLE IF NOT EXISTS`
/// so it's safe to call multiple times.
///
/// # Tables
///
/// - `entries`: Journal entry metadata
/// - `embeddings`: Vector embeddings for semantic search
/// - `entries_fts`: Full-text search index
/// - `insights`: AI-generated insights
/// - `reports`: Generated reports
/// - `backup_log`: History of all backups
/// - `backup_state`: Singleton tracking latest backup state
///
/// # Errors
///
/// Returns an error if any DDL statement fails.
pub fn create_tables(conn: &Connection) -> AppResult<()> {
    debug!("Creating database tables");

    // Enable foreign key constraints
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(DatabaseError::Sqlite)?;

    // Entries table: metadata about journal files
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            date DATE NOT NULL UNIQUE,
            checksum TEXT NOT NULL,
            word_count INTEGER NOT NULL DEFAULT 0,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            embedded_at DATETIME
        );

        CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date DESC);
        CREATE INDEX IF NOT EXISTS idx_entries_embedded_at ON entries(embedded_at);
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Embeddings table: vector embeddings for chunks
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS embeddings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL,
            chunk_idx INTEGER NOT NULL,
            embedding BLOB NOT NULL,
            checksum TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            UNIQUE(entry_id, chunk_idx)
        );

        CREATE INDEX IF NOT EXISTS idx_embeddings_entry_id ON embeddings(entry_id);
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Full-text search virtual table
    conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            entry_id UNINDEXED,
            date,
            content,
            content='',
            tokenize='porter unicode61'
        );
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Insights table: AI-generated insights about entries
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS insights (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            encrypted_content BLOB NOT NULL,
            score REAL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_insights_entry_id ON insights(entry_id);
        CREATE INDEX IF NOT EXISTS idx_insights_type ON insights(type);
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Reports table: generated reports
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            type TEXT NOT NULL,
            date_range TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_reports_type ON reports(type);
        CREATE INDEX IF NOT EXISTS idx_reports_created_at ON reports(created_at DESC);
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Backup log table: history of all backups
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS backup_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            backup_path TEXT NOT NULL,
            backup_type TEXT NOT NULL CHECK(backup_type IN ('full', 'incremental')),
            entries_count INTEGER NOT NULL,
            archive_size INTEGER NOT NULL,
            checksum TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_backup_log_created_at ON backup_log(created_at DESC);
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Backup state table: singleton tracking latest backup state
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS backup_state (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            last_backup_at DATETIME NOT NULL,
            last_backup_checksum TEXT NOT NULL
        );
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Schema version tracking table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL,
            applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .map_err(DatabaseError::Sqlite)?;

    // Record schema version if not already recorded
    let current_version = get_schema_version(conn)?;
    if current_version.is_none() {
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            [SCHEMA_VERSION],
        )
        .map_err(DatabaseError::Sqlite)?;
        info!("Initialized database schema version {}", SCHEMA_VERSION);
    } else {
        debug!("Schema version already recorded: {:?}", current_version);
    }

    debug!("Database tables created successfully");
    Ok(())
}

/// Gets the current schema version from the database.
///
/// Returns `None` if the schema_version table doesn't exist or is empty.
///
/// # Errors
///
/// Returns an error if the query fails for reasons other than missing table.
pub fn get_schema_version(conn: &Connection) -> AppResult<Option<i32>> {
    let result = conn.query_row(
        "SELECT version FROM schema_version ORDER BY applied_at DESC LIMIT 1",
        [],
        |row| row.get(0),
    );

    match result {
        Ok(version) => Ok(Some(version)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) if e.to_string().contains("no such table") => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_create_tables() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Verify entries table exists
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // Verify embeddings table exists
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='embeddings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // Verify FTS table exists
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // Verify insights table exists
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='insights'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // Verify reports table exists
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='reports'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Insert an entry
        conn.execute(
            "INSERT INTO entries (path, date, checksum, word_count) VALUES (?, ?, ?, ?)",
            ["test.md", "2024-01-01", "abc123", "100"],
        )
        .unwrap();

        // Insert an embedding
        let blob = vec![0u8; 100];
        conn.execute(
            "INSERT INTO embeddings (entry_id, chunk_idx, embedding, checksum) VALUES (?, ?, ?, ?)",
            rusqlite::params![1, 0, &blob, "abc123"],
        )
        .unwrap();

        // Try to insert embedding with non-existent entry_id
        let result = conn.execute(
            "INSERT INTO embeddings (entry_id, chunk_idx, embedding, checksum) VALUES (?, ?, ?, ?)",
            rusqlite::params![999, 0, &blob, "abc123"],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_indexes_created() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Check that indexes exist
        let index_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_count >= 5); // Should have at least 5 explicit indexes
    }

    #[test]
    fn test_create_tables_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Create tables twice - should not error
        create_tables(&conn).unwrap();
        create_tables(&conn).unwrap();
    }

    #[test]
    fn test_backup_tables_created() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Verify backup_log table exists
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='backup_log'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);

        // Verify backup_state table exists
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='backup_state'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);
    }

    #[test]
    fn test_backup_type_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Valid backup types should succeed
        conn.execute(
            "INSERT INTO backup_log (backup_path, backup_type, entries_count, archive_size, checksum) VALUES (?, ?, ?, ?, ?)",
            ["test.tar.gz.age", "full", "10", "1024", "abc123"],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO backup_log (backup_path, backup_type, entries_count, archive_size, checksum) VALUES (?, ?, ?, ?, ?)",
            ["test2.tar.gz.age", "incremental", "5", "512", "def456"],
        )
        .unwrap();

        // Invalid backup type should fail
        let result = conn.execute(
            "INSERT INTO backup_log (backup_path, backup_type, entries_count, archive_size, checksum) VALUES (?, ?, ?, ?, ?)",
            ["test3.tar.gz.age", "invalid", "5", "512", "ghi789"],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_backup_state_singleton() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // First insert should succeed
        conn.execute(
            "INSERT INTO backup_state (id, last_backup_at, last_backup_checksum) VALUES (?, ?, ?)",
            rusqlite::params![1, "2024-01-01 00:00:00", "abc123"],
        )
        .unwrap();

        // Second insert with id=1 should fail (unique constraint)
        let result = conn.execute(
            "INSERT INTO backup_state (id, last_backup_at, last_backup_checksum) VALUES (?, ?, ?)",
            rusqlite::params![1, "2024-01-02 00:00:00", "def456"],
        );
        assert!(result.is_err());

        // Update should work
        conn.execute(
            "UPDATE backup_state SET last_backup_at = ?, last_backup_checksum = ? WHERE id = 1",
            rusqlite::params!["2024-01-03 00:00:00", "ghi789"],
        )
        .unwrap();
    }
}
