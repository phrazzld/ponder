# Todo

## Errors Module
- [ ] **T001 · Chore · P2: Define custom error types for Ponder CLI application**
    - **Context:** Error Handling Foundation (`errors.rs`)
    - **Action:**
        1. Define `AppError` enum using `thiserror` with variants for I/O, config, journal logic, editor interaction, and argument parsing.
        2. Define `AppResult<T> = Result<T, AppError>`.
    - **Done‑when:**
        1. `AppError` enum is defined with required variants.
        2. `AppResult<T>` type alias is defined.
    - **Depends‑on:** none

## Configuration Module
- [ ] **T002 · Feature · P2: Implement configuration loading for Ponder CLI application**
    - **Context:** Configuration Module (`config.rs`)
    - **Action:**
        1. Define `Config` struct with fields for journal directory and editor command.
        2. Implement `Config::load()` method to read configuration from environment variables.
    - **Done‑when:**
        1. `Config` struct is defined with required fields.
        2. `Config::load()` method is implemented and returns `Result<Config, ConfigError>`.
    - **Depends‑on:** T001

## I/O Adapter Trait & Implementation
- [ ] **T003 · Refactor · P2: Define JournalIO trait for filesystem interactions**
    - **Context:** I/O Adapter Trait & Implementation (`journal/io.rs`)
    - **Action:**
        1. Define `JournalIO` trait with methods for directory creation, path generation, file existence check, read, append, and create.
    - **Done‑when:**
        1. `JournalIO` trait is defined with required methods.
    - **Depends‑on:** none
- [ ] **T004 · Refactor · P2: Implement FilesystemJournalIO struct**
    - **Context:** I/O Adapter Trait & Implementation (`journal/io.rs`)
    - **Action:**
        1. Implement `FilesystemJournalIO` struct that concretely implements `JournalIO` using `std::fs` and `std::path`.
    - **Done‑when:**
        1. `FilesystemJournalIO` struct is implemented and satisfies `JournalIO` trait.
    - **Depends‑on:** T003

## Editor Adapter Trait & Implementation
- [ ] **T005 · Refactor · P2: Define Editor trait for editor interactions**
    - **Context:** Editor Adapter Trait & Implementation (`editor.rs`)
    - **Action:**
        1. Define `Editor` trait with `open_files` method.
    - **Done‑when:**
        1. `Editor` trait is defined with `open_files` method.
    - **Depends‑on:** none
- [ ] **T006 · Refactor · P2: Implement SystemEditor struct**
    - **Context:** Editor Adapter Trait & Implementation (`editor.rs`)
    - **Action:**
        1. Implement `SystemEditor` struct that concretely implements `Editor` using `std::process::Command`.
    - **Done‑when:**
        1. `SystemEditor` struct is implemented and satisfies `Editor` trait.
    - **Depends‑on:** T005

## Core Journal Logic
- [ ] **T007 · Feature · P2: Implement JournalService struct**
    - **Context:** Core Journal Logic (`journal/mod.rs`)
    - **Action:**
        1. Define `JournalService` struct with dependencies on `Config`, `JournalIO`, and `Editor`.
        2. Implement core methods (`open_entry`, `open_retro_entry`, `open_reminisce_entry`).
    - **Done‑when:**
        1. `JournalService` struct is defined and implemented with required methods.
    - **Depends‑on:** T002, T003, T005

## CLI Module
- [ ] **T008 · Feature · P2: Implement CLI argument parsing using clap**
    - **Context:** CLI Module (`cli.rs`)
    - **Action:**
        1. Define `CliArgs` struct using `clap::Parser`.
        2. Implement `CliArgs::parse_args()` method.
    - **Done‑when:**
        1. `CliArgs` struct is defined and parses CLI arguments correctly.
    - **Depends‑on:** none

## Main Application Logic
- [ ] **T009 · Chore · P2: Implement main application logic in main.rs**
    - **Context:** Main Application Logic (`main.rs`)
    - **Action:**
        1. Initialize logging.
        2. Parse CLI arguments using `CliArgs`.
        3. Load configuration using `Config`.
        4. Instantiate `JournalService` with required dependencies.
        5. Handle `AppResult` from `JournalService` methods and log errors.
    - **Done‑when:**
        1. Main application logic is implemented and orchestrates the required components.
    - **Depends‑on:** T001, T002, T007, T008

## Testing Strategy
- [ ] **T010 · Test · P2: Write unit tests for pure logic in journal, config, and cli modules**
    - **Context:** Testing Strategy
    - **Action:**
        1. Write unit tests for date calculations, config parsing, and argument parsing.
    - **Done‑when:**
        1. Unit tests are written for the specified modules.
    - **Depends‑on:** T007, T002, T008
- [ ] **T011 · Test · P2: Write integration tests for JournalService with test doubles**
    - **Context:** Testing Strategy
    - **Action:**
        1. Implement test doubles (fakes/mocks) for `JournalIO` and `Editor` traits.
        2. Write integration tests for `JournalService` using test doubles.
    - **Done‑when:**
        1. Integration tests are written for `JournalService`.
    - **Depends‑on:** T007, T003, T005
- [ ] **T012 · Test · P2: Write CLI end-to-end tests using assert_cmd and predicates**
    - **Context:** Testing Strategy
    - **Action:**
        1. Write end-to-end tests for CLI commands using `assert_cmd` and `predicates`.
    - **Done‑when:**
        1. End-to-end tests are written for CLI commands.
    - **Depends‑on:** T008, T009

## Logging & Observability
- [ ] **T013 · Chore · P2: Implement logging using log crate and env_logger**
    - **Context:** Logging & Observability
    - **Action:**
        1. Configure `env_logger` for development.
        2. Implement logging events with structured fields.
    - **Done‑when:**
        1. Logging is implemented and configured correctly.
    - **Depends‑on:** none

## Security & Config
- [ ] **T014 · Chore · P2: Validate input and handle secrets appropriately**
    - **Context:** Security & Config
    - **Action:**
        1. Validate CLI arguments and configuration inputs.
        2. Ensure secrets are handled correctly (currently, no secrets are handled).
    - **Done‑when:**
        1. Input validation is implemented.
        2. Secrets handling is reviewed and documented.
    - **Depends‑on:** T008, T002

## Documentation
- [ ] **T015 · Chore · P2: Update README.md with project description, usage, and configuration details**
    - **Context:** Documentation
    - **Action:**
        1. Update `README.md` with required information.
    - **Done‑when:**
        1. `README.md` is updated with project description, usage, and configuration details.
    - **Depends‑on:** T009

### Clarifications & Assumptions
- [ ] **Issue: Minimum Supported Rust Version (MSRV) for Ponder CLI application**
    - **Context:** Risk Matrix
    - **Blocking?:** yes
- [ ] **Issue: Should Ponder support a configuration file in addition to environment variables?**
    - **Context:** Open Questions
    - **Blocking?:** no
- [ ] **Issue: Specific requirements for date/time formats in journal entries or filenames**
    - **Context:** Open Questions
    - **Blocking?:** no
- [ ] **Issue: Desired behavior if $EDITOR is not set and the default fallback is not found**
    - **Context:** Open Questions
    - **Blocking?:** no
- [ ] **Issue: Specifics of "reminisce" logic for choosing old entries**
    - **Context:** Open Questions
    - **Blocking?:** no