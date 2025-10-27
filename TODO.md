# TODO: Merge PR #50

**Branch**: `feat/v2.0-encrypted-ai-journaling`
**Tests**: 175 passing (11 ignored, 0 failed) | **Build**: ✅ PASSING
**Status**: ✅ All P0 fixes complete, ready to merge

---

## Completed P0 Fixes (Merge-Blocking)

All 4 critical security and bug fixes from PR #50 review feedback implemented:

1. ✅ **Temp file permission race** (`9af40a7`) - 20min
   - Pre-create files with 0o600 before decryption (no plaintext exposure window)

2. ✅ **Stale embeddings deletion** (`e037fc7`) - 15min
   - DELETE before INSERT prevents orphaned chunks in shortened entries

3. ✅ **Passphrase zeroization leak** (`2ccc386`) - 45min
   - Arc<SecretString> in SqlCipherConfig preserves zeroization across pool

4. ✅ **Session timeout bug** (`4c4dd76`) - 30min
   - get_passphrase() now updates last_access (sliding timeout window)

**Total effort**: ~2 hours | **Result**: 175 tests passing (added 1 new test)

---

## Critical Ultrathink Fixes (Pre-Merge)

**Source**: Ultrathink design review - critical issues requiring immediate attention
**Estimated Time**: ~50 minutes

- [x] **Add transaction atomicity to embedding generation** (30min) - `src/ops/edit.rs:253-297`
  - Issue: DELETE + INSERT loop has no transaction boundary - Ollama crash mid-loop leaves database with zero embeddings for entry
  - Fix: Wrap entire embedding generation in `conn.transaction()` for all-or-nothing semantics
  - Implementation details:
    ```rust
    fn generate_and_store_embeddings(...) -> AppResult<()> {
        let tx = conn.transaction()?;

        // Delete old embeddings (within transaction)
        tx.execute("DELETE FROM embeddings WHERE entry_id = ?", [entry_id])?;

        // Generate and insert all chunks (within transaction)
        for (idx, chunk) in chunks.iter().enumerate() {
            let embedding = ai_client.embed_with_retry(DEFAULT_EMBED_MODEL, chunk, 3)?;
            let chunk_hash = blake3::hash(chunk.as_bytes());
            let chunk_checksum = chunk_hash.to_hex().to_string();
            insert_embedding(&tx, entry_id, idx, &embedding, &chunk_checksum)?;
        }

        tx.commit()?;  // Atomic: all chunks inserted or none
        Ok(())
    }
    ```
  - Update `insert_embedding()` signature: accept `&Transaction` or `&Connection` (use generic trait bound)
  - Success: Embedding generation is atomic - partial failures roll back, no orphaned deletes

- [x] **Add user feedback to edit command** (15min) - `src/ops/edit.rs`
  - Issue: Silent success pattern - users see no confirmation after saving entry or generating embeddings
  - Fix: Add `eprintln!` output confirming successful operations
  - Implementation: At end of `edit_entry()` function (before final `Ok(())`):
    ```rust
    eprintln!("✓ Journal entry saved: {}", date);
    if content_changed && !chunks.is_empty() {
        eprintln!("  Generated {} embedding chunks", chunks.len());
    }
    ```
  - Success: Users see clear confirmation message after edit completes

- [x] **Add security warning to README** (5min) - `README.md`
  - Issue: Missing critical warning that forgotten passphrase = permanent data loss (zero-knowledge encryption)
  - Fix: Add prominent ⚠️ Security Notice section before Installation section
  - Content to add:
    ```markdown
    ## ⚠️ Security Notice

    Ponder uses **zero-knowledge encryption** - your passphrase encrypts all journal data.

    **CRITICAL**: If you forget your passphrase, **your data is permanently lost**. There is no recovery mechanism.

    **Best practices**:
    - Choose a passphrase you can remember (e.g., 4-5 random words)
    - Write it down and store in a secure physical location
    - Consider using a password manager
    - Test backup/restore before relying on Ponder for important data
    ```
  - Success: Warning is prominent, clear about consequences, provides actionable guidance

---

## Critical P1 Fixes (PR Code Review)

**Source**: @chatgpt-codex-connector code review comments on PR #50
**Estimated Time**: 50 minutes (25min complete, 25min remaining)

1. [x] **Fix secure_delete truncation vulnerability** (`3e4aaef`) - 10min - `src/crypto/temp.rs:221-243`
   - Issue: `File::create()` truncates file BEFORE overwriting, leaving plaintext journal content recoverable from disk sectors
   - Impact: Every journal edit leaves decrypted content on disk (defeats core security model)
   - Fix: Replace `File::create(path)` with `OpenOptions::new().write(true).open(path)` to preserve file size and actually overwrite sectors
   - Added regression test: `test_secure_delete_overwrites_without_truncation()`
   - Success: Plaintext sectors now actually overwritten with zeros before deletion (prevents trivial forensic recovery)

2. [x] **Fix backup/restore to honor Config.db_path** (`55341b6`) - 15min - `src/ops/backup.rs:84,401` + `src/main.rs:423,458`
   - Issue: `create_backup()` and `restore_backup()` hardcoded database path as `journal_dir/ponder.db`, ignoring `Config.db_path` (PONDER_DB env var)
   - Impact: Backups fail for custom database locations OR worse, back up wrong database if old one exists at default path; restore always writes to wrong location (data loss risk)
   - Fix: Add `db_path: &PathBuf` parameter to both functions, pass `config.db_path` from main.rs, update all 9 test cases
   - Success: Backup/restore now respects PONDER_DB configuration for all database locations

3. [x] **Fix auto-lock passphrase leak** - 25min - `src/crypto/session.rs:104-116,186-191`
   - Issue: `is_locked()` detects timeout but leaves passphrase cached in memory; `touch()` can revive expired sessions without re-prompting user
   - Impact: Auto-lock timeout is completely bypassable via `touch()` (called by migration code every 10 entries); passphrase remains in memory indefinitely during long operations, defeating stated security goal
   - Fix: Make `get_passphrase()` eagerly call `self.lock()` when timeout detected; change `touch()` condition from `passphrase.is_some()` to `!is_locked()`
   - Test: Added `test_touch_cannot_revive_timed_out_session()` regression test verifying passphrase is zeroized when timeout first detected
   - Success: Timed-out sessions cannot be revived without re-prompting; passphrase is actually cleared from memory on timeout (177 tests passing, +1 new test)

4. [x] **Fix world-readable temp files for new entries** - 15min - `src/ops/edit.rs:70-105`
   - Issue: `fs::write()` creates new entry temp files with default permissions (0o644 - world-readable); any local user can read plaintext journal content during editing
   - Impact: CRITICAL - Active plaintext exposure for every new entry creation; trivial exploitation (`cat /tmp/ponder-new-*.md`); violates core encrypted journaling security model
   - Fix: Apply same secure pattern as `decrypt_to_temp()`: use `OpenOptions` with `.mode(0o600)` BEFORE writing plaintext header
   - Test: Added regression test `test_new_entry_temp_file_has_secure_permissions()` verifying 0o600 permissions
   - Success: All temp files (new and existing entries) created with secure permissions; no plaintext exposure window (178 tests passing, +1 new test)

**Total effort**: 65 minutes | **Result**: All 4 P1 security fixes complete, 178 tests passing, clippy clean

---

## Pre-Merge Validation Checklist

- [x] All P0 fixes complete and tested
- [x] All Ultrathink critical fixes complete and tested
- [x] All P1 security fixes complete and tested (secure_delete, backup/restore config, auto-lock passphrase leak, world-readable temp files)
- [x] Full test suite passing: `cargo test --lib -- --test-threads=1` (178 passing, +2 new tests)
- [x] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] Manual QA: Verify session timeout, passphrase prompts, temp file perms, transaction rollback
- [ ] Update PR #50 description with P0 + Ultrathink fixes summary
- [ ] Ready to merge to `master`

---

**P1 post-merge work**: See `BACKLOG.md` (P1 - High-Priority Post-Merge section)
**Implementation details**: See `PR_REVIEW_RESPONSE.md`
