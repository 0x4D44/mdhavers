# Native RC benchmark baseline

`run.sh` builds each `.braw` as a native executable with `-O 2`, once with `MDH_RC` unset and once with `MDH_RC=1`. Pass it the release compiler built with `--no-default-features --features cli,llvm,native`:

```sh
./benchmarks/rc/run.sh /path/to/mdhavers [runs-per-variant] [--check-baseline]
```

The run count defaults to 3 and cannot be lower. Each executable runs with stdin closed under `deltic timeout 30`; `/usr/bin/time -l` supplies the `instructions retired` count, and the minimum run is reported in the section 0 table format. Each run's instruction count is also printed to stderr as evidence. Other timing fields are ignored. Capture files are capped below 1 MiB. Logs and generated executables remain in the unique `.scratch/rc-bench.*` directory printed to stderr for the lead to clean up.

Pass `--check-baseline` to stop on the first mismatch against section 0. That checks the five individual RC costs within 1 percentage point; for the remaining six it applies the section 0 range of -9% to +6%, with the same 1-point allowance. Baseline checking is opt-in so later phases can use the same runner after improving RC costs.

The 11-source corpus is `closure`, `closure_fn`, `particle_sim`, `method`, `dict`, `fib`, `intloop`, `alloc`, `strbuild`, `list`, and `expr_eval`. `particle_sim.braw` and `expr_eval.braw` are byte-identical to their copies in `benchmarks/applications/large/`.

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
