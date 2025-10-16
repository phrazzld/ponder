# Ponder v2.0: AI-Powered Encrypted Journaling System

**Status**: Specification Complete | **Target**: Q2 2025 MVP
**Current LOC**: ~1,900 production + ~2,200 tests = ~4,100 total
**Target LOC**: ~7,500 production + ~3,500 tests = ~11,000 total
**Effort Estimate**: 8-12 weeks full-time development

---

## Executive Summary

Transform Ponder from a simple markdown journaling CLI into a **security-first, AI-assisted, local-only** journaling system. Preserve the elegant simplicity of the current architecture while adding encryption (age), semantic search (SQLCipher + embeddings), and LLM-powered insights (Ollama integration).

**Core Value Propositions**:
1. **Privacy-First**: Everything encrypted at rest, local AI processing only
2. **Semantic Search**: Find entries by meaning, not just keywords
3. **AI Insights**: Reflect, summarize, identify patterns—all offline
4. **Git-Native**: Encrypted files are git-friendly for secure sync
5. **Zero-Config AI**: Works with Ollama out of the box

**Target Users**:
- Privacy-conscious journalers who want AI assistance without cloud compromise
- Developers comfortable with CLI tools and local LLM setup
- Power users seeking semantic search across years of journals
- Anyone wanting encrypted, syncable journal archives

---

## Part I: Current Architecture Analysis

### Existing Codebase (v1.0)

**Module Structure** (~1,900 LOC production code):

```rust
src/
├── main.rs                    // 206 lines - App orchestration + correlation IDs
├── cli/mod.rs                 // 92 lines  - Clap CLI argument parsing
├── config/mod.rs              // 285 lines - Env var loading, security validation
├── errors/mod.rs              // 258 lines - Structured error types (EditorError, LockError)
├── journal_core/mod.rs        // 228 lines - Pure date logic (DateSpecifier)
├── journal_io/mod.rs          // 701 lines - File I/O, editor launching, file locking
├── constants.rs               // 85 lines  - Centralized constants
└── lib.rs                     // 45 lines  - Public API exports
```

**Test Coverage** (~2,200 LOC):
- Unit tests colocated in each module (`#[cfg(test)]`)
- Integration tests in `tests/` directory
- File locking tests, security validation, error propagation

**Key Strengths to Preserve**:

1. **Clean Architecture**:
   - Pure logic (`journal_core`) separated from I/O (`journal_io`)
   - Excellent error handling with context-rich types
   - Structured logging with correlation IDs
   - Secure by default (0o600 file perms, editor validation)

2. **Well-Tested**:
   - File locking prevents concurrent access
   - Security validation (no shell injection in editor commands)
   - Error propagation maintains full context

3. **Simple Data Model**:
   ```
   ~/Documents/rubberducks/
     20250515.md  # Plain markdown
     20250514.md
     20250513.md
   ```

**Current User Flow**:
```
$ ponder                    # Opens today's entry in $EDITOR
$ ponder --retro            # Opens past 7 days (existing entries only)
$ ponder --reminisce        # Opens 1m, 3m, 6m, 1y ago entries
$ ponder --date 2024-01-15  # Opens specific date
```

**What Works Well** (Keep These):
- Direct editor integration (no custom UI)
- Markdown-first (portable, future-proof)
- File locking (prevents corruption)
- Security-conscious config validation
- Structured logging for debugging
- Simple CLI with clear options

**What's Missing** (Add in v2.0):
- Encryption (files readable by anyone with disk access)
- Search (no way to find entries except manual grep)
- AI insights (no pattern recognition or summaries)
- Semantic search (keyword-only via grep is insufficient)
- Sync-friendly encryption (plaintext .md files exposed in backups)

---

## Part II: Target Architecture

### High-Level Design

**Core Principle**: **Encrypt everything, decrypt late, process fast, forget early**

```
User Flow (v2.0):
┌─────────────────────────────────────────────────────────────────┐
│                         ponder CLI                               │
├─────────────────────────────────────────────────────────────────┤
│ 1. ponder                                                        │
│    → Unlock vault (get session key)                             │
│    → Create 20250515.md.age if new                              │
│    → Decrypt to /dev/shm/ponder/20250515.md (tmpfs)             │
│    → Launch $EDITOR on temp file                                │
│    → On editor exit: Re-encrypt to .age                         │
│    → Update ponder.db with new embeddings                       │
│    → Zeroize temp file and memory buffers                       │
│                                                                  │
│ 2. ponder ask "What energized me this week?"                    │
│    → Embed query with nomic-embed-text                          │
│    → Vector search in ponder.db (top-k=12)                      │
│    → Stream-decrypt relevant entries to RAM                     │
│    → Call Ollama (llama3.2:3b) with context                     │
│    → Print answer to stdout                                     │
│    → Zeroize decrypted content                                  │
│                                                                  │
│ 3. ponder reflect                                               │
│    → Analyze today's entry with LLM                             │
│    → Generate summary + 2-3 reflection prompts                  │
│    → Write encrypted report to reports/2025-05-15.md.age        │
│                                                                  │
│ 4. ponder search "moments of clarity"                           │
│    → Embed search phrase                                        │
│    → Vector similarity search                                   │
│    → Decrypt & show relevant snippets with dates               │
│                                                                  │
│ 5. ponder nightly                                               │
│    → Detect changed entries (mtime/checksum)                    │
│    → Re-embed modified entries                                  │
│    → Generate yesterday's daily report                          │
└─────────────────────────────────────────────────────────────────┘
```

### Data Model

**File Structure** (Encrypted at Rest):
```
~/Journal/  # New default location (vs ~/Documents/rubberducks)
├── 2025/
│   └── 05/
│       ├── 15.md.age  # Encrypted journal entry (age passphrase)
│       ├── 14.md.age
│       └── 13.md.age
├── reports/
│   ├── 2025-05-15.md.age  # Encrypted daily reflection
│   └── 2025-W20.md.age    # Encrypted weekly review
└── ponder.db              # SQLCipher encrypted database
```

**Database Schema** (SQLCipher with AES-256):
```sql
-- Entry metadata (NO plaintext content stored)
CREATE TABLE entries (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,          -- Relative path: "2025/05/15.md.age"
    date TEXT NOT NULL,                 -- ISO date: "2025-05-15"
    checksum TEXT NOT NULL,             -- BLAKE3 hash of encrypted file
    word_count INTEGER,                 -- Approximate (from last embedding)
    updated_at INTEGER NOT NULL,        -- Unix timestamp
    embedded_at INTEGER,                -- Last embedding generation time
    UNIQUE(date)
);

-- Vector embeddings (detached from plaintext for security)
CREATE TABLE embeddings (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL,
    chunk_idx INTEGER NOT NULL,        -- Chunk position within entry
    embedding BLOB NOT NULL,            -- nomic-embed-text vector (768 dims)
    checksum TEXT NOT NULL,             -- BLAKE3 of source chunk (for integrity)
    created_at INTEGER NOT NULL,
    FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE,
    UNIQUE(entry_id, chunk_idx)
);

-- Full-text search (encrypted keyword index)
CREATE VIRTUAL TABLE entries_fts USING fts5(
    entry_id UNINDEXED,
    date UNINDEXED,
    content,                            -- Encrypted keywords/stems
    tokenize = 'porter ascii'
);

-- Insights/summaries metadata
CREATE TABLE insights (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER,
    type TEXT NOT NULL,                 -- 'reflection', 'summary', 'theme'
    encrypted_content BLOB NOT NULL,    -- Age-encrypted JSON payload
    score REAL,                         -- Relevance/confidence score
    created_at INTEGER NOT NULL,
    FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE
);

-- Reports (daily/weekly summaries)
CREATE TABLE reports (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,          -- "reports/2025-05-15.md.age"
    type TEXT NOT NULL,                 -- 'daily', 'weekly'
    date_range TEXT NOT NULL,           -- "2025-05-15" or "2025-W20"
    created_at INTEGER NOT NULL
);
```

### Module Architecture

**New Directory Structure**:
```
src/
├── main.rs                    # Entry point (keep existing + add subcommands)
├── cli/                       # Argument parsing
│   ├── mod.rs                 # Existing clap + NEW subcommands
│   ├── commands.rs            # NEW: ask, reflect, search, nightly, etc.
│   └── args.rs                # NEW: Argument validators
├── config/
│   └── mod.rs                 # EXTEND: Add vault config, AI models, DB path
├── errors/
│   ├── mod.rs                 # EXTEND: Add CryptoError, DatabaseError, AIError
│   └── types.rs               # NEW: Specific error variants
├── constants.rs               # EXTEND: Add crypto params, AI defaults
├── journal_core/              # Keep existing pure logic
│   └── mod.rs                 # DateSpecifier, date resolution (unchanged)
├── journal_io/                # REFACTOR: Extract editor logic, add encryption
│   ├── mod.rs                 # Keep core file operations
│   ├── editor.rs              # NEW: Extracted launch_editor + temp handling
│   └── locking.rs             # NEW: Extracted file locking logic
├── crypto/                    # NEW: All encryption operations
│   ├── mod.rs                 # Public API
│   ├── age.rs                 # Age encryption/decryption (streaming)
│   ├── kdf.rs                 # Argon2id key derivation
│   ├── session.rs             # Session key management + auto-lock
│   └── temp.rs                # Secure temporary file handling (tmpfs)
├── db/                        # NEW: SQLCipher operations
│   ├── mod.rs                 # Public API + connection pool
│   ├── schema.rs              # Table definitions + migrations
│   ├── entries.rs             # Entry CRUD operations
│   ├── embeddings.rs          # Vector storage/retrieval
│   ├── fts.rs                 # Full-text search operations
│   └── reports.rs             # Report metadata
├── ai/                        # NEW: Ollama integration
│   ├── mod.rs                 # Public API
│   ├── ollama.rs              # HTTP client for Ollama API
│   ├── embeddings.rs          # Embedding generation (nomic-embed-text)
│   ├── chat.rs                # LLM chat completion (llama3.2:3b)
│   ├── chunking.rs            # Text chunking for embeddings
│   └── prompts.rs             # System prompts, templates
├── search/                    # NEW: Semantic + FTS search
│   ├── mod.rs                 # Public API
│   ├── vector.rs              # Cosine similarity, re-ranking
│   ├── keyword.rs             # FTS integration
│   └── hybrid.rs              # Combined vector + keyword search
├── ops/                       # NEW: High-level operations
│   ├── mod.rs                 # Public API
│   ├── edit.rs                # Decrypt → Edit → Re-encrypt + Embed
│   ├── ask.rs                 # RAG query implementation
│   ├── reflect.rs             # Entry reflection/summary
│   ├── search.rs              # User-facing search command
│   ├── nightly.rs             # Batch re-embedding + daily report
│   └── weekly.rs              # Weekly aggregation report
└── lib.rs                     # Public API exports
```

---

## Part III: Implementation Phases

### Phase 0: Foundation & Dependencies (Week 1)

**Goal**: Add dependencies, set up database, basic crypto

**New Dependencies** (Cargo.toml):
```toml
[dependencies]
# Existing (keep)
chrono = "0.4.26"
clap = { version = "4.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3.20", features = ["json", "env-filter", "fmt", "registry", "chrono"] }  # SECURITY: Upgrade from 0.3.19
uuid = { version = "1.4", features = ["v4", "fast-rng", "serde"] }
shellexpand = "3.1"
fs2 = "0.4.3"
tempfile = "3.3"

# NEW: Encryption
age = "0.10"
argon2 = "0.5"
zeroize = "1.7"
blake3 = "1.5"

# NEW: Database
rusqlite = { version = "0.31", features = ["bundled-sqlcipher", "blob"] }
r2d2 = "0.8"
r2d2_sqlite = "0.23"

# NEW: AI/Embeddings
reqwest = { version = "0.11", features = ["json", "blocking"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokenizers = "0.15"  # For chunking

# NEW: Optional keyring support
keyring = { version = "2.3", optional = true }

[dev-dependencies]
# Existing (keep)
assert_cmd = "2.0"
predicates = "2.1"
serial_test = "1.0"
serde_json = "1.0"
tempfile = "3.3"

# NEW: For AI/crypto testing
mockito = "1.2"
```

**Tasks**:
1. Fix CRITICAL: Upgrade `tracing-subscriber` from 0.3.19 → 0.3.20 (RUSTSEC-2025-0055)
2. Fix CRITICAL: Update `clap` API from deprecated `possible_values` → `value_parser`
3. Add new dependencies to Cargo.toml
4. Create `crypto/` module structure with empty implementations
5. Create `db/` module structure with SQLCipher connection test
6. Create `ai/` module structure with Ollama ping test
7. Update `constants.rs` with crypto/AI defaults
8. Extend `errors/mod.rs` with new error types

**Validation**:
- `cargo build` succeeds
- `cargo test` passes (existing tests unchanged)
- `cargo audit` clean
- `cargo clippy -- -D warnings` clean

---

### Phase 1: Encryption Core (Week 2-3)

**Goal**: Implement age encryption, session management, secure temp files

**Module: `crypto/`**

#### `crypto/age.rs` - Age Encryption

```rust
use age::secrecy::SecretString;
use std::io::{Read, Write};

/// Encrypt data using age with passphrase
pub fn encrypt_with_passphrase(
    plaintext: &[u8],
    passphrase: &SecretString,
) -> AppResult<Vec<u8>> {
    let encryptor = age::Encryptor::with_user_passphrase(passphrase.clone());
    let mut encrypted = Vec::new();
    let mut writer = encryptor.wrap_output(&mut encrypted)?;
    writer.write_all(plaintext)?;
    writer.finish()?;
    Ok(encrypted)
}

/// Decrypt age-encrypted data with passphrase
pub fn decrypt_with_passphrase(
    ciphertext: &[u8],
    passphrase: &SecretString,
) -> AppResult<Vec<u8>> {
    let decryptor = match age::Decryptor::new(ciphertext)? {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err(CryptoError::UnsupportedFormat.into()),
    };

    let mut decrypted = Vec::new();
    let mut reader = decryptor.decrypt(passphrase, None)?;
    reader.read_to_end(&mut decrypted)?;
    Ok(decrypted)
}

/// Streaming encryption for large files
pub fn encrypt_file_streaming(
    input_path: &Path,
    output_path: &Path,
    passphrase: &SecretString,
) -> AppResult<()> {
    // Implementation uses age streaming API
}

/// Streaming decryption for large files
pub fn decrypt_file_streaming(
    input_path: &Path,
    output_path: &Path,
    passphrase: &SecretString,
) -> AppResult<()> {
    // Implementation uses age streaming API
}
```

#### `crypto/session.rs` - Session Key Management

```rust
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

pub struct SessionManager {
    passphrase: Option<Zeroizing<SecretString>>,
    last_access: Option<Instant>,
    timeout: Duration,
}

impl SessionManager {
    pub fn new(timeout_minutes: u64) -> Self {
        Self {
            passphrase: None,
            last_access: None,
            timeout: Duration::from_secs(timeout_minutes * 60),
        }
    }

    /// Unlock vault with passphrase (prompt if not cached)
    pub fn unlock(&mut self) -> AppResult<&SecretString> {
        if self.is_locked() {
            self.prompt_passphrase()?;
        }
        self.last_access = Some(Instant::now());
        self.passphrase.as_ref()
            .map(|z| z.as_ref())
            .ok_or_else(|| CryptoError::VaultLocked.into())
    }

    /// Check if session has timed out
    pub fn is_locked(&self) -> bool {
        match (self.passphrase.as_ref(), self.last_access) {
            (Some(_), Some(last)) => last.elapsed() > self.timeout,
            _ => true,
        }
    }

    /// Explicitly lock vault and zeroize keys
    pub fn lock(&mut self) {
        if let Some(mut pass) = self.passphrase.take() {
            // Zeroizing automatically wipes on drop
        }
        self.last_access = None;
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.lock();
    }
}
```

#### `crypto/temp.rs` - Secure Temporary Files

```rust
use std::fs;
use std::path::{Path, PathBuf};

const TMPFS_PATHS: &[&str] = &["/dev/shm", "/run/shm"];

/// Get secure temporary directory (prefer tmpfs for RAM-only storage)
pub fn get_secure_temp_dir() -> AppResult<PathBuf> {
    // Try tmpfs first (Linux/BSD)
    for tmpfs in TMPFS_PATHS {
        let path = Path::new(tmpfs);
        if path.exists() && path.is_dir() {
            let ponder_tmp = path.join("ponder");
            fs::create_dir_all(&ponder_tmp)?;
            // Set 0o700 permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::Permissions::from_mode(0o700);
                fs::set_permissions(&ponder_tmp, perms)?;
            }
            return Ok(ponder_tmp);
        }
    }

    // Fallback to system temp (with security warning)
    tracing::warn!("tmpfs not available, using system temp (less secure)");
    let temp_dir = std::env::temp_dir().join("ponder");
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

/// Create temporary file, decrypt content, return path
pub fn decrypt_to_temp(
    encrypted_path: &Path,
    passphrase: &SecretString,
) -> AppResult<PathBuf> {
    let temp_dir = get_secure_temp_dir()?;
    let temp_file = temp_dir.join(
        encrypted_path.file_stem()
            .ok_or(CryptoError::InvalidPath)?
    );

    decrypt_file_streaming(encrypted_path, &temp_file, passphrase)?;

    // Set 0o600 permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&temp_file, perms)?;
    }

    Ok(temp_file)
}

/// Re-encrypt temp file, best-effort secure delete temp
pub fn encrypt_from_temp(
    temp_path: &Path,
    encrypted_path: &Path,
    passphrase: &SecretString,
) -> AppResult<()> {
    encrypt_file_streaming(temp_path, encrypted_path, passphrase)?;

    // Best-effort secure delete
    secure_delete(temp_path)?;
    Ok(())
}

/// Best-effort secure file deletion (overwrite + remove)
fn secure_delete(path: &Path) -> AppResult<()> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len() as usize;

    // Overwrite with zeros (not cryptographically secure but better than nothing)
    let zeros = vec![0u8; size.min(1024 * 1024)]; // 1MB chunks
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    for _ in 0..(size / zeros.len() + 1) {
        file.write_all(&zeros)?;
    }
    file.sync_all()?;
    drop(file);

    // Remove file
    fs::remove_file(path)?;
    Ok(())
}
```

**Integration with Existing Code**:

Modify `journal_io/mod.rs`:
```rust
// OLD (v1.0):
pub fn edit_journal_entries(
    config: &Config,
    dates: &[NaiveDate],
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    // ... existing logic ...
    let result = launch_editor(&config.editor, &paths_to_open);
    // ... existing logic ...
}

// NEW (v2.0 - with encryption):
pub fn edit_journal_entries_encrypted(
    config: &Config,
    session: &mut SessionManager,
    dates: &[NaiveDate],
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    // 1. Unlock vault
    let passphrase = session.unlock()?;

    // 2. For each encrypted entry, decrypt to temp
    let mut temp_paths = Vec::new();
    for date in dates {
        let encrypted_path = get_encrypted_entry_path(&config.journal_dir, *date);
        if encrypted_path.exists() {
            let temp_path = decrypt_to_temp(&encrypted_path, passphrase)?;
            temp_paths.push((encrypted_path, temp_path));
        } else if dates.len() == 1 {
            // Create new entry (decrypt empty content)
            let temp_path = create_temp_entry(*date, reference_datetime)?;
            temp_paths.push((encrypted_path, temp_path));
        }
    }

    // 3. Launch editor on temp files
    let temp_file_paths: Vec<_> = temp_paths.iter().map(|(_, t)| t.clone()).collect();
    let result = launch_editor(&config.editor, &temp_file_paths);

    // 4. Re-encrypt all temp files on success
    if result.is_ok() {
        for (encrypted_path, temp_path) in &temp_paths {
            encrypt_from_temp(temp_path, encrypted_path, passphrase)?;
        }
    }

    // 5. Cleanup happens via secure_delete in encrypt_from_temp
    result
}
```

**Testing**:
- Round-trip encryption/decryption
- Passphrase validation
- Session timeout behavior
- Secure delete verification (check temp dir is empty)
- Permission verification (0o600 for temps)

**Validation**:
- Encrypted files are not readable without passphrase
- Decrypted content matches original
- Session auto-locks after timeout
- Temp files are cleaned up even on error

---

### Phase 2: Database Layer (Week 4-5)

**Goal**: SQLCipher setup, schema, vector storage, FTS

**Module: `db/`**

#### `db/mod.rs` - Connection Pool

```rust
use rusqlite::Connection;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

type Pool = r2d2::Pool<SqliteConnectionManager>;

pub struct Database {
    pool: Pool,
}

impl Database {
    /// Open encrypted database with SQLCipher
    pub fn open(db_path: &Path, passphrase: &SecretString) -> AppResult<Self> {
        let manager = SqliteConnectionManager::file(db_path)
            .with_init(move |conn| {
                // Set SQLCipher key
                conn.pragma_update(None, "key", passphrase.expose_secret())?;
                // Set cipher config (AES-256)
                conn.pragma_update(None, "cipher_page_size", &4096)?;
                conn.pragma_update(None, "kdf_iter", &256000)?;
                Ok(())
            });

        let pool = r2d2::Pool::builder()
            .max_size(5)
            .build(manager)?;

        Ok(Self { pool })
    }

    /// Get connection from pool
    pub fn get_conn(&self) -> AppResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().map_err(Into::into)
    }

    /// Initialize schema if needed
    pub fn initialize_schema(&self) -> AppResult<()> {
        let conn = self.get_conn()?;
        schema::create_tables(&conn)?;
        Ok(())
    }
}
```

#### `db/schema.rs` - Table Definitions

```rust
pub fn create_tables(conn: &Connection) -> AppResult<()> {
    // Enable FTS5 extension
    conn.execute_batch("
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            date TEXT NOT NULL UNIQUE,
            checksum TEXT NOT NULL,
            word_count INTEGER,
            updated_at INTEGER NOT NULL,
            embedded_at INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date);
        CREATE INDEX IF NOT EXISTS idx_entries_updated ON entries(updated_at);

        CREATE TABLE IF NOT EXISTS embeddings (
            id INTEGER PRIMARY KEY,
            entry_id INTEGER NOT NULL,
            chunk_idx INTEGER NOT NULL,
            embedding BLOB NOT NULL,
            checksum TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            UNIQUE(entry_id, chunk_idx)
        );

        CREATE INDEX IF NOT EXISTS idx_embeddings_entry ON embeddings(entry_id);

        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            entry_id UNINDEXED,
            date UNINDEXED,
            content,
            tokenize = 'porter ascii'
        );

        CREATE TABLE IF NOT EXISTS insights (
            id INTEGER PRIMARY KEY,
            entry_id INTEGER,
            type TEXT NOT NULL,
            encrypted_content BLOB NOT NULL,
            score REAL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(entry_id) REFERENCES entries(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_insights_entry ON insights(entry_id);
        CREATE INDEX IF NOT EXISTS idx_insights_type ON insights(type);

        CREATE TABLE IF NOT EXISTS reports (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            type TEXT NOT NULL,
            date_range TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
    ")?;
    Ok(())
}
```

#### `db/embeddings.rs` - Vector Operations

```rust
/// Store embedding for entry chunk
pub fn insert_embedding(
    conn: &Connection,
    entry_id: i64,
    chunk_idx: usize,
    embedding: &[f32],  // 768 dimensions for nomic-embed-text
    checksum: &str,
) -> AppResult<()> {
    let embedding_bytes = bytemuck::cast_slice::<f32, u8>(embedding);

    conn.execute(
        "INSERT OR REPLACE INTO embeddings (entry_id, chunk_idx, embedding, checksum, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![entry_id, chunk_idx as i64, embedding_bytes, checksum, chrono::Utc::now().timestamp()],
    )?;
    Ok(())
}

/// Get all embeddings for an entry
pub fn get_entry_embeddings(conn: &Connection, entry_id: i64) -> AppResult<Vec<(usize, Vec<f32>)>> {
    let mut stmt = conn.prepare(
        "SELECT chunk_idx, embedding FROM embeddings WHERE entry_id = ?1 ORDER BY chunk_idx"
    )?;

    let embeddings = stmt.query_map([entry_id], |row| {
        let chunk_idx: i64 = row.get(0)?;
        let embedding_bytes: Vec<u8> = row.get(1)?;
        let embedding: Vec<f32> = bytemuck::cast_slice(&embedding_bytes).to_vec();
        Ok((chunk_idx as usize, embedding))
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(embeddings)
}

/// Vector similarity search (cosine similarity)
pub fn search_similar_chunks(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> AppResult<Vec<(i64, usize, f32)>> {  // (entry_id, chunk_idx, similarity)
    // This requires custom SQLite function for cosine similarity
    // Implementation uses rusqlite::functions::create_scalar_function

    let mut results = Vec::new();

    // Naive approach: Get all embeddings, compute similarity in Rust
    // TODO: Optimize with SQLite-VSS extension or custom C function
    let mut stmt = conn.prepare(
        "SELECT entry_id, chunk_idx, embedding FROM embeddings"
    )?;

    let rows = stmt.query_map([], |row| {
        let entry_id: i64 = row.get(0)?;
        let chunk_idx: i64 = row.get(1)?;
        let embedding_bytes: Vec<u8> = row.get(2)?;
        let embedding: Vec<f32> = bytemuck::cast_slice(&embedding_bytes).to_vec();
        Ok((entry_id, chunk_idx as usize, embedding))
    })?;

    for row_result in rows {
        let (entry_id, chunk_idx, embedding) = row_result?;
        let similarity = cosine_similarity(query_embedding, &embedding);
        results.push((entry_id, chunk_idx, similarity));
    }

    // Sort by similarity descending
    results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    results.truncate(limit);

    Ok(results)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}
```

**Testing**:
- SQLCipher encryption verification
- Schema migration
- Embedding storage/retrieval
- Vector search accuracy
- FTS integration

---

### Phase 3: AI Integration (Week 6-7)

**Goal**: Ollama client, embedding generation, chunking, RAG

**Module: `ai/`**

#### `ai/ollama.rs` - HTTP Client

```rust
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string()),
        }
    }

    /// Generate embedding for text
    pub fn embed(&self, model: &str, text: &str) -> AppResult<Vec<f32>> {
        #[derive(Serialize)]
        struct EmbedRequest<'a> {
            model: &'a str,
            prompt: &'a str,
        }

        #[derive(Deserialize)]
        struct EmbedResponse {
            embedding: Vec<f32>,
        }

        let response = self.client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&EmbedRequest { model, prompt: text })
            .send()?
            .error_for_status()?
            .json::<EmbedResponse>()?;

        Ok(response.embedding)
    }

    /// Chat completion
    pub fn chat(&self, model: &str, messages: &[Message]) -> AppResult<String> {
        #[derive(Serialize)]
        struct ChatRequest<'a> {
            model: &'a str,
            messages: &'a [Message],
            stream: bool,
        }

        #[derive(Deserialize)]
        struct ChatResponse {
            message: Message,
        }

        let response = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&ChatRequest {
                model,
                messages,
                stream: false,
            })
            .send()?
            .error_for_status()?
            .json::<ChatResponse>()?;

        Ok(response.message.content)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: String,  // "system", "user", "assistant"
    pub content: String,
}
```

#### `ai/chunking.rs` - Text Chunking

```rust
/// Chunk text for embedding (700 tokens, 100 overlap)
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    // Simple word-based chunking (TODO: Use tokenizers crate for accurate token count)
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();

    let mut i = 0;
    while i < words.len() {
        let end = (i + chunk_size).min(words.len());
        let chunk = words[i..end].join(" ");
        chunks.push(chunk);

        if end >= words.len() {
            break;
        }
        i += chunk_size - overlap;
    }

    chunks
}
```

#### `ai/prompts.rs` - System Prompts

```rust
pub const SYSTEM_PROMPT: &str =
    "You are a private journaling copilot. Be concise, concrete, and gentle. \
     Use only provided context. If missing info, say you don't know. \
     Prefer bullet points. Avoid clinical or moralizing language. \
     Suggest tiny next steps.";

pub fn reflect_prompt(entry_content: &str) -> Vec<Message> {
    vec![
        Message {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!(
                "Analyze this journal entry and provide:\n\
                 1. 5 bullets: events, emotions, energy, wins, blockers\n\
                 2. 2 prompts: specific, compassionate, forward-looking\n\n\
                 Entry:\n{}", entry_content
            ),
        },
    ]
}

pub fn ask_prompt(question: &str, context_chunks: &[String]) -> Vec<Message> {
    let context = context_chunks.join("\n\n---\n\n");

    vec![
        Message {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!(
                "Context from past journal entries:\n\n{}\n\n\
                 Question: {}\n\n\
                 Restate question → key observations → answer with evidence refs → 1 prompt",
                context, question
            ),
        },
    ]
}
```

---

### Phase 4: High-Level Operations (Week 8-9)

**Goal**: Implement user-facing commands: edit, ask, reflect, search

**Module: `ops/`**

#### `ops/edit.rs` - Edit with Encryption + Embedding

```rust
/// Edit journal entry with encryption and auto-embedding
pub fn edit_entry(
    config: &Config,
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    date: NaiveDate,
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()> {
    // 1. Get encrypted entry path
    let encrypted_path = get_encrypted_entry_path(&config.journal_dir, date);

    // 2. Unlock vault
    let passphrase = session.unlock()?;

    // 3. Decrypt to temp (or create new)
    let temp_path = if encrypted_path.exists() {
        decrypt_to_temp(&encrypted_path, passphrase)?
    } else {
        create_temp_entry(date, reference_datetime)?
    };

    // 4. Launch editor
    launch_editor(&config.editor, &[temp_path.clone()])?;

    // 5. Re-encrypt
    encrypt_from_temp(&temp_path, &encrypted_path, passphrase)?;

    // 6. Update database + generate embeddings
    update_entry_embeddings(db, ai_client, &encrypted_path, date, passphrase)?;

    Ok(())
}

fn update_entry_embeddings(
    db: &Database,
    ai_client: &OllamaClient,
    encrypted_path: &Path,
    date: NaiveDate,
    passphrase: &SecretString,
) -> AppResult<()> {
    // 1. Decrypt to memory (streaming)
    let content = decrypt_to_memory(encrypted_path, passphrase)?;
    let content_str = String::from_utf8_lossy(&content);

    // 2. Compute checksum
    let checksum = blake3::hash(&content).to_hex();

    // 3. Update entry metadata
    let conn = db.get_conn()?;
    let entry_id = db::entries::upsert_entry(
        &conn,
        encrypted_path,
        date,
        &checksum.to_string(),
        count_words(&content_str),
    )?;

    // 4. Check if embeddings need update (checksum changed)
    let needs_update = db::entries::needs_embedding_update(&conn, entry_id, &checksum.to_string())?;

    if needs_update {
        // 5. Generate chunks
        let chunks = chunk_text(&content_str, 700, 100);

        // 6. Generate embeddings
        tracing::info!("Generating embeddings for {} chunks", chunks.len());
        for (idx, chunk) in chunks.iter().enumerate() {
            let embedding = ai_client.embed("nomic-embed-text", chunk)?;
            let chunk_checksum = blake3::hash(chunk.as_bytes()).to_hex();
            db::embeddings::insert_embedding(
                &conn,
                entry_id,
                idx,
                &embedding,
                &chunk_checksum.to_string(),
            )?;
        }

        // 7. Update embedded_at timestamp
        db::entries::mark_embedded(&conn, entry_id)?;
    }

    // 8. Zeroize decrypted content
    drop(content);

    Ok(())
}
```

#### `ops/ask.rs` - RAG Query

```rust
/// Answer question using RAG
pub fn ask_question(
    config: &Config,
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    question: &str,
    window_days: Option<i64>,
) -> AppResult<String> {
    // 1. Generate query embedding
    let query_embedding = ai_client.embed("nomic-embed-text", question)?;

    // 2. Search for similar chunks
    let conn = db.get_conn()?;
    let similar_chunks = db::embeddings::search_similar_chunks(&conn, &query_embedding, 12)?;

    // 3. Filter by time window if specified
    let filtered_chunks = if let Some(days) = window_days {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days);
        db::entries::filter_chunks_by_date(&conn, similar_chunks, cutoff_date)?
    } else {
        similar_chunks
    };

    // 4. Decrypt relevant entry chunks
    let passphrase = session.unlock()?;
    let mut context_chunks = Vec::new();

    for (entry_id, chunk_idx, similarity) in filtered_chunks {
        let entry_path = db::entries::get_entry_path(&conn, entry_id)?;
        let full_content = decrypt_to_memory(&entry_path, passphrase)?;
        let content_str = String::from_utf8_lossy(&full_content);
        let chunks = chunk_text(&content_str, 700, 100);

        if let Some(chunk) = chunks.get(chunk_idx) {
            context_chunks.push(format!(
                "[{} §{}] {}",
                entry_path.file_stem().unwrap().to_string_lossy(),
                chunk_idx,
                chunk
            ));
        }
    }

    // 5. Generate prompt
    let messages = ask_prompt(question, &context_chunks);

    // 6. Call LLM
    let answer = ai_client.chat("llama3.2:3b", &messages)?;

    Ok(answer)
}
```

#### `ops/reflect.rs` - Entry Reflection

```rust
/// Generate reflection for today's entry
pub fn reflect_on_entry(
    config: &Config,
    db: &Database,
    session: &mut SessionManager,
    ai_client: &OllamaClient,
    date: Option<NaiveDate>,
) -> AppResult<String> {
    let target_date = date.unwrap_or_else(|| chrono::Local::now().naive_local().date());

    // 1. Get entry path
    let entry_path = get_encrypted_entry_path(&config.journal_dir, target_date);

    if !entry_path.exists() {
        return Err(AppError::Journal(format!(
            "No entry found for {}",
            target_date
        )));
    }

    // 2. Decrypt entry
    let passphrase = session.unlock()?;
    let content = decrypt_to_memory(&entry_path, passphrase)?;
    let content_str = String::from_utf8_lossy(&content);

    // 3. Generate reflection prompt
    let messages = reflect_prompt(&content_str);

    // 4. Call LLM
    let reflection = ai_client.chat("llama3.2:3b", &messages)?;

    // 5. Optionally save to encrypted report
    let report_path = config.journal_dir.join(format!("reports/{}.md.age", target_date));
    let report_content = format!(
        "# Daily Review — {}\n\n{}\n\n---\n(model=llama3.2:3b, generated={})",
        target_date,
        reflection,
        chrono::Utc::now().to_rfc3339()
    );

    encrypt_to_file(report_content.as_bytes(), &report_path, passphrase)?;

    Ok(reflection)
}
```

---

### Phase 5: CLI Integration & Polish (Week 10-11)

**Goal**: Wire up all commands, setup wizard, error handling

**Modify `cli/mod.rs`**:

```rust
#[derive(Parser)]
#[clap(name = "ponder", version, about = "AI-powered encrypted journaling")]
pub enum PonderCli {
    /// Open journal entry in editor (default: today)
    #[clap(visible_alias = "e")]
    Edit {
        /// Date to edit (YYYY-MM-DD, or leave empty for today)
        #[clap(short, long)]
        date: Option<String>,

        /// Open past 7 days (retro mode)
        #[clap(short, long)]
        retro: bool,

        /// Open entries from past intervals (reminisce mode)
        #[clap(short = 'm', long)]
        reminisce: bool,
    },

    /// Ask a question about your journal entries
    #[clap(visible_alias = "a")]
    Ask {
        /// Question to ask
        question: String,

        /// Only search entries from last N days
        #[clap(short, long, default_value = "30")]
        window: i64,
    },

    /// Generate reflection on an entry
    #[clap(visible_alias = "r")]
    Reflect {
        /// Date to reflect on (default: today)
        #[clap(short, long)]
        date: Option<String>,
    },

    /// Search journal entries semantically
    #[clap(visible_alias = "s")]
    Search {
        /// Search query
        query: String,

        /// Only search entries from last N days
        #[clap(short, long, default_value = "90")]
        window: i64,
    },

    /// Re-index changed entries and generate daily report
    Nightly,

    /// Generate weekly review
    Weekly,

    /// Lock vault and clear session keys
    Lock,

    /// Setup wizard for first-time configuration
    Setup,
}
```

**Modify `main.rs`**:

```rust
fn main() {
    // Parse CLI
    let cli = PonderCli::parse();

    // Initialize logging (existing code)
    // ...

    // Load config
    let config = Config::load()?;

    // Initialize session manager
    let mut session = SessionManager::new(config.session_timeout_minutes);

    // Initialize database
    let passphrase = session.unlock()?;  // Prompt on first command
    let db = Database::open(&config.db_path, passphrase)?;
    db.initialize_schema()?;

    // Initialize AI client
    let ai_client = OllamaClient::new();

    // Dispatch command
    match cli {
        PonderCli::Edit { date, retro, reminisce } => {
            let date_spec = DateSpecifier::from_cli_args(retro, reminisce, date.as_deref())?;
            let dates = date_spec.resolve_dates(chrono::Local::now().naive_local().date());

            if dates.len() == 1 {
                // Single entry: full encryption + embedding flow
                ops::edit::edit_entry(&config, &db, &mut session, &ai_client, dates[0], &chrono::Local::now())?;
            } else {
                // Multiple entries: simpler read-only flow (Phase 2 implementation)
                ops::edit::edit_multiple_entries(&config, &mut session, &dates)?;
            }
        }
        PonderCli::Ask { question, window } => {
            let answer = ops::ask::ask_question(&config, &db, &mut session, &ai_client, &question, Some(window))?;
            println!("{}", answer);
        }
        PonderCli::Reflect { date } => {
            let target_date = date.map(|d| parse_date(&d)).transpose()?.unwrap_or_else(|| chrono::Local::now().naive_local().date());
            let reflection = ops::reflect::reflect_on_entry(&config, &db, &mut session, &ai_client, Some(target_date))?;
            println!("{}", reflection);
        }
        PonderCli::Search { query, window } => {
            let results = ops::search::search_entries(&config, &db, &mut session, &ai_client, &query, Some(window))?;
            for result in results {
                println!("{}", result);
            }
        }
        PonderCli::Nightly => {
            ops::nightly::run_nightly(&config, &db, &mut session, &ai_client)?;
            println!("Nightly indexing complete");
        }
        PonderCli::Weekly => {
            ops::weekly::generate_weekly_review(&config, &db, &mut session, &ai_client)?;
            println!("Weekly review generated");
        }
        PonderCli::Lock => {
            session.lock();
            println!("Vault locked");
        }
        PonderCli::Setup => {
            ops::setup::run_setup_wizard(&config)?;
        }
    }
}
```

---

### Phase 6: Testing & Documentation (Week 12)

**Goal**: Comprehensive tests, migration guide, README updates

**Testing Strategy**:

1. **Unit Tests** (within each module):
   - Encryption round-trip
   - Database operations
   - Vector search accuracy
   - Chunking correctness
   - Session management

2. **Integration Tests** (`tests/` directory):
   - End-to-end edit → encrypt → embed flow
   - RAG query with mock Ollama responses
   - Search accuracy with sample corpus
   - Migration from plaintext to encrypted

3. **Security Tests**:
   - Temp file cleanup verification
   - Passphrase zeroization
   - No plaintext in database
   - Encrypted file unreadable without key

**Migration Guide** (`MIGRATION.md`):

```markdown
# Migrating from Ponder v1.0 to v2.0

## Overview
v2.0 adds encryption and AI features. Your existing plaintext journals will not be automatically migrated—you must explicitly convert them.

## Step-by-Step Migration

### 1. Backup Your Journals
```bash
cp -r ~/Documents/rubberducks ~/Documents/rubberducks.backup
```

### 2. Install Ollama (for AI features)
```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama3.2:3b
ollama pull nomic-embed-text
```

### 3. Run Setup Wizard
```bash
ponder setup
# Follow prompts to:
# - Set new journal directory (default: ~/Journal)
# - Choose encryption passphrase
# - Configure AI models
```

### 4. Migrate Existing Entries
```bash
ponder migrate --from ~/Documents/rubberducks --to ~/Journal
# This will:
# - Encrypt all .md files to .md.age
# - Generate embeddings for search
# - Create ponder.db database
```

### 5. Verify Migration
```bash
ponder search "recent thoughts"  # Test search
ponder reflect                    # Test AI on today's entry
```

## Backward Compatibility

You can continue using plaintext mode by setting:
```bash
export PONDER_ENCRYPTION=false
```

This disables encryption but maintains v1.0 behavior.
```

---

## Part IV: Technical Decisions & Rationales

### Why Age Encryption?

**Alternatives Considered**:
1. **GPG**: Too complex, requires key management infrastructure
2. **Native AES**: Requires custom KDF, MAC, mode selection
3. **Age**: Modern, simple, secure by default, passphrase support

**Decision**: Age
- Single-file format (`.age` extension)
- Built-in Argon2id KDF (secure passphrase derivation)
- Stream-friendly for large files
- Well-audited, actively maintained
- Git-friendly (small format overhead)

### Why SQLCipher Instead of Plaintext DB?

**Attack Scenario**: Encrypted journal files + plaintext database = metadata leakage (entry dates, word counts, embeddings reveal content structure)

**Decision**: SQLCipher
- Transparent encryption (same SQLite API)
- AES-256-CBC encryption
- Separate key from age passphrase (derived from same root)
- Protects: entry metadata, embeddings, FTS index

### Why Ollama?

**Alternatives Considered**:
1. **OpenAI API**: Cloud-based (violates local-first principle)
2. **llama.cpp**: Lower-level, requires manual model management
3. **Candle**: Rust-native but less mature ecosystem
4. **Ollama**: Easy setup, good model selection, HTTP API

**Decision**: Ollama
- Local-only (no cloud leakage)
- Pre-configured models (llama3.2, nomic-embed-text)
- Simple HTTP API
- Cross-platform (macOS, Linux, Windows)
- Active community

### Why Not Vector Database (Qdrant/Chroma)?

**Alternatives Considered**:
1. **Qdrant**: Requires separate process, complex deployment
2. **Chroma**: Python-focused, awkward Rust integration
3. **SQLite + manual vector search**: Simple, embedded, good enough

**Decision**: SQLite with manual cosine similarity
- No extra processes
- Good enough for personal journals (<10k entries)
- Can optimize later with SQLite-VSS extension
- Keeps architecture simple

### Security Trade-offs

**What We Protect**:
- ✅ Encrypted files at rest (age)
- ✅ Encrypted database (SQLCipher)
- ✅ Session timeout (auto-lock after 30 min)
- ✅ Temp files on tmpfs (RAM-only when available)
- ✅ Zeroize passphrases in memory
- ✅ No plaintext content in database
- ✅ Secure file permissions (0o600)

**What We Don't Protect** (Documented Limitations):
- ❌ Active malware/keyloggers (can capture passphrase input)
- ❌ RAM scraping during live session (decrypted content in memory)
- ❌ Forensic recovery from non-tmpfs systems
- ❌ Timing attacks on vector search (not constant-time)
- ❌ Side-channel attacks via Ollama API

**Mitigation Guidance** (in docs):
- Use full-disk encryption (FileVault/LUKS/BitLocker)
- Don't run ponder on compromised systems
- Short session timeout reduces exposure window
- Prefer tmpfs systems for journaling

---

## Part V: Success Metrics & Acceptance Criteria

### MVP Acceptance Criteria

**Must Have (Blocking Release)**:
1. ✅ `ponder` opens today's entry, encrypts on close, no plaintext residue
2. ✅ `ponder reflect` produces encrypted report with useful summary
3. ✅ `ponder ask "..."` answers coherently, cites dates, no plaintext files
4. ✅ `ponder search "..."` returns relevant results (>70% accuracy)
5. ✅ `ponder setup` completes successfully for non-technical users
6. ✅ `ponder nightly` indexes 1k notes in <2 minutes
7. ✅ All operations work offline (except AI calls fail gracefully)
8. ✅ Session auto-locks after timeout
9. ✅ Existing tests pass + new features have >80% coverage

**Performance Targets**:
- Open encrypted entry: <500ms overhead vs v1.0
- Generate embeddings (single entry): <5s on M2 Pro
- Vector search (1k entries): <1s
- Nightly batch (1k entries): <2 minutes

**User Experience**:
- Setup wizard: <3 minutes for first-time setup
- Clear error messages (no "failed to decrypt: InvalidData")
- Graceful Ollama offline handling (suggest `ollama serve`)

---

## Part VI: Migration Path & Rollout

### Alpha (Internal, Week 13)

**Goal**: Validate with developer use
- Build on macOS ARM64
- Test with personal journal (~100 entries)
- Fix critical bugs, UX friction
- Benchmark performance targets

### Beta (Trusted Users, Week 14-15)

**Goal**: Real-world validation with 5-10 users
- Package for macOS (Homebrew tap)
- Package for Linux (x86_64 binary)
- Collect feedback on:
  - Setup wizard clarity
  - Search result relevance
  - AI prompt quality
  - Performance on older hardware

### v2.0 Release (Week 16)

**Deliverables**:
- GitHub release with binaries (macOS ARM64/x86_64, Linux x86_64)
- Homebrew formula
- Migration guide from v1.0
- Updated README with encryption/AI features
- Architecture documentation

**Post-Launch Roadmap** (v2.1+):
- [ ] X25519 identity support (for SSH-key-based encryption)
- [ ] Stealth mode (filename obfuscation)
- [ ] Nvim plugin (`:PonderAsk`, `:PonderReflect`)
- [ ] Export to PDF/HTML with decryption
- [ ] Weekly/monthly automated reports
- [ ] Theme extraction ("What were my main concerns in Q1?")
- [ ] Streak tracking & statistics

---

## Part VII: Dependencies & Environment

### Required Software

**Runtime**:
- Rust 1.79+ (for building)
- Ollama 0.1.20+ (for AI features)
- macOS 13+ / Linux kernel 5.x+ (for tmpfs)

**Development**:
- pre-commit (Python tool for git hooks)
- cargo-audit (security vulnerability scanning)
- cargo-deny (license/dependency checking)

### Ollama Models

**Default Models** (auto-pulled on first use):
- `llama3.2:3b` (~2GB) - Chat/reasoning
- `nomic-embed-text` (~270MB) - Embeddings (768 dims)

**Optional Upgrades**:
- `llama3.2:7b` - Better reasoning, slower
- `mistral` - Alternative LLM
- `nomic-embed-text:latest` - Latest embedding model

### Disk Space Requirements

**Per User**:
- Ponder binary: ~15MB
- SQLite database: ~10MB per 1000 entries
- Encrypted entries: ~1.1x plaintext size
- Ollama models: ~2.3GB (llama3.2:3b + nomic-embed-text)
- Temp space: ~5MB per active session

**Example** (5 years of daily journaling):
- 1825 entries × 500 words avg × 5 bytes/word = ~4.5MB plaintext
- Encrypted: ~5MB
- Database + embeddings: ~20MB
- Total: ~25MB (excluding Ollama models)

---

## Part VIII: Open Questions & Risks

### Open Questions

1. **Embedding Model Lock-in**: What happens when nomic-embed-text v2 is released? Can we re-embed all entries gracefully?
   - **Mitigation**: Version embeddings in DB, support multiple embedding models

2. **SQLCipher Performance**: Does encryption overhead hurt vector search at scale (10k+ entries)?
   - **Mitigation**: Benchmark early, consider caching hot embeddings in RAM

3. **Ollama Stability**: What if Ollama crashes mid-operation?
   - **Mitigation**: Retry logic, clear error messages, graceful degradation

4. **Migration Complexity**: Will users successfully migrate from v1.0 without data loss?
   - **Mitigation**: Dry-run mode, verification step, clear rollback instructions

### Risks & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Ollama not installed | High | Medium | Clear error, link to install script |
| Slow embedding on old hardware | Medium | Medium | Progress bars, batch mode, skip AI features flag |
| Encryption key lost | Medium | Critical | Recovery mode with identity files, backup reminder in setup |
| Database corruption | Low | High | Atomic writes, checksums, backup before migrations |
| Vector search irrelevant results | Medium | Medium | Hybrid search (vector + FTS), user feedback loop |
| tmpfs unavailable on Windows | High | Low | Fallback to system temp + security warning |

---

## Appendix A: Codebase Change Summary

### Files to Modify

**Existing** (Preserve behavior, extend):
- `Cargo.toml` - Add dependencies
- `src/main.rs` - Add subcommand dispatch
- `src/cli/mod.rs` - Add new commands
- `src/config/mod.rs` - Add vault/AI config
- `src/errors/mod.rs` - Add crypto/DB/AI errors
- `src/constants.rs` - Add crypto/AI constants
- `src/journal_io/mod.rs` - Extract editor/locking to submodules

**New Files** (~5,500 LOC production code):
- `src/crypto/` - ~800 LOC (age, session, temp, kdf)
- `src/db/` - ~900 LOC (schema, entries, embeddings, fts, reports)
- `src/ai/` - ~600 LOC (ollama, embeddings, chat, chunking, prompts)
- `src/search/` - ~400 LOC (vector, keyword, hybrid)
- `src/ops/` - ~2,800 LOC (edit, ask, reflect, search, nightly, weekly, setup)

**New Tests** (~1,300 LOC):
- `tests/crypto_tests.rs` - Encryption round-trip, zeroization
- `tests/db_tests.rs` - SQLCipher, vector search, FTS
- `tests/ai_tests.rs` - Ollama client (mocked), chunking
- `tests/integration_tests.rs` - End-to-end flows
- `tests/migration_tests.rs` - v1.0 → v2.0 migration

### LOC Growth Breakdown

| Component | Current | Added | Total | Notes |
|-----------|---------|-------|-------|-------|
| Core logic | 1,900 | 0 | 1,900 | Preserved |
| Crypto | 0 | 800 | 800 | New module |
| Database | 0 | 900 | 900 | New module |
| AI | 0 | 600 | 600 | New module |
| Search | 0 | 400 | 400 | New module |
| Operations | 0 | 2,800 | 2,800 | New module |
| Tests | 2,200 | 1,300 | 3,500 | +60% coverage |
| **TOTAL** | **4,100** | **6,800** | **10,900** | **2.7x growth** |

---

## Appendix B: Quick Start (Post-MVP)

```bash
# Install Ponder v2.0
brew tap phrazzld/ponder
brew install ponder

# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama3.2:3b
ollama pull nomic-embed-text

# First-time setup
ponder setup
# Follow wizard: set passphrase, choose journal directory

# Open today's journal
ponder

# Ask about your week
ponder ask "What energized me this week?"

# Search past entries
ponder search "moments of clarity"

# Daily reflection
ponder reflect

# Weekly review
ponder weekly

# Run nightly indexing (cron this)
ponder nightly
```

---

**This specification is ready for implementation. Next step: `/plan` to break down into tasks.**
