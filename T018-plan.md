# T018: Code Formatting and Linting Plan

## Overview

This plan outlines the steps to ensure the codebase adheres to our formatting and linting standards. With the rustfmt configuration issues fixed in T022, we can now reliably apply formatting and address any linting issues across the codebase.

## Approach

### 1. Code Formatting

1. Run `cargo fmt` to apply the formatting rules defined in .rustfmt.toml to all Rust files
2. Verify formatting is correct with `cargo fmt -- --check`

### 2. Code Linting

1. Run `cargo clippy --all-targets -- -D warnings` to:
   - Check all code, including tests and examples
   - Treat all warnings as errors (fail on warnings)
2. Address any issues identified by Clippy

### 3. Validation

1. After making changes, re-run both tools to verify:
   - `cargo fmt -- --check` passes with no formatting issues
   - `cargo clippy --all-targets -- -D warnings` passes with no warnings
2. Run tests to ensure functionality is not affected: `cargo test --all-features`

## Implementation Guidelines

When addressing warnings or issues:

1. **No Suppressions**: Following the Development Philosophy, we will not use `#[allow(...)]` attributes to suppress warnings. Each warning must be properly addressed.

2. **Thoughtful Changes**: When making changes to fix linting issues, ensure they don't alter behavior or break API contracts.

3. **Documentation**: Update comments/documentation if necessary when changes affect documented behavior.

4. **Progressive Approach**: If we find many issues, address them in logical groups (e.g., by module or by warning type) to keep changes manageable.

## Success Criteria

- All code is formatted according to the project's rustfmt configuration
- Running `cargo clippy --all-targets -- -D warnings` produces no warnings
- All tests continue to pass
- Pre-commit hooks pass without errors