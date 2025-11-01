# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

**Build:**
```bash
cargo build
cargo build --release  # For production build
```

**Run tests:**
```bash
cargo test                    # Run all tests
cargo test --bin ponder      # Run tests for main binary
cargo test test_name         # Run a single test by name
cargo test --test cli_tests  # Run specific integration test file
```

**Lint and format:**
```bash
cargo fmt               # Format code
cargo fmt --check      # Check formatting without modifying
cargo clippy --all-targets -- -D warnings  # Run linter with strict settings
```

**Pre-commit hooks:**
```bash
pre-commit install     # Install hooks (requires Python)
pre-commit run --all-files  # Run hooks manually
```

## High-Level Architecture

Ponder v2.0 is an AI-powered encrypted journaling CLI tool built in Rust. The architecture follows a modular design with clear separation of concerns: CLI parsing, configuration, cryptography, database, AI operations, and high-level orchestration.

### Core Modules (v2.0)

- **`src/cli/`**: Command-line interface using clap with subcommand architecture.
  - Implements `PonderCommand` enum: Edit, Ask, Reflect, Search, Lock
  - `EditArgs`: Supports retro/reminisce/date modes
  - `AskArgs`, `ReflectArgs`, `SearchArgs`: AI operation parameters
  - Redacts sensitive journal queries in Debug output

- **`src/config/`**: Configuration management with v2.0 settings.
  - Environment variables: `PONDER_DIR`, `PONDER_EDITOR`, `PONDER_DB`, `PONDER_SESSION_TIMEOUT`, `OLLAMA_URL`
  - Validates paths, editor commands (strict security), and database locations
  - Default: 30min session timeout, local Ollama at http://127.0.0.1:11434

- **`src/errors/`**: Comprehensive error handling using `thiserror`.
  - `AppError`: Top-level error type with variants for Config, Crypto, Database, AI, Editor, Lock, I/O
  - Specialized error types: `CryptoError`, `DatabaseError`, `AIError`, `EditorError`, `LockError`
  - User-friendly error messages with actionable guidance (e.g., "Try: ollama pull {model}")

- **`src/crypto/`**: Age encryption and secure session management.
  - `age.rs`: File encryption/decryption with age passphrase-based encryption
  - `session.rs`: SessionManager with auto-lock timeout, secure passphrase caching (zeroization)
  - `temp.rs`: Secure temporary file handling with 0o600 permissions, automatic cleanup

- **`src/db/`**: SQLCipher encrypted database for metadata and embeddings.
  - Connection pooling with r2d2
  - `entries.rs`: Journal entry metadata (path, date, checksum, word count)
  - `embeddings.rs`: Vector embeddings with HNSW index for semantic search
  - Pragmas: journal_mode=WAL, synchronous=NORMAL, temp_store=MEMORY

- **`src/ai/`**: Ollama integration for embeddings and LLM operations.
  - `client.rs`: OllamaClient for API interactions (embed, chat)
  - `chunking.rs`: Text chunking with configurable size/overlap (default: 512/50 chars)
  - `prompts.rs`: Structured prompts for ask (RAG) and reflect operations
  - Models: nomic-embed-text (embeddings), gemma3:4b (chat)

- **`src/ops/`**: High-level user-facing operations combining crypto, DB, and AI.
  - `edit.rs`: Edit encrypted entries (YYYY/MM/DD.md.age), auto-generate embeddings on change
  - `ask.rs`: RAG query pipeline (embed → vector search → decrypt → LLM)
  - `reflect.rs`: AI reflection on journal entries
  - `search.rs`: Semantic search returning ranked excerpts with similarity scores

- **`src/journal_core/`**: Pure date logic (v1.0 compatibility).
  - `DateSpecifier`: Today, Retro, Reminisce, Specific date modes
  - Date resolution without I/O

- **`src/journal_io/`**: Legacy v1.0 journal I/O (maintained for compatibility).

- **`src/main.rs`**: Application entry point with command dispatch.
  1. Parse CLI subcommand
  2. Load and validate config
  3. Dispatch to command handler (cmd_edit, cmd_ask, cmd_reflect, cmd_search, cmd_lock)
  4. Each handler initializes: SessionManager → Database → OllamaClient → ops:: function

### Module Flow (v2.0)

```
main.rs
  ├─> cli/mod.rs (parse subcommand: Edit|Ask|Reflect|Search|Lock)
  ├─> config/mod.rs (load config with v2.0 settings)
  └─> Command dispatch:

      cmd_edit (edit subcommand):
        ├─> journal_io::ensure_journal_directory_exists()
        ├─> journal_core::DateSpecifier (resolve dates)
        ├─> crypto::SessionManager (unlock session)
        ├─> db::Database::open() (encrypted SQLite)
        ├─> ai::OllamaClient (for embeddings)
        └─> ops::edit_entry() for each date
              ├─> decrypt_to_temp() or create new
              ├─> launch_editor()
              ├─> encrypt_from_temp()
              ├─> db::entries::upsert_entry()
              └─> generate embeddings if content changed

      cmd_ask (RAG query):
        ├─> SessionManager + Database + OllamaClient
        └─> ops::ask_question()
              ├─> ai_client.embed(question)
              ├─> db::embeddings::search_similar_chunks()
              ├─> decrypt matching entries
              └─> ai_client.chat(context + question)

      cmd_reflect (AI reflection):
        ├─> SessionManager + Database + OllamaClient
        └─> ops::reflect_on_entry()
              ├─> db::entries::get_entry_by_date()
              ├─> decrypt entry
              └─> ai_client.chat(reflection prompt)

      cmd_search (semantic search):
        ├─> SessionManager + Database + OllamaClient
        └─> ops::search_entries()
              ├─> ai_client.embed(query)
              ├─> db::embeddings::search_similar_chunks()
              ├─> decrypt matching chunks
              └─> return SearchResults (date, excerpt, score)

      cmd_lock:
        └─> SessionManager::lock() (clear passphrase)

      cmd_converse (conversational interface - v2.1):
        ├─> SessionManager + Database + OllamaClient
        └─> ops::start_conversation()
              ├─> Interactive loop: read user input from stdin
              ├─> For each question:
              │   ├─> assemble_conversation_context() (RAG pipeline)
              │   │   ├─> ai_client.embed(question)
              │   │   ├─> db::embeddings::search_similar_chunks()
              │   │   ├─> decrypt matching entries
              │   │   └─> return (date, excerpt) pairs
              │   ├─> Build prompt with CoT system message + history + context
              │   ├─> ai_client.chat() (non-streaming for MVP)
              │   └─> Add question + response to in-memory history
              └─> Exit on "quit", "exit", or empty input
```

### Conversational Operations (v2.1)

**Philosophy**: Modern LLMs (2025) excel at natural language reasoning with rich context.
Instead of building 2016-style sentiment classifiers and statistical pattern detectors,
Ponder v2.1 provides a conversational interface where users explore their journal through
natural dialogue.

**High-Level Flow**:
1. User runs `ponder converse` to start interactive chat
2. Each question triggers RAG context assembly from encrypted journal
3. AI responds using Chain-of-Thought reasoning with specific entry citations
4. Conversation history maintained in-memory for multi-turn continuity
5. User can quit anytime with "quit", "exit", or empty input

**Example Interaction**:
```
$ ponder converse

🤖 Ponder Conversational Assistant
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Ask me anything about your journal entries!
I'll think through your questions step-by-step and cite specific entries.

You: Do you notice any emotional patterns in my recent entries?

🤖 Assistant: Let me think through this step-by-step...

First, I'm noticing a recurring theme of presentation anxiety in your entries
from January 15-16. You mentioned feeling "anxious" and "nervous" about an
upcoming technical presentation.

Second, I see a clear emotional arc: the anxiety transformed into confidence
after the presentation on January 20. You wrote "it felt natural" and noted
feeling "really proud."

Third, this suggests a pattern of anticipatory anxiety that resolves positively
through preparation and action. The January 21 entry shows you recognizing this
pattern yourself.

However, I should note this is based on a limited window. A longer timeframe
would show whether this is a consistent pattern.

Would you like to explore how you typically handle anxiety-inducing situations?
```

**Extends Existing RAG**:
- Reuses `db::embeddings::search_similar_chunks()` for vector search
- Reuses `crypto::age::decrypt_with_passphrase()` for entry decryption
- Reuses `ai::chunking::chunk_text()` for chunk extraction
- New: `assemble_conversation_context()` orchestrates these into (date, excerpt) pairs
- New: Chain-of-Thought system prompt encourages reasoning visibility

**Implementation Details**:
- **No new tables**: Uses existing `embeddings` table for vector search
- **No wrapper modules**: Direct calls to Ollama API via existing `ai::OllamaClient`
- **Streaming**: Non-streaming for MVP (can add later via Ollama native `stream=true`)
- **Context window**: Maintains last 20 messages + system prompt (auto-pruned)
- **Security**: Session timeout still applies (default 30min)

**Design Rationale** (per ultrathink analysis):
- Avoids shallow wrapper modules (streaming.rs, context.rs, memory.rs)
- Avoids redundant storage (conversation_memory table duplicates embeddings)
- Single deep module (ops/converse.rs) with simple public interface
- Information hiding: All RAG complexity hidden behind `start_conversation()`
- Extends 80% of existing infrastructure vs rebuilding from scratch

### Key Design Decisions (v2.0)

**Security:**
- Age passphrase-based encryption for journal files (.md.age extension)
- SQLCipher for encrypted database (metadata + embeddings)
- Session manager with auto-lock timeout (configurable, default 30min)
- Secure temp files with 0o600 permissions, automatic cleanup on panic
- Passphrase zeroization via SecretString
- Strict editor command validation (no spaces, args, shell metacharacters)

**Architecture:**
- Direct function calls instead of trait abstractions (simplicity)
- Modular design: crypto, db, ai, ops modules with clear boundaries
- Separation of pure logic (journal_core) from I/O (journal_io, ops)
- Deep modules: Simple interfaces hiding complex implementations
- No information leakage: Return domain objects, not raw DB rows

**Storage:**
- Encrypted entries: YYYY/MM/DD.md.age directory structure
- SQLite database: entry metadata + vector embeddings
- BLAKE3 checksums for content change detection
- Automatic embedding generation on content change only

**AI Integration:**
- Local Ollama for embeddings (nomic-embed-text) and chat (gemma3:4b)
- Text chunking: 512 chars with 50 char overlap
- Vector search with cosine similarity
- RAG pipeline: embed query → vector search → decrypt → LLM
- Graceful degradation when Ollama offline

**v1.0 Compatibility:**
- Legacy journal_io module maintained
- DateSpecifier (Today/Retro/Reminisce/Specific) preserved
- Edit command supports all v1.0 date modes

## Testing Structure (v2.0)

### Unit Tests
- Colocated with modules using `#[cfg(test)]` modules
- 129 unit tests covering:
  - `crypto/`: Encryption, decryption, session management, temp file handling
  - `db/`: Database operations, entries, embeddings, vector search
  - `ai/`: Chunking, prompts (LLM tests require Ollama)
  - `ops/`: Directory structure validation
  - `cli/`: Subcommand parsing (12 tests)
  - `config/`: v2.0 configuration loading
  - `errors/`: Error types, messages, source chaining (20 tests)
  - `journal_core/`: DateSpecifier resolution

### Integration Tests (`tests/`)
- `cli_tests.rs`: CLI argument parsing (v1.0)
- `journal_integration_tests.rs`: Full journal operations (v1.0)
- `config_tests.rs`: Configuration loading with v2.0 env vars
- `editor_error_integration_tests.rs`: Robust error handling patterns
- `locking_tests.rs`: File locking mechanisms
- `ops_integration_tests.rs`: Full v2.0 pipeline tests (requires Ollama)
- `security_tests.rs`: Security-focused tests (encryption, permissions, cleanup)

### Test Dependencies
- `tempfile`: Isolated filesystem operations
- `rusqlite`: In-memory databases for testing
- Mock `EDITOR` environment variable (use `echo` or `true` without args)
- **Ollama required** for AI integration tests (optional, can skip)

### Error Testing Best Practices

The project uses robust error testing patterns to prevent test brittleness:

**✅ Good practices:**
```rust
// Focus on user-visible behavior, not implementation details
assert!(stderr.contains("not found"));
assert!(stderr.contains(&command_name));

// Test error type propagation
match result {
    Err(AppError::Config(_)) => {}, // Expected
    other => panic!("Expected Config error, got: {:?}", other),
}

// Use pattern matching for key components
assert!(error_msg.contains("Configuration error"));
assert!(error_msg.contains("shell metacharacters"));
```

**❌ Avoid brittle patterns:**
```rust
// Don't test enum variant names in user output
assert!(stderr.contains("CommandNotFound"));

// Don't use exact string matching
assert_eq!(error_msg, "exact error message here");

// Don't test implementation details
assert!(error_msg.contains("EditorError::"));
```

### Test Execution

For reliable test execution, especially when debugging concurrency issues:
```bash
cargo test -- --test-threads=1  # Run tests sequentially
```

Concurrent test execution can cause file locking conflicts in integration tests.

## Pre-commit and CI

The project uses pre-commit hooks and GitHub Actions CI:
- Pre-commit runs `cargo fmt --check` and `cargo clippy`
- CI runs formatting, clippy, build, and tests in separate jobs
- Test code must meet same quality standards as production code

## Dependencies (v2.0)

### Core Dependencies
- **Encryption:** `age` (passphrase-based), `age-core` for streaming
- **Database:** `rusqlite` with `sqlcipher` feature, `r2d2` connection pool
- **AI:** `reqwest` for Ollama HTTP API, `serde_json` for JSON parsing
- **CLI:** `clap` with derive feature for subcommands
- **Hashing:** `blake3` for checksums
- **Utilities:** `chrono` (dates), `uuid` (temp files), `tracing`/`tracing-subscriber` (logging)

### Security Dependencies
- `age::secrecy::SecretString`: Zeroizing passphrase storage
- `sqlcipher`: SQLite with transparent 256-bit AES encryption
- `tempfile`: Secure temporary file creation with automatic cleanup

### Development Dependencies
- `tempfile`: Isolated test environments
- `criterion`: Performance benchmarking
- Pre-commit hooks: Python-based (cargo fmt, cargo clippy)

## Security Considerations

### Threat Model
- **Protects against:** Unauthorized file system access, memory dumps, swap
- **Does NOT protect against:** Malicious code execution, kernel-level attacks, physical RAM access with sophisticated tools

### Security Properties
1. **Encryption at rest:** All journal content encrypted with age, database encrypted with SQLCipher
2. **Passphrase security:** Zeroized on drop, locked after timeout
3. **Temp file safety:** 0o600 permissions, cleaned up even on panic
4. **No plaintext leakage:** Database stores only encrypted paths and embeddings (not plaintext)
5. **Metadata protection:** Entry dates/checksums encrypted in SQLCipher database

### Security Assumptions
- User's system is trusted (no keyloggers, screen capture malware)
- Ollama instance is trusted (localhost recommended)
- User chooses strong passphrase (no validation currently)
- File system permissions are respected by OS

### Known Limitations
- Embeddings are semantic vectors (could leak topic information to Ollama)
- Session timeout relies on system time (could be manipulated)
- No passphrase strength validation
- No rate limiting on passphrase attempts