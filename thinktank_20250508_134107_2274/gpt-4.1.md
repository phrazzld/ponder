# Todo

## project-setup
- [ ] **T001 · Chore · P0: initialize module file structure and update Cargo.toml**
    - **Context:** PLAN.md, Step 1
    - **Action:**
        1. Ensure `Cargo.toml` has `edition = "2021"` and MSRV comment.
        2. Create `src/main.rs`, `src/cli.rs`, `src/config.rs`, `src/journal/mod.rs`, `src/journal/io.rs`, `src/editor.rs`, `src/errors.rs`.
        3. Add dependencies: `clap` (with `derive`), `chrono`, `thiserror`, `log`, `env_logger`, `shellexpand`.
    - **Done‑when:**
        1. All files exist with module declarations.
        2. `Cargo.toml` includes required dependencies and correct edition.
    - **Verification:**
        1. `cargo build` and `cargo check` pass.
    - **Depends‑on:** none

## errors
- [ ] **T002 · Feature · P0: define AppError enum and AppResult type**
    - **Context:** PLAN.md, Step 2
    - **Action:**
        1. Implement `AppError` enum in `errors.rs` with variants for I/O, config, journal logic, editor, and argument parsing.
        2. Implement `AppResult<T> = Result<T, AppError>`.
    - **Done‑when:**
        1. `AppError` covers all error sources and implements `std::error::Error`.
        2. Can be imported and used in other modules.
    - **Verification:**
        1. Unit test: convert representative error types to `AppError`.
    - **Depends‑on:** [T001]

## config
- [ ] **T003 · Feature · P1: define Config struct and implement Config::load()**
    - **Context:** PLAN.md, Step 3
    - **Action:**
        1. Define `Config` struct with `journal_dir`, `editor_cmd`, and other settings in `config.rs`.
        2. Implement `Config::load()` to read environment variables, apply fallbacks, and validate.
        3. Define `ConfigError` and integrate into `AppError`.
    - **Done‑when:**
        1. `Config::load()` returns valid config or error for all scenarios.
    - **Verification:**
        1. Unit tests: load config with/without env vars, invalid values.
    - **Depends‑on:** [T002]

## journal-io
- [ ] **T004 · Feature · P1: define JournalIO trait in journal/io.rs**
    - **Context:** PLAN.md, Step 4
    - **Action:**
        1. Specify `JournalIO` trait with methods for dir creation, path generation, file existence, read, append, create.
        2. Use `AppResult` for fallible methods.
    - **Done‑when:**
        1. Trait is defined and compiles.
    - **Verification:**
        1. All methods accept/return correct types.
    - **Depends‑on:** [T002]
- [ ] **T005 · Feature · P1: implement FilesystemJournalIO struct**
    - **Context:** PLAN.md, Step 4
    - **Action:**
        1. Implement `JournalIO` for `FilesystemJournalIO` using `std::fs`/`std::path`.
        2. Ensure all file and directory operations handle errors robustly.
    - **Done‑when:**
        1. All trait methods are implemented and tested for common edge cases.
    - **Verification:**
        1. Unit or integration test: create/read/append/check/ensure files/dirs using temp dirs.
    - **Depends‑on:** [T004]

## editor
- [ ] **T006 · Feature · P1: define Editor trait in editor.rs**
    - **Context:** PLAN.md, Step 5
    - **Action:**
        1. Specify `Editor` trait with `open_files(&self, editor_cmd: &str, paths: &[PathBuf]) -> AppResult<()>`.
    - **Done‑when:**
        1. Trait is defined and compiles.
    - **Verification:**
        1. Trait signature covers required abstraction.
    - **Depends‑on:** [T002]
- [ ] **T007 · Feature · P1: implement SystemEditor struct**
    - **Context:** PLAN.md, Step 5
    - **Action:**
        1. Implement `Editor` trait for `SystemEditor` using `std::process::Command` to launch the editor.
        2. Handle command errors, exit statuses.
    - **Done‑when:**
        1. Editor launches specified command or returns error.
    - **Verification:**
        1. Unit test with fake or dry-run command.
    - **Depends‑on:** [T006]

## journal-core
- [ ] **T008 · Feature · P0: define DateSpecifier type in journal/mod.rs**
    - **Context:** PLAN.md, Step 6
    - **Action:**
        1. Define enum/struct to represent entry date variants (today, retro, specific date).
    - **Done‑when:**
        1. Type defined and usable in journal logic.
    - **Verification:**
        1. Unit test: construct from typical CLI inputs.
    - **Depends‑on:** [T001]
- [ ] **T009 · Feature · P0: define JournalService struct and constructor**
    - **Context:** PLAN.md, Step 6
    - **Action:**
        1. Implement `JournalService` struct with dependencies: `Config`, `Box<dyn JournalIO>`, `Box<dyn Editor>`.
        2. Provide constructor enforcing dependency injection.
    - **Done‑when:**
        1. Object can be created with test fakes and real implementations.
    - **Verification:**
        1. Unit test: construct with mocks/dummies.
    - **Depends‑on:** [T003, T005, T007, T008]
- [ ] **T010 · Feature · P0: implement open_entry, open_retro_entry, open_reminisce_entry**
    - **Context:** PLAN.md, Step 6
    - **Action:**
        1. Implement core service methods for each journal action, using `JournalIO` and `Editor`.
        2. Handle date calculation, file path, file creation, editor launch, and error propagation.
    - **Done‑when:**
        1. Each method performs all steps and returns appropriately.
    - **Verification:**
        1. Integration/unit tests: each method called with fakes/mocks, all paths exercised.
    - **Depends‑on:** [T009]
- [ ] **T011 · Test · P0: unit and integration tests for JournalService**
    - **Context:** PLAN.md, Step 6, Testing Strategy
    - **Action:**
        1. Write tests for date calculation, file path logic, and error handling in `JournalService`.
        2. Use test doubles for `JournalIO` and `Editor`.
    - **Done‑when:**
        1. All core journal logic code paths covered.
    - **Verification:**
        1. Run `cargo test`, measure >90% coverage for journal logic.
    - **Depends‑on:** [T010]

## cli
- [ ] **T012 · Feature · P1: define CliArgs struct using clap**
    - **Context:** PLAN.md, Step 7
    - **Action:**
        1. Define `CliArgs` struct with `clap::Parser` for all commands/options.
        2. Implement or derive argument parsing.
    - **Done‑when:**
        1. CLI parsing works for all command variants.
    - **Verification:**
        1. Unit test: parse sample args, check struct fields.
    - **Depends‑on:** [T001]
- [ ] **T013 · Test · P1: CLI argument parsing and validation tests**
    - **Context:** PLAN.md, Testing Strategy
    - **Action:**
        1. Write tests for valid and invalid CLI invocations.
    - **Done‑when:**
        1. All CLI argument edge cases are tested.
    - **Verification:**
        1. `cargo test` covers all command/option combinations.
    - **Depends‑on:** [T012]

## main
- [ ] **T014 · Feature · P0: implement application entrypoint in main.rs**
    - **Context:** PLAN.md, Step 8
    - **Action:**
        1. Initialize logging.
        2. Parse CLI args and load config.
        3. Instantiate concrete `FilesystemJournalIO` and `SystemEditor`.
        4. Create `JournalService` and dispatch command.
        5. Handle `AppResult`, log errors, print user messages, exit correctly.
    - **Done‑when:**
        1. All commands route from CLI to service, errors handled at top-level.
    - **Verification:**
        1. Manual run: invoke each command, observe output, error handling.
    - **Depends‑on:** [T003, T005, T007, T010, T012]

## logging-and-observability
- [ ] **T015 · Feature · P1: configure and initialize structured logging**
    - **Context:** PLAN.md, Logging & Observability
    - **Action:**
        1. Set up `env_logger` (or `tracing-subscriber`) for development and production.
        2. Ensure log fields: timestamp, level, target, message, and custom fields per event.
    - **Done‑when:**
        1. Logs are output in structured format with required fields for all key events and errors.
    - **Verification:**
        1. Manual: run app, check logs on stdout/stderr, examine error and info event fields.
    - **Depends‑on:** [T014]
- [ ] **T016 · Test · P2: logging behavior tests for key events and errors**
    - **Context:** PLAN.md, Logging & Observability
    - **Action:**
        1. Write tests verifying log output for key events and error scenarios.
    - **Done‑when:**
        1. Log output for config load, CLI exec, file ops, errors are tested.
    - **Verification:**
        1. Automated or manual inspection of log lines in test output.
    - **Depends‑on:** [T015]

## io-testing
- [ ] **T017 · Test · P1: integration tests for FilesystemJournalIO**
    - **Context:** PLAN.md, Testing Strategy
    - **Action:**
        1. Use `tempfile` to test file and directory operations.
        2. Cover creation, reading, writing, error cases.
    - **Done‑when:**
        1. All IO methods are covered for normal and error conditions.
    - **Verification:**
        1. `cargo test` passes; filesystem state matches expectations after each test.
    - **Depends‑on:** [T005]

## cli-e2e
- [ ] **T018 · Test · P1: CLI end-to-end tests via assert_cmd**
    - **Context:** PLAN.md, Testing Strategy
    - **Action:**
        1. Use `assert_cmd` and `predicates` to invoke the binary with various arguments.
        2. Verify exit codes, stdout/stderr, temp journal dir used.
    - **Done‑when:**
        1. All CLI command flows are tested end-to-end.
    - **Verification:**
        1. Tests pass for happy paths and error conditions.
    - **Depends‑on:** [T014]

## docs
- [ ] **T019 · Chore · P1: add and update Rustdoc comments on all public items**
    - **Context:** PLAN.md, Step 10, Documentation
    - **Action:**
        1. Write `///` comments for all public modules, structs, traits, enums, and functions.
        2. Include usage and error notes where relevant.
    - **Done‑when:**
        1. All public API elements are documented.
    - **Verification:**
        1. Run `cargo doc`, review for completeness and clarity.
    - **Depends‑on:** [T002, T003, T004, T006, T008, T009, T010, T012]
- [ ] **T020 · Chore · P2: write or update README.md with usage, config, and architecture**
    - **Context:** PLAN.md, Step 10, Documentation
    - **Action:**
        1. Add sections: project description, install, usage, CLI commands, config, architecture, contributing, license.
    - **Done‑when:**
        1. README includes all major sections and accurate examples.
    - **Verification:**
        1. Manual review for completeness.
    - **Depends‑on:** [T019]
- [ ] **T021 · Chore · P2: write CONTRIBUTING.md with dev setup and coding standards**
    - **Context:** PLAN.md, Documentation
    - **Action:**
        1. Document setup steps, style rules, test and PR process.
    - **Done‑when:**
        1. CONTRIBUTING.md covers all requisite info for new contributors.
    - **Verification:**
        1. Manual review.
    - **Depends‑on:** [T020]

## ci-and-automation
- [ ] **T022 · Chore · P1: add rustfmt.toml and enforce formatting in CI**
    - **Context:** PLAN.md, Step 9
    - **Action:**
        1. Add `rustfmt.toml` if custom rules needed (prefer defaults).
        2. Ensure formatting checked in CI.
    - **Done‑when:**
        1. All code is formatted; CI fails on misformat.
    - **Verification:**
        1. Run `cargo fmt --check` locally and in CI.
    - **Depends‑on:** [T001]
- [ ] **T023 · Chore · P1: configure and enforce clippy lints in CI**
    - **Context:** PLAN.md, Step 9
    - **Action:**
        1. Configure `clippy` for strict linting; document exceptions if any.
        2. Add to CI pipeline.
    - **Done‑when:**
        1. CI fails on clippy warnings (except documented exceptions).
    - **Verification:**
        1. Run `cargo clippy -- -D warnings` locally and in CI.
    - **Depends‑on:** [T001]
- [ ] **T024 · Chore · P1: add CI workflow for build, test, lint, audit, and coverage**
    - **Context:** PLAN.md, Step 9
    - **Action:**
        1. Add GitHub Actions or similar to run build, test, lint, `cargo audit`, and coverage.
    - **Done‑when:**
        1. CI passes for clean code, fails on errors, vulnerabilities, or coverage drop.
    - **Verification:**
        1. Push branch and observe CI run; simulate failure states.
    - **Depends‑on:** [T022, T023]
- [ ] **T025 · Chore · P2: integrate cargo-tarpaulin for test coverage enforcement**
    - **Context:** PLAN.md, Step 9, Testing Strategy
    - **Action:**
        1. Set up `cargo-tarpaulin` to measure coverage.
        2. Enforce thresholds in CI.
    - **Done‑when:**
        1. Coverage metrics visible in CI, failures on drop below thresholds.
    - **Verification:**
        1. Pull request with low coverage fails CI.
    - **Depends‑on:** [T024]

## open-questions
- [ ] **Issue: clarify and decide on Minimum Supported Rust Version (MSRV)**
    - **Context:** PLAN.md, Open Questions
    - **Blocking?:** yes
- [ ] **Issue: clarify approach for extended configuration file support**
    - **Context:** PLAN.md, Open Questions
    - **Blocking?:** no
- [ ] **Issue: clarify any specific requirements for date/time formats**
    - **Context:** PLAN.md, Open Questions
    - **Blocking?:** no
- [ ] **Issue: clarify desired fallback behavior if editor is unset or not found**
    - **Context:** PLAN.md, Open Questions
    - **Blocking?:** no
- [ ] **Issue: clarify desired reminiscence logic for selecting old entries**
    - **Context:** PLAN.md, Open Questions
    - **Blocking?:** no
