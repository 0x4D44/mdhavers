# MDH-BUG-FLU-00210 — Native: module functions cannae see the module's own aliased fetch (Undefined variable)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-28T19:30:20Z
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
- **State history:** Open (2026-10-28T19:30:20Z, raised via `deltic bugs new --land`) -> Fixed (2026-10-28T22:58:54Z, deltic:auto role=fix run=fix-20261028T214658Z-97bfca13 branch=task/bug-MDH-BUG-FLU-00210-run-fix-20261028T214658Z-97bfca13 code=96d07e4 gate=manual) -> Closed (2026-10-28T23:59:33Z, deltic:manual role=verify run=verify-20261028T233525Z-02cdce5b verified_fix_run=fix-20261028T214658Z-97bfca13 verdict=close)

## Observation

## Observation

Native compilation fails when a fetched module itself uses an aliased `fetch` and its functions use that alias. The interpreter and JS run it.

Found by the 206/207/208 verifier (Codex Sol); reproduced by the lead on trunk 23ecbe2, debug build `--no-default-features --features cli,llvm,native,wasm_runner`, macOS arm64.

### Repro (three files in one folder)

leaf.braw:
```
blether "leaf-init"
ken state = "leaf"
dae read_state() { gie state }
dae write_state(v) { state = v }
```

bridge.braw:
```
fetch "leaf" tae leaf
dae bridge_read() { gie leaf.read_state() }
dae bridge_write(v) { leaf.write_state(v) }
```

nested.braw:
```
fetch "bridge" tae bridge
fetch "leaf" tae leaf
blether bridge.bridge_read()
bridge.bridge_write("nested")
blether bridge.bridge_read()
blether leaf.read_state()
```

### Expected (interpreter, also JS)
```
leaf-init
leaf
nested
nested
```

### Actual (native)
`mdhavers build nested.braw` exits 1: `Blimey!: Och! Compilation went tits up: Undefined variable: leaf`

Control: when bridge.braw uses a plain `fetch "leaf"`, all three backends agree.

Verifier's inference (unconfirmed): `store_import_alias` (src/llvm/codegen.rs:~36433) stores an init-local alias while `in_user_function` is set, then the variable restore at ~36778 drops it before the exported functions are compiled.

Evidence fingerprint: `manual:v1:native-module-functions-cannae-see-the-module-s-19a3ce83f67913c9`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-28T23:59:33Z, deltic:manual run=verify-20261028T233525Z-02cdce5b verified_fix_run=fix-20261028T214658Z-97bfca13 verdict=close)

Verifier note: Manual verify: Independent Sol verifier on d36dafb: exact three-file repro prints leaf-init/leaf/nested/nested on both backends; root reversal fails 5 native assertions; nested, closure and sibling-alias probes match. The fix's shared helper regressed a coverage-only path (raised separately).

## Notes
