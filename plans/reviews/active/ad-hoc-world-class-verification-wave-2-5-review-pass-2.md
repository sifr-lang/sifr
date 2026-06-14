I have enough context to render the review. The nit-1 fix replaced the redundant substring assert with a tail-expression assert; nit-2 docs reflect the cache shift; inventory and docs are consistent.

# Wave 2.5 Codegen Red-Blocker Review — Pass 2

**Verdict:** approved, ready to merge

## Blocking findings
None.

## Pass-1 nit follow-up

1. **Tail-expression assertion (nit 1) — resolved.** `crates/sifr_codegen/src/lib_codegen_tests/structured_lowering_codegen_tests.rs:523` now asserts `";\n    __indexed_char\n}"`. This proves `__indexed_char` is used as the tail expression after the let-else guard, not just that the binding name appears (which the let-else pattern at line 522 already covered). The string-index safety contract — let-else, no `.unwrap()`, no `nth().unwrap()` panic path — is now load-bearing across three assertions: cache shape (line 519), guard shape (line 522), and tail use (line 523). No weakening; this is strictly stronger than the pre-Wave 2.5 substring check it replaced.

2. **Honest codegen-shift documentation (nit 2) — resolved.** Both `plans/issues/active/codegen-test-triage.md:22` and `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:408` now explicitly call out the shift from `chars().nth(...)` to a `Vec<char>` cache plus let-else guard. The "only the assertions were stale" framing is no longer overclaimed.

3. **Unused-mut (nit 3) — correctly deferred.** Asserting `let mut` in the test bakes in a codegen lint smell, but the test must match what codegen emits. Filing as future cleanup against the structured-return char-cache emitter is the right call for Wave 2.5 scope.

## Non-blocking observations
None new for this pass.

## Verification notes

- **Q1 — pass-1 nits addressed without weakening the contract.** Yes. The substring check was replaced with a tail-expression check that is *more* discriminating: it would fail if codegen regressed to emitting `__indexed_char.to_string()` or dropping the let-else binding from the tail position. Plus the let-else pattern and char-cache shape asserts remain.

- **Q2 — final diff readiness.** Diff is mergeable. All four Wave 2.5 reclassifications are honest (assertion refreshes only, no production code touched to make tests pass), each refreshed test preserves its load-bearing contract (cleanup-before-rethrow ordering, generator else-branch preservation, scoped clone suppression with negative assertions, compiler-verified string-index guard), and full local validation reports `708 passed / 0 failed / 708 total` with `red_blocker.status: closed`.

- **Q3 — inventory and docs consistency.** Confirmed by direct `jq`:
  - `failures` count: 52 total, 0 open, all `status: closed`.
  - `red_blocker`: `status: closed`, `failure_count: 0`, `test_result: 708/0/708`.
  - Subwave histogram: `1:20, 2:16, 3:6, 4:6, 5:4 = 52`. Matches the result-progression cadence `52→32→16→10→4→0`.
  - Classification: `stale-expectation:40, obsolete-test:6, compiler-bug:6 = 52`. Matches triage-doc counts at lines 17–19.
  - Phase plan Wave 2.5 Implementation Notes (lines 404–419) accurately mirror inventory state and now include the `Vec<char>`-cache/let-else honesty note plus the pass-1 review pointer with a "nits addressed" acknowledgment.

- **Validation evidence re-checked against prompt.** `cargo test -p sifr_codegen` 708/0/708, `cargo fmt --check` pass, `python3 scripts/check_file_size_guardrails.py` pass, `scripts/run_all_tests.sh --profile create-pr` pass at 168.01s with 100% e2e cache hit — all consistent with what the artifacts claim.

- **Gate closure.** Red-blocker gate is consistently closed across the three artifacts (inventory JSON, triage doc, phase plan). Wave 2.5 is in a green-merge state; promoting `sifr_codegen` into the merge profile under the Wave `2.final` slice is unblocked.
