# TODO: Ponder v2.0 - AI-Powered Encrypted Journaling

## Progress Summary

**Completed**: Phase 0-5 ✅, Phase 7 Critical Refactoring A-B, F ✅
**Current**: Phase 7 (CLI handlers + testing)
**Next**: Implement backup/restore handlers → Testing → Phase 8 (migration)

**Tests**: 198 passing (3 ignored) | **Build**: ✅ PASSING
**Architecture**: v2.0 complete - encrypted journaling with AI features operational

---

## Phase 6: Testing & Documentation (Remaining)

### Documentation
- [ ] Add Backup & Export section to README.md (30min)
  - Cover: `backup`, `restore` commands with examples
  - Security: Warn about backup storage security

- [ ] Rewrite MIGRATION.md migration strategy (45min)
  - Flip: Automatic detection (primary) vs manual (fallback)
  - Add: Interactive flow examples, resume capability

- [ ] Create docs/COMMANDS.md reference (30min)
  - Command reference: edit, ask, search, reflect, lock, backup, restore, cleanup-v1

### Quality Infrastructure
- [ ] Add security.yml workflow (30min)
  - Jobs: gitleaks history scan, cargo-audit vulnerabilities

---

## Phase 7: Backup & Archive System

### Completed ✅
- ✅ Dependencies (tar, flate2, walkdir, tempfile)
- ✅ Database schema (backup_log, backup_state tables)
- ✅ Database functions (record_backup, get_last_backup, get_backup_history)
- ✅ Module structure (BackupReport, BackupManifest, RestoreReport)
- ✅ `create_backup()` - Full encrypted archive creation
- ✅ `verify_backup()` - Integrity checking with manifest
- ✅ `restore_backup()` - Atomic extraction with checksum verification
- ✅ CLI command definitions (Backup, Restore in cli/mod.rs)
- ✅ Report symmetry (RestoreReport has checksum + duration)
- ✅ API cleanup (removed unused incremental parameter)

### Remaining Tasks

#### CLI Handlers (1.5hr)

- [ ] Implement cmd_backup() in main.rs (30min)
  - Initialize SessionManager + Database
  - Call `ops::create_backup()`
  - Optional: `ops::verify_backup()` if `args.verify`
  - Print BackupReport summary

- [ ] Implement cmd_restore() in main.rs (20min)
  - Initialize SessionManager
  - Confirm overwrite if target exists (unless `--force`)
  - Call `ops::restore_backup()`
  - Print RestoreReport summary

- [ ] Expose backup in ops/mod.rs (5min)
  - `pub mod backup;`
  - `pub use backup::{create_backup, verify_backup, restore_backup};`

#### Testing (2hr)

- [ ] Create tests/backup_tests.rs
  - test_create_full_backup: Create → verify size/checksum
  - test_verify_backup: Create → verify → check manifest accuracy
  - test_restore_backup: Create → delete original → restore → compare
  - test_backup_wrong_passphrase: Verify fails with incorrect passphrase

#### Optional Refactoring (Deferred)

**C. Extract Temporal Decomposition (2hr)** - Can defer to post-v2.1
- Extract helpers: `collect_backup_files()`, `create_tar_archive()`, `encrypt_and_write()`
- Benefit: Isolated testing, clearer intent

**D. Unified Archive System (3hr)** - Strategic, consider for v2.2
- Strategy pattern with `ArchiveFormat` enum
- Would eliminate duplication for future export formats
- Currently not blocking v2.1 release

**E. Schema Simplification (30min)** - Only if issues arise
- Remove `backup_state` table (derivable from `backup_log`)

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

### Migration Engine (4hr)
- [ ] Create `src/ops/migration.rs`
  - `migrate_entry()`: Read v1 plaintext → encrypt → write v2 → embed → verify
  - `verify_migration()`: Decrypt v2, compare with v1 byte-for-byte
  - `migrate_all_entries()`: Batch processor with progress callback

### CLI Integration (2hr)
- [ ] Add `--migrate` flag to EditArgs
- [ ] Add `CleanupV1` command
- [ ] Integrate detection into `cmd_edit()`: Auto-prompt on first v1.0 detection
- [ ] Implement `prompt_migration()`: Interactive choice
- [ ] Implement `print_progress()`: Progress bar with ETA
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
**Phase 7 Remaining**: ~2hr (handlers + tests)
**Phase 8**: ~8hr (migration system)

**Critical Path**:
1. Complete Phase 7 handlers (2hr)
2. Phase 8 migration system (8hr)
3. Documentation + validation (3hr)

**Total Remaining**: ~13hr

---

## BACKLOG (Post-v2.1)

Validated ideas deferred to future releases:
- Setup wizard (`ponder setup`)
- Nightly indexing for embedding backfill
- Weekly review aggregation (`ponder weekly`)
- Keyring integration (macOS Keychain, GNOME Keyring)
- Age identity support (SSH-key encryption)
- Hybrid search (vector + FTS)
- Export functionality (HTML, PDF)
- Statistics (streaks, word counts, trends)
- Git integration (auto-commit)
- Cloud sync (encrypted remote backup)

---

## Recent Commits

**Phase 7 Progress:**
- `026f9d0` fix(deps): move tempfile to production dependencies
- `f41958d` refactor(backup): remove unused incremental parameter
- `9b8934a` refactor(backup): add checksum and duration to RestoreReport
- `0328e86` feat(cli): add backup and restore commands

**Module Value Principle**: Each module must hide significant complexity behind simple interfaces. Deep modules win.

**Prerequisites**: Manual backup before Phase 7-8 work: `cp -r ~/Documents/rubberducks ~/Documents/rubberducks-backup-$(date +%Y%m%d-%H%M%S)`
