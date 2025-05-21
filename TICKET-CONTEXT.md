# Plan Details

# Action Plan for Ponder Module Refactoring

This plan outlines the steps needed to address the issues identified in the code review of the Ponder module refactoring. The issues are organized into logical groups and prioritized to ensure critical issues are resolved first.

## Overview

The refactoring successfully established clearer module boundaries by separating:
- Pure logic (`journal_core`)
- I/O operations (`journal_io`)
- CLI interface (`cli`)
- Configuration management (`config`)
- Error handling (`errors`)

However, several issues need to be addressed before merging this refactoring work.

## Phase 1: Critical Blockers

### Task 1: Fix Documentation References
**Priority: BLOCKER**
- [x] Find all references to deleted `MANIFESTO.md` in documentation
- [x] Remove references or update them to point to appropriate documents
- [x] Verify all documentation remains consistent

**Success Criteria**: No broken references to `MANIFESTO.md` remain in the codebase.

### Task 2: Fix Lossy `Clone` Implementation for `AppError::Io`
**Priority: BLOCKER**
- [ ] Assess if `AppError` actually needs to be `Clone`
- [ ] If needed, refactor `AppError::Io` to use `Arc<std::io::Error>` for lossless cloning
- [ ] If not needed, remove the `Clone` implementation and update any affected code
- [ ] Add tests to verify error context is preserved

**Success Criteria**: `Clone` implementation either removed or made lossless, with error chains preserved.

### Task 3: Add Security Tests for Editor Command Injection
**Priority: BLOCKER**
- [ ] Create `tests/security_tests.rs` for security-focused integration tests
- [ ] Add tests with malicious values for `PONDER_EDITOR` and `EDITOR` environment variables
- [ ] Verify that validation correctly rejects or sanitizes these values
- [ ] Document security testing approach in README.md

**Success Criteria**: Comprehensive tests verify that editor command validation works as expected at the application boundary.

## Phase 2: High Priority Improvements

### Task 4: Fix Error Handling and Logging
**Priority: HIGH**
- [ ] Remove double logging in `CliArgs::to_date_specifier`
- [ ] Update `AppError::Editor` to use an enum with distinct error variants
- [ ] Implement comprehensive error handling for external editor execution
- [ ] Fix error propagation in `main.rs` to avoid log duplication

**Success Criteria**: Errors are logged once at the appropriate boundary, with detailed error information preserved.

### Task 5: Address Security Concerns
**Priority: HIGH**
- [ ] Implement custom `Debug` for `CliArgs` and `Config` to redact sensitive information
- [ ] Add explicit file permission setting for journal files and directories (0o600/0o700)
- [ ] Add defense-in-depth path validation in `journal_io::ensure_journal_directory_exists`

**Success Criteria**: Sensitive information is redacted in logs, and file operations use secure permissions.

### Task 6: Improve Logging Infrastructure
**Priority: HIGH**
- [ ] Migrate from `env_logger` to `tracing` ecosystem
- [ ] Configure structured logging with all required context fields
- [ ] Add correlation IDs for each application invocation
- [ ] Ensure consistent log level usage across the application

**Success Criteria**: Logs include all required context fields in a structured format.

## Phase 3: Architectural Improvements

### Task 7: Refactor Wrapper Methods
**Priority: MEDIUM**
- [ ] Remove `CliArgs::to_date_specifier` and implement logic in `main.rs`
- [ ] Remove redundant `CliArgs::parse_args` wrapper
- [ ] Update visibility of `CliArgs::parse_date` to `pub(crate)`
- [ ] Update affected tests to use the proper public API

**Success Criteria**: Unnecessary wrapper methods removed, with logic moved to appropriate locations.

### Task 8: Improve Module Boundaries
**Priority: MEDIUM**
- [ ] Refactor `DateSpecifier::from_cli_args` to return a specific error type instead of `AppResult`
- [ ] Separate file initialization logic from `journal_io::open_journal_entries`
- [ ] Move file initialization decisions to `main.rs` or a dedicated orchestration function
- [ ] Update tests to verify improved separation of concerns

**Success Criteria**: Clear separation between pure logic and I/O operations, with proper error handling.

## Phase 4: Code Quality Improvements

### Task 9: Documentation Updates
**Priority: LOW**
- [ ] Add doc tests for all public API items
- [ ] Update outdated module references in documentation
- [ ] Improve API documentation clarity
- [ ] Run `cargo doc --open` to verify documentation completeness

**Success Criteria**: Complete, accurate documentation with executable examples.

### Task 10: Code Cleanup
**Priority: LOW**
- [ ] Define constants for magic numbers and strings
- [ ] Standardize date/time handling across the codebase
- [ ] Fix missing newlines at end of files
- [ ] Remove commented-out code and unused imports

**Success Criteria**: Code follows consistent patterns and standards.

## Verification

For each task:
1. Run full test suite: `cargo test --all-features`
2. Verify formatting: `cargo fmt --check`
3. Run linter: `cargo clippy --all-targets -- -D warnings`
4. Build in release mode: `cargo build --release`
5. Manually test key scenarios

## Final Verification

Before merging:
- [ ] All tasks completed and verified
- [ ] No regressions in functionality
- [ ] All tests passing
- [ ] Documentation up-to-date
- [ ] Branch rebased on latest master

## Metrics

- Critical blockers resolved: 0/3
- High priority issues resolved: 0/3
- Medium priority issues resolved: 0/2
- Low priority issues resolved: 0/2
- Overall progress: 0/10 tasks (0%)

## Task Breakdown Requirements
- Create atomic, independent tasks
- Ensure proper dependency mapping
- Include verification steps
- Follow project task ID and formatting conventions
