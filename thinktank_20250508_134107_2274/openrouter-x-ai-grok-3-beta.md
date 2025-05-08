# Todo

## Project Setup & Module Structure
- [ ] **T001 · Chore · P2: Set up project structure and dependencies**
    - **Context:** Project Setup & Module Structure (PLAN.md)
    - **Action:**
        1. Update `Cargo.toml` to specify `edition = "2021"` and define MSRV.
        2. Create module file structure: `src/main.rs`, `src/cli.rs`, `src/config.rs`, `src/journal/mod.rs`, `src/journal/io.rs`, `src/editor.rs`, `src/errors.rs`.
        3. Add dependencies to `Cargo.toml`: `clap` (with `derive`), `chrono`, `thiserror`, `log`, `env_logger`, `shellexpand`.
    - **Done-when:**
        1. Project structure is created as specified.
        2. Dependencies are added and `Cargo.toml` is updated.
    - **Depends-on:** none

## Error Handling
- [ ] **T002 · Feature · P1: Define centralized error handling with AppError**
    - **Context:** Error Handling Foundation (PLAN.md)
    - **Action:**
        1. Create `errors.rs` with `AppError` enum using `thiserror`.
        2. Include variants for I/O, config, journal logic, editor interaction, and argument parsing.
        3. Define `AppResult<T> = Result<T, AppError>`.
    - **Done-when:**
        1. `AppError` enum is defined with all necessary variants.
        2. `AppResult<T>` type alias is created and used consistently.
    - **Depends-on:** [T001]

## Configuration Management
- [ ] **T003 · Feature · P2: Implement configuration loading in config module**
    - **Context:** Configuration Module (PLAN.md)
    - **Action:**
        1. Define `Config` struct in `config.rs` for journal dir and editor command.
        2. Implement `Config::load()` to read `PONDER_DIR` and `EDITOR` with fallbacks.
        3. Return `Result<Config, ConfigError>` mapped to `AppError`.
    - **Done-when:**
        1. `Config` struct is defined with required fields.
        2. `Config::load()` handles environment variables and fallbacks correctly.
        3. Unit tests pass for config loading scenarios.
    - **Depends-on:** [T002]

## I/O Adapter
- [ ] **T004 · Feature · P2: Define and implement JournalIO trait for filesystem operations**
    - **Context:** I/O Adapter Trait & Implementation (PLAN.md)
    - **Action:**
        1. Define `JournalIO` trait in `journal/io.rs` with methods for file and directory operations.
        2. Implement `FilesystemJournalIO` struct to use `std::fs` and `std::path`.
        3. Use `PathBuf` for all path manipulations.
    - **Done-when:**
        1. `JournalIO` trait is defined with specified methods.
        2. `FilesystemJournalIO` implements the trait using standard filesystem operations.
        3. Unit tests verify correct path handling and error propagation.
    - **Depends-on:** [T002]

## Editor Adapter
- [ ] **T005 · Feature · P2: Define and implement Editor trait for editor interaction**
    - **Context:** Editor Adapter Trait & Implementation (PLAN.md)
    - **Action:**
        1. Define `Editor` trait in `editor.rs` with `open_files` method.
        2. Implement `SystemEditor` struct to launch editor using `std::process::Command`.
        3. Handle command execution errors with `AppResult`.
    - **Done-when:**
        1. `Editor` trait is defined with the correct method signature.
        2. `SystemEditor` implements the trait and handles errors appropriately.
        3. Unit tests verify editor command execution and error cases.
    - **Depends-on:** [T002]

## Core Journal Logic
- [ ] **T006 · Feature · P1: Implement core journal logic in JournalService**
    - **Context:** Core Journal Logic (PLAN.md)
    - **Action:**
        1. Define `DateSpecifier` enum in `journal/mod.rs` for date handling.
        2. Define `JournalService` struct with dependencies on `Config`, `JournalIO`, and `Editor`.
        3. Implement methods `open_entry`, `open_retro_entry`, and `open_reminisce_entry` using `chrono` and dependencies.
    - **Done-when:**
        1. `JournalService` is defined and implements core functionality.
        2. Methods handle date calculations, file operations, and editor launching.
        3. Unit tests cover happy paths and edge cases for date logic.
    - **Depends-on:** [T003, T004, T005]

## CLI Parsing
- [ ] **T007 · Feature · P2: Implement CLI argument parsing with clap**
    - **Context:** CLI Module (PLAN.md)
    - **Action:**
        1. Define `CliArgs` struct in `cli.rs` using `clap::Parser` for commands and options.
        2. Implement or derive `CliArgs::parse_args()` for argument parsing.
    - **Done-when:**
        1. `CliArgs` struct is defined with all necessary commands and options.
        2. Parsing logic handles user input correctly.
        3. Unit tests verify correct parsing of CLI arguments.
    - **Depends-on:** [T001]

## Main Application Logic
- [ ] **T008 · Feature · P1: Wire up main application logic in main.rs**
    - **Context:** Main Application Logic (PLAN.md)
    - **Action:**
        1. Initialize logging in `main.rs`.
        2. Parse CLI args, load config, instantiate adapters, and create `JournalService`.
        3. Match CLI args to call appropriate `JournalService` method and handle results.
    - **Done-when:**
        1. Main function initializes all components and orchestrates flow.
        2. Error handling in `main` logs details and provides user-friendly messages.
        3. Application exits with appropriate status codes.
    - **Depends-on:** [T003, T006, T007]

## Coding Standards & Automation
- [ ] **T009 · Chore · P2: Set up coding standards and CI automation**
    - **Context:** Coding Standards & Automation Setup (PLAN.md)
    - **Action:**
        1. Add `rustfmt.toml` if custom formatting is needed.
        2. Configure `clippy` lints in CI or `.cargo/config.toml`.
        3. Set up CI pipeline for formatting, linting, testing, auditing, and building.
    - **Done-when:**
        1. CI configuration files are created and enforce standards.
        2. Automated checks pass for formatting, linting, and testing.
    - **Depends-on:** [T001]

## Documentation
- [ ] **T010 · Chore · P2: Add initial project documentation**
    - **Context:** Initial Documentation (PLAN.md)
    - **Action:**
        1. Add Rustdoc comments to all public APIs.
        2. Update `README.md` with project details, usage, and configuration.
    - **Done-when:**
        1. All public modules and functions have Rustdoc comments.
        2. `README.md` contains comprehensive project information.
    - **Depends-on:** [T008]

## Testing - Unit Tests
- [ ] **T011 · Test · P1: Write unit tests for core logic modules**
    - **Context:** Testing Strategy - Unit Tests (PLAN.md)
    - **Action:**
        1. Write unit tests for `journal`, `config`, and `cli` modules.
        2. Focus on pure logic like date calculations and config parsing.
    - **Done-when:**
        1. Unit tests cover core logic with >90% line coverage for `journal` and `config`.
        2. Tests pass in CI environment.
    - **Depends-on:** [T003, T006, T007]

## Testing - Integration Tests
- [ ] **T012 · Test · P1: Write integration tests for module interactions**
    - **Context:** Testing Strategy - Integration Tests (PLAN.md)
    - **Action:**
        1. Write integration tests for `JournalService` with test doubles for `JournalIO` and `Editor`.
        2. Use `tempfile` for filesystem interaction tests where needed.
    - **Done-when:**
        1. Integration tests verify interactions between modules.
        2. Test doubles simulate external dependencies effectively.
        3. Tests pass in CI environment.
    - **Depends-on:** [T006]

## Testing - CLI End-to-End Tests
- [ ] **T013 · Test · P1: Write end-to-end tests for CLI behavior**
    - **Context:** Testing Strategy - CLI End-to-End Tests (PLAN.md)
    - **Action:**
        1. Use `assert_cmd` and `predicates` to test compiled binary behavior.
        2. Verify exit codes, `stdout`, and `stderr` for various CLI scenarios.
    - **Done-when:**
        1. E2E tests cover all CLI commands and key error paths.
        2. Tests pass in CI environment.
    - **Verification:**
        1. Manually run binary with test scenarios to confirm behavior matches test expectations.
    - **Depends-on:** [T008]

## Logging & Observability
- [ ] **T014 · Feature · P2: Implement structured logging with log crate**
    - **Context:** Logging & Observability (PLAN.md)
    - **Action:**
        1. Set up `log` crate with `env_logger` for development.
        2. Add log events with structured fields for app start, config load, CLI execution, file ops, and errors.
    - **Done-when:**
        1. Logging is initialized in `main.rs` and used across modules.
        2. Logs include standard fields and custom event data.
    - **Depends-on:** [T008]

## Security & Config Validation
- [ ] **T015 · Feature · P2: Implement input validation for CLI and config**
    - **Context:** Security & Config - Input Validation Hotspots (PLAN.md)
    - **Action:**
        1. Add custom validators in `cli.rs` for argument formats if needed.
        2. Validate paths and editor command in `config.rs` for safety.
    - **Done-when:**
        1. Input validation prevents invalid or unsafe data from being processed.
        2. Unit tests cover validation scenarios.
    - **Depends-on:** [T003, T007]

### Clarifications & Assumptions
- [ ] **Issue:** Minimum Supported Rust Version (MSRV) not specified**
    - **Context:** Open Questions - MSRV (PLAN.md)
    - **Blocking?:** no
- [ ] **Issue:** Support for configuration file in addition to env vars unclear**
    - **Context:** Open Questions - Extended Configuration (PLAN.md)
    - **Blocking?:** no
- [ ] **Issue:** Specific date/time formats for journal entries or filenames undefined**
    - **Context:** Open Questions - Date/Time Formatting (PLAN.md)
    - **Blocking?:** no
- [ ] **Issue:** Behavior for editor fallback if EDITOR not set or default not found unclear**
    - **Context:** Open Questions - Editor Fallback (PLAN.md)
    - **Blocking?:** no
- [ ] **Issue:** Specifics of reminiscence logic for selecting old entries undefined**
    - **Context:** Open Questions - Reminiscence Logic (PLAN.md)
    - **Blocking?:** no