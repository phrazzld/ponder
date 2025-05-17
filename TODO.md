# Todo

## CI Fixes (PR #5)
- [x] **T015 · Bugfix · P0: update deprecated `ensure_journal_dir` call in `config` tests**
    - **Context:** PLAN.md - Resolution Steps / Issue 1: Use of deprecated method `config::Config::ensure_journal_dir`
    - **Action:**
        1. In the test module of `src/config/mod.rs`, ensure `use crate::journal_logic;` is present.
        2. Around line 475, replace `config.ensure_journal_dir().unwrap();` with `journal_logic::ensure_journal_directory_exists(&config.journal_dir).unwrap();`.
    - **Done‑when:**
        1. `cargo clippy --all-targets -- -D warnings` no longer reports the deprecation warning for `ensure_journal_dir` in `src/config/mod.rs`.
        2. Tests in `src/config/mod.rs` (e.g., `cargo test --package ponder --test config`) pass.
        3. The CI `clippy` check for PR #5 passes related to this issue.
    - **Verification:**
        1. Locally run `cargo clippy --all-targets -- -D warnings` to confirm the specific error is resolved.
        2. Locally run `cargo test --package ponder --test config` to confirm tests pass.
    - **Depends‑on:** none

- [x] **T016 · Refactor · P0: apply clippy suggestion for `io::Error` construction in `errors` tests**
    - **Context:** PLAN.md - Resolution Steps / Issue 2: Clippy `io::Error` suggestion for simpler construction
    - **Action:**
        1. In `src/errors.rs`, around line 145, replace `Err(io::Error::new(io::ErrorKind::Other, "test error"));` with `Err(io::Error::other("test error"));`.
    - **Done‑when:**
        1. `cargo clippy --all-targets -- -D warnings` no longer reports the suggestion for `io::Error` construction in `src/errors.rs`.
        2. Tests in `src/errors.rs` (e.g., `cargo test --package ponder --test errors`) pass.
        3. The CI `clippy` check for PR #5 passes related to this issue.
    - **Verification:**
        1. Locally run `cargo clippy --all-targets -- -D warnings` to confirm the specific clippy suggestion is resolved.
        2. Locally run `cargo test --package ponder --test errors` to confirm tests pass.
    - **Depends‑on:** none

## Developer Workflow & Quality Gates
- [x] **T017 · Chore · P1: implement pre-commit hooks for `cargo fmt` and `clippy`**
    - **Context:** PLAN.md - Prevention Measures / 1. Enforce Mandatory Local Linting via Pre-commit Hooks
    - **Action:**
        1. Configure pre-commit hooks to execute `cargo fmt --check`.
        2. Configure pre-commit hooks to execute `cargo clippy --all-targets -- -D warnings`.
    - **Done‑when:**
        1. Pre-commit hooks are configured and functional within the project repository.
        2. Attempting to commit code that fails `cargo fmt --check` is blocked by the hooks.
        3. Attempting to commit code that fails `cargo clippy --all-targets -- -D warnings` is blocked by the hooks.
    - **Verification:**
        1. Intentionally introduce a formatting issue; verify `git commit` is blocked by `cargo fmt --check`.
        2. Intentionally introduce a clippy warning; verify `git commit` is blocked by `cargo clippy`.
        3. Correct issues and confirm a successful commit with hooks enabled.
    - **Depends‑on:** none

- [x] **T018 · Chore · P1: document pre-commit hook setup and usage in `CONTRIBUTING.md`**
    - **Context:** PLAN.md - Prevention Measures / 1. Enforce Mandatory Local Linting via Pre-commit Hooks
    - **Action:**
        1. Add clear, step-by-step instructions to `CONTRIBUTING.md` explaining how developers can install and use the project's pre-commit hooks.
    - **Done‑when:**
        1. `CONTRIBUTING.md` contains comprehensive documentation for setting up and using the pre-commit hooks.
    - **Verification:**
        1. A developer new to the project can follow the `CONTRIBUTING.md` instructions to successfully set up and run pre-commit hooks.
    - **Depends‑on:** [T017]

- [x] **T019 · Chore · P2: verify CI workflow enforces strict `clippy` settings**
    - **Context:** PLAN.md - Prevention Measures / 2. Maintain Strict CI/CD Quality Gates
    - **Action:**
        1. Review the CI workflow configuration files (e.g., GitHub Actions YAML).
        2. Confirm that `clippy` is executed with strict settings (e.g., `-D warnings` or equivalent) and that identified lints cause the CI build to fail.
    - **Done‑when:**
        1. The CI configuration is verified to enforce strict `clippy` checks that fail the build on any lint.
    - **Verification:**
        1. Inspect CI logs from PR #5 (Run ID: `15069522992`) or a similar recent failing build to confirm `clippy` strictness.
    - **Depends‑on:** none

- [x] **T020 · Chore · P2: update PR template with checklist for API deprecation self-review**
    - **Context:** PLAN.md - Prevention Measures / 3. Promote Thorough Self-Review During Refactoring
    - **Action:**
        1. Modify the project's Pull Request template (e.g., `.github/PULL_REQUEST_TEMPLATE.md`).
        2. Add a checklist item: "For PRs involving API deprecations or significant API changes: All usages of the old API (including in tests) have been identified and updated."
    - **Done‑when:**
        1. The Pull Request template includes the new checklist item for refactoring self-review.
    - **Verification:**
        1. Create a new draft Pull Request and confirm the updated template content is present.
    - **Depends‑on:** none

- [x] **T021 · Chore · P3: document IDE setup for `rustfmt` and `clippy` integration**
    - **Context:** PLAN.md - Prevention Measures / 4. Developer Education and Tooling Integration
    - **Action:**
        1. Provide guidance and example configurations in `CONTRIBUTING.md` or a dedicated `DEVELOPMENT_SETUP.md` file for integrating `rustfmt` and `clippy` (with project-aligned settings) into common IDEs/editors (e.g., VS Code with `rust-analyzer`).
    - **Done‑when:**
        1. Project documentation includes clear instructions for IDE integration of `rustfmt` and `clippy`.
    - **Verification:**
        1. A developer can follow the documentation to set up their IDE for real-time feedback.
    - **Depends‑on:** none

- [x] **T022 · Chore · P2: document test code quality standards in `CONTRIBUTING.md`**
    - **Context:** PLAN.md - Prevention Measures / 5. Documentation of Standards for Test Code
    - **Action:**
        1. Update `CONTRIBUTING.md` to explicitly state that test code is held to the same quality standards (linting, style, use of current APIs) as production code.
    - **Done‑when:**
        1. `CONTRIBUTING.md` clearly articulates that test code must meet production-level quality standards.
    - **Verification:**
        1. Review `CONTRIBUTING.md` to confirm the addition and clarity of the test code standards.
    - **Depends‑on:** none

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

## CI Resolution
- [x] **T001 · chore · P0: update ci workflow to set required environment variables**
    - **Context:** CI Resolution Plan, Action Items: 1. Fix CI Environment Configuration; Root Causes: 1. Missing PONDER_EDITOR Environment Variable
    - **Action:**
        1. Modify `.github/workflows/ci.yml` as per the plan:
           ```yaml
           - name: Run tests
             run: cargo test --verbose
             env:
               PONDER_EDITOR: echo
               PONDER_DIR: /tmp/ponder_ci_tests
               RUST_BACKTRACE: 1
           ```
    - **Done‑when:**
        1. The `.github/workflows/ci.yml` file is updated with the specified environment variables.
        2. The CI pipeline successfully passes the test step, utilizing these new environment variables.
        3. CI logs confirm that tests run with `PONDER_EDITOR=echo`, `PONDER_DIR=/tmp/ponder_ci_tests`, and `RUST_BACKTRACE=1`.
    - **Verification:**
        1. Push the changes to `.github/workflows/ci.yml` and observe a CI run.
        2. Check the CI logs for the "Run tests" step to confirm the environment variables are correctly set and utilized.
        3. Confirm that CI test failures previously attributed to missing `PONDER_EDITOR` are resolved or change nature.
    - **Depends‑on:** none

- [x] **T002 · refactor · P0: audit and refactor error handling in `journal_logic.rs`**
    - **Context:** CI Resolution Plan, Action Items: 2. Audit Error Handling in journal_logic.rs; Root Causes: 2. Unhandled Errors in OS Interactions
    - **Action:**
        1. In `src/journal_logic.rs`, review key functions (`launch_editor`, `create_or_open_entry_file`, `read_file_content`, `append_to_file`) for `.unwrap()` and `.expect()` calls related to OS interactions.
        2. Replace these calls using the `?` operator or `map_err()` to convert OS errors to `AppError`, ensuring robust error propagation.
    - **Done‑when:**
        1. All identified `.unwrap()` and `.expect()` calls in the specified functions within `src/journal_logic.rs` are replaced with proper error handling.
        2. The code compiles successfully and relevant unit tests pass.
        3. CI tests pass without panics originating from `journal_logic.rs` due to unhandled OS errors.
    - **Verification:**
        1. Manually review `src/journal_logic.rs` to confirm no `.unwrap()` or `.expect()` calls remain on OS interactions in the targeted functions.
        2. Run `cargo test --verbose` locally, ensuring tests covering these functions pass and errors are handled gracefully (e.g., by simulating OS error conditions if possible in unit tests).
        3. Observe CI logs after merging T001 and this change to ensure no panics.
    - **Depends‑on:** none

- [x] **T003 · test · P1: enhance integration test robustness with explicit environment variables**
    - **Context:** CI Resolution Plan, Action Items: 3. Enhance Integration Test Robustness; Root Causes: 3. Test Environment Assumptions
    - **Action:**
        1. Audit all integration tests in `tests/*.rs` that use `assert_cmd`.
        2. Modify these tests to explicitly set `PONDER_EDITOR` and `PONDER_DIR` using `cmd.env()`:
           ```rust
           cmd.env("PONDER_EDITOR", "echo");
           cmd.env("PONDER_DIR", temp_dir.path()); // Ensure temp_dir is properly set up
           ```
    - **Done‑when:**
        1. All relevant integration tests in `tests/*.rs` explicitly set `PONDER_EDITOR` and `PONDER_DIR`.
        2. All integration tests pass both locally (using `PONDER_EDITOR=echo cargo test`) and in the CI environment.
    - **Verification:**
        1. Run integration tests locally with minimal/conflicting global environment variables (e.g., `PONDER_EDITOR=some_failing_command cargo test`) to ensure tests use their explicitly set variables.
        2. Confirm all tests pass in CI after this change and prerequisite fixes.
    - **Depends‑on:** [T001, T002]

- [ ] **T004 · chore · P2: verify complete removal of old trait-based abstractions**
    - **Context:** CI Resolution Plan, Action Items: 4. Verify Complete Code Cleanup
    - **Action:**
        1. Conduct a thorough search of the codebase for any remaining references to old trait-based abstractions.
        2. Identify and remove any lingering mock implementations or `#[cfg(test)] pub` items that were specifically part of the old trait-based mocking system and are no longer needed.
    - **Done‑when:**
        1. The codebase is confirmed to be free of the specified old trait-based abstractions, their associated mocks, and unnecessary test-specific public modifiers related to them.
        2. The full test suite (`cargo test --all-targets --all-features`) passes locally and in CI, indicating no regressions from this cleanup.
    - **Verification:**
        1. Perform manual code review and use search tools (e.g., `grep`, IDE search) for old trait names, mock module names, and specific `#[cfg(test)] pub` patterns related to the removed abstractions.
    - **Depends‑on:** [T002, T003]

