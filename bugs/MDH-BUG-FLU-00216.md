# MDH-BUG-FLU-00216 — Native RC off: readers of a captured variable reassigned by another thread see torn strings

- **State:** Open
- **Priority:** Must
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-29T11:02:29Z
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
- **State history:** Open (2026-10-29T11:02:29Z, raised via `deltic bugs new --land`)

## Observation

## Observation

Native code compiled with RC off (the current default) breaks the documented guarantee for a captured variable that one thread reassigns while others read it. docs/src/reference/builtins.md ("Sharing between threads") says each read sees either the old value or the new one, with no mutex needed. Readers instead see strings with the wrong length.

Found by the RC7 phase 6 worker while writing threaded benchmarks (0.3.125, trunk 489d961, macOS arm64, M5 Max).

## Repro

```
dae run(n) {
    ken shared = "shared0"
    dae writer(count) {
        ken i = 0
        whiles i < count {
            shared = "shared" + tae_string(i % 10)
            i = i + 1
        }
        gie 0
    }
    dae reader(count) {
        ken total = 0
        ken i = 0
        whiles i < count {
            ken snapshot = shared
            total = total + len(snapshot)
            i = i + 1
        }
        gie total
    }
    ken a = thread_spawn(writer, [n])
    ken b = thread_spawn(reader, [n])
    ken c = thread_spawn(reader, [n])
    gie thread_join(a) + thread_join(b) + thread_join(c)
}
blether run(200000)
```

Every complete value has length 7, so the only correct output is `2800000`.

Build with `MDH_RC` unset: `mdhavers build repro.braw -O 2 -o repro && ./repro`.

## Expected vs actual

- Expected: `2800000` (interpreter prints this).
- Native RC off, three runs: `2767142`, `2774450`, `2753613` (exit 0, no crash).
- Native `MDH_RC=1`, three runs, with REPORT and with REPORT+NOFREE: `2800000` each, live counts all zero.
- The original benchmark fixture: interpreter `2800000`, native RC off `2788104`.

## Pointers (not a diagnosis)

The worker reports that src/llvm/codegen.rs:10310-10340 selects the locked shared-cell helpers only when RC is enabled; RC off emits plain loads and stores of the captured cell. Plausibly the string pointer is published before its bytes are visible to readers on ARM's weak memory model; the exact mechanism is not proven. Evidence: RC7 phase 6 worktree `.scratch/rc7p6/` (repro.log, repro-results.json, repro-cell-{off,on}.ll).

Evidence fingerprint: `manual:v1:native-rc-off-readers-of-a-captured-variable-re-816a7b48bac14376`


## Fix

<unfixed — raised only>

## Notes
