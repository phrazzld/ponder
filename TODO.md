# Todo

## Journal Logic Implementation

- [x] **T001 · Refactor · P0: Create `src/journal_logic.rs` and relocate `DateSpecifier`**
    - **Context:** PLAN.md, Detailed Build Steps, Step 1; Architecture Blueprint
    - **Action:**
        1. Create the new file `src/journal_logic.rs`.
        2. Move the `DateSpecifier` enum and its `impl` block (including `from_args`, `get_dates`) from `src/journal/mod.rs` to `src/journal_logic.rs`.
        3. Update all necessary imports related to `DateSpecifier`.
    - **Done‑when:**
        1. `src/journal_logic.rs` exists and contains the `DateSpecifier` code.
        2. The project compiles successfully after the move.
    - **Verification:**
        1. `cargo check` passes.
        2. Unit tests for date parsing (if any) run and pass.
    - **Depends‑on:** none

- [x] **T002 · Feature · P0: Implement `open_journal_entries` function signature**
    - **Context:** PLAN.md, Detailed Build Steps, Step 2; Architecture Blueprint (Public Interfaces)
    - **Action:**
        1. Define the public function signature `pub fn open_journal_entries(config: &Config, date_spec: &DateSpecifier) -> AppResult<()>` in `src/journal_logic.rs`.
        2. Implement it as a stub (returning `Ok(())` or `unimplemented!()`) for now.
    - **Done‑when:**
        1. The function signature is present in `src/journal_logic.rs` and the project compiles.
    - **Verification:**
        1. `cargo check` passes.
    - **Depends‑on:** [T001]

- [x] **T003 · Feature · P1: Implement `ensure_journal_directory_exists`**
    - **Context:** PLAN.md, Detailed Build Steps, Step 5; Architecture Blueprint (Public Interfaces)
    - **Action:**
        1. Implement `pub fn ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()>` in `src/journal_logic.rs`.
        2. Use `std::fs::create_dir_all` for the implementation.
        3. Map I/O errors to `AppError::Io`.
    - **Done‑when:**
        1. Function correctly creates the journal directory if it doesn't exist.
        2. Errors are correctly propagated as `AppError::Io`.
    - **Verification:**
        1. Manual: Run the application with a non-existent journal directory path to confirm it's created.
    - **Depends‑on:** [T001]

- [x] **T004 · Feature · P1: Implement filesystem helper functions**
    - **Context:** PLAN.md, Detailed Build Steps, Step 3; Architecture Blueprint (Private helpers)
    - **Action:**
        1. Implement the private helper function `fn get_entry_path_for_date(journal_dir: &Path, date: NaiveDate) -> PathBuf`.
        2. Implement `fn file_exists(path: &Path) -> bool`.
        3. Implement `fn create_or_open_entry_file(path: &Path) -> AppResult<std::fs::File>`.
        4. Implement `fn read_file_content(path: &Path) -> AppResult<String>`.
        5. Implement `fn append_to_file(file: &mut std::fs::File, content: &str) -> AppResult<()>`.
        6. Map I/O errors to `AppError::Io`.
    - **Done‑when:**
        1. All filesystem operations are implemented without using traits.
        2. Functions map to the equivalent functionality from the old `FileSystemIO` implementation.
    - **Verification:**
        1. Unit tests for date-to-path and file operations pass.
    - **Depends‑on:** [T001]

- [x] **T005 · Feature · P1: Implement editor launch logic**
    - **Context:** PLAN.md, Detailed Build Steps, Step 4; Architecture Blueprint (Private helpers)
    - **Action:**
        1. Implement the private helper function `fn launch_editor(editor_cmd: &str, files_to_open: &[PathBuf]) -> AppResult<()>` using `std::process::Command`.
        2. Map command execution errors to `AppError::Editor`.
    - **Done‑when:**
        1. Editor launching works as expected, similar to the original `SystemEditor` implementation.
        2. Error handling properly maps command execution errors to `AppError::Editor`.
    - **Verification:**
        1. Integration test or manual run verifies correct editor launch.
    - **Depends‑on:** [T001]

- [x] **T006 · Feature · P1: Implement date header append logic**
    - **Context:** PLAN.md, Detailed Build Steps, Step 5; Architecture Blueprint (Private helpers)
    - **Action:**
        1. Implement the private helper function `fn append_date_header_if_needed(path: &Path) -> AppResult<()>`.
        2. Logic should check if the file is new/empty and append the date-time header.
        3. Map I/O errors to `AppError::Io`.
    - **Done‑when:**
        1. Function correctly appends header only to new/empty files.
    - **Verification:**
        1. Filesystem test confirms header presence in new files.
    - **Depends‑on:** [T004]

- [x] **T007 · Feature · P0: Complete `open_journal_entries` implementation**
    - **Context:** PLAN.md, Detailed Build Steps, Step 2-5
    - **Action:**
        1. Complete the implementation of `open_journal_entries` function to replicate the logic from `JournalService::open_entries`.
        2. Use the helper functions implemented in T004, T005, and T006.
        3. Handle different `DateSpecifier` cases (Today, Retro, Reminisce, Specific).
    - **Done‑when:**
        1. Function is fully implemented and orchestrates all operations correctly.
    - **Verification:**
        1. Integration test passes for each date specifier type.
    - **Depends‑on:** [T002, T004, T005, T006]

## Main Refactoring

- [x] **T008 · Refactor · P0: Refactor `main.rs` to use new journal_logic API**
    - **Context:** PLAN.md, Detailed Build Steps, Step 6
    - **Action:**
        1. Remove instantiation and usage of `JournalService`, `FileSystemIO`, and `SystemEditor`.
        2. Update imports to use the new `journal_logic` module.
        3. Call `journal_logic::ensure_journal_directory_exists(&config.journal_dir)?` and `journal_logic::open_journal_entries(&config, &date_specifier)?` directly.
        4. Adjust error handling if needed.
    - **Done‑when:**
        1. `main.rs` uses the new `journal_logic` module directly.
        2. No trait objects or dependency injection is used.
    - **Verification:**
        1. All CLI flows compile and basic smoke testing works.
    - **Depends‑on:** [T003, T007]

## Codebase Cleanup

- [x] **T009 · Refactor · P0: Remove obsolete trait abstractions and related code**
    - **Context:** PLAN.md, Detailed Build Steps, Step 7
    - **Action:**
        1. Delete the `JournalIO` trait definition and `MockJournalIO` implementation.
        2. Delete the `Editor` trait definition and `MockEditor` implementation.
        3. Delete the `JournalService` struct and its implementation.
        4. Delete files: `src/journal/io/mod.rs` (and any `tests.rs` within), `src/editor.rs`.
        5. Review `src/journal/mod.rs` and delete or update as appropriate.
    - **Done‑when:**
        1. No trait or mock code remains in codebase.
        2. No references to deleted code.
    - **Verification:**
        1. `cargo check` and `cargo build` pass.
        2. No trait or struct is findable by search.
    - **Depends‑on:** [T008]

- [x] **T010 · Refactor · P1: Update `src/lib.rs` to reflect new module structure**
    - **Context:** PLAN.md, Detailed Build Steps, Step 8
    - **Action:**
        1. Remove `pub mod journal;` and `pub mod editor;` if they are no longer needed.
        2. Add `pub mod journal_logic;`.
        3. Adjust re-exports as necessary (e.g., `pub use journal_logic::DateSpecifier;`).
        4. Ensure public API remains consistent for external users.
    - **Done‑when:**
        1. Library root reflects the new module layout.
        2. Public API is consistent.
    - **Verification:**
        1. Library compiles and all consumers use the updated API.
    - **Depends‑on:** [T009]

## Testing

- [x] **T011 · Test · P0: Remove obsolete unit tests relying on trait mocks**
    - **Context:** PLAN.md, Detailed Build Steps, Step 9 (first bullet)
    - **Action:**
        1. Delete unit tests in `src/journal/io/tests.rs` and any in `src/journal/tests.rs` that depend on mocks.
        2. Remove `#[cfg(test)]` helpers on `JournalService`.
    - **Done‑when:**
        1. No tests depend on now-deleted trait-based code.
    - **Verification:**
        1. `cargo test` passes (excluding integration tests for now).
    - **Depends‑on:** [T009]

- [x] **T012 · Test · P0: Review and verify integration tests**
    - **Context:** PLAN.md, Detailed Build Steps, Step 9 (second/third bullets)
    - **Action:**
        1. Review `tests/cli_tests.rs`, `tests/journal_integration_tests.rs` for compatibility.
        2. Update if necessary for new module locations or function signatures.
        3. Run all integration tests.
        4. Ensure integration tests correctly use `tempfile` for isolated FS testing and can configure the `EDITOR` environment variable.
    - **Done‑when:**
        1. Integration tests run and pass.
        2. Tests interact with `journal_logic` through the binary as intended.
    - **Verification:**
        1. All integration tests pass.
    - **Depends‑on:** [T010]

## Code Quality

- [x] **T013 · Chore · P1: Code formatting and linting**
    - **Context:** PLAN.md, Detailed Build Steps, Step 10
    - **Action:**
        1. Run `cargo fmt` on the codebase.
        2. Run `cargo clippy -- -D warnings`.
        3. Remove dead code, unused imports, and outdated comments referencing old abstractions.
    - **Done‑when:**
        1. Code is formatted, linted, and free of warnings.
        2. No references to removed code remain in comments.
    - **Verification:**
        1. Formatting and lint checks pass.
    - **Depends‑on:** [T012]

## Documentation

- [x] **T014 · Chore · P2: Update documentation for new architecture**
    - **Context:** PLAN.md, Documentation section
    - **Action:**
        1. Revise `README.md` to remove references to trait-based architecture and highlight the simpler direct approach.
        2. Update Rustdoc comments for all public functions/types in `journal_logic.rs`, `config.rs`, `cli.rs`, and `main.rs` if signatures or behavior changed.
        3. Add/adjust module-level docs for `journal_logic.rs`.
    - **Done‑when:**
        1. Documentation reflects the new structure and is accurate.
    - **Verification:**
        1. `cargo doc` builds without warnings.
        2. README.md does not mention old abstractions.
    - **Depends‑on:** [T013]

## Clarifications & Assumptions

- [ ] **Issue: Should DateSpecifier remain a public type or become an internal helper?**
    - **Context:** PLAN.md, Architecture Blueprint and Build Step 1
    - **Blocking?:** no

- [ ] **Issue: Confirm if any code outside main.rs consumes JournalService directly**
    - **Context:** PLAN.md, Build Steps 6 and 7
    - **Blocking?:** no

- [ ] **Issue: Is there a need to merge journal_logic.rs into journal.rs and delete journal_logic.rs?**
    - **Context:** PLAN.md, Build Step 7 (file structure option)
    - **Blocking?:** no

- [ ] **Issue: Clarify if any additional integration tests are needed for new edge-cases**
    - **Context:** PLAN.md, Testing Strategy
    - **Blocking?:** no

- [ ] **Issue: Handling of retro/reminisce entries not found edge case**
    - **Context:** PLAN.md, Error & Edge-Case Strategy
    - **Blocking?:** no - The plan states it's "likely by creating new empty files as per current logic".