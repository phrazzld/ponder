//! Database operations for journal entries and embeddings.
//!
//! This module provides encrypted SQLite database operations using SQLCipher
//! for storing journal metadata, embeddings, and insights. It uses connection
//! pooling via r2d2 for efficient concurrent access.
//!
//! # Module Structure
//!
//! - `schema`: Table definitions and schema initialization
//! - `entries`: Entry CRUD operations
//! - `embeddings`: Vector storage and similarity search
//!
//! # Example
//!
//! ```no_run
//! use ponder::db::Database;
//! use age::secrecy::SecretString;
//! use std::path::Path;
//!
//! let db_path = Path::new("/tmp/ponder.db");
//! let passphrase = SecretString::new("secret".to_string());
//! let db = Database::open(db_path, &passphrase)?;
//! db.initialize_schema()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod embeddings;
pub mod entries;
pub mod schema;

use crate::errors::AppResult;
use age::secrecy::{ExposeSecret, SecretString};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use tracing::{debug, info, warn};

/// Type alias for a pooled SQLite connection.
pub type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Database handle with connection pooling.
///
/// This struct manages an encrypted SQLite database using SQLCipher.
/// The connection pool allows multiple concurrent operations while
/// maintaining encryption.
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Opens or creates an encrypted SQLite database.
    ///
    /// The database is encrypted using SQLCipher with the provided passphrase.
    /// If the database file doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the database file
    /// * `passphrase` - Encryption passphrase for SQLCipher
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database file cannot be opened
    /// - Wrong passphrase provided for existing database
    /// - Connection pool cannot be initialized
    pub fn open(db_path: &Path, passphrase: &SecretString) -> AppResult<Self> {
        debug!("Opening database at: {:?}", db_path);

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .max_size(5) // Allow up to 5 concurrent connections
            .connection_customizer(Box::new(SqlCipherConfig {
                passphrase: passphrase.expose_secret().to_string(),
            }))
            .build(manager)
            .map_err(crate::errors::DatabaseError::Pool)?;

        // Test the connection and passphrase
        let conn = pool.get().map_err(crate::errors::DatabaseError::Pool)?;

        // cipher_integrity_check will fail if passphrase is wrong
        if let Err(e) = conn.execute_batch("PRAGMA cipher_integrity_check") {
            // Check if this is a wrong passphrase error
            let err_msg = e.to_string();
            if err_msg.contains("file is not a database")
                || err_msg.contains("file is encrypted")
                || err_msg.contains("cipher")
            {
                debug!("Wrong passphrase detected: {}", err_msg);
                return Err(crate::errors::DatabaseError::WrongPassphrase.into());
            }
            return Err(crate::errors::DatabaseError::Sqlite(e).into());
        }

        drop(conn);

        let db = Database { pool };

        // Auto-initialize schema (idempotent - safe on existing databases)
        db.initialize_schema()?;

        info!("Database opened successfully");
        Ok(db)
    }

    /// Gets a connection from the pool.
    ///
    /// # Errors
    ///
    /// Returns an error if no connection is available or the pool is exhausted.
    pub fn get_conn(&self) -> AppResult<PooledConnection> {
        self.pool
            .get()
            .map_err(|e| crate::errors::DatabaseError::Pool(e).into())
    }

    /// Initializes the database schema.
    ///
    /// Creates all necessary tables and indexes if they don't exist.
    /// This is idempotent and safe to call multiple times.
    ///
    /// # Errors
    ///
    /// Returns an error if schema creation fails.
    pub fn initialize_schema(&self) -> AppResult<()> {
        let conn = self.get_conn()?;
        schema::create_tables(&conn)?;
        info!("Database schema initialized");
        Ok(())
    }

    /// Validates that the database schema is correct and complete.
    ///
    /// Checks that all required tables exist and schema version is compatible.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required tables are missing
    /// - Schema version is incompatible
    pub fn validate_schema(&self) -> AppResult<()> {
        let conn = self.get_conn()?;

        // Check required tables exist
        let required_tables = vec!["entries", "embeddings", "schema_version"];
        for table in required_tables {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?)",
                    [table],
                    |row| row.get(0),
                )
                .map_err(crate::errors::DatabaseError::Sqlite)?;

            if !exists {
                return Err(crate::errors::AppError::Database(
                    crate::errors::DatabaseError::Custom(format!(
                        "Required table '{}' missing. Database may be corrupted or incompletely initialized.",
                        table
                    )),
                ));
            }
        }

        // Check schema version
        let version = schema::get_schema_version(&conn)?;
        match version {
            Some(v) if v == schema::SCHEMA_VERSION => {
                debug!("Schema version {} is current", v);
            }
            Some(v) if v < schema::SCHEMA_VERSION => {
                warn!(
                    "Schema version {} is older than current version {}. Migration may be needed.",
                    v,
                    schema::SCHEMA_VERSION
                );
            }
            Some(v) => {
                warn!(
                    "Schema version {} is newer than expected {}. Application may be out of date.",
                    v,
                    schema::SCHEMA_VERSION
                );
            }
            None => {
                warn!("Schema version not found. Database may be from an older version.");
            }
        }

        Ok(())
    }

    /// Records a backup operation in the database.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup archive file
    /// * `backup_type` - Type of backup ("full" or "incremental")
    /// * `entries_count` - Number of entries in the backup
    /// * `archive_size` - Size of the archive in bytes
    /// * `checksum` - BLAKE3 checksum of the archive
    ///
    /// # Returns
    ///
    /// Returns the ID of the inserted backup record.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub fn record_backup(
        &self,
        backup_path: &str,
        backup_type: &str,
        entries_count: i64,
        archive_size: i64,
        checksum: &str,
    ) -> AppResult<i64> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO backup_log (backup_path, backup_type, entries_count, archive_size, checksum) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![backup_path, backup_type, entries_count, archive_size, checksum],
        )
        .map_err(crate::errors::DatabaseError::Sqlite)?;

        let id = conn.last_insert_rowid();
        debug!("Recorded backup {} with ID {}", backup_path, id);
        Ok(id)
    }

    /// Gets the most recent backup record.
    ///
    /// # Returns
    ///
    /// Returns `None` if no backups have been recorded.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_last_backup(&self) -> AppResult<Option<BackupRecord>> {
        let conn = self.get_conn()?;
        let result = conn.query_row(
            "SELECT id, backup_path, backup_type, entries_count, archive_size, checksum, created_at
             FROM backup_log
             ORDER BY created_at DESC
             LIMIT 1",
            [],
            |row| {
                Ok(BackupRecord {
                    id: row.get(0)?,
                    backup_path: row.get(1)?,
                    backup_type: row.get(2)?,
                    entries_count: row.get(3)?,
                    archive_size: row.get(4)?,
                    checksum: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        );

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(crate::errors::DatabaseError::Sqlite(e).into()),
        }
    }

    /// Gets backup history with a limit.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of records to return
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn get_backup_history(&self, limit: usize) -> AppResult<Vec<BackupRecord>> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, backup_path, backup_type, entries_count, archive_size, checksum, created_at
                 FROM backup_log
                 ORDER BY created_at DESC
                 LIMIT ?",
            )
            .map_err(crate::errors::DatabaseError::Sqlite)?;

        let records = stmt
            .query_map([limit], |row| {
                Ok(BackupRecord {
                    id: row.get(0)?,
                    backup_path: row.get(1)?,
                    backup_type: row.get(2)?,
                    entries_count: row.get(3)?,
                    archive_size: row.get(4)?,
                    checksum: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(crate::errors::DatabaseError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(crate::errors::DatabaseError::Sqlite)?;

        debug!("Retrieved {} backup records", records.len());
        Ok(records)
    }
}

/// Record of a backup operation.
#[derive(Debug, Clone, PartialEq)]
pub struct BackupRecord {
    /// Unique identifier
    pub id: i64,
    /// Path to the backup archive
    pub backup_path: String,
    /// Type of backup ("full" or "incremental")
    pub backup_type: String,
    /// Number of entries in the backup
    pub entries_count: i64,
    /// Size of the archive in bytes
    pub archive_size: i64,
    /// BLAKE3 checksum of the archive
    pub checksum: String,
    /// Timestamp when backup was created
    pub created_at: String,
}

/// Connection customizer that sets the SQLCipher key pragma.
#[derive(Debug)]
struct SqlCipherConfig {
    passphrase: String,
}

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for SqlCipherConfig {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // Set the encryption key
        conn.pragma_update(None, "key", &self.passphrase)?;
        // Use modern SQLCipher defaults (version 4)
        conn.pragma_update(None, "cipher_page_size", 4096)?;
        conn.pragma_update(None, "kdf_iter", 256000)?;
        conn.pragma_update(None, "cipher_hmac_algorithm", "HMAC_SHA512")?;
        conn.pragma_update(None, "cipher_kdf_algorithm", "PBKDF2_HMAC_SHA512")?;
        Ok(())
    }

    fn on_release(&self, _conn: Connection) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_open_and_connect() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());

        let db = Database::open(&db_path, &passphrase).unwrap();
        let conn = db.get_conn().unwrap();

        // Should be able to execute a simple query
        let result: i32 = conn
            .query_row("SELECT 1 + 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase1 = SecretString::new("correct_password".to_string());
        let passphrase2 = SecretString::new("wrong_password".to_string());

        // Create database with first passphrase and write some data
        let db1 = Database::open(&db_path, &passphrase1).unwrap();
        db1.initialize_schema().unwrap();
        let conn = db1.get_conn().unwrap();
        conn.execute(
            "INSERT INTO entries (path, date, checksum, word_count) VALUES (?, ?, ?, ?)",
            ["test.md", "2024-01-01", "abc123", "100"],
        )
        .unwrap();
        drop(conn);
        drop(db1);

        // Try to open with wrong passphrase - should fail when trying to read data
        let result = Database::open(&db_path, &passphrase2);

        // SQLCipher may not fail immediately, so try to read the schema
        if let Ok(db2) = result {
            let conn_result = db2.get_conn();
            if let Ok(conn) = conn_result {
                let read_result: Result<i32, _> =
                    conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0));
                // Wrong passphrase should cause read to fail
                assert!(
                    read_result.is_err(),
                    "Expected database read to fail with wrong passphrase"
                );
            }
        }
    }

    #[test]
    fn test_initialize_schema_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());

        let db = Database::open(&db_path, &passphrase).unwrap();

        // Initialize schema twice - should not error
        db.initialize_schema().unwrap();
        db.initialize_schema().unwrap();
    }

    #[test]
    fn test_record_backup() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());

        let db = Database::open(&db_path, &passphrase).unwrap();

        // Record a backup
        let id = db
            .record_backup("backup.tar.gz.age", "full", 10, 1024, "abc123")
            .unwrap();

        assert!(id > 0);

        // Verify it was recorded
        let conn = db.get_conn().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM backup_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_last_backup() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());

        let db = Database::open(&db_path, &passphrase).unwrap();

        // Initially no backups
        let last = db.get_last_backup().unwrap();
        assert!(last.is_none());

        // Record two backups
        let id1 = db
            .record_backup("backup1.tar.gz.age", "full", 10, 1024, "abc123")
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1)); // Ensure different timestamps
        let id2 = db
            .record_backup("backup2.tar.gz.age", "incremental", 5, 512, "def456")
            .unwrap();

        assert!(id2 > id1);

        // Get last backup - should be the second one (highest ID)
        let last = db.get_last_backup().unwrap();
        assert!(last.is_some());
        let record = last.unwrap();
        assert_eq!(record.id, id2);
        assert_eq!(record.backup_path, "backup2.tar.gz.age");
        assert_eq!(record.backup_type, "incremental");
        assert_eq!(record.entries_count, 5);
        assert_eq!(record.archive_size, 512);
        assert_eq!(record.checksum, "def456");
    }

    #[test]
    fn test_get_backup_history() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let passphrase = SecretString::new("test_password".to_string());

        let db = Database::open(&db_path, &passphrase).unwrap();

        // Record three backups
        let id1 = db
            .record_backup("backup1.tar.gz.age", "full", 10, 1024, "abc123")
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let id2 = db
            .record_backup("backup2.tar.gz.age", "incremental", 5, 512, "def456")
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let id3 = db
            .record_backup("backup3.tar.gz.age", "incremental", 3, 256, "ghi789")
            .unwrap();

        assert!(id2 > id1);
        assert!(id3 > id2);

        // Get history with limit 2
        let history = db.get_backup_history(2).unwrap();
        assert_eq!(history.len(), 2);

        // Should be ordered by most recent first (highest ID)
        assert_eq!(history[0].id, id3);
        assert_eq!(history[0].backup_path, "backup3.tar.gz.age");
        assert_eq!(history[1].id, id2);
        assert_eq!(history[1].backup_path, "backup2.tar.gz.age");

        // Get all history
        let all_history = db.get_backup_history(10).unwrap();
        assert_eq!(all_history.len(), 3);
        assert_eq!(all_history[0].id, id3);
        assert_eq!(all_history[1].id, id2);
        assert_eq!(all_history[2].id, id1);
    }
}
