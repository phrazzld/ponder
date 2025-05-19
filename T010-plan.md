# Plan: Extract I/O functions to journal_io

## Task Analysis
Task T010 involves extracting I/O functions from the original journal_logic.rs to the new journal_io/mod.rs module. 

Based on my examination of the codebase, it appears that most of this work has already been completed. The functions have been moved to journal_io/mod.rs, and journal_logic.rs now contains just a comment indicating the split.

## Implementation Plan
1. Verify that all required functions have been moved to journal_io/mod.rs with the correct signatures:
   - `ensure_journal_directory_exists(journal_dir: &Path) -> AppResult<()>`
   - `open_journal_entries(config: &config::Config, dates: &[NaiveDate]) -> AppResult<()>`
   - All helper functions made private unless justified

2. Check the implementations to ensure they match the original functionality

3. Verify that journal_logic.rs contains only a comment indicating that the module has been split

4. Run `cargo check` to ensure the code compiles correctly

## Verification
- Run `cargo check` to verify the code compiles
- Run `cargo test` to ensure all tests pass after the changes
- Verify the extracted functions have the correct signatures and behavior