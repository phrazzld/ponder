# TODO: Post-Refactoring CI Improvements

## File Locking Tests (Priority 1)

- [x] **T001: Make file locking test resilient to timing variations**
  - Update `test_file_locking_prevents_concurrent_access` in `locking_tests.rs` to use a more robust approach for the sentinel file
  - Incorporate appropriate timeouts for operations involving file state checks or inter-process coordination
  - **Verification**: Test passes consistently over 10+ local runs without timing-related failures

- [x] **T002: Add proper cleanup for temporary files and processes in locking test**
  - Ensure temporary files are always removed, even if the test fails
  - Ensure any child processes spawned by the test are properly cleaned up on test completion or failure
  - **Verification**: No temporary files or zombie processes persist after test execution

- [~] **T003: Add robust error handling for temporary file operations in locking test**
  - Enhance `test_file_locking_prevents_concurrent_access` with better error handling for all temporary file operations
  - Use `Result`, `?` operator, and clear error messages
  - **Verification**: Failures lead to clear, actionable error messages rather than panics

- [x] **T004: Implement retry logic for file locking test** (depends on T001)
  - Wrap the core logic with a retry mechanism for transient failures
  - Use a loop with a small, fixed number of attempts and short delays
  - **Verification**: Test attempts core assertions multiple times before finally failing

## CI Triggering & Workflow (Priority 0 - Immediate)

- [x] **T005: Push feat/refactor-module-boundaries branch to remote**
  - Push the local branch to the remote repository
  - **Verification**: Check the remote repository to confirm the branch exists

- [x] **T006: Create pull request for feat/refactor-module-boundaries** (depends on T005)
  - Create a PR from the branch to the main branch
  - Ensure CI workflow is automatically triggered
  - **Verification**: Observe the PR and check the CI status

- [x] **T007: Monitor initial CI workflow for the PR** (depends on T006)
  - Monitor the CI workflow run triggered by the PR creation
  - Document any failures or new issues that arise
  - **Verification**: Initial CI run has completed with results documented

- [x] **T008: Verify GitHub permissions and CI settings** (depends on T006)
  - Verify GitHub repository permissions and CI workflow configurations are correct
  - Check branch protection rules and workflow triggers
  - **Verification**: Permissions and settings confirmed correct or issues identified

- [x] **T014: Fix clippy CI failure blocking PR** (depends on T007)
  - Fix io::Error::other() clippy lint error in src/errors/mod.rs:344
  - Update test code to use newer io::Error::other() method
  - **Verification**: CI passes and PR can be merged

## Testing Robustness Improvements (Priority 2)

- [x] **T009: Add more robust error handling in tests (general)**
  - Review existing tests for areas where error handling can be improved
  - Implement improvements to ensure tests fail with clear, informative messages
  - **Verification**: Selected tests have improved error handling with clearer failure diagnostics

- [x] **T010: Use deterministic values in tests where possible**
  - Identify and refactor tests that use non-deterministic values
  - Replace with deterministic inputs or mock equivalents
  - **Verification**: Flakiness due to non-deterministic test data is reduced

- [x] **T011: Add better debug output for test failures**
  - Enhance selected tests to provide more detailed debug output when failures occur
  - Include relevant state and input values in failure logs
  - **Verification**: Diagnosing failures is easier due to improved contextual information

## CI Configuration Improvements (Priority 2)

- [x] **T012: Add CI config parameter to increase timeouts for flaky tests** (depends on T007)
  - Add or modify CI configuration to allow increased execution timeouts for flaky tests
  - Target file locking tests specifically
  - **Verification**: CI configuration reflects new timeout settings

- [ ] **T013: Add specific CI test step for file locking tests** (depends on T007)
  - Modify CI workflow to run file locking tests as a separate, distinct step
  - Allow for independent monitoring and timeout configuration
  - **Verification**: CI pipeline executes file locking tests in an isolated step

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