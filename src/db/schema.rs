//! Database schema definitions and initialization.
//!
//! This module defines the SQLite schema for journal entries, embeddings,
//! and related metadata. All tables are created with proper indexes and
//! foreign key constraints.

use crate::errors::{AppResult, DatabaseError};
use rusqlite::Connection;
use tracing::debug;

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

    debug!("Database tables created successfully");
    Ok(())
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
}
