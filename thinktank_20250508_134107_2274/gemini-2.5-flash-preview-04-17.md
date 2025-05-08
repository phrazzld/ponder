```markdown
# Todo

## Project Setup & Core

- [ ] **T001 · Chore · P1:** Setup basic Rust project structure and dependencies
    - **Context:** Detailed Build Steps, Step 1
    - **Action:**
        1. Initialize a new Rust binary project (`cargo new ponder --bin`).
        2. Update `Cargo.toml` to specify `edition = "2021"`.
        3. Add baseline dependencies: `clap` (with `derive`), `chrono`, `thiserror`, `log`, `env_logger`, `shellexpand`.
    - **Done‑when:**
        1. Project directory structure is created.
        2. `Cargo.toml` is updated with edition and initial dependencies.
        3. `cargo build` runs without errors.
    - **Depends‑on:** none

- [ ] **T002 · Refactor · P1:** Define central application error type
    - **Context:** Architecture Blueprint (Error & Edge-Case Strategy), Detailed Build Steps (Step 2)
    - **Action:**
        1. Create `src/errors.rs`.
        2. Define `AppError` enum using `thiserror` with variants for `ConfigError`, `Io`, `Journal`, `Editor`, and `CliArgs`.
        3. Define `AppResult<T>` type alias.
        4. Add basic error types (`ConfigError`, etc.) as simple structs/enums for now if not covered by `#[from]`.
    - **Done‑when:**
        1. `src/errors.rs` exists and defines `AppError` and `AppResult`.
        2. `cargo check` passes.
    - **Depends‑on:** [T001]

## Configuration

- [ ] **T003 · Refactor · P2:** Define Config struct and basic loading
    - **Context:** Architecture Blueprint (Modules/Packages, Public Interfaces), Detailed Build Steps (Step 3)
    - **Action:**
        1. Create `src/config.rs`.
        2. Define `Config` struct with `journal_dir` (PathBuf) and `editor_cmd` (String) fields.
        3. Implement a basic `Config::load()` function signature returning `AppResult<Config>`.
    - **Done‑when:**
        1. `src/config.rs` exists with `Config` struct and `load` function signature.
        2. `cargo check` passes.
    - **Depends‑on:** [T002]

- [ ] **T004 · Refactor · P2:** Implement Config loading logic
    - **Context:** Detailed Build Steps (Step 3)
    - **Action:**
        1. Implement `Config::load()` logic in `src/config.rs`.
        2. Read `PONDER_DIR` environment variable, falling back to a default path (e.g., `$HOME/Documents/ponder`). Use `shellexpand`.
        3. Read `EDITOR` environment variable, falling back to a default (e.g., `vim`).
        4. Handle potential errors during path expansion or environment variable reading, mapping them to `AppError::Config`.
    - **Done‑when:**
        1. `Config::load()` successfully reads and returns configuration based on environment variables and fallbacks.
        2. Unit tests verify `Config::load()` for different env var settings and fallbacks.
    - **Depends‑on:** [T003]

## Journal I/O Adapter

- [ ] **T005 · Refactor · P2:** Define JournalIO trait
    - **Context:** Architecture Blueprint (Modules/Packages, Public Interfaces), Detailed Build Steps (Step 4)
    - **Action:**
        1. Create `src/journal/mod.rs` and `src/journal/io.rs`.
        2. Define the `JournalIO` trait in `src/journal/io.rs` with methods: `ensure_journal_dir_exists`, `get_entry_path`, `read_entry`, `append_to_entry`, `entry_exists`, `create_file_if_not_exists`.
        3. Ensure methods use `&Path` or `PathBuf` and return `AppResult` where fallible.
    - **Done‑when:**
        1. `src/journal/io.rs` exists and defines the `JournalIO` trait.
        2. `cargo check` passes.
    - **Depends‑on:** [T002]

- [ ] **T006 · Refactor · P2:** Implement FilesystemJournalIO
    - **Context:** Architecture Blueprint (Modules/Packages), Detailed Build Steps (Step 4)
    - **Action:**
        1. Define `FilesystemJournalIO` struct in `src/journal/io.rs` (likely empty struct).
        2. Implement the `JournalIO` trait for `FilesystemJournalIO`, using `std::fs` and `std::path` for file operations.
        3. Map `std::io::Error` to `AppError::Io` using `?` or `map_err`.
    - **Done‑when:**
        1. `FilesystemJournalIO` struct is defined and implements the `JournalIO` trait.
        2. All trait methods have basic implementations using `std::fs`/`std::path`.
        3. `cargo check` passes.
    - **Depends‑on:** [T005]

- [ ] **T007 · Test · P2:** Add unit/integration tests for FilesystemJournalIO
    - **Context:** Testing Strategy (Integration Tests), Detailed Build Steps (Step 4)
    - **Action:**
        1. Add tests for `FilesystemJournalIO` in `src/journal/io.rs` (using `#[cfg(test)] mod tests`).
        2. Use `tempfile::tempdir()` to create isolated test directories.
        3. Test methods like `ensure_journal_dir_exists`, `create_file_if_not_exists`, `append_to_entry` against the temporary directory.
    - **Done‑when:**
        1. Tests for `FilesystemJournalIO` cover core file system interactions.
        2. All tests pass.
    - **Depends‑on:** [T006]

## Editor Adapter

- [ ] **T008 · Refactor · P2:** Define Editor trait
    - **Context:** Architecture Blueprint (Modules/Packages, Public Interfaces), Detailed Build Steps (Step 5)
    - **Action:**
        1. Create `src/editor.rs`.
        2. Define the `Editor` trait in `src/editor.rs` with the method `open_files(&self, editor_cmd: &str, paths: &[PathBuf]) -> AppResult<()>`.
    - **Done‑when:**
        1. `src/editor.rs` exists and defines the `Editor` trait.
        2. `cargo check` passes.
    - **Depends‑on:** [T002]

- [ ] **T009 · Refactor · P2:** Implement SystemEditor
    - **Context:** Architecture Blueprint (Modules/Packages), Detailed Build Steps (Step 5)
    - **Action:**
        1. Define `SystemEditor` struct in `src/editor.rs` (likely empty struct).
        2. Implement the `Editor` trait for `SystemEditor`, using `std::process::Command` to launch the editor process.
        3. Handle command execution errors, mapping them to `AppError::Editor`.
    - **Done‑when:**
        1. `SystemEditor` struct is defined and implements the `Editor` trait.
        2. The `open_files` method attempts to launch the specified editor command with the given file paths.
        3. `cargo check` passes.
    - **Depends‑on:** [T008]

- [ ] **T010 · Test · P2:** Add unit/integration tests for SystemEditor
    - **Context:** Testing Strategy (Integration Tests), Detailed Build Steps (Step 5)
    - **Action:**
        1. Add tests for `SystemEditor` in `src/editor.rs`.
        2. Test that `open_files` constructs the correct command based on input.
        3. Consider using platform-specific test commands (e.g., `echo`) instead of a real editor for simple command construction verification.
    - **Done‑when:**
        1. Tests for `SystemEditor` verify command construction and basic execution attempt.
        2. All tests pass.
    - **Depends‑on:** [T009]

## Core Journal Logic

- [ ] **T011 · Refactor · P2:** Define DateSpecifier and JournalService struct
    - **Context:** Architecture Blueprint (Modules/Packages, Public Interfaces), Detailed Build Steps (Step 6)
    - **Action:**
        1. Define `DateSpecifier` enum/struct in `src/journal/mod.rs`.
        2. Define `JournalService` struct in `src/journal/mod.rs`.
        3. Add fields to `JournalService` for `Config`, `Box<dyn JournalIO>`, and `Box<dyn Editor>`.
        4. Implement a constructor function (`NewJournalService`) for dependency injection.
    - **Done‑when:**
        1. `DateSpecifier` and `JournalService` struct are defined.
        2. `NewJournalService` function exists.
        3. `cargo check` passes.
    - **Depends‑on:** [T004, T005, T008]

- [ ] **T012 · Refactor · P2:** Implement JournalService::open_entry (Today/Specific Date)
    - **Context:** Architecture Blueprint (Public Interfaces), Detailed Build Steps (Step 6)
    - **Action:**
        1. Implement `open_entry` method on `JournalService`.
        2. Handle `DateSpecifier` variants for "today" and specific date strings.
        3. Use `chrono` to calculate the target date.
        4. Use the injected `JournalIO` dependency to get the entry path and ensure the directory/file exists.
        5. Use the injected `Editor` dependency to open the file.
        6. Return `AppResult<PathBuf>`.
    - **Done‑when:**
        1. `open_entry` method is implemented.
        2. Method uses `JournalIO` and `Editor` traits.
        3. Logic correctly handles date calculation and path generation.
        4. `cargo check` passes.
    - **Depends‑on:** [T011]

- [ ] **T013 · Refactor · P2:** Implement JournalService::open_retro_entry
    - **Context:** Architecture Blueprint (Public Interfaces), Detailed Build Steps (Step 6)
    - **Action:**
        1. Implement `open_retro_entry` method on `JournalService`.
        2. Use `chrono` to calculate the date `days_ago`.
        3. Use the injected `JournalIO` dependency to get the entry path and ensure the directory/file exists.
        4. Use the injected `Editor` dependency to open the file.
        5. Return `AppResult<PathBuf>`.
    - **Done‑when:**
        1. `open_retro_entry` method is implemented.
        2. Method uses `JournalIO` and `Editor` traits.
        3. Logic correctly handles retro date calculation and path generation.
        4. `cargo check` passes.
    - **Depends‑on:** [T011]

- [ ] **T014 · Refactor · P2:** Implement JournalService::open_reminisce_entry (Basic)
    - **Context:** Architecture Blueprint (Public Interfaces), Detailed Build Steps (Step 6)
    - **Action:**
        1. Implement `open_reminisce_entry` method on `JournalService`.
        2. For now, simply return an empty `Vec<PathBuf>` or a hardcoded path as a placeholder. The logic for finding old entries will be added later.
        3. Use the injected `Editor` dependency to open the file(s).
        4. Return `AppResult<Vec<PathBuf>>`.
    - **Done‑when:**
        1. `open_reminisce_entry` method is implemented and calls the editor.
        2. Method uses `Editor` trait.
        3. `cargo check` passes.
    - **Depends‑on:** [T011]

- [ ] **T015 · Test · P2:** Add unit tests for JournalService core logic (Date calculation, path generation)
    - **Context:** Testing Strategy (Unit Tests)
    - **Action:**
        1. Add tests for `JournalService` in `src/journal/mod.rs`.
        2. Focus tests on date calculation and the logic that determines which `JournalIO` methods are called with which arguments.
        3. Use mock/fake implementations of `JournalIO` and `Editor` traits for these tests.
    - **Done‑when:**
        1. Tests cover date calculation for "today", specific date, and "retro".
        2. Tests verify that `JournalService` calls the correct methods on its `JournalIO` and `Editor` dependencies with expected arguments.
        3. All tests pass.
    - **Depends‑on:** [T012, T013, T014]

## CLI Module

- [ ] **T016 · Refactor · P2:** Define CliArgs struct using clap
    - **Context:** Architecture Blueprint (Modules/Packages, Public Interfaces), Detailed Build Steps (Step 7)
    - **Action:**
        1. Create `src/cli.rs`.
        2. Define `CliArgs` struct using `#[derive(clap::Parser)]`.
        3. Define subcommands (`new`, `retro`, `reminisce`) and their arguments/options (e.g., `days_ago` for retro, optional date string for new).
    - **Done‑when:**
        1. `src/cli.rs` exists with `CliArgs` struct defined.
        2. `clap` attributes are added to define the CLI interface.
        3. `cargo check` passes.
    - **Depends‑on:** [T001]

- [ ] **T017 · Test · P2:** Add unit/integration tests for CLI argument parsing
    - **Context:** Testing Strategy (Unit Tests, CLI End-to-End Tests), Detailed Build Steps (Step 7)
    - **Action:**
        1. Add tests in `src/cli.rs` (or `tests/cli_tests.rs`) to verify `clap` parsing.
        2. Use `clap`'s testing features or `assert_cmd` to test different command line inputs.
        3. Verify that inputs map correctly to the fields in the parsed `CliArgs` struct.
    - **Done‑when:**
        1. Tests cover parsing for `new`, `retro`, and `reminisce` commands with various arguments.
        2. Tests verify invalid inputs result in expected errors or help messages (depending on `clap` config).
        3. All tests pass.
    - **Depends‑on:** [T016]

## Main Application Entry Point

- [ ] **T018 · Refactor · P1:** Setup main function structure and initialize components
    - **Context:** Architecture Blueprint (main.rs), Detailed Build Steps (Step 8)
    - **Action:**
        1. Update `src/main.rs`.
        2. Initialize logging (e.g., `env_logger::init()`).
        3. Call `CliArgs::parse_args()`.
        4. Call `Config::load()`, handling the `AppResult`.
        5. Instantiate `FilesystemJournalIO` and `SystemEditor`.
        6. Instantiate `JournalService` using the loaded config and the concrete adapter implementations.
    - **Done‑when:**
        1. `main.rs` parses args, loads config, instantiates I/O, Editor, and JournalService.
        2. Basic error handling is in place for config loading.
        3. `cargo build` passes.
    - **Depends‑on:** [T004, T006, T009, T011, T016]

- [ ] **T019 · Refactor · P1:** Implement main logic flow based on CLI commands
    - **Context:** Detailed Build Steps (Step 8)
    - **Action:**
        1. In `main.rs`, match on the parsed `CliArgs`.
        2. For each command (`new`, `retro`, `reminisce`), call the corresponding method on the `JournalService` instance.
        3. Handle the `AppResult` returned by the `JournalService` methods.
    - **Done‑when:**
        1. `main` function calls the appropriate `JournalService` method for each CLI command.
        2. Basic success paths are handled (e.g., printing the path opened).
        3. `cargo build` passes.
    - **Depends‑on:** [T018, T012, T013, T014]

- [ ] **T020 · Refactor · P1:** Implement top-level error handling in main
    - **Context:** Architecture Blueprint (Error & Edge-Case Strategy), Detailed Build Steps (Step 8)
    - **Action:**
        1. Wrap the main logic in `main.rs` with a `match` statement for the final `AppResult`.
        2. On `Err(app_error)`, log the detailed error using the logging facade.
        3. Print a user-friendly error message to `stderr`.
        4. Exit with a non-zero status code (e.g., `std::process::exit(1)`).
    - **Done‑when:**
        1. Any `AppError` propagated to `main` is caught.
        2. The error is logged (with details if possible).
        3. A user-facing error message is printed to `stderr`.
        4. The application exits with a non-zero status code.
    - **Depends‑on:** [T019]

- [ ] **T021 · Test · P1:** Add CLI End-to-End tests for successful paths
    - **Context:** Testing Strategy (CLI End-to-End Tests)
    - **Action:**
        1. Create `tests/e2e_cli_tests.rs`.
        2. Use `assert_cmd` and `predicates` to run the compiled binary.
        3. Test successful execution for `ponder new`, `ponder retro N`, `ponder reminisce` (assuming basic reminiscence impl).
        4. Use `tempfile::tempdir()` for a temporary journal directory.
        5. Verify exit code is 0 and expected output/side effects (like directory/file creation) occur.
    - **Done‑when:**
        1. E2E tests for happy paths of all main commands pass.
        2. Temporary files/dirs are cleaned up.
    - **Depends‑on:** [T020]

- [ ] **T022 · Test · P1:** Add CLI End-to-End tests for error paths
    - **Context:** Testing Strategy (CLI End-to-End Tests, Error & Edge-Case Strategy)
    - **Action:**
        1. Add tests in `tests/e2e_cli_tests.rs`.
        2. Use `assert_cmd` and `predicates` to run the compiled binary.
        3. Test scenarios that should result in errors (e.g., invalid arguments, editor command fails - potentially simulate this with a non-existent command or a script that exits non-zero).
        4. Verify a non-zero exit code and expected error output on `stderr`.
    - **Done‑when:**
        1. E2E tests for error scenarios pass.
        2. Application exits with non-zero status on error.
        3. Error messages are printed to stderr.
    - **Depends‑on:** [T020]

## Advanced Journal Logic & Testing

- [ ] **T023 · Feature · P2:** Implement JournalService::open_reminisce_entry logic
    - **Context:** Detailed Build Steps (Step 6), Open Questions (Reminiscence Logic)
    - **Action:**
        1. Refine the logic in `open_reminisce_entry` in `src/journal/mod.rs`.
        2. Add logic to find potential journal entries (e.g., list files in journal dir, filter by date format).
        3. Implement a simple selection mechanism (e.g., random selection, specific intervals like 1 year ago, 5 years ago).
        4. Use the injected `JournalIO` to read directory contents and check file existence.
    - **Done‑when:**
        1. `open_reminisce_entry` finds and selects journal files based on defined logic.
        2. The method returns `AppResult<Vec<PathBuf>>` with the selected paths.
    - **Depends‑on:** [T014, Clarification: Reminiscence Logic]

- [ ] **T024 · Test · P2:** Add tests for JournalService reminiscence logic
    - **Context:** Testing Strategy (Unit Tests, Integration Tests)
    - **Action:**
        1. Add tests for `open_reminisce_entry` in `src/journal/mod.rs` or `tests/`.
        2. Use a mock/fake `JournalIO` that returns a predefined list of "existing" journal files.
        3. Verify that the reminiscence logic correctly identifies and selects the expected files based on date and selection criteria.
    - **Done‑when:**
        1. Tests cover the reminiscence file finding and selection logic.
        2. Tests pass for various scenarios (e.g., no files, files from different dates, files matching criteria).
    - **Depends‑on:** [T023]

- [ ] **T025 · Feature · P3:** Add timestamp to new/retro entries
    - **Context:** Detailed Build Steps (Step 6)
    - **Action:**
        1. Modify `JournalService::open_entry` and `open_retro_entry`.
        2. Before opening the file in the editor, use the injected `JournalIO` to append a timestamp line (e.g., `## YYYY-MM-DD HH:MM`) if the file was just created or is currently empty.
    - **Done‑when:**
        1. New or retro journal files automatically get a timestamp appended upon opening if they are empty.
        2. Tests verify this behavior using a fake `JournalIO` that records appends.
    - **Depends‑on:** [T012, T013]

## Tooling & Automation

- [ ] **T026 · Chore · P1:** Configure and enforce rustfmt
    - **Context:** Detailed Build Steps (Step 9), DEVELOPMENT_PHILOSOPHY_APPENDIX_RUST (Section 2)
    - **Action:**
        1. Add a `rustfmt.toml` if custom rules are desired (prefer defaults).
        2. Add a CI job step to run `cargo fmt --check`.
    - **Done‑when:**
        1. `cargo fmt --check` is configured and runs in CI.
        2. CI build fails if code is not formatted correctly.
    - **Depends‑on:** [T001]

- [ ] **T027 · Chore · P1:** Configure and enforce clippy
    - **Context:** Detailed Build Steps (Step 9), DEVELOPMENT_PHILOSOPHY_APPENDIX_RUST (Section 3)
    - **Action:**
        1. Add `.cargo/config.toml` or configure CI flags to run `cargo clippy --all-targets --all-features -- -D warnings`.
        2. Review initial clippy warnings and fix/deny them appropriately.
    - **Done‑when:**
        1. `cargo clippy --all-targets --all-features -- -D warnings` is configured and runs in CI.
        2. CI build fails on clippy warnings.
    - **Depends‑on:** [T001]

- [ ] **T028 · Chore · P1:** Setup CI pipeline (build, test, lint, format, audit)
    - **Context:** Detailed Build Steps (Step 9), Automation, Quality Gates, and CI/CD
    - **Action:**
        1. Create CI workflow file (e.g., `.github/workflows/ci.yml`).
        2. Configure jobs for building (`cargo build`), running tests (`cargo test --all-features`), running format check (`cargo fmt --check`), running clippy (`cargo clippy ...`), and running audit (`cargo audit`).
        3. Ensure these steps are mandatory quality gates for merging.
    - **Done‑when:**
        1. CI pipeline runs on pushes/PRs.
        2. Pipeline includes build, test, format check, clippy, and audit steps.
        3. CI build fails if any of these steps fail.
    - **Depends‑on:** [T001, T026, T027, T029]

- [ ] **T029 · Chore · P1:** Integrate cargo-audit into CI
    - **Context:** Detailed Build Steps (Step 9), Security Considerations (Dependency Management Security), DEVELOPMENT_PHILOSOPHY_APPENDIX_RUST (Section 13)
    - **Action:**
        1. Add `cargo-audit` to the project (or ensure it's available in CI environment).
        2. Add a step in the CI pipeline to run `cargo audit`.
        3. Configure the step to fail the build on critical/high severity vulnerabilities.
    - **Done‑when:**
        1. `cargo audit` runs in CI.
        2. CI build fails if critical/high vulnerabilities are found.
    - **Depends‑on:** [T001]

- [ ] **T030 · Test · P2:** Integrate test coverage reporting into CI
    - **Context:** Testing Strategy (Coverage Targets), DEVELOPMENT_PHILOSOPHY_APPENDIX_RUST (Section 10)
    - **Action:**
        1. Choose a Rust coverage tool (e.g., `cargo-tarpaulin` or `grcov`).
        2. Add the tool and configure it in CI to collect coverage data (`cargo test --all-features` with coverage flags).
        3. Publish the coverage report to a service (e.g., Codecov, Coveralls) or report it in the CI output.
        4. (Optional but recommended) Configure CI to fail if coverage drops below the target threshold (e.g., 80% overall).
    - **Done‑when:**
        1. Test coverage is measured and reported in CI.
        2. (Optional) CI enforces minimum coverage threshold.
    - **Depends‑on:** [T021, T022]

## Documentation

- [ ] **T031 · Chore · P2:** Add/Update Rustdoc comments
    - **Context:** Detailed Build Steps (Step 10), Documentation (Code Self-Doc Patterns), DEVELOPMENT_PHILOSOPHY_APPENDIX_RUST (Section 11)
    - **Action:**
        1. Add `///` comments to all public items (modules, structs, enums, traits, functions) in `src/lib.rs` (if applicable), `src/cli.rs`, `src/config.rs`, `src/errors.rs`, `src/editor.rs`, `src/journal/mod.rs`, `src/journal/io.rs`.
        2. Explain purpose, arguments, return values, and potential errors (`# Errors`).
    - **Done‑when:**
        1. All public items have clear Rustdoc comments.
        2. `cargo doc` runs without warnings.
    - **Depends‑on:** [T019]

- [ ] **T032 · Chore · P2:** Create/Update README.md
    - **Context:** Detailed Build Steps (Step 10), Documentation (Required Readme)
    - **Action:**
        1. Create or update `README.md`.
        2. Include project description, features, installation (from source), usage examples for all commands, configuration details, architecture overview, contributing link, and license.
    - **Done‑when:**
        1. `README.md` is present and contains the required sections.
        2. Usage examples are clear and correct.
    - **Depends‑on:** [T019]

- [ ] **T033 · Chore · P3:** Create CONTRIBUTING.md
    - **Context:** Documentation (Required Readme)
    - **Action:**
        1. Create `CONTRIBUTING.md`.
        2. Include sections on development setup, code style (`rustfmt`, `clippy`), testing procedures, and PR process.
    - **Done‑when:**
        1. `CONTRIBUTING.md` is present and outlines how to contribute.
    - **Depends‑on:** [T026, T027, T030]

### Clarifications & Assumptions

- [ ] **Issue:** Minimum Supported Rust Version (MSRV) not defined.
    - **Context:** Open Questions (Minimum Supported Rust Version (MSRV))
    - **Blocking?:** no

- [ ] **Issue:** Configuration file support is an open question.
    - **Context:** Open Questions (Extended Configuration)
    - **Blocking?:** no (Proceeding with environment variables first as per recommendation)

- [ ] **Issue:** Specific date/time formats for entries/filenames need clarification.
    - **Context:** Open Questions (Date/Time Formatting)
    - **Blocking?:** no (Proceeding with YYYY-MM-DD format for now)

- [ ] **Issue:** Editor fallback behavior if `$EDITOR` and default (`vim`)