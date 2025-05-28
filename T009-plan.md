# T009: Add more robust error handling in tests (general) - Implementation Plan

## Overview
Based on comprehensive analysis, we will systematically refactor targeted tests to return `Result<(), Box<dyn std::error::Error>>`, replacing panicking constructs with the `?` operator and enhancing assertion messages with rich contextual information.

## Strategic Approach

### Core Philosophy Alignment
- **Simplicity**: Use standard Rust error handling patterns (`Result`, `?`, `Box<dyn std::error::Error>`)
- **Modularity**: Localize changes within individual test functions
- **Testability**: Provide clear, informative failure messages for better debuggability
- **Coding Standards**: Treat test code as production code with explicit error handling

### Priority Targets
1. **High-Impact Integration Tests**: `locking_tests.rs`, `journal_integration_tests.rs`, `editor_error_integration_tests.rs`
2. **Complex Unit Tests**: Those involving File I/O, process spawning, configuration loading
3. **Helper Functions**: Common test utilities that perform fallible operations

## Implementation Plan

### Phase 1: Assessment and Setup (20 minutes)
1. **Audit Current State**:
   ```bash
   rg "unwrap\(\)" tests/ src/ --type rust
   rg "expect\(" tests/ src/ --type rust  
   rg "panic!\(" tests/ src/ --type rust
   ```

2. **Identify Priority Files**:
   - `tests/locking_tests.rs` (already has sophisticated error handling patterns)
   - `tests/editor_error_integration_tests.rs` (error-focused tests)
   - `tests/journal_integration_tests.rs` (file I/O operations)
   - `src/config/mod.rs` (unit tests with configuration loading)
   - `src/cli/mod.rs` (unit tests with argument parsing)

### Phase 2: Core Test Improvements (60 minutes)

#### 2.1 Convert Test Function Signatures
**Pattern**: `fn test_name()` → `fn test_name() -> Result<(), Box<dyn std::error::Error>>`

#### 2.2 Replace Panicking Constructs
- **`Result.unwrap()`** → `?` operator
- **`Option.unwrap()`** → `.ok_or_else(|| "Descriptive error".into())?`
- **`panic!("message")`** → `return Err("contextual message".into())`

#### 2.3 Enhance Assertion Messages
```rust
// Before
assert_eq!(actual, expected);

// After  
assert_eq!(actual, expected, 
    "Configuration mismatch for '{}': expected {:?}, got {:?}", 
    config_key, expected, actual);
```

#### 2.4 Improve Error Context
```rust
// Before
let data = fs::read_to_string("config.toml").unwrap();

// After
let config_path = "config.toml";
let data = fs::read_to_string(config_path)
    .map_err(|e| format!("Failed to read config file '{}': {}", config_path, e))?;
```

### Phase 3: Specific Area Improvements (40 minutes)

#### 3.1 File I/O Operations
- Add context to all `std::fs` operations
- Include file paths in error messages
- Handle `tempfile` operations with `?`

#### 3.2 Process Spawning
- Capture stdout/stderr in command failures
- Provide command context in error messages
- Ensure RAII guards maintain error visibility

#### 3.3 Configuration Tests
- Propagate `Config::load()` errors with `?`
- Add validation context to configuration tests

### Phase 4: Validation and Testing (20 minutes)

#### 4.1 Test Error Messages
- Temporarily introduce failures to verify error message quality
- Ensure messages are actionable and include relevant context
- Test with `cargo test` to verify functionality

#### 4.2 Quality Checks
- Run `cargo fmt` for formatting
- Run `cargo clippy --all-targets -- -D warnings` for linting
- Verify no performance degradation

## Success Criteria
- [ ] Selected tests provide informative failure messages with context
- [ ] No test logic changes, only error reporting improvements
- [ ] All tests pass with new error handling
- [ ] Clippy and formatting checks pass
- [ ] Error messages include relevant variable values and operation context
- [ ] Manual failure testing shows clear, actionable diagnostics

## Risk Mitigation
- Make changes incrementally, testing after each modification
- Preserve original test logic and assertions
- Focus on error handling improvements, not test behavior changes
- Maintain compatibility with existing test infrastructure

## Files to Modify (Estimated)
1. `tests/editor_error_integration_tests.rs` - moderate complexity
2. `tests/journal_integration_tests.rs` - moderate complexity  
3. `src/config/mod.rs` - unit tests section
4. `src/cli/mod.rs` - unit tests section
5. Selected functions in `tests/locking_tests.rs` - (already well-structured)

## Implementation Notes
- Start with simpler unit tests to establish patterns
- Use consistent error message formatting
- Leverage existing error types where available
- Maintain test isolation and independence