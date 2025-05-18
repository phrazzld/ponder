# Todo

## Config Module Test Fixes
- [x] **T001 · Bugfix · P0: apply #[serial] attribute to fix config test interference**
    - **Context:** Root Cause Analysis; Resolution Steps > 1. Fix Test Interference by Enforcing Serial Execution
    - **Action:**
        1. In `src/config/mod.rs`, add `use serial_test::serial;` to the tests module imports.
        2. Add the `#[serial]` attribute to these test functions: `test_load_with_default_editor`, `test_load_with_editor_env`, `test_load_with_custom_dir`, and `test_load_config_with_invalid_editor`.
    - **Done‑when:**
        1. Code changes are implemented in `src/config/mod.rs`.
        2. `cargo test` passes locally multiple times consistently for the `config::tests` module.
        3. CI build job passes successfully after changes are pushed.
    - **Verification:**
        1. Run `cargo test -- --test-threads=1 && cargo test -- --test-threads=X` (where X > 1) locally multiple times, ensuring all tests in `config::tests` pass consistently.
        2. Push changes to a PR branch and monitor the CI build job to confirm it passes.
    - **Depends‑on:** none

## Developer Documentation & Guidelines
- [ ] **T002 · Chore · P2: update contributing guidelines for global state test isolation**
    - **Context:** Prevention Measures > 1. Update Guidelines; Next Steps > 3. Update team documentation
    - **Action:**
        1. Edit the project's contributing documentation (e.g., `CONTRIBUTING.md`).
        2. Add a guideline stating that tests modifying global state (e.g., environment variables) must use `#[serial]` to ensure isolation and prevent parallel execution issues.
    - **Done‑when:**
        1. Contributing documentation is updated with the new guideline.
        2. The rationale for using `#[serial]` is briefly explained.
    - **Depends‑on:** none

- [ ] **T003 · Chore · P2: add pr checklist item for global state test isolation**
    - **Context:** Prevention Measures > 2. PR Checklist
    - **Action:**
        1. Update the project's Pull Request template.
        2. Add a checklist item for authors/reviewers to verify that tests modifying global state are properly isolated (e.g., using `#[serial]`).
    - **Done‑when:**
        1. PR template includes the new checklist item.
    - **Depends‑on:** [T002]

## Team Process & Education
- [ ] **T004 · Chore · P2: educate team on parallel test execution and #[serial] usage**
    - **Context:** Prevention Measures > 3. Team Education; Next Steps > 3. Update team documentation
    - **Action:**
        1. Prepare and share educational material (e.g., internal wiki page, short presentation) explaining Rust's parallel test execution, associated risks with global state, and the correct usage of `serial_test::serial`.
    - **Done‑when:**
        1. Educational material is created and disseminated to the development team.
        2. Team members acknowledge receipt or understanding.
    - **Depends‑on:** [T002]

- [ ] **T005 · Chore · P2: establish process for regular review of flaky tests**
    - **Context:** Prevention Measures > 5. Regular Reviews
    - **Action:**
        1. Define and document a process for regularly monitoring CI test results for flaky tests.
        2. Outline steps for investigating identified flaky tests, with a focus on potential race conditions or global state interference.
    - **Done‑when:**
        1. A documented process for monitoring and investigating flaky tests is established.
        2. The process is communicated to the team.
    - **Depends‑on:** none

## Test Infrastructure Enhancements
- [ ] **T006 · Chore · P3: investigate and propose TestEnvGuard utility for env var management**
    - **Context:** Prevention Measures > 4. Test Utilities
    - **Action:**
        1. Research existing patterns or crates for safer environment variable manipulation in Rust tests.
        2. Draft a proposal (e.g., internal document, GitHub issue) for a `TestEnvGuard` utility, outlining its potential design, benefits, and an example of use.
    - **Done‑when:**
        1. Research findings are documented.
        2. A proposal for `TestEnvGuard` is created and shared for team review.
    - **Depends‑on:** none