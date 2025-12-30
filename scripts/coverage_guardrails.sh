#!/usr/bin/env bash
set -euo pipefail

SUMMARY_PATH="${SUMMARY_PATH:-target/llvm-cov-summary.json}"
COV_TIMEOUT_SECONDS="${COV_TIMEOUT_SECONDS:-1800}"

timeout "${COV_TIMEOUT_SECONDS}" cargo llvm-cov --no-default-features --features cli,llvm,native \
  --summary-only --json --output-path "${SUMMARY_PATH}"

jq '.data[0].totals' "${SUMMARY_PATH}"

# Floors: these should not regress.
jq -e '
  .data[0].totals.functions.percent == 100
  and .data[0].totals.instantiations.percent == 100
' "${SUMMARY_PATH}" >/dev/null

# Hotspot ordering guardrail (see wrk_docs coverage plan).
INTERPRETER_MISSED_REGIONS="$(
  jq -r '
    .data[0].files[]
    | select(.filename | endswith("/src/interpreter.rs"))
    | .summary.regions.notcovered
  ' "${SUMMARY_PATH}"
)"
CODEGEN_MISSED_REGIONS="$(
  jq -r '
    .data[0].files[]
    | select(.filename | endswith("/src/llvm/codegen.rs"))
    | .summary.regions.notcovered
  ' "${SUMMARY_PATH}"
)"

if [[ -z "${INTERPRETER_MISSED_REGIONS}" || -z "${CODEGEN_MISSED_REGIONS}" ]]; then
  echo "error: failed to locate interpreter/codegen in ${SUMMARY_PATH}" >&2
  exit 1
fi

if [[ "${INTERPRETER_MISSED_REGIONS}" -gt "${CODEGEN_MISSED_REGIONS}" ]]; then
  echo "error: hotspot ordering invariant violated (interpreter ${INTERPRETER_MISSED_REGIONS} > codegen ${CODEGEN_MISSED_REGIONS})" >&2
  exit 1
fi

echo "OK: coverage guardrails satisfied (interpreter ${INTERPRETER_MISSED_REGIONS} <= codegen ${CODEGEN_MISSED_REGIONS})"
