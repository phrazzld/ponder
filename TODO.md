# Todo

## Logging Remediation (cr-02)
- [x] **T021 · Refactor · P1: replace `println!` at `src/journal/mod.rs:536` with `log::info!`**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-02 / Steps / 1
    - **Action:**
        1. In `src/journal/mod.rs:536`, replace `println!("No entries found for the past week");` with `log::info!("No entries found for the past week");`.
    - **Done‑when:**
        1. The `println!` call is replaced with `log::info!`.
        2. The application compiles and runs.
        3. `cargo clippy --all-targets -- -D warnings` passes regarding this change.
    - **Depends‑on:** none

- [x] **T022 · Refactor · P1: replace `println!` at `src/journal/mod.rs:544` with appropriate `log` macro**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-02 / Steps / 2
    - **Action:**
        1. In `src/journal/mod.rs:544`, replace `println!("No entries found for reminisce intervals");` with `log::info!("No entries found for reminisce intervals");` (or `log::warn!` if this state indicates a potential issue or deviation from expected behavior, as per plan guidance).
    - **Done‑when:**
        1. The `println!` call is replaced with an appropriate `log` macro.
        2. The application compiles and runs.
        3. `cargo clippy --all-targets -- -D warnings` passes regarding this change.
    - **Depends‑on:** none

- [ ] **T023 · Refactor · P1: initialize structured logging in application entry point**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-02 / Steps / 3
    - **Action:**
        1. Ensure a logging implementation (e.g., `env_logger`, `tracing-subscriber`) is properly initialized in the application's main entry point (`src/main.rs` or equivalent) to output messages in the standard structured format (e.g., JSON).
    - **Done‑when:**
        1. Structured logging is initialized and configured for JSON output.
        2. The application compiles and runs.
        3. `cargo clippy --all-targets -- -D warnings` passes regarding this change.
    - **Depends‑on:** none

## Dead Code Remediation (cr-01)
- [ ] **T024 · Chore · P1: update `TODO.md` to mark tasks T016-T020 as incomplete**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-01 / Steps / 1
    - **Action:**
        1. In `TODO.md`, change the status for tasks T016-T020 back to `[ ]` (incomplete).
    - **Done‑when:**
        1. `TODO.md` tasks T016-T020 are marked as `[ ]`.
    - **Depends‑on:** none

- [ ] **T025 · Refactor · P1: scope `CliArgs::parse_date()` to tests and remove `dead_code` suppression**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-01 / Steps / 2 (`src/cli/mod.rs:155`)
    - **Action:**
        1. In `src/cli/mod.rs`, move `CliArgs::parse_date()` into a `#[cfg(test)] mod tests { ... }` block, or change its signature to `#[cfg(test)] pub fn parse_date(&self) ...`.
        2. Remove the `#[allow(dead_code)]` attribute from `CliArgs::parse_date()`.
    - **Done‑when:**
        1. `CliArgs::parse_date()` is correctly scoped for test-only use.
        2. The `#[allow(dead_code)]` attribute is removed.
        3. `cargo clippy --all-targets -- -D warnings` passes regarding this change.
    - **Depends‑on:** none

- [ ] **T026 · Refactor · P1: scope `Config::new()` to tests and remove `dead_code` suppression**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-01 / Steps / 3 (`src/config/mod.rs:85`)
    - **Action:**
        1. In `src/config/mod.rs`, move `Config::new()` into a `#[cfg(test)] mod tests { ... }` block, or change its signature to `#[cfg(test)] pub fn new() ...`.
        2. Remove the `#[allow(dead_code)]` attribute from `Config::new()`.
    - **Done‑when:**
        1. `Config::new()` is correctly scoped for test-only use.
        2. The `#[allow(dead_code)]` attribute is removed.
        3. `cargo clippy --all-targets -- -D warnings` passes regarding this change.
    - **Depends‑on:** none

- [ ] **T027 · Refactor · P1: address `dead_code` for `JournalIO::ensure_journal_dir()`**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-01 / Steps / 4 (`src/journal/io/mod.rs:47`)
    - **Action:**
        1. Review `JournalIO::ensure_journal_dir()`: ensure a primary consumer of the `JournalIO` trait calls this method (e.g., `JournalService::new` or application setup logic).
        2. If actively used, remove the `#[allow(dead_code)]` attribute from the `ensure_journal_dir()` method definition in the `JournalIO` trait.
        3. If, after review, it's determined that this method is not a general part of the `JournalIO` contract for all consumers, remove it from the trait definition.
    - **Done‑when:**
        1. `JournalIO::ensure_journal_dir()` is either actively used via the trait (and `#[allow(dead_code)]` removed) OR it is removed from the trait definition.
        2. `cargo clippy --all-targets -- -D warnings` passes regarding this change.
    - **Depends‑on:** none

- [ ] **T028 · Refactor · P1: scope `JournalService` test-only `pub` methods to tests and remove `dead_code` suppressions**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-01 / Steps / 5 (`src/journal/mod.rs`)
    - **Action:**
        1. For methods like `get_editor_cmd`, `get_journal_dir`, `open_entry`, etc., in `src/journal/mod.rs` marked with `#[allow(dead_code)]` and intended only for tests: move these methods into a `#[cfg(test)] mod tests { ... }` block or change their signatures to `#[cfg(test)] pub fn method_name(...)`.
        2. Remove the associated `#[allow(dead_code)]` attributes.
    - **Done‑when:**
        1. Specified `JournalService` test-only `pub` methods are correctly scoped using `#[cfg(test)]`.
        2. Their `#[allow(dead_code)]` attributes are removed.
        3. `cargo clippy --all-targets -- -D warnings` passes regarding these changes.
    - **Depends‑on:** none

- [ ] **T029 · Refactor · P0: perform final `dead_code` sweep and fix any remaining issues**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-01 / Steps / 6
    - **Action:**
        1. Run `cargo clippy --all-targets -- -D warnings`.
        2. For any other `pub` items flagged with `dead_code`, apply similar logic: remove if unused, scope to `#[cfg(test)]` if test-only, or make private/`pub(crate)` if internal.
    - **Done‑when:**
        1. All `#[allow(dead_code)]` attributes on `pub` items are removed from the codebase.
        2. All code previously suppressed by `#[allow(dead_code)]` is correctly scoped, actively used, or removed.
        3. `cargo clippy --all-targets -- -D warnings` runs successfully without any `dead_code` errors.
    - **Depends‑on:** [T025, T026, T027, T028]

## Final Verification
- [ ] **T030 · Test · P1: manually verify operational messages are output via structured logging**
    - **Context:** Remediation Plan – Sprint 1 / Validation Checklist & cr-02 / Done-When
    - **Action:**
        1. Run the application in a way that triggers the log messages modified in T021 and T022.
        2. Confirm that these operational messages are output via the structured logging system in the expected format (e.g., JSON, including timestamp, level, message).
    - **Done‑when:**
        1. Operational messages from T021 and T022 are confirmed to be output correctly via the structured logging system.
    - **Verification:**
        1. Observe log output for correct JSON structure and content for the specified messages.
    - **Depends‑on:** [T021, T022, T023]

- [ ] **T031 · Chore · P0: update `TODO.md` to mark tasks T016-T020 as complete**
    - **Context:** Remediation Plan – Sprint 1 / Detailed Remedies / cr-01 / Done-When
    - **Action:**
        1. After successfully implementing all `dead_code` remediation tasks, update `TODO.md` to mark tasks T016-T020 as complete (`[x]`).
    - **Done‑when:**
        1. Tasks T016-T020 in `TODO.md` are marked as complete (`[x]`).
        2. All tasks from T024-T029 are completed.
    - **Verification:**
        1. `cargo clippy --all-targets -- -D warnings` runs successfully without any `dead_code` errors.
    - **Depends‑on:** [T024, T025, T026, T027, T028, T029]

## Implementation Notes

1. The tickets are organized into logical groups corresponding to the critical issues identified in the code review.
2. Each ticket has clear actions, completion criteria, and dependency information.
3. Tasks T021-T023 address the logging remediation issue by replacing `println!` calls with proper logging macros.
4. Tasks T024-T029 address the `dead_code` violations by properly scoping test-only code and removing suppressions.
5. Tasks T030-T031 provide final verification and update tasks T016-T020 to correctly reflect their completion status.
6. Dependencies are set up to ensure proper sequencing of tasks, especially for the final verification tasks.