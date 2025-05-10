# Todo

## Build and CI Configuration
- [x] **T001 · Chore · P0: remove `--cap-lints=warn` from cargo config**
    - **Context:** REMEDIATION_PLAN.md - Section 3.1, Step 1
    - **Action:**
        1. Open `.cargo/config.toml`.
        2. Remove the line `rustflags = ["--cap-lints=warn"]`.
    - **Done‑when:**
        1. The `rustflags = ["--cap-lints=warn"]` line is removed from `.cargo/config.toml`.
        2. Lint warnings are no longer suppressed by this flag.
    - **Verification:**
        1. Introduce a temporary lint warning (e.g., an unused variable if not denied by clippy.toml yet) and confirm `cargo clippy` reports it as an error if other stricter settings are active, or as a warning if not.
    - **Depends‑on:** none
- [x] **T002 · Chore · P0: remove `dead_code = "allow"` clippy lint from cargo config**
    - **Context:** REMEDIATION_PLAN.md - Section 3.1, Step 2
    - **Action:**
        1. Open `.cargo/config.toml`.
        2. Remove the `[lints.clippy]` section specifically containing `dead_code = "allow"`.
    - **Done‑when:**
        1. The `dead_code = "allow"` override for clippy is removed from `.cargo/config.toml`.
        2. `dead_code` lint behavior is solely determined by `.cargo/clippy.toml`.
    - **Depends‑on:** [T001]
- [x] **T003 · Chore · P0: verify and update `.cargo/clippy.toml` for strict lint enforcement**
    - **Context:** REMEDIATION_PLAN.md - Section 3.1, Step 3
    - **Action:**
        1. Open `.cargo/clippy.toml`.
        2. Ensure it includes `deny = ["clippy::all", "dead_code"]` and other desired strict lints.
        3. Add or modify entries as needed to enforce strict linting.
    - **Done‑when:**
        1. `.cargo/clippy.toml` correctly configures all clippy lints to `deny` as per project standards.
        2. `cargo clippy` locally reflects these strict settings (e.g., dead code causes an error).
    - **Verification:**
        1. Introduce temporary code that violates a denied lint (e.g., dead code) and confirm `cargo clippy` fails.
    - **Depends‑on:** [T001, T002]
- [x] **T004 · Chore · P0: enforce strict clippy checks in ci pipeline**
    - **Context:** REMEDIATION_PLAN.md - Section 3.1, Step 4
    - **Action:**
        1. Modify the CI pipeline configuration file (e.g., GitHub Actions workflow).
        2. Ensure the command `cargo clippy --all-targets -- -D warnings` is executed.
        3. Confirm the CI build fails if this command reports any warnings/errors.
    - **Done‑when:**
        1. CI pipeline configuration is updated to run strict clippy checks.
        2. A test Pull Request with a clippy violation (that should be an error) causes the CI build to fail at the clippy step.
    - **Verification:**
        1. Push a branch with a deliberate clippy violation and observe CI failure.
        2. Fix the violation and observe CI pass.
    - **Depends‑on:** [T003]

## API Path Handling
- [x] **T005 · Refactor · P0: update `editor` trait `open_files` signature to use `asref<path>`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 1
    - **Action:**
        1. Open `src/editor.rs`.
        2. Modify the `Editor::open_files` signature to use path types instead of strings.
    - **Done‑when:**
        1. The `Editor::open_files` trait method signature is updated in `src/editor.rs`.
        2. Code compiles (implementations will be updated in subsequent tickets).
    - **Depends‑on:** none
- [x] **T006 · Refactor · P0: update `systemeditor` `open_files` implementation for `asref<path>`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 1
    - **Action:**
        1. Open `src/editor.rs`.
        2. Update the `SystemEditor::open_files` implementation to match the new `Editor` trait signature, using `path.as_ref()` internally.
    - **Done‑when:**
        1. `SystemEditor::open_files` implementation is updated and compiles.
        2. Relevant unit tests for `SystemEditor` pass.
    - **Depends‑on:** [T005]
- [x] **T007 · Refactor · P0: update `mockeditor` `open_files` implementation for `asref<path>`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 1
    - **Action:**
        1. Open `src/editor.rs`.
        2. Update the `MockEditor::open_files` implementation to match the new `Editor` trait signature, potentially storing paths as `PathBuf`.
    - **Done‑when:**
        1. `MockEditor::open_files` implementation is updated and compiles.
        2. Relevant unit tests for `MockEditor` pass.
    - **Depends‑on:** [T005]
- [x] **T008 · Refactor · P0: change `filesystemio::journal_dir` field type to `pathbuf`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 2
    - **Action:**
        1. Open `src/journal/io/mod.rs`.
        2. Change the type of the `journal_dir` field in the `FileSystemIO` struct from `String` to `PathBuf`.
    - **Done‑when:**
        1. `FileSystemIO::journal_dir` field type is `PathBuf`.
        2. Code compiles (instantiations updated in next ticket).
    - **Depends‑on:** none
- [x] **T009 · Refactor · P0: update `filesystemio` instantiations for `pathbuf` `journal_dir`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 2
    - **Action:**
        1. Find all places where `FileSystemIO` is instantiated (e.g., `src/main.rs`, tests).
        2. Update instantiation code to provide a `PathBuf` for `journal_dir` (e.g., using `PathBuf::from(string_path)`).
    - **Done‑when:**
        1. All instantiations of `FileSystemIO` correctly use `PathBuf` for `journal_dir`.
        2. Code compiles.
    - **Depends‑on:** [T008]
- [x] **T010 · Refactor · P0: change `journalio` trait path generation methods to return `appresult<pathbuf>`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 2
    - **Action:**
        1. Open `src/journal/io/mod.rs`.
        2. Modify methods in the `JournalIO` trait that generate paths (e.g., `generate_path_for_date`) to return `AppResult<PathBuf>` instead of `AppResult<String>`.
    - **Done‑when:**
        1. Path generation method signatures in the `JournalIO` trait are updated.
        2. Code compiles (implementations updated in next ticket).
    - **Depends‑on:** none
- [x] **T011 · Refactor · P0: update `filesystemio` path generation method implementations to return `appresult<pathbuf>`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 2
    - **Action:**
        1. Open `src/journal/io/mod.rs`.
        2. Update implementations of path generation methods in `FileSystemIO` to match the `JournalIO` trait, returning `PathBuf`s.
    - **Done‑when:**
        1. `FileSystemIO` path generation methods are updated and compile.
        2. Relevant unit tests pass.
    - **Depends‑on:** [T008, T010]
- [x] **T012 · Refactor · P0: update `journalio` trait methods accepting paths to use `asref<path>` or `&path`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 2
    - **Action:**
        1. Open `src/journal/io/mod.rs`.
        2. Modify methods in the `JournalIO` trait that accept path strings (e.g., `file_exists`, `create_or_open_file`, `read_file_content`) to accept `impl AsRef<Path>` or `&Path`.
    - **Done‑when:**
        1. Method signatures in `JournalIO` trait accepting paths are updated.
        2. Code compiles (implementations updated in next ticket).
    - **Depends‑on:** none
- [x] **T013 · Refactor · P0: update `filesystemio` method implementations accepting paths to use `asref<path>` or `&path`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 2
    - **Action:**
        1. Open `src/journal/io/mod.rs`.
        2. Update implementations of methods in `FileSystemIO` that accept paths to match the `JournalIO` trait, using `path.as_ref()`.
    - **Done‑when:**
        1. `FileSystemIO` methods accepting paths are updated and compile.
        2. Relevant unit tests pass.
    - **Depends‑on:** [T008, T012]
- [x] **T014 · Refactor · P0: update `journalservice` to use new path types from `editor` and `journalio`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 3
    - **Action:**
        1. Open `src/journal/mod.rs` (for `JournalService`).
        2. Update calls to `Editor` and `JournalIO` methods to pass `PathBuf` or `&Path` as required by their updated signatures.
    - **Done‑when:**
        1. `JournalService` correctly interacts with `Editor` and `JournalIO` using new path types.
        2. Code compiles and relevant `JournalService` tests pass.
    - **Depends‑on:** [T006, T007, T011, T013]
- [x] **T015 · Refactor · P0: update `main.rs` to use new path types when calling `filesystemio` and `journalservice`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.2, Step 3
    - **Action:**
        1. Open `src/main.rs`.
        2. Update calls to `FileSystemIO` (instantiation) and `JournalService` methods to pass/receive `PathBuf` or `&Path` as required.
    - **Done‑when:**
        1. `main.rs` correctly uses new path types for interacting with `FileSystemIO` and `JournalService`.
        2. Application compiles and CLI E2E tests related to file operations pass.
    - **Verification:**
        1. Manually test CLI commands that involve file path creation or opening.
    - **Depends‑on:** [T009, T014]

## Code Cleanup
- [x] **T016 · Refactor · P0: review `#[allow(dead_code)]` in `src/cli/mod.rs`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.3
    - **Action:**
        1. Open `src/cli/mod.rs`.
        2. For each `#[allow(dead_code)]` (e.g., `CliArgs::parse_date()`, `parse_args()`), determine if the code is used or genuinely unused.
        3. Document findings for action in T020.
    - **Done‑when:**
        1. All `#[allow(dead_code)]` instances in `src/cli/mod.rs` are reviewed and documented.
    - **Depends‑on:** [T003]
- [x] **T017 · Refactor · P0: review `#[allow(dead_code)]` in `src/config/mod.rs`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.3
    - **Action:**
        1. Open `src/config/mod.rs`.
        2. For `#[allow(dead_code)]` on `Config::new()`, determine if used or genuinely unused.
        3. Document findings for action in T020.
    - **Done‑when:**
        1. The `#[allow(dead_code)]` instance in `src/config/mod.rs` is reviewed and documented.
    - **Depends‑on:** [T003]
- [ ] **T018 · Refactor · P0: review `#[allow(dead_code)]` in `src/journal/mod.rs`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.3
    - **Action:**
        1. Open `src/journal/mod.rs`.
        2. For each `#[allow(dead_code)]` (in `DateSpecifier`, `JournalService`), determine if used or genuinely unused.
        3. Document findings for action in T020.
    - **Done‑when:**
        1. All `#[allow(dead_code)]` instances in `src/journal/mod.rs` are reviewed and documented.
    - **Depends‑on:** [T003]
- [ ] **T019 · Refactor · P0: review `#[allow(dead_code)]` in `src/journal/io/mod.rs`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.3
    - **Action:**
        1. Open `src/journal/io/mod.rs`.
        2. For `#[allow(dead_code)]` on `JournalIO::ensure_journal_dir()`, determine if used or genuinely unused.
        3. Document findings for action in T020.
    - **Done‑when:**
        1. The `#[allow(dead_code)]` instance in `src/journal/io/mod.rs` is reviewed and documented.
    - **Depends‑on:** [T003]
- [ ] **T020 · Refactor · P0: apply actions from `dead_code` review**
    - **Context:** REMEDIATION_PLAN.md - Section 3.3, Steps 2-3
    - **Action:**
        1. Based on findings from T016-T019, delete genuinely unused code.
        2. Remove `#[allow(dead_code)]` attributes for code that is confirmed to be used (e.g., by tests or as public API).
        3. Add `#[deprecated]` attributes where appropriate for planned future removal.
    - **Done‑when:**
        1. Unused code is deleted and `#[allow(dead_code)]` attributes are appropriately handled.
        2. `cargo clippy --all-targets -- -D warnings` passes without `dead_code` warnings related to the reviewed code (unless intentionally deprecated).
    - **Depends‑on:** [T016, T017, T018, T019]

## Test Infrastructure
- [ ] **T021 · Test · P1: refactor `filesystemio` tests to use actual struct with `tempfile`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.4, Step 1
    - **Action:**
        1. Open `src/journal/io/tests.rs`.
        2. Remove the duplicated `TestFileSystemIO` struct and its `impl JournalIO`.
        3. Update tests to instantiate and use the actual `FileSystemIO`, using `tempfile::tempdir()` for creating temporary journal directories.
    - **Done‑when:**
        1. `TestFileSystemIO` is removed from `src/journal/io/tests.rs`.
        2. All tests in `src/journal/io/tests.rs` pass using the actual `FileSystemIO` struct.
    - **Verification:**
        1. Verify all file operations (read, write, create) are correctly tested.
    - **Depends‑on:** [T009]
- [ ] **T022 · Test · P1: enhance `mockeditor` with configurable success/failure behavior**
    - **Context:** REMEDIATION_PLAN.md - Section 3.4, Step 2
    - **Action:**
        1. Open `src/editor.rs`.
        2. Add fields to `MockEditor` to control success/failure behavior (e.g., `should_fail: bool`, `failure_error: Option<AppError>`).
        3. Implement methods to set these fields.
        4. Update the `open_files` implementation to return errors based on these settings.
    - **Done‑when:**
        1. `MockEditor` has configurable success/failure behavior.
        2. Tests using `MockEditor` still pass.
    - **Verification:**
        1. Add tests that verify both success and error paths using the new configuration options.
    - **Depends‑on:** [T007]
- [ ] **T023 · Test · P1: enhance `mockjournalio` with configurable return values and error conditions**
    - **Context:** REMEDIATION_PLAN.md - Section 3.4, Step 2
    - **Action:**
        1. Create/Find the `MockJournalIO` implementation.
        2. Add fields or methods to control return values and error conditions for each method (may involve `HashMap`s or similar mechanisms).
        3. Update method implementations to use these configurations.
    - **Done‑when:**
        1. `MockJournalIO` has configurable success/failure behavior and return values.
        2. Tests using `MockJournalIO` still pass.
    - **Verification:**
        1. Add tests that verify both success and error paths using the new configuration options.
    - **Depends‑on:** [T011, T013]
- [ ] **T024 · Test · P1: update tests to use enhanced mocks for thorough coverage**
    - **Context:** REMEDIATION_PLAN.md - Section 3.4, Step 2
    - **Action:**
        1. Find tests that use `MockEditor` and `MockJournalIO`.
        2. Update tests to exercise both success and failure paths using the enhanced mocks.
        3. Add new tests as needed to cover error conditions.
    - **Done‑when:**
        1. Tests using mocks achieve better coverage of both success and failure paths.
        2. All tests pass.
    - **Depends‑on:** [T022, T023]

## Deprecated Code Removal
- [ ] **T025 · Refactor · P1: remove old `journal<t: journalio>` struct and implementation**
    - **Context:** REMEDIATION_PLAN.md - Section 3.5, Step 1
    - **Action:**
        1. Open `src/journal/mod.rs`.
        2. Remove the old `struct Journal<T: JournalIO>` and its `impl` block.
    - **Done‑when:**
        1. The old `Journal` struct is removed.
        2. The codebase compiles and all tests pass without this struct.
    - **Verification:**
        1. Check for any dependencies on the removed code.
    - **Depends‑on:** [T014, T015]
- [ ] **T026 · Refactor · P1: remove `fn parse_args()` backward-compatibility function**
    - **Context:** REMEDIATION_PLAN.md - Section 3.5, Step 2
    - **Action:**
        1. Open `src/cli/mod.rs`.
        2. Remove the `fn parse_args()` function.
    - **Done‑when:**
        1. The `parse_args()` function is removed.
        2. The codebase compiles and all tests pass without this function.
    - **Verification:**
        1. Check for any dependencies on the removed function.
    - **Depends‑on:** [T020]

## Logical Refinements
- [ ] **T027 · Refactor · P2: centralize date calculation logic in `datespecifier`**
    - **Context:** REMEDIATION_PLAN.md - Section 3.6, Step 1
    - **Action:**
        1. Open `src/journal/mod.rs` (for `DateSpecifier` and `JournalService`).
        2. Identify date calculation logic in `JournalService` (e.g., `get_reminisce_entries`) that could be moved to `DateSpecifier`.
        3. Move the logic to `DateSpecifier::get_dates()` or related methods.
        4. Update `JournalService` to use dates from `DateSpecifier`.
    - **Done‑when:**
        1. Date calculation logic is centralized in `DateSpecifier`.
        2. `JournalService` uses `DateSpecifier` for date calculations.
        3. The codebase compiles and all tests pass.
    - **Verification:**
        1. Manually test CLI commands using different date specifications (today, retro, reminisce) to ensure they still work correctly.
    - **Depends‑on:** [T015]
- [ ] **T028 · Refactor · P2: replace magic numbers in reminisce logic with named constants**
    - **Context:** REMEDIATION_PLAN.md - Section 3.6, Step 2
    - **Action:**
        1. Open `src/journal/mod.rs` or other relevant files with reminisce logic.
        2. Find the magic number `100` (years) in reminisce logic.
        3. Replace it with a named constant: `const MAX_REMINISCE_YEARS_AGO: u32 = 100;`.
        4. Replace any other magic numbers with meaningful constants.
    - **Done‑when:**
        1. Magic numbers in reminisce logic are replaced with named constants.
        2. The codebase compiles and all tests pass.
    - **Depends‑on:** [T027]

## Final Verification
- [ ] **T029 · Test · P1: run comprehensive test suite to verify all changes**
    - **Context:** REMEDIATION_PLAN.md - Section 6
    - **Action:**
        1. Run all unit tests: `cargo test --all`.
        2. Run all integration tests: `cargo test --test '*'`.
        3. Run end-to-end tests: `cargo run --bin assert_cmd_test` (adjust as needed for your project).
    - **Done‑when:**
        1. All tests pass.
    - **Verification:**
        1. Manually test key CLI functionality to ensure no regressions.
    - **Depends‑on:** All tickets needed for a complete implementation
- [ ] **T030 · Chore · P1: run linting and formatting checks**
    - **Context:** REMEDIATION_PLAN.md - Section 6
    - **Action:**
        1. Run `cargo clippy --all-targets -- -D warnings`.
        2. Run `cargo fmt --check`.
    - **Done‑when:**
        1. No linting errors or warnings are reported.
        2. Code formatting is correct.
    - **Depends‑on:** All tickets needed for a complete implementation

## Implementation Notes

1. The tickets are organized into logical groups with dependencies to ensure the right sequence of changes.
2. High-priority issues (P0) from the remediation plan are addressed first, followed by medium (P1) and low (P2) priority items.
3. Task dependencies reflect the logical relationships noted in section 4 of the remediation plan.
4. Final verification tasks ensure the entire codebase is properly tested and lint-checked after all changes.
5. The T001-T004 tickets addressing linting configuration should ideally be completed first, as they affect how clippy identifies issues during subsequent development.
6. Path handling changes (T005-T015) form the core of the refactoring and require careful coordination to ensure all API boundaries are correctly updated.
7. The `#[allow(dead_code)]` review (T016-T020) is structured as a review phase followed by implementation, to ensure a systematic approach to code cleanup.

**Note:** The implementation timeline of 5-8 working days is reasonable given the ticket breakdown. Tasks should be completed by priority, with P0 (high priority) tasks completed first.