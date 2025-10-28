# Repository Guidelines

## Project Structure & Module Organization
Ponder is a Rust CLI; the binary entry point lives in `src/main.rs` and reuses modules declared in `src/lib.rs`. Domain modules sit under `src/ai`, `src/crypto`, `src/db`, `src/journal_core`, and `src/journal_io`; keep new code aligned with these boundaries instead of creating duplicate helpers. CLI surfaces live in `src/cli`, while configuration defaults sit in `src/config` and `src/constants.rs`. Integration tests belong in `tests/`, Criterion benches in `benches/`, docs and ADR-style notes in `docs/` and `*glance.md`, and reusable scripts in `scripts/`.

## Build, Test, and Development Commands
Use `cargo build` for a debug build and `cargo build --release` for production binaries. Run the app locally with `cargo run -- <command>` (for example `cargo run -- edit`). Format and lint with `cargo fmt` and `cargo clippy --all-targets -- -D warnings`; these are wired into `pre-commit run --all-files`. Run the full test suite with `PONDER_TEST_PASSPHRASE=test-passphrase cargo test`. `cargo bench` executes Criterion benches when you need perf numbers.

## Coding Style & Naming Conventions
The project follows stable Rust style (4-space indent, trailing commas). Favor module-level `mod.rs` plus `snake_case` filenames, `CamelCase` types, and `SCREAMING_SNAKE_CASE` constants. Keep public APIs documented with `///` comments and prefer immutable bindings until mutation is required. Run `cargo fmt` before pushing; clippy warnings must be resolved, not silenced.

## Testing Guidelines
Unit tests live next to the code they cover; integration and regression cases go under `tests/`. Use descriptive `test_target_behavior` function names. When touching global state (env vars, temp dirs, database files) annotate with `#[serial]` via the `serial_test` crate. Include happy-path and failure coverage for new features, and update fixtures if encryption or database schemas change. Snapshot any AI-facing prompts under `tests/fixtures`.

## Commit & Pull Request Guidelines
Commits follow Conventional Commits (`feat(cli): add retro prompts`). Keep commits scoped, include migrations or docs together with code, and let Git Cliff derive changelog entries. PRs need a clear summary, testing notes, linked issues, and screenshots or terminal transcripts when UX shifts. Ensure CI hooks pass locally before requesting review.

## Security & Configuration Tips
Never log plaintext passphrases or decrypted entries. Set `PONDER_TEST_PASSPHRASE` in shells and CI to avoid interactive prompts. Document any SQLCipher or Ollama model requirements when features depend on them, and keep key material in `.env` or OS keychains rather than hard-coding.
