# TODO: Ponder v2.0 - AI-Powered Encrypted Journaling

## Progress Summary

**Completed**: Phase 0 (dependencies) ✅ | Phase 1 (crypto) ✅ | Phase 2 (database) ✅
**Current**: Phase 3 (AI integration)
**Remaining**: Phases 3-6 (AI, ops, CLI, testing/docs)

**Tests**: 103 passing (75 unit + 28 integration)
**Modules**: crypto/, db/, with full error handling and constants

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

**All 103 tests passing** ✅

---

## Phase 3: AI Integration (Current, Est: 6-8hr)

### Module: ai/ (Ollama Client)

- [ ] **Create ai module structure** (30min)
  ```
  Files:
    src/ai/mod.rs (new, public API)
    src/ai/ollama.rs (new, HTTP client)
    src/ai/embeddings.rs (new, embedding generation)
    src/ai/chat.rs (new, LLM chat)
    src/ai/chunking.rs (new, text chunking)
    src/ai/prompts.rs (new, system prompts)
  Success: Module compiles, can ping http://127.0.0.1:11434
  Module: ai - Hides HTTP/JSON, exposes embed/chat functions
  ```

- [ ] **Implement ai/ollama.rs - HTTP client** (2hr)
  ```
  Files: src/ai/ollama.rs (~100 LOC)
  Approach: Use reqwest::blocking::Client
  Public API:
    pub struct OllamaClient { /* private */ }
    pub fn new() -> Self
    pub fn embed(&self, model: &str, text: &str) -> AppResult<Vec<f32>>
    pub fn chat(&self, model: &str, messages: &[Message]) -> AppResult<String>
  Test: Unit tests with mockito for endpoint calls, error handling
  ```

- [ ] **Implement ai/chunking.rs - Text chunking** (1hr)
  ```
  Files: src/ai/chunking.rs (~60 LOC)
  Approach: Word-based sliding window (700 words, 100 overlap)
  Public API:
    pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String>
  Test: Overlap correctness, edge cases (text < chunk_size, empty)
  ```

- [ ] **Implement ai/prompts.rs - System prompts** (1hr)
  ```
  Files: src/ai/prompts.rs (~80 LOC)
  Approach: Const strings + builder functions
  Public API:
    pub const SYSTEM_PROMPT: &str = "...";
    pub fn reflect_prompt(entry_content: &str) -> Vec<Message>
    pub fn ask_prompt(question: &str, context_chunks: &[String]) -> Vec<Message>
  Test: Message formatting, content inclusion
  ```

- [ ] **Extend errors/mod.rs with AIError** (20min)
  ```
  Files: src/errors/mod.rs
  New variants:
    pub enum AIError {
        #[error("Ollama API error: {0}. Is Ollama running? Try: ollama serve")]
        OllamaOffline(#[source] reqwest::Error),
        #[error("Model not found: {0}. Try: ollama pull {0}")]
        ModelNotFound(String),
        #[error("Invalid response from Ollama: {0}")]
        InvalidResponse(String),
    }
  Add to AppError:
    AI(#[from] AIError),
  ```

- [ ] **Update constants.rs with AI defaults** (5min)
  ```
  Files: src/constants.rs
  pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
  pub const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";
  pub const DEFAULT_CHAT_MODEL: &str = "llama3.2:3b";
  pub const DEFAULT_CHUNK_SIZE: usize = 700;
  pub const DEFAULT_CHUNK_OVERLAP: usize = 100;
  ```

- [ ] **Add ai module to lib.rs exports** (5min)

- [ ] **Create tests/ai_tests.rs - AI integration tests** (2hr)
  ```
  Files: tests/ai_tests.rs (~200 LOC)
  Approach: Use mockito for HTTP mocking
  Tests:
    - OllamaClient embed() with mock server
    - OllamaClient chat() with mock server
    - Chunking preserves content
    - Prompt generation correct
    - Error handling (mock 500 errors, timeouts)
  ```

**Validation**: `cargo test -- --test-threads=1` all pass

---

## Phase 4: High-Level Operations (Est: 8-10hr)

### Module: ops/ (User-Facing Operations)

- [ ] **Create ops module structure** (20min)
  ```
  Files:
    src/ops/mod.rs (new, public API)
    src/ops/edit.rs (new, edit with encryption + embedding)
    src/ops/ask.rs (new, RAG query)
    src/ops/reflect.rs (new, entry reflection)
    src/ops/search.rs (new, semantic search)
  ```

- [ ] **Implement ops/edit.rs** (3hr)
  ```
  Files: src/ops/edit.rs (~200 LOC)
  Flow: decrypt→editor→re-encrypt→checksum→embed if changed
  Public API:
    pub fn edit_entry(config: &Config, db: &Database, session: &mut SessionManager,
                      ai_client: &OllamaClient, date: NaiveDate,
                      reference_datetime: &DateTime<Local>) -> AppResult<()>
  ```

- [ ] **Implement ops/ask.rs - RAG query** (2.5hr)
  ```
  Files: src/ops/ask.rs (~150 LOC)
  Flow: embed query → vector search → decrypt chunks → LLM
  ```

- [ ] **Implement ops/reflect.rs - Entry reflection** (2hr)
  ```
  Files: src/ops/reflect.rs (~100 LOC)
  Flow: decrypt entry → LLM reflection → save encrypted report
  ```

- [ ] **Implement ops/search.rs - Semantic search** (2hr)
  ```
  Files: src/ops/search.rs (~120 LOC)
  Flow: embed query → vector search → decrypt snippets → format
  ```

- [ ] **Add ops module to lib.rs** (5min)

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

**Completed**: Phase 0-2 (~8hr actual)
**Remaining**:
- Phase 3 (AI): 6-8hr
- Phase 4 (ops): 8-10hr
- Phase 5 (CLI): 4-6hr
- Phase 6 (testing/docs): 4-6hr

**Total Remaining**: ~22-30hr
**Total Project**: ~30-38hr

**Critical Path**: Phase 3 → Phase 4 → Phase 5 → Phase 6

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
