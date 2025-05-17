# T003 Plan: Enhance integration test robustness with explicit environment variables

## Implementation Approach

1. Audit all integration test files in `tests/` directory to find tests using `assert_cmd`
2. Modify each test to explicitly set:
   - `PONDER_EDITOR`: Set to `echo` (a simple command that exists on all systems)
   - `PONDER_DIR`: Set to appropriate temporary directory path
3. Ensure all tests are self-contained and don't rely on global environment variables

## Expected Changes

- `tests/cli_tests.rs`: Add env vars to CLI command tests
- `tests/journal_integration_tests.rs`: Add env vars to journal integration tests
- `tests/config_tests.rs`: Check if it uses assert_cmd and add env vars if needed

## Goal

Make tests deterministic and independent of the user's environment, preventing test failures in CI due to missing environment variables.