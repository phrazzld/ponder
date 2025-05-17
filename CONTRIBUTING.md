# Contributing to Ponder

Thank you for your interest in contributing to Ponder! This guide will help you get started with development and ensure your contributions meet our quality standards.

## Development Environment Setup

### Prerequisites

- Rust toolchain (latest stable version)
- Git
- Python 3.x (for pre-commit framework)
- Pre-commit framework

### Installing Pre-commit Framework

The pre-commit framework is required to run our code quality checks before commits. Install it using one of these methods:

```bash
# Using pip
pip install pre-commit

# Using homebrew (macOS)
brew install pre-commit

# Using your system's package manager (varies by OS)
# Ubuntu/Debian: apt install pre-commit
# Fedora: dnf install pre-commit
```

### Setting Up Pre-commit Hooks

After cloning the repository and installing the pre-commit framework, run:

```bash
pre-commit install
```

This command installs the Git hooks defined in `.pre-commit-config.yaml`.

## Pre-commit Hooks

We use pre-commit hooks to maintain code quality and prevent CI failures. The following checks run automatically before each commit:

1. **cargo fmt --check**: Ensures code follows rustfmt standards
2. **cargo clippy**: Lints code for common issues and enforces best practices

### Why Pre-commit Hooks?

Pre-commit hooks help us:
- Catch formatting and linting issues early, before they reach CI
- Maintain consistent code style across all contributions
- Prevent broken commits from being pushed to the repository
- Save time by avoiding CI failures due to simple formatting issues

### Using Pre-commit Hooks

Once installed, the hooks run automatically when you commit:

```bash
git add .
git commit -m "feat: add new feature"
```

If any checks fail, the commit will be blocked. You'll see output like:

```
cargo fmt --check........................................................Failed
- hook id: cargo-fmt-check
- exit code: 1

Diff in /path/to/file.rs
```

### Fixing Hook Failures

When a hook fails:

1. **For formatting issues** (cargo fmt):
   ```bash
   cargo fmt
   ```

2. **For clippy warnings**:
   - Review the specific warnings in the output
   - Fix the code issues identified by clippy
   - Common fixes include removing unused imports, fixing type annotations, or addressing potential bugs

3. After fixing, stage your changes and commit again:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

### Bypassing Hooks (Not Recommended)

In rare cases where you need to bypass hooks:

```bash
git commit --no-verify -m "commit message"
```

**Warning**: Bypassing hooks is strongly discouraged and may result in CI failures. Only use this if absolutely necessary and ensure your code passes all checks before pushing.

### Troubleshooting

**Hooks not running?**
- Ensure pre-commit is installed: `pre-commit --version`
- Reinstall hooks: `pre-commit install`

**Cargo commands not found?**
- Ensure Rust toolchain is installed and in PATH
- Run `cargo --version` to verify

**Hooks running slowly?**
- Consider using `cargo check` during development for faster feedback
- The hooks run full checks to match CI behavior

## IDE Setup

For detailed instructions on configuring your IDE with `rustfmt` and `clippy` integration for real-time feedback and automatic code formatting, see [Development Environment Setup](docs/DEVELOPMENT_SETUP.md).

## Code Standards

All code, including tests, must meet our quality standards:

- Pass `cargo fmt --check`
- Pass `cargo clippy --all-targets -- -D warnings`
- Include appropriate tests for new functionality
- Follow the patterns established in the existing codebase

### Test Code Standards

Test code is held to the same quality standards as production code. This means:

- Test code must pass all formatting checks (`cargo fmt`)
- Test code must pass all linting checks (`cargo clippy`)
- Test code must use current, non-deprecated APIs
- Test code should be well-structured and maintainable
- Test code should follow the same naming conventions and patterns as production code

Treat your tests as first-class code—they are essential for maintaining project quality and should be as clean and well-written as the code they test.

## Submitting Changes

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all pre-commit hooks pass
5. Push your branch and open a pull request

## Additional Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Pre-commit Documentation](https://pre-commit.com/)