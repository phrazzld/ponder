# T016 Plan: Implement Structured Logging

## Overview

This plan outlines how to implement structured logging in the Ponder application using the `tracing` ecosystem. We'll migrate from the current `env_logger`-based approach to a more comprehensive solution with structured JSON output, correlation IDs, and proper context fields.

## Current State Analysis

- Ponder currently uses `env_logger` and the `log` crate for logging
- Logging is initialized in `main.rs` with a custom JSON formatter
- Log statements are used in `main.rs` and `journal_io/mod.rs`
- Log levels used include `debug`, `info`, and potentially others
- There are no specific tests that verify logging behavior

## Implementation Plan

### 1. Update Dependencies

- Remove `env_logger` from `Cargo.toml`
- Add new dependencies:
  - `tracing` (version 0.1)
  - `tracing-subscriber` (version 0.3) with features `json`, `env-filter`, `fmt`, and `registry`
  - `uuid` (version 1.4) with features `v4`, `fast-rng`, and `serde`
- Add dev dependencies for testing:
  - `serde_json` (version 1.0) for parsing JSON logs in tests

### 2. Modify CLI Arguments

- Update `src/cli/mod.rs` to add a new optional argument `--log-format <FORMAT>` that accepts "json" or "text"
- Ensure this new argument is properly documented in help text
- Update the `CliArgs` struct to include this new option

### 3. Initialize Structured Logging in main.rs

- Generate a correlation ID (UUID v4) at the start of the `main` function
- Configure `tracing_subscriber` with appropriate layers:
  - JSON log formatting for CI environments or when `--log-format json` is specified
  - Human-readable pretty printing for local development
  - Use RFC3339 timestamps for JSON output
  - Capture proper span context
- Create a root span with `correlation_id` and other context fields
- Enter the root span for the duration of the application

### 4. Update Log Statements

- Replace all `log::debug!`, `log::info!`, etc. calls with their `tracing` equivalents
- Update import statements to use `tracing` instead of `log`
- Enhance error logging to include structured error information

### 5. Add Testing for Structured Logging

- Create tests that verify JSON log output
- Ensure tests validate:
  - Presence of required fields (timestamp, level, message, correlation_id)
  - Consistency of correlation IDs across log entries
  - Valid JSON formatting
  - Inclusion of error details in error logs

### 6. Update Documentation

- Update README.md to document:
  - How to control log verbosity with the `RUST_LOG` environment variable
  - How to enable JSON logs
  - The structure of the logs and key fields

## Verification Steps

1. Manual verification:
   - Run `CI=true RUST_LOG=trace cargo run`
   - Run with `--log-format json` argument
   - Verify both human-readable and JSON output formats
   - Check for proper correlation ID propagation

2. Automated testing:
   - Run integration tests that verify structured output
   - Ensure all tests pass

3. Confirm that logs include all required fields:
   - timestamp (RFC3339 format)
   - level
   - message
   - correlation_id
   - span context
   - service_name

## Implementation Details

### JSON Log Format Example

```json
{
  "timestamp": "2025-05-21T12:02:32.743Z",
  "level": "INFO",
  "target": "ponder::main",
  "fields": {
    "message": "Starting ponder"
  },
  "span": {
    "name": "app_invocation",
    "service_name": "ponder",
    "correlation_id": "9ae2867d-55ec-45d8-bb11-f14b2a932dad"
  }
}
```

### Human-Readable Format Example

```
2025-05-21T12:02:32.743Z INFO [ponder::main{app_invocation{service_name=ponder correlation_id=9ae2867d-55ec-45d8-bb11-f14b2a932dad}}] Starting ponder
```

### Error Logging Format

```json
{
  "timestamp": "2025-05-21T12:02:32.743Z",
  "level": "ERROR",
  "target": "ponder::journal_io",
  "fields": {
    "message": "Failed to open journal entry",
    "error.message": "No such file or directory",
    "error.details": "Os { code: 2, kind: NotFound, message: \"No such file or directory\" }"
  },
  "span": {
    "name": "app_invocation",
    "service_name": "ponder",
    "correlation_id": "9ae2867d-55ec-45d8-bb11-f14b2a932dad"
  }
}
```

## Task Breakdown

1. Update Cargo.toml with new dependencies
2. Update CLI args to support log format specification
3. Implement logging initialization in main.rs
4. Add correlation ID generation
5. Update all logging statements to use tracing
6. Enhance error logging with structured context
7. Add tests for structured logging
8. Update documentation
9. Manual verification
10. Final review and cleanup