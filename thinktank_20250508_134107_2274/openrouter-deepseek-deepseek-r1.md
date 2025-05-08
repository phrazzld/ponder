# Todo

## Core Infrastructure
- [ ] **T001 · Refactor · P0: Create errors module with AppError and AppResult**
    - **Context:** Error Handling Foundation (errors.rs)
    - **Action:**
        1. Create errors.rs with AppError enum using thiserror
        2. Define ConfigError, IoError, JournalError, EditorError variants
        3. Implement From traits for error conversions
    - **Done‑when:**
        1. All error variants defined with proper source tracking
        2. cargo check passes with error types used in function signatures
    - **Depends‑on:** none

## Configuration
- [ ] **T002 · Refactor · P1: Implement config module with loading logic**
    - **Context:** Configuration Module (config.rs)
    - **Action:**
        1. Create Config struct with journal_dir and editor_cmd fields
        2. Implement Config::load() using shellexpand for path resolution
        3. Add environment variable fallbacks with validation
    - **Done‑when:**
        1. Config loads successfully in test cases
        2. Handles missing env vars with default paths
    - **Verification:**
        1. Test with PONDER_DIR unset validates default location
    - **Depends‑on:** T001

## File System Abstraction
- [ ] **T003 · Refactor · P1: Implement JournalIO trait with filesystem adapter**
    - **Context:** I/O Adapter Trait & Implementation (journal/io.rs)
    - **Action:
        1. Define JournalIO trait with file operations
        2. Create FilesystemJournalIO struct implementing trait
        3. Add PathBuf handling for journal entries
    - **Done‑when:**
        1. All trait methods have concrete implementations
        2. Unit tests demonstrate file operations without side effects
    - **Depends‑on:** T001

## Editor Integration
- [ ] **T004 · Refactor · P1: Create editor trait and system implementation**
    - **Context:** Editor Adapter Trait & Implementation (editor.rs)
    - **Action:
        1. Define Editor trait with open_files method
        2. Implement SystemEditor using std::process::Command
        3. Handle command execution errors properly
    - **Done‑when:
        1. Mock editor command execution verified in tests
        2. Error handling captures failed process launches
    - **Depends‑on:** T001

## Core Logic
- [ ] **T005 · Refactor · P2: Implement journal service with date handling**
    - **Context:** Core Journal Logic (journal/mod.rs)
    - **Action:
        1. Create JournalService struct with dependency injection
        2. Implement date calculations using chrono
        3. Integrate JournalIO and Editor traits
    - **Done‑when:
        1. open_entry creates files with correct paths/dates
        2. reminisce_entry returns valid historical paths
    - **Depends‑on:** T002, T003, T004

## CLI Interface
- [ ] **T006 · Refactor · P2: Build CLI argument parser using clap**
    - **Context:** CLI Module (cli.rs)
    - **Action:
        1. Implement CliArgs struct with clap derive
        2. Add subcommands for new/retro/reminisce
        3. Validate date format inputs
    - **Done‑when:
        1. All subcommands parse successfully in tests
        2. Help text generated correctly
    - **Depends‑on:** none

## Application Wiring
- [ ] **T007 · Refactor · P2: Implement main application wiring**
    - **Context:** Main Application Logic (main.rs)
    - **Action:
        1. Initialize logging with env_logger
        2. Wire config, journal service, and CLI args
        3. Implement top-level error handling
    - **Done‑when:
        1. End-to-end test opens editor with test file
        2. Errors display user-friendly messages
    - **Depends‑on:** T005, T006

## Testing Infrastructure
- [ ] **T008 · Test · P1: Create unit test framework for core modules**
    - **Context:** Testing Strategy - Unit Tests
    - **Action:
        1. Add test modules for journal, config, errors
        2. Implement test doubles for JournalIO/Editor
        3. Add date calculation test cases
    - **Done‑when:
        1. cargo test runs isolated unit tests
        2. 90% line coverage for journal module
    - **Depends‑on:** T003, T004

## Integration Testing
- [ ] **T009 · Test · P2: Implement integration tests with tempdir**
    - **Context:** Testing Strategy - Integration Tests
    - **Action:
        1. Create tests/ directory structure
        2. Test filesystem interactions using tempfile
        3. Verify editor command invocation patterns
    - **Done‑when:
        1. Tests validate journal file creation flow
        2. Mock editor confirms correct file paths received
    - **Depends‑on:** T007

## Observability
- [ ] **T010 · Feature · P2: Add structured logging implementation**
    - **Context:** Logging & Observability
    - **Action:
        1. Configure env_logger with JSON output
        2. Add contextual logging to key operations
        3. Implement correlation ID for traceability
    - **Done‑when:
        1. Logs show structured events in CI output
        2. Error logs include source chain details
    - **Depends‑on:** T007

## Security & Validation
- [ ] **T011 · Bugfix · P1: Add input sanitization for editor commands**
    - **Context:** Security & Config - Input Validation
    - **Action:
        1. Validate editor_cmd format before execution
        2. Add security tests for command injection vectors
    - **Done‑when:
        1. Malformed editor commands rejected at startup
        2. Cargo audit shows no vulnerabilities
    - **Depends‑on:** T002, T004

## Documentation
- [ ] **T012 · Chore · P3: Update README with installation/usage**
    - **Context:** Documentation - Required Readme Updates
    - **Action:
        1. Document CLI commands and configuration
        2. Add examples for common workflows
    - **Done‑when:
        1. README.md passes markdown linter checks
        2. New contributors can run basic operations
    - **Depends‑on:** T007

### Clarifications & Assumptions
- [ ] **Issue:** MSRV (Minimum Supported Rust Version) undefined
    - **Context:** Risk Matrix - MSRV question
    - **Blocking?:** yes
- [ ] **Issue:** Date format standardization needed
    - **Context:** Open Questions - Date/Time Formatting
    - **Blocking?:** no
- [ ] **Issue:** Editor fallback behavior unclear
    - **Context:** Open Questions - Editor Fallback
    - **Blocking?:** yes