# Plan: Update CLI module

## Task Analysis
Task T013 involves refining the CLI module to ensure it only handles argument parsing, with business logic being invoked from main.rs. The main part of this task is moving any business logic out of the CLI module to maintain a clean separation of concerns.

## Current State
- The CLI module (`src/cli/mod.rs`) already primarily focuses on argument parsing using `clap`
- It defines the `CliArgs` struct for storing parsed command-line arguments
- It implements a `parse()` method and a test-only `parse_date()` method
- The `get_date_specifier_from_args` function is currently in main.rs, but should arguably be part of the CLI module since it directly deals with command-line arguments

## Implementation Plan
1. Review the CLI module to ensure it only handles argument parsing
   - The current code already seems well-focused on parsing arguments
   - The `parse_date()` method could be made public (not just for tests) to help with date parsing

2. Move the `get_date_specifier_from_args` function from main.rs to the CLI module
   - This function converts CLI arguments to a `DateSpecifier` and is a logical part of the CLI functionality
   - Update to use the existing test-only `parse_date()` method if appropriate

3. Update main.rs to use the updated CLI API
   - Change main.rs to call the moved function from the CLI module

## Verification
- Run `cargo check` to ensure the code compiles
- Run `cargo test` to ensure all tests pass
- Verify that the CLI module only handles argument parsing and the main.rs properly delegates to it