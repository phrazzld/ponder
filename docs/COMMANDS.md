# Ponder v2.0 Command Reference

Complete reference for all Ponder commands, options, and usage patterns.

## Table of Contents

- [Global Options](#global-options)
- [edit](#edit) - Edit journal entries
- [ask](#ask) - Query journal with AI (RAG)
- [reflect](#reflect) - Generate AI reflection
- [search](#search) - Semantic search
- [lock](#lock) - Lock encrypted session
- [backup](#backup) - Create encrypted backup
- [restore](#restore) - Restore from backup
- [cleanup-v1](#cleanup-v1) - Remove migrated v1.0 files
- [Environment Variables](#environment-variables)
- [Exit Codes](#exit-codes)

---

## Global Options

Options available for all commands:

```bash
ponder [OPTIONS] <COMMAND>
```

| Option | Short | Description |
|--------|-------|-------------|
| `--verbose` | `-v` | Enable verbose logging output |
| `--log-format FORMAT` | | Set log format: `text` (default) or `json` |
| `--help` | `-h` | Show help information |
| `--version` | `-V` | Show version information |

**Examples:**
```bash
# Verbose output
ponder -v edit

# JSON logs for parsing
ponder --log-format json edit

# Show version
ponder --version
```

---

## edit

Edit encrypted journal entries with automatic encryption and embedding generation.

### Usage

```bash
ponder edit [OPTIONS]
ponder [OPTIONS]  # edit is the default command
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--retro` | `-r` | Edit past week entries (excluding today) |
| `--reminisce` | `-m` | Edit significant past intervals |
| `--date DATE` | `-d DATE` | Edit entry for specific date |
| `--migrate` | | Trigger v1.0 → v2.0 migration |

### Date Format

The `--date` option accepts:
- `YYYY-MM-DD` format: `2024-01-15`
- `YYYYMMDD` format: `20240115`

### Modes

**Default Mode (Today)**
```bash
ponder edit
ponder  # Same as above
```
Opens today's journal entry for editing.

**Retro Mode**
```bash
ponder edit --retro
ponder edit -r
```
Opens entries from the past 7 days (excluding today) in chronological order. Useful for weekly review.

**Reminisce Mode**
```bash
ponder edit --reminisce
ponder edit -m
```
Opens entries from significant past intervals:
- 1 month ago
- 3 months ago
- 6 months ago
- 1 year ago
- 2 years ago
- 3 years ago

**Specific Date**
```bash
ponder edit --date 2024-01-15
ponder edit -d 20240115
```
Opens the entry for the specified date.

**Migration Mode**
```bash
ponder edit --migrate
```
Manually triggers v1.0 → v2.0 migration (bypasses auto-detection prompt).

### Behavior

1. **Session Management**: Prompts for passphrase if session is locked
2. **Auto-Detection**: On first run with v1.0 entries, prompts for migration
3. **Encryption**: Automatically encrypts content on save using age encryption
4. **Embedding Generation**: Generates AI embeddings if content changed (requires Ollama)
5. **Database Update**: Updates entry metadata (checksum, word count)

### Examples

```bash
# Edit today's entry
ponder

# Edit last week's entries
ponder edit --retro

# Edit specific date
ponder edit --date 2024-06-15

# Migrate v1.0 entries
ponder edit --migrate
```

---

## ask

Query your journal using AI with Retrieval-Augmented Generation (RAG).

### Usage

```bash
ponder ask [OPTIONS] <QUERY>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `QUERY` | The question to ask about your journal (required) |

### Options

| Option | Description |
|--------|-------------|
| `--from DATE` | Filter results from this date (inclusive) |
| `--to DATE` | Filter results until this date (inclusive) |

### How It Works

1. Converts your query into an embedding vector
2. Searches journal entries for semantically similar content
3. Retrieves relevant excerpts from matching entries
4. Generates AI response using retrieved context

### Examples

```bash
# Ask a general question
ponder ask "What were my main goals last month?"

# Ask with date range
ponder ask "What did I learn about Rust?" --from 2024-01-01 --to 2024-06-30

# Ask about recent thoughts
ponder ask "How have I been feeling lately?" --from 2024-11-01
```

### Requirements

- Ollama must be running
- `nomic-embed-text` model (for embeddings)
- `gemma3:4b` model (for chat)
- Existing embeddings in database (generated during edit)

### Notes

- Query is **not** logged or stored in database
- Responses are generated fresh each time (not cached)
- Works best with specific, focused questions

---

## reflect

Generate AI reflection on a journal entry.

### Usage

```bash
ponder reflect [OPTIONS]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--date DATE` | `-d DATE` | Date of entry to reflect on (defaults to today) |

### How It Works

1. Retrieves the specified journal entry
2. Decrypts the content
3. Sends to AI for reflection generation
4. Displays thoughtful insights and observations

### Examples

```bash
# Reflect on today's entry
ponder reflect

# Reflect on specific date
ponder reflect --date 2024-01-15
ponder reflect -d 20240115
```

### Requirements

- Ollama must be running
- `gemma3:4b` model (for chat)
- Existing entry for the specified date

### Notes

- Reflections are generated fresh each time (not stored)
- Works best with substantial entry content (>100 words)
- Prompts designed for thoughtful, non-judgmental insights

---

## search

Semantic search over journal entries by meaning, not just keywords.

### Usage

```bash
ponder search [OPTIONS] <QUERY>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `QUERY` | Search query describing what to find (required) |

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--limit N` | `-l N` | Maximum number of results (default: 5) |
| `--from DATE` | | Filter results from this date (inclusive) |
| `--to DATE` | | Filter results until this date (inclusive) |

### How It Works

1. Converts query into an embedding vector
2. Compares against all entry embeddings using cosine similarity
3. Returns top N most semantically similar chunks
4. Decrypts and displays matching excerpts with similarity scores

### Examples

```bash
# Basic search
ponder search "anxiety and coping strategies"

# Search with custom limit
ponder search "project ideas" --limit 10

# Search within date range
ponder search "productivity" --from 2024-01-01 --to 2024-06-30 --limit 5
```

### Output Format

```
📝 Search Results (5 matches)

[1] 2024-06-15 (similarity: 0.92)
    ...excerpt from matching entry...

[2] 2024-05-20 (similarity: 0.87)
    ...excerpt from matching entry...
```

### Requirements

- Ollama must be running
- `nomic-embed-text` model
- Existing embeddings in database

### Notes

- Searches by **meaning**, not exact keywords
- Similarity scores range from 0.0 (unrelated) to 1.0 (identical)
- Results include 100 characters of context around match

---

## lock

Lock the encrypted session (clear passphrase from memory).

### Usage

```bash
ponder lock
```

### Behavior

1. Clears cached passphrase from memory
2. Zeroizes passphrase for security
3. Next command will prompt for passphrase

### When To Use

- After finishing journaling session
- Before leaving computer unattended
- When sharing terminal/screen
- After period of inactivity

### Notes

- Session also auto-locks after timeout (default: 30 minutes)
- No options or arguments
- Safe to run multiple times (idempotent)

### Example

```bash
# Finish journaling
ponder edit

# Lock session before stepping away
ponder lock
```

---

## backup

Create encrypted backup archive of entire journal.

### Usage

```bash
ponder backup [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--verify` | Verify backup integrity after creation |

### Backup Contents

The backup archive (`.tar.age`) contains:
- All encrypted journal entries (`*.md.age`)
- Encrypted database (`ponder.db`)
- Backup manifest with checksums and metadata

### Backup Location

Default: `$PONDER_DIR/backups/ponder-backup-YYYYMMDD-HHMMSS.tar.age`

### Backup Process

1. Prompts for confirmation
2. Creates `backups/` directory if needed
3. Collects all journal files
4. Generates manifest with BLAKE3 checksums
5. Creates tar archive
6. Encrypts archive with journal passphrase
7. Optionally verifies integrity

### Examples

```bash
# Create backup
ponder backup

# Create and verify backup (recommended)
ponder backup --verify
```

### Output

```
📦 Creating Backup

Collecting files...
Found 127 encrypted entries
Database size: 15.3 MB

Creating archive...
Encrypting with your journal passphrase...

✅ Backup Complete
Location: ~/Documents/rubberducks/backups/ponder-backup-20241022-143052.tar.age
Size: 18.7 MB
Files: 128
Checksum: blake3:a7f3b9c2...
```

### Security Notes

- Backup encrypted with same passphrase as journal
- Store backups in secure location (encrypted drive, secure cloud)
- Use `--verify` to ensure backup integrity
- Test restores periodically

---

## restore

Restore journal from encrypted backup archive.

### Usage

```bash
ponder restore [OPTIONS] <BACKUP_FILE>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `BACKUP_FILE` | Path to backup archive (`.tar.age`) (required) |

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--force` | `-f` | Skip confirmation and overwrite existing files |

### Restore Process

1. Prompts for passphrase
2. Decrypts backup archive
3. Verifies manifest checksums
4. Confirms overwrite (unless `--force`)
5. Extracts files to `$PONDER_DIR`
6. Validates restored checksums

### Examples

```bash
# Restore from backup (with confirmation)
ponder restore ~/backups/ponder-backup-20241022-143052.tar.age

# Force restore (skip confirmation)
ponder restore ~/backups/ponder-backup-20241022-143052.tar.age --force
```

### Output

```
🔓 Restoring from Backup

Decrypting backup archive...
Verifying manifest...

⚠️  Warning: This will overwrite existing journal files
Backup contains 128 files (18.7 MB)

Proceed with restore? [y/N]: y

Extracting files...
Verifying checksums...

✅ Restore Complete
Restored: 128 files
Total size: 18.7 MB
All checksums verified
```

### Safety

- Original files overwritten (backup beforehand if needed)
- Checksum verification ensures data integrity
- Prompts for confirmation by default
- Failed checksum stops restore

---

## cleanup-v1

Delete v1.0 plaintext files after successful migration to v2.0.

### Usage

```bash
ponder cleanup-v1 [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `--yes` | Skip confirmation prompt |

### Safety Features

Only deletes files that are:
- Successfully migrated to v2.0
- Verified with checksum match
- Status: "verified" or "migrated" in database

**Never deletes:**
- Files with failed migration
- Files not yet migrated
- Files with checksum mismatch

### Cleanup Process

1. Scans for v1.0 entries (`YYYYMMDD.md`)
2. Checks migration status in database
3. Identifies safe-to-delete files
4. Prompts for confirmation (unless `--yes`)
5. Deletes only verified files
6. Reports deletion summary

### Examples

```bash
# Delete with confirmation
ponder cleanup-v1

# Delete without prompt
ponder cleanup-v1 --yes
```

### Output

```
🧹 Cleaning Up v1.0 Entries

Found 15 v1.0 entries
Safe to delete: 14 entries
Will keep: 1 entry (migration failed)

Delete 14 verified v1.0 files? [y/N]: y

Deleting verified entries...
✓ Deleted: 20240115.md
✓ Deleted: 20240116.md
...

✅ Cleanup Complete
Deleted: 14 files
Kept: 1 file (migration verification pending)
```

### Recommendations

1. **Verify migration first**: Use `ponder search` or `ponder edit` to confirm entries are accessible
2. **Backup before cleanup**: Run `ponder backup --verify` first
3. **Test restores**: Ensure backups work before deleting originals
4. **Keep one backup**: Store final v1.0 backup in secure location

---

## Environment Variables

Ponder can be configured using environment variables:

### Core Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PONDER_DIR` | Journal directory path | `~/Documents/rubberducks` |
| `PONDER_EDITOR` | Editor command (no spaces/args) | `$EDITOR` or `vim` |
| `EDITOR` | Fallback editor if `PONDER_EDITOR` not set | `vim` |

### v2.0 Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PONDER_DB` | Database file path | `$PONDER_DIR/ponder.db` |
| `PONDER_SESSION_TIMEOUT` | Session timeout (minutes) | `30` |
| `OLLAMA_URL` | Ollama API endpoint | `http://127.0.0.1:11434` |

### Logging Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (error/warn/info/debug/trace) | `info` |
| `CI` | Force JSON log format (any value) | Not set |

### Example Configuration

```bash
# Add to ~/.bashrc or ~/.zshrc
export PONDER_DIR="$HOME/Journal"
export PONDER_EDITOR="nvim"
export PONDER_SESSION_TIMEOUT=60
export RUST_LOG=info
```

---

## Exit Codes

Ponder uses standard exit codes:

| Code | Meaning | Description |
|------|---------|-------------|
| `0` | Success | Command completed successfully |
| `1` | Error | General error (see error message) |
| `2` | Usage error | Invalid arguments or options |

### Common Error Scenarios

**Session locked:**
```bash
$ ponder edit
Error: Session locked. Please enter passphrase.
$ echo $?
1
```

**Ollama not running:**
```bash
$ ponder ask "What did I do today?"
Error: Failed to connect to Ollama at http://127.0.0.1:11434
$ echo $?
1
```

**Invalid date format:**
```bash
$ ponder edit --date "tomorrow"
Error: Invalid date format. Use YYYY-MM-DD or YYYYMMDD
$ echo $?
2
```

---

## Tips and Best Practices

### Editor Configuration

For editors requiring arguments, use a wrapper script:

```bash
#!/bin/bash
# ~/bin/ponder-code
exec code -w "$@"
```

```bash
chmod +x ~/bin/ponder-code
export PONDER_EDITOR="ponder-code"
```

### Workflow Recommendations

**Daily journaling:**
```bash
# Morning: Edit today's entry
ponder

# Evening: Reflect on today
ponder reflect
```

**Weekly review:**
```bash
# Review last week
ponder edit --retro

# Search for patterns
ponder search "progress on goals"
```

**Monthly reflection:**
```bash
# Ask about achievements
ponder ask "What did I accomplish this month?" --from 2024-11-01

# Review significant dates
ponder edit --reminisce
```

### Backup Strategy (3-2-1 Rule)

1. **3 copies**: Original + 2 backups
2. **2 different media**: Local drive + cloud storage
3. **1 offsite**: Cloud or remote location

```bash
# Weekly backup
ponder backup --verify

# Copy to external drive
cp ~/Documents/rubberducks/backups/*.tar.age /Volumes/Backup/

# Upload to secure cloud (encrypted)
rclone copy ~/Documents/rubberducks/backups/ remote:encrypted-backups/
```

### Security Checklist

- ✅ Use strong passphrase (20+ characters)
- ✅ Enable full-disk encryption
- ✅ Keep session timeout ≤ 30 minutes
- ✅ Run `ponder lock` when stepping away
- ✅ Store backups securely (encrypted cloud/drive)
- ✅ Test restores periodically
- ✅ Use Ollama locally (don't expose to network)

---

## See Also

- [README.md](../README.md) - Overview and installation
- [MIGRATION.md](../MIGRATION.md) - v1.0 → v2.0 migration guide
- [CLAUDE.md](../CLAUDE.md) - Architecture and design
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guide
