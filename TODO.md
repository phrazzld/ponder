# TODO: Ponder v2.0 - AI-Powered Encrypted Journaling

## Progress Summary

**Completed**: Phase 0 ✅ | Phase 1 (crypto) ✅ | Phase 2 (database) ✅ | Phase 3 (AI) ✅ | Phase 4 (ops) ✅
**Current**: Phase 5 (CLI Integration)
**Remaining**: Phases 5-6 (CLI, testing/docs)

**Tests**: 121 passing (93 unit + 28 integration)
**Modules**: crypto/, db/, ai/, ops/ - all with error handling and constants
**Operations**: edit_entry, ask_question, reflect_on_entry, search_entries (574 LOC)

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

---

## ✅ Phase 0-2: COMPLETED

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

### Phase 3: AI Integration ✅
- ✅ ai/ollama.rs: HTTP client for embeddings/chat (250 LOC)
- ✅ ai/chunking.rs: Sliding window text chunking (110 LOC)
- ✅ ai/prompts.rs: System prompts + message builders (180 LOC)
- ✅ AIError enum with helpful suggestions
- ✅ AI constants (Ollama URL, models, chunk sizes)
- ✅ 18 unit tests

**All 121 tests passing** ✅

---

## Phase 4: High-Level Operations (Est: 8-10hr)

### Module: ops/ (User-Facing Operations)

- [x] **Create ops module structure** (20min)
  ```
  Files:
    src/ops/mod.rs (new, public API) ✅
    src/ops/edit.rs (new, edit with encryption + embedding) ✅
    src/ops/ask.rs (new, RAG query) ✅
    src/ops/reflect.rs (new, entry reflection) ✅
    src/ops/search.rs (new, semantic search) ✅
  ```

- [x] **Implement ops/edit.rs** (3hr)
  ```
  Files: src/ops/edit.rs (217 LOC) ✅
  Flow: decrypt→editor→re-encrypt→checksum→embed if changed
  Features:
    - YYYY/MM/DD.md.age directory structure
    - BLAKE3 checksums for change detection
    - Auto-generates embeddings on content change
    - Secure temp file handling with cleanup
    - Proper editor error handling
  ```

- [x] **Implement ops/ask.rs - RAG query** (2.5hr)
  ```
  Files: src/ops/ask.rs (140 LOC) ✅
  Flow: embed query → vector search → decrypt chunks → LLM
  Features:
    - Top-5 semantic similarity search
    - Groups chunks by entry to minimize decryptions
    - Includes entry dates in context for LLM
    - Gracefully handles no results case
  ```

- [x] **Implement ops/reflect.rs - Entry reflection** (2hr)
  ```
  Files: src/ops/reflect.rs (72 LOC) ✅
  Flow: lookup by date → decrypt entry → LLM reflection
  Features:
    - Returns reflection text directly
    - Validates entry exists before processing
    - Includes word count in debug logging
  ```

- [x] **Implement ops/search.rs - Semantic search** (2hr)
  ```
  Files: src/ops/search.rs (145 LOC) ✅
  Flow: embed query → vector search → decrypt → format with scores
  Features:
    - Groups chunks by entry to minimize decryptions
    - Returns SearchResult with date, excerpt, score
    - Sorted by similarity score descending
  ```

- [x] **Add ops module to lib.rs** (5min)

- [ ] **Create tests/ops_integration_tests.rs** (3hr)
  ```
  Tests:
    - edit_entry() full flow
    - ask_question() RAG pipeline
    - reflect_on_entry() generation
    - search_entries() semantic search
    - Error recovery (Ollama offline, wrong passphrase)
  ```

---

## Phase 5: CLI Integration (Est: 4-6hr)

### CLI Refactoring

- [ ] **Refactor cli/mod.rs for subcommands** (2hr)
  ```
  Files: src/cli/mod.rs
  Change CliArgs to enum:
    pub enum PonderCli {
        Edit { date, retro, reminisce },
        Ask { question, window },
        Reflect { date },
        Search { query, window },
        Lock,
    }
  ```

- [ ] **Update main.rs with command dispatch** (2.5hr)
  ```
  Files: src/main.rs
  Flow:
    - Parse CLI
    - Load config
    - Initialize session (unlock on first command)
    - Open database
    - Initialize AI client
    - Dispatch to ops:: based on command
  ```

- [ ] **Extend config/mod.rs with v2.0 settings** (1hr)
  ```
  New fields:
    pub db_path: PathBuf,  // PONDER_DB or journal_dir/ponder.db
    pub session_timeout_minutes: u64,  // PONDER_SESSION_TIMEOUT or 30
    pub ollama_url: String,  // OLLAMA_URL or http://127.0.0.1:11434
  ```

### Polish & Error Handling

- [ ] **Improve error messages** (1.5hr)
  ```
  Add helpful context:
    - CryptoError::VaultLocked → suggest unlocking
    - AIError::OllamaOffline → suggest `ollama serve`
    - AIError::ModelNotFound → suggest `ollama pull {model}`
    - DatabaseError → suggest checking passphrase
  ```

- [ ] **Add validation for encrypted directory structure** (1hr)
  ```
  Files: src/ops/edit.rs
  Organize as: YYYY/MM/DD.md.age
  ```

---

## Phase 6: Testing & Documentation (Est: 4-6hr)

### Comprehensive Testing

- [ ] **Add security-focused integration tests** (2.5hr)
  ```
  Files: tests/security_tests.rs
  Tests:
    - Encrypted files unreadable without passphrase
    - Temp files cleaned up even on panic
    - File permissions 0o600 (Unix)
    - No plaintext in database
    - Session timeout enforced
    - Passphrase zeroization
  ```

- [ ] **Add performance benchmarks** (1.5hr)
  ```
  Files: benches/crypto_bench.rs
  Add criterion to dev-dependencies
  Benchmarks:
    - File encryption/decryption (various sizes)
    - Vector search (various DB sizes)
    - Embedding generation (mock)
  ```

- [ ] **Run full test suite validation** (30min)
  ```
  Commands:
    cargo test --verbose -- --test-threads=1
    cargo test --test security_tests
    cargo test --test ops_integration_tests
  Success: All tests pass, >85% coverage
  ```

### Documentation

- [ ] **Update README.md with v2.0 features** (1.5hr)
  ```
  Add sections:
    - Encryption (age, SQLCipher)
    - AI features (Ollama setup, models)
    - New commands (ask, reflect, search)
    - Configuration (new env vars)
  ```

- [ ] **Create MIGRATION.md guide** (1hr)
  ```
  Sections:
    - Backup instructions
    - Ollama setup
    - ponder setup wizard
    - Migration verification
    - Rollback procedure
  ```

- [ ] **Update CLAUDE.md with v2.0 architecture** (30min)
  ```
  Updates:
    - New module structure (crypto, db, ai, ops)
    - New dependencies
    - New test structure
    - Security considerations
  ```

---

## Design Iteration Checkpoints

### After Phase 4 (Operations)
- Review: Is the ops:: module too shallow? Should we have domain-specific modules?
- Consider: Caching embeddings in memory for hot entries?
- Refactor: Extract common decrypt→process→encrypt pattern?

### After Phase 6 (Complete)
- Review: Module boundaries clear? Any leaky abstractions?
- Consider: Performance optimizations based on benchmarks?
- Plan: v2.1 features based on user feedback

---

## Validation Checklist (Before v2.0 Release)

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --verbose -- --test-threads=1` all pass
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo audit` clean (no vulnerabilities)
- [ ] All integration tests pass
- [ ] Security tests pass
- [ ] README accurate and complete
- [ ] MIGRATION.md tested manually
- [ ] Performance meets targets
- [ ] Ollama integration tested manually (real models)

---

## Time Estimates

**Completed**:
- Phase 0-2: ~8hr actual
- Phase 3 (AI): ~2hr actual

**Remaining**:
- Phase 4 (ops): 8-10hr
- Phase 5 (CLI): 4-6hr
- Phase 6 (testing/docs): 4-6hr

**Total Completed**: ~10hr
**Total Remaining**: ~16-22hr
**Total Project**: ~26-32hr (revised down from 30-38hr)

**Critical Path**: Phase 4 → Phase 5 → Phase 6

---

## BACKLOG (Post-v2.0)

These are validated but not blocking v2.0 release:

- Setup wizard (`ponder setup` command) - interactive first-time config
- Migration tool (`ponder migrate --from ~/old`) - v1.0 → v2.0 migration
- Nightly indexing (`ponder nightly`) - batch re-embedding
- Weekly review (`ponder weekly`) - aggregated insights
- Keyring integration - store passphrase in system keyring (optional)
- X25519 identity support - SSH-key-based encryption (age identities)
- Hybrid search - combine vector + FTS for better results
- Export command - decrypt to PDF/HTML
- Statistics - streak tracking, word counts, trends
