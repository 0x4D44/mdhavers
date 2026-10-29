# MDH-BUG-FLU-00214 — Native: operator-overload default cannae see an earlier parameter (Undefined variable)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-28T22:58:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261029T163736Z-d84bcff3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00214-run-fix-20261029T163736Z-d84bcff3
- **Owner base:** a8453ca1d5cd8fb16d8b4522efc806bf82b6b41c
- **Owner fingerprint:** -
- **Owner since:** 2026-10-29T16:37:36Z
- **Owner until:** 2026-10-29T20:37:36Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261029T163736Z-d84bcff3)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T22:58:08Z, raised via `deltic bugs new --land`)

## Observation

## Observation

An operator-overload method whose default refers to an earlier parameter does not compile natively. The interpreter runs it.

```braw
kin Ops { dae __tak_awa__(that, extra = that + 1) { gie extra } }
blether Ops() - 1
```

- Interpreter: prints `2`, exit 0.
- Native (`mdhavers build`): `Compilation went tits up: Undefined variable: that`.

The same default works natively on an ordinary method (`o.f(1)` with `dae f(that, extra = that + 1)`) and on a top-level function, so only operator-overload dispatch is affected.

Environment: macOS, `--no-default-features --features cli,llvm,native` debug build of the MDH-BUG-FLU-00208/210/211 fix branch (based on 1d961e8, 0.3.116). Found by the Sol bug-lane worker and reproduced by the lead.

Pointer (unconfirmed): the default evaluation in the dynamic operator-overload dispatch in src/llvm/codegen.rs compiles defaults without binding the earlier parameters.

Evidence fingerprint: `manual:v1:native-operator-overload-default-cannae-see-an--f02e7488fb2ef8f4`


## Fix

<unfixed — raised only>

## Notes
