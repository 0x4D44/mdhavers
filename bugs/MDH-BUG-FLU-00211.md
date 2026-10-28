# MDH-BUG-FLU-00211 — Native: dynamic operator-overload dispatch nil-fills a missing required argument

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-28T19:36:25Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261028T214658Z-62353abf
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00211-run-fix-20261028T214658Z-62353abf
- **Owner base:** af5199bdf98a8bf4cce0da1cfeb080f3a94793ef
- **Owner fingerprint:** -
- **Owner since:** 2026-10-28T21:46:58Z
- **Owner until:** 2026-10-29T01:46:58Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261028T214658Z-62353abf)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T19:36:25Z, raised via `deltic bugs new --land`)

## Observation

## Observation

Native dynamic operator-overload dispatch pads a missing required parameter with `naething` instead of raising an arity error. The interpreter raises one, and so does the static native path.

Found by the MDH-BUG-FLU-00209 worker (Codex Sol); reproduced by the lead on 2d306bb (trunk + the 209 fix, which does not touch codegen), debug build `--no-default-features --features cli,llvm,native`, macOS arm64.

### Repro

```
kin Ops {
    dae __tak_awa__(that, extra) { gie 7 }
}
blether Ops() - 1
```

### Expected (interpreter)
Exit 1: `Function '__tak_awa__' expects 2 arguments but ye gave it 1 at line 4`

### Actual (native)
`mdhavers build` succeeds; the binary prints `7` and exits 0.

Control: the static form `ken a = Ops()` then `blether a - 1` fails to compile natively with the same arity error, as round 3 (decisions 138/148) requires.

Worker's pointer (unconfirmed): the dynamic overload fallback at src/llvm/codegen.rs:~15761 (nil fill at ~15767/15772).

Evidence fingerprint: `manual:v1:native-dynamic-operator-overload-dispatch-nil-f-1081caea5f5a416e`


## Fix

<unfixed — raised only>

## Notes
