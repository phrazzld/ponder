# Todo

## Standardize Error Handling & Logging

- [x] **T001 · Refactor · P0: replace `unwrap()` for log filter setup in `main.rs`**
    - **Context:** PLAN.md: Phase 1, Step 1.1
    - **Action:**
        1. Modify `src/main.rs:93` to replace `unwrap()` with `.map_err(|e| AppError::Config(format!("Invalid log level configuration: {}", e)))?` as specified in the target code.
    - **Done‑when:**
        1. No `unwrap()` or `expect()` calls remain in `src/main.rs` production code.
        2. Log configuration failures propagate as `AppError::Config`.
        3. A clear error message for invalid log level settings is produced.
    - **Verification:**
        1. Run the application with a valid log level environment variable (e.g., `RUST_LOG=info`) and verify it starts.
        2. Attempt to run the application with an invalid log level (e.g., by setting `constants::DEFAULT_LOG_LEVEL` to an invalid string temporarily, or via environment variable if that path is tested) and verify it exits gracefully with the specified error message.
    - **Depends‑on:** none

- [x] **T002 · Refactor · P1: remove `error!()` logging from lock acquisition functions in `journal_io/mod.rs`**
    - **Context:** PLAN.md: Phase 2, Step 2.1
    - **Action:**
        1. In `src/journal_io/mod.rs`, remove the `error!()` macro calls from lines 322 and 329, ensuring only the `Err(...)` is returned.
    - **Done‑when:**
        1. The specified lines in `src/journal_io/mod.rs` no longer contain `error!()` calls related to lock acquisition errors.
        2. Lock acquisition errors are still correctly returned as `LockError`.
    - **Depends‑on:** none

- [x] **T003 · Refactor · P1: remove `error!()` logging from editor execution functions in `journal_io/mod.rs`**
    - **Context:** PLAN.md: Phase 2, Step 2.2
    - **Action:**
        1. In `src/journal_io/mod.rs`, identify and remove all `error!()` macro calls that occur immediately before returning `EditorError` variants.
    - **Done‑when:**
        1. All `error!()` calls related to `EditorError` in `src/journal_io/mod.rs` are removed from positions immediately before returning an error.
        2. Editor execution errors are still correctly returned as `EditorError` with their context.
    - **Depends‑on:** none

- [x] **T004 · Refactor · P1: enhance `Display` implementations for `AppError` and its sources**
    - **Context:** PLAN.md: Phase 3, Step 3.1; Phase 4, Step 4.2; Risk 2 Mitigation
    - **Action:**
        1. Review `Display` implementations for `AppError` and all its domain-specific variants (Config, Io, Journal, Editor, Lock errors).
        2. Ensure each `Display` implementation provides comprehensive, user-friendly, and actionable context: operation performed, resource involved, underlying failure, and resolution hints if applicable.
        3. Update implementations as necessary.
    - **Done‑when:**
        1. All relevant error types have `Display` implementations that produce clear, informative, and actionable messages suitable for top-level logging.
        2. Error messages include context as outlined in PLAN.md (Phase 4, Step 4.2).
    - **Depends‑on:** none

- [x] **T005 · Test · P1: add integration test for single error logging on lock failure**
    - **Context:** PLAN.md: Phase 4, Step 4.1
    - **Action:**
        1. Create an integration test `test_single_error_logging_for_lock_failure`.
        2. Simulate a scenario that triggers a `LockError` (e.g., `LockError::FileBusy`).
        3. Capture log output and assert that the `LockError` is logged exactly once at the application boundary.
    - **Done‑when:**
        1. The new integration test passes, confirming single error log entry for lock failures.
    - **Depends‑on:** [T002, T004]

- [x] **T006 · Test · P1: add integration test for single error logging on editor failure**
    - **Context:** PLAN.md: Phase 4, Step 4.1
    - **Action:**
        1. Create an integration test `test_single_error_logging_for_editor_failure`.
        2. Simulate a scenario that triggers an `EditorError` (e.g., editor command not found).
        3. Capture log output and assert that the `EditorError` is logged exactly once at the application boundary.
    - **Done‑when:**
        1. The new integration test passes, confirming single error log entry for editor failures.
    - **Depends‑on:** [T003, T004]

- [ ] **T007 · Test · P1: add integration test for `main()` error propagation and formatting**
    - **Context:** PLAN.md: Phase 4, Step 4.1
    - **Action:**
        1. Create an integration test `test_main_error_propagation`.
        2. Trigger various `AppError` conditions that propagate to `main()`.
        3. Assert that `main()` handles and formats the errors correctly via their `Display` trait for output.
    - **Done‑when:**
        1. The new integration test passes, confirming correct error handling and formatting by `main()`.
    - **Depends‑on:** [T001, T004]

- [ ] **T008 · Test · P2: add unit tests for error construction and `source()` chaining**
    - **Context:** PLAN.md: Testing Strategy - Unit Tests (Error Construction Tests, Error Chaining Tests)
    - **Action:**
        1. Write unit tests to verify that all `AppError` variants and their underlying domain-specific errors are constructed correctly with proper context.
        2. Verify that `source()` chains work as expected for these error types.
    - **Done‑when:**
        1. Unit tests cover error construction and `source()` chaining for all key error types.
    - **Depends‑on:** none

- [ ] **T009 · Test · P2: add unit tests for error `Display` implementations**
    - **Context:** PLAN.md: Testing Strategy - Unit Tests (Error Display Tests)
    - **Action:**
        1. Write unit tests for the `Display` implementations of `AppError` and its domain-specific variants.
        2. Assert that the generated error messages are clear, actionable, and contain the expected contextual information.
    - **Done‑when:**
        1. Unit tests cover the `Display` output for all key error types, verifying message content and format.
    - **Depends‑on:** [T004]

- [ ] **T010 · Test · P2: perform manual testing of specified error scenarios**
    - **Context:** PLAN.md: Testing Strategy - Manual Testing
    - **Action:**
        1. Manually test behavior with invalid `PONDER_DIR` and `PONDER_EDITOR`.
        2. Manually test behavior with simulated filesystem errors (e.g., permission denied, disk full).
        3. Manually test behavior with editor failures (e.g., missing editor, non-zero exit codes).
    - **Done‑when:**
        1. All manual test scenarios result in graceful error handling.
        2. Errors are logged exactly once with clear, actionable messages.
        3. No panics occur in recoverable error paths.
    - **Verification:**
        1. Document the outcome of each manual test scenario, including observed error messages and application behavior.
    - **Depends‑on:** [T001, T002, T003, T004, T005, T006, T007]

- [ ] **T011 · Chore · P2: implement pre-commit hook to detect `unwrap`/`expect` in production code**
    - **Context:** PLAN.md: Risk Analysis & Mitigation - Risk 4 Mitigation
    - **Action:**
        1. Configure a pre-commit hook (e.g., using shell script, `grep`, or a dedicated tool) to scan the `src/` directory for `.unwrap()` and `.expect()` calls.
        2. Ensure the hook fails the commit if such calls are detected.
    - **Done‑when:**
        1. A pre-commit hook is active and successfully prevents commits containing `unwrap()` or `expect()` in `src/`.
    - **Verification:**
        1. Attempt to commit a file in `src/` containing a deliberate `unwrap()` call; confirm the commit is blocked by the hook.
    - **Depends‑on:** none

- [ ] **T012 · Chore · P2: ensure `cargo clippy --all-targets -- -D warnings` passes**
    - **Context:** PLAN.md: Quality Gates
    - **Action:**
        1. After all other relevant code changes are complete, run `cargo clippy --all-targets -- -D warnings`.
        2. Address any lints or warnings reported by Clippy until the command passes cleanly.
    - **Done‑when:**
        1. `cargo clippy --all-targets -- -D warnings` executes without emitting any warnings or errors.
    - **Depends‑on:** [T001, T002, T003, T004]