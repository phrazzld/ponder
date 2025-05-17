# Plan: Eliminate Unnecessary Trait Abstractions (`JournalIO`, `Editor`)

## Chosen Approach (One‑liner)

Aggressively refactor by removing `JournalIO` and `Editor` traits, their mocks, and the `JournalService` struct, consolidating logic into a new `src/journal_logic.rs` module that uses `std::fs` and `std::process::Command` directly.

## Architecture Blueprint

-   **Modules / Packages**
    -   `src/main.rs`
        -   Responsibility: Application entry point, CLI argument parsing, configuration loading, logging setup, orchestrating calls to `journal_logic`.
    -   `src/cli.rs`
        -   Responsibility: Defines CLI argument structure using `clap`. (Largely unchanged)
    -   `src/config.rs`
        -   Responsibility: Defines `Config` struct, loads configuration from environment variables and defaults. Contains `ensure_journal_dir` logic. (Largely unchanged, but `ensure_journal_dir` might become a free function or part of `journal_logic` if `Config` becomes pure data).
    -   `src/errors.rs`
        -   Responsibility: Defines `AppError` enum and `AppResult` type alias. (Unchanged)
    -   `src/journal_logic.rs` (new, effectively replaces `src/journal/mod.rs`, `src/journal/io/mod.rs`, `src/editor.rs`)
        -   Responsibility: Contains all core journaling operations (opening entries, appending headers), date calculations (`DateSpecifier`), direct file system interactions (using `std::fs`, `std::io`), and editor invocation logic (using `std::process::Command`).
    -   `src/lib.rs`
        -   Responsibility: Crate root, re-exports public types from `cli`, `config`, `errors`, and `journal_logic`. (Updated to reflect module changes)

    **Removed Modules/Files:**
    -   `src/journal/mod.rs` (as a container for `JournalService` and `DateSpecifier` if `DateSpecifier` moves to `journal_logic.rs`)
    -   `src/journal/io/mod.rs` (and any `tests.rs` within it)
    -   `src/editor.rs`
    -   Test files or modules relying solely on the removed mocks (e.g., parts of `src/journal/tests.rs`).

-   **Public Interfaces / Contracts**

    Key functions and types in `src/journal_logic.rs`:

    ```rust
    // In src/journal_logic.rs

    use crate::config::Config;
    use crate::errors::AppResult;
    use chrono::NaiveDate;
    use std::path::{Path, PathBuf};

    // DateSpecifier enum, moved here from src/journal/mod.rs
    pub enum DateSpecifier {
        Today,
        Retro,
        Reminisce,
        Specific(NaiveDate),
    }

    impl DateSpecifier {
        // Parses CLI args to create a DateSpecifier
        pub fn from_args(retro: bool, reminisce: bool, date_str: Option<&str>) -> AppResult<Self>;
        // Gets the concrete dates based on the specifier
        pub fn get_dates(&self) -> Vec<NaiveDate>;
        // (Private helper for parsing date string if needed)
    }

    // Main operational function, replaces JournalService::open_entries
    pub fn open_journal_entries(config: &Config, date_spec: &DateSpecifier) -> AppResult<()>;

    // Ensures the journal directory exists, potentially moved from Config or main
    pub fn ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()>;

    // Private helper functions (examples, actual breakdown TBD during implementation):
    // fn get_entry_path_for_date(journal_dir: &Path, date: NaiveDate) -> PathBuf;
    // fn create_or_open_entry_file(path: &Path) -> AppResult<std::fs::File>;
    // fn append_date_time_header_if_new(file: &mut std::fs::File, path: &Path) -> AppResult<()>;
    // fn launch_editor_for_files(editor_cmd: &str, file_paths: &[PathBuf]) -> AppResult<()>;
    ```

-   **Data Flow Diagram**

    ```mermaid
    graph TD
        User -->|CLI Args| MainRS[main.rs::main]
        MainRS -->|Parse Args| CliRS[cli::CliArgs::parse]
        CliRS -->|Date Flags, Date String| DateSpecifierInit[journal_logic::DateSpecifier::from_args]
        MainRS -->|Load Config| ConfigRS[config::Config::load]
        ConfigRS -->|Validate Config| ConfigRSValidate[config::Config::validate]
        MainRS -->|Get Journal Dir Path| ConfigRS
        MainRS -->|Ensure Dir| JournalLogicEnsureDir[journal_logic::ensure_journal_directory_exists]
        MainRS -->|Pass Config, DateSpec| JournalLogicOpen[journal_logic::open_journal_entries]
        
        JournalLogicOpen -->|Calculate Paths| InternalPathHelpers
        InternalPathHelpers -->|Use journal_dir| ConfigRS
        JournalLogicOpen -->|Filesystem Ops| StdFs[std::fs / std::io]
        JournalLogicOpen -->|Editor Invocation| StdProcess[std::process::Command]
        
        StdFs -->|Read/Write Files| Disk[Physical Disk]
        StdProcess -->|Launch Editor Process| OS[Operating System Process]
        
        JournalLogicOpen -->|Error?| MainRS
        DateSpecifierInit -->|Error?| MainRS
        ConfigRS -->|Error?| MainRS
        JournalLogicEnsureDir -->|Error?| MainRS
        MainRS -->|Log Error & Exit| User
    ```

-   **Error & Edge‑Case Strategy**
    -   Continue using the `AppError` enum and `AppResult<T>` alias from `src/errors.rs`.
    -   All I/O operations in `journal_logic.rs` using `std::fs` or `std::process::Command` will map their errors to appropriate `AppError` variants (e.g., `AppError::Io`, `AppError::Editor`).
    -   `DateSpecifier::from_args` will return `AppError::Journal` for invalid date formats.
    -   Edge cases:
        -   Journal directory not found/creatable: Handled by `ensure_journal_directory_exists`.
        -   File permissions issues: Propagated as `AppError::Io`.
        -   Editor command not found or fails: Propagated as `AppError::Editor`.
        -   Retro/reminisce entries not found: Handled within `open_journal_entries` (likely by creating new empty files as per current logic, or logging if behavior changes).
    -   The fragile `Clone` implementation for `AppError::Io` is acknowledged as existing technical debt, not addressed by this task.

## Detailed Build Steps

1.  **Create `src/journal_logic.rs`:**
    *   Create the new file.
    *   Move the `DateSpecifier` enum and its `impl` block from `src/journal/mod.rs` (or wherever it currently resides) to `src/journal_logic.rs`. Update imports.
2.  **Define Core Operational Function in `journal_logic.rs`:**
    *   Create `pub fn open_journal_entries(config: &Config, date_spec: &DateSpecifier) -> AppResult<()>` in `src/journal_logic.rs`.
    *   Begin by outlining the high-level logic from the old `JournalService::open_entries` method.
3.  **Implement Filesystem Logic Directly:**
    *   Identify all file system operations previously done by `FileSystemIO` (e.g., `generate_path_for_date`, `file_exists`, `create_or_open_file`, `read_file_content`, `append_to_file`).
    *   Re-implement these as private helper functions within `src/journal_logic.rs` using `std::fs`, `std::io`, and `std::path::Path/PathBuf`.
    *   Example: `fn get_entry_path(journal_dir: &Path, date: NaiveDate) -> PathBuf`.
    *   Integrate these helpers into `open_journal_entries`.
4.  **Implement Editor Launch Logic Directly:**
    *   Identify the editor launching logic previously in `SystemEditor::open_files`.
    *   Re-implement this as a private helper function, e.g., `fn launch_editor(editor_cmd: &str, files_to_open: &[PathBuf]) -> AppResult<()>` in `src/journal_logic.rs`, using `std::process::Command`.
    *   Integrate this helper into `open_journal_entries`.
5.  **Implement Supporting Logic:**
    *   Re-implement logic from `JournalService::append_date_time` as a private helper, e.g., `fn append_date_header_if_needed(entry_path: &Path) -> AppResult<()>` in `src/journal_logic.rs`.
    *   Implement `pub fn ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()>` using `std::fs::create_dir_all`.
6.  **Refactor `main.rs`:**
    *   Remove instantiation and usage of `JournalService`, `FileSystemIO`, `SystemEditor`.
    *   After loading `config` and creating `date_specifier` (now from `journal_logic::DateSpecifier::from_args`):
        *   Call `journal_logic::ensure_journal_directory_exists(&config.journal_dir)?`.
        *   Call `journal_logic::open_journal_entries(&config, &date_specifier)?`.
    *   Update any `use` statements.
7.  **Remove Obsolete Code and Files:**
    *   Delete the `JournalIO` trait definition.
    *   Delete the `MockJournalIO` struct and its `impl`.
    *   Delete the `Editor` trait definition.
    *   Delete the `MockEditor` struct and its `impl`.
    *   Delete the `JournalService` struct and its `impl` block.
    *   Delete the files: `src/journal/io/mod.rs` (and any `tests.rs` within), `src/editor.rs`.
    *   If `src/journal/mod.rs` becomes empty or only re-exports from `journal_logic.rs`, consider merging `journal_logic.rs` into `journal.rs` and deleting `journal_logic.rs`, or simply deleting `src/journal/mod.rs` if `journal_logic.rs` is at `src/` level. For this plan, we assume `journal_logic.rs` is the new home.
8.  **Update `src/lib.rs`:**
    *   Remove `pub mod journal;` and `pub mod editor;` if they are fully superseded.
    *   Add `pub mod journal_logic;`.
    *   Adjust re-exports as necessary (e.g., `pub use journal_logic::DateSpecifier;`).
9.  **Adjust Tests:**
    *   Delete any unit tests that were solely testing the mock implementations (e.g., in `src/journal/io/tests.rs` or `src/journal/tests.rs` if they relied on mocks).
    *   Review existing integration tests (e.g., `tests/cli_tests.rs`, `tests/journal_integration_tests.rs`). They should require minimal changes as they test behavior, but verify they compile and pass.
    *   Remove any `#[cfg(test)]` helper methods on `JournalService` that are now gone.
10. **Code Cleanup:**
    *   Run `cargo fmt` and `cargo clippy -- -D warnings`.
    *   Review all changed files for dead code, unnecessary imports, or comments referring to the old abstractions.

## Testing Strategy

-   **Test layers**:
    -   **Unit Tests**: Focus on pure functions and isolated logic within `journal_logic.rs` (e.g., `DateSpecifier::get_dates`, date string parsing, path generation logic if complex enough to warrant isolation). **No mocking of internal components.**
    -   **Integration Tests**: Primary verification method. These tests (in the `tests/` directory) will execute the compiled binary using `assert_cmd`.
        -   They will interact with a temporary file system (`tempfile` crate) to simulate the journal directory.
        -   The editor command will be configured (e.g., via environment variable for tests) to use a harmless command like `echo` or `true` to verify invocation without actual UI interaction.
-   **What to mock**: **No internal components will be mocked.** Mocking is strictly for true external system boundaries. For this CLI:
    -   The filesystem is interacted with directly via `std::fs` but within a controlled temporary directory for tests.
    -   The external editor process is a true external boundary; its command can be overridden in tests to a non-interactive one.
-   **Coverage targets & edge‑case notes**:
    -   Aim for high integration test coverage of all CLI commands and options (`today`, `retro`, `reminisce`, specific dates).
    -   Test journal directory creation.
    -   Test file creation, content of new files (e.g., date header).
    -   Test appending to existing files.
    -   Test editor invocation with correct file paths (single and multiple).
    -   Test error paths: invalid date formats, non-existent editor command (if feasible to simulate by setting `EDITOR` to a bogus command).

## Logging & Observability

-   The existing `env_logger` setup in `main.rs` for structured JSON logging will remain.
-   Existing `log::info!`, `debug!`, `error!` calls will continue to function.
-   This refactoring does not introduce new specific log events but simplifies the code paths where logs are emitted.
-   No correlation ID propagation is currently implemented or required by this change for this simple CLI tool.

## Security & Config

-   **Input validation hotspots**:
    -   CLI arguments parsed by `clap`.
    -   Date strings parsed by `chrono` (within `DateSpecifier::from_args`).
    -   Editor command string from `Config` (potential for command injection if not handled carefully, though `std::process::Command` is generally safe if the command itself is trusted and arguments are passed separately).
-   **Secrets handling**: The editor command path is user-supplied configuration, not a secret managed by the application. No other secrets involved.
-   **Least‑privilege notes**: Filesystem operations are performed with the user's privileges. `ensure_journal_directory_exists` uses `std::fs::create_dir_all` which creates directories with default permissions.

## Documentation

-   **Code self‑doc patterns**:
    -   Update Rustdoc comments (`///`) for all public functions in `src/journal_logic.rs`, `src/config.rs`, `src/cli.rs`, and `src/main.rs` whose signatures or behavior might have subtly changed or been clarified.
    -   Ensure module-level documentation for `src/journal_logic.rs` explains its role.
-   **Any required readme or openapi updates**:
    -   Update `README.md` to remove any architectural descriptions mentioning trait-based dependency injection or the old `JournalService`/`JournalIO`/`Editor` structure. Emphasize the simpler, direct architecture.

## Risk Matrix

| Risk                                                                 | Severity | Mitigation                                                                                                |
| -------------------------------------------------------------------- | -------- | --------------------------------------------------------------------------------------------------------- |
| Breaking existing mock-based unit tests                              | High     | Expected. These tests are to be removed. Functionality verification shifts to integration tests.        |
| Introducing regressions due to incorrect reimplementation of logic   | Medium   | Comprehensive integration tests covering all CLI commands and edge cases. Thorough code review.             |
| `journal_logic.rs` becomes a monolith / "god module"                 | Medium   | Keep helper functions within `journal_logic.rs` well-defined and focused. Future refactoring if it grows too large. |
| Incomplete removal of old code/files/dependencies                    | Low      | Careful execution of removal steps, `cargo clippy` for unused code/imports, `cargo tree` for dependencies.  |
| Test setup for editor invocation becomes complex                     | Low      | Use simple, reliable commands like `echo` or `true` for the test editor, controlled via environment variables. |
| Fragile `AppError::Io::Clone` behavior remains                       | Low      | Acknowledged pre-existing issue; not in scope for this task. To be addressed by "Standardize Error Handling". |

## Open Questions

-   None that block execution of *this* specific task. The primary goal is clear: remove the specified traits and mocks, and simplify. The exact internal structure of `journal_logic.rs` (e.g., how many private helpers) can be determined during implementation, guided by readability and simplicity.