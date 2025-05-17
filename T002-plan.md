# T002 Plan: Audit and refactor error handling in journal_logic.rs

## Implementation Approach

1. Review the specified functions in `journal_logic.rs` for `.unwrap()` and `.expect()` calls:
   - `launch_editor`
   - `create_or_open_entry_file` 
   - `read_file_content`
   - `append_to_file`

2. Replace all `.unwrap()` and `.expect()` calls with proper error handling:
   - Use `?` operator for direct propagation
   - Use `map_err()` to convert OS errors to `AppError::Io`
   - Ensure all error paths are properly handled

3. Focus on OS interactions (file operations, process execution) where errors should be propagated rather than panicking

## Expected Changes

Replace panic-inducing calls with proper error propagation to prevent crashes in CI and ensure robust error handling throughout the application.