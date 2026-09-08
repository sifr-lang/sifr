# 12K-B40: six saved captures and proposed B24 delivery route

Final handoff only, 2026-09-08. No new execution, implementation, test, review,
gate or allocation follows this interpretation. B40 diagnostic apparatus and
its one allocated run are complete; B24 causal/production qualification and the
original emitted-Rust phase are not complete.

Source/prelaunch: `3e32b301fa3f1e4ba8eae0127415fe21ce094848`.
Evidence: `/private/tmp/sifr-b40.urwpgv/evidence/analysis.json`,
SHA256 `b3e44d9c1bb92b2c79ace683ed2008baf17eac4a518ca95af7bcf216f9942fac`.
The unchanged saved analyzer ran once. This interpretation uses that output,
not a new measurement or analyzer rerun.

## What the six saved captures actually show

These are six LLDB captures: P0 is the fixed warmup, P1-P5 the five observations.
All have20 events, complete acknowledged custody and canonical two-file output.
The following are cumulative counters at the measured completion endpoint,
not whole-process exit measurements. User/system values retain the recorded
SDK time units; they are not Python elapsed seconds.

| Capture/PID | Instructions | Cycles | User time (raw SDK) | System time (raw SDK) | Page-ins | Disk read bytes | Disk written bytes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P0 warmup/84536 | 74840858 | 48863467 | 84261 | 265010 | 5 | 16384 | 0 |
| P1/84969 | 47688199 | 24223689 | 88195 | 87805 | 0 | 16384 | 0 |
| P2/85183 | 47286671 | 25037791 | 86871 | 94871 | 0 | 0 | 0 |
| P3/85473 | 47713462 | 23871356 | 87785 | 88191 | 0 | 0 | 0 |
| P4/85744 | 46937835 | 24321242 | 88662 | 88470 | 0 | 0 | 0 |
| P5/85977 | 48230057 | 24595114 | 89994 | 94039 | 0 | 16384 | 0 |

P0's instruction and cycle totals are much higher, accompanied by higher
system-time accounting, not higher recorded user time. Extra instruction work
is spread over fixups, loader-other, CLI parsing, config, discovery and first
file check. It is not isolated to the loader. This is an observation about this
instrumented sequence, not proof of a cold-cache, OS/security or debugger cause.

Instruction partitions distinguish warmup from the five observations:

| Non-overlapping partition | P0 | P1-P5 minimum | P1-P5 maximum |
| --- | --- | --- | --- |
| fixups | 19358475 | 15123070 | 15297786 |
| loader_other | 6641684 | 5541836 | 5622435 |
| library_initialization | 3809225 | 3643551 | 3778023 |
| cli_parse | 11656457 | 5518337 | 5698899 |
| config | 6994555 | 1642141 | 1910209 |
| discovery | 11592486 | 6467867 | 6704442 |
| file_check_1 | 7174040 | 1981696 | 2004751 |
| file_check_2 | 1967992 | 1785075 | 1803493 |
| completion_residual | 423711 | 250778 | 602397 |

Among P1-P5, config spans268068 instructions and discovery236575, while the
two file-check spans are23055 and18418. This makes config/discovery and residual
bookkeeping useful places to discriminate a hypothesis before proposing a
parse/format-core optimization. It does not establish that their variation
caused the original uninstrumented formatter budget failure. Loader/fixup and
other intervals still vary; interval extrema belong to different processes
and must not be added as a total variance explanation.

P0's five page-ins fall in the saved library-initialization partition; P1-P5
have zero measured page-ins. The16384-byte disk-read measurements occur in
P0/P1/P5, attributed only to main-to-completion, not a particular file or cause.
Zero page-ins or disk-byte readings do not prove zero filesystem activity.
All seven metrics remain available rather than replacing the original metric
with the most favorable one.

## Limits and unsupported claims

Intervals are process-wide, including other threads; stopping and observation
can perturb execution. There is no debugger-overhead subtraction, matched
uninstrumented intervention, randomization or causal control in these captures.
Instruction stability is not elapsed-time stability or the original budget's CV.
P0 is not a sixth production observation and its excess is not a failed budget.
The pre-first-stop prefix is an already-cumulative counter, not an observed
interval. The completion-to-exit tail is unmeasured. Zero
remaining_preconstructor is accounting-union coverage, not proof that all
underlying work is understood. Inclusive loader/fixup/library details overlap
and must not be summed; use the explicit partition instead. Different clock
domains remain separate.

The binary is immutable experiment09 with its diagnostic patch, not current
main or B22's production candidate. No main/B22 acceptance is inferred. No
historical setsid, new OS/API/security, loader-exclusive, cache-prewarming or
linker-option remedy is supported by this run. No new mechanism defect was
established. F1's special initial-group contradiction/corroboration route did
not occur in any of228 accepted live custody observations; its support remains
the frozen negative and expanded coherent-positive offline tests.

## Proposed next route: causation, then production, then joint qualification

This is a proposal for coordinator/owner3776 adjudication, not a started item.

1. **Saved-evidence causal reconciliation first.** Crosswalk the retained
   B19/B20 failing uninstrumented samples and their exact measured metric,
   warmup policy, source/binary/workload and invocation identities against
   experiment09 and these B40 phase boundaries. The immediate question is
   whether the residual original failure tracks configuration/discovery or
   other startup/accounting, rather than assuming that the warmup's broad
   excess explains it. Do not compare different-source totals as a controlled
   treatment. This crosswalk is proposed; B40 has not performed or certified it.
2. **One specific, falsifiable mechanism before a production patch.** The
   leading Sifr-owned candidate boundary to investigate is repeated
   configuration/discovery work: it varies more than the steady file-check
   intervals here and is directly relevant to the retained B20/B22 history.
   If the saved/source crosswalk identifies a concrete redundant operation,
   register a one-variable contrast removing only that operation while
   preserving exact selected paths/order, config and ignore behavior, errors,
   output and workload. Predict the affected phase and original end-to-end
   metric, keep the contrary startup/accounting explanation, and set a
   bounded stop criterion before acquisition. If no such operation is
   established, no production correction is justified; the next registered
   causal discrimination must answer the remaining specific question.
   Do not substitute another unchanged LLDB run or speculative compiler,
   SDK, OS/API, security, priority, linker or cache-prewarming experiment.
   No new external quiet-window prerequisite is inferred from B40: actual
   local admission and continuous monitoring passed.
3. **Only a supported correction becomes a production candidate.** Freeze its
   exact manifest/SHA, semantic evidence and source-to-binary provenance in
   that separately scoped B24 work. Use the qualified B23 lifecycle; if B20's
   discovery is retained, require the necessary qualified B22 correction.
   Otherwise exclude B20 and leave B22 separately open. Do not import the
   B20/B22 stack wholesale or reset exhausted review/gate allowances.
4. **B24's original acceptance is unchanged.** After a causally supported,
   fully implemented correction, both
   `python3 verification/areas/performance/run_benchmarks.py --self-test`
   and `python3 verification/areas/performance/check_budgets.py --self-test`
   remain required, plus
   `uv run --project verification --locked python -m sifr_verify areas run --area performance --suite representative`.
   This means the original full10-case selection including the unchanged
   formatter, producer followed by dependent budget in the SAME invocation
   ID, original thresholds/baselines/workload and all failures retained.
   A benchmark-subset-only call, six diagnostic captures, lucky stable sample,
   filtered case set, warmed retry or threshold change is not acceptance.
5. **Then B26's complete B27 delivery obligations still apply.** A terminal
   causally qualified B24 delta is a new exact input, not this diagnostic SHA.
   Reconcile fresh main with the preserved fork156157-to98480 compiler delta,
   H/I/M1/12B contracts, B25 assertions and exact65 base6bd085-to4c7068 policy
   delta/28 paths and owner-confirmed consumer contracts. Do not require65's
   main delivery first or copy an old whole tree over newer main.
   Retain all264 companion pairs, the90-source native check/run contract and
   separate411 check-only corpus, all30 Python variants/five suite identities,
   source-attributed12-surface/14-file debt proof, SQL, all20 merge-profile
   areas/13 guards/production92/all31 full-mode crate commands/both toolchains
   and the rest of the B26 input inventory. Actual current selection governs.
   Preserve residual Item12 exclusion and its open all-surface audit.
   Historical partial gates/reviews are not a complete joint pass. The later
   genuinely changed integration scope needs its own explicitly adjudicated
   allowance; B40 supplies no review, gate or merge authority.

Binding sources are the original full B24 registration at
`/private/tmp/sifr-b24.4MtvfE/sifr/plans/issues/active/ad-hoc-emitted-rust-b24-experiment-registration.md`
and the B26 assessment/input inventory beside it:
`ad-hoc-joint-emitted-rust-delivery-assessment.md` and
`ad-hoc-joint-emitted-rust-delivery-inputs.md`. This summary does not narrow them.

## Terminal boundary

B40: coverage540 PASS, observer192 PASS, unchanged analyzer84 PASS reused.
ONE live setup/LLDB, six targets,106.315466seconds total; root exit0,
342subjects/probes and all14groups proven absent within540, joins recorded.
Resources released; diagnostic-stage blocker NONE. No Opus, Sifr gate, PR,
merge, production acceptance, next-item work or further allocation.
The parent/coordinator receive this record and the terminal hash; this worker
stops after publication.
