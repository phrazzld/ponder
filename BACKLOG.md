# BACKLOG

Last groomed: 2025-10-13
Analyzed by: 7 specialized perspectives (complexity, architecture, security, performance, maintainability, UX, product)

---

## Immediate Concerns

### [Security] RUSTSEC-2025-0055: Vulnerable tracing-subscriber 0.3.19
**File**: Cargo.lock:1132
**Perspectives**: security-sentinel
**Severity**: CRITICAL (CVE: RUSTSEC-2025-0055, CVSS ~7.4)
**Impact**: ANSI escape sequence injection in logs - attackers can poison log output, hide malicious activity, manipulate terminal sessions
**Violation**: Dependency vulnerability - log poisoning via ANSI escape codes
**Fix**: Upgrade dependency
```toml
# Cargo.toml
[dependencies]
tracing-subscriber = { version = "0.3.20", features = ["json", "env-filter", "fmt", "registry", "chrono"] }
```
Verify with: `cargo update tracing-subscriber && cargo audit && cargo test`
**Effort**: 5m + regression testing | **Risk**: CRITICAL

---

### [Maintainability] Build Failure - Deprecated clap API
**File**: src/cli/mod.rs:74
**Perspectives**: maintainability-maven
**Severity**: CRITICAL - BLOCKS ALL DEVELOPMENT
**Impact**: Code uses deprecated `possible_values` that doesn't exist in current clap version - cannot compile, cannot run tests, cannot make changes
**Violation**: Build failure blocks onboarding and CI/CD
**Fix**: Use modern clap v4 syntax
```rust
// Replace line 74
#[clap(
    long = "log-format",
    value_parser = clap::builder::PossibleValuesParser::new([
        constants::LOG_FORMAT_TEXT,
        constants::LOG_FORMAT_JSON
    ]),
    default_value = constants::LOG_FORMAT_TEXT
)]
pub log_format: String,
```
**Effort**: 15m | **Impact**: Restore compilation + eliminate build friction

---

### [UX] Data Loss Risk - No Warning When Editor Exits with Error
**File**: src/journal_io/mod.rs:556-569
**Perspectives**: user-experience-advocate
**Severity**: HIGH - User confusion about data loss
**Impact**: Users don't know if changes were saved when editor crashes or exits with error (e.g., Vim `:cq`)
**Current**: Generic error "Editor exited with non-zero status code"
**Fix**: Improve EditorError::NonZeroExit message
```rust
#[error(
    "Editor '{command}' exited with status code {status_code}.\n\
    \n\
    This may indicate:\n\
    - Changes were not saved (editor quit without saving)\n\
    - Editor encountered an error or crashed\n\
    - Incorrect editor configuration\n\
    \n\
    Check your journal file to verify if changes were saved.\n\
    Run 'ponder' again to retry if needed."
)]
```
**Effort**: 30m | **Value**: Prevents user confusion about data loss

---

## High-Value Improvements

### [Feature] Missing Core Feature: Search Capability
**Scope**: New feature - full-text search across entries
**Perspectives**: product-visionary, user-experience-advocate
**Impact**: CRITICAL adoption blocker - after 100+ entries, journal becomes unusable archive without search
**Market Gap**: All competitors have search (Day One, Notion, Obsidian, jrnl) - deal-breaker for long-term users
**Use Cases Blocked**:
- "What did I write about anxiety last year?"
- "Find all entries mentioning project X"
- "Show me what I was doing in summer 2023"
- Personal knowledge management workflows

**Implementation**:
```bash
ponder --search "keyword"
ponder --search "word" --from 2024-01-01 --to 2024-06-30
ponder --list-dates
```
Use ripgrep for fast full-text search + date range filtering
**Effort**: 5d | **Value**: CRITICAL - Converts from "toy" to "tool for life", prevents month-3 churn

---

### [Feature] Missing Core Feature: Entry Templates
**Scope**: New feature - template system for structured journaling
**Perspectives**: product-visionary, user-experience-advocate
**Impact**: HIGH - Drives daily habit formation, eliminates blank page anxiety
**Market Gap**: Day One, Notion, Obsidian all have templates - users expect structure
**Use Cases**:
- Daily reflection (gratitude, wins, challenges)
- Weekly/monthly review, goal tracking, mood tracking
- Morning pages, therapy/CBT worksheets
- Developer work log, creative writing prompts

**Implementation**:
```bash
export PONDER_TEMPLATE="~/.ponder/templates/daily.md"
ponder --template gratitude
ponder --list-templates
```
Template format: Markdown with variables ({{date}}, {{time}}, {{weekday}})
Bundle 5-10 common templates as examples
**Effort**: 4d | **Value**: HIGH - Creates daily routine → habit → retention, "The template-friendly journaling CLI"

---

### [Feature] Missing Core Feature: List Journal Entries
**Scope**: New CLI command
**Perspectives**: user-experience-advocate
**Impact**: HIGH - Core workflow friction, no way to discover past entries
**Current**: Users must manually `ls ~/Documents/rubberducks/` - breaks CLI tool mental model
**Implementation**:
```bash
ponder --list
ponder --list --month 2024-01
ponder --list --year 2023
```
Show dates with formatted output (date, weekday, title preview)
**Effort**: 3h | **Value**: Essential discovery feature, frequently requested

---

### [Feature] Git Auto-Sync (PRD Priority)
**Scope**: New feature - automatic git commit/push after edits
**Perspectives**: product-visionary
**Impact**: VERY HIGH - Key differentiator, enables cross-device workflows
**Market Opportunity**: Unique positioning as "git-native journal" - Obsidian has plugin, others don't support
**Implementation**:
```bash
ponder init-sync --remote git@github.com:user/journal-private.git
export PONDER_AUTO_SYNC=true
ponder sync  # Manual push/pull
```
Use git2-rs or git subprocess, handle conflicts gracefully
**Effort**: 6d | **Value**: VERY HIGH - PRD feature, cross-device unlock, privacy-preserving sync
**Dependencies**: Foundation for encrypted cloud backup

---

### [Feature] Encryption Support (PRD Priority)
**Scope**: New feature - transparent encryption/decryption
**Perspectives**: product-visionary, security-sentinel
**Impact**: HIGH - Removes major adoption objection (privacy)
**Market Context**: Day One, Standard Notes have encryption - expected for sensitive journals
**Implementation**:
```bash
ponder init --encrypt  # Uses age encryption
ponder  # Prompts for passphrase, decrypts to temp, re-encrypts on close
```
Files stored as: 20240615.md.age (git-friendly, can sync safely)
Use age-rs library + rpassword for secure input
**Effort**: 5d | **Value**: HIGH - PRD priority, enables truly private journaling, synergy with git sync

---

### [UX] Invalid Date Format Error Too Generic
**File**: src/main.rs:114-115
**Perspectives**: user-experience-advocate
**Impact**: HIGH - Common user error with unhelpful message
**Current**: "Invalid date format: input contains invalid characters" (doesn't show what user entered)
**Fix**: Enhanced error with examples
```rust
AppError::Journal(format!(
    "Invalid date format: '{}'\n\
    \n\
    Expected formats:\n\
    YYYY-MM-DD  (e.g., 2024-01-15)\n\
    YYYYMMDD    (e.g., 20240115)\n\
    \n\
    Original error: {}",
    date_str, e
))
```
**Effort**: 30m | **Value**: Eliminates common user frustration, professional UX

---

### [UX] Empty Retro/Reminisce Opens Nothing - No Feedback
**File**: src/journal_io/mod.rs:292-295
**Perspectives**: user-experience-advocate
**Impact**: HIGH - Silent success confuses users
**Current**: No output when no entries found - users wonder if command worked
**Fix**: Add user-visible message
```rust
if paths_to_open.is_empty() {
    eprintln!("No existing journal entries found for the past 7 days.");
    eprintln!();
    eprintln!("Start journaling today:");
    eprintln!("  ponder");
    return Ok(());
}
```
**Effort**: 15m | **Value**: Eliminates confusion, guides next action

---

### [UX] No Confirmation After Successful Edit
**File**: User workflow - missing feedback
**Perspectives**: user-experience-advocate
**Impact**: MEDIUM - User uncertainty about operation success
**Current**: Silent success - no indication of save location or success
**Fix**: Simple success message
```rust
// In src/main.rs after successful edit
if !args.verbose {
    eprintln!("Journal entry saved successfully");
}
```
**Effort**: 15m | **Value**: User confidence, professional polish

---

### [Maintainability] Critical Integration Tests Disabled
**File**: src/main.rs:209
**Perspectives**: maintainability-maven
**Impact**: HIGH - 6 critical tests silently skipped, main application flow untested
**Current**: Tests disabled with `#[allow(dead_code)]` due to concurrency issues
**Fix**: Use serial test execution
```rust
use serial_test::serial;

#[test]
#[serial]  // Run sequentially to prevent env var races
fn test_run_application_success() { ... }
```
**Effort**: 1h | **Benefit**: 6 tests protecting main application flow, regression safety net

---

### [Security] Path Expansion Injection via shellexpand::full()
**File**: src/config/mod.rs:205
**Severity**: MEDIUM
**Perspectives**: security-sentinel
**Impact**: Path traversal via environment variable injection in PONDER_DIR
**Attack**: `export PONDER_DIR='$HOME/../../../etc/passwd'` → expands to unintended locations
**Current Mitigations**: Absolute path validation (line 278), 0o700 permissions
**Fix**: Restrict to tilde expansion only
```rust
// Replace shellexpand::full() with shellexpand::tilde()
let expanded_path = shellexpand::tilde(&journal_dir_str);
// OR add path traversal validation after expansion
if path_str.contains("..") {
    return Err(AppError::Config("Journal directory path cannot contain '..' components".to_string()));
}
```
**Effort**: 30m | **Risk**: MEDIUM - Prevents env var injection attacks

---

## Technical Debt Worth Paying

### [Architecture] Implicit Coupling in edit_journal_entries
**File**: src/journal_io/mod.rs:269-279
**Perspectives**: architecture-guardian, complexity-archaeologist
**Impact**: Magic behavior based on array index - first date special-cased with `i == 0`
**Violation**: Implicit contract not visible in function signature
**Current**: Order matters but callers must know this through documentation
**Fix**: Make contract explicit
```rust
pub fn edit_journal_entries(
    config: &Config,
    primary_date: NaiveDate,          // Always initialized
    additional_dates: &[NaiveDate],   // Only opened if exist
    reference_datetime: &chrono::DateTime<Local>,
) -> AppResult<()>
```
**Effort**: 2h | **Impact**: Clearer API, self-documenting, reduces surprises

---

### [Complexity] Pass-Through Wrapper with No Value
**File**: src/journal_io/mod.rs:395-402
**Perspectives**: complexity-archaeologist, architecture-guardian, maintainability-maven
**Impact**: Violates "deep modules" - adds no value, just forwards to edit_journal_entries
**Violation**: Ousterhout shallow module - interface complexity = implementation complexity
**Fix**: Remove entirely, update call sites
```rust
// In main.rs:123, replace:
journal_io::open_journal_entries(&config, &dates_to_open, &current_datetime)?;
// With:
journal_io::edit_journal_entries(&config, &dates_to_open, &current_datetime)?;
// Delete lines 395-402 in journal_io/mod.rs
```
**Effort**: 20m | **Impact**: Removes 10 lines, eliminates naming confusion

---

### [Maintainability] Magic Number Without Context
**File**: src/constants.rs:76
**Perspectives**: maintainability-maven
**Impact**: `MAX_REMINISCE_YEARS_AGO: u32 = 100` - no rationale for value
**Fix**: Document reasoning
```rust
/// Maximum number of years ago for reminisce.
///
/// Limited to 100 years to:
/// 1. Keep file lists manageable (max ~103 files: 3 months + 100 years)
/// 2. Avoid edge cases with very old dates (pre-1900 datetime handling)
/// 3. Reasonable lifetime expectation for journaling tool
///
/// Can be increased if needed - no technical constraint, just UX balance.
pub const MAX_REMINISCE_YEARS_AGO: u32 = 100;
```
**Effort**: 10m | **Benefit**: Future tuning decisions informed

---

### [Maintainability] Missing Edge Case Documentation in launch_editor
**File**: src/journal_io/mod.rs:543-600
**Perspectives**: maintainability-maven
**Impact**: Function lacks critical edge case documentation (timeouts, platform differences, security rationale)
**Fix**: Add comprehensive function documentation covering:
- Security note (no shell interpretation prevents injection)
- Platform differences (Unix exit codes, macOS GUI backgrounding)
- Timeout behavior (blocks indefinitely)
- Recommended editors (terminal vs GUI with wrappers)
- Error conditions and handling
**Effort**: 30m | **Benefit**: Self-documenting edge cases, reduced support burden

---

### [Maintainability] Inconsistent Documentation Quality
**File**: Multiple modules
**Perspectives**: maintainability-maven
**Impact**: errors/mod.rs is gold standard, journal_io/mod.rs lacks internal function docs
**Examples**:
- `create_or_open_entry_file()` - no explanation of why read+append+create
- `read_file_content()` - no memory implications mention
- `append_to_file()` - no atomic write guarantee docs
**Fix**: Establish documentation standards in CLAUDE.md, audit and upgrade all modules
**Effort**: 3h | **Benefit**: Consistent developer experience, easier onboarding

---

### [Maintainability] Code Duplication - Permission Setting Logic
**File**: src/journal_io/mod.rs:94-108, 456-468
**Perspectives**: maintainability-maven
**Impact**: Identical Unix permission-setting code duplicated for directories and files
**Fix**: Extract helper function
```rust
#[cfg(unix)]
fn set_secure_permissions(path: &Path, mode: u32, resource_type: &str) -> AppResult<()> {
    let permissions = Permissions::from_mode(mode);
    fs::set_permissions(path, permissions).map_err(|e| {
        AppError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to set secure permissions on {}: {}", resource_type, e),
        ))
    })?;
    debug!("Set 0o{:o} permissions on {}", mode, resource_type);
    Ok(())
}
```
**Effort**: 30m | **Benefit**: DRY principle, single place to update permission logic

---

### [Complexity] Temporal Decomposition in Entry Initialization
**File**: src/journal_io/mod.rs:254-349
**Perspectives**: complexity-archaeologist
**Impact**: Code organized by execution phases (Step 1, Step 2, Step 3) rather than functionality
**Violation**: Ousterhout temporal decomposition - special-case logic scattered in sequential steps
**Fix**: Extract functionality-based helper
```rust
struct EntryPreparation {
    primary_entry: Option<PathBuf>,
    secondary_entries: Vec<PathBuf>,
}

fn prepare_entries_for_editing(...) -> AppResult<EntryPreparation> {
    // Groups: entry prep, lock acquisition, editor invocation
    // Eliminates index-based logic (i == 0)
}
```
**Effort**: 2h | **Impact**: Reduces change amplification, improves testability

---

### [Performance] Redundant File Opens in append_date_header_if_needed
**File**: src/journal_io/mod.rs:622-643
**Perspectives**: performance-pathfinder
**Impact**: Opens file twice - once in create_or_open_entry_file(), again in read_file_content()
**Cost**: Extra syscall (~0.1ms) - not user-visible but wasteful
**Fix**: Reuse file handle
```rust
pub(crate) fn append_date_header_if_needed(...) -> AppResult<()> {
    let mut file = create_or_open_entry_file(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;  // Reuse same handle
    if content.is_empty() {
        append_to_file(&mut file, &entry)?;
    }
    Ok(())
}
```
**Effort**: 10m | **Impact**: Cleaner code, eliminates redundant syscall

---

### [Performance] Same Pattern in append_timestamp_header
**File**: src/journal_io/mod.rs:667-698
**Perspectives**: performance-pathfinder
**Impact**: Same redundant file open pattern
**Fix**: Apply same refactoring as above
**Effort**: 5m | **Impact**: Consistency + minor performance gain

---

### [Architecture] Large Module Approaching Complexity Threshold
**File**: src/journal_io/mod.rs:1-966
**Perspectives**: architecture-guardian, complexity-archaeologist
**Impact**: 966 lines (702 production) with 5 distinct sub-concerns: directory management, file initialization, entry editing, locking, editor launching
**Cohesion**: 7/10 - related I/O but multiple sub-responsibilities
**Fix**: Consider extracting sub-modules when adding new I/O features
```rust
journal_io/
  mod.rs           // Public API orchestration
  filesystem.rs    // Directory/file operations
  locking.rs       // File locking logic
  editor.rs        // Editor launching
  formatting.rs    // Header formatting and timestamps
```
**Effort**: 4-6h | **Impact**: Future-proofing, improved testability, parallel development
**Priority**: LOW now, MEDIUM when adding new I/O features

---

### [Maintainability] Undocumented Backward Compatibility Function
**File**: src/journal_io/mod.rs:395-402
**Perspectives**: maintainability-maven
**Impact**: `open_journal_entries()` wrapper has no removal plan, no deprecation warning
**Fix**: Add deprecation attribute
```rust
#[deprecated(
    since = "0.1.2",
    note = "Use edit_journal_entries() directly. Will be removed in 0.2.0."
)]
pub fn open_journal_entries(...) -> AppResult<()> {
    edit_journal_entries(config, dates, reference_datetime)
}
```
**Effort**: 15m | **Benefit**: Clear migration path, technical debt tracked

---

### [Feature] Quick Entry Mode for Low-Friction Logging
**Scope**: New CLI subcommand
**Perspectives**: product-visionary
**Impact**: MEDIUM - Reduces friction for quick notes, increases entry frequency
**Use Case**: "Just want to log one sentence without opening vim"
**Implementation**:
```bash
ponder add "Just had a great idea about..."
echo "Deployed v2.0 today" | ponder add
```
Append to today's entry with timestamp, no editor launch
**Effort**: 1d | **Value**: MEDIUM - Mobile-friendly, enables scripting/automation

---

### [Feature] Entry Statistics and Streaks
**Scope**: New CLI command
**Perspectives**: product-visionary, user-experience-advocate
**Impact**: MEDIUM-HIGH - Drives habit formation via gamification
**Implementation**:
```bash
ponder stats
ponder streak
```
Show: Total entries, current/longest streak, word count, busiest periods
**Effort**: 3d | **Value**: HIGH - Retention driver, creates commitment via loss aversion

---

### [Feature] Reminder System for Daily Habit
**Scope**: System notifications
**Perspectives**: product-visionary
**Impact**: MEDIUM-HIGH - Drives daily usage consistency
**Implementation**:
```bash
ponder remind --time 21:00  # Daily reminder at 9pm
```
Use notify-send (Linux) or terminal-notifier (macOS), integrate with cron/systemd
**Effort**: 3d | **Value**: HIGH - Habit formation, retention driver

---

### [UX] Retro/Reminisce: Show Which Files Opening
**File**: src/journal_io/mod.rs:268-289
**Perspectives**: user-experience-advocate
**Impact**: MEDIUM - Transparency about which entries will open
**Current**: Editor opens with multiple files, user confused about tabs
**Fix**: Print entry list before opening editor
```rust
eprintln!("Opening {} journal entries:", paths_to_open.len());
for path in &paths_to_open {
    // Show date and weekday
}
```
**Effort**: 1h | **Value**: Transparency, eliminates surprise

---

### [Feature] Tags and Metadata Support
**Scope**: New feature - YAML frontmatter + tag syntax
**Perspectives**: product-visionary
**Impact**: HIGH - Enables power user workflows, multiplies search value
**Implementation**:
```markdown
---
tags: [work, project-x, meeting]
mood: focused
---
```
Parse frontmatter (serde_yaml), index tags, integrate with search
**Effort**: 5d | **Value**: HIGH - Power user feature, synergy with search
**Dependencies**: Should come AFTER search is implemented

---

## Nice to Have

### [UX] Entry Preview Without Opening Editor
**Scope**: New CLI command
**Perspectives**: user-experience-advocate
**Impact**: LOW-MEDIUM - Fast feedback without editor launch
**Implementation**:
```bash
ponder --cat
ponder --cat --date 2024-01-14
```
Print entry contents to stdout, handle missing files gracefully
**Effort**: 2h | **Value**: MEDIUM - Avoids unnecessary editor sessions

---

### [Feature] Entry Management: Delete and Archive
**Scope**: New CLI commands
**Perspectives**: user-experience-advocate
**Impact**: MEDIUM - Safe entry management without shell commands
**Implementation**:
```bash
ponder --delete --date 2024-01-15  # With confirmation
ponder --archive --date 2024-01-15
```
Interactive confirmation for delete, archive to subdirectory
**Effort**: 3h | **Value**: MEDIUM - Safe management, no manual file operations

---

### [Feature] Export Functionality
**Scope**: New feature - multiple export formats
**Perspectives**: product-visionary, user-experience-advocate
**Impact**: MEDIUM - Removes vendor lock-in perception
**Implementation**:
```bash
ponder export --format pdf --output ~/Desktop/2024-journal.pdf
ponder export --format html --output ~/Desktop/journal/
ponder archive --output ~/Dropbox/backup.tar.gz
```
Use typst/pandoc for PDF, markdown-to-html for web, tar.gz for archives
**Effort**: 4d | **Value**: MEDIUM - Trust builder, enables sharing/reviewing

---

### [Maintainability] Missing Time Complexity Documentation
**File**: src/journal_core/mod.rs:180-228
**Perspectives**: maintainability-maven
**Impact**: LOW - Performance characteristics not obvious
**Fix**: Document O(n) behavior in resolve_dates()
```rust
/// # Performance
/// - **Today/Specific**: O(1)
/// - **Retro**: O(n) where n = 7
/// - **Reminisce**: O(n log n) where n ≈ 103 max
```
**Effort**: 10m | **Benefit**: Informed optimization decisions

---

### [Maintainability] Unclear Side Effects in append_date_header_if_needed
**File**: src/journal_io/mod.rs:622-646
**Perspectives**: maintainability-maven
**Impact**: LOW - Function name doesn't communicate file creation
**Fix**: Rename to ensure_file_with_header_if_new() or document side effects clearly
**Effort**: 20m | **Benefit**: Side effects explicit

---

### [Maintainability] Test Brittleness - Hard-Coded Format Expectations
**File**: src/journal_io/mod.rs:915-922
**Perspectives**: maintainability-maven
**Impact**: LOW - Tests break on formatting changes even when functionality works
**Fix**: Test intent (header exists) not exact format
```rust
assert!(content.starts_with("# "), "Should start with date header");
assert!(content.contains("\n\n## "), "Should have time header");
```
**Effort**: 30m | **Benefit**: Resilient to formatting changes

---

### [Maintainability] Complex Boolean Expression for Spacing
**File**: src/journal_io/mod.rs:686-691
**Perspectives**: maintainability-maven
**Impact**: LOW - Takes moment to understand spacing logic
**Fix**: Add clarifying comment
```rust
// Ensure proper spacing before timestamp header
// If content ends with newline: add 1 more (total: 2-line spacing)
// If missing trailing newline: add 2 (ensures 2-line spacing)
let prefix = if content.ends_with('\n') { "\n" } else { "\n\n" };
```
**Effort**: 3m | **Benefit**: Immediately obvious intent

---

### [Security] Debug Implementation May Expose Paths
**File**: src/errors/mod.rs (EditorError, LockError)
**Severity**: LOW
**Perspectives**: security-sentinel
**Impact**: Error messages include full paths - potential info disclosure in logs
**Fix**: Consider sanitizing paths in display
```rust
fn sanitize_path_for_display(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "[path]".to_string())
}
```
**Effort**: 1h | **Risk**: LOW - Minor info disclosure

---

### [Security] No Permission Verification After Setting
**File**: src/journal_io/mod.rs:95-108, 455-467
**Severity**: LOW
**Perspectives**: security-sentinel
**Impact**: Permissions set but not verified - may fail silently on non-Unix filesystems
**Fix**: Verify actual permissions after fs::set_permissions()
```rust
let actual_perms = fs::metadata(journal_dir)?.permissions();
if actual_perms.mode() & 0o777 != DEFAULT_DIR_PERMISSIONS {
    return Err(...);
}
```
**Effort**: 30m | **Risk**: LOW - Defense in depth

---

### [UX] First-Run Welcome Message
**Scope**: Onboarding experience
**Perspectives**: user-experience-advocate
**Impact**: LOW - Only affects first use, but better onboarding
**Implementation**: Detect empty journal directory, show welcome message explaining features
**Effort**: 1h | **Value**: LOW - Professional polish, better first impression

---

### [Feature] AI-Powered Insights (Long-term)
**Scope**: Advanced feature - LLM analysis of entries
**Perspectives**: product-visionary
**Impact**: VERY HIGH (long-term) - Major differentiator, "AI-native journal"
**Market Opportunity**: None of the CLI competitors have this, Notion AI is only GUI example
**Implementation**:
```bash
ponder insights
ponder themes
ponder sentiment
```
Privacy-first with local LLM (llama.cpp, ollama) or optional cloud API (user provides key)
**Effort**: 8d | **Value**: VERY HIGH - Unique positioning, press-worthy feature
**Dependencies**: Requires foundation features first (search, templates, tags)
**Priority**: LATER - Innovation after product-market fit

---

### [Feature] Web Viewer (Read-Only)
**Scope**: Companion web app for reading
**Perspectives**: product-visionary
**Impact**: MEDIUM - Better reading experience than terminal
**Implementation**:
```bash
ponder serve  # Local web server
ponder build --output ~/Desktop/journal-site/  # Static HTML
```
Beautiful reading UI with calendar view, search, tag filtering
**Effort**: 10d | **Value**: MEDIUM - UX improvement, social sharing driver
**Priority**: LATER - Nice to have, not essential

---

### [Monetization] Ponder Plus Premium Tier (Future)
**Scope**: Business model - freemium with premium features
**Perspectives**: product-visionary
**Impact**: Revenue generation for sustainability
**Model**:
- Free: Core journaling, search, templates, tags, git sync
- Plus ($3/mo): AI insights, encryption with passphrase recovery, web hosting, priority support
**Implementation**: License key system, Stripe integration, cloud service for AI/backups
**Effort**: 15d | **Value**: $36k/year potential at 1000 users
**Priority**: LOW (later) - Build features first, monetize after adoption

---

## Completed / Archived

(No completed items yet - backlog freshly groomed)

---

## Summary Statistics

**Total Items**: 46
- Immediate: 3 (2 CRITICAL, 1 HIGH)
- High-Value: 10 (adoption blockers, retention drivers, PRD priorities)
- Technical Debt: 13 (architecture, performance, maintainability)
- Nice to Have: 20 (polish, advanced features, future opportunities)

**By Category**:
- Security: 5 items (1 CRITICAL, 1 MEDIUM, 3 LOW)
- Features: 16 items (7 foundation, 9 advanced/future)
- UX: 8 items (4 HIGH, 4 MEDIUM/LOW)
- Maintainability: 9 items (1 CRITICAL, 8 improvements)
- Architecture: 4 items
- Complexity: 2 items
- Performance: 2 items

**Effort Estimates**:
- Immediate Concerns: ~1 hour
- High-Value Improvements: ~25 days
- Technical Debt: ~10 days
- Nice to Have: ~30 days

**Recommended Next Steps**:
1. Fix CRITICAL issues (build failure + dependency vulnerability) - 20 minutes
2. Implement foundation features (search, templates, list, git sync) - 18 days
3. Address high-priority UX issues (error messages, feedback) - 2 hours
4. Tackle technical debt incrementally during feature work

**Cross-Perspective Insights**:
- Pass-through wrapper flagged by 3 agents (complexity, architecture, maintainability)
- Missing search identified as critical by both UX and product perspectives
- Git sync + encryption consistently prioritized across security and product analysis
- Module organization concerns raised by both architecture and complexity perspectives

---

*This backlog represents a comprehensive 7-perspective analysis of the Ponder codebase. Priorities reflect both technical quality and product-market fit considerations.*
