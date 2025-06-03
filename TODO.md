# Implementation TODO: Fix Structured Error Logging at Application Boundary

**Issue**: #43 - No structured error logging at application boundary  
**Target**: Implement proper structured logging for all application errors  
**Status**: Ready to execute

---

## Phase 1: Core Implementation (45 minutes)

### T100: Function Extraction
- [ ] T100.1: Create `run_application(correlation_id: &str) -> AppResult<()>` function
- [ ] T100.2: Move all main() logic except CLI parsing to `run_application()`
- [ ] T100.3: Pass correlation_id parameter to `run_application()`
- [ ] T100.4: Update tracing span creation to use passed correlation_id
- [ ] T100.5: Verify no logic is lost in the move

### T101: Error Boundary Implementation  
- [ ] T101.1: Modify main() to not return Result
- [ ] T101.2: Add match statement to handle `run_application()` result
- [ ] T101.3: Implement structured error logging with tracing::error!
- [ ] T101.4: Include error, error_chain, and correlation_id in structured log
- [ ] T101.5: Add user-friendly eprintln! for CLI users
- [ ] T101.6: Implement proper exit codes (0 for success, 1 for error)

### T102: Context Preservation
- [ ] T102.1: Ensure correlation_id is properly propagated  
- [ ] T102.2: Verify tracing spans are maintained correctly
- [ ] T102.3: Test both JSON and text logging formats work
- [ ] T102.4: Confirm no logging context is lost

---

## Phase 2: Testing Implementation (60 minutes)

### T200: Unit Tests
- [ ] T200.1: Add test for `run_application()` success case
- [ ] T200.2: Add test for `run_application()` with Config error
- [ ] T200.3: Add test for `run_application()` with Journal error  
- [ ] T200.4: Add test for `run_application()` with Editor error
- [ ] T200.5: Add test for `run_application()` with Lock error
- [ ] T200.6: Add test for `run_application()` with IO error
- [ ] T200.7: Verify error propagation through function boundary

### T201: Integration Tests for Error Boundary
- [ ] T201.1: Add `test_structured_error_logging_boundary()` function
- [ ] T201.2: Test that errors produce structured JSON logs when CI=true
- [ ] T201.3: Test that correlation IDs appear in error logs
- [ ] T201.4: Test that error chains are preserved in structured logs
- [ ] T201.5: Test that service_name and app_invocation span context preserved

### T202: User Experience Tests
- [ ] T202.1: Add `test_user_friendly_error_output()` function
- [ ] T202.2: Test that stderr contains readable error messages
- [ ] T202.3: Test proper exit codes for different error types
- [ ] T202.4: Test that error messages don't contain raw Debug formatting
- [ ] T202.5: Verify enhanced error messages (from T004) still appear

### T203: Error Format Tests
- [ ] T203.1: Test JSON logging format in CI environment
- [ ] T203.2: Test text logging format in development  
- [ ] T203.3: Test correlation_id presence in both formats
- [ ] T203.4: Test error chain visibility in structured logs
- [ ] T203.5: Test that no double logging occurs (verify T002 still works)

---

## Phase 3: Quality Assurance (30 minutes)

### T300: Code Quality
- [ ] T300.1: Run `cargo test` - ensure all existing tests pass
- [ ] T300.2: Run `cargo clippy --all-targets -- -D warnings`
- [ ] T300.3: Run `cargo fmt --check`
- [ ] T300.4: Check for any new compiler warnings
- [ ] T300.5: Verify no performance regressions in test timing

### T301: Documentation Updates
- [ ] T301.1: Update main() function documentation
- [ ] T301.2: Add documentation for `run_application()` function
- [ ] T301.3: Add inline comments explaining error boundary logic
- [ ] T301.4: Update examples in doc comments to reflect new structure
- [ ] T301.5: Ensure CLAUDE.md reflects any architectural changes

### T302: Manual Testing
- [ ] T302.1: Test config error scenario manually (invalid editor)
- [ ] T302.2: Test journal error scenario manually (invalid date)
- [ ] T302.3: Test editor error scenario manually (missing editor)
- [ ] T302.4: Test lock error scenario manually (concurrent access)
- [ ] T302.5: Test IO error scenario manually (permission denied)
- [ ] T302.6: Verify correlation IDs in JSON logs manually
- [ ] T302.7: Verify user-friendly messages in text mode manually

---

## Phase 4: Validation & Completion (15 minutes)

### T400: Final Validation
- [ ] T400.1: Run complete test suite one final time
- [ ] T400.2: Test in CI environment (CI=true) for JSON logging
- [ ] T400.3: Test in development environment for text logging  
- [ ] T400.4: Verify issue #43 requirements are fully met
- [ ] T400.5: Check that no existing functionality is broken

### T401: Issue Resolution
- [ ] T401.1: Update issue #43 with implementation details
- [ ] T401.2: Document any changes in behavior for users
- [ ] T401.3: Update related GitHub issues if needed
- [ ] T401.4: Prepare summary of changes for code review

---

## Implementation Notes

### Critical Success Criteria
- All application errors MUST produce structured logs with correlation_id
- User experience MUST remain unchanged (friendly error messages)
- Exit codes MUST be preserved (0 success, 1 failure)
- All existing tests MUST continue to pass
- No double logging MUST be maintained (T002 compliance)

### Key Technical Details  
- `run_application()` signature: `fn run_application(correlation_id: &str) -> AppResult<()>`
- Error boundary in main() uses `tracing::error!` for structured logging
- Correlation ID passed explicitly to maintain context
- Both JSON (CI) and text (dev) logging formats supported

### Risk Mitigation
- Move main() logic exactly as-is to minimize regression risk
- Test each error type individually to ensure proper handling
- Verify correlation ID propagation with specific tests
- Maintain existing error message quality and format

---

## Execution Status

**Started**: Not yet started  
**Current Phase**: Phase 1 - Core Implementation  
**Next Task**: T100.1 - Create run_application function  
**Estimated Completion**: 2.5 hours from start