# T017: Review Public API Surface - Implementation Plan

## Overview

This plan outlines the changes needed to minimize the public API surface of the ponder application. Following the Development Philosophy principle of "Module by Feature" and "Visibility" outlined in the Rust appendix, we'll carefully audit all `pub` items and adjust their visibility to be as restrictive as appropriate.

## Current Public API Status

After analyzing the codebase, here's the current public API surface:

### 1. Module Structure in lib.rs
- All modules are declared as `pub` in lib.rs
- Some key types are re-exported for convenience (CliArgs, Config, AppError, AppResult, DateSpecifier)

### 2. Module-by-Module Analysis

#### 2.1 cli/mod.rs
- `pub struct CliArgs`: Public struct with all fields public
- `pub fn parse()`: Public method for parsing command line args
- `pub fn parse_date()`: Public method for parsing dates
- `pub fn to_date_specifier()`: Public method for creating DateSpecifier from CLI args

#### 2.2 config/mod.rs
- `pub struct Config`: Public struct with all fields public
- `pub fn new()`: Public constructor (only in test builds)
- `pub fn load()`: Public method for loading config from environment
- `pub fn validate()`: Public method for validating config
- `fn validate_editor_command()`: Private utility function

#### 2.3 errors/mod.rs
- `pub enum AppError`: Public error enum with all variants public
- `pub type AppResult<T>`: Public type alias for Result<T, AppError>
- `impl Clone for AppError`: Public implementation

#### 2.4 journal_core/mod.rs
- `pub enum DateSpecifier`: Public enum with all variants public
- `pub fn from_cli_args()`: Public method for creating DateSpecifier from CLI args
- `pub fn resolve_dates()`: Public method for resolving dates
- `fn parse_date_string()`: Private utility function
- Several private constants

#### 2.5 journal_io/mod.rs
- `pub fn ensure_journal_directory_exists()`: Public function for directory creation
- `pub fn open_journal_entries()`: Public function for opening journal entries
- `pub fn append_date_header_if_needed()`: Public function for adding headers to files
- Several private utility functions

## Visibility Adjustments Plan

Based on the analysis, here are the planned visibility adjustments:

### 1. lib.rs
- Keep all module declarations as `pub` (needed for external access)
- Keep current re-exports (they define the main public API)

### 2. cli/mod.rs
- Keep `pub struct CliArgs` and its public fields (needed for clap and public API)
- Keep `pub fn parse()` (part of public API)
- Make `pub fn parse_date()` -> `pub(crate)` (only used internally by to_date_specifier)
- Keep `pub fn to_date_specifier()` (part of main.rs API)

### 3. config/mod.rs
- Keep `pub struct Config` and its public fields (part of public API)
- Keep `pub fn load()` (part of public API)
- Keep `pub fn validate()` (used by main.rs)
- Keep `fn validate_editor_command()` as private

### 4. errors/mod.rs
- Keep `pub enum AppError` and its variants (part of public API)
- Keep `pub type AppResult<T>` (part of public API)

### 5. journal_core/mod.rs
- Keep `pub enum DateSpecifier` (part of public API)
- Keep `pub fn from_cli_args()` (used by cli module)
- Keep `pub fn resolve_dates()` (used by main.rs)
- Keep `fn parse_date_string()` as private

### 6. journal_io/mod.rs
- Keep `pub fn ensure_journal_directory_exists()` (used by main.rs)
- Keep `pub fn open_journal_entries()` (used by main.rs)
- Change `pub fn append_date_header_if_needed()` -> `pub(crate)` (should only be used within crate)
- Keep private utility functions as private

## Implementation Steps

1. Modify cli/mod.rs to change visibility of `parse_date()`
2. Modify journal_io/mod.rs to change visibility of `append_date_header_if_needed()`
3. Run tests to ensure changes don't break functionality
4. Update documentation as needed