# T015 - Write unit tests for each module

## Task Description
Add comprehensive unit tests for each module in the Ponder application.

## Approach
1. Identify all modules that need testing
2. For each module:
   - Add `#[cfg(test)] mod tests` section if not present
   - Set up appropriate mocks when needed
   - Write tests for core functionality
   - Test error conditions and edge cases

## Implementation Plan
1. Review existing tests to understand current coverage
2. Prioritize testing core modules:
   - config
   - journal (including DateSpecifier)
   - cli
   - editor
   - errors
3. Create or expand test modules for each
4. Run all tests to verify changes

## Success Criteria
- Each module has comprehensive unit tests
- All tests pass
- Code coverage is sufficient
- Edge cases and error conditions are tested