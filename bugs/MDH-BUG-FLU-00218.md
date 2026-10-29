# MDH-BUG-FLU-00218 — Native: snooze, noo and tick in a loop grow the stack every iteration until it overflows

- **State:** Open
- **Priority:** Must
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-29T16:53:25Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261029T170202Z-661f2b2b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00218-run-fix-20261029T170202Z-661f2b2b
- **Owner base:** e6acf783f9a42bcba4ad2463de3eb56b0ff02a63
- **Owner fingerprint:** -
- **Owner since:** 2026-10-29T17:02:02Z
- **Owner until:** 2026-10-29T21:02:02Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261029T170202Z-661f2b2b)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-29T16:53:25Z, raised via `deltic bugs new --land`)

## Observation

## Observation

A native program that calls `snooze`, `noo()` or `tick()` inside a loop grows its stack on every iteration and eventually dies with the stack-overflow error, though nothing recurses. Bogle Castle (native, RC on, 0.3.129) exited with status 1 after about 43 minutes of play: `Stack's fair puggled! Too many nested calls at line 562 - yer recursion's gone radge!` (the reported line, in games/bogle/engine/render.braw's text drawer, is not recursive). Its shell loop is `whiles nae screen_should_close() { ... snooze(1) }`.

## Repro

```
ken i = 0
whiles i < 2000000 {
    snooze(0)
    i = i + 1
}
blether "done"
```

`mdhavers build r.braw -O 2 -o r && ./r`

## Expected vs actual

- Expected: `done`.
- Actual, RC on and `MDH_RC=0` alike: `Stack's fair puggled! Too many nested calls at line 3 - yer recursion's gone radge!`
- The same loop with `t = noo()` or `t = tick()` in place of `snooze(0)`: the same error (line 4).
- The `snooze` loop inside a function (`dae run(n) { whiles ... }`): the same error.
- Controls that print `done`: `t = mono_ms()` in the loop; `snooze(0)` inside a helper function called from the loop; a `fer` loop nested in the `whiles`.

Observed with a debug compiler built from 0.3.130 plus an unrelated uncommitted change (bug 214 worktree), macOS arm64.

## Pointers (not a diagnosis)

Bogle's IR (`--emit-llvm`) has `%timespec = alloca { i64, i64 }` in main's `snooze_ok` block, not the entry block; it is main's only alloca outside the entry block. `src/llvm/codegen.rs` builds `timespec` allocas with `self.builder.build_alloca` at about lines 31374 (inline_noo), 31443 (inline_tick) and 43035; there are about 110 direct `build_alloca` calls in that file, and the scan also shows loop counters (`for_str_idx`, `i`, `counter`) allocated outside entry blocks in many functions, so other builtins or statements may leak the same way inside a loop.

Evidence fingerprint: `manual:v1:native-snooze-noo-and-tick-in-a-loop-grow-the-s-6b763b760e195457`


## Fix

<unfixed — raised only>

## Notes
