# Native RC benchmark baseline

`run.sh` builds each `.braw` as a native executable with `-O 2`, once with `MDH_RC=0` and once with `MDH_RC=1`. Pass it the release compiler built with `--no-default-features --features cli,llvm,native`:

```sh
./benchmarks/rc/run.sh /path/to/mdhavers [runs-per-variant] [--check-baseline]
```

The run count defaults to 3 and cannot be lower. Each executable runs with stdin closed under `deltic timeout 30`; `/usr/bin/time -l` supplies the `instructions retired` count, and the minimum run is reported in the section 0 table format. Each run's instruction count is also printed to stderr as evidence. Other timing fields are ignored. Capture files are capped below 1 MiB. Logs and generated executables remain in the unique `.scratch/rc-bench.*` directory printed to stderr for the lead to clean up.

Pass `--check-baseline` to stop on the first mismatch against section 0. That checks the five individual RC costs within 1 percentage point; for the remaining six it applies the section 0 range of -9% to +6%, with the same 1-point allowance. Baseline checking is opt-in so later phases can use the same runner after improving RC costs.

The 11-source corpus is `closure`, `closure_fn`, `particle_sim`, `method`, `dict`, `fib`, `intloop`, `alloc`, `strbuild`, `list`, and `expr_eval`. `particle_sim.braw` and `expr_eval.braw` are byte-identical to their copies in `benchmarks/applications/large/`.

## Phase 6 threaded fixtures

The runner appends `thr_*.braw` after the original corpus. These rows use exact checksums from `threaded-expected.txt` and are excluded from the section 0 baseline check. The four-column table and original 11 benchmark rows are unchanged. A threaded RC-off checksum mismatch adds `(RC-off output mismatch)` to its row name, retains the instruction counts and computed percentage, and continues. That percentage uses an incorrect-output baseline and needs lead review. Each mismatching run is logged to stderr and a per-variant mismatch log; separate output and instruction logs are retained for every threaded run. RC-on mismatches, failed processes, and missing counters remain fatal.

| Fixture | Work | Expected checksum |
|---|---|---:|
| `thr_globals` | 200000 global string replacements, two readers with 200000 snapshots each | 2800000 |
| `thr_cell` | The same workload through a captured cell | 2800000 |
| `thr_containers` | 100000 list/dict replacements, two readers with 100000 reads of each container; every access uses one mutex | 2800000 |
| `thr_closure` | Two workers make one million total calls through a shared closure list | 250004000000 |
| `thr_chars` | Two workers index ASCII and multibyte Unicode characters one million times | 22769750000 |
| `thr_spawn` | 8192 short tasks, 128 integer-loop steps each, spawned and joined in batches of eight | 100134912 |

The intended results are independent of scheduling. Workers return nonnegative integers and every handle is joined. The interpreter only supports builtins in `thread_spawn`, so `launch` catches that specific limitation, evaluates the same kernel serially, and wraps its result in a builtin thread handle. Native builds run real worker threads. Unexpected errors are rethrown. Update `threaded-expected.txt` and recheck the interpreter if a work size changes.

Measured on 0.3.127 with the MDH-BUG-FLU-00216 fix, three runs each, instructions retired:

| Fixture | RC off | RC on | RC cost | RC-off output |
|---|---:|---:|---:|---|
| `thr_closure` | 157.7M | 334.1M | +111.8% | correct |
| `thr_globals` | 374.8M | 639.8M | +70.7% | correct |
| `thr_cell` | 372.6M | 618.8M | +66.1% | correct |
| `thr_containers` | 370.8M | 562.9M | +51.8% | correct |
| `thr_spawn` | 1,003.9M | 1,036.7M | +3.3% | correct |
| `thr_chars` | 622.1M | 490.0M | -21.2% | correct |

Both modes give the exact checksum on every fixture; RC on has zero live objects under `MDH_RC_REPORT` (with and without `MDH_RC_NOFREE`). In a program that spawns, RC off now reads and writes globals and captured variables under the same striped locks as RC on, without the counting. Against 0.3.125, when RC off still tore those strings, that raised the RC-off counts of `thr_globals` by 6.6%, `thr_cell` by 5.0% and `thr_closure` (which reads a global closure list) by 20%. Threaded programs get none of the RC7 borrowing (C(a) excludes programs that spawn), which is why their RC cost is far above the single-threaded corpus.

## Phase 0 measurement status

Attempted against pristine release compiler `HEAD 6aac928`, built with `--no-default-features --features cli,llvm,native`, using:

```sh
bash benchmarks/rc/run.sh .scratch/rc7-baseline-mdhavers --check-baseline \
  > .scratch/rc7/phase0-table.md 2> .scratch/rc7/phase0-runs.log
```

The first `closure` RC-off run stopped before producing an instruction count. The program printed `500004000000`, then `/usr/bin/time -l` exited with `time: sysctl kern.clockrate: Operation not permitted`. No baseline table or Phase 0 acceptance is claimed; the required instruction counter is blocked by the host sandbox.

The lead runs the following outside the sandbox, from the worktree root. The first command uses the preserved pristine compiler; the second uses the changed release compiler built with the same features. Only the baseline command checks the old costs, since Change A should improve them.

```sh
bash benchmarks/rc/run.sh .scratch/rc7-baseline-mdhavers 3 --check-baseline > .scratch/rc7/lead-baseline-table.md 2> .scratch/rc7/lead-baseline-runs.log
bash benchmarks/rc/run.sh target/release/mdhavers 3 > .scratch/rc7/lead-after-table.md 2> .scratch/rc7/lead-after-runs.log
```
