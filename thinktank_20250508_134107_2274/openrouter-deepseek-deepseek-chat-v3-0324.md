# Todo

## Core Modules
- [ ] **T001 · Refactor · P1: create module structure**
    - **Context:** Project Setup & Module Structure
    - **Action:**
        1. Create `src/` with `main.rs`, `cli.rs`, `config.rs`, `errors.rs`, `editor.rs`
        2. Create `src/journal/` with `mod.rs` and `io.rs`
    - **Done-when:**
        1. All module files exist with basic module declarations
    - **Depends-on:** none

- [ ] **T002 · Refactor · P1: define AppError in errors.rs**
    - **Context:** Error Handling Foundation
    - **Action:**
        1. Define `AppError` enum with variants for I/O, config, journal, editor
        2. Implement `std::error::Error` using `thiserror` derive
        3. Define `AppResult<T>` type alias
    - **Done-when:**
        1. All error variants defined and compiles with `thiserror`
    - **Depends-on:** none

## Configuration
- [ ] **T003 · Feature · P1: implement Config struct**
    - **Context:** Configuration Module
    - **Action:**
        1. Define `Config` struct with journal_dir and editor_cmd fields
        2. Implement `Config::load()` to read env vars
    - **Done-when:**
        1. Struct defined and can load basic config from env
    - **Depends-on:** [T002]

## CLI
- [ ] **T004 · Feature · P1: implement CLI argument parsing**
    - **Context:** CLI Module
    - **Action:**
        1. Define `CliArgs` struct with clap attributes
        2. Implement subcommands for new/retro/reminisce
    - **Done-when:**
        1. Basic CLI parsing works for all commands
    - **Depends-on:** none

## Journal I/O
- [ ] **T005 · Feature · P1: define JournalIO trait**
    - **Context:** I/O Adapter Trait
    - **Action:**
        1. Define trait with fs operation methods
        2. Document all methods with Rustdoc
    - **Done-when:**
        1. Trait compiles with all required methods
    - **Depends-on:** [T002]

- [ ] **T006 · Feature · P2: implement FilesystemJournalIO**
    - **Context:** I/O Adapter Implementation
    - **Action:**
        1. Implement `JournalIO` for `FilesystemJournalIO` struct
        2. Use std::fs for actual operations
    - **Done-when:**
        1. All trait methods implemented
    - **Depends-on:** [T005]

## Editor Integration
- [ ] **T007 · Feature · P1: define Editor trait**
    - **Context:** Editor Adapter Trait
    - **Action:**
        1. Define trait with editor launch method
        2. Document with Rustdoc
    - **Done-when:**
        1. Trait compiles with required method
    - **Depends-on:** [T002]

- [ ] **T008 · Feature · P2: implement SystemEditor**
    - **Context:** Editor Adapter Implementation
    - **Action:**
        1. Implement `Editor` for `SystemEditor` struct
        2. Use std::process::Command for editor launch
    - **Done-when:**
        1. Editor can launch external process
    - **Depends-on:** [T007]

## Core Logic
- [ ] **T009 · Feature · P1: define JournalService**
    - **Context:** Core Journal Logic
    - **Action:**
        1. Define `JournalService` struct with dependencies
        2. Implement constructor injection
    - **Done-when:**
        1. Struct compiles with all dependencies
    - **Depends-on:** [T005, T007]

- [ ] **T010 · Feature · P2: implement journal operations**
    - **Context:** Core Journal Logic
    - **Action:**
        1. Implement open_entry, open_retro_entry, open_reminisce_entry
        2. Use chrono for date calculations
    - **Done-when:**
        1. All core methods implemented
    - **Depends-on:** [T009]

## Main Application
- [ ] **T011 · Feature · P1: wire components in main.rs**
    - **Context:** Main Application Logic
    - **Action:**
        1. Initialize logging
        2. Wire CLI, Config, JournalService together
    - **Done-when:**
        1. Basic command flow works
    - **Depends-on:** [T003, T004, T010]

## Testing
- [ ] **T012 · Test · P2: implement unit tests for journal logic**
    - **Context:** Testing Strategy
    - **Action:**
        1. Create test module in journal.rs
        2. Test date calculations and path generation
    - **Done-when:**
        1. Core logic has >90% coverage
    - **Depends-on:** [T010]

- [ ] **T013 · Test · P2: implement integration tests**
    - **Context:** Testing Strategy
    - **Action:**
        1. Create tests/ directory
        2. Test JournalService with mock IO/Editor
    - **Done-when:**
        1. Integration tests pass with mocks
    - **Depends-on:** [T010]

## Documentation
- [ ] **T014 · Chore · P3: update README.md**
    - **Context:** Documentation
    - **Action:**
        1. Document installation and usage
        2. Add architecture overview
    - **Done-when:**
        1. README covers all basic usage
    - **Depends-on:** none

## CI/CD
- [ ] **T015 · Chore · P2: setup CI pipeline**
    - **Context:** Coding Standards & Automation Setup
    - **Action:**
        1. Add GitHub Actions workflow
        2. Configure fmt, clippy, test, audit
    - **Done-when:**
        1. CI passes on all checks
    - **Depends-on:** none

### Clarifications & Assumptions
- [ ] **Issue:** MSRV not specified
    - **Context:** Open Questions
    - **Blocking?:** no