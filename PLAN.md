# Plan: Enforce Strict Module Boundaries and Feature-Based Organization

## Chosen Approach (One‑liner)

Systematically reorganize the `src/` directory into feature-based modules (`cli`, `config`, `errors`, `journal_core`, `journal_io`) with clear responsibilities, high cohesion, and minimal public APIs, strictly adhering to the "Package by Feature, Not Type" guideline.

## Architecture Blueprint

-   **Modules / Packages**:
    -   `main.rs` → Application entry point. Orchestrates top-level flow: argument parsing, configuration loading, error handling, and delegation to feature modules.
        -   *Responsibility*: Orchestration and top-level error handling.
    -   `cli` (`src/cli/mod.rs`) → Handles all command-line argument parsing (using `clap`) and deriving feature-specific requests (like `DateSpecifier` creation intent) from those arguments.
        -   *Responsibility*: User command-line interaction contract and initial input validation.
    -   `config` (`src/config/mod.rs`) → Manages application configuration loading (from environment variables, with defaults), validation, and provides access to configuration values.
        -   *Responsibility*: Application runtime configuration and validation.
    -   `errors` (`src/errors/mod.rs`) → Defines the application's custom error types (`AppError`, `AppResult`) and centralizes error-related utilities. Replaces current `errors.rs`.
        -   *Responsibility*: Standardized error representation and propagation.
    -   `journal_core` (`src/journal_core/mod.rs`) → Encapsulates pure logic related to journal entries, primarily date calculations, `DateSpecifier` definition, and determination of target dates. This module will have no side effects (no I/O).
        -   *Responsibility*: Pure business logic for journal date specifications and calculations.
    -   `journal_io` (`src/journal_io/mod.rs`) → Handles all I/O operations for journal entries. This includes file path generation, directory creation, file creation/reading/writing, and launching the external editor. Depends on `journal_core` for date logic and `config` for paths/editor.
        -   *Responsibility*: Infrastructure and side-effectful operations for journal management.
    -   `lib.rs` → Declares all public modules (`pub mod ...;`) and re-exports essential types forming the library's public API. Primarily for binary use, but allows for integration testing and potential library usage.

-   **Public Interfaces / Contracts** (Illustrative Signatures):
    -   `cli::CliArgs` (struct, `pub`)
        -   `CliArgs::parse_args() -> Self` (or direct `clap::Parser::parse()` in `main.rs`)
    -   `config::Config` (struct, `pub`)
        -   `Config::load() -> AppResult<Self>`
        -   `Config::journal_dir(&self) -> &Path`
        -   `Config::editor_command(&self) -> &str`
        -   `Config::validate(&self) -> AppResult<()>`
    -   `errors::AppError` (enum, `pub`)
    -   `errors::AppResult<T>` (type alias, `pub`)
    -   `journal_core::DateSpecifier` (enum, `pub`)
        -   `DateSpecifier::from_cli_args(retro: bool, reminisce: bool, date_str: Option<&str>) -> AppResult<Self>`
        -   `DateSpecifier::resolve_dates(&self, reference_date: NaiveDate) -> Vec<NaiveDate>`
    -   `journal_io::open_journal_entries(config: &config::Config, dates: &[NaiveDate]) -> AppResult<()>`
    -   `journal_io::ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()>`

-   **Data Flow Diagram** (Mermaid):
    ```mermaid
    graph TD
        User -- CLI Args --> MainRS(main.rs);
        MainRS -- Parse Args --> CLI(cli_module);
        CLI -- CliArgs --> MainRS;
        MainRS -- Load Config --> ConfigMod(config_module);
        ConfigMod -- Config --> MainRS;
        MainRS -- Validate Config --> ConfigMod;

        MainRS -- CliArgs --> JCore(journal_core_module);
        JCore -- DateSpecifier --> MainRS;
        MainRS -- DateSpecifier --> JCore;
        JCore -- Vec<NaiveDate> --> MainRS;

        MainRS -- Config, Vec<NaiveDate> --> JIO(journal_io_module);
        JIO -- Ensure Dir --> FileSystem((File System));
        JIO -- Open Entries/File Ops --> FileSystem;
        JIO -- Launch Editor --> EditorProcess((External Editor));

        CLI -- Errors --> ErrMod(errors_module);
        ConfigMod -- Errors --> ErrMod;
        JCore -- Errors --> ErrMod;
        JIO -- Errors --> ErrMod;
        ErrMod -- AppError/AppResult --> MainRS;
        MainRS -- User-Facing Error --> StdErr;
    ```

-   **Error & Edge‑Case Strategy**:
    -   All fallible operations return `errors::AppResult<T>`.
    -   `errors::AppError` is the unified error type. Variants should clearly map to potential failure domains (e.g., `Config`, `Io`, `JournalCore`, `JournalIo`, `CliParse`, `EditorLaunch`).
    -   Errors are propagated upwards using `?`. `main.rs` is the final boundary for catching and reporting errors to the user.
    -   Input validation occurs at module boundaries:
        -   `cli`: Validates CLI argument structure (via `clap`).
        -   `config`: Validates loaded configuration values (editor command, path validity).
        -   `journal_core`: Validates date string formats if provided for `DateSpecifier`.
        -   `journal_io`: Handles I/O errors robustly.
    -   Edge cases: Empty journal directory, non-existent files, invalid dates, invalid editor configurations, permission issues.

## Detailed Build Steps

1.  **Preparation**:
    1.  Ensure a clean working directory on the main branch.
    2.  Verify all current tests pass: `cargo test --all-features`.
    3.  Create a new feature branch: `git checkout -b feat/refactor-module-boundaries`.

2.  **Establish New Module Structure**:
    1.  Create directories: `src/errors`, `src/journal_core`, `src/journal_io`.
    2.  Create `mod.rs` files: `src/errors/mod.rs`, `src/journal_core/mod.rs`, `src/journal_io/mod.rs`.
    3.  In `src/lib.rs`, update module declarations and re-exports:
        ```rust
        // Remove:
        // pub mod errors;
        // pub mod journal_logic;

        // Add/Keep:
        pub mod cli;
        pub mod config;
        pub mod errors; // New module
        pub mod journal_core; // New module
        pub mod journal_io; // New module

        // Update re-exports:
        pub use cli::CliArgs;
        pub use config::Config;
        pub use errors::{AppError, AppResult};
        pub use journal_core::DateSpecifier;
        // Potentially add key functions from journal_io if Ponder aims to be a usable library.
        // For now, focus on binary structure; main.rs will use journal_io directly.
        ```
    4.  In `src/main.rs`, remove any `mod` declarations. Update `use` statements to reflect new paths (e.g., `use ponder::errors::AppResult;`).

3.  **Migrate `errors.rs` to `src/errors/mod.rs`**:
    1.  Move content of `src/errors.rs` to `src/errors/mod.rs`.
    2.  Consider refining `AppError` variants for more specificity if beneficial (e.g., `AppError::Journal` -> `AppError::JournalCore` or `AppError::JournalIo`).
    3.  Update all `use crate::errors` paths to `use crate::errors` (if previously `crate::errors_file_name`). The module name is `errors`.
    4.  Run `cargo check`. Fix compiler errors.
    5.  Delete `src/errors.rs`.

4.  **Refactor `journal_logic.rs` into `journal_core` and `journal_io`**:
    1.  **`src/journal_core/mod.rs`**:
        *   Move `DateSpecifier` enum definition.
        *   Move `DateSpecifier::from_args` (rename to `DateSpecifier::from_cli_args(retro: bool, reminisce: bool, date_str: Option<&str>) -> AppResult<Self>`). This function should contain the logic to parse date strings and determine the specifier type. It should *not* depend on `CliArgs` directly.
        *   Move `DateSpecifier::get_dates` (rename to `DateSpecifier::resolve_dates(&self, reference_date: NaiveDate) -> Vec<NaiveDate>`). This function takes the current date as a parameter for calculations.
        *   Ensure this module contains only pure logic (no `std::fs`, no `std::process::Command`).
    2.  **`src/journal_io/mod.rs`**:
        *   Move remaining functions from `journal_logic.rs`:
            *   `ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()>`
            *   `open_journal_entries` (adapt signature to `open_journal_entries(config: &config::Config, dates: &[NaiveDate]) -> AppResult<()>)`. This function will iterate `dates`, call `get_entry_path_for_date` for each, `create_or_open_entry_file`, `append_date_header_if_needed`, and finally `launch_editor`.
            *   Helper functions (make private to `journal_io` module unless strong justification for `pub(crate)`): `get_entry_path_for_date`, `file_exists`, `create_or_open_entry_file`, `read_file_content`, `append_to_file`, `launch_editor`, `append_date_header_if_needed`.
        *   This module will depend on `crate::config` and `crate::journal_core` (for types like `NaiveDate` if passed, or `DateSpecifier` if logic requires it, though ideally it just gets `Vec<NaiveDate>`).
    3.  Update all `use crate::journal_logic` paths to `crate::journal_core` or `crate::journal_io` as appropriate.
    4.  Run `cargo check`. Fix compiler errors.
    5.  Delete `src/journal_logic.rs`.

5.  **Adapt `cli/mod.rs`**:
    1.  The `CliArgs` struct remains.
    2.  Logic previously in `main.rs`'s `get_date_specifier_from_args` that called `DateSpecifier::from_args` will now be handled by `main.rs` calling `journal_core::DateSpecifier::from_cli_args` directly, passing fields from `CliArgs`.
    3.  Ensure `cli` module focuses solely on parsing arguments into `CliArgs`.

6.  **Update `config/mod.rs`**:
    1.  Remove the deprecated `Config::ensure_journal_dir` method. This functionality is now in `journal_io::ensure_journal_directory_exists`.
    2.  Ensure `use` paths for `AppError`, `AppResult` point to `crate::errors`.

7.  **Refactor `main.rs` Orchestration**:
    1.  Parse CLI args: `let args = CliArgs::parse_args();` (or `CliArgs::parse()`).
    2.  Load config: `let config = Config::load()?;`.
    3.  Validate config: `config.validate()?;`.
    4.  Ensure journal directory: `journal_io::ensure_journal_directory_exists(config.journal_dir())?;`.
    5.  Determine date specifier: `let date_spec = journal_core::DateSpecifier::from_cli_args(args.retro, args.reminisce, args.date.as_deref())?;`.
    6.  Resolve dates: `let today = Local::now().date_naive(); let dates_to_open = date_spec.resolve_dates(today);`.
    7.  Open entries: `journal_io::open_journal_entries(&config, &dates_to_open)?;`.
    8.  Ensure `main.rs` is lean, delegating tasks and handling top-level errors.

8.  **Iterative Build and Test**:
    1.  After each significant code migration: `cargo check`, then `cargo test --all-features`. Fix issues immediately.

9.  **Review Public APIs and Encapsulation**:
    1.  For each module (`cli`, `config`, `errors`, `journal_core`, `journal_io`), critically review every `pub` item.
    2.  Default to private. Use `pub(crate)` if needed internally within the crate. Use `pub` only for items that form the deliberate public contract of that module *or* the library as a whole (via `lib.rs` re-exports).
    3.  Minimize public surface area.

10. **Code Cleanup and Final Checks**:
    1.  Run `cargo fmt`.
    2.  Run `cargo clippy --all-targets -- -D warnings`. Address all lints.
    3.  Manually inspect `src/` to confirm the new structure and removal of old files.

11. **Documentation Update**:
    1.  Add module-level documentation (`//! ...`) to `src/cli/mod.rs`, `src/config/mod.rs`, `src/errors/mod.rs`, `src/journal_core/mod.rs`, and `src/journal_io/mod.rs`, explaining responsibilities and public API.
    2.  Update `src/lib.rs` crate-level documentation.
    3.  Update "Architecture" sections in `README.md` and `CLAUDE.md`.
    4.  Verify `cargo doc --open` and review generated documentation.

12. **Final Verification**:
    1.  Run `cargo test --all-features` one last time.
    2.  Manually execute the binary with various flags to confirm E2E functionality.

## Testing Strategy

-   **Test Layers**:
    -   **Unit Tests**: Colocated `#[cfg(test)]` modules will test private functions and the internal logic of public functions within their respective modules. `journal_core` should be highly amenable to unit testing due to its pure nature.
    -   **Integration Tests**: Existing tests in `tests/` directory (`cli_tests.rs`, `config_tests.rs`, `journal_integration_tests.rs`, etc.) are crucial. They will primarily test through the public API exposed by `lib.rs` or by invoking the binary. These tests will verify that the refactored modules integrate correctly. Update their `use` statements as needed.
-   **What to mock**:
    -   Strictly adhere to "Mock ONLY True External System Boundaries".
    -   For `journal_io` tests: Use `tempfile` for filesystem interactions. Mock environment variables for editor command testing (e.g., using `assert_cmd::Command::env`).
    -   **NO mocking of internal modules** (e.g., `journal_core` will not be mocked by `journal_io` tests). If a module is hard to test without mocking its internal Rust dependencies, it's a signal that its boundaries or responsibilities need refinement.
-   **Coverage targets**: Maintain or exceed existing coverage targets (e.g., >85% overall, >95% for `journal_core` and `config`). Run `cargo-tarpaulin` or similar to verify.
-   **Edge‑case notes**: Ensure existing tests for invalid inputs, file system errors, editor failures, etc., are still effective and cover the refactored code paths.

## Logging & Observability

-   No fundamental changes to the logging mechanism (`env_logger` setup in `main.rs`).
-   Log statements will now originate from their new modules. The `log` macros automatically include the module path in `target`.
-   Ensure `main.rs` handles top-level error logging before exit.
-   Correlation ID (if implemented or planned) propagation is unaffected by this structural refactor but would flow through these well-defined module interactions.

## Security & Config

-   **Input Validation Hotspots**:
    -   `cli`: Initial argument structure (via `clap`).
    -   `config`: Editor command validation (`Config::validate_editor_command`) remains critical. Path validation for `journal_dir`.
    -   `journal_core`: Date string parsing.
-   **Secrets Handling**: No direct secrets handled. Editor command is sensitive; validation in `config` is key.
-   **Least-Privilege Notes**:
    -   File permissions (0600 for files, 0700 for directories) handled by `journal_io` using standard library functions that respect umask.
    -   Editor launching in `journal_io` must continue to execute commands directly, not via a shell, to prevent injection.

## Documentation

-   **Code Self-Doc Patterns**:
    -   All `pub` items in all modules must have comprehensive Rustdoc comments (`///`).
    -   Module-level docs (`//!`) for every `mod.rs` file.
    -   Use comments for "why," not "how," for non-obvious logic.
-   **Required Readme/OpenAPI Updates**:
    -   Update "Architecture" section in `README.md`.
    -   Update "High-Level Architecture" and "Module Flow" in `CLAUDE.md`.
    -   Ensure `lib.rs` documentation accurately reflects the library's public API.

## Risk Matrix

| Risk                                         | Severity | Mitigation                                                                                                |
| :------------------------------------------- | :------- | :-------------------------------------------------------------------------------------------------------- |
| Breaking existing functionality              | High     | Comprehensive integration tests; incremental refactoring with frequent `cargo test`; careful code review. |
| Introducing circular module dependencies     | Medium   | Clear, hierarchical design (`journal_io` depends on `journal_core`, not vice-versa); compiler will error.   |
| Incorrectly defined module responsibilities  | Medium   | Strict adherence to "Package by Feature"; review against module responsibility statements.                  |
| Public API overexposure / leaky abstractions | Medium   | Rigorous review of `pub` items; default to private/`pub(crate)`; focus on minimal public surface.       |
| Test suite gaps due to code movement         | Medium   | Review test coverage post-refactor; ensure tests target behavior through new module boundaries.             |
| Merge conflicts with ongoing work            | Medium   | Communicate refactoring timeline; keep branch short-lived or rebase frequently.                             |

## Open Questions

-   Are the proposed public interfaces for `journal_core` and `journal_io` optimal for `main.rs`'s orchestration needs, or is further refinement needed during implementation?
-   Should `journal_core::DateSpecifier::resolve_dates` take `chrono::Local::now().date_naive()` as an argument, or should `main.rs` pass it, to keep `journal_core` fully deterministic and independent of system time? (Decision: Pass as argument for testability and purity, as reflected in plan).
-   Are there any subtle interdependencies in the current `journal_logic.rs` that might complicate the split into `_core` and `_io` more than anticipated? (To be discovered during detailed implementation).
