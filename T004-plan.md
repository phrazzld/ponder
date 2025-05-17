# T004 Plan: Verify complete removal of old trait-based abstractions

## Implementation Approach

1. Search for old trait names that should have been removed:
   - `JournalIO` trait
   - `MockJournalIO` implementation
   - `Editor` trait
   - `MockEditor` implementation
   - `JournalService` struct

2. Search for old module references:
   - `src/journal/io/` directory
   - `src/editor.rs` file

3. Search for test-specific public modifiers (`#[cfg(test)] pub`) that were part of the old mocking system

4. Remove any lingering references found

5. Ensure tests still pass after cleanup

## Expected Patterns to Search

- Trait definitions: `trait JournalIO`, `trait Editor`
- Mock implementations: `MockJournalIO`, `MockEditor`
- Service structures: `JournalService`
- Test-specific visibility: `#[cfg(test)] pub`
- Module references: `mod journal`, `mod editor`