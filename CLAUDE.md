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

- **`src/cli/`**: Command-line interface using clap. Handles argument parsing and creates `DateSpecifier` from user input.

- **`src/config/`**: Configuration management. Loads settings from environment variables (`PONDER_DIR`, `PONDER_EDITOR`, `EDITOR`) with defaults. Validates configuration and ensures paths are properly expanded.

- **`src/journal_logic.rs`**: Core functionality. This is the main entry point for journal operations. Contains:
  - `open_journal_entries()`: Main function that orchestrates opening journal entries
  - `ensure_journal_directory_exists()`: Creates journal directory if needed
  - `DateSpecifier`: Enum defining different date selection modes (Today, Retro, Reminisce, Specific)
  - Private helper functions for file operations and editor launching

- **`src/errors.rs`**: Error handling infrastructure using `thiserror`. Defines `AppError` and `AppResult` types used throughout the codebase.

- **`src/main.rs`**: Application entry point. Coordinates the flow:
  1. Parse CLI args
  2. Load configuration
  3. Ensure journal directory exists
  4. Open journal entries based on user's date specification

### Module Flow

```
main.rs
  ├─> cli/mod.rs (parse args)
  ├─> config/mod.rs (load config)
  └─> journal_logic.rs (core operations)
        ├─> ensure_journal_directory_exists()
        └─> open_journal_entries()
              ├─> get dates based on DateSpecifier
              ├─> create/open journal files
              └─> launch editor
```

### Key Design Decisions

- Direct function calls instead of trait abstractions for simplicity
- All filesystem operations in `journal_logic.rs` without abstractions
- Journal entries stored as `YYYYMMDD.md` files in configured directory
- Automatic timestamp header added to new journal files
- Uses system environment for editor configuration

## Testing Structure

- Unit tests are colocated with modules using `#[cfg(test)]` modules
- Integration tests in `tests/` directory:
  - `cli_tests.rs`: Tests CLI argument parsing
  - `journal_integration_tests.rs`: Tests full journal operations
  - `config_tests.rs`: Tests configuration loading

Tests use `tempfile` for isolated filesystem operations and mock the `EDITOR` environment variable for testing.

## Pre-commit and CI

The project uses pre-commit hooks and GitHub Actions CI:
- Pre-commit runs `cargo fmt --check` and `cargo clippy`
- CI runs formatting, clippy, build, and tests in separate jobs
- Test code must meet same quality standards as production code