# T015 Plan: Standardize Date/Time Handling

## Task Description
- Obtain current date once at high level and pass it down
- Use consistent method for getting date/time values
- **Verification**: Consistent date handling throughout code

## Current Issues
- Current date/time is obtained using `Local::now()` in multiple places:
  - `src/main.rs` (line 79, 115)
  - `src/journal_io/mod.rs` (line 522-523)
- This creates inconsistency and makes testing difficult

## Implementation Plan

1. **Modify src/main.rs**:
   - Obtain the current date once at the start of the main function
   - Pass this date down to all functions that need it
   - Use the obtained date for both logging timestamp and resolving dates

2. **Modify src/journal_io/mod.rs**:
   - Update `append_date_header_if_needed` to accept a reference date parameter
   - Pass the date from main.rs instead of getting it inside the function
   - This will make the function more testable and consistent

3. **Additional Improvements**:
   - Ensure all date formatting is consistent throughout the application
   - Add docstrings explaining the date handling approach
   - Update any tests that might be affected by these changes

## Verification Steps
- Run all tests to ensure functionality is preserved
- Verify that the current date is obtained only once in main.rs
- Verify that all functions use the passed date reference
- Run linting and formatting checks