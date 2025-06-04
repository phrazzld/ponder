# Error Handling Change Guidelines

This document provides guidelines for developers making changes to error handling in the ponder application. Following these guidelines helps maintain consistency, prevents test brittleness, and ensures a good user experience.

## Overview

Error handling is a critical part of user experience. Changes to error messages should be made thoughtfully, with consideration for:
- **User experience**: Error messages should be helpful and actionable
- **Test stability**: Changes shouldn't break existing tests unnecessarily  
- **Consistency**: Error formats should follow established patterns
- **Backwards compatibility**: Consider impact on downstream consumers

## When Error Message Changes Are Appropriate

### ✅ Acceptable Changes

1. **Improving clarity**: Making error messages more helpful to users
   ```rust
   // Before: "Invalid editor"
   // After: "Editor command contains invalid characters: ';'. Use a wrapper script instead."
   ```

2. **Adding context**: Including more relevant information
   ```rust
   // Before: "File not found"
   // After: "Journal file not found: /path/to/file.md. Please check the path exists."
   ```

3. **Fixing typos or grammar**: Correcting mistakes in existing messages

4. **Adding new error types**: Expanding error coverage for better debugging

5. **Standardizing formats**: Making error messages consistent across the application

### ⚠️ Changes Requiring Careful Consideration

1. **Changing error type classifications**: Moving errors between Config/Editor/IO/etc.
2. **Removing information**: Taking away context that users might rely on
3. **Changing structured log format**: Modifying JSON schema in CI environments
4. **Changing error codes**: If the application uses exit codes systematically

### ❌ Changes to Avoid

1. **Breaking structured logging**: Don't remove correlation_id or essential fields
2. **Making messages less helpful**: Removing useful context or guidance
3. **Inconsistent formatting**: Deviating from established error message patterns
4. **Exposing internal details**: Including debug-only information in user messages

## Process for Making Error Handling Changes

### Step 1: Plan the Change

1. **Review current usage**: Check `ERROR_MESSAGE_FORMATS.md` for current patterns
2. **Consider impact**: Will this break existing tests or user scripts?
3. **Design new format**: Ensure it follows established patterns
4. **Document rationale**: Why is this change necessary?

### Step 2: Update Error Definitions

1. **Modify error types** in `src/errors/mod.rs`:
   ```rust
   #[error("New improved message format: {field}. Helpful guidance here.")]
   ErrorVariant { field: String },
   ```

2. **Maintain consistency** with other error messages in the same category

3. **Include actionable guidance** when possible

### Step 3: Update Documentation

1. **Update `ERROR_MESSAGE_FORMATS.md`** with new format
2. **Document the change** in version history
3. **Update examples** to reflect new messages

### Step 4: Update Tests Robustly

**Do this:**
```rust
// Focus on essential content
assert!(error_msg.contains("not found"));
assert!(error_msg.contains(&expected_command));

// Test error type propagation
match error {
    AppError::Editor(EditorError::CommandNotFound { .. }) => {}, // Expected
    other => panic!("Expected CommandNotFound, got: {:?}", other),
}
```

**Don't do this:**
```rust
// Brittle exact matching
assert_eq!(error_msg, "Editor command 'vim' not found: No such file or directory (os error 2). Please check that the editor is installed and available in your PATH.");

// Looking for enum variant names
assert!(stderr.contains("CommandNotFound"));
```

### Step 5: Test Thoroughly

1. **Run full test suite**: `cargo test -- --test-threads=1`
2. **Test both environments**: Local (text) and CI (JSON) logging
3. **Manual testing**: Verify error messages appear correctly to users
4. **Backwards compatibility**: Check that scripts/tools still work

## Testing Guidelines for Error Changes

### Writing Robust Error Tests

1. **Test behavior, not implementation**:
   ```rust
   // Good: Tests user-visible behavior
   assert!(!success, "Should fail with invalid editor");
   assert!(stderr.contains("not found"));
   assert!(stderr.contains("vim"));
   
   // Bad: Tests implementation details
   assert!(stderr.contains("CommandNotFound"));
   ```

2. **Use pattern matching for error types**:
   ```rust
   // Good: Verifies correct error classification
   match result {
       Err(AppError::Config(_)) => {}, // Expected
       Ok(_) => panic!("Should have failed"),
       Err(other) => panic!("Expected Config error, got: {:?}", other),
   }
   ```

3. **Focus on essential information**:
   ```rust
   // Test that key elements are present
   assert!(stderr.contains("Configuration error"));
   assert!(stderr.contains("semicolon")); // The problematic character
   assert!(stderr.contains("wrapper script")); // The suggested solution
   ```

### Avoiding Test Brittleness

1. **Use partial matching**: Check for keywords rather than exact strings
2. **Test multiple scenarios**: Different error conditions should be covered
3. **Add explanatory comments**: Explain what the test validates
4. **Group related assertions**: Make clear what aspects are being tested

## Structured Logging Considerations

### JSON Format Requirements

When CI=true, errors are logged as structured JSON. Maintain these fields:

```json
{
  "timestamp": "ISO8601",
  "level": "ERROR",
  "message": "Application failed",
  "error": "User-friendly error message",
  "error_chain": "Debug representation of error chain",
  "correlation_id": "UUID for request tracking",
  "target": "ponder",
  "span": {
    "correlation_id": "UUID",
    "service_name": "ponder",
    "name": "app_invocation"
  }
}
```

### Key Requirements

1. **Preserve correlation_id**: Essential for request tracking
2. **Maintain error and error_chain**: Support both user and debug views
3. **Keep span context**: Important for distributed tracing
4. **Ensure valid JSON**: Malformed JSON breaks log aggregation

## Common Pitfalls and How to Avoid Them

### Pitfall 1: Over-Detailed Error Messages
```rust
// Bad: Too much internal detail
#[error("Failed to acquire lock in function acquire_exclusive_lock at line 123 due to WouldBlock")]

// Good: User-focused with actionable guidance
#[error("Journal file is currently being edited by another process: {path}. Please wait for the other editor to close.")]
```

### Pitfall 2: Inconsistent Error Formats
```rust
// Bad: Inconsistent style
#[error("Config problem: {0}")]
#[error("An error occurred with the editor: {0}")]

// Good: Consistent format
#[error("Configuration error: {0}")]
#[error("Editor error: {0}")]
```

### Pitfall 3: Testing Implementation Details
```rust
// Bad: Brittle test tied to internals
assert!(error_output.contains("EditorError::CommandNotFound"));

// Good: Tests user experience
assert!(error_output.contains("not found"));
assert!(error_output.contains("check that the editor is installed"));
```

## Checklist for Error Handling Changes

Before submitting changes:

- [ ] **Documentation updated**: `ERROR_MESSAGE_FORMATS.md` reflects changes
- [ ] **Tests use robust patterns**: Focus on behavior, not implementation
- [ ] **Full test suite passes**: All 107+ tests pass
- [ ] **Manual testing done**: Verified user-visible error messages
- [ ] **JSON logging tested**: CI environment produces valid structured logs
- [ ] **Consistency maintained**: New errors follow established patterns
- [ ] **Guidance included**: Error messages help users resolve issues
- [ ] **No regressions**: Existing functionality remains intact

## Related Documentation

- `ERROR_MESSAGE_FORMATS.md`: Reference for current error message patterns
- `src/errors/mod.rs`: Error type definitions and implementations
- `CLAUDE.md`: Project-specific development guidelines
- `tests/`: Integration tests demonstrating robust error testing patterns

## Version History

- **v1.0**: Initial guidelines after structured error logging implementation
- **v1.1**: Added safeguards against test brittleness
- **v1.2**: Enhanced guidance on robust testing patterns