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

## Critical P1 Security Fix (PR Code Review)

**Source**: @chatgpt-codex-connector code review comment on PR #50
**Estimated Time**: 10 minutes

- [x] **Fix secure_delete truncation vulnerability** (`3e4aaef`) - 10min - `src/crypto/temp.rs:221-243`
  - Issue: `File::create()` truncates file BEFORE overwriting, leaving plaintext journal content recoverable from disk sectors
  - Impact: Every journal edit leaves decrypted content on disk (defeats core security model)
  - Fix: Replace `File::create(path)` with `OpenOptions::new().write(true).open(path)` to preserve file size and actually overwrite sectors
  - Added regression test: `test_secure_delete_overwrites_without_truncation()`
  - Success: Plaintext sectors now actually overwritten with zeros before deletion (prevents trivial forensic recovery)

**Total effort**: 10 minutes | **Result**: 7 crypto::temp tests passing (added 1 new test)

---

## Pre-Merge Validation Checklist

- [x] All P0 fixes complete and tested
- [x] All Ultrathink critical fixes complete and tested
- [x] All P1 security fixes complete and tested (secure_delete truncation)
- [x] Full test suite passing: `cargo test --lib -- --test-threads=1` (176 passing)
- [x] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] Manual QA: Verify session timeout, passphrase prompts, temp file perms, transaction rollback
- [ ] Update PR #50 description with P0 + Ultrathink fixes summary
- [ ] Ready to merge to `master`

---

**P1 post-merge work**: See `BACKLOG.md` (P1 - High-Priority Post-Merge section)
**Implementation details**: See `PR_REVIEW_RESPONSE.md`
