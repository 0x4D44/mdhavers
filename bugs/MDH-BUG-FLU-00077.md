# MDH-BUG-FLU-00077 — JS backend sort uses its own ordering, diverging from the interpreter on mixed types

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-ad7a735d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00077-run-verify-20261026T225713Z-ad7a735d
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:690695a2bf0f4e295c12b8e3ef099a4ea66ca2f56885f6dde783dc7f17ce1c26
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:45:08Z, deltic:auto role=fix run=fix-20261023T034502Z-95b64d22 branch=task/bug-MDH-BUG-FLU-00077-run-fix-20261023T034502Z-95b64d22 code=54979e9 gate=manual)

## Observation

Found by the interpreter lane: src/compiler.rs ~258 sort helper does not follow the interpreter's new total order (numbers, then strings, then others).

## Fix

<unfixed — raised only>

## Notes
