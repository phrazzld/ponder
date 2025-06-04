# Error Message Format Documentation

This document defines the expected error message formats for the ponder application. These formats should be maintained for consistency and to prevent test brittleness.

## Purpose

This documentation serves as a reference for:
- Developers making changes to error handling
- Test writers creating error message validation
- Maintaining consistent user experience across error scenarios

## Error Message Hierarchy

### AppError (Top-level Application Errors)

The main application error type follows these patterns:

- **Config**: `Configuration error: {detailed_message}`
- **I/O**: `I/O error: {underlying_io_error}`  
- **Journal**: `Journal logic error: {detailed_message}`
- **Editor**: `Editor error: {EditorError_message}`
- **Lock**: `File locking error: {LockError_message}`

### EditorError (Specific Editor-related Errors)

Editor errors provide detailed context about what went wrong:

- **CommandNotFound**: 
  ```
  Editor command '{command}' not found: {source}. Please check that the editor is installed and available in your PATH.
  ```

- **PermissionDenied**:
  ```
  Permission denied when trying to execute editor '{command}': {source}. Please check file permissions or try running with appropriate access rights.
  ```

- **ExecutionFailed**:
  ```
  Failed to execute editor '{command}': {source}. Please check system resources, disk space, or editor installation.
  ```

- **NonZeroExit**:
  ```
  Editor '{command}' exited with non-zero status code: {status_code}. This may indicate an issue with editor configuration or the file being edited.
  ```

- **Other**:
  ```
  An unexpected issue occurred while trying to use editor '{command}': {message}
  ```

### LockError (File Locking Errors)

File locking errors indicate concurrent access issues:

- **FileBusy**:
  ```
  Journal file is currently being edited by another process: {path}. Please wait for the other editor to close or check for existing ponder processes.
  ```

- **AcquisitionFailed**:
  ```
  Failed to acquire lock for journal file {path}: {source}. Please check file permissions and ensure the directory is accessible.
  ```

## Testing Guidelines

### Robust Error Message Testing

When writing tests that validate error messages:

1. **Focus on Key Components**: Test for presence of key elements rather than exact string matches
   - Error type indicators (e.g., "Configuration error", "Editor error")
   - Relevant context (command names, file paths, status codes)
   - Helpful guidance text

2. **Use Pattern Matching**: Prefer regex or partial matching over exact string comparison
   ```rust
   // Good: Focus on essential information
   assert!(error_msg.contains("Configuration error"));
   assert!(error_msg.contains("shell metacharacters"));
   
   // Avoid: Brittle exact matching
   assert_eq!(error_msg, "Configuration error: Editor command cannot contain shell metacharacters: ';'. Use a wrapper script or shell alias instead");
   ```

3. **Test Error Type Propagation**: Focus on verifying the correct error type is returned
   ```rust
   match error {
       AppError::Config(_) => {}, // Expected
       other => panic!("Expected Config error, got: {:?}", other),
   }
   ```

### Structured Logging Format

In CI environments (CI=true), errors are logged as structured JSON:

```json
{
  "timestamp": "2025-06-04T05:02:36.870252+00:00",
  "level": "ERROR", 
  "message": "Application failed",
  "error": "Configuration error: {details}",
  "error_chain": "Config(\"{details}\")",
  "correlation_id": "{uuid}",
  "target": "ponder",
  "span": {
    "correlation_id": "{uuid}",
    "service_name": "ponder", 
    "name": "app_invocation"
  }
}
```

## Maintenance Notes

### When Modifying Error Messages

1. **Update This Documentation**: Keep this file in sync with actual error message formats
2. **Update Tests**: Ensure tests use robust pattern matching rather than exact string matching
3. **Maintain User-Friendliness**: Error messages should be helpful to end users
4. **Preserve Context**: Include relevant information (commands, paths, codes) in error messages

### Version History

- **v1.0**: Initial documentation of error message formats after structured logging implementation
- Added comprehensive error type coverage
- Established testing guidelines for robust error validation

## Related Files

- `src/errors/mod.rs`: Error type definitions and message implementations
- `tests/`: Integration tests that validate error scenarios
- `src/main.rs`: Error boundary implementation with structured logging