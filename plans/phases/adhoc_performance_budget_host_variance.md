# Ad Hoc Phase: Representative Performance Budget Stability

Status: deferred follow-up; not a prerequisite for Phase 40.

## Problem

The representative performance budget is not stable enough on the current
macOS host to distinguish a product regression from host variance. All
benchmark commands complete successfully, but repeated five-sample medians
cross fixed thresholds by small and inconsistent amounts.

This follow-up must not change performance baselines or add waivers merely to
make one host pass.

The same host also exceeded the create-PR lane's aggregate Python-interop step
budget despite every selected case passing. That timing variance belongs to
this follow-up rather than to stable-release governance.

## Evidence

On merged main commit `56f8c41eec`:

- the full merge lane completed every preceding area, then reported medians
  0.5–2.0% above threshold for three check/diagnostic benchmarks;
- an isolated representative retry cleared one prior benchmark, retained two
  small regressions, and introduced a bimodal LSP microbenchmark result;
- the raw LSP samples split between roughly 5 ms and 12 ms.

On immediate parent `082988df1f`, using the same worktree and cache:

- the same representative suite also failed;
- it failed a different two-benchmark set, including a larger diagnostic
  regression.

The Phase 40 milestone changed only stable-release governance scripts and the
read-only self-update receipt channel in compiled sources. The immediate parent
comparison demonstrates that this budget failure is not introduced by the
milestone.

During the stable incident-governance work, the create-PR profile completed all
19 Python-interop variants with zero failures, but the aggregate step took
788.45 seconds against its 600-second budget. The slowest individual cases were
callback examples (161.70 seconds), CPython buffer compatibility (142.55
seconds), read-only check/doctor (126.69 seconds), and buffer examples (113.33
seconds). The lane exited only on the elapsed-time budget after reporting every
case as passing; the incident-governance diff does not change Python-interop or
compiler implementation files.

## Scope

- Reproduce representative measurements across controlled warm and cold runs.
- Record host thermal state, load, CPU frequency behavior, and cache state.
- Determine why command medians and LSP samples are bimodal.
- Determine why cold Python-interop fixtures can exceed the aggregate
  create-PR step budget while all functional variants pass.
- Make sampling, warm-up, isolation, or threshold derivation robust enough that
  the merge gate is repeatable without hiding real regressions.
- Add a deterministic self-test for the chosen stability rule.

## Definition of Done

- The representative suite produces a repeatable verdict across at least five
  consecutive controlled runs on a supported host.
- The same policy still rejects a seeded regression above the governed budget.
- No product milestone needs a waiver for unrelated host variance.
- The merge profile and performance documentation describe the controlled
  measurement conditions.
