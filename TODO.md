# TODO: Merge PR #50

**Branch**: `feat/v2.0-encrypted-ai-journaling`
**Tests**: 212 passing (14 ignored) | **Build**: ✅ PASSING
**Status**: PR review feedback being addressed

---

## Critical Fixes (Merge-Blocking)

**Source**: PR #50 review feedback
**Estimated Time**: ~3 hours

### P0: Security & Bugs (Must Fix)

- [x] **Fix temp file permission race** (20min) - `src/crypto/temp.rs:88-106`
  - Create temp files with 0o600 from start, not after writing plaintext
  - Security: World-readable window exposes journal content
  - Test: Verify permissions immediately after creation

- [~] **Delete stale embeddings** (15min) - `src/ops/edit.rs:280-284`
  - Add `DELETE FROM embeddings WHERE entry_id = ?` before inserting new chunks
  - Bug: Shortened entries leave orphaned chunks in DB
  - Test: Edit long entry → short entry, verify chunk count

- [ ] **Fix passphrase zeroization leak** (45min) - `src/db/mod.rs:75,199`
  - Remove `SqlCipherConfig` struct, use closure with `SecretString`
  - Security: Plain `String` in pool defeats zeroization
  - Verify: Database still opens, `SqlCipherConfig` removed

- [ ] **Fix session timeout bug** (30min) - `src/crypto/session.rs:103-112`
  - Change `get_passphrase()` to `&mut self`, update `last_access` on every call
  - Bug: Active users locked out during normal use
  - Update call sites in: edit.rs, ask.rs, reflect.rs, search.rs
  - Test: Verify timeout extends on passphrase access

### P1: High-Priority Improvements

- [ ] **Add passphrase strength validation** (45min) - `src/crypto/session.rs:181-183`
  - Add `WeakPassphrase` error variant, validate 8+ character minimum
  - Update both passphrase prompts to use validation
  - Test: Reject passphrases <8 chars, accept >=8 chars

- [ ] **Guard test passphrase env var** (5min) - `src/crypto/session.rs:236-238`
  - Wrap `PONDER_TEST_PASSPHRASE` check in `#[cfg(test)]`
  - Verify: Not accessible in release builds

- [ ] **Add passphrase recovery warning** (15min) - `README.md`
  - Add security notice section explaining permanent data loss if forgotten
  - Update quickstart with passphrase choice guidance

- [ ] **Add passphrase retry logic** (30min) - `src/main.rs` command handlers
  - Allow 3 attempts on wrong passphrase before failing
  - Optional: May defer if refactoring too complex

---

## Validation Before Merge

- [ ] All P0 fixes complete and tested
- [ ] All P1 fixes complete (or retry logic deferred with note)
- [ ] Full test suite passing: `cargo test --verbose -- --test-threads=1`
- [ ] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] Manual QA: Session timeout, passphrase prompts, temp file security
- [ ] PR description updated with fixes applied

---

**Detailed implementation notes**: See `PR_REVIEW_RESPONSE.md`
**Follow-up work**: See `BACKLOG.md` (PR Review Feedback section)
