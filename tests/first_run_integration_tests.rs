//! Integration tests for first-run database initialization.
//!
//! These tests verify that Database::open() properly initializes schema
//! on first run, preventing "no such table" errors.

use age::secrecy::SecretString;
use ponder::db::Database;
use tempfile::TempDir;

/// Test that opening a non-existent database creates and initializes it.
///
/// This is a regression test for the bug where Database::open() created
/// an empty database without tables, causing "no such table: entries" errors.
#[test]
fn test_first_run_auto_initializes_schema() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("first_run.db");
    let passphrase = SecretString::new("test-passphrase".to_string());

    // First run - database doesn't exist
    assert!(!db_path.exists(), "Database should not exist yet");

    // Opening should create AND initialize schema automatically
    let db = Database::open(&db_path, &passphrase).expect("open database on first run");

    // Database file should now exist
    assert!(db_path.exists(), "Database file should be created");

    // Should be able to insert into entries table immediately
    let conn = db.get_conn().expect("get connection");
    let result = conn.execute(
        "INSERT INTO entries (path, date, checksum, word_count) VALUES (?, ?, ?, ?)",
        ["test.md.age", "2025-01-01", "abc123", "10"],
    );

    assert!(
        result.is_ok(),
        "Should be able to insert into entries table on first run without manual initialize_schema(). Error: {:?}",
        result.err()
    );

    let row_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
        .expect("count entries");
    assert_eq!(row_count, 1, "Should have 1 entry after insert");

    temp_dir.close().expect("cleanup");
}

/// Test that opening an existing database works correctly.
#[test]
fn test_opening_existing_database_works() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("existing.db");
    let passphrase = SecretString::new("test".to_string());

    // First run - create database
    {
        let db = Database::open(&db_path, &passphrase).expect("create database");
        let conn = db.get_conn().expect("get connection");
        conn.execute(
            "INSERT INTO entries (path, date, checksum, word_count) VALUES (?, ?, ?, ?)",
            ["entry1.md.age", "2025-01-01", "hash1", "100"],
        )
        .expect("insert entry");
    }

    // Second run - open existing database
    let db = Database::open(&db_path, &passphrase).expect("open existing database");
    let conn = db.get_conn().expect("get connection");

    // Should be able to query existing data
    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
        .expect("count entries");
    assert_eq!(count, 1, "Should have 1 entry from first run");

    // Should be able to insert more data
    conn.execute(
        "INSERT INTO entries (path, date, checksum, word_count) VALUES (?, ?, ?, ?)",
        ["entry2.md.age", "2025-01-02", "hash2", "200"],
    )
    .expect("insert second entry");

    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
        .expect("count entries after insert");
    assert_eq!(count, 2, "Should have 2 entries after second insert");

    temp_dir.close().expect("cleanup");
}

/// Test that schema version is recorded on first run.
#[test]
fn test_schema_version_recorded_on_first_run() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("version_test.db");
    let passphrase = SecretString::new("test".to_string());

    // Create database
    let db = Database::open(&db_path, &passphrase).expect("open database");
    let conn = db.get_conn().expect("get connection");

    // Check schema version table exists
    let version: Option<i32> = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY applied_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    assert!(
        version.is_some(),
        "Schema version should be recorded on first run"
    );
    assert_eq!(
        version.unwrap(),
        ponder::db::schema::SCHEMA_VERSION,
        "Recorded version should match current schema version"
    );

    temp_dir.close().expect("cleanup");
}

/// Test that schema validation passes on a newly created database.
#[test]
fn test_schema_validation_passes_on_first_run() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("validation_test.db");
    let passphrase = SecretString::new("test".to_string());

    // Create database
    let db = Database::open(&db_path, &passphrase).expect("open database");

    // Validation should pass
    let validation_result = db.validate_schema();
    assert!(
        validation_result.is_ok(),
        "Schema validation should pass on first run. Error: {:?}",
        validation_result.err()
    );

    temp_dir.close().expect("cleanup");
}

/// Test that all required tables are created on first run.
#[test]
fn test_all_required_tables_created() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("tables_test.db");
    let passphrase = SecretString::new("test".to_string());

    // Create database
    let db = Database::open(&db_path, &passphrase).expect("open database");
    let conn = db.get_conn().expect("get connection");

    // Check that required tables exist
    let required_tables = vec![
        "entries",
        "embeddings",
        "entries_fts",
        "insights",
        "reports",
        "schema_version",
    ];

    for table in required_tables {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?)",
                [table],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| panic!("check if table '{}' exists", table));

        assert!(exists, "Table '{}' should be created on first run", table);
    }

    temp_dir.close().expect("cleanup");
}

/// Test that idempotent initialization doesn't duplicate data.
#[test]
fn test_initialize_schema_idempotent() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("idempotent_test.db");
    let passphrase = SecretString::new("test".to_string());

    // Create database
    let db = Database::open(&db_path, &passphrase).expect("open database");

    // Manually call initialize_schema multiple times
    db.initialize_schema()
        .expect("first initialize_schema call");
    db.initialize_schema()
        .expect("second initialize_schema call");
    db.initialize_schema()
        .expect("third initialize_schema call");

    // Should still have exactly one schema version record
    let conn = db.get_conn().expect("get connection");
    let version_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))
        .expect("count schema versions");

    assert_eq!(
        version_count, 1,
        "Should have exactly 1 schema version record even after multiple initialize calls"
    );

    temp_dir.close().expect("cleanup");
}
