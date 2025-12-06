# Installation

Get mdhavers running on your system in just a few minutes.

## Prerequisites

mdhavers is written in Rust, so you'll need Rust installed on your system.

### Installing Rust

If you don't have Rust installed:

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**
Download and run [rustup-init.exe](https://win.rustup.rs/)

After installation, restart your terminal and verify:
```bash
rustc --version
cargo --version
```

## Installing mdhavers

### From Source (Recommended)

1. **Clone the repository:**
   ```bash
   git clone https://github.com/0x4d44/mdhavers.git
   cd mdhavers
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Add to your PATH:**

   **Linux/macOS:**
   ```bash
   # Add this line to your ~/.bashrc, ~/.zshrc, or ~/.profile
   export PATH="$PATH:/path/to/mdhavers/target/release"
   ```

   **Windows (PowerShell):**
   ```powershell
   # Add to your system PATH, or run from the directory
   $env:PATH += ";C:\path\to\mdhavers\target\release"
   ```

4. **Verify installation:**
   ```bash
   mdhavers --help
   ```

### Quick Test

Create a file called `test.braw` with:
```scots
blether "It works! Braw!"
```

Run it:
```bash
mdhavers test.braw
```

You should see:
```
It works! Braw!
```

## Building the LSP Server

For IDE support with features like auto-completion and error highlighting:

```bash
cargo build --release
```

The LSP binary will be at `target/release/mdhavers-lsp`.

See [Editor Setup](./editor-setup.md) for configuring your editor.

## Troubleshooting

### "Command not found" after installation

Make sure the `target/release` directory is in your PATH. You can verify with:
```bash
echo $PATH  # Linux/macOS
echo %PATH%  # Windows cmd
```

### Permission denied on Linux/macOS

If you get permission errors when running, ensure the binary is executable:
```bash
chmod +x target/release/mdhavers
```

### Compilation errors

Make sure you have the latest stable Rust:
```bash
rustup update stable
```

## Next Steps

Now that mdhavers is installed, let's write your [first program](./hello-world.md)!
