# Implementation Plan Context

## Target Issue: #43 - No structured error logging at application boundary

**Priority**: BLOCKER  
**Created**: Based on CODE_REVIEW.md findings (CB-1)  
**Related**: PR #42 (error handling standardization)

## Problem Statement

The `main` function in `src/main.rs` returns `AppResult<()>`. When an error occurs, Rust's default behavior prints the `Debug` representation to `stderr`, completely bypassing our configured structured logging system (JSON output with timestamps, correlation IDs, service name, etc.).

This results in a critical loss of operational visibility, making production debugging, monitoring, and alerting based on structured error logs impossible. While we've successfully eliminated double logging, we now have *no* proper structured logging for errors at the application boundary.

## Impact Analysis

- **Severity**: BLOCKER
- **Violation**: Logging Strategy requirements for structured logging with mandatory context fields
- **Effect**: Production monitoring and debugging capabilities are severely compromised
- **Operational Impact**: 
  - No correlation IDs for error tracking
  - No structured JSON logs for automated monitoring
  - Missing service name and tracing context
  - Debug formatting not suitable for production logs

## Current Implementation

```rust
// src/main.rs - Current problematic approach
fn main() -> AppResult<()> {
    // Application logic that can return errors
    // When errors occur, Rust prints Debug to stderr
    // Bypasses tracing/structured logging entirely
}
```

## Proposed Solution Overview

1. Modify `fn main()` to not return `Result`
2. Wrap the core application logic within `main` in a block that catches any `AppError`
3. If an `Err(e)` is caught:
   - Use structured logging: `tracing::error!(error = %e, error_chain = ?e, "Application failed");`
   - Print a user-friendly error message to `stderr` for CLI users
   - Exit with non-zero status: `std::process::exit(1);`

## Key Requirements

- Maintain all structured logging context (correlation_id, service_name, timestamps)
- Ensure user-friendly CLI error messages
- Preserve proper exit codes for CI/CD and scripting
- No double logging (maintain existing error handling improvements)
- Full error chain visibility in structured logs

## Success Criteria

- All application errors logged with structured format
- Correlation IDs present in all error logs
- User sees friendly error messages on stderr
- Proper exit codes maintained
- Integration tests verify structured logging behavior
- No regression in existing error handling improvements