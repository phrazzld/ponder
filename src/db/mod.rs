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
use tracing::{debug, info};

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
        conn.execute_batch("PRAGMA cipher_integrity_check")
            .map_err(crate::errors::DatabaseError::Sqlite)?;
        drop(conn);

        info!("Database opened successfully");
        Ok(Database { pool })
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
}
