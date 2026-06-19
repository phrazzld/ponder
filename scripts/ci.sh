#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
export RUSTFLAGS="${RUSTFLAGS:--C debuginfo=0}"
export PONDER_EDITOR="${PONDER_EDITOR:-echo}"
export PONDER_DIR="${PONDER_DIR:-/tmp/ponder_ci_tests}"
export PONDER_TEST_PASSPHRASE="${PONDER_TEST_PASSPHRASE:-test-passphrase}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

mode="${1:-gate}"

case "$mode" in
  gate)
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    cargo build --verbose
    cargo test --verbose -- --test-threads=1
    ;;
  coverage)
    if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
      printf 'cargo-tarpaulin is required for coverage reports.\n' >&2
      exit 1
    fi
    cargo tarpaulin \
      --out Html \
      --out Xml \
      --output-dir coverage \
      --exclude-files 'tests/*' \
      --timeout 300
    test -f coverage/cobertura.xml
    ;;
  bench)
    cargo bench
    ;;
  *)
    printf 'usage: %s [gate|coverage|bench]\n' "$0" >&2
    exit 2
    ;;
esac
