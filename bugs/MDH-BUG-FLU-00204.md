# MDH-BUG-FLU-00204 — LSP stdout-disconnect coverage test races under parallel load and sees exit 0

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** lsp
- **Raised:** 2026-10-27T06:58:38Z
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
- **State history:** Open (2026-10-27T06:58:38Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T07:16:25Z, deltic:auto role=fix run=fix-20261027T070623Z-ff615304 branch=task/bug-MDH-BUG-FLU-00204-run-fix-20261027T070623Z-ff615304 code=a6b3584 gate=manual)

## Observation

tests/lsp_coverage.rs:396 `lsp_binary_reports_io_thread_join_error_after_stdout_disconnect_for_coverage` failed once in a full parallel run (2026-10-27 06:55 GMT, `--features wasm_runner`, `--test-threads=6`, load ~60, trunk + an unrelated runtime-only change):

    expected mdhavers-lsp to fail joining IO threads after stdout disconnect; status=Some(0)

stderr showed a normal start-up and "shuttin' doon. Cheerio!". Run alone five times right after, it passed 5/5. The test depends on the order in which the server's stdout is closed and its IO threads notice; under load the server can finish cleanly before the disconnect is seen, so it exits 0. Same class as the other load-sensitive tests raised this night (198, 201, 202).

Expected: the test forces the disconnect before the server can shut down cleanly (or waits on a deterministic signal), so it fails only when the join error is not reported.

Evidence fingerprint: `manual:v1:lsp-stdout-disconnect-coverage-test-races-under-09272688d5f44bb2`


## Fix

<unfixed — raised only>

## Notes
