# Todo

## Project Setup
- [x] **T001 · Chore · P0: Initialize module file structure and update Cargo.toml**
    - **Context:** PLAN.md, Project Setup & Module Structure
    - **Action:**
        1. Ensure `Cargo.toml` has `edition = "2021"` and a sensible Minimum Supported Rust Version (MSRV).
        2. Create `src/main.rs`, `src/cli.rs`, `src/config.rs`, `src/journal/mod.rs`, `src/journal/io.rs`, `src/editor.rs`, `src/errors.rs`.
        3. Add dependencies to `Cargo.toml`: `clap` (with `derive`), `chrono`, `thiserror`, `log`, `env_logger`, `shellexpand`.
    - **Done‑when:**
        1. All specified files and directories exist with basic module declarations.
        2. `Cargo.toml` includes required dependencies and the correct Rust edition.
        3. `cargo build` and `cargo check` pass without errors.
    - **Verification:**
        1. `cargo build` completes successfully.
    - **Depends‑on:** none

## Error Handling Foundation
- [x] **T002 · Feature · P0: Define AppError enum and AppResult type**
    - **Context:** PLAN.md, Error Handling Foundation
    - **Action:**
        1. Implement `AppError` enum in `src/errors.rs` using `thiserror`, with variants for I/O, config, journal logic, editor interaction, and argument parsing errors.
        2. Define the type alias `AppResult<T> = Result<T, AppError>`.
    - **Done‑when:**
        1. `AppError` enum is defined, covering all specified error sources, and implements `std::error::Error`.
        2. `AppResult<T>` type alias is defined and usable.
        3. Code compiles and `AppError` can be imported and used in other modules.
    - **Verification:**
        1. Unit test: demonstrate conversion of representative underlying error types (e.g., `std::io::Error`) to `AppError` variants.
    - **Depends‑on:** [T001]

## Configuration
- [x] **T003 · Feature · P1: Define Config struct and implement Config::load()**
    - **Context:** PLAN.md, Configuration Module
    - **Action:**
        1. Define `Config` struct in `src/config.rs` with fields like `journal_dir: PathBuf` and `editor_cmd: String`.
        2. Implement `Config::load()` method to read configuration from environment variables (e.g., `PONDER_DIR`, `EDITOR`), apply sensible fallbacks (using `shellexpand` for directory paths), and perform basic validation.
        3. Define a `ConfigError` type (e.g., an enum variant within `AppError` or a dedicated error type convertible to `AppError`) for configuration-specific errors.
    - **Done‑when:**
        1. `Config` struct is defined with all necessary fields.
        2. `Config::load()` correctly returns a `Config` instance or an `AppError` for all tested scenarios.
    - **Verification:**
        1. Unit tests: verify `Config::load()` behavior with environment variables set, unset, and set to invalid values.
    - **Depends‑on:** [T002]

## Journal I/O
- [x] **T004 · Feature · P1: Define JournalIO trait in journal/io.rs**
    - **Context:** PLAN.md, I/O Adapter Trait & Implementation
    - **Action:**
        1. Specify the `JournalIO` trait in `src/journal/io.rs` with methods for directory creation, path generation, file existence checks, file reading, appending to files, and file creation.
        2. Ensure all fallible methods in the trait return `AppResult`.
    - **Done‑when:**
        1. `JournalIO` trait is defined with all specified methods and compiles successfully.
    - **Verification:**
        1. All methods accept and return correct types.
    - **Depends‑on:** [T002]

- [x] **T005 · Feature · P1: Implement FilesystemJournalIO struct for JournalIO trait**
    - **Context:** PLAN.md, I/O Adapter Trait & Implementation
    - **Action:**
        1. Implement the `FilesystemJournalIO` struct in `src/journal/io.rs`.
        2. Implement the `JournalIO` trait for `FilesystemJournalIO` using `std::fs` and `std::path` for actual filesystem operations.
        3. Ensure all file and directory operations robustly handle potential errors, converting them to `AppError`.
    - **Done‑when:**
        1. All `JournalIO` trait methods are implemented for `FilesystemJournalIO`.
        2. Implementation handles common edge cases like missing directories.
    - **Verification:**
        1. Unit or integration tests: create/read/append/check/ensure files/dirs using temporary directories.
    - **Depends‑on:** [T004]

## Editor
- [x] **T006 · Feature · P1: Define Editor trait in editor.rs**
    - **Context:** PLAN.md, Editor Adapter Trait & Implementation
    - **Action:**
        1. Specify the `Editor` trait in `src/editor.rs` with the method `open_files(&self, editor_cmd: &str, paths: &[PathBuf]) -> AppResult<()>`.
    - **Done‑when:**
        1. `Editor` trait is defined and compiles successfully.
    - **Verification:**
        1. Trait signature covers the required abstraction.
    - **Depends‑on:** [T002]

- [x] **T007 · Feature · P1: Implement SystemEditor struct for Editor trait**
    - **Context:** PLAN.md, Editor Adapter Trait & Implementation
    - **Action:**
        1. Implement the `SystemEditor` struct in `src/editor.rs`.
        2. Implement the `Editor` trait for `SystemEditor` using `std::process::Command` to launch the editor.
        3. Ensure proper error handling for command execution failures.
    - **Done‑when:**
        1. `SystemEditor` can successfully launch the editor with specified files or return appropriate errors.
    - **Verification:**
        1. Unit test with mock or dry-run command.
    - **Depends‑on:** [T006]

## Journal Core
- [ ] **T008 · Feature · P0: Define DateSpecifier type in journal/mod.rs**
    - **Context:** PLAN.md, Core Journal Logic
    - **Action:**
        1. Define an enum or struct in `src/journal/mod.rs` to represent different ways of specifying a date (today, retro, specific date string).
    - **Done‑when:**
        1. `DateSpecifier` type is defined and can be used in journal logic.
    - **Verification:**
        1. Unit test: construct from typical CLI inputs.
    - **Depends‑on:** [T001]

- [ ] **T009 · Feature · P0: Define JournalService struct and constructor**
    - **Context:** PLAN.md, Core Journal Logic
    - **Action:**
        1. Implement `JournalService` struct in `src/journal/mod.rs` that takes `Config`, `Box<dyn JournalIO>`, and `Box<dyn Editor>` as dependencies.
        2. Provide a constructor that enforces dependency injection.
    - **Done‑when:**
        1. `JournalService` struct is defined with dependencies properly injected.
    - **Verification:**
        1. Unit test: construct with mock dependencies.
    - **Depends‑on:** [T003, T004, T006, T008]

- [ ] **T010 · Feature · P1: Implement core JournalService methods**
    - **Context:** PLAN.md, Core Journal Logic
    - **Action:**
        1. Implement `open_entry`, `open_retro_entry`, and `open_reminisce_entry` methods for `JournalService`.
        2. Use `chrono` for date calculations, `JournalIO` for file operations, and `Editor` for opening files.
    - **Done‑when:**
        1. All core methods are implemented and handle expected use cases.
    - **Verification:**
        1. Unit tests with mocked `JournalIO` and `Editor` dependencies.
        2. Tests cover happy paths and error conditions.
    - **Depends‑on:** [T009, T005, T007]

## CLI
- [x] **T011 · Feature · P1: Define CliArgs struct using clap**
    - **Context:** PLAN.md, CLI Module
    - **Action:**
        1. Define `CliArgs` struct in `src/cli.rs` using `clap::Parser`.
        2. Add CLI commands and options (new, retro, reminisce) with appropriate descriptions.
    - **Done‑when:**
        1. `CliArgs` struct is defined with all needed CLI options and compiles successfully.
    - **Verification:**
        1. Unit test: parse example command-line arguments.
    - **Depends‑on:** [T001]

- [x] **T012 · Feature · P1: Implement CliArgs::parse_args()**
    - **Context:** PLAN.md, CLI Module
    - **Action:**
        1. Implement `CliArgs::parse_args()` method (or let clap derive it).
        2. Add any custom validation or transformation of parsed arguments.
    - **Done‑when:**
        1. Calling `CliArgs::parse_args()` produces expected results for valid inputs and errors for invalid ones.
    - **Verification:**
        1. Unit tests for valid and invalid argument combinations.
    - **Depends‑on:** [T011]

## Main Application
- [x] **T013 · Feature · P0: Initialize logging in main.rs**
    - **Context:** PLAN.md, Main Application Logic, Logging & Observability
    - **Action:**
        1. Set up logging initialization in `src/main.rs` using `env_logger` or similar.
        2. Configure log level based on `RUST_LOG` environment variable.
    - **Done‑when:**
        1. Logging is initialized and can be used throughout the application.
    - **Verification:**
        1. Verify log output with different log levels.
    - **Depends‑on:** [T001]

- [ ] **T014 · Feature · P0: Implement main application flow**
    - **Context:** PLAN.md, Main Application Logic
    - **Action:**
        1. In `src/main.rs`, implement the main function to:
           - Parse CLI arguments
           - Load configuration
           - Instantiate required components (`FilesystemJournalIO`, `SystemEditor`, `JournalService`)
           - Call appropriate `JournalService` methods based on CLI args
           - Handle success/error results
    - **Done‑when:**
        1. `main` function orchestrates the entire application flow.
        2. Error handling is robust and user-friendly.
    - **Verification:**
        1. Integration tests for full application flow.
        2. Test error handling paths.
    - **Depends‑on:** [T012, T010, T013]

## Testing
- [ ] **T015 · Test · P1: Write unit tests for each module**
    - **Context:** PLAN.md, Testing Strategy
    - **Action:**
        1. Add `#[cfg(test)] mod tests` to each module with unit tests.
        2. Focus on testing pure logic and module-specific functionality.
    - **Done‑when:**
        1. Each module has comprehensive unit tests.
        2. All tests pass.
    - **Verification:**
        1. `cargo test` passes for all unit tests.
    - **Depends‑on:** [T003, T005, T007, T010, T012]

- [ ] **T016 · Test · P1: Write integration tests**
    - **Context:** PLAN.md, Testing Strategy
    - **Action:**
        1. Create tests directory with integration tests.
        2. Test interactions between modules, especially for `JournalService` with its dependencies.
        3. Test CLI behavior using `assert_cmd` and similar crates.
    - **Done‑when:**
        1. Integration tests cover key application flows.
        2. All tests pass.
    - **Verification:**
        1. `cargo test --test '*'` passes for all integration tests.
    - **Depends‑on:** [T014, T015]

## Documentation
- [ ] **T017 · Doc · P2: Add Rustdoc comments**
    - **Context:** PLAN.md, Documentation, Code Self-Doc Patterns
    - **Action:**
        1. Add comprehensive Rustdoc comments (`///`) to all public items.
        2. Include examples where appropriate.
        3. Document error conditions in the `# Errors` section.
    - **Done‑when:**
        1. All public items have documentation.
        2. Documentation is informative and clear.
    - **Verification:**
        1. `cargo doc --no-deps` builds without warnings.
    - **Depends‑on:** [T014]

- [ ] **T018 · Doc · P2: Update README.md**
    - **Context:** PLAN.md, Documentation, Required Readme Updates
    - **Action:**
        1. Update `README.md` with detailed project description, features, installation instructions, usage examples, configuration details, and architecture overview.
    - **Done‑when:**
        1. `README.md` contains comprehensive information about the project.
    - **Verification:**
        1. README renders correctly on GitHub.
    - **Depends‑on:** [T014]

## Automation Setup
- [ ] **T019 · CI · P2: Configure rustfmt**
    - **Context:** PLAN.md, Coding Standards & Automation Setup
    - **Action:**
        1. Add `rustfmt.toml` if needed (prefer defaults).
        2. Ensure all code is formatted according to `rustfmt` guidelines.
    - **Done‑when:**
        1. `cargo fmt --check` passes.
    - **Verification:**
        1. All code is consistently formatted.
    - **Depends‑on:** [T014]

- [ ] **T020 · CI · P2: Configure clippy**
    - **Context:** PLAN.md, Coding Standards & Automation Setup
    - **Action:**
        1. Configure `clippy` lints (e.g., via `.cargo/config.toml`).
        2. Fix any clippy warnings in the codebase.
    - **Done‑when:**
        1. `cargo clippy --all-targets --all-features -- -D warnings` passes.
    - **Verification:**
        1. No clippy warnings in the codebase.
    - **Depends‑on:** [T014]