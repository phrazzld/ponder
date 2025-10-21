# TODO: Ponder v2.0 - AI-Powered Encrypted Journaling

## Progress Summary

**Completed**: Phase 0-5 ✅ (crypto, db, AI, ops, CLI integration)
**Current**: Phase 6 (Testing & Documentation), Phase 7 (Backup System), Phase 8 (Migration System)
**Remaining**: Phase 6-8 tasks

**Tests**: 198 passing (3 ignored)
**Modules**: crypto/, db/, ai/, ops/ - all operational with subcommand CLI
**Architecture**: v2.0 complete - encrypted journaling with AI features

---

## Context

**Approach**: Modular addition of encryption, database, and AI capabilities while preserving existing v1.0 architecture

**Key Patterns**:
- Module structure: `src/{module}/mod.rs` with public API + `#[cfg(test)]` colocated tests
- Error handling: Derive `thiserror::Error`, wrap in `AppError` enum
- Config: Environment variables with `shellexpand` for paths
- Testing: Unit tests colocated, integration in `tests/`, use `tempfile` for isolation
- CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test -- --test-threads=1`

**Module Value Test**: Each new module must hide significant complexity behind simple interfaces

**Prerequisites**: Before implementing Phase 7-8, create manual backup: `cp -r ~/Documents/rubberducks ~/Documents/rubberducks-backup-$(date +%Y%m%d-%H%M%S)`

---

## ✅ Phase 0-5: COMPLETED

*(See existing content below for details)*

### Phase 0: Foundation
- ✅ Fixed RUSTSEC-2025-0055 (tracing-subscriber → 0.3.20)
- ✅ Fixed deprecated clap API
- ✅ Added all dependencies (crypto, db, AI, dev/test)

### Phase 1: Encryption Core
- ✅ crypto/age.rs: Age encryption with streaming support (206 LOC)
- ✅ crypto/session.rs: Session management with timeout (264 LOC)
- ✅ crypto/temp.rs: Secure tmpfs temp files (327 LOC)
- ✅ CryptoError enum + constants
- ✅ 17 unit tests + 11 integration tests

### Phase 2: Database Layer
- ✅ db/mod.rs: SQLCipher connection pooling (210 LOC)
- ✅ db/schema.rs: 5 tables (entries, embeddings, FTS, insights, reports)
- ✅ db/entries.rs: Entry CRUD with checksums (260 LOC)
- ✅ db/embeddings.rs: 768-dim vectors + cosine similarity (280 LOC)
- ✅ DatabaseError enum + constants
- ✅ 22 unit tests

### Phase 3: AI Integration
- ✅ ai/ollama.rs: HTTP client for embeddings/chat (250 LOC)
- ✅ ai/chunking.rs: Sliding window text chunking (110 LOC)
- ✅ ai/prompts.rs: System prompts + message builders (180 LOC)
- ✅ AIError enum with helpful suggestions
- ✅ AI constants (Ollama URL, models, chunk sizes)
- ✅ 18 unit tests

### Phase 4: High-Level Operations
- ✅ ops/edit.rs: Edit with encryption + embedding (217 LOC)
- ✅ ops/ask.rs: RAG query (140 LOC)
- ✅ ops/reflect.rs: Entry reflection (72 LOC)
- ✅ ops/search.rs: Semantic search (145 LOC)

### Phase 5: CLI Integration
- ✅ cli/mod.rs: Subcommand architecture (Edit, Ask, Reflect, Search, Lock)
- ✅ main.rs: Command dispatch with SessionManager + Database + OllamaClient
- ✅ config/mod.rs: v2.0 settings (db_path, session_timeout, ollama_url)

---

## Phase 6: Testing & Documentation (In Progress)

### Documentation Audit

- [x] **Fix model reference in README.md** (5min)
  ```
  File: README.md line 54
  Change: `ollama pull qwen2.5:3b` → `ollama pull gemma3:4b`
  Reason: Code uses gemma3:4b (src/constants.rs:146), not qwen2.5
  Success: Grep shows no qwen2.5 references in user-facing docs
  ```

- [x] **Fix model references in CLAUDE.md** (5min)
  ```
  Files: CLAUDE.md lines 71, 163
  Change: Replace all `qwen2.5:3b` with `gemma3:4b`
  Verify: `grep -n "qwen\|gemma" CLAUDE.md` shows only gemma3:4b
  Success: Model references consistent with DEFAULT_CHAT_MODEL constant
  ```

- [ ] **Add Backup & Export section to README.md** (30min)
  ```
  File: README.md
  Location: After "Configuration" section, before "File Structure"
  Content:
    - Backup Commands subsection (ponder backup, --incremental, --verify)
    - Export Commands subsection (export --format markdown/html, date ranges)
    - Restore from Backup subsection (ponder restore, --force flag)
    - Security warning for unencrypted exports
  Success: User can understand backup workflow without reading MIGRATION.md
  ```

- [ ] **Rewrite Migration Strategy section in MIGRATION.md** (45min)
  ```
  File: MIGRATION.md
  Replace: Manual migration instructions (Option 2)
  With: Automatic migration (Option 1) as primary, manual as fallback
  Add: Interactive migration flow example with user prompts
  Add: Resume capability documentation
  Add: Non-destructive guarantee (keeps v1.0 files)
  Success: User understands automatic detection on first run
  ```

- [ ] **Create docs/COMMANDS.md reference** (30min)
  ```
  File: docs/COMMANDS.md (new)
  Structure:
    - Core Commands: edit, ask, search, reflect, lock
    - Backup & Migration: backup, export, restore, cleanup-v1
    - Global Options: -v, --log-format, -h, -V
  Format: Command → Usage → Options → Examples
  Success: Complete command reference for all subcommands
  ```

### Comprehensive Testing

- [x] **Add security-focused integration tests** (2.5hr) ✅
  ```
  Files: tests/security_tests.rs
  Tests: 15 total (encryption, temp cleanup, permissions, timeouts, zeroization)
  Status: All passing
  ```

- [x] **Add performance benchmarks** (1.5hr)
  ```
  Files: benches/crypto_bench.rs (new)
  Dependencies: Add criterion = "0.5" to [dev-dependencies]
  Benchmarks:
    - encrypt_with_passphrase() with 1KB, 100KB, 1MB inputs
    - decrypt_with_passphrase() with same sizes
    - search_similar_chunks() with 10, 100, 1000 entry databases
  Success: Baseline metrics established for regression detection
  ```

### Quality Infrastructure

- [x] **Fix Ollama test dependency blocking CI** (2hr) **CRITICAL**
  ```
  Problem: ops_integration_tests.rs hangs in CI waiting for Ollama
  Solution: Already implemented in commit 5e0d91b (different approach)
  - Added CI/PONDER_TEST_PASSPHRASE env var checks in src/setup.rs:157
  - Skips model availability checks in test/CI environments
  - mockito = "1.2" already in Cargo.toml (for future use)
  - Current tests don't actually call Ollama (only test constructors)
  Status: CI passing (run 18663037261), all tests pass locally
  Work Log:
  - Investigated ops_integration_tests.rs - only 3 tests, none call Ollama
  - Checked CI history - most recent run succeeded in 4m25s
  - Found existing solution in src/setup.rs check_model_available()
  - Task was anticipatory or already solved differently
  ```

- [x] **Add secrets scanning with Gitleaks** (30min) **CRITICAL**
  ```
  Files: .pre-commit-config.yaml, .github/workflows/security.yml (new)
  Pre-commit:
    - Add repo: https://github.com/gitleaks/gitleaks
      hooks: gitleaks protect --staged --verbose --redact
  CI workflow:
    - Job: secrets-scan
    - Uses: gitleaks/gitleaks-action@v2
    - Run on: push, pull_request, schedule (weekly)
  Success: Commits with PONDER_TEST_PASSPHRASE or secrets blocked locally and in CI
  ```

- [x] **Configure Dependabot for dependency updates** (15min)
  ```
  File: .github/dependabot.yml (new)
  Config:
    package-ecosystem: cargo
    directory: /
    schedule: weekly
    open-pull-requests-limit: 5
    groups: { dev-dependencies: { dependency-type: development } }
  Success: Dependabot creates weekly PRs for outdated crates
  ```

- [x] **Add cargo-audit to CI** (20min)
  ```
  File: .github/workflows/ci.yml or security.yml
  Job: audit
    - Install: cargo install cargo-audit --locked
    - Run: cargo audit --deny warnings
    - Triggers: push, pull_request, schedule (weekly)
  Success: CI fails on HIGH/CRITICAL vulnerabilities from RustSec
  ```

---

## Phase 7: Backup & Export System (Est: 5-7hr)

### Dependencies & Schema

- [ ] **Add archive dependencies to Cargo.toml** (5min)
  ```
  File: Cargo.toml
  Add to [dependencies]:
    tar = "0.4"      # Cross-platform tar archive creation
    flate2 = "1.0"   # Gzip compression for archives
    walkdir = "2.4"  # Recursive directory traversal
  Reason: Need tar.gz creation, age crate handles encryption layer
  Success: cargo check passes, no version conflicts
  ```

- [ ] **Add backup tables to database schema** (15min)
  ```
  File: src/db/schema.rs in create_tables()
  Add:
    backup_log table: id, backup_path, backup_type (full|incremental),
      entries_count, archive_size, created_at, checksum (BLAKE3)
    backup_state table: id (singleton), last_backup_at, last_backup_checksum
  Add indexes: idx_backup_log_created_at DESC
  Success: In-memory DB test creates tables without error
  ```

- [ ] **Add database functions for backup tracking** (30min)
  ```
  File: src/db/mod.rs
  Functions:
    - record_backup(path, type, count, size, checksum) -> AppResult<i64>
    - get_last_backup() -> AppResult<Option<BackupRecord>>
    - get_backup_history(limit) -> AppResult<Vec<BackupRecord>>
  Success: Unit test records backup, retrieves by ID and timestamp
  ```

### Core Backup Module

- [ ] **Create src/ops/backup.rs module structure** (15min)
  ```
  File: src/ops/backup.rs (new)
  Structs:
    - pub struct BackupReport { total_entries, archive_size, checksum, duration }
    - pub struct BackupManifest { entries: Vec<PathBuf>, db_path: PathBuf }
  Functions (stubs for now):
    - pub fn create_backup(...) -> AppResult<BackupReport>
    - pub fn verify_backup(...) -> AppResult<BackupManifest>
  Success: Module compiles, stubs return dummy data
  ```

- [ ] **Implement create_backup() for full encrypted archive** (2hr)
  ```
  File: src/ops/backup.rs
  Flow:
    1. Use walkdir to collect all .md.age files from journal_dir
    2. Include ponder.db database file
    3. Create tar::Builder in temp location with flate2::GzEncoder
    4. Add each file with relative paths (YYYY/MM/DD.md.age)
    5. Finish tar, get compressed bytes
    6. Encrypt tar.gz with encrypt_with_passphrase() from crypto::age
    7. Write to output_path with .age extension
    8. Calculate BLAKE3 checksum of encrypted archive
    9. Record in backup_log via db.record_backup()
  Error handling: File not found, disk full, encryption failure
  Success: Creates encrypted archive, restores to identical directory tree
  ```

- [ ] **Implement verify_backup() for integrity checking** (1hr)
  ```
  File: src/ops/backup.rs
  Flow:
    1. Read encrypted archive from path
    2. Decrypt with decrypt_with_passphrase()
    3. Extract tar.gz to temp directory (use tempfile::TempDir)
    4. Verify database file exists and can be opened with passphrase
    5. Count .md.age files, collect paths
    6. Cleanup temp directory (RAII via TempDir drop)
    7. Return BackupManifest with file list
  Success: Valid backup returns manifest, corrupted backup returns error
  ```

- [ ] **Implement restore_backup() for extraction** (1.5hr)
  ```
  File: src/ops/backup.rs
  Flow:
    1. Call verify_backup() to validate archive
    2. Check if target_dir exists, error if not force flag
    3. Extract to temp location first (atomic operation)
    4. Move files from temp to target_dir (overwrites if force)
    5. Verify database opens with provided passphrase
    6. Return RestoreReport { entries_restored, db_size }
  Safety: Never partially restore (atomic via temp → move)
  Success: Restores all files, database connection succeeds
  ```

- [ ] **Implement incremental backup** (1hr)
  ```
  File: src/ops/backup.rs
  Function: create_incremental_backup(...)
  Flow:
    1. Query backup_state for last_backup_at timestamp
    2. Filter entries with updated_at > last_backup_at
    3. Create tar with only changed files
    4. Encrypt and write archive
    5. Update backup_state with new timestamp
  Success: Second incremental backup includes only files changed since first
  ```

### Export Module

- [ ] **Create src/ops/export.rs module structure** (10min)
  ```
  File: src/ops/export.rs (new)
  Structs:
    - pub struct ExportReport { entries_count, output_size, format }
  Functions (stubs):
    - pub fn export_plaintext(...) -> AppResult<ExportReport>
    - pub fn export_html(...) -> AppResult<ExportReport>
  Success: Module compiles with stub implementations
  ```

- [ ] **Implement export_plaintext() for markdown export** (1hr)
  ```
  File: src/ops/export.rs
  Flow:
    1. Query database for entries in date_range (or all if None)
    2. For each entry:
       - Decrypt .md.age file to plaintext
       - Create output path: output_dir/YYYY/MM/DD.md
       - Create parent directories with fs::create_dir_all()
       - Write plaintext markdown to output file
    3. Create output_dir/index.md with table of contents
    4. Return ExportReport with count
  Warning: Log warning that plaintext export is unencrypted
  Success: Export → import → content matches original
  ```

- [ ] **Implement export_html() for searchable HTML export** (2hr)
  ```
  File: src/ops/export.rs
  Flow:
    1. Query and decrypt all entries in date_range
    2. Generate HTML structure:
       - <!DOCTYPE html> with embedded CSS
       - Navigation sidebar with dates (links to #date-YYYYMMDD anchors)
       - Main content area with entries as sections
       - JavaScript for search functionality (filter by text)
    3. Use markdown crate to convert MD → HTML for entry content
    4. Write self-contained HTML file (no external resources)
  Dependencies: Add markdown = "1.0" to Cargo.toml
  Success: HTML opens in browser, search filters entries, all dates accessible
  ```

### CLI Integration

- [ ] **Add backup/export commands to CLI** (30min)
  ```
  File: src/cli/mod.rs
  Add to PonderCommand enum:
    - Backup(BackupArgs)
    - Export(ExportArgs)
    - Restore(RestoreArgs)
  New structs:
    - BackupArgs { output: PathBuf, incremental: bool, verify: bool }
    - ExportArgs { output: PathBuf, format: String, from: Option<String>, to: Option<String> }
    - RestoreArgs { backup: PathBuf, force: bool }
  Success: cargo check passes, new commands in --help output
  ```

- [ ] **Implement cmd_backup() handler** (30min)
  ```
  File: src/main.rs
  Function: cmd_backup(config, args) -> AppResult<()>
  Flow:
    1. Initialize SessionManager (need passphrase for encryption)
    2. Initialize Database (for backup tracking)
    3. Call ops::create_backup() or create_incremental_backup()
    4. If args.verify, call ops::verify_backup()
    5. Print BackupReport (entries count, size, checksum)
  Success: `ponder backup test.tar.gz.age` creates encrypted archive
  ```

- [ ] **Implement cmd_export() handler** (30min)
  ```
  File: src/main.rs
  Function: cmd_export(config, args) -> AppResult<()>
  Flow:
    1. Initialize SessionManager + Database
    2. Parse date_range from args.from/args.to
    3. Match args.format:
       - "markdown" → ops::export_plaintext()
       - "html" → ops::export_html()
    4. Print warning about unencrypted export
    5. Print ExportReport (count, output path)
  Success: `ponder export out/ --format markdown` creates plaintext files
  ```

- [ ] **Implement cmd_restore() handler** (20min)
  ```
  File: src/main.rs
  Function: cmd_restore(config, args) -> AppResult<()>
  Flow:
    1. Initialize SessionManager (for decryption)
    2. Call ops::restore_backup(args.backup, config.journal_dir, args.force)
    3. Print RestoreReport (entries restored, database size)
  Validation: Prompt for confirmation if target directory exists and !force
  Success: `ponder restore backup.tar.gz.age` restores all files
  ```

- [ ] **Update src/ops/mod.rs to expose backup/export** (5min)
  ```
  File: src/ops/mod.rs
  Add:
    pub mod backup;
    pub mod export;
  Re-export:
    pub use backup::{create_backup, verify_backup, restore_backup};
    pub use export::{export_plaintext, export_html};
  Success: Other modules can import with `use ponder::ops::backup::*`
  ```

### Testing

- [ ] **Create tests/backup_tests.rs** (2hr)
  ```
  File: tests/backup_tests.rs (new)
  Tests:
    - test_create_full_backup: 3 entries → archive → verify size
    - test_verify_backup: Create backup → verify → check manifest
    - test_restore_backup: Create backup → delete original → restore → compare
    - test_incremental_backup: Full backup → modify 1 entry → incremental → check size
    - test_backup_with_wrong_passphrase: Verify fails with incorrect passphrase
    - test_export_plaintext: Encrypt 3 entries → export → compare content
    - test_export_html: Export → check HTML structure, search functionality
  Success: All 7 tests pass, coverage >80% for backup/export modules
  ```

---

## Phase 8: Migration System (Est: 6-8hr)

### Detection Logic

- [ ] **Create src/ops/detection.rs module** (30min)
  ```
  File: src/ops/detection.rs (new)
  Structs:
    - pub struct V1Entry { path: PathBuf, date: NaiveDate }
    - pub struct MigrationState { v1_entries_found: usize, v2_exists: bool, unmigrated: Vec<V1Entry> }
  Functions:
    - pub fn scan_v1_entries(journal_dir) -> AppResult<Vec<V1Entry>>
    - pub fn is_migrated(db, v1_path) -> AppResult<bool>
    - pub fn detect_migration_state(journal_dir, db) -> AppResult<MigrationState>
  Success: Module compiles, stubs return empty/default values
  ```

- [ ] **Implement scan_v1_entries() to detect plaintext files** (45min)
  ```
  File: src/ops/detection.rs
  Flow:
    1. Use glob pattern "journal_dir/*.md" (files in root, not subdirs)
    2. Filter filenames matching YYYYMMDD.md (8-digit date pattern)
    3. Parse date from filename with NaiveDate::parse_from_str(filename, "%Y%m%d")
    4. Verify file is readable (skip if permission denied)
    5. Return Vec<V1Entry> sorted by date ascending
  Edge cases: Ignore .md.age files, ignore subdirectories, skip invalid dates
  Success: Test directory with 10 v1.0 + 3 v2.0 files returns 10 entries
  ```

- [ ] **Implement is_migrated() to check migration log** (20min)
  ```
  File: src/ops/detection.rs
  Flow:
    1. Query migration_log table: SELECT status FROM migration_log WHERE v1_path = ?
    2. Return true if status = 'completed'
    3. Return false if not found or status != 'completed'
  Success: After migrating entry, is_migrated() returns true
  ```

- [ ] **Implement detect_migration_state() for complete analysis** (30min)
  ```
  File: src/ops/detection.rs
  Flow:
    1. Call scan_v1_entries() to find all plaintext files
    2. Check if any v2.0 files exist (glob "journal_dir/*/*/*.md.age")
    3. For each v1_entry, call is_migrated() to check log
    4. Build MigrationState with counts and unmigrated list
  Return: Complete picture of migration status
  Success: Mixed v1/v2 directory returns accurate counts for each category
  ```

### Migration Schema & Database

- [ ] **Add migration tables to database schema** (20min)
  ```
  File: src/db/schema.rs in create_tables()
  Add:
    migration_log table: id, v1_path (unique), v2_path, date, migrated_at,
      status (pending|completed|failed), error_message, checksum_match (bool)
    migration_state table: id (singleton), started_at, completed_at, total_entries,
      migrated_count, failed_count, mode (batch|lazy|manual)
  Indexes: idx_migration_status, idx_migration_date
  Success: Schema creates tables, foreign keys enforced
  ```

- [ ] **Add database functions for migration tracking** (45min)
  ```
  File: src/db/mod.rs
  Functions:
    - init_migration_state(total, mode) -> AppResult<()>
    - record_migration(v1_path, v2_path, date, status, error) -> AppResult<()>
    - update_migration_progress(completed, failed) -> AppResult<()>
    - finalize_migration() -> AppResult<()>
    - get_migration_progress() -> AppResult<Option<MigrationProgress>>
    - get_completed_migrations() -> AppResult<Vec<PathBuf>>
  Success: Unit tests verify singleton pattern for migration_state, log records persisted
  ```

### Migration Engine

- [ ] **Create src/ops/migration.rs module structure** (20min)
  ```
  File: src/ops/migration.rs (new)
  Structs:
    - pub struct MigrateOptions { skip_embeddings: bool, batch_size: usize }
    - pub struct MigrationResult { v1_path, v2_path, date, word_count, verified: bool }
    - pub struct MigrationReport { total, completed, failed: Vec<(PathBuf, String)>, duration }
  Functions (stubs):
    - pub fn migrate_entry(...) -> AppResult<MigrationResult>
    - pub fn migrate_all_entries<F>(..., callback: F) -> AppResult<MigrationReport>
  Success: Module compiles with placeholder implementations
  ```

- [ ] **Implement migrate_entry() for single entry migration** (2hr)
  ```
  File: src/ops/migration.rs
  Flow:
    1. Read v1.0 plaintext content from v1_entry.path
    2. Determine v2.0 path using get_encrypted_entry_path(journal_dir, date)
    3. Create parent directories with fs::create_dir_all()
    4. Encrypt content with encrypt_with_passphrase(content, passphrase)
    5. Write encrypted bytes to v2_path
    6. Calculate BLAKE3 checksum of encrypted file
    7. Count words in plaintext: content.split_whitespace().count()
    8. Insert into database: upsert_entry(conn, v2_path, date, checksum, word_count)
    9. Generate embeddings unless skip_embeddings flag set
    10. Verify: decrypt v2_path and compare with v1 content
    11. Record in migration_log with status based on verification result
  Error handling: Read failure, encryption failure, DB error (log and continue)
  Success: v1.0 entry → encrypted v2.0 entry, DB updated, embeddings generated
  ```

- [ ] **Implement verify_migration() for content comparison** (30min)
  ```
  File: src/ops/migration.rs
  Function: verify_migration(v1_path, v2_path, passphrase) -> AppResult<bool>
  Flow:
    1. Read v1 plaintext content
    2. Read v2 encrypted file, decrypt with passphrase
    3. Compare byte-by-byte (or string equality after normalization)
    4. Return true if identical, false if mismatch
  Success: Migrated entry verifies as identical, corrupted file returns false
  ```

- [ ] **Implement migrate_all_entries() batch processor** (2hr)
  ```
  File: src/ops/migration.rs
  Flow:
    1. Initialize migration_state in database with total count
    2. For each v1_entry in Vec:
       - Call progress_callback(current, total, filename) for UI updates
       - Call migrate_entry() for single entry
       - On success: increment completed, update DB progress
       - On error: push to failed Vec with error message
    3. Calculate duration (started_at → completed_at)
    4. Finalize migration_state in database
    5. Return MigrationReport with counts and failures
  Callback: Generic F: Fn(usize, usize, &str) for progress tracking
  Success: 100 entries migrate with progress updates, report shows 98 completed + 2 failed
  ```

### CLI Integration

- [ ] **Add migration flags to EditArgs** (10min)
  ```
  File: src/cli/mod.rs in EditArgs struct
  Add fields:
    - #[clap(long)] pub migrate: bool  // Force migration prompt
    - #[clap(long)] pub skip_embeddings: bool  // Faster migration
  Success: `ponder edit --migrate --skip-embeddings` accepted by parser
  ```

- [ ] **Add CleanupV1 command to PonderCommand** (5min)
  ```
  File: src/cli/mod.rs
  Add to PonderCommand enum:
    - CleanupV1  // No arguments needed (operates on journal_dir)
  Success: `ponder cleanup-v1` recognized as valid command
  ```

- [ ] **Integrate migration detection into cmd_edit()** (2hr)
  ```
  File: src/main.rs in cmd_edit()
  After ensure_journal_directory_exists(), before editing:
    1. Call detect_migration_state(journal_dir, db)
    2. If v1_entries_found > 0 && !v2_exists:
       - Print: "✨ Detected {count} plaintext journal entries!"
       - Call prompt_migration() for user choice
       - If yes: call migrate_all_entries() with print_progress callback
       - Print migration report (success/failure counts)
       - Recommend cleanup-v1 command
    3. Else if v1_entries_found > 0 && migration_in_progress:
       - Print: "⚠️ {unmigrated} entries still need migration"
  User experience: Non-blocking prompt, clear feedback, actionable next steps
  Success: First run with 1000 v1.0 files prompts migration, shows progress
  ```

- [ ] **Implement prompt_migration() helper** (30min)
  ```
  File: src/main.rs
  Function: prompt_migration() -> bool
  UI:
    1. Print migration explanation (encrypt, embed, preserve originals)
    2. Print options: [Y] all now, [s] on-demand, [n] skip
    3. Read stdin input
    4. Return true for 'y', false for 'n' or 's'
  Success: User can choose migration strategy interactively
  ```

- [ ] **Implement print_progress() helper** (20min)
  ```
  File: src/main.rs
  Function: print_progress(current: usize, total: usize, filename: &str)
  UI:
    1. Calculate percentage: (current * 100) / total
    2. Estimate ETA based on elapsed time (if >5 entries processed)
    3. Print: "[████░░░░] 45/100 (45%) - ETA: 2h 15m"
    4. Print: "  ✓ 2020-01-13: encrypted + embedded (127 words)"
  Use: Unicode block chars for progress bar, \r to overwrite line
  Success: Migration shows live progress with ETA
  ```

- [ ] **Implement cmd_cleanup_v1() handler** (45min)
  ```
  File: src/main.rs
  Function: cmd_cleanup_v1(config) -> AppResult<()>
  Flow:
    1. Initialize Database to check migration_log
    2. Scan for v1.0 files in journal_dir root
    3. For each v1_file:
       - Check if migration_log shows status='completed'
       - If completed: add to deletion list
       - If not migrated: warn user
    4. Prompt for confirmation: "Delete {count} v1.0 files?"
    5. If confirmed: delete files, print count deleted
  Safety: Only delete if migration_log confirms successful migration
  Success: Cleanup removes only successfully migrated files
  ```

- [ ] **Update src/ops/mod.rs to expose migration modules** (5min)
  ```
  File: src/ops/mod.rs
  Add:
    pub mod detection;
    pub mod migration;
  Re-export:
    pub use detection::{scan_v1_entries, detect_migration_state};
    pub use migration::{migrate_entry, migrate_all_entries};
  Success: Import with `use ponder::ops::migration::*` works
  ```

### Testing

- [ ] **Create tests/migration_tests.rs** (2hr)
  ```
  File: tests/migration_tests.rs (new)
  Tests:
    - test_detect_v1_entries: Create 10 YYYYMMDD.md files → scan → verify count
    - test_migrate_single_entry: v1.0 file → migrate → verify v2.0 encrypted
    - test_migration_verification: Migrate → decrypt → compare with original
    - test_migration_resume: Migrate 5, fail on #3, resume → skip 1-2, retry 3-5
    - test_skip_embeddings_flag: Migrate with flag → verify no embeddings in DB
    - test_incremental_migration: Migrate 3, add 2 more v1.0 → detect unmigrated
    - test_cleanup_v1_safety: Cleanup → verify only migrated files deleted
  Success: All 7 tests pass, migration workflow robust to interruption
  ```

- [ ] **Add migration integration test to ops_integration_tests.rs** (1hr)
  ```
  File: tests/ops_integration_tests.rs
  Test: test_full_migration_workflow
  Flow:
    1. Create temp directory with 5 v1.0 entries (2020-01-13 to 2020-01-17)
    2. Initialize v2.0 database and session
    3. Call migrate_all_entries() with mock OllamaClient
    4. Verify: 5 v2.0 files created, all encrypted, DB has 5 entries
    5. Verify: Each entry decrypts to original plaintext content
    6. Test semantic search works on migrated entries
  Success: End-to-end migration validates data integrity and AI features
  ```

---

## Phase 6 Continued: Remaining Tasks

### Quality Infrastructure

- [ ] **Add performance benchmarks** (1.5hr)
  ```
  Files: benches/crypto_bench.rs
  Add criterion to dev-dependencies
  Benchmarks:
    - File encryption/decryption (various sizes)
    - Vector search (various DB sizes)
    - Embedding generation (mock)
  ```

- [ ] **Add coverage tracking with cargo-tarpaulin** (1hr)
  ```
  Files: .github/workflows/ci.yml, .gitignore (add coverage/)
  Installation: cargo install cargo-tarpaulin
  Run command: cargo tarpaulin --out Html --exclude-files tests/ --output-dir coverage
  Set coverage floors for critical modules:
    --fail-under 90 (for src/crypto/)
    --fail-under 80 (for src/db/)
    --fail-under 75 (for src/ops/)
  Optional: Upload to Coveralls or Codecov
  Success criteria: Coverage report generated, critical paths meet thresholds
  ```

- [ ] **Categorize tests by speed/dependencies** (1.5hr)
  ```
  Files: All test files (tests/*.rs, src/*/mod.rs #[cfg(test)])
  Add attributes:
    #[test] - Fast unit tests (no I/O, no external deps)
    #[test]
    #[ignore = "integration"] - Integration tests (I/O, no Ollama)
    #[test]
    #[ignore = "e2e"] - E2E tests (requires Ollama)
  Update CI:
    - Default: cargo test (fast unit tests only)
    - Integration: cargo test -- --ignored --skip e2e
    - E2E: cargo test -- --ignored (only with Ollama setup)
  Success criteria: cargo test <2min, full suite with --ignored <5min
  ```

- [x] **Move cargo clippy to pre-push hook** (15min)
  ```
  Files: .pre-commit-config.yaml
  Problem: clippy on every commit slows workflow (5-10s)
  Solution:
    - Remove clippy from pre-commit hooks
    - Add to pre-push hooks instead
    - Keeps commits fast (<2s), catches issues before CI
  Pre-commit (fast): cargo fmt, gitleaks only
  Pre-push (thorough): cargo clippy, cargo test --lib
  Success criteria: Commits <2s, push <30s with checks
  ```

- [x] **Set up git-cliff for automated changelog** (45min)
  ```
  Files: cliff.toml (new), .github/workflows/release.yml (optional)
  Installation: cargo install git-cliff
  Configuration (cliff.toml):
    conventional_commits = true
    commit_parsers:
      - { message = "^feat", group = "Features" }
      - { message = "^fix", group = "Bug Fixes" }
      - { message = "^perf", group = "Performance" }
  Generate: git cliff --output CHANGELOG.md
  Automate: Add to release workflow or git tag hook
  Success criteria: CHANGELOG.md generated from commit history
  ```

- [ ] **Add security.yml workflow** (30min)
  ```
  Files: .github/workflows/security.yml (new)
  Jobs:
    1. secrets: gitleaks/gitleaks-action@v2 (scan git history)
    2. audit: cargo audit --deny warnings (vulnerabilities)
  Triggers: push, pull_request, schedule (weekly)
  Success criteria: Automated security scanning on all PRs and weekly
  ```

---

## Design Iteration Checkpoints

### After Phase 7 (Backup System)
- Review: Is backup/export API too complex? Should we simplify flags?
- Consider: Background backup scheduling (cron-like)?
- Refactor: Extract common archive creation pattern?

### After Phase 8 (Migration System)
- Review: Is automatic detection too aggressive? Should we add cooldown?
- Consider: Migration progress persistence (survive process crash)?
- Plan: Incremental embedding generation for large backlogs?

---

## Validation Checklist (Before v2.1 Release)

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --verbose -- --test-threads=1` all pass
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo audit` clean (no vulnerabilities)
- [ ] All integration tests pass
- [ ] Security tests pass
- [ ] Backup/restore tested with real journal (small subset)
- [ ] Migration tested with 10+ v1.0 entries
- [ ] README accurate with backup/migration sections
- [ ] MIGRATION.md updated with automatic detection flow
- [ ] docs/COMMANDS.md complete with all subcommands
- [ ] Performance benchmarks establish baseline metrics
- [ ] Ollama integration tested manually (real models)

---

## Time Estimates

**Completed**:
- Phase 0-5: ~10hr actual
- Phase 6 (partial): ~3hr

**Remaining**:
- Phase 6 (docs + quality): ~4hr
- Phase 7 (backup): ~7hr
- Phase 8 (migration): ~8hr

**Total Completed**: ~13hr
**Total Remaining**: ~19hr
**Total Project**: ~32hr

**Critical Path**: Phase 6 docs → Phase 7 backup → Phase 8 migration

---

## BACKLOG (Post-v2.1)

These are validated ideas not blocking current release:

- Setup wizard (`ponder setup` command) - interactive first-time config
- Nightly indexing (`ponder nightly`) - batch re-embedding for backfill
- Weekly review (`ponder weekly`) - aggregated insights from past week
- Keyring integration - store passphrase in system keyring (macOS Keychain, GNOME Keyring)
- X25519 identity support - SSH-key-based encryption (age identities)
- Hybrid search - combine vector + FTS for better results
- PDF export - requires pandoc dependency, lower priority than HTML
- Statistics - streak tracking, word counts, writing trends over time
- Git integration - auto-commit on save, versioned journal history
- Cloud sync - encrypted remote backup (careful with privacy)
