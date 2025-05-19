# Task Details

**Task ID:** T009
**Title:** Extract DateSpecifier to journal_core

## Original Ticket Text

- Move `DateSpecifier` enum to `src/journal_core/mod.rs`
- Move and rename `from_args` to `from_cli_args(retro: bool, reminisce: bool, date_str: Option<&str>)`
- Move and rename `get_dates` to `resolve_dates(&self, reference_date: NaiveDate)`
- Ensure no I/O or side effects in journal_core
- **Verification**: Pure logic isolated, `cargo check` passes

## Implementation Approach Analysis Prompt

Analyze this refactoring task carefully. We need to extract the DateSpecifier enum and its associated logic from journal_logic.rs to journal_core module, ensuring complete separation of pure logic from I/O operations.

Please provide a comprehensive plan that includes:
1. Complete analysis of what code needs to be moved
2. Dependencies and imports that need to be updated
3. Method signature changes and their impacts
4. How to ensure no I/O operations remain in journal_core
5. All files that will be affected by this change
6. Testing strategy to ensure functionality is preserved

Consider the development philosophy of separating concerns and maintaining pure functions where possible. The journal_core module should contain only pure logic without any filesystem operations.