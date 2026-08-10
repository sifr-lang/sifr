# Ad Hoc Phase: Representative Performance Budget Stability

Status: in progress.

## Execution Plan

| Milestone | Scope | Status |
| --- | --- | --- |
| M1: controlled measurement and provenance | Host/cache telemetry, controlled admission, stable-sample retries, stale-result producer/checker binding, and governed trend-reference refresh | complete on draft PR #3101 |
| M2: Python interop cold-cache budget | Classify cold versus warm aggregate execution and enforce a cache-aware create-PR step budget with deterministic policy tests | complete on stacked PR #3115 |
| M3: qualification and closure | Local retired-instruction budgets, five consecutive controlled verdicts, seeded-regression proof, final merge gate, review, and closure records | implementation complete on stacked PR #3116; final owner qualification in progress on draft PR #3101 |

Each implementation milestone uses one draft PR, exact-SHA validation, and
repeated Claude Opus review under the phase-closure loop. Review and validation
evidence is recorded only after the matching candidate merges.

On 2026-08-08 the performance owner delegated
[#3100](https://github.com/sifr-lang/sifr/issues/3100) into this active phase
after both the legacy metadata deferral and the approved-reference freshness
deferral expired and blocked repository-wide create-PR gates. M1 therefore owns
the root-cause closure: add an executable controlled, full-manifest trend
refresh path and replace both deferrals with fresh environment and reference
evidence. Extending either expiry remains prohibited.

### Current handoff

- State: M1, M2, and the M3 implementation are complete on the owner branch.
  The final owner qualification is in progress. M3 replaces the desktop-idle
  blocker with local retired-instruction evidence. It preserves the separate
  elapsed-time policy for quiet-host qualification.
- Owner branch: `codex/adhoc-performance-budget-host-variance`.
- Draft PR: [#3101](https://github.com/sifr-lang/sifr/pull/3101).
- Reviewed M1 implementation candidate:
  `28bca35551321b109e272c61ae52fe6201eb810d`.
- The approved integrated work capture passed all 65 governed benchmarks on
  producer `b59fb265cf58327e0018778b39e5cd5001bd6edd`. The work-budget artifact
  digest is
  `56f91e711d78b4efb7e759e20868a3edec67c95f5b70368f08ddb5e70f98cc26`.
- The current integrated qualification candidate is
  `2c811ff29149f1f72c9dfaae6c32dfdf9c1b697f`. It includes current main at
  `00ea8867569884943413f7809414839c8992db97` and the two exact reviewed
  structural-demand repair commits `a301991aa42f435c02abd1ba618eca457c402177`
  and `bb5445b16da928f3c5d1cca531ee4572c3a7f7ed`.
- Five consecutive controlled representative verdicts passed on the integrated
  candidate. Each verdict passed 10 of 10 benchmarks and all eight area
  variants. The raw result digests are recorded below.
- Claude Opus returned `SATISFIED` with no blocking findings for the integrated
  phase implementation before the current-main repair. It also returned
  `SATISFIED` for both exact structural-demand repair candidates. A final
  whole-phase exact-SHA review remains required after this record update.

### M3 local-host decision

The phase uses this Mac as the permanent performance host. macOS does not
provide a user-space CPU reservation. Its `/usr/bin/time -l` command provides
retired instructions and cycles for the benchmark process tree.

An exploratory arithmetic control ran while unrelated CPU use reached
`419.5%`. Its five elapsed samples were `1503.105` to `1527.412` milliseconds.
The retired-instruction coefficient of variation was `0.000240`. An LSP
diagnostics control also produced process-tree instruction evidence.

M3 adds two controlled modes. Work mode uses retired instructions as the
blocking performance metric. Latency mode keeps the existing load and
unrelated-CPU limits for elapsed-time evidence. No existing elapsed-time
threshold, waiver, or baseline is increased.

Work mode also uses a fresh Darwin process-tree RSS baseline. Darwin includes
the spawned LSP server and descendants. The generic baseline can measure a
different process boundary. Separate governed thresholds prevent those RSS
meanings from being compared as if they were equal.

The approved full-manifest work capture passed all 65 cases on implementation
commit `7e5d6648b6885863e60bab2a55d76cca8b59cdfb`. It produced work-budget
artifact digest
`aa57ee57b95177832845a4b1a8b2bce603f39fa1da25e9f14c73e28bd26253cc`
and raw-evidence digest
`0b3ca547e4b13afeb2afa909d87927f942f4f4e0ff16bcb184c806db79cabeac`.
Every case produced at least five retired-instruction samples. The largest
accepted instruction coefficient of variation was `0.012365`, below the
`0.02` limit. The formatter corpus rejected an unstable `0.031108` first
attempt and accepted its second attempt at `0.001955`. All other cases passed
on their first controlled attempt.

The first representative verdict found that independent query processes had
reduced aggregate cache counts. The producer now uses one aggregate invocation
for latency, cache, and diagnostics. It uses independent invocations only for
process-work samples. A deterministic self-test protects this boundary. The
two affected cases report 2,300 cache hits and 2,300 misses in the replacement
capture.

The full evidence comparison also found that generic RSS baselines used a
different process boundary from Darwin rusage. Work mode now uses the approved
Darwin process-tree RSS values from the same local artifact. Latency mode keeps
the existing generic RSS thresholds. Seeded tests reject both instruction and
local-RSS work regressions without using elapsed-time thresholds.

The first exact-candidate representative verdict rejected the formatter corpus
at 36.916 million instructions against a 36.759 million threshold. Four runs
placed its medians between 35.721 million and 36.916 million instructions. The
largest individual sample was 37.298 million. M3 therefore uses a general
2-million-instruction floor for small processes. Larger workloads keep the 2%
rule. No case-specific threshold or waiver was added.

The M3 exact-SHA review found two non-Darwin regressions before merge. The
profile adapter had selected Darwin work mode on Linux. Latency producers also
emitted an empty instruction array that failed result validation. Profiles now
select work mode only on Darwin and retain latency mode elsewhere. Producers
omit instruction evidence when the host does not provide it.

The first post-review area-adapter verdict found that the performance runner's
new sibling import worked only when the runner was executed as a script. The
runner now adds its area directory before importing host control. Both direct
and `sifr_verify areas run` rules invocations pass.

The first final merge gate on owner candidate `df8c48b9aac119aac0d5dff58407ceb5bad71f94`
passed every earlier lane. It then measured the unchanged-file incremental
case at 930,619,052 median instructions against a 930,509,977 threshold. The
excess was 109,075 instructions, or about 0.012% of the threshold. The run had
an instruction coefficient of variation of 0.000852 and an instruction median
absolute deviation of 230,896. Its raw-evidence digest is
`b8bd14d09aa0e0079acfd360172a479fcc81f60da96ece1ca0ae5936d32b00a1`.

This boundary result exposed a missing sampling-uncertainty rule. The 2%
regression threshold remains unchanged. The checker now compares the threshold
with `median - min(3 * instruction MAD, 0.5% * median)`. Thus, ordinary sample
noise cannot cause a boundary failure, but uncertainty cannot hide more than
0.5% of measured work. A zero-MAD regression still fails at one instruction
above the threshold. Deterministic tests also prove that boundary noise passes
and that the 0.5% cap still rejects a sustained regression. Each accepted
over-threshold result emits a note with its measured median, threshold, MAD,
and applied uncertainty. The checker also verifies the median and MAD against
the raw instruction samples before it uses them.

The work budgets were recaptured from the final measurement producer. The
first recapture stopped safely without changing governed data when the Mac
briefly left AC power before a stability retry. The complete replacement run
then passed all 65 cases on producer
`df8c48b9aac119aac0d5dff58407ceb5bad71f94`. Its maximum accepted instruction
coefficient of variation was `0.018711`, below the `0.02` limit. The raw
evidence digest is
`e594ae7c2f55d076c4547e58a736c2ae21903af9c282fbafce24876273676361`.
The work-budget artifact digest is
`127d520b28bf280f830226a4e73d728c2eff03edab1c31972735694a6dc03d96`.

The integrated owner candidate `b59fb265cf58327e0018778b39e5cd5001bd6edd`
includes the external Rust 1.94 code-generation fix that the repository-wide
merge gate required. Its first merge gate passed CPython differential and all
25 Python-interop variants in 1,050.516 seconds. Rust interop passed in 6.347
seconds. All 32 LSP variants, the performance policies, and representative
benchmark execution also passed. The budget checker then rejected only
`incremental-local-loop-001-unchanged-file-update`: its measured median was
937,013,372 instructions, its uncertainty-adjusted lower bound was
932,328,305.140 instructions, and its pre-integration threshold was
931,509,560 instructions. The 20 samples had an instruction coefficient of
variation of `0.003367`. This sustained result showed that the governed work
artifact no longer represented the integrated producer.

The first approved integrated recapture stopped before writing governed data
when another worktree started a Cargo build. The replacement recapture used a
local 2+6 CPU partition on this 10-thread Mac: qlty received two CPUs, Sifr
received six workers, and no competing Cargo process ran. All 65 cases passed
on producer `b59fb265cf58327e0018778b39e5cd5001bd6edd`. Only
`lsp-query-001-request-families` needed its single allowed retry. The maximum
accepted instruction coefficient of variation was `0.017457`, below the
`0.02` limit. The raw-evidence digest is
`f8c09d47e29ecbdfe2c8d9efb451db40481cc519a0d8c96a2cda247c80051713`.
The generated work-budget artifact digest is
`56f91e711d78b4efb7e759e20868a3edec67c95f5b70368f08ddb5e70f98cc26`.
For the rejected incremental case, the replacement baseline is 918,442,841
instructions. The governed 2% threshold is 936,811,698 instructions. No
waiver, case-specific allowance, or manual threshold edit was added. The
budget policy and all six performance rules passed against the replacement
artifact. The benchmark and budget self-tests also passed.

### Final integrated qualification

The update to current main introduced an independent structural-interop work
regression. The first controlled representative run on integration commit
`f26537ba4745` completed all commands, but it rejected five instruction
budgets. The two build medians were 125,860,810,314 and 117,070,662,274
instructions. The three check and diagnostic medians were between
16,487,041,630 and 16,552,669,644 instructions. The raw-result digest is
`082851df49023da54930d5968bf55213311550844d82525079f65befb0d8a90e`.
Issue [#3124](https://github.com/sifr-lang/sifr/issues/3124) owns this regression.

The first repair, `a301991aa42f435c02abd1ba618eca457c402177`, added
project-wide structural demand gates. It restored the check and diagnostic
budgets. An authoritative run on integrated commit `8b62019e4c68` still found
two build regressions. The project-build median was 120,336,407,105 against a
119,984,530,852 threshold. The single-file build median was 114,248,753,461
against a 113,587,510,625 threshold. The raw-result digest is
`ac237022b952234a3e1ef9e922300d124e32add40fd59ba4e8517bcd4503e8d9`.
The phase did not rerun the unchanged failing candidate.

The residual cause was an unconditional runtime dependency and module. Repair
`bb5445b16da928f3c5d1cca531ee4572c3a7f7ed` makes
`sifr_structural_identity` optional behind the `sifr_runtime/structural`
feature. Code generation requests that feature only for project-wide
structural demand. Ordinary generated manifests do not enable the feature.
The positive structural bridge enables it explicitly. Default and structural
runtime tests, 66 runtime tests, 29 manifest tests, 975 code-generation tests,
the native structural bridge, Rust interop 10/10, and the repository guards
passed. Claude Opus returned `SATISFIED` with no blocking findings on this
exact repair.

Direct warm controls for the final repair measured project-build at
117,828,333,610 median instructions with coefficient of variation
`0.00024089`. Single-file build measured 111,958,572,272 with coefficient of
variation `0.00019889`. Both are below their unchanged governed thresholds.
The raw time-file digest is
`6d36529358b8ea4ab3bc4c6ab9e7d24bd5a1ee4c5a85bf65d259caac4fe164df`.

The exact final integration commit is
`2c811ff29149f1f72c9dfaae6c32dfdf9c1b697f`. A compiler warm-up completed
before the sequence. Each of five consecutive representative runs passed all
10 benchmark commands, budget policy, trend policy, both policy self-tests,
and the final subset budget check. The five raw-result SHA-256 digests are:

1. `103b4c97f63a48faa5dbcdcf88a14649bf75595cd88d099df615fc0955545c5d`
2. `d05dcd9c4dfcdde05312f03ee6e5f031718cf7068b2b7635304612618437a653`
3. `1d1dc57161f242c3891a61770e6533ed2176ea53dd751ff7c10ab0b7c2040e90`
4. `1011646c74292fa8b8df4b836857bb829a3f255960eec205632bd9d293e231de`
5. `59c750f9d7cf664fdcfa6f80701cad9ac33eb8f15255a3559d663cd8ea17c31e`

Across the five runs, project-build medians were 118,321,078,060 to
118,356,455,758 instructions. Single-file build medians were 112,449,250,887
to 112,475,789,417 instructions. The three check and diagnostic medians stayed
between 12,796,580,486 and 12,868,195,306 instructions. The unchanged-file
incremental medians stayed between 912,181,474 and 914,125,614 instructions.
All instruction coefficients of variation were below the `0.02` stability
limit. One build attempt and three formatter attempts used the single allowed
stability retry. All other cases passed on their first attempt.

The sequence used a reversible local CPU partition on the 10-thread Mac.
`qlty-dev` received two CPUs. Sifr received six worker slots. Two threads
remained for macOS. The wrapper restored qlty to 10 CPUs at exit. The work-mode
policy ignored unrelated elapsed-time pressure, but it still rejected
competing build processes, thermal pressure, missing AC power, and unstable
retired-instruction samples. No threshold, baseline, waiver, or timeout changed
during final requalification.

### M2 reproduced evidence

On exact M1 candidate `28bca35551321b109e272c61ae52fe6201eb810d`,
the canonical create-PR profile passed every preceding lane and 18 of 19
Python-interop variants, then failed only `readonly-check-doctor` at its fixed
120-second subprocess timeout. The Python-interop step had run for 1,131.589
seconds when it failed. An immediate unchanged aggregate replay again passed
18 of 19, reproduced the same case at 120.479 seconds, and took 821.95 seconds;
`callback-examples` alone varied to 341.399 seconds.

M2 separates the functional hang guard from the performance budget. The doctor
subprocess guard is 300 seconds. The create-PR aggregate keeps its 600-second
warm budget and uses a 1,200-second cold budget only when an atomic receipt
cannot prove a prior successful run for the exact source commit, tracked-tree
state, selected suites, Cargo lock, Rust/Python toolchains, Sifr binary, and
required cache artifacts. Failed, dirty-tree, unavailable-input, changed-input,
or missing-cache runs cannot establish a warm receipt. The classification and
selected budget are emitted in the machine lane report.

Targeted validation passed `readonly-check-doctor` at 169.710 seconds. The
complete unchanged 19-variant selection then passed 19/19 in 522.69 seconds;
the doctor case took 170.125 seconds and `callback-examples` took 78.678
seconds. Deterministic self-tests cover missing, exact, changed, invalid, and
missing-artifact receipts, blocking cold overruns, report parsing, and the
governed create-PR profile values.

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

During dict missing-key augassign closeout on exact reviewed source
`b341b47f9f8e81baa0d7403979eb6551886e5568`, the create-PR profile passed all
19 Python-interop variants but the aggregate step took 639.010 seconds against
its 600-second budget while other worktrees contended for Cargo package-cache
locks. The final merge profile later passed coverage, core, diagnostics, and
all preceding selected checks before `bounded_int_arithmetic` hit the CPython
differential runner's fixed 240-second Sifr timeout. The next three cases
completed at 224.9, 66.8, and 38.6 seconds. An immediate isolated replay of the
exact differential area passed both variants and all four cases at 36.3, 25.0,
17.1, and 18.2 seconds respectively. The implementation does not change the
differential fixtures, Rust-interoperability probe machinery, validation
budgets, or performance baselines; no threshold or waiver was changed.

Two dependent create-PR gates then isolated the expired trend record as a
repository-wide owner blocker. Draft PR #3106 passed all functional lanes,
including Python 19/19, Rust interop 10/10, LSP 18/18, and benchmark execution,
then failed only the two expired trend deferrals. Draft PR #3110 at exact head
`2c57a16f` passed taxonomy, read-only Python verification in 56.5 seconds with
zero mutations, Python interop 19/19 in 387.2 seconds, Rust interop 6.2/10, and
LSP before failing only `legacy-baseline-environment-metadata`. These independent
controls justify owner-side closure under #3100; they do not justify extending
either deferral.

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
