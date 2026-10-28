# MDH-BUG-FLU-00210 — Native: module functions cannae see the module's own aliased fetch (Undefined variable)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-28T19:30:20Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261028T214658Z-97bfca13
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00210-run-fix-20261028T214658Z-97bfca13
- **Owner base:** af5199bdf98a8bf4cce0da1cfeb080f3a94793ef
- **Owner fingerprint:** -
- **Owner since:** 2026-10-28T21:46:58Z
- **Owner until:** 2026-10-29T01:46:58Z
- **Owner session:** flux/2a9090d7-08f7-48c7-bee8-7b7d1d963ab2 (run fix-20261028T214658Z-97bfca13)
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-28T19:30:20Z, raised via `deltic bugs new --land`)

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

## Notes
