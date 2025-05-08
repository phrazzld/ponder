# Todo

## Errors
- [ ] **T001 · Refactor · P2: Define AppError enum**
    - **Context:** Detailed Build Steps / 2. Error Handling Foundation
    - **Action:**
        1. Create `src/errors.rs` and define `AppError` enum using `thiserror`, including variants for I/O, config, journal, editor.
        2. Define `AppResult<T> = Result<T, AppError>`.
    - **Done‑when:**
        1. `AppError` compiles and implements `std::error::Error`.
        2. Error variants cover I/O, config, journal, editor, and general cases.
    - **Verification:**
        1. Run `cargo check` to confirm no compilation errors.
    - **Depends‑on:** none
- [ ] **T002 · Refactor · P2: Add ConfigError to AppError**
    - **Context:** Detailed Build Steps / 3. Configuration Module
    - **Action:**
        1. In `src/errors.rs`, add a variant for `ConfigError` within `AppError`.
    - **Done‑when:**
        1. `ConfigError` variant is defined and usable in `AppResult`.
    - **Verification:**
        1. Test with a simple function returning `AppResult` involving `ConfigError`.
    - **Depends‑on:** [T001]

## Config
- [ ] **T003 · Refactor · P2: Implement Config struct**
    - **Context:** Detailed Build Steps / 3. Configuration Module
    - **Action:**
        1. In `src/config.rs`, define `Config` struct with fields for journal_dir and editor_cmd.
    - **Done‑when:**
        1. `Config` struct compiles and includes required fields.
    - **Verification:**
        1. Create an instance and assert field presence in tests.
    - **Depends‑on:** [T002]
- [ ] **T004 · Refactor · P2: Implement Config load function**
    - **Context:** Detailed Build Steps / 3. Configuration Module
    - **Action:**
        1. In `src/config.rs`, add `Config::load()` to read from environment variables and return `AppResult<Config>`.
    - **Done‑when:**
        1. `Config::load()` handles env vars like `PONDER_DIR` and `EDITOR`, falling back to defaults.
        2. Function returns errors via `AppResult`.
    - **Verification:**
        1. Set env vars and call `load()`; check output with assertions.
    - **Depends‑on:** [T003]

## Journal IO
- [ ] **T005 · Refactor · P2: Define JournalIO trait**
    - **Context:** Detailed Build Steps / 4. I/O Adapter Trait & Implementation
    - **Action:**
        1. In `src/journal/io.rs`, define `JournalIO` trait with methods for file operations.
    - **Done‑when:**
        1. Trait includes methods like `ensure_journal_dir_exists`, `get_entry_path`.
        2. Trait compiles without errors.
    - **Verification:**
        1. Implement a mock and verify method signatures.
    - **Depends‑on:** none
- [ ] **T006 · Refactor · P2: Implement FilesystemJournalIO**
    - **Context:** Detailed Build Steps / 4. I/O Adapter Trait & Implementation
    - **Action:**
        1. In `src/journal/io.rs`, implement `FilesystemJournalIO` struct for the `JournalIO` trait using `std::fs`.
    - **Done‑when:**
        1. All trait methods are implemented and handle file operations correctly.
        2. Implementation returns `AppResult` for fallible operations.
    - **Verification:**
        1. Use temporary files to test methods manually or in integration tests.
    - **Depends‑on:** [T005]

## Editor
- [ ] **T007 · Refactor · P2: Define Editor trait**
    - **Context:** Detailed Build Steps / 5. Editor Adapter Trait & Implementation
    - **Action:**
        1. In `src/editor.rs`, define `Editor` trait with `open_files` method.
    - **Done‑when:**
        1. Trait compiles and specifies method for launching editor.
    - **Verification:**
        1. Create a mock implementation and call the method.
    - **Depends‑on:** none
- [ ] **T008 · Refactor · P2: Implement SystemEditor**
    - **Context:** Detailed Build Steps / 5. Editor Adapter Trait & Implementation
    - **Action:**
        1. In `src/editor.rs`, implement `SystemEditor` struct for the `Editor` trait using `std::process::Command`.
    - **Done‑when:**
        1. Method launches external editor and handles errors via `AppResult`.
    - **Verification:**
        1. Test with a dummy command to ensure process spawning works without actual editor.
    - **Depends‑on:** [T007]

## Journal
- [ ] **T009 · Refactor · P2: Define DateSpecifier enum**
    - **Context:** Detailed Build Steps / 6. Core Journal Logic
    - **Action:**
        1. In `src/journal/mod.rs`, define `DateSpecifier` enum for date types.
    - **Done‑when:**
        1. Enum compiles and covers variants like today, retro, specific date.
    - **Verification:**
        1. Use in a test function to pattern-match variants.
    - **Depends‑on:** none
- [ ] **T010 · Refactor · P2: Implement JournalService struct**
    - **Context:** Detailed Build Steps / 6. Core Journal Logic
    - **Action:**
        1. In `src/journal/mod.rs`, define `JournalService` struct with dependencies like `Box<dyn JournalIO>` and `Box<dyn Editor>`.
    - **Done‑when:**
        1. Struct compiles with constructor for dependencies.
    - **Verification:**
        1. Instantiate and call a method in tests.
    - **Depends‑on:** [T005, T007]
- [ ] **T011 · Refactor · P2: Implement JournalService methods**
    - **Context:** Detailed Build Steps / 6. Core Journal Logic
    - **Action:**
        1. In `src/journal/mod.rs`, add methods like `open_entry` using `JournalIO` and `Editor`.
    - **Done‑when:**
        1. Methods compile and orchestrate journal operations.
        2. Each method returns `AppResult`.
    - **Verification:**
        1. Run unit tests for logic paths.
    - **Depends‑on:** [T010]

## CLI
- [ ] **T012 · Refactor · P2: Define CliArgs struct**
    - **Context:** Detailed Build Steps / 7. CLI Module
    - **Action:**
        1. In `src/cli.rs`, define `CliArgs` using `clap::Parser`.
    - **Done‑when:**
        1. Struct compiles and handles command parsing.
    - **Verification:**
        1. Parse sample arguments and assert results.
    - **Depends‑on:** none

## Main
- [ ] **T013 · Refactor · P2: Implement main function logic**
    - **Context:** Detailed Build Steps / 8. Main Application Logic
    - **Action:**
        1. In `src/main.rs`, add code to parse args, load config, instantiate services, and call methods.
    - **Done‑when:**
        1. Main function handles errors and orchestrates flow.
        2. Program exits with appropriate status codes.
    - **Verification:**
        1. Run the binary with test inputs and check outputs.
    - **Depends‑on:** [T003, T012, T011]

## Testing
- [ ] **T014 · Refactor · P2: Add unit tests for journal module**
    - **Context:** Testing Strategy / Unit Tests
    - **Action:**
        1. In `src/journal/mod.rs`, add `#[cfg(test)]` tests for date calculations.
    - **Done‑when:**
        1. Tests pass for happy and edge cases.
    - **Verification:**
        1. Run `cargo test` and check coverage.
    - **Depends‑on:** [T009]
- [ ] **T015 · Refactor · P2: Add integration tests for JournalService**
    - **Context:** Testing Strategy / Integration Tests
    - **Action:**
        1. In `tests/`, create tests using fakes for JournalIO and Editor.
    - **Done‑when:**
        1. Tests verify interactions between modules.
    - **Verification:**
        1. Execute tests and assert mock behaviors.
    - **Depends‑on:** [T010]
- [ ] **T016 · Refactor · P2: Add CLI end-to-end tests**
    - **Context:** Testing Strategy / CLI End-to-End Tests
    - **Action:**
        1. In `tests/`, use `assert_cmd` to test binary invocations.
    - **Done‑when:**
        1. Tests cover all commands and error scenarios.
    - **Verification:**
        1. Run tests in CI and check pass/fail.
    - **Depends‑on:** [T013]

## Documentation
- [ ] **T017 · Chore · P2: Update README.md**
    - **Context:** Initial Documentation / Required Readme
    - **Action:**
        1. In root, update `README.md` with usage and setup.
    - **Done‑when:**
        1. File includes project description and examples.
        2. Content is accurate and complete.
    - **Verification:**
        1. Review for completeness and build the app to verify examples.
    - **Depends‑on:** [T013]

### Clarifications & Assumptions
- [ ] **Issue:** Determine Minimum Supported Rust Version (MSRV)
    - **Context:** Open Questions / Minimum Supported Rust Version
    - **Blocking?:** yes
- [ ] **Issue:** Decide on extended configuration support
    - **Context:** Open Questions / Extended Configuration
    - **Blocking?:** no
- [ ] **Issue:** Clarify date/time formatting requirements
    - **Context:** Open Questions / Date/Time Formatting
    - **Blocking?:** no
- [ ] **Issue:** Define editor fallback behavior
    - **Context:** Open Questions / Editor Fallback
    - **Blocking?:** no
- [ ] **Issue:** Specify reminiscence logic details
    - **Context:** Open Questions / Reminiscence Logic
    - **Blocking?:** no