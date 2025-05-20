# T014 Plan: Update config module

## Analysis

Current task: **T014** - Update config module

The task involves:
1. Removing the deprecated `Config::ensure_journal_dir` method
2. Updating error imports to `crate::errors`

## Current State

- The `Config::ensure_journal_dir` method is properly marked as deprecated with since="0.1.2" and a note to use `journal_io::ensure_journal_directory_exists` instead.
- Error imports in `src/config/mod.rs` already use `crate::errors::{AppError, AppResult}`
- No code in tests or main.rs appears to call `config.ensure_journal_dir()` directly; they all use `journal_io::ensure_journal_directory_exists` instead.

## Implementation Plan

1. Remove the deprecated `Config::ensure_journal_dir` method entirely from `src/config/mod.rs`
2. Ensure error imports are correct
3. Run `cargo check` to verify the removal hasn't broken anything
4. Run tests to ensure functionality is maintained
5. Update TODO.md to mark the task as completed

## Complexity Assessment

This is a **Simple** task. It involves removing deprecated code that is no longer used, with minimal risk of regression since the replacement functionality is already in place and being used.