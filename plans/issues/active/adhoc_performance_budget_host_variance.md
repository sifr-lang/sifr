# Ad Hoc Phase: Representative Performance Budget Stability

Status: deferred follow-up; not a prerequisite for Phase 40.

## Execution Plan

| Milestone | Scope | Status |
| --- | --- | --- |
| M1: controlled measurement and provenance | Host/cache telemetry, controlled admission, stable-sample retries, and stale-result producer/checker binding | in progress |
| M2: Python interop cold-cache budget | Classify cold versus warm aggregate execution and enforce a cache-aware create-PR step budget with deterministic policy tests | pending |
| M3: qualification and closure | Five consecutive controlled representative verdicts, seeded-regression proof, final merge gate, full-phase review, and closure records | pending |

Each implementation milestone uses one draft PR, exact-SHA validation, and
repeated Claude Opus review under the phase-closure loop. Review and validation
evidence is recorded only after the matching candidate merges.

Out-of-scope validation failure: the pre-existing
`legacy-baseline-environment-metadata` trend deferral expired on 2026-07-31 and
now fails the performance `rules` suite. It is recorded under
[#3100](https://github.com/sifr-lang/sifr/issues/3100); this phase does not
extend the deferral or change the checked-in trend baseline to make the gate
pass.

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

During the schema-v2 preview epoch bootstrap wave, the final create-PR profile
again completed all 19 Python-interop variants with zero failures, but the
aggregate step took 690.10 seconds against the same 600-second budget. The
slowest cases were callback examples (145.16 seconds), read-only check/doctor
(121.95 seconds), CPython buffer compatibility (106.41 seconds), buffer
examples (102.67 seconds), and DLPack examples (80.62 seconds); Cargo also
reported package-cache file-lock waits. Two preceding runs of the same wave
completed that functional step within budget at 456.79 and 455.79 seconds.
The bootstrap diff does not change Python-interop or compiler implementation
files, and no threshold, baseline, or waiver was changed.

During stable documentation/editor qualification on source head `147296fb0`,
the create-PR profile passed coverage, core, diagnostics, and 18 of 19 selected
Python-interop variants. `readonly-check-doctor` alone exceeded its internal
120-second timeout while running `sifr python check --json`; an immediate
isolated retry reproduced the timeout at 124.62 seconds. All later selected
Python-interop cases in the full lane passed, including binding authoring,
callbacks, buffer, Arrow, DLPack, async, and CPython 3.11 compatibility. The
lane took 855.07 seconds with no swaps. The qualifying change does not modify
Python-interop implementation or verification, so this remains follow-up
evidence rather than a stable-release-governance prerequisite or a reason to
weaken the timeout.

During the Phase 40 single-maintainer approval-boundary review on 2026-07-29,
the unchanged Python-interop lane again reproduced the
`readonly-check-doctor` 120-second subprocess timeout. An exact isolated replay
failed at the same command and boundary. The subsequent authoritative
create-PR profile passed every preceding blocking lane, then reproduced that
timeout and a separate 180-second `binding-authoring` runtime timeout while
other worktrees were compiling concurrently; the following declaration,
tier-1, and callback cases passed immediately. The approval-boundary diff does
not modify compiler or Python-interop sources. No timeout, threshold, waiver,
baseline, or profile selection was changed.

During final candidate qualification on exact source
`7242e4737b1ee89f9f02a3b4793d5cdb13d372ea`, the canonical release profile
passed coverage, core, diagnostics, CPython differential, all 25 selected
Python-interop variants, the consumed Rust-interop gate, frontend guardrails,
all 48 developer-tooling variants, and documentation before three
check/diagnostic medians exceeded their budgets. An immediate standalone full
performance retry failed on a different four-benchmark, five-metric set.

A same-host control at the previously passing source
`c17f3c7d1ea1ed97ca125eb7a43344b30cf9413b` then timed out after 120 seconds
on its first benchmark build even though that source had previously passed the
complete full suite. After removing disposable build cache from a separate
completed worktree, the unchanged full performance suite at `7242e4737b1e`
passed all eight variants. Its key check/diagnostic samples clustered around
1.27–1.31 seconds, while the pressure-affected runs spiked as high as 3.88
seconds. A later end-to-end report attempt again saw mid-run spikes despite the
preceding standalone pass. No threshold, baseline, waiver, source file, or
profile selection changed across these observations. The exact commands,
result paths, digests, and representative measurements are archived in
`plans/reviews/archive/phase-40-milestone-40-4-exact-source-evidence.md`.

After the unrelated competing reviewer task was stopped and archived, another
unchanged canonical release-profile invocation passed all eight full
performance variants. That invocation continued through all 56
distribution-release variants and failed later only when installed sysroot
self-update encountered the live schema-v1 preview index. This independently
separates `PERF-HOST` from the Phase 40 schema-epoch bootstrap and confirms that
the performance lane can pass without a release waiver.

During Rust-interop Track A closeout on exact source
`017c1df411f78ffb786775fdf4bd60e52424f839`, every representative benchmark
command completed, but the merge-profile comparison rejected project-graph,
arithmetic, JSON-diagnostic, and LSP diagnostic samples at
1358.717/1357.524 ms, 1366.015/1334.139 ms, 1354.814/1335.954 ms, and
5.962/5.91 ms median plus 11.664/10.933 ms p95 respectively. The closeout diff
does not change compiler, frontend, diagnostic, or LSP implementation paths.
Within minutes, an unrelated class-field branch on the same host failed the
same four cases much more severely at 3313.437 ms, 4612.439 ms, 4132.029 ms,
and 17.918 ms median plus 22.939 ms p95. The cross-branch control and earlier
closeout retries that moved between affected cases demonstrate the same
bimodal shared-host condition; the closeout changed no baseline, threshold,
waiver, or profile selection.

The timed-out control also showed that `full/budget-subset` can read the prior
`full.budget.latest.json` after `full/benchmark-subset` fails before replacing
it. The lane remains failed, so this does not create a false pass, but the stale
secondary diagnostics should be corrected by this follow-up.

## Scope

- Reproduce representative measurements across controlled warm and cold runs.
- Record host thermal state, load, CPU frequency behavior, and cache state.
- Determine why command medians and LSP samples are bimodal.
- Prevent a failed benchmark producer from feeding stale prior-run results to
  the same invocation's budget-subset diagnostic.
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
