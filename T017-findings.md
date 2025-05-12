# T017 Findings: `#[allow(dead_code)]` in src/config/mod.rs

## Instances Found

1. **Instance #1** - Line 98-99:
   ```rust
   #[allow(dead_code)]
   pub fn new() -> Self {
   ```

   **Analysis:**
   - This method is used exclusively in tests:
     - The comment on line 97 mentions "Note: For normal application usage, prefer `Config::load()`"
     - It's called from the test function `test_new_config_defaults()` at line 282-286
     - It's not used in the main application code
   - The method is marked as `pub` which makes it part of the public API
   - The `#[allow(dead_code)]` attribute suppresses the warning that would otherwise occur

   **Recommendation:**
   - Replace `#[allow(dead_code)]` with `#[cfg(test)]` to properly scope this method to tests
   - This change will:
     1. Remove the need for the `#[allow(dead_code)]` attribute
     2. Make the method only available in test builds
     3. Remove it from the public API in non-test builds
     4. Satisfy the "Address Violations, Don't Suppress" principle

## Summary

There is only one instance of `#[allow(dead_code)]` in `src/config/mod.rs`. It's applied to the `new()` method which is only used in tests and should be scoped accordingly with `#[cfg(test)]` instead of suppressing the warning.