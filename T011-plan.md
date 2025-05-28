# T011 Implementation Plan: Add Better Debug Output for Test Failures

## Overview
This plan enhances debug output for test failures to make diagnostic information more actionable and comprehensive.

## Analysis of Current State

### High-Value Test Files for Enhancement
1. **locking_tests.rs** - Complex integration tests with process management and file synchronization
2. **editor_error_integration_tests.rs** - Error condition tests that need better diagnostic context
3. **journal_integration_tests.rs** - Core functionality tests with file operations
4. **config_tests.rs** - Configuration validation tests with environment dependencies

### Common Failure Scenarios Needing Better Debug Output
1. **File System Operations**: File existence, directory creation, permissions
2. **Process Execution**: Command execution, exit codes, output capture
3. **Configuration State**: Environment variables, path resolution, validation
4. **Error Message Validation**: Expected vs actual error text
5. **Timing/Synchronization**: File locking, process coordination

## Implementation Strategy

### Phase 1: Debug Helper Functions (Foundation)
Create reusable debug output functions in a test utilities module:

1. **File System Debug Helpers**
   - `debug_directory_state()`: List directory contents with metadata
   - `debug_file_info()`: File permissions, size, timestamps
   - `debug_path_resolution()`: Show path expansion and validation

2. **Process Debug Helpers**  
   - `debug_command_execution()`: Command, args, env vars, working directory
   - `debug_process_output()`: Full stdout/stderr with timing information
   - `debug_environment_state()`: Relevant environment variables

3. **Assertion Enhancement Helpers**
   - `assert_with_context!()`: Macro that includes contextual debug info
   - `debug_comparison()`: Show expected vs actual with formatting

### Phase 2: Enhanced Assertions in Target Tests

1. **locking_tests.rs Enhancements**
   - Add process state debugging when process coordination fails
   - Include file system state when sentinel files aren't created/removed
   - Show timing information for synchronization failures
   - Add environment variable context for command execution

2. **editor_error_integration_tests.rs Enhancements**
   - Show full command execution details when editor tests fail
   - Include file permissions and path information for permission tests
   - Add environment context for editor command resolution
   - Enhanced error message comparison with diff-style output

3. **journal_integration_tests.rs Enhancements**
   - Directory state debugging when file creation fails
   - Date/time context for deterministic test failures
   - Configuration state when setup fails
   - File content preview for validation failures

4. **config_tests.rs Enhancements**
   - Environment variable state for configuration loading failures
   - Path resolution debugging for directory validation failures
   - Configuration value comparison with detailed diffs

### Phase 3: Specific Debug Output Patterns

1. **File Existence Assertions**
   ```rust
   // Before: assert!(file.exists());
   // After: assert!(file.exists(), "File does not exist: {}\nDirectory contents: {}", 
   //        file.display(), debug_directory_state(file.parent()));
   ```

2. **Process Execution Assertions**
   ```rust
   // Add context about command execution when process tests fail
   // Include full stdout/stderr, exit code, environment state
   ```

3. **Error Message Validation**
   ```rust
   // Show full error message with highlighting of expected patterns
   // Include similar error message comparison for disambiguation
   ```

4. **Configuration Validation**
   ```rust
   // Show configuration state, environment variables, resolved paths
   // Include step-by-step resolution process for path validation
   ```

## Test Strategy

### What to Test
- Debug helpers produce useful, formatted output
- Enhanced assertions provide actionable information
- Debug output doesn't interfere with successful test execution
- Performance impact is minimal

### How to Test
- Create intentionally failing tests to verify debug output quality
- Run enhanced tests to ensure no regressions
- Measure performance impact of debug enhancements
- Validate debug output usefulness with complex failure scenarios

## Risk Mitigation

### Risks
1. Debug output might be too verbose and overwhelming
2. Performance impact from debug information collection
3. Debug helpers might introduce test dependencies or complexity

### Mitigation
1. Focus on high-value information, avoid noise
2. Lazy evaluation - only collect debug info when tests actually fail
3. Keep debug helpers simple and self-contained
4. Use conditional compilation for debug features if needed

## Expected File Changes

1. **New file: `tests/debug_helpers.rs`**
   - Debug utility functions
   - Assertion enhancement macros
   - Common debug output formatters

2. **Enhanced files:**
   - `tests/locking_tests.rs`: Process and file system debug context
   - `tests/editor_error_integration_tests.rs`: Command execution debug info
   - `tests/journal_integration_tests.rs`: File operation debug context
   - `tests/config_tests.rs`: Configuration and environment debug info

## Success Criteria

- [ ] Debug helper functions provide useful, actionable information
- [ ] Enhanced assertions give clear context about why tests failed
- [ ] Debug output is formatted clearly and isn't overwhelming
- [ ] No performance degradation for successful test execution
- [ ] Debug enhancements follow consistent patterns across test files
- [ ] Documentation explains debug output patterns for future tests

## Performance Considerations

- Use lazy evaluation for expensive debug operations
- Only collect debug information when assertions actually fail
- Keep debug helpers lightweight and efficient
- Consider conditional compilation if debug overhead is significant