# Todo

## Security Remediation - cr-01 (Editor Command Injection)
- [x] **T001 · bugfix · P0: implement strict validation for editor command string in config loading**
    - **Context:** PLAN.md / Detailed Remedies / cr‑01 Command Injection Vulnerability via Editor Configuration / Steps / 1. Modify Configuration Loading (`src/config/mod.rs`)
    - **Action:**
        1. In `src/config/mod.rs`, implement logic to load `PONDER_EDITOR` or `EDITOR` environment variables.
        2. Validate that the loaded string is not empty, contains NO shell metacharacters (e.g., `|`, `&`, `;`, `$`, `(`, `)`, `` ` ``, `\`, `<`, `>`, `'`, `"`), and contains NO spaces.
        3. If any validation fails, ensure Ponder returns a clear `AppError::Config` error and refuses to start, explaining the restriction.
    - **Done‑when:**
        1. Code implementing strict validation of `editor_cmd` (rejecting empty strings, shell metacharacters, and spaces) is merged into `src/config/mod.rs`.
        2. Improperly formatted editor strings are rejected at startup with a clear `AppError::Config` error message.
    - **Verification:**
        1. Set `PONDER_EDITOR="vim --noplugin"`. Run Ponder. Verify it exits with `AppError::Config` and an explanatory message about spaces/arguments.
        2. Set `PONDER_EDITOR="echo pwned > /tmp/file"`. Run Ponder. Verify it exits with `AppError::Config` and an explanatory message about metacharacters/spaces.
        3. Set `PONDER_EDITOR` to an empty string. Run Ponder. Verify it exits with `AppError::Config` and an explanatory message about empty string.
    - **Depends‑on:** none

- [x] **T002 · bugfix · P0: update editor launch logic to use pre-validated command**
    - **Context:** PLAN.md / Detailed Remedies / cr‑01 Command Injection Vulnerability via Editor Configuration / Steps / 2. Update Editor Launch Logic (`src/journal_logic.rs::launch_editor`)
    - **Action:**
        1. Modify `src/journal_logic.rs::launch_editor` to receive the already validated editor command string from the configuration.
        2. Pass this string directly to `std::process::Command::new()` as the executable to be run.
        3. Ensure no further parsing, shell interpretation, or argument splitting occurs on the command string within `launch_editor`.
    - **Done‑when:**
        1. `launch_editor` uses the validated editor command string safely by passing it directly to `std::process::Command::new()`.
        2. The application correctly attempts to launch a valid, single-word editor command when required.
    - **Verification:**
        1. Using a valid `PONDER_EDITOR` (e.g., a path to a test script that simply exits with status 0), trigger an action that launches the editor. Verify the script is executed without unexpected arguments or shell interpretation, by checking for expected side-effects of the script or its exit code.
    - **Depends‑on:** [T001]

- [ ] **T003 · chore · P1: update documentation for editor command restrictions and workarounds**
    - **Context:** PLAN.md / Detailed Remedies / cr‑01 Command Injection Vulnerability via Editor Configuration / Steps / 3. Update Documentation
    - **Action:**
        1. Update `README.md` and any relevant configuration guides to clearly state that `PONDER_EDITOR`/`EDITOR` must be a single command or an absolute/relative path to an executable, without any embedded spaces or arguments.
        2. Provide examples of how users can use OS-level shell aliases or simple wrapper scripts if they need to pass default arguments or handle complex paths.
    - **Done‑when:**
        1. Documentation accurately reflects the new restrictions for `PONDER_EDITOR`/`EDITOR`.
        2. User guidance with examples for using shell aliases or wrapper scripts is included and clear.
    - **Verification:**
        1. Review the updated `README.md` and any other configuration documentation for clarity, accuracy, and completeness regarding the new editor command rules and suggested workarounds.
    - **Depends‑on:** [T001]

- [ ] **T004 · test · P0: add integration tests for editor command validation and launch**
    - **Context:** PLAN.md / Detailed Remedies / cr‑01 Command Injection Vulnerability via Editor Configuration / Steps / 4. Add Integration Tests
    - **Action:**
        1. Create integration test: `PONDER_EDITOR="vim --noplugin"`. Verify Ponder rejects this configuration at startup with an `AppError::Config`.
        2. Create integration test: `PONDER_EDITOR="echo hello > /tmp/pwned"`. Verify Ponder rejects this configuration at startup and ensure `/tmp/pwned` is NOT created.
        3. Create integration test: `PONDER_EDITOR="sh -c 'touch /tmp/pwned_shell'"`. Verify Ponder rejects this configuration at startup and ensure `/tmp/pwned_shell` is NOT created.
        4. Create integration test: `PONDER_EDITOR` is set to a valid single command (e.g., a test script that creates a sentinel file and exits successfully). Verify Ponder attempts to launch it correctly when an action requiring the editor is triggered (check for sentinel file).
        5. Create integration test: `PONDER_EDITOR` is set to an empty string. Verify Ponder rejects this configuration at startup with an `AppError::Config`.
    - **Done‑when:**
        1. All new integration tests are implemented and pass.
        2. Tests confirm rejection of invalid inputs at startup and safe invocation attempt of valid inputs by `launch_editor`.
    - **Verification:**
        1. Execute the entire test suite and confirm all new tests pass.
        2. For tests involving potentially malicious commands, manually verify that no unintended side effects (like file creation in `/tmp/`) occurred.
    - **Depends‑on:** [T001, T002]

---

## Clarifications & Assumptions
- [ ] **T005 · chore · P0: obtain full remediation plan details for cr-02 path traversal**
    - **Context:** PLAN.md / Detailed Remedies / cr‑02 Path Traversal Vulnerability via Date Input (section is incomplete)
    - **Action:**
        1. Request the complete "Detailed Remedies" section for cr-02, including "Impact," "Chosen Fix," "Steps," and "Done‑When" criteria.
    - **Done‑when:**
        1. The complete remediation plan details for cr-02 are provided.
    - **Verification:**
        1. Review the provided details to confirm they are sufficient for breaking down cr-02 into actionable engineering tickets.
    - **Depends‑on:** none
    - **Issue:** The remediation plan for `cr-02 Path Traversal Vulnerability via Date Input` is truncated. Full details (Impact, Chosen Fix, Steps, Done-When) are missing.
    - **Blocking?:** yes (for creating detailed tasks for cr-02)
```