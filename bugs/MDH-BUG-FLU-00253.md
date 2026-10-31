# MDH-BUG-FLU-00253 — Native: lambda capturing a parameter after an element assignment breaks the second call

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-31T08:08:04Z
- **Discovery source:** Agent
- **Owner:** -
- **Owner role:** -
- **Owner run:** -
- **Owner host:** -
- **Owner branch:** -
- **Owner base:** -
- **Owner fingerprint:** -
- **Owner since:** -
- **Owner until:** -
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-31T08:08:04Z, raised via `deltic bugs new --land`)

## Observation

## Observation
In a native LLVM build, a function that assigns into an element of a dict
parameter and then creates a lambda capturing that parameter fails on its
second call. The interpreter is correct.

```
dae tick(box) {
    box["n"] = box["n"] + 1
    ken f = |x| box["n"]
    gie box["n"]
}
ken b = {"n": 1}
blether tick(b)
blether tick(b)
```

- Interpreter (`mdhavers run`): prints `2` then `3`, exit 0.
- Native (`mdhavers build`, then run), MDH_RC=0 and MDH_RC=1 alike: prints
  `Och! Type error in 'index': got bool and string`, exit 1.

## Environment
macOS arm64, trunk compiler at 0.3.155 (mdh release build of e6252e9+),
LLVM 19. Found by a worker on the wasm branch; reproduced by the lead with
the trunk compiler, so it is not caused by that branch.

## Expected
Native output equals the interpreter's: `2`, `3`.

Evidence fingerprint: `manual:v1:native-lambda-capturing-a-parameter-after-an-el-4e8a07accbd1d3ee`


## Fix

<unfixed — raised only>

## Notes
