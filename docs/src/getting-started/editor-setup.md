# Editor Setup

Get syntax highlighting and IDE features for mdhavers in your favorite editor.

## VS Code (Recommended)

VS Code offers the best experience with full LSP support.

### Installing the Extension

1. Navigate to the `editor/vscode` directory in your mdhavers installation
2. Install dependencies and compile:
   ```bash
   cd editor/vscode
   npm install
   npm run compile
   ```
3. Copy the folder to your VS Code extensions directory:
   - **Linux:** `~/.vscode/extensions/`
   - **macOS:** `~/.vscode/extensions/`
   - **Windows:** `%USERPROFILE%\.vscode\extensions\`
4. Restart VS Code

### Features

Once installed, you get:
- Syntax highlighting for `.braw` files
- Real-time error diagnostics
- Hover documentation for keywords and built-ins
- Auto-completion with Scottish-flavored suggestions
- Code snippets

### Configuration

In VS Code settings, you can configure:

```json
{
    "mdhavers.lsp.path": "mdhavers-lsp",
    "mdhavers.lsp.enable": true
}
```

If the LSP binary isn't in your PATH, provide the full path:
```json
{
    "mdhavers.lsp.path": "/path/to/mdhavers/target/release/mdhavers-lsp"
}
```

## Vim / Neovim

### Syntax Highlighting

1. Copy the syntax files to your Vim configuration:
   ```bash
   # For Vim
   cp -r editor/vim/* ~/.vim/

   # For Neovim
   cp -r editor/vim/* ~/.config/nvim/
   ```

2. Add to your `.vimrc` or `init.vim`:
   ```vim
   au BufNewFile,BufRead *.braw set filetype=mdhavers
   ```

### LSP Support (Neovim)

For Neovim users with nvim-lspconfig:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define mdhavers LSP
configs.mdhavers = {
  default_config = {
    cmd = { 'mdhavers-lsp' },
    filetypes = { 'mdhavers' },
    root_dir = lspconfig.util.find_git_ancestor,
    single_file_support = true,
  },
}

-- Set up the LSP
lspconfig.mdhavers.setup({})
```

This gives you:
- Error diagnostics
- Hover information
- Basic completion

## Sublime Text / TextMate

Use the TextMate grammar file:

1. Locate `editor/mdhavers.tmLanguage.json`
2. Import it into your editor:
   - **Sublime Text:** Copy to `Packages/User/`
   - **TextMate:** Bundle it as a language grammar

## Other Editors

Any editor with LSP support can use mdhavers-lsp. Configure it with:

- **Command:** `mdhavers-lsp`
- **File types:** `*.braw`
- **Communication:** stdio

### Example: Emacs with lsp-mode

```elisp
(add-to-list 'auto-mode-alist '("\\.braw\\'" . prog-mode))

(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration
    '(prog-mode . "mdhavers"))

  (lsp-register-client
    (make-lsp-client
      :new-connection (lsp-stdio-connection '("mdhavers-lsp"))
      :major-modes '(prog-mode)
      :server-id 'mdhavers)))
```

## Manual Syntax Highlighting

If your editor doesn't support TextMate grammars or LSP, here are the key patterns:

### Keywords
```
ken, gin, ither, than, whiles, fer, gie, blether, speir, fae, tae, an, or,
nae, aye, naething, dae, thing, fetch, kin, brak, haud, in, is, masel,
hae_a_bash, gin_it_gangs_wrang, keek, whan, mak_siccar
```

### Comments
```
# Single line comments start with hash
```

### Strings
```
"double quoted strings"
'single quoted strings'
f"interpolated strings with {variables}"
```

### Numbers
```
42        # Integer
3.14      # Float
-17       # Negative
```

### Operators
```
+ - * / %     # Arithmetic
== != < > <= >=  # Comparison
an or nae     # Logical (and, or, not)
|>            # Pipe
...           # Spread
..            # Range
```

## Verifying Your Setup

Create a test file `test.braw`:

```scots
# Test file for editor setup
ken name = "mdhavers"
ken version = 1

dae greet(who) {
    blether f"Hullo fae {who}!"
}

gin version > 0 {
    greet(name)
}
```

If syntax highlighting is working, you should see:
- Keywords (`ken`, `dae`, `gin`, `blether`) in one color
- Strings in another color
- Comments in a muted color
- Numbers highlighted

If you have LSP working, you should see:
- No error squiggles on valid code
- Hover information when you mouse over keywords
- Completion suggestions as you type

## Troubleshooting

### No syntax highlighting

- Ensure the file extension is `.braw`
- Check that syntax files are in the correct location
- Restart your editor after adding files

### LSP not connecting

- Verify `mdhavers-lsp` is in your PATH: `which mdhavers-lsp`
- Check your editor's LSP logs for connection errors
- Ensure the binary has execute permissions

### Wrong highlighting

The TextMate grammar might conflict with other grammars. Try:
- Disabling other language extensions temporarily
- Setting the file type explicitly
