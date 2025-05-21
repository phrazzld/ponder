# BACKLOG

## Critical Priority

### Architectural Foundation & Core Principles

-   **[Refactor]**: Enforce Strict Module Boundaries and Feature-Based Organization
    -   **Complexity**: Medium
    -   **Rationale**: **CRITICAL** for "Modularity is Mandatory" and "Architecture Guidelines (Package/Module Structure)" (Dev Philosophy). A clear, feature-oriented module structure improves maintainability, navigability, and separation of concerns, making the codebase easier to understand and evolve.
    -   **Expected Outcome**: The `src/` directory and module structure are reviewed and refactored to strictly adhere to "Package by Feature, Not Type." Modules exhibit high cohesion, minimal public APIs, and clear responsibilities aligned with distinct application features or domains (e.g., `cli`, `config`, `journal_operations`, `error_handling`).
    -   **Dependencies**: None. Facilitates many other refactoring and feature development tasks.

### Testing Strategy

-   **[Enhancement]**: Implement Behavior-Driven Integration Tests for All Core User Flows
    -   **Complexity**: Medium
    -   **Rationale**: **CRITICAL** for "Design for Testability (Integration Testing Over Mock-Based Unit Tests, Verify Behavior)" and "Testing Strategy (Integration / Workflow Tests)" (Dev Philosophy). Ensures the application works as expected from a user's perspective, increasing confidence in releases and reducing reliance on internal mocks.
    -   **Expected Outcome**: A robust suite of integration tests (e.g., using `assert_cmd` and temporary filesystem fixtures) covering all primary user stories and CLI commands (e.g., new entry for today/specific date/retro, reminisce, edit). These tests will replace existing mock-based unit tests for core logic.
    -   **Dependencies**: Eliminate Internal Mocking and Unnecessary Trait Abstractions in Core Logic.

## High Priority

### Error Handling & Observability

-   **[Refactor]**: Standardize Error Handling, Eliminate Panics, and Prevent Double Logging
    -   **Complexity**: Medium
    -   **Rationale**: Addresses "Inconsistent error handling and logging" and aligns with "Consistent Error Handling" and "Robustness" (Dev Philosophy). Current `unwrap()`/`expect()` usage and error logging patterns lead to fragile application behavior and cluttered logs.
    -   **Expected Outcome**: All `unwrap()`/`expect()` calls on `Result`/`Option` in recoverable paths are replaced with `?` or proper error handling. Custom error types using `thiserror` provide clear context. Errors are propagated cleanly and logged only once at the application boundary (e.g., in `main.rs`). The fragile `Clone` impl for `AppError::Io` is fixed or `Clone` is removed if not needed. `main` returns `Result`.
    -   **Implementation Steps**:
        - Remove or fix `Clone` implementation for `AppError`.
        - Utilize `thiserror` for comprehensive error context and source chaining.
        - Eliminate `.map_err(|e| { error!(...); e })` pattern and similar double logging.
        - Ensure errors are logged exclusively at the top level (e.g., `main.rs`).
        - Systematically replace all `unwrap()`/`expect()` calls with robust error handling.
    -   **Dependencies**: None. Enables "Implement Structured, Contextual Logging with `tracing`" and "Implement Comprehensive Error Handling for External Command Execution (Editor)".

-   **[Enhancement]**: Implement Comprehensive Error Handling for External Command Execution (Editor)
    -   **Complexity**: Medium
    -   **Rationale**: Critical for robustness and user experience when interacting with user-configured editors. Aligns with "Consistent Error Handling" and "API Design (for CLIs - clear error reporting)".
    -   **Expected Outcome**: Enhanced error handling for launching the external editor (`std::process::Command`). Capture and report distinct error types (e.g., command not found, non-zero exit status, permission issues) as specific `AppError` variants with user-friendly messages.
    -   **Dependencies**: Standardize Error Handling, Eliminate Panics, and Prevent Double Logging.

-   **[Enhancement]**: Implement Structured, Contextual Logging with `tracing`
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Logging Strategy" ("Structured Logging is Mandatory", "Context Propagation"). `tracing` provides richer, structured logs essential for debugging and observability, replacing basic `env_logger`.
    -   **Expected Outcome**: `tracing` and `tracing-subscriber` are integrated. Logs are output in JSON format (especially in CI/non-interactive environments). All log entries include mandatory context fields: `timestamp`, `level`, `message`, `target` (module path), `service_name` (static "ponder"), `correlation_id` (unique ID generated per application invocation), and `error_details` (for ERROR level logs, including error source chain).
    -   **Implementation Steps**:
        - Add dependencies for `tracing`, `tracing-subscriber`, and related crates.
        - Implement correlation ID generation per application invocation.
        - Configure JSON output with all mandatory context fields.
        - Ensure logging occurs only at appropriate boundaries (e.g., `main.rs` for top-level errors, service boundaries for key operations).
        - Statically define `service_name` in log output.
    -   **Dependencies**: Standardize Error Handling, Eliminate Panics, and Prevent Double Logging.

### Developer Workflow & Automation

-   **[Enhancement]**: Implement CI Pipeline for Automated Checks (e.g., GitHub Actions)
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Automate Everything", "Quality Gates", and "CI/CD" philosophy. Essential for consistent code quality, early regression detection, and automating repetitive checks.
    -   **Expected Outcome**: A CI workflow (e.g., in `.github/workflows`) runs on every push and pull request, executing:
        1.  Code formatting check (`cargo fmt --check`).
        2.  Strict linting (`cargo clippy --all-targets -- -D warnings`).
        3.  Automated tests (`cargo test --all-features`), including integration tests.
        4.  Test coverage check and reporting (e.g., using `cargo-tarpaulin` or `grcov`), failing if below a defined threshold.
        5.  Dependency vulnerability scanning (`cargo audit`).
        Builds fail on any violation. A build status badge is added to `README.md`.
    -   **Dependencies**: Enforce Consistent Code Formatting, Enforce Strict Code Linting, Implement Behavior-Driven Integration Tests for All Core User Flows.

-   **[Enhancement]**: Setup Local Pre-commit Hooks
    -   **Complexity**: Simple
    -   **Rationale**: Aligns with "Local Development: Pre-commit Hooks" philosophy. Catches issues locally before commit, reducing CI failures and providing faster feedback.
    -   **Expected Outcome**: A pre-commit framework (e.g., `pre-commit` with Rust support) is configured to run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo check`, and conventional commit message linting before each commit. Commits are blocked if hooks fail.
    -   **Implementation Steps**:
        - Install and configure a pre-commit framework (e.g., `pre-commit`).
        - Add hooks for formatting, linting, and type checking.
        - Integrate conventional commit linting (e.g., `commitlint`).
        - Optionally, add hooks to prevent commit of sensitive data or large files.
        - Document usage for contributors.
    -   **Dependencies**: Enforce Consistent Code Formatting, Enforce Strict Code Linting, Adopt Conventional Commits and Automate Changelog Generation.

-   **[Enhancement]**: Enforce Consistent Code Formatting (`rustfmt`)
    -   **Complexity**: Simple
    -   **Rationale**: Aligns with "Code Quality" and Rust Appendix (Tooling, Formatting). Ensures a consistent code style, improving readability.
    -   **Expected Outcome**: `rustfmt` is configured (default or with `rustfmt.toml`). All existing code is formatted. Formatting is enforced by pre-commit hooks and CI checks.
    -   **Dependencies**: None. Integrated into "Implement CI Pipeline" and "Setup Local Pre-commit Hooks".

-   **[Enhancement]**: Enforce Strict Code Linting (`clippy`)
    -   **Complexity**: Simple
    -   **Rationale**: Aligns with "Code Quality" and Rust Appendix (Linting). `clippy` catches common pitfalls, potential bugs, and style issues.
    -   **Expected Outcome**: `clippy` is configured for strictness (e.g., `cargo clippy --all-targets -- -D warnings`). All `clippy` checks pass in pre-commit hooks and CI. Existing lints are fixed.
    -   **Dependencies**: None. Integrated into "Implement CI Pipeline" and "Setup Local Pre-commit Hooks".

-   **[Enhancement]**: Adopt Conventional Commits and Automate Changelog Generation
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Semantic Versioning and Release Automation" philosophy. Improves commit history readability and enables automated changelog generation and versioning.
    -   **Expected Outcome**: Project adopts Conventional Commits standard. Commit message linting (e.g., `commitlint`) integrated into pre-commit hooks. Tooling (e.g., `git-cliff`) configured to generate/update `CHANGELOG.md` from commit history.
    -   **Dependencies**: Setup Local Pre-commit Hooks. Enables "Automate Release Process".

-   **[Enhancement]**: Automate Release Process (Semantic Release)
    -   **Complexity**: Medium
    -   **Rationale**: Streamlines the release process, ensures consistent versioning, and automates the creation of release artifacts and publishing, aligning with "Automate Everything".
    -   **Expected Outcome**: An automated process (e.g., using GitHub Actions with tools like `semantic-release` or equivalent Rust ecosystem tools) for version bumping, Git tagging, changelog updating, and publishing binaries to GitHub Releases.
    -   **Dependencies**: Adopt Conventional Commits and Automate Changelog Generation, Implement CI Pipeline for Automated Checks, Build Optimized Release Binaries (Static/Cross-Platform) in CI.

### User-Facing Features

-   **[Feature]**: Encryption for Journal Files (at rest)
    -   **Complexity**: Complex
    -   **Rationale**: Enhances user privacy and data security (User Value Prioritization #3, Manifesto 1). Implemented by wrapping established external tools (Manifesto 3.4, 4), prioritizing security and simplicity.
    -   **Expected Outcome**: Option to encrypt journal files (e.g., using a library like `age` or by shelling out to `age` CLI). Requires a password or key to decrypt and access entries. Clear CLI commands/options for managing encrypted journals (e.g., initial encryption, decrypting for edit, re-encrypting). Secure key/passphrase handling.
    -   **Implementation Steps**:
        - Research and select robust encryption approach (e.g., `age` library or CLI wrapper).
        - Integrate encryption/decryption into file read/write operations.
        - Implement secure password/key management (e.g., OS keychain or user-provided).
        - Update CLI to support creating/managing encrypted journals.
        - Document encrypted journal usage and security considerations.
    -   **Dependencies**: Implement Externalized Configuration Management, Bolster Security Practices.

-   **[Feature]**: Support for Cross-Device Sync
    -   **Complexity**: Complex
    -   **Rationale**: Enables users to maintain their journal across multiple devices (Manifesto 1), enhancing utility and accessibility. Implemented by leveraging established external tools (Manifesto 3.4, 4) like Git.
    -   **Expected Outcome**: Journal entries can be synchronized between multiple devices using a user-configured Git repository. Simple CLI commands (e.g., `ponder sync pull`, `ponder sync push`) to manage synchronization. Basic conflict resolution strategy (e.g., defer to Git, notify user). Clear documentation for setup and usage.
    -   **Implementation Steps**:
        - Design sync mechanism leveraging Git.
        - Implement sync commands in CLI (wrapper around Git commands).
        - Add conflict detection and user notification.
        - Document sync setup, usage, and potential conflict resolution steps for users.
        - Ensure secure sync when combined with encryption (e.g., encrypt before commit, decrypt after pull).
    -   **Dependencies**: Implement Externalized Configuration Management. Consider with "Encryption for Journal Files" for secure sync.

## Medium Priority

### Configuration & CLI

-   **[Feature]**: Implement Externalized Configuration Management
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Configuration Management" philosophy. Hardcoded paths and sole reliance on `$EDITOR` limit user flexibility. Externalized configuration improves usability, adaptability, and is a prerequisite for features like encryption, sync, and custom reminisce intervals.
    -   **Expected Outcome**: Journal directory path, default editor fallback, and potentially other settings (e.g., date formats, reminisce intervals, sync repository path) are configurable via environment variables (e.g., `PONDER_DIR`, `PONDER_EDITOR`) and/or a configuration file (e.g., XDG compliant `~/.config/ponder/config.toml`). Sensible defaults are used if no configuration is found. Configuration loading logic is centralized and documented.
    -   **Implementation Steps**:
        - Define configuration file format (e.g., TOML).
        - Implement config file loading logic (respecting XDG base directory spec).
        - Add environment variable overrides for configuration options.
        - Establish clear precedence (e.g., CLI args > env vars > config file > defaults).
        - Set sensible defaults for all options.
        - Document all configuration options and their sources.
    -   **Dependencies**: Enforce Strict Module Boundaries and Feature-Based Organization (to have a clear config module).

-   **[Enhancement]**: Apply Newtype Pattern for Sensitive or Validated Configuration Strings
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Coding Standards (Leverage Types Diligently)" and "Security Considerations (Input Validation)". Enhances type safety for configuration values like editor commands and paths, centralizing validation.
    -   **Expected Outcome**: Configuration fields like `Config::editor` and potentially file paths are refactored from `String`/`PathBuf` to dedicated newtypes (e.g., `EditorCommand(String)`, `JournalPath(PathBuf)`). Validation logic (e.g., `TryFrom<String>` or a `new` method enforcing constraints) is encapsulated within the newtype.
    -   **Dependencies**: Implement Externalized Configuration Management.

-   **[Enhancement]**: Improve CLI Argument Parsing and Help Messages
    -   **Complexity**: Simple
    -   **Rationale**: Aligns with "API Design (for CLIs)" and improves "Enhanced usability" (#4 user value). Clear CLI is essential for user experience and feature discoverability.
    -   **Expected Outcome**: `clap` struct definitions are reviewed and enhanced for clarity and consistency. All options and subcommands have comprehensive, user-friendly help messages, including examples where appropriate. Argument naming follows standard conventions. Redundant wrappers like `CliArgs::parse()` are removed in favor of direct `clap::Parser::parse()`.
    -   **Implementation Steps**:
        - Review and refactor `clap` struct definitions for clarity and conciseness.
        - Add detailed `long_help` messages and practical examples for all subcommands and complex options.
        - Ensure argument naming is consistent and follows common CLI conventions.
        - Replace custom `CliArgs::parse()` with direct usage of `clap::Parser::parse()`.
    -   **Dependencies**: Enforce Strict Module Boundaries and Feature-Based Organization (ensures CLI module is well-defined).

### Security & Robustness

-   **[Enhancement]**: Bolster Security Practices (Secure File Handling, Env Var Validation)
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Security Considerations" and Rust Appendix (`cargo audit`). Proactively addresses security, ensures supply chain integrity, and improves robustness of file operations and environment interactions.
    -   **Expected Outcome**:
        -   `cargo audit` integrated into the CI pipeline (covered by "Implement CI Pipeline").
        -   All file path constructions use `std::path::PathBuf` for cross-platform compatibility and safety, avoiding string concatenation.
        -   Application robustly checks for and creates the notes directory (`std::fs::create_dir_all`) if it doesn't exist before attempting file operations.
        -   Defensive handling and validation of critical environment variables (e.g., `HOME`, `EDITOR`, `PONDER_DIR`) beyond basic presence checks, considering potential malicious inputs.
    -   **Implementation Steps**:
        - Add `cargo-audit` to CI pipeline (if not already covered).
        - Review all file path manipulation logic for `PathBuf` usage and safety.
        - Ensure directory creation is robust and handles pre-existing directories gracefully.
        - Implement validation logic for environment variables during configuration loading.
    -   **Dependencies**: Implement CI Pipeline for Automated Checks.

-   **[Enhancement]**: Implement Custom `Debug` for Sensitive Structs & Secure Logging
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Logging Strategy" ("What NOT to Log") and "Security Considerations". Prevents accidental leakage of sensitive data (paths, editor commands, config values, PII) in logs.
    -   **Expected Outcome**: Structs like `CliArgs`, `Config`, and any error types containing paths or commands have custom `std::fmt::Debug` implementations that redact or summarize sensitive fields. Logs are reviewed to ensure no PII or sensitive operational data is inadvertently exposed.
    -   **Implementation Steps**:
        - Identify all structs and enums that might handle or store sensitive data.
        - Implement `std::fmt::Debug` manually for these types, redacting or summarizing sensitive fields.
        - Review all logging statements to ensure they don't inadvertently log sensitive information directly.
        - Test logging output to verify sensitive data is properly handled.
        - Document this practice for future development.
    -   **Dependencies**: Implement Structured, Contextual Logging with `tracing`.

-   **[Enhancement]**: Add Security-Focused Integration Tests for Editor Command Injection
    -   **Complexity**: Medium
    -   **Rationale**: Proactively test against vulnerabilities related to editor command construction and execution, as per "Security Considerations (Input Validation, Security Through Simplicity)".
    -   **Expected Outcome**: Specific integration tests designed to probe for command injection vulnerabilities through crafted editor strings in configuration. Tests verify that the application correctly sanitizes or rejects malicious configurations.
    -   **Dependencies**: Implement Behavior-Driven Integration Tests for All Core User Flows, Implement Externalized Configuration Management.

### Documentation & Developer Experience

-   **[Enhancement]**: Improve Code and Project Documentation (Comprehensive)
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Documentation Approach" philosophy. Essential for maintainability, onboarding new contributors, and user adoption.
    -   **Expected Outcome**:
        -   `README.md`: Fully updated per Dev Philosophy (Title, Purpose, Badges, Features, Installation, Build, Test, Usage, Configuration, High-Level Architecture, Contributing, License).
        -   `CONTRIBUTING.md`: Created/Updated, detailing development workflow, coding style, testing strategy, pre-commit hook usage, and pull request process.
        -   Rustdoc comments (`///`) for all public API elements (modules, functions, structs, enums, traits).
        -   Inline comments (`//`) explaining "the why" for non-obvious logic or important decisions.
    -   **Dependencies**: Enforce Strict Module Boundaries and Feature-Based Organization (clarifies public API for Rustdoc).

-   **[Enhancement]**: Implement Doc Tests for All Public API Elements
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Documentation Approach (Prioritize Self-Documenting Code, Code Comments)" and "Testing Strategy (Doc Tests)". Ensures examples are practical, verify core functionality, and keep documentation synchronized with code.
    -   **Expected Outcome**: All public functions, methods, structs, and enums exposed by the library crate and any public modules have Rust documentation tests (`/// # Examples`). Examples are practical and verify core functionality.
    -   **Dependencies**: Improve Code and Project Documentation (Comprehensive).

-   **[Enhancement]**: Build Optimized Release Binaries (Static/Cross-Platform) in CI
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Automation, Quality Gates, and CI/CD" and "Builds and Deployment Artifacts (Rust Appendix - Static Binaries)". Makes the application easily distributable for users on various platforms.
    -   **Expected Outcome**: The CI pipeline is extended (or a separate release workflow created) to build optimized, statically-linked release binaries for key target platforms (e.g., x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc). These binaries are attached to GitHub Releases.
    -   **Dependencies**: Implement CI Pipeline for Automated Checks. Integrated with "Automate Release Process".

### Code Quality & Maintainability

-   **[Refactor]**: Extract Pure Functions and Isolate Side Effects in Core Logic
    -   **Complexity**: Medium
    -   **Rationale**: Aligns with "Prioritize Pure Functions," "Separation of Concerns," and "Design for Testability." Increases testability of pure logic in isolation, clarifies responsibilities, and makes side effects easier to manage.
    -   **Expected Outcome**: Core modules (`journal_core` and `journal_io`) are refactored to further separate pure computational logic (e.g., date calculations, text manipulation, filename generation) from functions with side effects (e.g., filesystem access, process spawning). Pure functions are moved into well-defined scopes, potentially new sub-modules.
    -   **Dependencies**: Eliminate Internal Mocking and Unnecessary Trait Abstractions in Core Logic.

-   **[Fix]**: Awkward Test-Only Public Struct Fields and Public Methods
    -   **Complexity**: Medium
    -   **Rationale**: Making fields and methods conditionally public with `#[cfg(test)] pub` complicates the API, exposes internals unnecessarily, and can be avoided with better test design that focuses on public interfaces.
    -   **Expected Outcome**: Cleaner API without test-only public fields/methods. Test-specific functionality (e.g., `Config::default()` for tests) moved to appropriate test modules or test helper functions, or tests refactored to not need internal access.
    -   **Dependencies**: Implement Behavior-Driven Integration Tests for All Core User Flows.

-   **[Fix]**: Fragile `Clone` Implementation for `AppError::Io`
    -   **Complexity**: Medium
    -   **Rationale**: The current manual `Clone` implementation for `AppError::Io` is lossy. Error details should be preserved if cloning is necessary, or cloning should be reconsidered.
    -   **Expected Outcome**: `AppError::Io` correctly handles cloning (e.g., by storing `io::ErrorKind` and a `String` message, or wrapping the original `std::io::Error` in an `Arc` if full fidelity is needed and cloning `AppError` is essential). Alternatively, `AppError` no longer derives `Clone` if not truly required. This item might be fully addressed by "Standardize Error Handling..." but is kept for specific focus if needed.
    -   **Dependencies**: Standardize Error Handling, Eliminate Panics, and Prevent Double Logging.

-   **[Refactor]**: Replace Magic Strings and Numbers with Constants
    -   **Complexity**: Simple
    -   **Rationale**: Improves code readability, maintainability, and reduces the risk of typos or inconsistencies by using named constants.
    -   **Expected Outcome**: Key literal values (e.g., default editor name, directory names, file extensions, reminisce default offsets) are defined as well-named constants in appropriate scopes (e.g., a `constants.rs` module or within relevant modules).
    -   **Dependencies**: Enforce Strict Module Boundaries and Feature-Based Organization (provides logical places for constants).

-   **[Fix]**: Complete Refactoring of Deprecated `Config::ensure_journal_dir`
    -   **Complexity**: Simple
    -   **Rationale**: Aligns with "Core Principles (Simplicity First)" by removing dead/deprecated code, reducing codebase size and potential for confusion.
    -   **Expected Outcome**: The deprecated `Config::ensure_journal_dir` method is removed from `src/config/mod.rs`. All call sites, particularly in tests, are updated to use the designated alternative (likely `journal_io::ensure_journal_directory_exists` or similar).
    -   **Dependencies**: Enforce Strict Module Boundaries and Feature-Based Organization.

## Low Priority

### Usability Enhancements & Minor Features

-   **[Feature]**: Enhanced Reminisce Mode with Configurable Intervals
    -   **Complexity**: Medium
    -   **Rationale**: Increases user value by making the "reminisce" feature more flexible and powerful, allowing users to tailor it to their preferred reflection cadences (User Value Prioritization #4).
    -   **Expected Outcome**: Users can configure reminisce intervals (e.g., "1 week ago", "1 month ago", "3 months ago", "1 year ago") via CLI options or the configuration file. The tool can open multiple past entries if they exist for the specified intervals.
    -   **Dependencies**: Implement Externalized Configuration Management.

### Code Quality & Developer Experience

-   **[Enhancement]**: Specify Rust Edition and MSRV (Minimum Supported Rust Version) in `Cargo.toml` & CI
    -   **Complexity**: Simple
    -   **Rationale**: Aligns with Rust Appendix (Tooling and Environment). Clearly defines the project's language features and supported Rust versions, ensuring build consistency for contributors.
    -   **Expected Outcome**: `Cargo.toml` explicitly defines the `edition` (e.g., "2021") and `rust-version` (e.g., "1.70"). CI includes a job that builds/tests using this specific MSRV.
    -   **Dependencies**: Implement CI Pipeline for Automated Checks.

-   **[Enhancement]**: Enforce File and Function Length Guidelines
    -   **Complexity**: Simple
    -   **Rationale**: Aligns with Dev Philosophy ("Adhere to Length Guidelines", "Maintainability"). Promotes better code organization, readability, and discourages overly complex components.
    -   **Expected Outcome**: `clippy.toml` configured to
