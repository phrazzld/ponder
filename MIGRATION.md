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

### Option 1: Start Fresh (Recommended for New Users)

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

### Option 2: Manual Migration (For Existing v1.0 Users)

If you have existing v1.0 plaintext journals and want to migrate them:

1. **Backup your existing journals:**
   ```bash
   cp -r ~/Documents/rubberducks ~/Documents/rubberducks.backup
   ```

2. **Set up v2.0 in a new directory:**
   ```bash
   export PONDER_DIR="$HOME/Documents/ponder-v2"
   ponder edit  # Creates encrypted journal
   ```

3. **Manually encrypt old entries:**

   For each old entry, you can:
   - Open the v1.0 plaintext file
   - Copy the content
   - Use `ponder edit --date YYYY-MM-DD` to create the encrypted version
   - Paste and save

   Example:
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

### Option 3: Keep Both Versions

v1.0 and v2.0 can coexist safely because they use different file naming:
- v1.0: `YYYYMMDD.md` (20250115.md)
- v2.0: `YYYY/MM/DD.md.age` (2025/01/15.md.age)

You can:
1. Continue using v1.0 for old entries (read-only)
2. Use v2.0 for new entries going forward
3. Gradually migrate old entries when you review them

## First-Run Experience

### Creating New Journal (First Time)

```
$ ponder edit

🔐 Creating new encrypted journal
Choose a strong passphrase to protect your journal entries.

Enter passphrase: ****
Confirm passphrase: ****

Embedding model 'embeddinggemma' is not installed.
Would you like to pull it now? [Y/n] y

Pulling embeddinggemma (this may take a minute)...
✓ embeddinggemma ready!

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
  ollama pull embeddinggemma
  ollama pull gemma3:4b
  ```

### Disk Space

- Ponder binary: ~15MB
- SQLCipher database: ~10MB per 1000 entries
- Encrypted entries: ~1.1x plaintext size
- Ollama models: ~2.3GB total (embeddinggemma + gemma3:4b)

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
ollama pull embeddinggemma
ollama pull gemma3:4b
```

### Models not installed

**Symptom**: `Embedding model 'embeddinggemma' is not installed`

**Cause**: Ollama models not pulled

**Solution**:
```bash
# Option 1: Let ponder auto-install (interactive)
ponder edit  # Will prompt to install

# Option 2: Manual installation
ollama pull embeddinggemma
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
A: No, migration is manual. This ensures you maintain control over sensitive data.

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
- `embeddinggemma` for embeddings
- `gemma3:4b` for chat/reflections

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
- New commands: `ask`, `reflect`, `search`, `lock`
- Interactive passphrase prompts with confirmation
- Auto-installation of Ollama models
- Better error messages and guidance

See CHANGELOG.md for full details.
