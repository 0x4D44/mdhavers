# mdhavers Installer

This directory contains the installation scripts for mdhavers.

## Quick Install

### From Local Build

```bash
# Build and install
make install-local

# Or manually:
./installer/install.sh --local
```

### From Release (Future)

```bash
curl -sSf https://raw.githubusercontent.com/.../install.sh | sh
```

## Installation Details

The installer will:
1. Create `~/.mdhavers/` directory
2. Install binaries to `~/.mdhavers/bin/`
3. Install shell completions to `~/.mdhavers/completions/`
4. Install standard library to `~/.mdhavers/lib/`
5. Install examples to `~/.mdhavers/examples/`
6. Add mdhavers to your PATH via shell rc file

## After Installation

Restart your shell or run:
```bash
source ~/.mdhavers/env
```

Then verify:
```bash
mdhavers --version
```

## Uninstall

```bash
make uninstall

# Or manually:
./installer/uninstall.sh
```

## Directory Structure After Install

```
~/.mdhavers/
├── bin/
│   ├── mdhavers          # Main compiler/interpreter
│   └── mdhavers-lsp      # Language Server Protocol server
├── completions/
│   ├── mdhavers.bash     # Bash completions
│   ├── mdhavers.zsh      # Zsh completions
│   └── mdhavers.fish     # Fish completions
├── lib/                  # Standard library
├── examples/             # Example programs
└── env                   # Environment setup script
```

## Options

### install.sh

- `--local` - Install from local build (development)
- `--yes`, `-y` - Skip confirmation prompts
- `--help`, `-h` - Show help

### uninstall.sh

- `--yes`, `-y` - Skip confirmation prompts
- `--help`, `-h` - Show help

## Shell Completions

After installation, completions should work automatically. If not, add to your rc file:

**Bash** (~/.bashrc):
```bash
source ~/.mdhavers/completions/mdhavers.bash
```

**Zsh** (~/.zshrc):
```zsh
source ~/.mdhavers/completions/mdhavers.zsh
```

**Fish** (~/.config/fish/config.fish):
```fish
source ~/.mdhavers/completions/mdhavers.fish
```
