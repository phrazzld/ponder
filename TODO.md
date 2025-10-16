# TODO: Ponder v2.0 - AI-Powered Encrypted Journaling

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

## Phase 0: Critical Fixes & Foundation (Week 1)

### Security & Deprecation Fixes

- [ ] **CRITICAL: Upgrade tracing-subscriber to 0.3.20**
  ```
  Files: Cargo.toml:7
  Approach: Change version = "0.3" → version = "0.3.20" (RUSTSEC-2025-0055)
  Success: `cargo audit` clean, `cargo build` succeeds
  Test: Existing tests pass unchanged
  Module: N/A (dependency fix)
  Time: 5min
  ```

- [ ] **CRITICAL: Fix deprecated clap API in cli/mod.rs**
  ```
  Files: src/cli/mod.rs:~50-60
  Approach: Replace `.possible_values()` with `.value_parser()`
  Success: No deprecation warnings in `cargo build`
  Test: Existing CLI tests pass (tests/cli_tests.rs)
  Module: N/A (API migration)
  Time: 15min
  ```

### Dependency Setup

- [ ] **Add encryption dependencies to Cargo.toml**
  ```
  Files: Cargo.toml:10-15
  Approach: Add age, argon2, zeroize, blake3 to [dependencies]
  Success: `cargo build` resolves all deps without conflicts
  Test: `cargo tree` shows no duplicate/conflicting versions
  Module: N/A (infrastructure)
  Time: 10min
  Dependencies:
    age = "0.10"
    argon2 = "0.5"
    zeroize = "1.7"
    blake3 = "1.5"
  ```

- [ ] **Add database dependencies to Cargo.toml**
  ```
  Files: Cargo.toml:16-19
  Approach: Add rusqlite (bundled-sqlcipher), r2d2, r2d2_sqlite
  Success: `cargo build` succeeds, no version conflicts
  Test: Can create Connection with bundled-sqlcipher feature
  Module: N/A (infrastructure)
  Time: 15min
  Dependencies:
    rusqlite = { version = "0.31", features = ["bundled-sqlcipher", "blob"] }
    r2d2 = "0.8"
    r2d2_sqlite = "0.23"
    bytemuck = "1.14"  # For f32 <-> u8 conversion in embeddings
  ```

- [ ] **Add AI/HTTP dependencies to Cargo.toml**
  ```
  Files: Cargo.toml:20-23
  Approach: Add reqwest, serde, serde_json for Ollama HTTP client
  Success: `cargo build` succeeds
  Test: Can make HTTP request to localhost
  Module: N/A (infrastructure)
  Time: 10min
  Dependencies:
    reqwest = { version = "0.11", features = ["json", "blocking"] }
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
  ```

- [ ] **Add dev/test dependencies**
  ```
  Files: Cargo.toml:30-31
  Approach: Add mockito for mocking Ollama API in tests
  Success: `cargo test` can compile mock servers
  Test: Mock HTTP server responds correctly
  Module: N/A (test infrastructure)
  Time: 5min
  Dependencies (under [dev-dependencies]):
    mockito = "1.2"
  ```

### Validation Checkpoint

- [ ] **Run full validation suite**
  ```
  Commands:
    cargo build --verbose
    cargo clippy --all-targets -- -D warnings
    cargo test --verbose -- --test-threads=1
    cargo audit
  Success: All pass, no new warnings/errors
  Time: 10min
  ```

---

## Phase 1: Encryption Core (Weeks 2-3)

### Module: crypto/ (Age Encryption)

- [ ] **Create crypto module structure**
  ```
  Files:
    src/crypto/mod.rs (new, public API)
    src/crypto/age.rs (new, age encryption)
    src/crypto/session.rs (new, session mgmt)
    src/crypto/temp.rs (new, tmpfs handling)
  Approach: Follow src/config/mod.rs pattern for module structure
  Success: `use ponder::crypto;` compiles, module visible in lib.rs
  Test: Module compiles, empty tests pass
  Module: crypto - Hides encryption complexity, exposes encrypt/decrypt functions
  Time: 20min
  ```

- [ ] **Implement crypto/age.rs - Core encryption functions**
  ```
  Files: src/crypto/age.rs (new, ~150 LOC)
  Approach: Use age::Encryptor/Decryptor with SecretString
  Success: Round-trip encrypt→decrypt returns original plaintext
  Test: Unit tests for:
    - encrypt_with_passphrase() / decrypt_with_passphrase()
    - encrypt_file_streaming() / decrypt_file_streaming()
    - Error cases (wrong passphrase, corrupted data)
  Module: Deep module - simple encrypt/decrypt API, hides age complexity
  Time: 2hr
  Public API:
    pub fn encrypt_with_passphrase(plaintext: &[u8], pass: &SecretString) -> AppResult<Vec<u8>>
    pub fn decrypt_with_passphrase(ciphertext: &[u8], pass: &SecretString) -> AppResult<Vec<u8>>
    pub fn encrypt_file_streaming(in: &Path, out: &Path, pass: &SecretString) -> AppResult<()>
    pub fn decrypt_file_streaming(in: &Path, out: &Path, pass: &SecretString) -> AppResult<()>
  ```

- [ ] **Implement crypto/session.rs - Session key management**
  ```
  Files: src/crypto/session.rs (new, ~100 LOC)
  Approach: Use Zeroizing<SecretString>, Instant for timeout tracking
  Success: Session auto-locks after timeout, passphrase zeroized on drop
  Test: Unit tests for:
    - unlock() prompts when locked, returns cached when unlocked
    - is_locked() true after timeout
    - lock() zeroizes memory
    - Drop trait zeroizes on scope exit
  Module: Manages session state, hides timeout logic
  Time: 1.5hr
  Public API:
    pub struct SessionManager { /* private */ }
    pub fn new(timeout_minutes: u64) -> Self
    pub fn unlock(&mut self) -> AppResult<&SecretString>
    pub fn is_locked(&self) -> bool
    pub fn lock(&mut self)
  ```

- [ ] **Implement crypto/temp.rs - Secure temp file handling**
  ```
  Files: src/crypto/temp.rs (new, ~120 LOC)
  Approach: Check /dev/shm, /run/shm for tmpfs, fallback to std::env::temp_dir()
  Success: Temp files created with 0o600, cleaned up on error, tmpfs preferred
  Test: Unit tests for:
    - get_secure_temp_dir() finds tmpfs on Linux, warns on fallback
    - decrypt_to_temp() creates file with correct perms
    - encrypt_from_temp() re-encrypts and securely deletes temp
    - secure_delete() overwrites file before removal
  Module: Hides tmpfs detection, provides simple decrypt/encrypt temp API
  Time: 1.5hr
  Public API:
    pub fn get_secure_temp_dir() -> AppResult<PathBuf>
    pub fn decrypt_to_temp(encrypted: &Path, pass: &SecretString) -> AppResult<PathBuf>
    pub fn encrypt_from_temp(temp: &Path, encrypted: &Path, pass: &SecretString) -> AppResult<()>
  ```

- [ ] **Extend errors/mod.rs with CryptoError**
  ```
  Files: src/errors/mod.rs:~210-230
  Approach: Follow EditorError/LockError pattern with #[derive(Error)]
  Success: CryptoError converts to AppError via From trait
  Test: Error messages contain helpful context
  Module: Extends existing error hierarchy
  Time: 30min
  New variants:
    #[derive(Debug, Error)]
    pub enum CryptoError {
        #[error("Vault is locked. Run command again to unlock.")]
        VaultLocked,
        #[error("Incorrect passphrase")]
        InvalidPassphrase(#[source] age::DecryptError),
        #[error("Unsupported encryption format")]
        UnsupportedFormat,
        #[error("Invalid file path: {0}")]
        InvalidPath(String),
    }
  Add to AppError enum:
    Crypto(#[from] CryptoError),
  ```

- [ ] **Update constants.rs with crypto defaults**
  ```
  Files: src/constants.rs:~85-95
  Approach: Add crypto-related constants at end of file
  Success: Constants compile, used in crypto module
  Test: N/A (constants)
  Module: N/A (shared constants)
  Time: 10min
  New constants:
    pub const DEFAULT_SESSION_TIMEOUT_MINUTES: u64 = 30;
    pub const ENCRYPTED_FILE_EXTENSION: &str = ".age";
    pub const TMPFS_PATHS: &[&str] = &["/dev/shm", "/run/shm"];
  ```

- [ ] **Add crypto module to lib.rs exports**
  ```
  Files: src/lib.rs:60-65
  Approach: Add `pub mod crypto;` after existing modules
  Success: `use ponder::crypto::SessionManager;` works in tests
  Test: Integration test can import crypto types
  Module: N/A (public API)
  Time: 5min
  ```

### Integration Tests

- [ ] **Create tests/crypto_tests.rs - Encryption integration tests**
  ```
  Files: tests/crypto_tests.rs (new, ~200 LOC)
  Approach: Follow tests/config_tests.rs pattern with tempfile
  Success: All encryption scenarios pass
  Test: Integration tests for:
    - Round-trip file encryption
    - Streaming large files (>10MB)
    - Session timeout behavior
    - Temp file cleanup on error
    - Permission verification (0o600)
    - Zeroization (memory inspection if possible)
  Module: N/A (test suite)
  Time: 2hr
  ```

---

## Phase 2: Database Layer (Weeks 4-5)

### Module: db/ (SQLCipher Operations)

- [ ] **Create db module structure**
  ```
  Files:
    src/db/mod.rs (new, public API + connection pool)
    src/db/schema.rs (new, table definitions)
    src/db/entries.rs (new, entry CRUD)
    src/db/embeddings.rs (new, vector operations)
  Approach: Follow src/journal_io module structure with submodules
  Success: Database opens with SQLCipher key, schema created
  Test: Can create encrypted DB, tables exist
  Module: db - Hides SQLite/SQLCipher complexity, exposes simple CRUD
  Time: 30min
  ```

- [ ] **Implement db/mod.rs - Connection pool**
  ```
  Files: src/db/mod.rs (new, ~80 LOC)
  Approach: Use r2d2::Pool with SqliteConnectionManager
  Success: Pool created with SQLCipher key pragma, connections work
  Test: Unit tests for:
    - open() creates encrypted DB
    - get_conn() returns working connection
    - initialize_schema() creates all tables
    - Wrong passphrase fails to open DB
  Module: Deep module - connection pooling hidden, simple open/get_conn API
  Time: 1.5hr
  Public API:
    pub struct Database { /* private pool */ }
    pub fn open(db_path: &Path, passphrase: &SecretString) -> AppResult<Self>
    pub fn get_conn(&self) -> AppResult<PooledConnection<...>>
    pub fn initialize_schema(&self) -> AppResult<()>
  ```

- [ ] **Implement db/schema.rs - Table definitions**
  ```
  Files: src/db/schema.rs (new, ~100 LOC)
  Approach: Use conn.execute_batch() for CREATE TABLE statements
  Success: All tables created with correct indexes
  Test: Unit tests for:
    - create_tables() creates all tables
    - Foreign keys enforced
    - Indexes exist
    - FTS5 virtual table works
  Module: Schema management, hides SQL DDL
  Time: 1.5hr
  Tables:
    - entries (id, path, date, checksum, word_count, updated_at, embedded_at)
    - embeddings (id, entry_id, chunk_idx, embedding BLOB, checksum, created_at)
    - entries_fts (FTS5: entry_id, date, content)
    - insights (id, entry_id, type, encrypted_content, score, created_at)
    - reports (id, path, type, date_range, created_at)
  ```

- [ ] **Implement db/entries.rs - Entry CRUD operations**
  ```
  Files: src/db/entries.rs (new, ~150 LOC)
  Approach: Use rusqlite::params! macro for queries
  Success: Can insert/update/query entries
  Test: Unit tests for:
    - upsert_entry() creates or updates entry
    - get_entry_by_date() retrieves entry
    - get_entry_path() returns PathBuf
    - needs_embedding_update() checks checksum
    - mark_embedded() updates timestamp
  Module: Entry operations, hides SQL details
  Time: 2hr
  Public API:
    pub fn upsert_entry(conn: &Connection, path: &Path, date: NaiveDate,
                        checksum: &str, word_count: usize) -> AppResult<i64>
    pub fn get_entry_by_date(conn: &Connection, date: NaiveDate) -> AppResult<Option<Entry>>
    pub fn get_entry_path(conn: &Connection, entry_id: i64) -> AppResult<PathBuf>
    pub fn needs_embedding_update(conn: &Connection, entry_id: i64,
                                   checksum: &str) -> AppResult<bool>
    pub fn mark_embedded(conn: &Connection, entry_id: i64) -> AppResult<()>
  ```

- [ ] **Implement db/embeddings.rs - Vector operations**
  ```
  Files: src/db/embeddings.rs (new, ~180 LOC)
  Approach: Use bytemuck for f32 <-> u8 conversion, manual cosine similarity
  Success: Embeddings stored/retrieved, vector search returns similar chunks
  Test: Unit tests for:
    - insert_embedding() stores 768-dim vector
    - get_entry_embeddings() retrieves all chunks
    - search_similar_chunks() ranks by cosine similarity
    - cosine_similarity() math correct
  Module: Vector storage, hides blob serialization and similarity computation
  Time: 2.5hr
  Public API:
    pub fn insert_embedding(conn: &Connection, entry_id: i64, chunk_idx: usize,
                           embedding: &[f32], checksum: &str) -> AppResult<()>
    pub fn get_entry_embeddings(conn: &Connection, entry_id: i64)
                                -> AppResult<Vec<(usize, Vec<f32>)>>
    pub fn search_similar_chunks(conn: &Connection, query_embedding: &[f32],
                                 limit: usize) -> AppResult<Vec<(i64, usize, f32)>>
  ```

- [ ] **Extend errors/mod.rs with DatabaseError**
  ```
  Files: src/errors/mod.rs:~240-260
  Approach: Follow CryptoError pattern
  Success: Database errors wrapped in AppError
  Test: Error messages helpful
  Module: Extends error hierarchy
  Time: 20min
  New variants:
    #[derive(Debug, Error)]
    pub enum DatabaseError {
        #[error("Database error: {0}")]
        Sqlite(#[from] rusqlite::Error),
        #[error("Failed to get connection from pool: {0}")]
        Pool(#[from] r2d2::Error),
        #[error("Entry not found: {0}")]
        NotFound(String),
    }
  Add to AppError:
    Database(#[from] DatabaseError),
  ```

- [ ] **Update constants.rs with database defaults**
  ```
  Files: src/constants.rs:~95-105
  Approach: Add DB-related constants
  Success: Constants used in db module
  Test: N/A (constants)
  Module: N/A (shared constants)
  Time: 5min
  New constants:
    pub const DEFAULT_DB_FILENAME: &str = "ponder.db";
    pub const EMBEDDING_DIMENSIONS: usize = 768;  // nomic-embed-text
    pub const VECTOR_SEARCH_LIMIT: usize = 12;
  ```

- [ ] **Add db module to lib.rs exports**
  ```
  Files: src/lib.rs:66-68
  Approach: Add `pub mod db;` after crypto
  Success: `use ponder::db::Database;` works
  Test: Can import in integration tests
  Module: N/A (public API)
  Time: 5min
  ```

### Integration Tests

- [ ] **Create tests/db_tests.rs - Database integration tests**
  ```
  Files: tests/db_tests.rs (new, ~250 LOC)
  Approach: Follow tests/journal_integration_tests.rs pattern with tempfile
  Success: All database operations work correctly
  Test: Integration tests for:
    - Open encrypted DB with passphrase
    - Schema creation (all tables + indexes)
    - Entry CRUD operations
    - Embedding storage and retrieval
    - Vector similarity search accuracy (known vectors)
    - Wrong passphrase fails gracefully
    - Connection pooling (concurrent access)
  Module: N/A (test suite)
  Time: 2.5hr
  ```

---

## Phase 3: AI Integration (Weeks 6-7)

### Module: ai/ (Ollama Client)

- [ ] **Create ai module structure**
  ```
  Files:
    src/ai/mod.rs (new, public API)
    src/ai/ollama.rs (new, HTTP client)
    src/ai/embeddings.rs (new, embedding generation)
    src/ai/chat.rs (new, LLM chat)
    src/ai/chunking.rs (new, text chunking)
    src/ai/prompts.rs (new, system prompts)
  Approach: Follow db module structure
  Success: Module compiles, Ollama client can ping API
  Test: Can make HTTP request to http://127.0.0.1:11434
  Module: ai - Hides HTTP/JSON complexity, exposes embed/chat functions
  Time: 30min
  ```

- [ ] **Implement ai/ollama.rs - HTTP client**
  ```
  Files: src/ai/ollama.rs (new, ~100 LOC)
  Approach: Use reqwest::blocking::Client for HTTP requests
  Success: Can call Ollama /api/embeddings and /api/chat
  Test: Unit tests with mockito:
    - embed() calls correct endpoint with request body
    - chat() calls correct endpoint, parses response
    - Error handling (Ollama offline, wrong model)
  Module: Ollama HTTP API client, hides JSON serialization
  Time: 2hr
  Public API:
    pub struct OllamaClient { /* private */ }
    pub fn new() -> Self
    pub fn embed(&self, model: &str, text: &str) -> AppResult<Vec<f32>>
    pub fn chat(&self, model: &str, messages: &[Message]) -> AppResult<String>
  ```

- [ ] **Implement ai/chunking.rs - Text chunking**
  ```
  Files: src/ai/chunking.rs (new, ~60 LOC)
  Approach: Simple word-based chunking with overlap
  Success: Text split into ~700-word chunks with 100-word overlap
  Test: Unit tests for:
    - chunk_text() splits correctly
    - Overlap works (last 100 words of chunk N = first 100 of chunk N+1)
    - Edge cases (text < chunk_size, empty text)
  Module: Chunking logic, simple sliding window
  Time: 1hr
  Public API:
    pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String>
  ```

- [ ] **Implement ai/prompts.rs - System prompts**
  ```
  Files: src/ai/prompts.rs (new, ~80 LOC)
  Approach: Const strings + builder functions
  Success: Prompts generate correct Message structs
  Test: Unit tests for:
    - reflect_prompt() includes entry content
    - ask_prompt() includes context chunks
    - Message formatting correct
  Module: Prompt templates, hides string formatting
  Time: 1hr
  Public API:
    pub const SYSTEM_PROMPT: &str = "...";
    pub fn reflect_prompt(entry_content: &str) -> Vec<Message>
    pub fn ask_prompt(question: &str, context_chunks: &[String]) -> Vec<Message>
  ```

- [ ] **Extend errors/mod.rs with AIError**
  ```
  Files: src/errors/mod.rs:~270-290
  Approach: Follow DatabaseError pattern
  Success: AI errors wrapped in AppError
  Test: Error messages helpful (suggest ollama serve)
  Module: Extends error hierarchy
  Time: 20min
  New variants:
    #[derive(Debug, Error)]
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

- [ ] **Update constants.rs with AI defaults**
  ```
  Files: src/constants.rs:~105-115
  Approach: Add AI-related constants
  Success: Constants used in ai module
  Test: N/A (constants)
  Module: N/A (shared constants)
  Time: 5min
  New constants:
    pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
    pub const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";
    pub const DEFAULT_CHAT_MODEL: &str = "llama3.2:3b";
    pub const DEFAULT_CHUNK_SIZE: usize = 700;
    pub const DEFAULT_CHUNK_OVERLAP: usize = 100;
  ```

- [ ] **Add ai module to lib.rs exports**
  ```
  Files: src/lib.rs:69-71
  Approach: Add `pub mod ai;` after db
  Success: `use ponder::ai::OllamaClient;` works
  Test: Can import in tests
  Module: N/A (public API)
  Time: 5min
  ```

### Integration Tests

- [ ] **Create tests/ai_tests.rs - AI integration tests**
  ```
  Files: tests/ai_tests.rs (new, ~200 LOC)
  Approach: Use mockito for HTTP mocking
  Success: All AI operations work with mocked responses
  Test: Integration tests for:
    - OllamaClient embed() with mock server
    - OllamaClient chat() with mock server
    - Chunking preserves content
    - Prompt generation correct
    - Error handling (mock 500 errors, timeouts)
  Module: N/A (test suite)
  Time: 2hr
  Note: Real Ollama tests in separate manual test suite
  ```

---

## Phase 4: High-Level Operations (Weeks 8-9)

### Module: ops/ (User-Facing Operations)

- [ ] **Create ops module structure**
  ```
  Files:
    src/ops/mod.rs (new, public API)
    src/ops/edit.rs (new, edit with encryption + embedding)
    src/ops/ask.rs (new, RAG query)
    src/ops/reflect.rs (new, entry reflection)
    src/ops/search.rs (new, semantic search)
  Approach: Follow existing module structure
  Success: Module compiles, operations callable
  Test: Module structure correct
  Module: ops - Orchestrates crypto + db + ai, hides coordination logic
  Time: 20min
  ```

- [ ] **Implement ops/edit.rs - Edit with encryption + embedding**
  ```
  Files: src/ops/edit.rs (new, ~200 LOC)
  Approach: Combine crypto, db, ai modules for full edit flow
  Success: Entry encrypted on close, embeddings generated
  Test: Integration test:
    - Edit creates encrypted .age file
    - Database updated with entry metadata
    - Embeddings generated for chunks
    - Temp file cleaned up
    - Can decrypt and re-read entry
  Module: Edit operation orchestration, hides multi-step workflow
  Time: 3hr
  Public API:
    pub fn edit_entry(config: &Config, db: &Database, session: &mut SessionManager,
                      ai_client: &OllamaClient, date: NaiveDate,
                      reference_datetime: &DateTime<Local>) -> AppResult<()>
  ```

- [ ] **Implement ops/ask.rs - RAG query**
  ```
  Files: src/ops/ask.rs (new, ~150 LOC)
  Approach: Embed query → vector search → decrypt chunks → LLM call
  Success: Question answered with citations
  Test: Integration test:
    - Query embedding generated
    - Similar chunks found
    - Chunks decrypted
    - LLM response coherent
    - No plaintext files left
  Module: RAG pipeline, hides vector search + decryption coordination
  Time: 2.5hr
  Public API:
    pub fn ask_question(config: &Config, db: &Database, session: &mut SessionManager,
                        ai_client: &OllamaClient, question: &str,
                        window_days: Option<i64>) -> AppResult<String>
  ```

- [ ] **Implement ops/reflect.rs - Entry reflection**
  ```
  Files: src/ops/reflect.rs (new, ~100 LOC)
  Approach: Decrypt entry → LLM reflection → save encrypted report
  Success: Reflection generated and saved as .age file
  Test: Integration test:
    - Entry decrypted
    - Reflection prompt correct
    - LLM response formatted
    - Report saved encrypted
  Module: Reflection operation, hides prompt generation + report saving
  Time: 2hr
  Public API:
    pub fn reflect_on_entry(config: &Config, db: &Database,
                           session: &mut SessionManager, ai_client: &OllamaClient,
                           date: Option<NaiveDate>) -> AppResult<String>
  ```

- [ ] **Implement ops/search.rs - Semantic search**
  ```
  Files: src/ops/search.rs (new, ~120 LOC)
  Approach: Embed query → vector search → decrypt snippets → format results
  Success: Search results ranked by relevance with citations
  Test: Integration test:
    - Query embedding generated
    - Similar chunks ranked
    - Snippets extracted and formatted
    - No plaintext files left
  Module: Search operation, hides ranking + snippet extraction
  Time: 2hr
  Public API:
    pub fn search_entries(config: &Config, db: &Database,
                         session: &mut SessionManager, ai_client: &OllamaClient,
                         query: &str, window_days: Option<i64>)
                         -> AppResult<Vec<SearchResult>>
  ```

- [ ] **Add ops module to lib.rs exports**
  ```
  Files: src/lib.rs:72-74
  Approach: Add `pub mod ops;` after ai
  Success: `use ponder::ops;` works
  Test: Can import in main.rs
  Module: N/A (public API)
  Time: 5min
  ```

### Integration Tests

- [ ] **Create tests/ops_integration_tests.rs - End-to-end operations**
  ```
  Files: tests/ops_integration_tests.rs (new, ~300 LOC)
  Approach: Full workflow tests with temp dirs, mock Ollama
  Success: All operations work end-to-end
  Test: Integration tests for:
    - edit_entry() full flow (create, edit, re-encrypt, embed)
    - ask_question() RAG pipeline
    - reflect_on_entry() reflection generation
    - search_entries() semantic search
    - Error recovery (Ollama offline, wrong passphrase)
  Module: N/A (test suite)
  Time: 3hr
  ```

---

## Phase 5: CLI Integration (Weeks 10-11)

### CLI Refactoring

- [ ] **Refactor cli/mod.rs to support subcommands**
  ```
  Files: src/cli/mod.rs:~30-80
  Approach: Change CliArgs to enum with subcommands (Edit, Ask, Reflect, Search)
  Success: `ponder ask "question"` parses correctly
  Test: Unit tests for all subcommand parsing
  Module: CLI argument parsing, hides clap complexity
  Time: 2hr
  New structure:
    #[derive(Parser)]
    pub enum PonderCli {
        Edit { date, retro, reminisce },
        Ask { question, window },
        Reflect { date },
        Search { query, window },
        Lock,
    }
  ```

- [ ] **Update main.rs with command dispatch**
  ```
  Files: src/main.rs:~50-150
  Approach: Match on PonderCli enum, call ops:: functions
  Success: All commands work, session unlocked on first use
  Test: Integration tests for each command
  Module: Command dispatcher, orchestrates config + session + ops
  Time: 2.5hr
  Flow:
    - Parse CLI
    - Load config
    - Initialize session (unlock on first command)
    - Open database
    - Initialize AI client
    - Dispatch to ops:: based on command
  ```

- [ ] **Extend config/mod.rs with v2.0 settings**
  ```
  Files: src/config/mod.rs:~52-66
  Approach: Add fields to Config struct, load from env vars
  Success: Config loads new settings, validates paths
  Test: Unit tests for new config fields
  Module: Config loading, adds new fields while preserving existing
  Time: 1hr
  New fields:
    pub db_path: PathBuf,  // PONDER_DB or journal_dir/ponder.db
    pub session_timeout_minutes: u64,  // PONDER_SESSION_TIMEOUT or 30
    pub ollama_url: String,  // OLLAMA_URL or http://127.0.0.1:11434
  ```

### Polish & Error Handling

- [ ] **Improve error messages for common failures**
  ```
  Files: src/errors/mod.rs:~300-350
  Approach: Add context to error Display impls
  Success: Errors suggest actionable fixes
  Test: Error message clarity review
  Module: Error display, adds helpful context
  Time: 1.5hr
  Improvements:
    - CryptoError::VaultLocked → suggest unlocking
    - AIError::OllamaOffline → suggest `ollama serve`
    - AIError::ModelNotFound → suggest `ollama pull {model}`
    - DatabaseError → suggest checking passphrase
  ```

- [ ] **Add validation for encrypted journal directory structure**
  ```
  Files: src/ops/edit.rs:~20-40
  Approach: Check for .age files, create year/month subdirs
  Success: Encrypted files organized as YYYY/MM/DD.md.age
  Test: Unit test for directory creation
  Module: Directory structure management
  Time: 1hr
  ```

---

## Phase 6: Testing & Documentation (Week 12)

### Comprehensive Testing

- [ ] **Add security-focused integration tests**
  ```
  Files: tests/security_tests.rs (new, ~200 LOC)
  Approach: Verify no plaintext leakage, permissions correct
  Success: All security checks pass
  Test:
    - Encrypted files unreadable without passphrase
    - Temp files cleaned up even on panic
    - File permissions 0o600 (Unix)
    - No plaintext in database
    - Session timeout enforced
    - Passphrase zeroization (inspect memory if possible)
  Module: N/A (security test suite)
  Time: 2.5hr
  ```

- [ ] **Add performance benchmarks**
  ```
  Files: benches/crypto_bench.rs (new, ~100 LOC)
  Approach: Use criterion crate for benchmarking
  Success: Benchmarks run, baseline established
  Test: Benchmarks for:
    - File encryption/decryption (various sizes)
    - Vector search (various DB sizes)
    - Embedding generation (mock)
  Module: N/A (benchmark suite)
  Time: 1.5hr
  Note: Add criterion to dev-dependencies
  ```

- [ ] **Run full test suite validation**
  ```
  Commands:
    cargo test --verbose -- --test-threads=1
    cargo test --test security_tests
    cargo test --test ops_integration_tests
  Success: All tests pass, >85% coverage
  Time: 30min
  ```

### Documentation

- [ ] **Update README.md with v2.0 features**
  ```
  Files: README.md:~50-150
  Approach: Add encryption, AI, search sections
  Success: README clear, installation updated
  Test: Follow README from scratch (manual)
  Module: N/A (documentation)
  Time: 1.5hr
  Sections to add:
    - Encryption (age, SQLCipher)
    - AI features (Ollama setup, models)
    - New commands (ask, reflect, search)
    - Configuration (new env vars)
  ```

- [ ] **Create MIGRATION.md guide**
  ```
  Files: MIGRATION.md (new, ~100 lines)
  Approach: Step-by-step migration from v1.0
  Success: Migration guide clear, tested
  Test: Follow guide with sample v1.0 journal
  Module: N/A (documentation)
  Time: 1hr
  Sections:
    - Backup instructions
    - Ollama setup
    - ponder setup wizard
    - Migration verification
    - Rollback procedure
  ```

- [ ] **Update CLAUDE.md with v2.0 architecture**
  ```
  Files: CLAUDE.md:~150-250
  Approach: Add new modules, update build commands
  Success: CLAUDE.md reflects current architecture
  Test: Review accuracy
  Module: N/A (documentation)
  Time: 30min
  Updates:
    - New module structure (crypto, db, ai, ops)
    - New dependencies
    - New test structure
    - Security considerations
  ```

---

## Design Iteration Checkpoints

### After Phase 2 (Database)
- Review: Are vector search results accurate enough? Need hybrid FTS+vector?
- Consider: SQLite-VSS extension for better performance?
- Refactor: Extract common DB patterns into helper functions?

### After Phase 4 (Operations)
- Review: Is the ops:: module too shallow? Should we have domain-specific modules?
- Consider: Caching embeddings in memory for hot entries?
- Refactor: Extract common decrypt→process→encrypt pattern?

### After Phase 6 (Complete)
- Review: Module boundaries clear? Any leaky abstractions?
- Consider: Performance optimizations based on benchmarks?
- Plan: v2.1 features based on user feedback

---

## Automation Opportunities

1. **Pre-commit hook for security checks**: Scan for hardcoded secrets, check file permissions
2. **Benchmark CI**: Run benchmarks on PRs, track performance regressions
3. **Encryption test helper**: Macro for common encrypt→process→decrypt test pattern
4. **Mock Ollama server**: Reusable test fixture for AI integration tests

---

## BACKLOG (Post-v2.0)

These are validated but not blocking v2.0 release:

- [ ] **Setup wizard** (`ponder setup` command) - interactive first-time config
- [ ] **Migration tool** (`ponder migrate --from ~/old`) - v1.0 → v2.0 migration
- [ ] **Nightly indexing** (`ponder nightly`) - batch re-embedding
- [ ] **Weekly review** (`ponder weekly`) - aggregated insights
- [ ] **Keyring integration** - store passphrase in system keyring (optional)
- [ ] **X25519 identity support** - SSH-key-based encryption (age identities)
- [ ] **Hybrid search** - combine vector + FTS for better results
- [ ] **Export command** - decrypt to PDF/HTML
- [ ] **Statistics** - streak tracking, word counts, trends

---

## Validation Checklist

Before v2.0 release:

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --verbose -- --test-threads=1` all pass
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo audit` clean (no vulnerabilities)
- [ ] All integration tests pass
- [ ] Security tests pass
- [ ] README accurate and complete
- [ ] MIGRATION.md tested manually
- [ ] Performance meets targets (see TASK.md Part V)
- [ ] Ollama integration tested manually (real models)

---

**Total Estimated Time**: 10-12 weeks full-time

**Critical Path**: Phase 0 → Phase 1 (crypto) → Phase 2 (db) → Phase 3 (ai) → Phase 4 (ops) → Phase 5 (CLI) → Phase 6 (testing/docs)

**Parallelization Opportunities**:
- Phase 1-2: crypto and db can be developed independently
- Phase 3: ai can be developed while db is in testing
- Phase 4: ops submodules (ask, reflect, search) can be parallelized
- Phase 6: Documentation can start during Phase 5
