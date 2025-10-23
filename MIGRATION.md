# Migrating to Ponder v2.0

This guide explains how to migrate from Ponder v1.0 (plaintext) to v2.0 (encrypted with AI features).

## Overview

Ponder v2.0 introduces significant changes to data storage and security:

- **Encryption**: All journal entries are encrypted using age passphrase encryption
- **Database**: Metadata and embeddings stored in SQLCipher encrypted database
- **Directory Structure**: Changed from flat to YYYY/MM/DD.md.age hierarchy
- **AI Features**: Semantic search, reflections, and RAG queries

## Key Differences

### v1.0 (Plaintext)
```
~/Documents/rubberducks/
├── 20250115.md
├── 20250116.md
└── 20250117.md
```

### v2.0 (Encrypted)
```
~/Documents/rubberducks/
├── 2025/
│   └── 01/
│       ├── 15.md.age  (encrypted)
│       ├── 16.md.age  (encrypted)
│       └── 17.md.age  (encrypted)
└── ponder.db  (encrypted SQLCipher database)
```

## Migration Strategy

### Option 1: Automatic Migration (Recommended)

**Ponder v2.0 automatically detects v1.0 entries and offers to migrate them.**

When you run `ponder edit` for the first time with existing v1.0 entries (files matching `YYYYMMDD.md` in your journal directory), Ponder will:

1. **Detect v1.0 entries:**
   ```
   📦 v1.0 Journal Entries Detected
   Found 15 plaintext v1.0 entries that can be migrated to encrypted v2.0 format.

   What happens during migration:
   • Read each v1.0 plaintext entry (YYYYMMDD.md)
   • Encrypt content with your passphrase (age encryption)
   • Save as v2.0 encrypted entry (YYYY/MM/DD.md.age)
   • Generate AI embeddings for semantic search (optional)
   • Verify encryption succeeded (checksum validation)
   • Original v1.0 files remain untouched

   Migrate now? [y/N]:
   ```

2. **Interactive migration process:**
   ```
   [1/15] ✓ Migrated: 2024-01-15 (512 bytes)
   [2/15] ✓ Migrated: 2024-01-16 (1.2 KB)
   [3/15] ✗ Failed: 2024-01-17 - File not found
   [4/15] ✓ Migrated: 2024-01-18 (2.4 KB)
   ...
   [15/15] ✓ Migrated: 2024-06-30 (890 bytes)

   ✅ Migration Complete
   Successfully migrated: 14 entries
   Failed: 1 entry
   Total size: 18.2 KB

   Original v1.0 files remain in your journal directory.
   Use 'ponder cleanup-v1 --yes' to delete them after verification.
   ```

3. **Resume capability:**
   If migration is interrupted or partially completed, Ponder will:
   - Skip already-migrated entries (tracked in database)
   - Only migrate pending entries
   - Show updated progress (e.g., "2 already migrated, 13 pending")

4. **Verify migration:**
   ```bash
   # Check all entries are accessible
   ponder search "test"

   # Verify specific dates
   ponder edit --date 2024-01-15
   ```

5. **Clean up v1.0 files (optional):**
   ```bash
   # After verifying migration, delete original v1.0 plaintext files
   ponder cleanup-v1

   # Or skip confirmation
   ponder cleanup-v1 --yes
   ```

**Safety features:**
- ✅ Original v1.0 files never deleted automatically
- ✅ Each entry verified with BLAKE3 checksum after encryption
- ✅ Migration state tracked in database (resume from interruption)
- ✅ Non-fatal errors (one failure doesn't stop entire migration)
- ✅ Detailed progress display with file counts and sizes

**Manual migration trigger:**
```bash
# Explicitly trigger migration (bypass auto-prompt)
ponder edit --migrate
```

### Option 2: Start Fresh (New Users)

If you don't have existing v1.0 journals, simply start using v2.0:

```bash
# First run will prompt for passphrase
ponder edit
```

You'll see:
```
🔐 Creating new encrypted journal
Choose a strong passphrase to protect your journal entries.

Enter passphrase: ****
Confirm passphrase: ****
```

### Option 3: Manual Migration (Fallback)

If automatic migration doesn't work for your use case:

1. **Backup your existing journals:**
   ```bash
   cp -r ~/Documents/rubberducks ~/Documents/rubberducks-backup-$(date +%Y%m%d)
   ```

2. **Set up v2.0 in same directory:**
   ```bash
   ponder edit  # Creates encrypted journal, detects v1.0 entries
   ```

3. **Manually encrypt specific entries:**
   ```bash
   # For entry 20250115.md
   cat ~/Documents/rubberducks/20250115.md
   # Copy output

   ponder edit --date 2025-01-15
   # Paste content in editor, save
   ```

4. **Verify migration:**
   ```bash
   ponder search "test query"  # Should find migrated entries
   ```

### Option 4: Keep Both Versions

v1.0 and v2.0 can coexist safely because they use different file naming:
- v1.0: `YYYYMMDD.md` (20250115.md) - plaintext
- v2.0: `YYYY/MM/DD.md.age` (2025/01/15.md.age) - encrypted

You can:
1. Continue using v1.0 for old entries (read-only)
2. Use v2.0 for new entries going forward
3. Gradually migrate old entries when you review them
4. Use `ponder edit --migrate` when ready to migrate remaining v1.0 entries

## First-Run Experience

### Creating New Journal (First Time)

```
$ ponder edit

🔐 Creating new encrypted journal
Choose a strong passphrase to protect your journal entries.

Enter passphrase: ****
Confirm passphrase: ****

Embedding model 'nomic-embed-text' is not installed.
Would you like to pull it now? [Y/n] y

Pulling nomic-embed-text (this may take a minute)...
✓ nomic-embed-text ready!

[Editor opens]
```

### Unlocking Existing Journal

```
$ ponder edit

🔓 Unlocking encrypted journal
Enter passphrase: ****

[Editor opens]
```

### Wrong Passphrase Handling

```
$ ponder edit

🔓 Unlocking encrypted journal
Enter passphrase: ****

Incorrect passphrase. Please try again (attempt 2/3).

Enter passphrase: ****

[Editor opens]
```

After 3 failed attempts:
```
Error: Maximum passphrase attempts exceeded. Please try again later.
```

## Prerequisites

### System Requirements

- **Rust**: 1.79+ (for building from source)
- **Ollama**: 0.1.20+ (for AI features)
  ```bash
  # Install Ollama
  curl -fsSL https://ollama.com/install.sh | sh

  # Pull required models
  ollama pull nomic-embed-text
  ollama pull gemma3:4b
  ```

### Disk Space

- Ponder binary: ~15MB
- SQLCipher database: ~10MB per 1000 entries
- Encrypted entries: ~1.1x plaintext size
- Ollama models: ~3.6GB total (nomic-embed-text ~270MB + gemma3:4b ~3.3GB)

## Configuration

### Environment Variables

v2.0 adds new configuration options:

```bash
# Journal directory (default: ~/Documents/rubberducks)
export PONDER_DIR="$HOME/Journal"

# Editor command (default: vim)
export PONDER_EDITOR="nvim"

# Database path (default: $PONDER_DIR/ponder.db)
export PONDER_DB="$HOME/.local/share/ponder/ponder.db"

# Session timeout in minutes (default: 30)
export PONDER_SESSION_TIMEOUT=60

# Ollama API URL (default: http://127.0.0.1:11434)
export OLLAMA_URL="http://localhost:11434"
```

### Passphrase Best Practices

1. **Use a strong passphrase**
   - At least 20 characters
   - Mix of words, numbers, symbols
   - Use a passphrase manager if needed

2. **Don't forget your passphrase**
   - **There is NO recovery option**
   - If lost, all encrypted data is permanently inaccessible
   - Consider storing it securely (password manager, secure note)

3. **Session timeout**
   - Default: 30 minutes of inactivity
   - Adjust via `PONDER_SESSION_TIMEOUT`
   - Shorter = more secure, more prompts
   - Longer = more convenient, less secure

## Security Considerations

### What v2.0 Protects

✅ **Encrypted at rest**: All journal files and database
✅ **Session timeout**: Auto-lock after inactivity
✅ **Secure temp files**: RAM-only storage when available (tmpfs)
✅ **Passphrase zeroization**: Memory cleared after use
✅ **File permissions**: 0o600 (owner read/write only)

### What v2.0 Does NOT Protect

❌ **Active malware**: Keyloggers can capture passphrase
❌ **RAM scraping**: Decrypted content in memory during active session
❌ **Forensic recovery**: Temp files may persist on non-tmpfs systems
❌ **Weak passphrases**: Dictionary attacks are possible

### Recommendations

1. **Enable full-disk encryption** (FileVault/LUKS/BitLocker)
2. **Use Ollama locally** (don't expose to network)
3. **Run on trusted systems only**
4. **Use tmpfs** for temp files (Linux/BSD: /dev/shm)
5. **Keep session timeout short** (30 minutes or less)

## Testing Your Migration

After migrating, verify everything works:

```bash
# 1. Create a test entry
ponder edit --date 2025-01-01

# 2. Search for it
ponder search "test"

# 3. Ask a question
ponder ask "What did I write on January 1st?"

# 4. Generate reflection
ponder reflect --date 2025-01-01

# 5. Lock session
ponder lock

# 6. Verify re-prompt works
ponder edit
```

## Troubleshooting

### "Session locked" error on first run

**Symptom**: `Error: Session locked. Run command again...`

**Cause**: This was a bug in early v2.0 builds (fixed in commit e331a65+)

**Solution**: Update to latest v2.0 or rebuild from source

### "Wrong passphrase" error

**Symptom**: `Incorrect passphrase for database`

**Cause**:
- Typo in passphrase
- Using wrong passphrase for this journal
- Corrupted database

**Solution**:
1. Triple-check passphrase (case-sensitive!)
2. If forgotten, data is unrecoverable (restore from backup)
3. If corrupted, restore from backup or start fresh

### Ollama not found

**Symptom**: `Failed to connect to Ollama`

**Cause**: Ollama not installed or not running

**Solution**:
```bash
# Check if Ollama is running
ollama list

# If not installed
curl -fsSL https://ollama.com/install.sh | sh

# If installed but not running
ollama serve &

# Pull required models
ollama pull nomic-embed-text
ollama pull gemma3:4b
```

### Models not installed

**Symptom**: `Embedding model 'nomic-embed-text' is not installed`

**Cause**: Ollama models not pulled

**Solution**:
```bash
# Option 1: Let ponder auto-install (interactive)
ponder edit  # Will prompt to install

# Option 2: Manual installation
ollama pull nomic-embed-text
ollama pull gemma3:4b
```

## Rollback to v1.0

If you need to rollback to v1.0:

1. **Keep v1.0 binary** before upgrading
2. **Use separate directory** for v2.0 data
3. **Restore from backup** if needed

```bash
# Rollback steps
mv ~/Documents/ponder-v2 ~/Documents/ponder-v2.backup
git checkout v1.0  # or use v1.0 binary
cargo build --release
./target/release/ponder
```

## FAQ

**Q: Can I use v1.0 and v2.0 simultaneously?**
A: Yes, they use different file formats and can coexist in the same directory.

**Q: Is there automatic migration from v1.0 to v2.0?**
A: Yes! Ponder v2.0 automatically detects v1.0 entries and offers to migrate them. When you run `ponder edit` with v1.0 entries in your journal directory, you'll be prompted to migrate. The process is interactive, safe (original files never deleted automatically), and can be resumed if interrupted.

**Q: What happens if I forget my passphrase?**
A: **All encrypted data is permanently lost**. There is no recovery mechanism. This is by design for security.

**Q: Can I change my passphrase later?**
A: Not currently supported. You would need to:
1. Decrypt all entries with old passphrase
2. Create new encrypted journal with new passphrase
3. Re-encrypt all entries

**Q: How does session timeout work?**
A: After 30 minutes (configurable) of inactivity, the cached passphrase is cleared. You'll need to re-enter it on the next command.

**Q: Can I disable encryption?**
A: No, v2.0 requires encryption. Use v1.0 if you don't want encryption.

**Q: What if Ollama is offline?**
A: AI features (ask, reflect, search) will fail gracefully with a clear error. You can still edit entries.

**Q: How much disk space do embeddings use?**
A: Approximately 10MB per 1000 entries (768-dimensional vectors + metadata).

**Q: Can I use different Ollama models?**
A: Not currently configurable. The defaults are:
- `nomic-embed-text` for embeddings
- `gemma3:4b` for chat/reflections

**Q: How do I safely delete v1.0 files after migration?**
A: Use the `ponder cleanup-v1` command. It will:
1. Verify that each v1.0 file has been successfully migrated and encrypted
2. Only delete files with "verified" or "migrated" status in the database
3. Prompt for confirmation before deleting (unless you use `--yes`)
4. Never delete files that failed migration or haven't been migrated yet

**Q: What if migration is interrupted?**
A: Migration can be safely resumed. Ponder tracks migration state in the database. When you run `ponder edit --migrate` again, it will skip already-migrated entries and only process pending ones. Your progress is never lost.

## Getting Help

- **GitHub Issues**: https://github.com/phrazzld/ponder/issues
- **Documentation**: See CLAUDE.md for architecture details
- **Security Issues**: Report privately to maintainers

## Changelog Summary

### v2.0 Major Changes

**Security:**
- Age passphrase-based encryption for all journal files
- SQLCipher encrypted database for metadata/embeddings
- Session management with auto-lock timeout
- Secure temporary file handling

**AI Features:**
- Semantic search over journal entries
- RAG (Retrieval-Augmented Generation) queries
- AI-powered reflections on entries
- Automatic embedding generation

**Architecture:**
- Hierarchical directory structure (YYYY/MM/DD.md.age)
- SQLCipher database for entry metadata
- Ollama integration for local AI
- Enhanced error handling and retry logic

**CLI:**
- New commands: `ask`, `reflect`, `search`, `lock`, `backup`, `restore`, `cleanup-v1`
- Automatic v1.0 migration detection with interactive prompt
- Migration resume capability (tracks state in database)
- `--migrate` flag to manually trigger migration
- Interactive passphrase prompts with confirmation
- Auto-installation of Ollama models
- Better error messages and guidance

See CHANGELOG.md for full details.
