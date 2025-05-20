# TODO

Module Boundary Refactoring Tasks - Synthesized from thinktank analysis of PLAN.md

## Prerequisites
- [x] **T001**: Ensure clean working directory and create feature branch
  - Verify git status is clean on main branch
  - Run `cargo test --all-features` to ensure baseline
  - Create branch: `git checkout -b feat/refactor-module-boundaries`
  - **Verification**: All tests pass, new branch created

## Module Structure Setup
- [x] **T002**: Create new module directories
  - Create: `src/errors/`, `src/journal_core/`, `src/journal_io/`
  - **Verification**: Directories exist via `ls src/`

- [x] **T003**: Create mod.rs files for new modules  
  - Create: `src/errors/mod.rs`, `src/journal_core/mod.rs`, `src/journal_io/mod.rs`
  - **Verification**: Files exist in each directory

- [x] **T004**: Update lib.rs module declarations
  - Remove old declarations: `errors`, `journal_logic`
  - Add new declarations: `errors`, `journal_core`, `journal_io`
  - Update re-exports: `CliArgs`, `Config`, `AppError`, `AppResult`, `DateSpecifier`
  - **Verification**: `cargo check` passes

- [x] **T005**: Update main.rs imports
  - Remove any `mod` declarations
  - Update all `use` statements to new paths (e.g., `use ponder::errors::AppResult`)
  - **Verification**: `cargo check` passes

## Errors Module Migration
- [x] **T006**: Migrate errors.rs to src/errors/mod.rs
  - Move content from `src/errors.rs` to `src/errors/mod.rs`
  - Optional: Refine `AppError` variants for better specificity
  - **Verification**: Content migrated correctly

- [x] **T007**: Update error imports throughout codebase
  - Search and replace all `use crate::errors` paths
  - Ensure all error imports point to new module
  - **Verification**: `cargo check` passes

- [x] **T008**: Delete old errors.rs
  - Remove `src/errors.rs`
  - **Verification**: File removed, `cargo check` still passes

## Journal Logic Refactoring
- [x] **T009**: Extract DateSpecifier to journal_core
  - Move `DateSpecifier` enum to `src/journal_core/mod.rs`
  - Move and rename `from_args` to `from_cli_args(retro: bool, reminisce: bool, date_str: Option<&str>)`
  - Move and rename `get_dates` to `resolve_dates(&self, reference_date: NaiveDate)`
  - Ensure no I/O or side effects in journal_core
  - **Verification**: Pure logic isolated, `cargo check` passes

- [x] **T010**: Extract I/O functions to journal_io
  - Move `ensure_journal_directory_exists` with signature `(journal_dir: &Path) -> AppResult<()>`
  - Move `open_journal_entries` with signature `(config: &config::Config, dates: &[NaiveDate]) -> AppResult<()>`
  - Move helper functions, make private unless justified
  - **Verification**: All I/O isolated, `cargo check` passes

- [x] **T011**: Update journal logic imports
  - Replace all `use crate::journal_logic` with appropriate new module
  - **Verification**: No references to old module remain

- [x] **T012**: Delete journal_logic.rs
  - Remove `src/journal_logic.rs`
  - **Verification**: File removed, `cargo check` passes

## Module-Specific Updates
- [x] **T013**: Update CLI module
  - Ensure it only handles argument parsing
  - Business logic should be invoked from main.rs
  - **Verification**: Clean separation of concerns

- [x] **T014**: Update config module
  - Remove deprecated `Config::ensure_journal_dir` method
  - Update error imports to `crate::errors`
  - **Verification**: `cargo check` passes

## Main.rs Orchestration
- [x] **T015**: Refactor main.rs flow
  - Parse CLI args: `let args = CliArgs::parse();`
  - Load config: `let config = Config::load()?;`
  - Validate config: `config.validate()?;`
  - Ensure journal dir: `journal_io::ensure_journal_directory_exists(config.journal_dir())?;`
  - Create date spec: `let date_spec = journal_core::DateSpecifier::from_cli_args(...)?;`
  - Resolve dates: `let dates_to_open = date_spec.resolve_dates(Local::now().date_naive());`
  - Open entries: `journal_io::open_journal_entries(&config, &dates_to_open)?;`
  - **Verification**: Clean orchestration, `cargo test` passes

## Quality Assurance
- [x] **T016**: Run tests after each major change
  - Run `cargo check` after each file move
  - Run `cargo test --all-features` after each module completion
  - Fix any issues immediately
  - **Verification**: All tests green

- [x] **T017**: Review public API surface
  - Audit all `pub` items in each module
  - Use `pub(crate)` or private visibility where appropriate
  - Minimize public surface area
  - **Verification**: Only necessary items are public

## Final Steps
- [x] **T018**: Code formatting and linting
  - Run `cargo fmt`
  - Run `cargo clippy --all-targets -- -D warnings`
  - Address all issues
  - **Verification**: No warnings or errors

- [x] **T019**: Update documentation
  - Add module-level docs (`//!`) to all new `mod.rs` files
  - Update crate-level docs in `lib.rs`
  - Update architecture sections in README.md and CLAUDE.md
  - **Verification**: `cargo doc --open` shows complete docs

- [x] **T020**: Final verification
  - Run `cargo test --all-features` one final time
  - Manually test binary with various flags
  - Verify the new module structure is correct
  - **Verification**: Everything works as before refactoring

## Testing Updates
- [ ] **T021**: Update test imports and structure
  - Fix test imports to use new module paths
  - Ensure integration tests work with new structure
  - Maintain or improve test coverage
  - **Verification**: Coverage targets met

## Notes
- Perform iterative builds after each significant change
- Keep changes atomic and testable
- Maintain backwards compatibility for external API
- Document any discovered interdependencies or complications
- Fix pre-commit hook formatting issues with rustfmt configuration

## Additional Tasks
- [x] **T022**: Fix rustfmt configuration issues
  - Update rustfmt.toml to be compatible with stable channel
  - Ensure import ordering is consistent
  - Fix blank line handling between functions
  - **Verification**: pre-commit hooks pass with no warnings