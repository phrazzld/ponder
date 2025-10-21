//! Performance benchmarks for crypto and database operations.
//!
//! Run with: cargo bench
//!
//! These benchmarks establish baseline performance metrics for:
//! - Encryption/decryption at various file sizes
//! - Vector similarity search at various database sizes

use age::secrecy::SecretString;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ponder::crypto::age::{decrypt_with_passphrase, encrypt_with_passphrase};
use ponder::db::embeddings::{insert_embedding, search_similar_chunks};
use ponder::db::entries::upsert_entry;
use ponder::db::Database;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Benchmark encryption performance with various payload sizes.
fn bench_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt");

    let passphrase = SecretString::new("benchmark-passphrase".to_string());
    let sizes = vec![("1KB", 1024), ("100KB", 100 * 1024), ("1MB", 1024 * 1024)];

    for (name, size) in sizes {
        let data = vec![b'x'; size];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &data, |b, data| {
            b.iter(|| {
                let encrypted = encrypt_with_passphrase(black_box(data), black_box(&passphrase))
                    .expect("encryption failed");
                black_box(encrypted);
            });
        });
    }

    group.finish();
}

/// Benchmark decryption performance with various payload sizes.
fn bench_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("decrypt");

    let passphrase = SecretString::new("benchmark-passphrase".to_string());
    let sizes = vec![("1KB", 1024), ("100KB", 100 * 1024), ("1MB", 1024 * 1024)];

    for (name, size) in sizes {
        let data = vec![b'x'; size];
        let encrypted =
            encrypt_with_passphrase(&data, &passphrase).expect("encryption failed for benchmark");

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &encrypted,
            |b, encrypted| {
                b.iter(|| {
                    let decrypted =
                        decrypt_with_passphrase(black_box(encrypted), black_box(&passphrase))
                            .expect("decryption failed");
                    black_box(decrypted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark vector similarity search with various database sizes.
fn bench_vector_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_search");

    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("bench.db");
    let passphrase = SecretString::new("benchmark-passphrase".to_string());

    // Setup: Create database and populate with entries
    let db = Database::open(&db_path, &passphrase).expect("open database");
    db.initialize_schema().expect("initialize schema");

    // Create sample embeddings (768-dim vectors)
    let sample_embedding: Vec<f32> = (0..768).map(|i| (i as f32) * 0.001).collect();
    let query_embedding: Vec<f32> = (0..768).map(|i| (i as f32) * 0.001 + 0.1).collect();

    let sizes = vec![("10_entries", 10), ("100_entries", 100)];

    for (name, num_entries) in sizes {
        let conn = db.get_conn().expect("get connection");

        // Populate database with embeddings
        for i in 0..num_entries {
            let entry_path = PathBuf::from(format!("/test/entries/{}.md.age", i));
            let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
                .unwrap()
                .checked_add_days(chrono::Days::new(i as u64))
                .unwrap();
            let checksum = format!("bench{}", i);

            // Insert entry first
            let entry_id =
                upsert_entry(&conn, &entry_path, date, &checksum, 100).expect("upsert entry");

            // Insert embedding
            let chunk_checksum = format!("chunk{}", i);
            insert_embedding(&conn, entry_id, 0, &sample_embedding, &chunk_checksum)
                .expect("insert embedding");
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &query_embedding,
            |b, query| {
                b.iter(|| {
                    let results =
                        search_similar_chunks(black_box(&conn), black_box(query), black_box(5))
                            .expect("search failed");
                    black_box(results);
                });
            },
        );

        // Cleanup entries for next iteration
        conn.execute("DELETE FROM entries", [])
            .expect("cleanup entries");
        conn.execute("DELETE FROM embeddings", [])
            .expect("cleanup embeddings");
    }

    group.finish();

    // Cleanup
    drop(db);
    fs::remove_file(&db_path).ok();
    temp_dir.close().expect("cleanup temp dir");
}

criterion_group!(benches, bench_encrypt, bench_decrypt, bench_vector_search);
criterion_main!(benches);
