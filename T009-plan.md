# T009 Plan: Extract DateSpecifier to journal_core

## Objective
Extract DateSpecifier and its pure date calculation logic from journal_logic.rs to journal_core/mod.rs, updating method signatures to ensure strict I/O separation.

## Implementation Strategy

### 1. Module Preparation
- Add necessary imports to `src/journal_core/mod.rs`:
  ```rust
  use chrono::{Datelike, Duration, Months, NaiveDate};
  use crate::errors::AppError;
  ```

### 2. Code Migration
1. Move from `journal_logic.rs` to `journal_core/mod.rs`:
   - `DateSpecifier` enum definition
   - Associated constants (REMINISCE_ONE_MONTH_AGO, etc.)
   - Complete `impl DateSpecifier` block
   - Helper function `parse_date_string`

2. Method Signature Updates:
   - Rename `from_args` to `from_cli_args`
   - Update signature to: `pub fn from_cli_args(retro: bool, reminisce: bool, date_str: Option<&str>) -> Result<Self, AppError>`
   - Rename `get_dates` to `resolve_dates`
   - Update signature to: `pub fn resolve_dates(&self, reference_date: NaiveDate) -> Vec<NaiveDate>`

3. Implementation Changes:
   - `resolve_dates` must use passed `reference_date` instead of `Local::now()`
   - Ensure all date calculations are pure (no I/O or side effects)

### 3. Update Dependent Code
1. In `journal_logic.rs`:
   - Remove moved code
   - Add import: `use crate::journal_core::DateSpecifier;`
   - Update calls to use new method names and signatures

2. In `main.rs`:
   - Import `DateSpecifier` from new location
   - Update function to use `DateSpecifier::from_cli_args`
   - Handle the Result type

3. In `lib.rs`:
   - Uncomment re-export: `pub use journal_core::DateSpecifier;`

### 4. Test Migration and Updates
1. Move unit tests from `journal_logic.rs` to `journal_core/mod.rs`
2. Update tests for new method signatures
3. Add deterministic tests using fixed reference dates
4. Update integration tests to use new import paths

### 5. Files to Modify
- `src/journal_logic.rs` - Remove DateSpecifier code, update imports
- `src/journal_core/mod.rs` - Add DateSpecifier code
- `src/lib.rs` - Update re-exports
- `src/main.rs` - Update imports and method calls
- Test files - Update imports and method calls

### Acceptance Criteria
- DateSpecifier fully moved to journal_core
- No I/O operations in journal_core module
- Method signatures updated as specified
- All tests pass
- `cargo check` succeeds without errors