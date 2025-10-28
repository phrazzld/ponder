# Ponder 🤔

Ponder v2.0 is an AI-powered encrypted journaling tool for daily reflections. It combines strong encryption (age + SQLCipher) with local AI capabilities (Ollama) to provide semantic search, RAG queries, and AI-generated reflections—all while keeping your data private and secure.

## Features 🌟

### Core Features (v1.0)
- **Today's Entry**: Quickly create or edit today's journal entry
- **Retro Mode**: Review entries from the past week (excluding today)
- **Reminisce Mode**: Review entries from significant past intervals (1 month ago, 3 months ago, 6 months ago, etc.)
- **Specific Date Access**: Open an entry for any specific date
- **Markdown Support**: Journal entries stored as encrypted markdown files

### Security Features (v2.0)
- **End-to-End Encryption**: All journal entries encrypted with [age](https://github.com/FiloSottile/age) passphrase-based encryption
- **Encrypted Database**: Metadata and embeddings stored in SQLCipher (256-bit AES)
- **Secure Session Management**: Auto-lock timeout with passphrase zeroization
- **Secure Temp Files**: 0o600 permissions with automatic cleanup
- **No Plaintext Leakage**: All sensitive data encrypted at rest

### AI Features (v2.0)
- **Semantic Search**: Find journal entries by meaning, not just keywords
- **RAG Queries**: Ask questions about your journal using AI (Retrieval-Augmented Generation)
- **AI Reflections**: Get thoughtful AI-generated insights on your entries
- **Automatic Embeddings**: Content automatically vectorized for semantic search
- **Local-First AI**: Uses local Ollama instance (your data stays on your machine)

## ⚠️ Security Notice

Ponder uses **zero-knowledge encryption** - your passphrase encrypts all journal data.

**CRITICAL**: If you forget your passphrase, **your data is permanently lost**. There is no recovery mechanism.

**Best practices**:
- Choose a passphrase you can remember (e.g., 4-5 random words)
- Write it down and store in a secure physical location
- Consider using a password manager
- Test backup/restore before relying on Ponder for important data

## Installation 🔧

### Prerequisites

**Required:**
1. Rust and Cargo ([rustup.rs](https://rustup.rs))
2. SQLCipher support (for encrypted database)
   ```bash
   # macOS
   brew install sqlcipher

   # Ubuntu/Debian
   sudo apt-get install libsqlcipher-dev

   # Fedora
   sudo dnf install sqlcipher-devel
   ```

**Optional (for AI features):**
3. [Ollama](https://ollama.ai) (for semantic search and AI features)
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh

   # Pull required models
   ollama pull nomic-embed-text    # For embeddings
   ollama pull gemma3:4b           # For chat/reflections
   ```

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/phrazzld/ponder.git
   cd ponder
   ```

2. Build and install:
   ```bash
   cargo build --release
   cargo install --path .
   ```

3. (Optional) Start Ollama for AI features:
   ```bash
   ollama serve
   ```

## Usage 📝

### Basic Usage

Ponder v2.0 uses a subcommand architecture:

```bash
ponder <COMMAND> [OPTIONS]
```

### Commands

| Command | Description |
|---------|-------------|
| `edit` | Edit journal entries (with encryption) |
| `ask` | Query your journal using AI (RAG) |
| `reflect` | Generate AI reflection on an entry |
| `search` | Semantic search over journal entries |
| `lock` | Lock the encrypted session |
| `backup` | Create encrypted backup archive |
| `restore` | Restore from encrypted backup |

### Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enables verbose output for debugging |
| `--log-format FORMAT` | Sets the log output format ("text" or "json") |
| `-h, --help` | Displays help information |
| `-V, --version` | Displays version information |

### Edit Command

Edit encrypted journal entries (v1.0 compatibility mode):

```bash
# Edit today's entry (default)
ponder edit

# Or just:
ponder

# Edit past week entries (retro mode)
ponder edit --retro

# Edit significant past intervals (reminisce mode)
ponder edit --reminisce

# Edit specific date
ponder edit --date 2024-01-15
```

**Edit Options:**
- `-r, --retro`: Opens entries from the past week (excluding today)
- `-m, --reminisce`: Opens entries from significant past intervals
- `-d, --date DATE`: Opens entry for specific date (YYYY-MM-DD or YYYYMMDD)

### Ask Command (v2.0)

Query your journal using AI with RAG (Retrieval-Augmented Generation):

```bash
# Ask a question
ponder ask "What were my main goals last month?"

# Ask with date range filter
ponder ask "What did I learn about Rust?" --from 2024-01-01 --to 2024-06-30
```

**Ask Options:**
- `--from DATE`: Filter results from this date
- `--to DATE`: Filter results until this date

### Reflect Command (v2.0)

Generate AI reflection on a journal entry:

```bash
# Reflect on today's entry
ponder reflect

# Reflect on specific date
ponder reflect --date 2024-01-15
```

**Reflect Options:**
- `-d, --date DATE`: Date of entry to reflect on (defaults to today)

### Search Command (v2.0)

Semantic search over journal entries:

```bash
# Search for entries about a topic
ponder search "anxiety and coping strategies"

# Search with custom result limit
ponder search "project ideas" --limit 10

# Search within date range
ponder search "productivity" --from 2024-01-01 --to 2024-06-30 --limit 5
```

**Search Options:**
- `-l, --limit N`: Maximum number of results (default: 5)
- `--from DATE`: Filter results from this date
- `--to DATE`: Filter results until this date

### Lock Command (v2.0)

Lock the encrypted session (clear passphrase from memory):

```bash
ponder lock
```

### Backup Command (v2.0)

Create encrypted backup archive of your entire journal:

```bash
# Create backup (prompts for confirmation)
ponder backup

# Verify backup integrity (optional)
ponder backup --verify
```

**Backup Details:**
- Creates encrypted `.tar.age` archive containing:
  - All encrypted journal entries (`*.md.age`)
  - Encrypted database (`ponder.db`)
  - Backup manifest with checksums
- Default location: `$PONDER_DIR/backups/ponder-backup-YYYYMMDD-HHMMSS.tar.age`
- Encrypted with your journal passphrase
- Includes metadata: timestamp, file count, total size

**Backup Options:**
- `--verify`: Verify backup integrity after creation (recommended)

### Restore Command (v2.0)

Restore journal from encrypted backup archive:

```bash
# Restore from backup (prompts for confirmation)
ponder restore /path/to/backup.tar.age

# Force restore (skip confirmation)
ponder restore /path/to/backup.tar.age --force
```

**Restore Details:**
- Extracts and verifies all files from backup archive
- Validates checksums against backup manifest
- Restores to current `$PONDER_DIR` (overwrites existing files)
- Reports: files restored, total size, checksum verification status

**Restore Options:**
- `-f, --force`: Skip confirmation prompt and overwrite existing files

**⚠️ Backup Security Warning:**

Backup archives are encrypted with your journal passphrase and provide strong security **at rest**. However:

1. **Storage Security**: Store backups in a secure location (encrypted external drive, secure cloud storage with encryption at rest)
2. **Passphrase Security**: Your backup is only as secure as your passphrase. Use a strong, unique passphrase.
3. **Transport Security**: When transferring backups, use encrypted channels (HTTPS, SFTP, encrypted email)
4. **Access Control**: Limit who can access your backup files (file permissions, access controls)
5. **Retention**: Securely delete old backups you no longer need

**Best Practices:**
- Test restores periodically to verify backup integrity
- Store backups in multiple secure locations (3-2-1 backup strategy)
- Use `--verify` flag to check backup integrity immediately after creation
- Keep backups separate from your primary journal location
- Consider using encrypted cloud storage (e.g., encrypted S3 buckets, Tresorit, Sync.com)

## Configuration ⚙️

Ponder can be configured using environment variables:

### Core Configuration

| Environment Variable | Description | Default |
|----------------------|-------------|---------|
| `PONDER_DIR` | Directory where encrypted journal entries are stored | `~/Documents/rubberducks` |
| `PONDER_EDITOR` | Editor to use for journal entries | Uses `EDITOR` if set |
| `EDITOR` | Fallback editor if `PONDER_EDITOR` is not set | `vim` |

### v2.0 Configuration

| Environment Variable | Description | Default |
|----------------------|-------------|---------|
| `PONDER_DB` | Path to encrypted SQLite database | `$PONDER_DIR/ponder.db` |
| `PONDER_SESSION_TIMEOUT` | Session timeout in minutes | `30` |
| `OLLAMA_URL` | Ollama API URL | `http://127.0.0.1:11434` |

### Logging Configuration

| Environment Variable | Description | Default |
|----------------------|-------------|---------|
| `RUST_LOG` | Controls log filtering and verbosity | `info` |
| `CI` | When set to any value, forces JSON log format | Not set |

### Editor Configuration Security

⚠️ **Important Security Restriction**: For security reasons, `PONDER_EDITOR` and `EDITOR` must be set to a single command or an absolute/relative path to an executable, without any embedded spaces or arguments. This prevents command injection vulnerabilities.

**Valid examples:**
```bash
export PONDER_EDITOR="vim"
export PONDER_EDITOR="/usr/bin/nano"
export PONDER_EDITOR="./my-editor"
```

**Invalid examples (will be rejected):**
```bash
export PONDER_EDITOR="vim --noplugin"      # Contains spaces and arguments
export PONDER_EDITOR="code -w"             # Contains spaces and arguments
export PONDER_EDITOR="echo > /tmp/file"    # Contains shell metacharacters
```

### Workarounds for Editor Arguments

If you need to pass arguments to your editor, you can use one of these approaches:

#### Method 1: Shell Alias
Create an alias in your shell configuration file:

```bash
# In your .bashrc, .zshrc, etc.
alias ponder-editor='code -w'
export PONDER_EDITOR="ponder-editor"
```

#### Method 2: Wrapper Script
Create a wrapper script that calls your editor with the desired arguments:

```bash
#!/bin/bash
# Save as ~/bin/ponder-code (or any location in your PATH)
exec code -w "$@"
```

Make it executable:
```bash
chmod +x ~/bin/ponder-code
export PONDER_EDITOR="ponder-code"
```

### Example Configuration

```bash
# Add to your .bashrc, .zshrc, etc.
export PONDER_DIR="~/journals"
export PONDER_EDITOR="vim"  # Simple command without arguments
export RUST_LOG="debug"     # For more verbose logging
```

### Logging Configuration

Ponder uses structured logging with support for both human-readable and JSON output formats:

#### Log Levels

You can control log verbosity using the `RUST_LOG` environment variable:

```bash
# Show only info, warn, and error logs (default)
export RUST_LOG=info

# Show debug logs and above
export RUST_LOG=debug

# Show trace logs (most verbose)
export RUST_LOG=trace

# Filter logs from specific modules
export RUST_LOG=ponder::journal_io=debug,info
```

#### Log Formats

Ponder supports two output formats:

1. **Text** (default): Human-readable output for development
2. **JSON**: Structured output for parsing and analysis

You can select the format using the `--log-format` CLI option:

```bash
# Human-readable output
ponder --log-format text

# JSON output
ponder --log-format json
```

Setting the `CI` environment variable will also force JSON output:

```bash
CI=true ponder
```

#### Correlation IDs

All logs include a correlation ID that uniquely identifies each application invocation. This makes it easier to trace all operations from a single run of the application.

JSON log entries include:
- `timestamp`: ISO 8601 timestamp
- `level`: Log level (`INFO`, `DEBUG`, etc.)
- `target`: Module path
- `span`: Contains `correlation_id` and context information
- `fields`: Contains the log message and other fields

## File Structure 📁

### v2.0 Encrypted Structure

Journal entries are encrypted and organized in a hierarchical structure:

```
~/Documents/rubberducks/
├── ponder.db              # Encrypted SQLite database (SQLCipher)
├── 2024/
│   ├── 01/
│   │   ├── 01.md.age      # January 1st, 2024 (encrypted)
│   │   ├── 15.md.age      # January 15th, 2024 (encrypted)
│   │   └── ...
│   ├── 02/
│   │   └── ...
│   └── ...
└── ...
```

**File Format:**
- Entries: `YYYY/MM/DD.md.age` (age-encrypted markdown)
- Database: `ponder.db` (SQLCipher with 256-bit AES)

**Database Contents (all encrypted):**
- Entry metadata: paths, dates, checksums, word counts
- Vector embeddings: semantic representations for AI search
- No plaintext content stored in database

### Security Properties

1. **Encryption at Rest**: All journal content encrypted with age
2. **Encrypted Database**: Metadata and embeddings protected with SQLCipher
3. **Secure Temp Files**: 0o600 permissions, automatic cleanup
4. **Passphrase Protection**: Session-based with auto-lock timeout
5. **Change Detection**: BLAKE3 checksums prevent unnecessary re-embedding

## Architecture 🏗️

Ponder v2.0 follows a modular architecture with clear separation of concerns:

### Core Modules

- **`cli`**: Subcommand-based CLI (Edit|Ask|Reflect|Search|Lock)
- **`config`**: Configuration with v2.0 settings (db_path, session_timeout, ollama_url)
- **`errors`**: Comprehensive error handling with actionable messages
- **`crypto`**: Age encryption + session management + secure temp files
- **`db`**: SQLCipher database + entries + embeddings with vector search
- **`ai`**: Ollama client + text chunking + AI prompts
- **`ops`**: High-level operations combining crypto, DB, and AI
- **`journal_core`**: Pure date logic (v1.0 compatibility)
- **`journal_io`**: Legacy I/O operations (v1.0 compatibility)

### Design Principles

- **Deep Modules**: Simple interfaces hiding complex implementations
- **No Information Leakage**: Return domain objects, not raw DB rows
- **Explicitness**: Dependencies visible in function signatures
- **Security First**: Encryption, zeroization, secure temp files
- **Local-First AI**: Ollama runs locally, your data stays private

### Concurrent Edits

Ponder is designed as a single-user journaling tool with per-file encryption. The v2.0 architecture naturally minimizes conflicts:

- Each date has a separate encrypted file (`YYYY/MM/DD.md.age`)
- Only concurrent edits to the **same date** can conflict (rare scenario)
- If a conflict occurs, you'll receive a warning before saving
- **Last-write-wins**: Your save proceeds and overwrites any concurrent changes

**Conflict Detection Example**:
```
⚠️  Warning: This entry was modified while you were editing.
   Your changes will overwrite those modifications.
```

This design is intentionally simple:
- No complex file locking mechanisms
- No platform-specific behavior
- Clear user feedback for rare edge cases
- Reliability over paranoia for single-user workflows

For detailed architecture documentation, see [CLAUDE.md](./CLAUDE.md).

## Contributing 🤝

We welcome contributions to Ponder! If you'd like to contribute:

1. Fork the repository
2. Create a new branch for your feature (`git checkout -b feature/my-new-feature`)
3. Add tests for your changes
4. Make your changes and ensure all tests pass (`cargo test`)
5. Ensure your code is properly formatted and passes linting checks
6. Commit your changes following the Conventional Commits specification
7. Submit a pull request

For more details, please see [CONTRIBUTING.md](./CONTRIBUTING.md).

### Development Setup

To set up your development environment:

```bash
# Clone the repository
git clone https://github.com/phrazzld/ponder.git
cd ponder

# Install pre-commit hooks (recommended)
pip install pre-commit
pre-commit install
```

### Code Formatting and Linting

We use automated tools to ensure consistent code style:

- **rustfmt**: Automatically formats Rust code
  ```bash
  cargo fmt
  ```

- **clippy**: Provides linting and code improvement suggestions
  ```bash
  cargo clippy --all-targets -- -D warnings
  ```

- **pre-commit hooks**: Automatically run formatters and linters before each commit
  - The hooks will run automatically when you commit changes
  - To run them manually: `pre-commit run --all-files`

The repository includes:
- `.rustfmt.toml` - Configuration for rustfmt
- `.pre-commit-config.yaml` - Configuration for pre-commit hooks
- `.vscode/settings.json` - Settings for VS Code users
- `.editorconfig` - Settings for various editors

Please check out the open issues for tasks that need help, or feel free to propose new features or improvements.

## License 📜

Ponder is released under the [MIT License](./LICENSE).