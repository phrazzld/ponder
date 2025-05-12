# T019 Findings: `#[allow(dead_code)]` in src/journal/mod.rs

## Instances Found

1. **Instance #1** - Line 250-251:
   ```rust
   #[allow(dead_code)]
   config: Config,
   ```

   **Analysis:**
   - This is a field in the `JournalService` struct
   - The field is not used directly in the main application code
   - However, it's stored during construction at line 300: `JournalService { config, io, editor }`
   - It's also accessed by the getter methods `get_editor_cmd` and `get_journal_dir` which are themselves marked with `#[allow(dead_code)]`
   - Both the field and the getters are used in tests according to their comments

   **Recommendation:**
   - The field itself should remain in the struct as it's part of the service's state
   - Remove the `#[allow(dead_code)]` attribute since it's used indirectly by the getter methods
   - If the field is truly only needed for tests, consider moving the getters to a `#[cfg(test)]` block to properly scope them without suppressing warnings

2. **Instance #2** - Line 310-311:
   ```rust
   #[allow(dead_code)]
   pub fn get_editor_cmd(&self) -> &str {
   ```

   **Analysis:**
   - Method is explicitly noted to be "used in tests to verify JournalService construction" at line 309
   - This method is not used in the main application code
   - It's a getter for the `editor` field of the `config` field

   **Recommendation:**
   - Mark with `#[cfg(test)]` instead of `#[allow(dead_code)]` to properly scope it to tests
   - This will remove it from the public API in non-test builds

3. **Instance #3** - Line 322-323:
   ```rust
   #[allow(dead_code)]
   pub fn get_journal_dir(&self) -> &PathBuf {
   ```

   **Analysis:**
   - Method is explicitly noted to be "used in tests and integration tests to access the journal directory" at line 321
   - This method is not used in the main application code
   - It's a getter for the `journal_dir` field of the `config` field

   **Recommendation:**
   - Mark with `#[cfg(test)]` instead of `#[allow(dead_code)]` to properly scope it to tests
   - This will remove it from the public API in non-test builds

4. **Instance #4** - Line 563-564:
   ```rust
   #[allow(dead_code)]
   pub fn open_entry(&self) -> AppResult<()> {
   ```

   **Analysis:**
   - Method is explicitly noted to be "a convenience method used in tests and integration tests" at line 562
   - This method is not used in the main application code
   - It's a thin wrapper around `open_entries(&DateSpecifier::Today)`

   **Recommendation:**
   - Mark with `#[cfg(test)]` instead of `#[allow(dead_code)]` to properly scope it to tests
   - This will remove it from the public API in non-test builds

5. **Instance #5** - Line 571-572:
   ```rust
   #[allow(dead_code)]
   pub fn open_retro_entry(&self) -> AppResult<()> {
   ```

   **Analysis:**
   - Method is explicitly noted to be "a convenience method used in tests and integration tests" at line 570
   - This method is not used in the main application code
   - It's a thin wrapper around `open_entries(&DateSpecifier::Retro)`

   **Recommendation:**
   - Mark with `#[cfg(test)]` instead of `#[allow(dead_code)]` to properly scope it to tests
   - This will remove it from the public API in non-test builds

6. **Instance #6** - Line 579-580:
   ```rust
   #[allow(dead_code)]
   pub fn open_reminisce_entry(&self) -> AppResult<()> {
   ```

   **Analysis:**
   - Method is explicitly noted to be "a convenience method used in tests and integration tests" at line 578
   - This method is not used in the main application code
   - It's a thin wrapper around `open_entries(&DateSpecifier::Reminisce)`

   **Recommendation:**
   - Mark with `#[cfg(test)]` instead of `#[allow(dead_code)]` to properly scope it to tests
   - This will remove it from the public API in non-test builds

7. **Instance #7** - Line 587-588:
   ```rust
   #[allow(dead_code)]
   pub fn open_specific_entry(&self, date: NaiveDate) -> AppResult<()> {
   ```

   **Analysis:**
   - Method is explicitly noted to be "a convenience method used in tests and integration tests" at line 586
   - This method is not used in the main application code
   - It's a thin wrapper around `open_entries(&DateSpecifier::Specific(date))`

   **Recommendation:**
   - Mark with `#[cfg(test)]` instead of `#[allow(dead_code)]` to properly scope it to tests
   - This will remove it from the public API in non-test builds

## Summary

There are seven instances of `#[allow(dead_code)]` in `src/journal/mod.rs`:
1. The `config` field in the `JournalService` struct
2. Six methods of `JournalService` that are only used in tests:
   - `get_editor_cmd`
   - `get_journal_dir`
   - `open_entry`
   - `open_retro_entry`
   - `open_reminisce_entry`
   - `open_specific_entry`

All of these methods should be marked with `#[cfg(test)]` instead of suppressing the warning with `#[allow(dead_code)]`. The `config` field should remain in the struct but can have its `#[allow(dead_code)]` removed if the getter methods are properly scoped with `#[cfg(test)]`.