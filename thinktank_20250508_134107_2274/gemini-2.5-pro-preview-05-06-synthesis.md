# Todo

## Project Setup
- [ ] **T001 · Chore · P0: initialize module file structure and update Cargo.toml**
    - **Context:** PLAN.md, Project Setup & Module Structure
    - **Action:**
        1. Ensure `Cargo.toml` has `edition = "2021"` and a placeholder comment for MSRV.
        2. Create `src/main.rs`, `src/cli.rs`, `src/config.rs`, `src/journal/mod.rs`, `src/journal/io.rs`, `src/editor.rs`, `src/errors.rs`.
        3. Add dependencies to `Cargo.toml`: `clap` (with `derive`), `chrono`, `thiserror`, `log`, `env_logger`, `shellexpand`.
    - **Done‑when:**
        1. All specified files and directories exist with basic module declarations.
        2. `Cargo.toml` includes required dependencies and the correct Rust edition.
        3. `cargo build` and `cargo check` pass without errors.
    - **Verification:**
        1. `cargo build` completes successfully.
    - **Depends‑on:** none

## Errors
- [ ] **T002 · Feature · P0: define AppError enum and AppResult type**
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
- [ ] **T003 · Feature · P1: define Config struct and implement Config::load()**
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
- [ ] **T004 · Feature · P1: define JournalIO trait in journal/io.rs**
    - **Context:** PLAN.md, I/O Adapter Trait & Implementation
    - **Action:**
        1. Specify the `JournalIO` trait in `src/journal/io.rs` with methods for directory creation, path generation, file existence checks, file reading, appending to files, and file creation.
        2. Ensure all fallible methods in the trait return `AppResult`.
    - **Done‑when:**
        1. `JournalIO` trait is defined with all specified methods and compiles successfully.
    - **Depends‑on:** [T002]
- [ ] **T005 · Feature · P1: implement FilesystemJournalIO struct for JournalIO trait**
    - **Context:** PLAN.md, I/O Adapter Trait & Implementation
    - **Action:**
        1. Implement the `FilesystemJournalIO` struct in `src/journal/io.rs`.
        2. Implement the `JournalIO` trait for `FilesystemJournalIO` using `std::fs` and `std::path` for actual filesystem operations.
        3. Ensure all file and directory operations robustly handle potential errors, converting them to `AppError`.
    - **Done‑when:**
        1. All `JournalIO` trait methods are implemented for `FilesystemJournalIO`.
    - **Depends‑on:** [T004]
- [ ] **T006 · Test · P1: test FilesystemJournalIO implementation**
    - **Context:** PLAN.md, Testing Strategy (Integration Tests for IO)
    - **Action:**
        1. Write integration tests for `FilesystemJournalIO` using the `tempfile` crate to