# Ad Hoc Issue: Python Interop Read-Only Inspection Timeout

## Status

Complete (2026-08-10). The remediation merged through
[#3110](https://github.com/sifr-lang/sifr/pull/3110) at merge commit
`883477deabacd980097b41695d62065e3d3d17fe`. The final reviewed implementation
candidate was `6eb0729ce58b9054645cc4e05ca53e09ede73323`.

## Original Failure

The Phase 40 production-wiring create-PR profile passed 18 of 19 selected
Python interop variants. `readonly-check-doctor` timed out during the first
read-only library inspection at the then-current 120-second command guard. An
immediate isolated run reproduced the same failure. Phase 40 did not change a
timeout, add a waiver, or absorb the Python interop failure.

## Root Cause and Remediation

Read-only Python inspection resolved hundreds of declarations against the same
backend. Each declaration repeated the generated bridge namespace source scan.
Direct Rust probes also recomputed the SQLx offline metadata digest through
repeated workspace and package manifest traversal.

PR #3110 added resolver-scoped positive and negative caches for both immutable
inspections. The cache entries preserve the existing paths, bytes, trust
inputs, probe keys, and diagnostics. Embedded CPython now disables bytecode
writes so inspection cannot create `.pyc` files. Focused tests cover positive
and negative cache polarity, prove one inspection per resolver, and verify
`sys.dont_write_bytecode`.

## Acceptance Evidence

- [x] The isolated `readonly-check-doctor` suite passed in 92.436 seconds
  under the then-current 120-second command guard.
- [x] The authoritative create-PR Python lane passed `readonly-check-doctor`
  again in 56.532 seconds and passed all 19 selected variants in 387.206
  seconds under the then-current 600-second blocking step budget. It reported
  `mutations=0`; no timeout waiver or budget increase was used by #3110.
- [x] The final exact-candidate merge gate, `scripts/run_all_tests.sh`, exited
  0 on `6eb0729ce58b9054645cc4e05ca53e09ede73323`. It passed all 25 registered
  Python interop variants with zero failures and zero mutations, all 694 E2E
  fixtures, and 268 hardening variants with zero failures. The phase-specific
  `python_read_only_cli` integration suite passed all 6 tests. The SQLx
  positive, missing, and stale metadata generated-build cases also passed.
- [x] `cargo fmt --check`, HIR maintainability, transfer inventory, and
  file-size guardrails passed. The seven-file implementation diff contained
  165 insertions and 10 deletions; the largest touched first-party source file
  remained below 900 lines.
- [x] Milestone, full-implementation, integrated, final exact-candidate, and
  timeout-provenance Claude Opus reviews returned `SATISFIED` with no blocking
  findings. Final review evidence is keyed outside the reviewed Git tree by
  candidate SHA as required by the phase-closure workflow.

## Timeout Provenance

The 92.436-second isolated result and 56.532-second create-PR result are the
evidence for the former 120-second guard.

Before final integration, the independently owned performance work in
[#3101](https://github.com/sifr-lang/sifr/pull/3101) changed the runner guard to
300 seconds in commit `9eea091010d65cbd409c472b327543ac12773e25`. PR #3110
does not modify that runner, a validation profile, a timeout, or a budget.

The final merge gate therefore ran under the current 300-second per-command
guard. Its cold, contended `readonly-check-doctor` whole-suite aggregate was
460.150 seconds; this is not a 120-second result and is not comparable to the
earlier warm measurements. The full merge gate took 8,125.36 seconds, rebuilt
all 179 E2E groups, and reported the advisory `warm wall-time budget exceeded`.
The final reviewer classified the current-main guard as an externally owned
base change that does not invalidate #3110 or its preserved 120-second-tier
evidence.

## Deferred Work

No in-scope remediation remains. Cold aggregate budget policy and the
current-main command guard remain owned by the performance work merged through
PR #3101; #3110 neither changes nor waives them.
