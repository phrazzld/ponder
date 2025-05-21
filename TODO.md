# TODO: Post-Refactoring Cleanup

## Critical Fixes (Must Complete)

- [x] **T001: Fix Documentation References**
  - Remove all references to deleted `MANIFESTO.md` in `README.md` and `PRD.md`
  - Update documentation to maintain consistency with current architecture
  - **Verification**: No broken references remain

- [x] **T002: Fix `AppError::Io` Clone Implementation**
  - Location: `src/errors/mod.rs:45-52`
  - Either remove `Clone` if not needed, OR
  - Implement lossless clone using `Arc<std::io::Error>`
  - **Verification**: Error chains are preserved through cloning

- [x] **T003: Add Security Tests for Editor Command Injection**
  - Create `tests/editor_security_tests.rs`
  - Test malicious values for `PONDER_EDITOR` and `EDITOR` environment variables
  - Verify validation correctly rejects dangerous commands
  - **Verification**: All security tests pass in CI

## High Priority Fixes

- [x] **T004: Fix Double Error Logging**
  - Remove error logging in `src/cli/mod.rs:159-167` (`to_date_specifier`)
  - Remove duplicate error logging in `main.rs` (multiple locations)
  - **Verification**: Errors are logged exactly once at the appropriate boundary

- [x] **T005: Improve Error Handling for External Editor**
  - Refactor `AppError::Editor` to use an enum with specific variants
  - Update `launch_editor` to map different failure modes to specific error variants
  - **Verification**: Error messages clearly indicate why editor launch failed

- [x] **T006: Implement Custom Debug for Sensitive Data Types**
  - Add custom `Debug` implementations for `CliArgs` and `Config`
  - Redact sensitive fields (paths, commands) in debug output
  - **Verification**: Debug logs don't expose sensitive information

- [x] **T007: Set Secure File Permissions**
  - Update `ensure_journal_directory_exists()` to set 0o700 permissions
  - Update file creation to set 0o600 permissions
  - **Verification**: Created files/directories have correct permissions

## Architecture Improvements

- [x] **T008: Remove Unnecessary Wrapper Methods**
  - Remove `CliArgs::to_date_specifier`, move logic to `main.rs`
  - Remove redundant `CliArgs::parse_args` wrapper
  - Change `CliArgs::parse_date` visibility to `pub(crate)`
  - **Verification**: No unnecessary indirection in code flow

- [x] **T009: Decouple Core Logic from App Error Type**
  - Update `DateSpecifier::from_cli_args` to return `Result<Self, chrono::ParseError>`
  - Map specific errors to `AppError` at the call site
  - **Verification**: `journal_core` has no dependencies on application error types

- [x] **T010: Separate File Initialization from Open Logic**
  - Refactor `journal_io::open_journal_entries` to only handle opening
  - Move initialization logic to `main.rs` or a separate function
  - **Verification**: Clear separation of responsibilities in code

- [x] **T011: Add Defense-in-Depth Path Validation**
  - Add absolute path check in `journal_io::ensure_journal_directory_exists`
  - Return error for non-absolute paths as a secondary safety check
  - **Verification**: Function fails on non-absolute paths

## Code Quality Improvements

- [x] **T012: Add Doc Tests for Public API**
  - Add comprehensive doc tests for all public items
  - Include basic usage examples and expected outcomes
  - **Verification**: `cargo test --doc` passes

- [x] **T013: Replace Magic Numbers/Strings with Constants**
  - Define constants for reminisce intervals in `journal_core`
  - Define constant for file extension in `journal_io`
  - **Verification**: No hardcoded literals in logic

- [x] **T014: Fix Documentation References to Old Modules**
  - Search and replace all references to `journal_logic`
  - Update with correct module paths (`journal_core`, `journal_io`)
  - **Verification**: No outdated module references remain

- [ ] **T015: Standardize Date/Time Handling**
  - Obtain current date once at high level and pass it down
  - Use consistent method for getting date/time values
  - **Verification**: Consistent date handling throughout code

## Long-Term Improvements (Future Work)

- [ ] **T016: Implement Structured Logging**
  - Migrate from `env_logger` to `tracing` ecosystem
  - Configure structured JSON logging with all required context fields
  - Add correlation IDs for each application invocation
  - **Verification**: Logs include structured, searchable context fields

- [ ] **T017: Add File Locking for Concurrent Access**
  - Implement advisory locks for journal files
  - Prevent data corruption from simultaneous writes
  - **Verification**: No data loss under concurrent access

- [ ] **T018: Create Centralized Constants Module**
  - Extract all constants to a dedicated module
  - Group related constants together
  - **Verification**: No scattered constants throughout codebase

## Verification Steps

Before submitting PR:
1. Run all tests: `cargo test --all-features`
2. Check formatting: `cargo fmt --check`
3. Run linter: `cargo clippy --all-targets -- -D warnings`
4. Build release: `cargo build --release`
5. Manual verification:
   - Test help output: `cargo run -- --help`
   - Test today entry: `cargo run`
   - Test retro: `cargo run -- --retro`
   - Test reminisce: `cargo run -- --reminisce`
   - Test specific date: `cargo run -- --date 2023-01-01`