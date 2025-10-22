# TODO: Ponder v2.0 - AI-Powered Encrypted Journaling

## Progress Summary

**Completed**: Phase 0-5 ✅ (crypto, db, AI, ops, CLI)
**Current**: Phase 6 (docs), Phase 7 (backup system - core done, CLI/testing/refactoring remaining)
**Next**: Complete Phase 7 refinements → Phase 8 (migration)

**Tests**: 198 passing (3 ignored) | **Build**: ❌ BLOCKED (dependency issue)
**Architecture**: v2.0 foundation complete - encrypted journaling with AI features operational

---

## 🚨 CRITICAL BLOCKERS

### 1. Build Failure: tempfile Dependency Misconfiguration
**Impact**: Cannot compile, blocks all development
**Issue**: `tempfile` used in `src/ops/backup.rs:301,438` but only in `[dev-dependencies]`
**Fix**: Move to `[dependencies]` in Cargo.toml
```toml
[dependencies]
tempfile = "3.3"  # Required for backup/restore temp operations
```
**Time**: 2 minutes
**Priority**: CRITICAL - fix immediately

### 2. YAGNI Violation: Unused Incremental Backup Parameter
**Impact**: Confusing API, implementation promises undelivered functionality
**Issue**: `create_backup(..., _incremental: bool)` parameter unused, suppressed with `_` prefix
**Fix**: Remove parameter entirely, defer incremental until actually implemented
**Affected**: `src/ops/backup.rs:86`, CLI args planning
**Time**: 15 minutes
**Priority**: HIGH - clean up before adding CLI layer

---

## Phase 6: Testing & Documentation (Remaining)

### Documentation
- [ ] Add Backup & Export section to README.md (30min)
  - Location: After "Configuration", before "File Structure"
  - Cover: `backup`, `export`, `restore` commands with examples
  - Warn: Unencrypted export security implications

- [ ] Rewrite MIGRATION.md migration strategy (45min)
  - Flip: Automatic detection (primary) vs manual (fallback)
  - Add: Interactive flow examples, resume capability, non-destructive guarantee

- [ ] Create docs/COMMANDS.md reference (30min)
  - Comprehensive command reference: edit, ask, search, reflect, lock, backup, export, restore, cleanup-v1
  - Format: Usage → Options → Examples

### Quality Infrastructure
- [ ] Add security.yml workflow (30min)
  - Jobs: gitleaks history scan, cargo-audit vulnerabilities
  - Triggers: push, PR, weekly schedule

---

## Phase 7: Backup & Archive System (Core ✅, Refactoring Required)

### Completed
- ✅ Dependencies (tar, flate2, walkdir)
- ✅ Database schema (backup_log, backup_state tables)
- ✅ Database functions (record_backup, get_last_backup, get_backup_history)
- ✅ Module structure (BackupReport, BackupManifest, RestoreReport)
- ✅ `create_backup()` - Full encrypted archive creation
- ✅ `verify_backup()` - Integrity checking with manifest
- ✅ `restore_backup()` - Atomic extraction

### Critical Refactoring (Based on Ultrathink Analysis)

#### A. Fix Build Blocker (5min) - IMMEDIATE
- Move `tempfile` from dev-dependencies → dependencies
- Verify: `cargo check` succeeds

#### B. Remove Incremental API (15min) - BEFORE CLI INTEGRATION
- Delete `_incremental: bool` parameter from `create_backup()`
- Remove hardcoded `backup_type = "full"` logic (line 216)
- Update function docs to reflect full-backup-only behavior
- **Rationale**: Don't design interfaces for unimplemented features (YAGNI)

#### C. Extract Temporal Decomposition (2hr) - RECOMMENDED
**Problem**: `create_backup()` organized by execution steps, not functionality
**Fix**: Extract testable helpers
```rust
fn collect_backup_files(journal_dir: &Path, db_path: &Path) -> AppResult<Vec<PathBuf>>
fn create_tar_archive(files: &[PathBuf], base_dir: &Path) -> AppResult<Vec<u8>>
fn encrypt_and_write(data: &[u8], output: &Path, passphrase: &str) -> AppResult<String>
```
**Benefit**: Isolated testing, clearer intent, change isolation
**Optional**: Can defer to post-v2.1

#### D. Unified Archive System (3hr) - STRATEGIC
**Problem**: Planned `export.rs` will duplicate 80% of backup logic (directory walking, file collection, archival)
**Solution**: Strategy pattern for output format
```rust
pub enum ArchiveFormat {
    Encrypted,           // Current backup behavior
    PlaintextMarkdown,   // Flat directory export
    StaticHTML,          // Self-contained HTML with search
}

pub fn create_archive(
    format: ArchiveFormat,
    session: Option<&mut SessionManager>,  // Only for Encrypted
    journal_dir: &Path,
    output_path: &Path,
) -> AppResult<ArchiveReport>
```
**Benefit**: Eliminates 200+ LOC duplication, future formats (PDF, JSON) trivial
**Tradeoff**: Slightly more complex core vs. two duplicate modules
**Recommendation**: Implement before adding export CLI

#### E. Schema Simplification (30min) - OPTIONAL
- Consider removing `backup_state` table (singleton pattern complexity)
- State derivable via `SELECT MAX(created_at) FROM backup_log`
- Only implement if singleton creates maintenance issues

#### F. Report Symmetry (15min) - POLISH
- Add `checksum: String` and `duration: Duration` to `RestoreReport`
- Matches `BackupReport` structure for consistency

### CLI Integration (After Refactoring)

- [ ] Add Backup/Export/Restore commands to cli/mod.rs (30min)
  - `PonderCommand::Backup(BackupArgs)` with `output: PathBuf`, `verify: bool`
  - `PonderCommand::Export(ExportArgs)` with `format: String`, date range filters
  - `PonderCommand::Restore(RestoreArgs)` with `backup: PathBuf`, `force: bool`

- [ ] Implement cmd_backup() handler in main.rs (30min)
  - Initialize SessionManager + Database
  - Call `ops::create_backup()`
  - Optional: `ops::verify_backup()` if `args.verify`
  - Print BackupReport summary

- [ ] Implement cmd_export() handler (45min)
  - Match on format: "markdown" | "html"
  - Warn user about unencrypted output
  - Call unified `create_archive()` with appropriate format

- [ ] Implement cmd_restore() handler (20min)
  - Confirm overwrite if target exists (unless `--force`)
  - Call `ops::restore_backup()`
  - Print RestoreReport summary

- [ ] Expose backup/export in ops/mod.rs (5min)

### Testing

- [ ] Create tests/backup_tests.rs (2hr)
  - test_create_full_backup: Create → verify size/checksum
  - test_verify_backup: Create → verify → check manifest accuracy
  - test_restore_backup: Create → delete original → restore → compare
  - test_backup_wrong_passphrase: Verify fails with incorrect passphrase
  - test_export_plaintext: Encrypt entries → export → verify content match
  - test_export_html: Export → validate HTML structure and search functionality

---

## Phase 8: Migration System (Est: 6-8hr)

### Detection Logic (1.5hr)
- [ ] Create `src/ops/detection.rs`
  - `scan_v1_entries()`: Glob `*.md` (root only), parse YYYYMMDD.md dates
  - `is_migrated()`: Query migration_log for completion status
  - `detect_migration_state()`: Full analysis of v1/v2 mix

### Migration Schema (1hr)
- [ ] Add migration tables to db/schema.rs
  - `migration_log`: v1_path, v2_path, date, status, checksum_match
  - `migration_state`: started_at, completed_at, total/migrated/failed counts
- [ ] Add migration tracking functions to db/mod.rs
  - `record_migration()`, `update_migration_progress()`, `get_migration_progress()`

### Migration Engine (4hr)
- [ ] Create `src/ops/migration.rs`
  - `migrate_entry()`: Read v1 plaintext → encrypt → write v2 → embed → verify
  - `verify_migration()`: Decrypt v2, compare with v1 byte-for-byte
  - `migrate_all_entries()`: Batch processor with progress callback

### CLI Integration (2hr)
- [ ] Add `--migrate` flag to EditArgs for forced migration prompt
- [ ] Add `CleanupV1` command for safe deletion of migrated files
- [ ] Integrate detection into `cmd_edit()`: Auto-prompt on first v1.0 detection
- [ ] Implement `prompt_migration()`: Interactive choice (all now / on-demand / skip)
- [ ] Implement `print_progress()`: Live progress bar with ETA
- [ ] Implement `cmd_cleanup_v1()`: Delete only verified-migrated files

### Testing (3hr)
- [ ] Create tests/migration_tests.rs
  - test_detect_v1_entries, test_migrate_single_entry, test_migration_verification
  - test_migration_resume, test_skip_embeddings_flag, test_cleanup_v1_safety
- [ ] Add full migration workflow to ops_integration_tests.rs

---

## Validation Checklist (Before v2.1 Release)

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --verbose -- --test-threads=1` all pass
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo audit` clean
- [ ] Manual QA: backup → restore → verify integrity
- [ ] Manual QA: migration with 10+ v1.0 entries
- [ ] Documentation: README backup section, MIGRATION.md updated, COMMANDS.md complete
- [ ] Performance benchmarks baseline established

---

## Time Estimates

**Phase 6 Remaining**: ~2hr (docs)
**Phase 7 Remaining**: ~4hr (refactoring + CLI + tests) or ~7hr (with unified archive system)
**Phase 8**: ~8hr (migration system)

**Critical Path**:
1. Fix tempfile dependency (2min) ← **IMMEDIATE**
2. Remove incremental API (15min) ← **BEFORE CLI**
3. Decide: Quick path (4hr) vs Strategic path with unified archive (7hr)
4. Complete Phase 7 → Phase 8 → Final validation

**Total Remaining**: ~14hr (quick) or ~17hr (strategic)

---

## BACKLOG (Post-v2.1)

Validated ideas deferred to future releases:
- Setup wizard (`ponder setup`)
- Nightly indexing for embedding backfill
- Weekly review aggregation
- Keyring integration (macOS Keychain, GNOME Keyring)
- Age identity support (SSH-key encryption)
- Hybrid search (vector + FTS)
- PDF export (requires pandoc)
- Statistics (streaks, word counts, trends)
- Git integration (auto-commit)
- Cloud sync (encrypted remote backup)

---

**Module Value Principle**: Each module must hide significant complexity behind simple interfaces. Deep modules win.

**Prerequisites**: Manual backup before Phase 7-8 work: `cp -r ~/Documents/rubberducks ~/Documents/rubberducks-backup-$(date +%Y%m%d-%H%M%S)`
