# Todo

## Core: Project setup
- [ ] **T001 · Chore · P1: initialize cargo project with edition and MSRV placeholder**
    - Context: PLAN.md “Project Setup & Module Structure”
    - Action:
        1. Create new Rust binary project (`cargo init`) or manually create `Cargo.toml` with `edition = "2021"` and placeholder for MSRV.
        2. Create `src/` directory with empty `main.rs`.
    - Done-when:
        1. `cargo build` succeeds.
    - Verification:
        1. Running `cargo run` prints default Rust “Hello” message.
    - Depends-on: none

- [ ] **T002 · Chore · P1: add required dependencies to Cargo.toml**
    - Context: PLAN.md “Add necessary dependencies” in step 1
    - Action:
        1. Add to `Cargo.toml` dependencies: `clap` (derive), `chrono`, `thiserror`, `log`, `env_logger` (or `tracing-subscriber`), `shellexpand`.
        2. Run `cargo build` to validate.
    - Done-when:
        1. Dependencies listed in `Cargo.toml`.
        2. `cargo build` completes successfully.
    - Depends-on: [T001]

## Module: errors
- [ ] **T003 · Feature · P1: define AppError and AppResult in errors.rs**
    - Context: PLAN.md “Error Handling Foundation (`errors.rs`)”
    - Action:
        1. Create `src/errors.rs`.
        2. Define `enum AppError` with variants for config, I/O, journal, editor errors using `thiserror`.
        3. Define `type AppResult<T> = Result<T, AppError>`.
    - Done-when:
        1. Code compiles.
        2. `AppError` and `AppResult` types are exported.
    - Depends-on: [T001]

## Module: config
- [ ] **T004 · Feature · P2: create Config struct and load signature in config.rs**
    - Context: PLAN.md “Configuration Module”
    - Action:
        1. Create `src/config.rs` with `struct Config { journal_dir: PathBuf, editor_cmd: String, … }`.
        2. Declare `fn load() -> Result<Config, ConfigError>`.
    - Done-when:
        1. Config struct and load signature compile successfully.
    - Depends-on: [T003]

- [ ] **T005 · Feature · P2: implement Config::load to read env vars with defaults**
    - Context: PLAN.md “Implement Config::load”
    - Action:
        1. Read `PONDER_DIR`, fallback to `$HOME/Documents/ponder` via `shellexpand`.
        2. Read `EDITOR`, fallback to `vim`.
        3. Map errors into `ConfigError` and wrap in `AppError`.
    - Done-when:
        1. `Config::load` returns correct `Config` in unit tests.
    - Verification:
        1. Unit tests for presence and absence of env vars using temporary env manipulation.
    - Depends-on: [T004, T003]

## Module: journal/io
- [ ] **T006 · Feature · P2: define JournalIO trait in src/journal/io.rs**
    - Context: PLAN.md “I/O Adapter Trait & Implementation”
    - Action:
        1. Create `src/journal/io.rs`.
        2. Define `trait JournalIO` with methods: `ensure_journal_dir_exists`, `get_entry_path`, `read_entry`, `append_to_entry`, `entry_exists`, `create_file_if_not_exists`.
    - Done-when:
        1. Trait compiles and is publicly exported.
    - Depends-on: [T003]

- [ ] **T007 · Feature · P2: implement FilesystemJournalIO for JournalIO**
    - Context: PLAN.md “Implement FilesystemJournalIO”
    - Action:
        1. Implement `struct FilesystemJournalIO`.
        2. Implement `JournalIO` methods using `std::fs` and `std::path`.
    - Done-when:
        1. Implementation compiles.
        2. Unit tests using `tempfile::tempdir()` verify FS operations.
    - Depends-on: [T006]

## Module: editor
- [ ] **T008 · Feature · P2: define Editor trait in src/editor.rs**
    - Context: PLAN.md “Editor Adapter Trait & Implementation”
    - Action:
        1. Create `src/editor.rs`.
        2. Define `trait Editor { fn open_files(&self, editor_cmd: &str, paths: &[PathBuf]) -> AppResult<()>; }`.
    - Done-when:
        1. Trait compiles.
    - Depends-on: [T003]

- [ ] **T009 · Feature · P2: implement SystemEditor using std::process::Command**
    - Context: PLAN.md “Implement SystemEditor”
    - Action:
        1. Implement `struct SystemEditor`.
        2. Implement `Editor` trait launching external editor via `Command`.
    - Done-when:
        1. `SystemEditor` compiles.
    - Verification:
        1. Write a test invoking `open_files` with a harmless command (e.g., `true`).
    - Depends-on: [T008]

## Module: journal logic
- [ ] **T010 · Feature · P2: define DateSpecifier and JournalService skeleton in journal/mod.rs**
    - Context: PLAN.md “Core Journal Logic”
    - Action:
        1. Create `src/journal/mod.rs`.
        2. Define `enum DateSpecifier { Today, Retro(u32), Specific(DateTime), … }`.
        3. Define `struct JournalService { config: Config, io: Box<dyn JournalIO>, editor: Box<dyn Editor> }` and constructor.
    - Done-when:
        1. Skeleton compiles.
    - Depends-on: [T003, T006, T008, T004]

- [ ] **T011 · Feature · P2: implement JournalService methods open_entry, open_retro_entry, open_reminisce_entry**
    - Context: PLAN.md “Implement core methods”
    - Action:
        1. Use `chrono` for date calculations.
        2. Call `JournalIO` for file ops and `Editor` for launching editor.
        3. Propagate errors via `AppResult`.
    - Done-when:
        1. Methods behave correctly in unit tests.
    - Verification:
        1. Unit tests cover happy paths and error cases.
    - Depends-on: [T010, T006, T009, T005]

## Module: cli
- [ ] **T012 · Feature · P2: define CliArgs with clap in cli.rs**
    - Context: PLAN.md “CLI Module”
    - Action:
        1. Create `src/cli.rs` with `#[derive(clap::Parser)] struct CliArgs { … }`.
        2. Define `new`, `retro`, `reminisce` commands and options.
    - Done-when:
        1. `CliArgs::parse()` compiles and parses sample args.
    - Verification:
        1. Unit tests for parsing example CLI inputs.
    - Depends-on: [T002]

## Module: main
- [ ] **T013 · Feature · P1: implement main.rs with app initialization and dispatch**
    - Context: PLAN.md “Main Application Logic”
    - Action:
        1. Initialize logging.
        2. Parse CLI args.
        3. Load config.
        4. Instantiate `FilesystemJournalIO`, `SystemEditor`.
        5. Instantiate `JournalService`.
        6. Match on `CliArgs` to call appropriate method.
        7. Handle `AppResult`: print messages or errors to stderr and exit with code.
    - Done-when:
        1. `cargo run -- new|retro|reminisce` dispatches without panics.
        2. Errors yield non-zero exit with friendly message.
    - Depends-on: [T005, T011, T012, T007, T009]

## Module: logging
- [ ] **T014 · Chore · P2: integrate structured logging with env_logger in main**
    - Context: PLAN.md “Logging & Observability”
    - Action:
        1. Configure `env_logger` (or `tracing-subscriber`) in `main.rs`.
        2. Add log calls for startup, config load, command dispatch.
    - Done-when:
        1. Logs include `timestamp`, `level`, `target`, `message`.
        2. Running with `RUST_LOG=ponder=debug` outputs logs.
    - Depends-on: [T013, T003]

## Module: tooling
- [ ] **T015 · Chore · P2: configure rustfmt and clippy for project**
    - Context: PLAN.md “Coding Standards & Automation Setup”
    - Action:
        1. Add `rustfmt.toml` if needed (prefer defaults).
        2. Create `.cargo/config.toml` or CI flags to enforce `clippy` (`-D warnings`).
    - Done-when:
        1. `cargo fmt -- --check` passes.
        2. `cargo clippy -- -D warnings` passes.
    - Depends-on: [T001]

## Module: CI
- [ ] **T016 · Chore · P2: set up CI pipeline for build, lint, test, audit**
    - Context: PLAN.md “Setup CI”
    - Action:
        1. Create `.github/workflows/ci.yml`.
        2. Steps: checkout, setup Rust, `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`, `cargo build --release`.
    - Done-when:
        1. CI passes on initial commit.
    - Depends-on: [T015]

## Module: testing
- [ ] **T017 · Test · P2: write unit tests for config, journal date logic, CLI parsing**
    - Context: PLAN.md “Unit Tests”
    - Action:
        1. Add `#[cfg(test)]` tests in `config.rs`, `journal/mod.rs`, `cli.rs`.
        2. Cover date calculations, env var parsing, CLI parsing.
    - Done-when:
        1. `cargo test` passes with >90% coverage for core logic.
    - Depends-on: [T005, T010, T012]

- [ ] **T018 · Test · P2: write integration tests for JournalService with mocks**
    - Context: PLAN.md “Integration Tests”
    - Action:
        1. Under `tests/`, create `integration_journal.rs`.
        2. Implement `MockJournalIO` and `MockEditor`.
        3. Assert `JournalService` behavior without actual FS or editor.
    - Done-when:
        1. Integration tests pass.
    - Depends-on: [T011, T006, T008]

- [ ] **T019 · Test · P2: write CLI end-to-end tests using assert_cmd**
    - Context: PLAN.md “CLI End-to-End Tests”
    - Action:
        1. Under `tests/`, create `cli_e2e.rs`.
        2. Use `assert_cmd` to run the compiled binary against temp journal dir.
        3. Verify exit codes and output.
    - Done-when:
        1. E2E tests pass in CI.
    - Depends-on: [T013]

- [ ] **T020 · Chore · P2: enforce code coverage reporting with cargo-tarpaulin**
    - Context: PLAN.md “Coverage Targets”
    - Action:
        1. Add `cargo-tarpaulin` step in CI.
        2. Fail CI if coverage <80%.
    - Done-when:
        1. CI enforces coverage threshold.
    - Depends-on: [T016]

## Module: documentation
- [ ] **T021 · Chore · P2: create README.md with project overview and usage**
    - Context: PLAN.md “Initial Documentation”
    - Action:
        1. Write `README.md` covering purpose, install, usage, config, examples.
    - Done-when:
        1. README displays correctly on GitHub.
    - Depends-on: [T013]

- [ ] **T022 · Chore · P2: create CONTRIBUTING.md with dev guidelines**
    - Context: PLAN.md “Initial Documentation”
    - Action:
        1. Write `CONTRIBUTING.md` with setup, style, testing, PR process.
    - Done-when:
        1. CONTRIBUTING file exists and is referenced from README.
    - Depends-on: [T021]

### Clarifications & Assumptions
- [ ] **Issue:** determine the Minimum Supported Rust Version (MSRV) target  
    - Context: PLAN.md “Open Questions: MSRV”  
    - Blocking?: yes

- [ ] **Issue:** decide if Ponder should support a config file in addition to env vars  
    - Context: PLAN.md “Open Questions: Extended Configuration”  
    - Blocking?: no

- [ ] **Issue:** specify date/time formatting requirements beyond YYYY-MM-DD  
    - Context: PLAN.md “Open Questions: Date/Time Formatting”  
    - Blocking?: no

- [ ] **Issue:** define fallback editor behavior when $EDITOR and default not found  
    - Context: PLAN.md “Open Questions: Editor Fallback”  
    - Blocking?: no

- [ ] **Issue:** clarify reminiscence entry selection logic  
    - Context: PLAN.md “Open Questions: Reminiscence Logic”  
    - Blocking?: yes