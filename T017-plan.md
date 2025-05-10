# T017 - Review `#[allow(dead_code)]` in `src/config/mod.rs`

## Findings

After reviewing `src/config/mod.rs`, I've found one instance of `#[allow(dead_code)]`:

1. `Config::new()` method (line 95)
   - This method is a simple constructor that calls `Self::default()`
   - It's used in a test within the same file: `test_new_config_defaults` (line 279-283)
   - No other usages were found in the codebase outside of the Config module
   - **Conclusion**: The method is genuinely used by tests, but not in production code.

## Recommendations for T020

1. For `Config::new()`:
   - Since the method is used in tests, the `#[allow(dead_code)]` attribute can be safely removed
   - Alternatively, if the method is genuinely only meant for tests, consider adding `#[cfg(test)]` to make its purpose clearer
   - If kept as a public API without the test-only designation, documentation should be improved to clarify that `Config::load()` is the preferred method for normal usage