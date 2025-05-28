# T010 Implementation Plan: Use Deterministic Values in Tests

## Overview
This plan addresses non-deterministic values in the test suite to reduce flakiness and improve test reliability.

## Sources of Non-Determinism Identified

### 1. Date/Time Dependencies
- `journal_integration_tests.rs`: Uses `Local::now()` in 4 test functions
- `locking_tests.rs`: Uses `chrono::Local::now()` for date formatting
- Tests create journal entries based on current system date

### 2. Environment Variables  
- Multiple tests manipulate environment variables (PONDER_DIR, PONDER_EDITOR, EDITOR, HOME)
- Current approach saves/restores values but could be more robust

### 3. Temporary Directories
- All test files use `tempdir()` which creates randomly named directories
- This is actually beneficial for test isolation - NO CHANGES NEEDED

## Implementation Steps

### Phase 1: Date/Time Determinism (Priority: High)

1. **Create Fixed Test Dates**
   - Define constant test dates for consistency:
     - FIXED_TEST_DATE: 2024-01-15
     - FIXED_TEST_DATETIME: 2024-01-15 14:30:00
   - These provide predictable, meaningful dates that won't change

2. **Refactor journal_integration_tests.rs**
   - Replace `Local::now()` with fixed test datetime
   - Update test expectations to use fixed dates
   - Ensure date calculations (retro, reminisce) work from fixed reference

3. **Refactor locking_tests.rs**  
   - Replace `chrono::Local::now()` with fixed date
   - Update date string formatting to use consistent date

### Phase 2: Environment Variable Robustness (Priority: Medium)

1. **Enhance Environment Cleanup**
   - Already well-handled with save/restore pattern
   - Consider adding explicit clearing at test start for extra safety
   - Document pattern for future tests

### Phase 3: Validation & Documentation (Priority: High)

1. **Test Coverage Verification**
   - Ensure all tests still validate their intended functionality
   - Run tests multiple times to verify determinism
   - Check for any date-dependent edge cases

2. **Documentation Updates**
   - Add comments explaining why fixed dates are used
   - Document the test date constants for future reference

## Test Strategy

### What to Test
- All date/time functionality works with fixed dates
- Journal entry creation uses correct filenames
- Date calculations (retro, reminisce) produce expected results
- File locking works with deterministic dates

### How to Test  
- Run full test suite to ensure no regressions
- Run date-dependent tests multiple times to verify consistency
- Manually verify journal filenames match expected patterns

## Risk Mitigation

### Risks
1. Tests might not catch date-related bugs if always using same date
2. Fixed dates might not test edge cases (month boundaries, leap years)

### Mitigation
1. Use multiple fixed dates for different scenarios if needed
2. Document why specific dates were chosen
3. Keep some integration tests with real dates for end-to-end validation

## Expected File Changes

1. `tests/journal_integration_tests.rs`
   - Add date constants
   - Replace 4 instances of `Local::now()`
   - Update test assertions if needed

2. `tests/locking_tests.rs`
   - Add date constants  
   - Replace 2 instances of `chrono::Local::now()`
   - Ensure date strings are consistent

## Success Criteria

- [ ] All identified non-deterministic date/time usage replaced
- [ ] Tests pass consistently across multiple runs
- [ ] No loss of test coverage or validation capability
- [ ] Code follows Rust conventions and project style
- [ ] Clear documentation of changes and rationale