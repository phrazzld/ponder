# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

**Build:**
```bash
cargo build
cargo build --release  # For production build
```

**Run tests:**
```bash
cargo test                    # Run all tests
cargo test --bin ponder      # Run tests for main binary
cargo test test_name         # Run a single test by name
cargo test --test cli_tests  # Run specific integration test file
```

**Lint and format:**
```bash
cargo fmt               # Format code
cargo fmt --check      # Check formatting without modifying
cargo clippy --all-targets -- -D warnings  # Run linter with strict settings
```

**Pre-commit hooks:**
```bash
pre-commit install     # Install hooks (requires Python)
pre-commit run --all-files  # Run hooks manually
```

## High-Level Architecture

Ponder is a simple journaling CLI tool built in Rust with a modular architecture focused on simplicity and direct function calls rather than abstractions.

### Core Modules

- **`src/cli/`**: Command-line interface using clap. Handles argument parsing and provides methods to convert between CLI arguments and `DateSpecifier`.

- **`src/config/`**: Configuration management. Loads settings from environment variables (`PONDER_DIR`, `PONDER_EDITOR`, `EDITOR`) with defaults. Validates configuration and ensures paths are properly expanded. Note: Editor commands are strictly validated for security - they must be single commands without spaces, arguments, or shell metacharacters.

- **`src/errors/`**: Error handling infrastructure using `thiserror`. Defines `AppError` and `AppResult` types used throughout the codebase.

- **`src/journal_core/`**: Core journal functionality without I/O operations. Contains:
  - `DateSpecifier`: Enum defining different date selection modes (Today, Retro, Reminisce, Specific)
  - Pure logic functions for date handling and calculations
  - No filesystem or I/O operations

- **`src/journal_io/`**: Journal I/O operations and file management. Contains:
  - `ensure_journal_directory_exists()`: Creates journal directory if needed
  - `open_journal_entries()`: Main function that orchestrates opening journal entries
  - Helper functions for file operations and editor launching
  - All filesystem interactions

- **`src/main.rs`**: Application entry point. Coordinates the flow:
  1. Parse CLI args
  2. Load configuration
  3. Ensure journal directory exists
  4. Convert CLI args to DateSpecifier
  5. Resolve dates based on DateSpecifier
  6. Open journal entries for the resolved dates

### Module Flow

```
main.rs
  ├─> cli/mod.rs (parse CLI args)
  ├─> config/mod.rs (load and validate config)
  ├─> journal_io/mod.rs (ensure journal directory exists)
  ├─> journal_core/mod.rs (create DateSpecifier, resolve dates)
  └─> journal_io/mod.rs (open journal entries)
        ├─> get_entry_path_for_date()
        ├─> append_date_header_if_needed()
        └─> launch_editor()
```

### Key Design Decisions

- Direct function calls instead of trait abstractions for simplicity
- Separation of pure logic (`journal_core`) from I/O operations (`journal_io`)
- All filesystem operations isolated in the `journal_io` module
- Journal entries stored as `YYYYMMDD.md` files in configured directory
- Automatic timestamp header added to new journal files
- Uses system environment for editor configuration
- Strict validation of editor commands for security

## Testing Structure

- Unit tests are colocated with modules using `#[cfg(test)]` modules
- Integration tests in `tests/` directory:
  - `cli_tests.rs`: Tests CLI argument parsing
  - `journal_integration_tests.rs`: Tests full journal operations
  - `config_tests.rs`: Tests configuration loading

Tests use `tempfile` for isolated filesystem operations and mock the `EDITOR` environment variable for testing. Note: Test configurations should use simple editor commands like `echo` or `true` without arguments to pass validation.

## Pre-commit and CI

The project uses pre-commit hooks and GitHub Actions CI:
- Pre-commit runs `cargo fmt --check` and `cargo clippy`
- CI runs formatting, clippy, build, and tests in separate jobs
- Test code must meet same quality standards as production code