# T016 - Review `#[allow(dead_code)]` in `src/cli/mod.rs`

## Findings

After reviewing `src/cli/mod.rs`, I've found two instances of `#[allow(dead_code)]`:

1. `CliArgs::parse_date()` method (line 129)
   - This method is used in tests within the same file (test_parse_date function)
   - However, it doesn't appear to be used elsewhere in the codebase
   - Instead, the DateSpecifier::parse_date_string method in journal/mod.rs seems to handle date parsing for the main program flow
   - **Conclusion**: Genuinely unused in production code, but used in tests. Should be retained but potentially made private or test-only.

2. `parse_args()` function (line 149)
   - This is explicitly marked as a "backward compatibility function"
   - Serves as a top-level wrapper around `CliArgs::parse()`
   - No usages found in the codebase, including in main.rs
   - **Conclusion**: Genuinely unused. Could be removed or marked as `#[deprecated]` if backward compatibility is still a concern.

## Recommendations for T020

1. For `CliArgs::parse_date()`:
   - Consider making it private or moving it to a test module
   - Alternatively, keep it but mark it with `#[cfg(test)]` to indicate it's only used in tests
   - If keeping the method as public, remove the `#[allow(dead_code)]` attribute since it's used in tests

2. For `parse_args()` function:
   - Since it's explicitly for backward compatibility but not actually used, mark it as `#[deprecated]` to signal that it will be removed in a future version
   - Add a note to the deprecation attribute explaining that `CliArgs::parse()` should be used instead
   - Plan to remove it completely in a future cleanup (as part of T026)