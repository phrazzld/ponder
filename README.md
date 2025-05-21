# Ponder 🤔

Ponder is a simple journaling tool for daily reflections, designed to help users maintain a journal with minimal friction. It provides functionality for creating and viewing daily journal entries, as well as reviewing past entries.

## Features 🌟

- **Today's Entry**: Quickly create or edit today's journal entry
- **Retro Mode**: Review entries from the past week (excluding today)
- **Reminisce Mode**: Review entries from significant past intervals (1 month ago, 3 months ago, 6 months ago, etc.)
- **Specific Date Access**: Open an entry for any specific date
- **Customizable**: Configure your preferred editor and journal directory
- **Simple Interface**: Minimal CLI interface with intuitive commands
- **Markdown Support**: Journal entries are stored as plain markdown files
- **Concurrent Access Protection**: File locking prevents data corruption when multiple processes access the same journal file

## Installation 🔧

### From Source

1. Ensure you have Rust and Cargo installed ([rustup.rs](https://rustup.rs))
2. Clone the repository:
   ```
   git clone https://github.com/phrazzld/ponder.git
   cd ponder
   ```
3. Build and install:
   ```
   cargo build --release
   cargo install --path .
   ```

## Usage 📝

### Basic Usage

To open today's journal entry:

```bash
ponder
```

### Command Line Options

| Option | Description |
|--------|-------------|
| `-r, --retro` | Opens entries from the past week (excluding today) |
| `-m, --reminisce` | Opens entries from significant past time intervals |
| `-d, --date DATE` | Opens an entry for a specific date (YYYY-MM-DD or YYYYMMDD format) |
| `-v, --verbose` | Enables verbose output for debugging |
| `--log-format FORMAT` | Sets the log output format ("text" or "json") |
| `-h, --help` | Displays help information |
| `-V, --version` | Displays version information |

### Examples

Open today's journal entry:
```bash
ponder
```

Open entries from the past week (retro mode):
```bash
ponder --retro
```

Open entries from significant past intervals (reminisce mode):
```bash
ponder --reminisce
```

Open an entry for a specific date:
```bash
ponder --date 2023-05-15
# or
ponder --date 20230515
```

## Configuration ⚙️

Ponder can be configured using environment variables:

| Environment Variable | Description | Default |
|----------------------|-------------|---------|
| `PONDER_DIR` | The directory where journal entries are stored | `~/Documents/rubberducks` |
| `PONDER_EDITOR` | The editor to use for journal entries | Uses `EDITOR` if set |
| `EDITOR` | Fallback editor if `PONDER_EDITOR` is not set | `vim` |
| `RUST_LOG` | Controls log filtering and verbosity | `info` |
| `CI` | When set to any value, forces JSON log format | Not set |

### Editor Configuration Security

⚠️ **Important Security Restriction**: For security reasons, `PONDER_EDITOR` and `EDITOR` must be set to a single command or an absolute/relative path to an executable, without any embedded spaces or arguments. This prevents command injection vulnerabilities.

**Valid examples:**
```bash
export PONDER_EDITOR="vim"
export PONDER_EDITOR="/usr/bin/nano"
export PONDER_EDITOR="./my-editor"
```

**Invalid examples (will be rejected):**
```bash
export PONDER_EDITOR="vim --noplugin"      # Contains spaces and arguments
export PONDER_EDITOR="code -w"             # Contains spaces and arguments
export PONDER_EDITOR="echo > /tmp/file"    # Contains shell metacharacters
```

### Workarounds for Editor Arguments

If you need to pass arguments to your editor, you can use one of these approaches:

#### Method 1: Shell Alias
Create an alias in your shell configuration file:

```bash
# In your .bashrc, .zshrc, etc.
alias ponder-editor='code -w'
export PONDER_EDITOR="ponder-editor"
```

#### Method 2: Wrapper Script
Create a wrapper script that calls your editor with the desired arguments:

```bash
#!/bin/bash
# Save as ~/bin/ponder-code (or any location in your PATH)
exec code -w "$@"
```

Make it executable:
```bash
chmod +x ~/bin/ponder-code
export PONDER_EDITOR="ponder-code"
```

### Example Configuration

```bash
# Add to your .bashrc, .zshrc, etc.
export PONDER_DIR="~/journals"
export PONDER_EDITOR="vim"  # Simple command without arguments
export RUST_LOG="debug"     # For more verbose logging
```

### Logging Configuration

Ponder uses structured logging with support for both human-readable and JSON output formats:

#### Log Levels

You can control log verbosity using the `RUST_LOG` environment variable:

```bash
# Show only info, warn, and error logs (default)
export RUST_LOG=info

# Show debug logs and above
export RUST_LOG=debug

# Show trace logs (most verbose)
export RUST_LOG=trace

# Filter logs from specific modules
export RUST_LOG=ponder::journal_io=debug,info
```

#### Log Formats

Ponder supports two output formats:

1. **Text** (default): Human-readable output for development
2. **JSON**: Structured output for parsing and analysis

You can select the format using the `--log-format` CLI option:

```bash
# Human-readable output
ponder --log-format text

# JSON output
ponder --log-format json
```

Setting the `CI` environment variable will also force JSON output:

```bash
CI=true ponder
```

#### Correlation IDs

All logs include a correlation ID that uniquely identifies each application invocation. This makes it easier to trace all operations from a single run of the application.

JSON log entries include:
- `timestamp`: ISO 8601 timestamp
- `level`: Log level (`INFO`, `DEBUG`, etc.)
- `target`: Module path
- `span`: Contains `correlation_id` and context information
- `fields`: Contains the log message and other fields

## File Structure 📁

Journal entries are stored as markdown files in the configured journal directory, with filenames in the format `YYYYMMDD.md` (e.g., `20240508.md`).

When you open a journal entry, the current timestamp is automatically added to the file if it doesn't already exist, providing a convenient way to track when you wrote each entry.

### File Locking

Ponder uses advisory file locking to prevent data corruption when multiple processes attempt to access the same journal file simultaneously. This ensures that concurrent invocations of Ponder won't corrupt your journal entries.

If you try to open a journal file that is already being edited in another instance of Ponder, you'll see an error message like:

```
File locking error: Journal file is currently being edited by another process: /path/to/journal/20240521.md
```

Once the first Ponder process completes (the editor is closed), the locks are automatically released, allowing subsequent access to the file.

## Architecture 🏗️

Ponder follows a modular architecture with clear separation of concerns:

- `cli`: Command-line interface handling using clap
- `config`: Configuration loading and validation
- `errors`: Error handling infrastructure
- `journal_core`: Core journal logic without I/O operations
- `journal_io`: Journal I/O operations and file management

The codebase is designed with simplicity and maintainability in mind, using direct function calls and standard library features rather than abstractions. This approach improves readability and makes the code easier to reason about.

The architecture separates pure logic (in `journal_core`) from I/O operations (in `journal_io`), which improves testability and maintainability. Each module has a clearly defined responsibility, with minimal dependencies between modules.

## Contributing 🤝

We welcome contributions to Ponder! If you'd like to contribute:

1. Fork the repository
2. Create a new branch for your feature (`git checkout -b feature/my-new-feature`)
3. Add tests for your changes
4. Make your changes and ensure all tests pass (`cargo test`)
5. Ensure your code is properly formatted and passes linting checks
6. Commit your changes following the Conventional Commits specification
7. Submit a pull request

For more details, please see [CONTRIBUTING.md](./CONTRIBUTING.md).

### Development Setup

To set up your development environment:

```bash
# Clone the repository
git clone https://github.com/phrazzld/ponder.git
cd ponder

# Install pre-commit hooks (recommended)
pip install pre-commit
pre-commit install
```

### Code Formatting and Linting

We use automated tools to ensure consistent code style:

- **rustfmt**: Automatically formats Rust code
  ```bash
  cargo fmt
  ```

- **clippy**: Provides linting and code improvement suggestions
  ```bash
  cargo clippy --all-targets -- -D warnings
  ```

- **pre-commit hooks**: Automatically run formatters and linters before each commit
  - The hooks will run automatically when you commit changes
  - To run them manually: `pre-commit run --all-files`

The repository includes:
- `.rustfmt.toml` - Configuration for rustfmt
- `.pre-commit-config.yaml` - Configuration for pre-commit hooks
- `.vscode/settings.json` - Settings for VS Code users
- `.editorconfig` - Settings for various editors

Please check out the open issues for tasks that need help, or feel free to propose new features or improvements.

## License 📜

Ponder is released under the [MIT License](./LICENSE).