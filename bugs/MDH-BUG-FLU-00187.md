# MDH-BUG-FLU-00187 — Native tri: Renderar.loop holds the engine borrow and state lock while user code runs

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T11:40:45Z
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
- **State history:** Open (2026-10-26T11:40:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N11 2026-10-26 (code read, not executed: needs a window): __mdh_tri_rs_call holds the tri state lock and the engine RefCell borrow for the whole of Renderar.loop (runtime/mdh_runtime_rs/src/tri_runtime.rs tri_rs_call -> with_object; tri_engine.rs:39 with_engine). A loop callback that touches any tri object or calls with_engine would deadlock or panic 'already borrowed', so a normal render loop that moves a mesh cannot work. Fix: release the lock and borrow around each callback invocation; add a headless or offscreen test if feasible.

Evidence fingerprint: `manual:v1:native-tri-renderar-loop-holds-the-engine-borro-4e0d0adff35191f0`


## Fix

<unfixed — raised only>

## Notes
