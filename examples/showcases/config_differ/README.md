# Config Differ - pretty-print, diff and patch nested data

Two versions of a bothy's booking config, run through a small toolkit:

- a `Printer` kin that lays out any value with sorted keys and indentation.
  It keeps a value on one line when it fits the width.
- a structural `Differ`. One `==` skips any subtree that matches, because
  dicts compare by content. It reports `+`, `-` and `~` changes by path, such
  as `rooms[0].beds`.
- a tally of changes by kind. It leaves out kinds with none, so "no changes"
  is an empty dict, and an empty dict is falsy.
- a patcher that replays the diff on a deep copy of the old config. It then
  checks that the result `==` the new config.
- exact integers: a change of 1 at 2^53 + 1 is still seen, and a total that
  overflows is caught
- required-field checks built as one fresh closure per `fer` iteration

## Running

```bash
mdhavers run examples/showcases/config_differ/config_differ.braw

mdhavers compile examples/showcases/config_differ/config_differ.braw -o differ.js
node differ.js
```

## Backend status (2026-10-24)

- Interpreter and JavaScript: identical output.
- Native (LLVM): the binary segfaults. Inside `inline`, a lambda in a `gin`
  branch captures the parameter `v`. The capture boxes `v` only on that
  branch, but code after the branch reads `v` as boxed on every path. If
  `inline`'s dict branch uses a `fer` loop instead of the lambda, the native
  output matches.
- WASM: does not compile (no builtins or `kin`).

The golden test `tests/golden/integration/showcase_config_differ.braw`
records these gaps.
