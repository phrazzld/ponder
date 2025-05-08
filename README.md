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

### Example Configuration

```bash
# Add to your .bashrc, .zshrc, etc.
export PONDER_DIR="~/journals"
export PONDER_EDITOR="code -w"
```

## File Structure 📁

Journal entries are stored as markdown files in the configured journal directory, with filenames in the format `YYYYMMDD.md` (e.g., `20240508.md`).

When you open a journal entry, the current timestamp is automatically added to the file if it doesn't already exist, providing a convenient way to track when you wrote each entry.

## Architecture 🏗️

Ponder follows a modular architecture with clear separation of concerns:

- `cli`: Command-line interface handling using clap
- `config`: Configuration loading and validation
- `editor`: Editor abstraction for opening journal files
- `errors`: Error handling infrastructure
- `journal`: Core journal functionality with dependency injection

The codebase is designed with testability in mind, using dependency injection and trait-based abstractions to allow for easy mocking and testing.

## Contributing 🤝

We welcome contributions to Ponder! If you'd like to contribute:

1. Fork the repository
2. Create a new branch for your feature (`git checkout -b feature/my-new-feature`)
3. Add tests for your changes
4. Make your changes and ensure all tests pass (`cargo test`)
5. Run `cargo fmt` and `cargo clippy` to ensure code quality
6. Commit your changes following the Conventional Commits specification
7. Submit a pull request

Please check out the open issues for tasks that need help, or feel free to propose new features or improvements.

## License 📜

Ponder is released under the [MIT License](./LICENSE).