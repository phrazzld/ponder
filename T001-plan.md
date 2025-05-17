# T001 Plan: Update CI workflow to set required environment variables

## Implementation Approach

1. Modify `.github/workflows/ci.yml` to add required environment variables to the test step
2. Set the following environment variables:
   - `PONDER_EDITOR`: Set to `echo` (a simple command that accepts arguments)
   - `PONDER_DIR`: Set to `/tmp/ponder_ci_tests` (a temporary directory for CI testing)
   - `RUST_BACKTRACE`: Set to `1` (to get better error messages in CI logs)

## Implementation Details

The modification will be made to the test step in the CI workflow, adding an `env:` section with the required variables.

This will prevent tests from panicking due to missing `PONDER_EDITOR` environment variable, which was the root cause of CI failures.