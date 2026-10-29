# MDH-BUG-FLU-00213 — Native RC: a string returned through thread_join is still live at exit

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-28T21:49:28Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261029T161549Z-8ed87b09
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00213-run-fix-20261029T161549Z-8ed87b09
- **Owner base:** 48408743063f7079c91c32eebf020e9213747dc3
- **Owner fingerprint:** -
- **Owner since:** 2026-10-29T16:15:49Z
- **Owner until:** 2026-10-29T20:15:49Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261029T161549Z-8ed87b09)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T21:49:28Z, raised via `deltic bugs new --land`)

## Observation

## Observation

Under `MDH_RC=1`, a heap string returned from a spawned thread and passed through `thread_join` is still live at exit.

Found by the RC7 phase B worker (Codex Sol); reproduced by it on both the pristine trunk compiler (0.3.114) and the phase B candidate, macOS arm64, `--no-default-features --features cli,llvm,native`. Not related to phase B. (Different from MDH-BUG-FLU-00152, closed, which was the spawned function and argument list.)

### Repro

```
ken message = "from-main" + ""
dae worker() { gie message }
blether thread_join(thread_spawn(worker, []))
```

Run natively with `MDH_RC=1 MDH_RC_REPORT=1`.

### Expected
Output `from-main`, exit 0, REPORT `total=0`.

### Actual
Output `from-main`, exit 0, but REPORT: `mdhavers rc: live at exit: string=1 list=0 dict=0 closure=0 instance=0 bytes=0 bound method=0 native=0 total=1`.

Pointer (inference, unconfirmed): the runtime keeps the compiled result in the thread record (runtime/mdh_runtime.c ~4402) and thread_join returns it (~4469); the record's or the join's ownership of that result looks unbalanced.

Evidence fingerprint: `manual:v1:native-rc-a-string-returned-through-thread-join-9fce0006985b988f`


## Fix

<unfixed — raised only>

## Notes
