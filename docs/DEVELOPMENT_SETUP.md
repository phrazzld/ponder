# Development Environment Setup

This guide helps you configure your IDE for Rust development with automatic formatting (`rustfmt`) and linting (`clippy`) to maintain consistent code quality throughout the project.

## Prerequisites

Before setting up your IDE, ensure you have the following components installed:

### Rust Toolchain
Install Rust via [rustup](https://rustup.rs/) if you haven't already:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Required Components
Add the necessary Rust components:
```bash
rustup component add rustfmt
rustup component add clippy
rustup component add rust-analyzer  # Language server (optional but recommended)
```

### Verify Installation
Confirm the tools are available:
```bash
cargo fmt --version
cargo clippy --version
rust-analyzer --version  # If installed
```

## Project Configuration

This project uses specific formatting and linting configurations:

- **rustfmt**: Configuration in `.rustfmt.toml` at the project root
- **clippy**: Standard warning level `-D warnings` used in pre-commit hooks and CI

Your IDE should automatically detect and use these project-specific settings.

## IDE Setup Guides

### VS Code

1. **Install rust-analyzer extension**
   - Open VS Code
   - Go to Extensions (Ctrl+Shift+X / Cmd+Shift+X)
   - Search for "rust-analyzer"
   - Install the official rust-analyzer extension

2. **Configure workspace settings**
   
   Create or update `.vscode/settings.json` in your project:
   ```json
   {
       "editor.formatOnSave": true,
       "[rust]": {
           "editor.defaultFormatter": "rust-lang.rust-analyzer"
       },
       "rust-analyzer.checkOnSave.command": "clippy",
       "rust-analyzer.checkOnSave.allTargets": true,
       "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"]
   }
   ```

3. **Alternative: User settings**
   
   For global configuration, add to your user settings (File > Preferences > Settings > Open Settings JSON):
   ```json
   {
       "editor.formatOnSave": true,
       "[rust]": {
           "editor.defaultFormatter": "rust-lang.rust-analyzer"
       },
       "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```

### IntelliJ IDEA / RustRover

1. **Install Rust plugin (IntelliJ IDEA only)**
   - RustRover has built-in Rust support
   - For IntelliJ IDEA: File > Settings > Plugins > Search "Rust" > Install

2. **Configure formatting**
   - File > Settings > Languages & Frameworks > Rust
   - Under "Rustfmt": Check "Use rustfmt instead of built-in formatter"
   - Ensure "Run rustfmt on Save" is checked

3. **Configure linting**
   - File > Settings > Languages & Frameworks > Rust
   - Under "External Linters": Check "Run external linter to analyze code on the fly"
   - Select "Clippy" from the dropdown

4. **Enable format on save**
   - File > Settings > Editor > General > Save Files
   - Check "Reformat code on save"
   - Ensure Rust files are included in the file mask

### Neovim

1. **Prerequisites**
   - An LSP client plugin (e.g., `nvim-lspconfig`)
   - An LSP installer (optional, e.g., `mason.nvim`)

2. **Install rust-analyzer**
   
   Using mason.nvim:
   ```vim
   :MasonInstall rust-analyzer
   ```
   
   Or manually:
   ```bash
   rustup component add rust-analyzer
   ```

3. **Configure LSP**
   
   Add to your Neovim configuration (init.lua):
   ```lua
   -- Assuming you have nvim-lspconfig installed
   local lspconfig = require('lspconfig')
   
   lspconfig.rust_analyzer.setup({
       settings = {
           ['rust-analyzer'] = {
               checkOnSave = {
                   command = "clippy",
                   allTargets = true,
                   extraArgs = { "--", "-D", "warnings" }
               },
               -- rust-analyzer respects .rustfmt.toml automatically
           }
       },
       on_attach = function(client, bufnr)
           -- Enable format on save
           if client.server_capabilities.documentFormattingProvider then
               vim.api.nvim_create_autocmd("BufWritePre", {
                   pattern = "*.rs",
                   callback = function()
                       vim.lsp.buf.format({ async = false })
                   end,
                   buffer = bufnr,
               })
           end
           
           -- Optional: Set up key mappings
           local opts = { buffer = bufnr }
           vim.keymap.set('n', 'gD', vim.lsp.buf.declaration, opts)
           vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
           vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
           vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, opts)
       end
   })
   ```

## Verification

After setting up your IDE, verify the configuration works correctly:

### 1. Test Formatting
1. Open any `.rs` file in the project
2. Add incorrect formatting:
   ```rust
   fn test() {
   let x=1;
       let y=2;
   }
   ```
3. Save the file
4. The code should automatically reformat to:
   ```rust
   fn test() {
       let x = 1;
       let y = 2;
   }
   ```

### 2. Test Linting
1. Add code that triggers a clippy warning:
   ```rust
   fn main() {
       let v = vec![1, 2, 3];
       let _cloned = v.clone();  // Unnecessary clone
   }
   ```
2. Save the file or trigger linting
3. You should see a warning about unnecessary `.clone()`

### 3. Test Project Configuration
1. Create a file with specific formatting:
   ```rust
   // This should respect the project's .rustfmt.toml settings
   fn very_long_function_name_that_might_exceed_line_width(param1: String, param2: String) -> Result<(), Error> {
       Ok(())
   }
   ```
2. Save and verify it formats according to project rules

## Troubleshooting

### rustfmt/clippy not found
- Ensure `~/.cargo/bin` is in your system PATH
- Verify with: `which cargo` and `which rustfmt`
- Restart your IDE after installing components

### Format-on-save not working
- Check for conflicting formatter extensions
- Verify the correct formatter is set as default for Rust files
- Try manually running: `cargo fmt` to ensure it works

### Clippy warnings not appearing
- Check IDE output/log panels for errors
- Verify rust-analyzer is running: Look for rust-analyzer in IDE's language server status
- Try running `cargo clippy` manually to ensure it works

### Wrong formatting rules applied
- Ensure no user-level `~/.rustfmt.toml` is overriding project settings
- Verify your IDE is using the project directory as the workspace root
- Check that `.rustfmt.toml` exists in the project root

### VS Code specific issues
- If rust-analyzer isn't starting, check: View > Output > rust-analyzer
- Try reloading the window: Ctrl+Shift+P > "Developer: Reload Window"

### IntelliJ specific issues
- Invalidate caches: File > Invalidate Caches and Restart
- Check plugin compatibility with your IDE version

### Neovim specific issues
- Check LSP status: `:LspInfo`
- View LSP logs: `:LspLog`
- Ensure your Neovim version supports LSP (0.5.0+)

## Additional Resources

- [rust-analyzer Manual](https://rust-analyzer.github.io/manual.html)
- [rustfmt Configuration](https://rust-lang.github.io/rustfmt/)
- [Clippy Lints Documentation](https://rust-lang.github.io/rust-clippy/master/)
- [VS Code Rust Extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [IntelliJ Rust Plugin](https://www.jetbrains.com/rust/)
- [Neovim LSP Documentation](https://neovim.io/doc/user/lsp.html)

## Contributing

If you encounter issues with these instructions or have suggestions for other IDEs, please open an issue or submit a pull request to improve this documentation.