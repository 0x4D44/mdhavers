# mdhavers Playground

Interactive web-based IDE for the mdhavers Scots programming language.

## Features

- **Live Code Execution**: Run mdhavers code directly in your browser (client-side WASM)
- **Syntax Highlighting**: JetBrains Mono font with dark theme
- **Code Formatting**: Auto-format your code with one click
- **JavaScript Compilation**: See the compiled JavaScript output
- **Example Code**: Built-in examples covering all language features
- **Share Links**: Share your code via URL

## Building

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- wasm-pack (`cargo install wasm-pack`)

### Build Steps

```bash
# From the playground directory
./build.sh

# Or manually:
wasm-pack build --target web
cp -r pkg web/
```

## Running Locally

```bash
cd web
python3 -m http.server 8080
# Open http://localhost:8080
```

## Deployment

The `web/` directory contains everything needed for deployment:
- `index.html` - Main page
- `styles.css` - Styling
- `app.js` - Application logic
- `pkg/` - WASM module and JavaScript bindings

Simply serve the `web/` directory from any static file host (GitHub Pages, Netlify, Vercel, etc.).

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Enter` | Run code |
| `Ctrl+Shift+F` | Format code |
| `Tab` | Insert 4 spaces |
| `Escape` | Close modal |
