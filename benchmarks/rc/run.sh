#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
  printf 'Usage: %s /path/to/release/mdhavers [runs-per-variant] [--check-baseline]\n' "$0" >&2
  printf 'Runs per variant defaults to 3 and must be at least 3.\n' >&2
}

if [[ $# -lt 1 || $# -gt 3 ]]; then
  usage
  exit 2
fi

compiler=$1
runs=3
runs_set=0
check_baseline=0
shift
for argument in "$@"; do
  if [[ $argument == --check-baseline ]]; then
    if (( check_baseline )); then
      printf 'duplicate --check-baseline flag\n' >&2
      exit 2
    fi
    check_baseline=1
  elif (( ! runs_set )) && [[ $argument =~ ^[0-9]+$ ]]; then
    runs=$argument
    runs_set=1
  else
    usage
    exit 2
  fi
done
if [[ ! $runs =~ ^[0-9]+$ ]] || (( runs < 3 )); then
  printf 'runs-per-variant must be an integer of at least 3\n' >&2
  exit 2
fi
if [[ ! -x $compiler ]]; then
  printf 'compiler is not executable: %s\n' "$compiler" >&2
  exit 2
fi
if [[ ! -x /usr/bin/time ]]; then
  printf '/usr/bin/time is unavailable\n' >&2
  exit 2
fi
if ! command -v deltic >/dev/null 2>&1; then
  printf 'deltic is required for the 30-second run timeout\n' >&2
  exit 2
fi

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)
bench_dir=$repo_root/benchmarks/rc
scratch_dir=$repo_root/.scratch
mkdir -p "$scratch_dir"
work_dir=$(mktemp -d "$scratch_dir/rc-bench.XXXXXX")
printf 'Benchmark artifacts retained in %s\n' "$work_dir" >&2

readonly output_limit=1048576
export LC_ALL=C

# Run a command with closed stdin and capture at most 1 MiB in the worktree scratch area.
# The output cap is deliberately strict: filling the capture is treated as a failure.
capture_capped() {
  local output_file=$1
  shift
  local command_status
  local head_status
  local output_size
  local -a statuses

  set +e
  "$@" </dev/null 2>&1 | head -c "$output_limit" >"$output_file"
  statuses=("${PIPESTATUS[@]}")
  set -e

  command_status=${statuses[0]}
  head_status=${statuses[1]}
  output_size=$(wc -c <"$output_file" | tr -d '[:space:]')
  if (( output_size >= output_limit )); then
    printf 'captured output reached the 1 MiB limit: %s\n' "$output_file" >&2
    return 1
  fi
  if (( command_status != 0 || head_status != 0 )); then
    printf 'command failed (status %s, capture status %s)\n' "$command_status" "$head_status" >&2
    return 1
  fi
}

format_millions() {
  awk -v count="$1" 'function commas(n, out) {
    out = ""
    while (length(n) > 3) {
      out = "," substr(n, length(n) - 2) out
      n = substr(n, 1, length(n) - 3)
    }
    return n out
  }
  BEGIN {
    value = sprintf("%.1f", count / 1000000)
    split(value, parts, ".")
    printf "%s.%sM", commas(parts[1]), parts[2]
  }'
}

measure_variant() {
  local benchmark=$1
  local rc_mode=$2
  local source=$bench_dir/$benchmark.braw
  local binary=$work_dir/$benchmark-$rc_mode
  local compile_log=$work_dir/$benchmark-$rc_mode-compile.log
  local run_log=$work_dir/$benchmark-$rc_mode-run.log
  local -a compile_args
  local minimum=
  local instructions
  local run
  local threaded_expected=
  local expected_output=$work_dir/$benchmark-expected.out
  local timing_log
  local count_log
  local mismatch_log=$work_dir/$benchmark-$rc_mode-mismatches.log

  if [[ $benchmark == thr_* ]]; then
    threaded_expected=$(awk -v name="$benchmark" '$1 == name { print $2 }' "$bench_dir/threaded-expected.txt")
    if [[ ! $threaded_expected =~ ^[0-9]+$ ]]; then
      printf 'missing or invalid threaded checksum for %s\n' "$benchmark" >&2
      return 1
    fi
    printf '%s\n' "$threaded_expected" >"$expected_output"
  fi

  if [[ ! -f $source ]]; then
    printf 'missing benchmark source: %s\n' "$source" >&2
    return 1
  fi

  compile_args=(build "$source" --output "$binary" -O 2)
  if [[ $rc_mode == on ]]; then
    if ! capture_capped "$compile_log" deltic timeout 120 env -u MDH_RC_NOFREE -u MDH_RC_REPORT MDH_RC=1 \
      "$compiler" "${compile_args[@]}"; then
      printf 'failed to compile %s with RC on\n' "$benchmark" >&2
      cat "$compile_log" >&2
      return 1
    fi
  else
    if ! capture_capped "$compile_log" deltic timeout 120 env -u MDH_RC_NOFREE -u MDH_RC_REPORT MDH_RC=0 \
      "$compiler" "${compile_args[@]}"; then
      printf 'failed to compile %s with RC off\n' "$benchmark" >&2
      cat "$compile_log" >&2
      return 1
    fi
  fi
  if [[ ! -x $binary ]]; then
    printf 'compiler did not produce an executable: %s\n' "$binary" >&2
    cat "$compile_log" >&2
    return 1
  fi

  for ((run = 1; run <= runs; run++)); do
    printf '  %s RC %s run %d/%d\n' "$benchmark" "$rc_mode" "$run" "$runs" >&2
    if [[ -n $threaded_expected ]]; then
      # Keep program output separate so the entire checksum can be compared exactly.
      run_log=$work_dir/$benchmark-$rc_mode-run-$run.log
      timing_log=$work_dir/$benchmark-$rc_mode-time-$run.log
      if ! capture_capped "$run_log" env -u MDH_RC -u MDH_RC_NOFREE -u MDH_RC_REPORT \
        deltic timeout 30 /usr/bin/time -l -o "$timing_log" "$binary"; then
        printf 'failed to run %s with RC %s (run %d)\n' "$benchmark" "$rc_mode" "$run" >&2
        cat "$run_log" >&2
        return 1
      fi
      count_log=$timing_log
      if ! cmp -s "$expected_output" "$run_log"; then
        printf '    OUTPUT MISMATCH: %s RC %s run %d; expected %s; actual output in %s\n' \
          "$benchmark" "$rc_mode" "$run" "$threaded_expected" "$run_log" | tee -a "$mismatch_log" >&2
        if [[ $rc_mode == on ]]; then
          return 1
        fi
      fi
    else
      if ! capture_capped "$run_log" env -u MDH_RC -u MDH_RC_NOFREE -u MDH_RC_REPORT \
        deltic timeout 30 /usr/bin/time -l "$binary"; then
        printf 'failed to run %s with RC %s (run %d)\n' "$benchmark" "$rc_mode" "$run" >&2
        sed -E '/^[[:space:]]*[0-9]+([.][0-9]+)? real([[:space:]]|$)/d' "$run_log" >&2
        return 1
      fi
      count_log=$run_log
    fi

    instructions=$(sed -nE 's/^[[:space:]]*([0-9][0-9,]*)[[:space:]]+instructions retired.*$/\1/p' "$count_log" | tr -d ',')
    if [[ ! $instructions =~ ^[0-9]+$ ]]; then
      printf 'could not parse the instructions retired count for %s RC %s run %d\n' \
        "$benchmark" "$rc_mode" "$run" >&2
      sed -E '/^[[:space:]]*[0-9]+([.][0-9]+)? real([[:space:]]|$)/d' "$count_log" >&2
      return 1
    fi
    printf '    instructions retired: %s\n' "$instructions" >&2
    if [[ -z $minimum ]] || (( instructions < minimum )); then
      minimum=$instructions
    fi
  done

  printf '%s\n' "$minimum"
}

baseline_check() {
  local benchmark=$1
  local actual=$2
  local expected=
  local difference

  case $benchmark in
    thr_*) return 0 ;; # Phase 6 has no section 0 baseline.
    closure) expected=69.3 ;;
    closure_fn) expected=40.1 ;;
    particle_sim) expected=15.4 ;;
    method) expected=12.5 ;;
    dict) expected=7.8 ;;
    *)
      # Section 0 gives the remaining six as a -9% to +6% range, not individual values.
      if awk -v value="$actual" 'BEGIN { exit !(value >= -10.0 && value <= 7.0) }'; then
        return 0
      fi
      printf 'baseline mismatch: %s RC cost %s%% is outside section 0 range -9%% to +6%% by more than 1pp; stopping\n' \
        "$benchmark" "$actual" >&2
      return 1
      ;;
  esac

  difference=$(awk -v actual="$actual" -v expected="$expected" 'BEGIN {
    delta = actual - expected
    if (delta < 0) delta = -delta
    printf "%.6f", delta
  }')
  if awk -v difference="$difference" 'BEGIN { exit !(difference <= 1.0) }'; then
    return 0
  fi
  printf 'baseline mismatch: %s RC cost %s%% differs from section 0 (%s%%) by %spp; stopping\n' \
    "$benchmark" "$actual" "$expected" "$difference" >&2
  return 1
}

printf '| Benchmark | Off | On | RC cost |\n'
printf '|---|---:|---:|---:|\n'

# Keep section 0's five individually tabulated benchmarks first so a mismatch stops promptly.
benchmarks=(closure closure_fn particle_sim method dict fib intloop alloc strbuild list expr_eval)
for source in "$bench_dir"/thr_*.braw; do
  [[ -f $source ]] || continue
  benchmark=${source##*/}
  benchmarks+=("${benchmark%.braw}")
done
for benchmark in "${benchmarks[@]}"; do
  printf 'Measuring %s\n' "$benchmark" >&2
  off=$(measure_variant "$benchmark" off)
  on=$(measure_variant "$benchmark" on)
  cost=$(awk -v off="$off" -v on="$on" 'BEGIN { printf "%.6f", (on - off) * 100 / off }')
  displayed_cost=$(awk -v cost="$cost" 'BEGIN { printf "%+.1f", cost }')
  off_m=$(format_millions "$off")
  on_m=$(format_millions "$on")

  row_name=$benchmark
  if [[ -s $work_dir/$benchmark-off-mismatches.log ]]; then
    row_name="$benchmark (RC-off output mismatch)"
    printf '  WARNING: %s RC cost uses an incorrect-output baseline; counts retained for lead review\n' "$benchmark" >&2
  fi
  printf '| %s | %s | %s | %s%% |\n' "$row_name" "$off_m" "$on_m" "$displayed_cost"
  if (( check_baseline )) && ! baseline_check "$benchmark" "$cost"; then
    exit 1
  fi
done
