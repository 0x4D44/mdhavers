# mdhavers VSCode Extension

This folder contains a VSCode extension that provides:

- Syntax highlighting for `.braw` files
- Optional LSP integration via the `mdhavers-lsp` binary

## Build

From the repo root:

```bash
cd editor/vscode
npm run compile
```

The compiled extension entrypoint is `editor/vscode/out/extension.js`.
Sourcemaps (`editor/vscode/out/*.map`) are intentionally ignored.

## LSP setup

The extension runs the language server using either:

- `mdhavers.lsp.path` (VSCode setting), or
- a locally-built repo binary (prefers `target/release/mdhavers-lsp`, then `target/debug/mdhavers-lsp`), or
- `mdhavers-lsp` on `PATH`

Build the language server with:

```bash
cargo build --release --features cli
```

## Notes on vendoring

This repository currently commits `editor/vscode/node_modules/`.
If you update dependencies, re-run `npm install` and recompile the extension.
