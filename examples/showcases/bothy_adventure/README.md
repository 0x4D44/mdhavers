# The Bothy at Glen Haar - a wee text adventure

A small adventure engine that plays a scripted walk through a Highland glen.
It reads no input. It has:

- rooms as dicts, with exits, items and a dark cellar
- a pack (inventory) and a visited-rooms list, both creels
- a `Parser` kin that lower-cases, drops filler words ("the", "a", "tae"),
  maps synonyms (`get`/`grab` -> `tak`, `n` -> `north`) and turns a bare
  direction into `go`. Parsed commands are dicts, so `parse("n") ==
  parse("go north")` holds.
- verbs as lambdas in a dispatch dict
- mistakes hurled and caught with Scots complaints
- the script queued as one closure per `fer` iteration, then played in order
- a final reckoning that uses dict and creel equality

## Running

```bash
mdhavers run examples/showcases/bothy_adventure/bothy_adventure.braw
```

## Backend status (2026-10-24)

- Interpreter: the reference output.
- JavaScript: fails. The JS runtime has no creel builtins (`creel`,
  `toss_in`, `heave_oot`, `is_in_creel`, `creel_tae_list`). With a
  list-backed creel shim the output matches.
- Native (LLVM): runs, but prints a stray `Ye see: ` after a room is emptied.
  A condition that reads an empty list straight from an index
  (`gin room["items"]`) is treated as truthy.
- WASM: does not compile (no builtins or `kin`).

The golden test `tests/golden/integration/showcase_bothy_adventure.braw`
records these gaps.
