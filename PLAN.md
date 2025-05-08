# Plan: Refactor Ponder CLI Application for Enhanced Modularity and Maintainability

## Chosen Approach (One‑liner)

Re-architect the Ponder CLI application into distinct, loosely coupled Rust modules with clear interfaces and dependencies, strictly adhering to principles of simplicity, modularity, separation of concerns, and testability, while implementing robust error handling and structured logging.

## Architecture Blueprint

Ponder will adopt a modular architecture where core journaling logic is decoupled from CLI interactions, configuration management, file system operations, and external editor interactions. Dependencies will flow inwards, primarily managed through traits for external boundaries.

-   **Modules / Packages**
    -   `cli`: Handles command-line argument parsing (using `clap`), user input validation, and help text generation. Single responsibility: CLI interaction.
    -   `config`: Manages loading and providing application configuration (e.g., journal directory, editor path, date formats). Single responsibility: Configuration management.
    -   `journal`: Contains the core business logic for determining journal entry dates, generating filenames, and orchestrating interactions with storage and the editor. Single responsibility: Core journaling logic and workflow.
    -   `journal::io` (or `io_adapter`): Handles low-level file system operations related to journal entries (reading, writing, creating directories/files). Implements a trait defined by the `journal` module. Single responsibility: Journal file system abstraction.
    -   `editor` (or `editor_adapter`): Handles launching and managing the external editor process. Implements a trait defined by the `journal` module. Single responsibility: External editor interaction abstraction.
    -   `errors`: Defines custom error types (`AppError`) used across the application, facilitating consistent error handling. Single responsibility: Centralized error definitions.
    -   `main.rs`: Serves as the thin application entry point, responsible for initializing components, orchestrating the main flow by wiring modules together, and handling top-level errors.

-   **Public Interfaces / Contracts (Signature Sketches or Type Aliases)**
    -   `errors::AppError`: Custom enum or struct implementing `std::error::Error` (likely using `thiserror`).
        ```rust
        // In errors.rs
        #[derive(Debug, thiserror::Error)]
        pub enum AppError {
            #[error("Configuration error: {0}")]
            Config(#[from] ConfigError),
            #[error("I/O error: {0}")]
            Io(#[from] std::io::Error),
            #[error("Journal logic error: {0}")]
            Journal(String),
            #[error("Editor interaction error: {0}")]
            Editor(String),
            // ... other specific errors
        }
        pub type AppResult<T> = Result<T, AppError>;
        ```
    -   `config::Config`: Struct holding application settings.
        ```rust
        // In config.rs
        pub struct Config {
            pub journal_dir: PathBuf,
            pub editor_cmd: String,
            // ... other settings
        }
        impl Config {
            pub fn load() -> Result<Self, ConfigError>; // ConfigError part of AppError
        }
        ```
    -   `journal::JournalService`: Trait (or struct with public methods) for core operations.
        ```rust
        // In journal.rs
        pub trait JournalService {
            fn open_entry(&self, date_specifier: DateSpecifier) -> AppResult<PathBuf>;
            fn open_retro_entry(&self, days_ago: u32) -> AppResult<PathBuf>;
            fn open_reminisce_entry(&self) -> AppResult<Vec<PathBuf>>;
        }
        ```
    -   `journal::io::JournalIO` (or `io_adapter::JournalIO`): Trait for filesystem interactions.
        ```rust
        // In journal/io.rs or io_adapter.rs
        pub trait JournalIO {
            fn ensure_journal_dir_exists(&self, path: &Path) -> AppResult<()>;
            fn get_entry_path(&self, base_dir: &Path, date: chrono::NaiveDate) -> PathBuf;
            fn read_entry(&self, path: &Path) -> AppResult<String>;
            fn append_to_entry(&self, path: &Path, content: &str) -> AppResult<()>;
            fn entry_exists(&self, path: &Path) -> bool;
            fn create_file_if_not_exists(&self, path: &Path) -> AppResult<()>;
        }
        ```
    -   `editor::Editor` (or `editor_adapter::Editor`): Trait for editor interaction.
        ```rust
        // In editor.rs or editor_adapter.rs
        pub trait Editor {
            fn open_files(&self, editor_cmd: &str, paths: &[PathBuf]) -> AppResult<()>;
        }
        ```
    -   `cli::CliArgs`: Struct representing parsed command-line arguments (from `clap`).
        ```rust
        // In cli.rs
        #[derive(Debug, clap::Parser)]
        pub struct CliArgs { /* ... clap attributes ... */ }
        impl CliArgs {
            pub fn parse_args() -> Self;
        }
        ```

-   **Data Flow Diagram (mermaid)**
    ```mermaid
    graph TD
        User -->|CLI Input| CLI(cli::CliArgs::parse_args);
        CLI --> Main(main.rs);
        Main --> ConfigMod(config::Config::load);
        ConfigMod --Config Data--> Main;

        subgraph CoreLogic
            direction LR
            JournalSvc(journal::JournalService Impl)
            JournalIOMod(journal::io::JournalIO Impl)
            EditorMod(editor::Editor Impl)
        end

        Main --Config, Args--> JournalSvc;
        JournalSvc --Uses--> JournalIOMod;
        JournalSvc --Uses--> EditorMod;

        JournalIOMod -->|FS Ops| FileSystem[(File System)];
        EditorMod -->|Process Launch| ExtEditor[(External Editor Process)];

        JournalSvc --AppResult--> Main;
        ConfigMod --AppResult--> Main;
        Main -->|Output/Exit Code| User;

        ErrorsMod(errors::AppError) -.-> Main;
        ErrorsMod -.-> JournalSvc;
        ErrorsMod -.-> ConfigMod;
        ErrorsMod -.-> JournalIOMod;
        ErrorsMod -.-> EditorMod;
    ```

-   **Error & Edge‑Case Strategy**
    -   A unified `errors::AppError` enum (using `thiserror`) will wrap all specific error types (I/O, config, editor, etc.) and provide contextual information.
    -   All fallible operations across modules will return `Result<T, AppError>`.
    -   **Strictly forbid** `unwrap()` and `expect()` on `Result` or `Option` in any recoverable code path. Use `?` for propagation or explicit `match`/`map_err` for handling.
    -   The `main` function will be the ultimate error handler: it will match on the `AppError`, log detailed error information (including backtrace if enabled for debug builds or via config), and print a user-friendly message to `stderr` before exiting with an appropriate non-zero status code.
    -   Edge cases (e.g., journal directory not found/creatable, editor command invalid, file permissions, invalid date inputs, empty reminiscence list) will be explicitly handled and result in specific `AppError` variants.

## Detailed Build Steps

1.  **Project Setup & Module Structure**:
    -   Ensure `Cargo.toml` specifies `edition = "2021"` and a sensible Minimum Supported Rust Version (MSRV).
    -   Create the module file structure: `src/main.rs`, `src/cli.rs`, `src/config.rs`, `src/journal/mod.rs`, `src/journal/io.rs`, `src/editor.rs`, `src/errors.rs`.
    -   Add necessary dependencies to `Cargo.toml`: `clap` (with `derive` feature), `chrono`, `thiserror`, `log`, `env_logger` (or `tracing-subscriber`), `shellexpand`.
2.  **Error Handling Foundation (`errors.rs`)**:
    -   Define `AppError` enum using `thiserror`, including variants for I/O, config, journal logic, editor interaction, and argument parsing.
    -   Define `AppResult<T> = Result<T, AppError>`.
3.  **Configuration Module (`config.rs`)**:
    -   Define `Config` struct (journal dir, editor command).
    -   Implement `Config::load()`:
        -   Read `PONDER_DIR` (fallback `$HOME/Documents/ponder` or similar, use `shellexpand`).
        -   Read `EDITOR` (fallback `vim` or platform-specific sensible default).
        -   Return `Result<Config, ConfigError>` (where `ConfigError` is part of `AppError`).
4.  **I/O Adapter Trait & Implementation (`journal/io.rs`)**:
    -   Define `JournalIO` trait with methods for directory creation, path generation, file existence check, read, append, create.
    -   Implement `FilesystemJournalIO` struct that concretely implements `JournalIO` using `std::fs` and `std::path`.
    -   Ensure `PathBuf` is used for all path manipulations.
5.  **Editor Adapter Trait & Implementation (`editor.rs`)**:
    -   Define `Editor` trait with `open_files(&self, editor_cmd: &str, paths: &[PathBuf]) -> AppResult<()`.
    -   Implement `SystemEditor` struct that concretely implements `Editor` using `std::process::Command` to launch the editor. Handle command execution errors.
6.  **Core Journal Logic (`journal/mod.rs`)**:
    -   Define `DateSpecifier` enum/struct for different ways to specify a date (today, retro, specific date string).
    -   Define `JournalService` struct. It will take `Config`, `Box<dyn JournalIO>`, and `Box<dyn Editor>` as dependencies (constructor injection).
    -   Implement core methods (`open_entry`, `open_retro_entry`, `open_reminisce_entry`):
        -   Calculate target dates using `chrono`.
        -   Use `JournalIO` for path generation and file operations (e.g., `ensure_journal_dir_exists`, `create_file_if_not_exists`, `append_to_entry` for timestamp).
        -   Use `Editor` to open the file(s).
        -   Handle logic for finding reminiscence entries (e.g., random selection from past entries).
7.  **CLI Module (`cli.rs`)**:
    -   Define `CliArgs` struct using `clap::Parser` to represent commands (`new`, `retro`, `reminisce`) and their options/flags.
    -   Implement `CliArgs::parse_args()` (or let `clap` derive it).
8.  **Main Application Logic (`main.rs`)**:
    -   Initialize logging (see Logging & Observability).
    -   Call `CliArgs::parse_args()`.
    -   Call `Config::load()`. Handle potential errors.
    -   Instantiate concrete `FilesystemJournalIO` and `SystemEditor`.
    -   Instantiate `JournalService` with the loaded config and adapter implementations.
    -   Match on parsed `CliArgs` to determine which `JournalService` method to call.
    -   Handle the `AppResult` from the service:
        -   On `Ok`, print success messages if applicable.
        -   On `Err(app_error)`, log the detailed error and print a user-friendly message to `stderr`. Exit with a non-zero status.
9.  **Coding Standards & Automation Setup**:
    -   Add `rustfmt.toml` if custom formatting rules are needed (prefer defaults).
    -   Configure `clippy` lints (e.g., in `.cargo/config.toml` or via CI flags): `cargo clippy -- -D warnings -A clippy::too_many_arguments -A clippy::type_complexity` (adjust as needed).
    -   Setup CI (e.g., GitHub Actions):
        -   Run `cargo fmt --check`.
        -   Run `cargo clippy --all-targets --all-features -- -D warnings`.
        -   Run `cargo test --all-features`.
        -   Run `cargo audit` for security vulnerabilities.
        -   Run `cargo build --release`.
10. **Initial Documentation**:
    -   Add basic Rustdoc comments (`///`) to all public modules, structs, traits, enums, and functions.
    -   Create/update `README.md` with project purpose, basic usage, and setup.

## Testing Strategy

-   **Test Layers**:
    -   **Unit Tests (`#[cfg(test)]` mods within each `src` file)**:
        -   Focus on pure logic: date calculations in `journal`, config parsing variations in `config`, argument parsing in `cli` (if complex beyond `clap`'s capabilities), filename generation.
        -   No mocking of internal modules. Test public interfaces of small units.
    -   **Integration Tests (`tests/` directory)**:
        -   Test the interaction between modules, especially `JournalService` with its `JournalIO` and `Editor` dependencies.
        -   Use **test doubles (fakes/mocks)** for `JournalIO` and `Editor` traits to control external interactions and assert calls without actual filesystem changes or editor spawning.
            -   Example: A `MockJournalIO` that records paths it's asked to write to, or returns pre-set content.
            -   Example: A `MockEditor` that records which files it was asked to open.
        -   Use `tempfile::tempdir()` for tests that *do* need to interact with a temporary filesystem (e.g., testing the `FilesystemJournalIO` concrete implementation).
    -   **CLI End-to-End Tests (also in `tests/` directory)**:
        -   Use crates like `assert_cmd` and `predicates` to test the compiled binary.
        -   Verify exit codes, `stdout`, `stderr` for various CLI invocations.
        -   Interact with a temporary journal directory using `tempfile`.

-   **What to Mock (and Why)**:
    -   Mock **ONLY true external boundaries** abstracted by traits:
        -   `JournalIO`: To isolate `JournalService` logic from actual disk I/O, making tests faster, more reliable, and allowing simulation of various FS states/errors.
        -   `Editor`: To prevent actual editor processes from spawning during tests and to verify that the correct files are passed to the editor command.
    -   **DO NOT MOCK** internal Rust modules or structs directly. If testing module A requires mocking module B, it's a sign of tight coupling; refactor to use traits or improve separation.

-   **Coverage Targets & Edge‑Case Notes**:
    -   Target **>90% line coverage** for core logic (`journal`, `config`).
    -   Target **>80% overall line coverage**. Enforce via CI (e.g., using `cargo-tarpaulin` and Codecov/Coveralls).
    -   Ensure tests cover:
        -   Happy paths for all CLI commands (`new`, `retro`, `reminisce`).
        -   Error conditions: invalid config, missing editor, file permission errors, non-existent journal directory (should be created), invalid date arguments.
        -   Edge cases: no entries for reminiscence, retro to a day with an existing entry, empty journal files.

## Logging & Observability

-   **Facade**: Use the `log` crate facade.
-   **Implementation**:
    -   Development: `env_logger`, configured via `RUST_LOG` environment variable (e.g., `RUST_LOG=ponder=debug,info`).
    -   Production/CI (optional for CLI, but good practice): Consider `tracing` with `tracing-subscriber` for structured JSON logging if advanced features are needed. For a simple CLI, `env_logger` might suffice if it can output structured logs.
-   **Log Events + Structured Fields**:
    -   Application start/end: `(level: INFO, event: "app_start", version: "x.y.z")`
    -   Config loaded: `(level: DEBUG, event: "config_loaded", journal_dir: "...", editor_cmd: "...")`
    -   CLI command execution: `(level: INFO, event: "command_exec", command: "new", args: "{...}")`
    -   File operation: `(level: DEBUG, event: "file_opened", path: "...", mode: "append")`
    -   Editor launched: `(level: DEBUG, event: "editor_launched", command: "vim /path/to/file")`
    -   Errors: `(level: ERROR, event: "error_occurred", error_type: "AppError::Io", message: "...", details: "...", source_error: "...", backtrace: "...")` (backtrace optional/conditional).
-   **Standard Fields**: `timestamp`, `level`, `target` (module path), `message`. Add custom fields as relevant per event.
-   **Correlation ID Propagation**: Not typically critical for a single-invocation CLI. If Ponder were to evolve into a longer-running process or service, this would be important.

## Security & Config

-   **Input Validation Hotspots**:
    -   `cli`: `clap` handles basic type validation. Add custom validators for specific argument formats if needed (e.g., date strings if not using `chrono` parsing directly).
    -   `config`: Validate paths from environment variables (e.g., ensure `journal_dir` is a creatable/writable path). Sanitize or validate editor command string if it's constructed dynamically (prefer fixed strings from config).
    -   Use `shellexpand` for environment variables like `$HOME` in paths to handle them correctly.
-   **Secrets Handling**:
    -   Currently, Ponder does not handle secrets.
    -   If secrets were introduced (e.g., for encryption keys), they must **NEVER** be hardcoded. Load from environment variables, dedicated secrets management tools, or secure OS keychains. Do not log secrets.
-   **Least-Privilege Notes**:
    -   The application runs with the user's privileges.
    -   File operations are scoped to the configured `journal_dir`. Ensure directory creation and file writes use standard, safe APIs (e.g., `std::fs::create_dir_all`, `OpenOptions::append`).
    -   Be cautious if parsing external input to form parts of commands (e.g., editor command); prefer pre-defined commands.

## Documentation

-   **Code Self‑Doc Patterns**:
    -   Comprehensive Rustdoc comments (`///`) for all public modules, structs, enums, traits, functions, and significant `pub(crate)` items. Explain purpose, parameters, return values, and potential errors (`# Errors` section).
    -   Include runnable examples in Rustdoc where appropriate to demonstrate API usage.
    -   Use inline comments (`//`) sparingly to explain *why* non-obvious code exists, not *what* it does (which should be clear from the code itself).
    -   Use meaningful names for variables, functions, types, and modules.
-   **Required Readme or OpenAPI Updates**:
    -   **`README.md`**:
        -   Detailed project description and purpose.
        -   Features list.
        -   Installation instructions (from source, from crates.io if published). Include MSRV.
        -   Comprehensive Usage section with examples for all CLI commands and options.
        -   Configuration details: environment variables (`PONDER_DIR`, `EDITOR`), default values, how to customize.
        -   Brief architecture overview.
        -   Contributing guidelines (link to `CONTRIBUTING.md`).
        -   License information.
    -   **`CONTRIBUTING.md`**:
        -   Development setup (required tools: Rust MSRV, `cargo-audit`, `cargo-tarpaulin`).
        -   Code style (mention `rustfmt` and `clippy` usage, link to `rustfmt.toml` if it exists).
        -   Testing procedures (how to run tests, coverage expectations).
        -   Pull Request process.
    -   No OpenAPI specification needed as this is a CLI application, not an HTTP API.

## Risk Matrix

| Risk                                                 | Severity | Mitigation                                                                                                                                                              |
| :--------------------------------------------------- | :------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Incomplete or incorrect error propagation/handling   | High     | Consistent use of `AppResult<T>` and `thiserror`; `main` as central handler; strict `clippy` lints (`-D warnings`, no `unwrap`); thorough testing of error paths.      |
| High module coupling despite refactoring efforts     | High     | Strict adherence to trait-based dependency injection for external I/O (filesystem, editor); code reviews focusing on dependency boundaries and module responsibilities. |
| Test coverage gaps, especially for edge cases        | High     | Define clear test cases for known edge conditions; enforce >80-90% coverage in CI; use `assert_cmd` for CLI behavior; use fakes/mocks for isolating units.        |
| Brittle configuration loading or unclear precedence  | Medium   | Centralize config logic in `config` module; clear documentation of env vars and fallbacks; unit tests for config loading scenarios.                                     |
| Filesystem interaction edge cases (permissions, etc.)| Medium   | Use `PathBuf` and standard `std::fs` operations robustly; test `FilesystemJournalIO` with `tempfile` under various (simulated, if needed) conditions.               |
| Editor launching inconsistencies across platforms    | Medium   | Use `std::process::Command` carefully; clear documentation on `EDITOR` var; provide sensible fallbacks; test on target platforms if feasible.                          |
| Introduction of vulnerable dependencies              | Medium   | Integrate `cargo audit` into CI pipeline and fail on high/critical vulnerabilities; regularly update dependencies using `cargo update`.                               |
| Over-engineering with premature abstractions         | Medium   | Start with necessary abstractions (I/O, Editor traits). Defer more complex abstractions until clear needs arise. Focus on YAGNI for initial refactor.                |

## Open Questions

-   **Minimum Supported Rust Version (MSRV)**: What MSRV should be targeted and enforced in CI? (e.g., latest stable minus 2-3 versions, or a specific LTS version if applicable).
-   **Extended Configuration**: Should Ponder support a configuration file (e.g., TOML in XDG config dir) in addition to environment variables? (Recommendation: Start with env vars for simplicity, consider config file as a future enhancement if requested).
-   **Date/Time Formatting**: Are there specific requirements for date/time formats in journal entries or filenames beyond current `chrono` defaults or simple `YYYY-MM-DD`?
-   **Editor Fallback**: What is the desired behavior if `$EDITOR` is not set and the default fallback (e.g., `vim`) is also not found? (e.g., error out with a clear message, attempt a more universal fallback like `nano` or `vi`).
-   **Reminiscence Logic**: Specifics of "reminisce" logic (e.g., how are old entries chosen? Randomly? Specific intervals?).