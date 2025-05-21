# T017-plan: Add File Locking for Concurrent Access

This plan outlines the implementation approach for adding file locking to prevent data corruption from simultaneous writes in Ponder.

## Implementation Approach

We'll use the `fs2` crate to implement OS-level advisory file locking, which provides a cross-platform abstraction over native file locking mechanisms (`flock` on Unix, `LockFileEx` on Windows). This approach aligns with Ponder's design philosophy of simplicity and direct function calls without unnecessary abstractions.

### Key design decisions:

1. **Locking Strategy:** Use exclusive advisory locks on journal files before launching the editor, maintaining the lock until the editor exits.
2. **Error Handling:** Add a new `LockError` enum to represent different lock failure scenarios, and incorporate it into the existing error infrastructure.
3. **Scope:** Focus changes in the `journal_io` module to respect the project's separation of concerns.
4. **Granularity:** Lock all journal files being opened for a session, releasing locks only after the editor process exits.
5. **Testing:** Create integration tests that simulate concurrent access to verify locking behavior.

## Implementation Steps

1. **Add `fs2` Dependency to Cargo.toml**
   ```toml
   [dependencies]
   # ... other dependencies
   fs2 = "0.4"
   ```

2. **Enhance Error Types in `src/errors/mod.rs`**
   - Add a new `LockError` enum with variants for different locking failures.
   - Add a new `Lock` variant to the `AppError` enum to incorporate lock errors.

3. **Refactor Journal I/O in `src/journal_io/mod.rs`**
   - Create a new `edit_journal_entries` function that handles initializing, locking, and opening files in the editor.
   - Maintain the existing file structure so initialization is still separated from opening.
   - Implement file locking logic using `fs2::FileExt` traits.
   - Handle lock acquisition failures with appropriate error propagation.

4. **Update Main Application Flow in `src/main.rs`**
   - Update the main function to call the new `edit_journal_entries` function instead of `open_journal_entries`.

5. **Add Integration Tests**
   - Create a new integration test in `tests/locking_tests.rs` that verifies:
     - A process can acquire a lock on a journal file.
     - A second process cannot access the same file while it's locked.
     - After the first process completes, the second process can acquire the lock.
   - Test various locking scenarios with multiple files (Today, Retro, Reminisce).

6. **Update Documentation**
   - Add file locking information to comments in the code.
   - Update README.md to mention the concurrency protection.

## Verification Criteria

- All tests pass (`cargo test`).
- Code is properly formatted (`cargo fmt --check`).
- Linting passes without warnings (`cargo clippy --all-targets -- -D warnings`).
- Manual testing confirms:
  - Multiple instances cannot edit the same journal file concurrently.
  - Lock errors are clearly communicated to the user.
  - Locks are properly released when the editor exits.

## Edge Cases to Handle

1. **Lock Acquisition Failures**
   - Handle when a file is already locked by another process.
   - Provide clear error messages to the user.

2. **Multiple Files in a Session**
   - Lock all files being opened in a single editor session.
   - Ensure proper cleanup if locking any file fails.

3. **Crash Recovery**
   - Leverage OS-level advisory locks which are automatically released if a process crashes.

4. **Platform Compatibility**
   - Use `fs2` for cross-platform compatibility between Unix and Windows.