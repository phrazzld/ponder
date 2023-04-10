# ponder 🤔

`ponder` is a small Rust CLI app that opens a file for journaling your thoughts, often referred to as rubber ducking. Each time you run `ponder`, it creates or opens a file named by the current date in the `rubberducks` directory under your home folder, and appends the current time to the file.

## Features 🌟

- Automatically creates or opens a file for the current date
- Appends the current time to the file
- Opens the file in the user's default text editor (falls back to `vim` if not set)

## Usage 🚀

To use `ponder`, simply run it from the command line. This will open the journal file for the current date in your default text editor, or `vim` if you haven't set a default editor.

## Contributing 🤝

We welcome contributions to the project! If you're interested in contributing, please check out the open issues or submit a pull request with your improvements. We encourage you to open issues and pull requests with questions, problems, and solutions.

## License 📜

`ponder` is released under the [MIT License](./LICENSE).
