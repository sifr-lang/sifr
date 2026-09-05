# Post-M10 Adapter Policy Adherence Audit — agent Review, Pass 2

## Verdict: PASS

Pass 1 was already clean on the adapter-policy audit itself. Pass 2 verifies
that the follow-up changes made after pass 1 — the trend-policy deferral and
its self-test hardening — do not weaken any invariant. They don't. The audit
result recorded in `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md`
still matches the current `stdlib/_sifr/*.sifr` sources and the
`sifr_stdlib_model` feature table; the new perf-trend deferral is
time-boxed, only defers freshness (not thresholds or smoke execution), and
`check_trend_policy.py --self-test` still proves the stale/missing gates when
that deferral is absent.

Evidence independently reconfirmed since pass 1:

- 115 completed `@rust(sifr_stdlib..., panic=trusted_no_panic)` declarations
  across `_sifr.platform/html/calendar/uuid/math/crypto/regex`; zero
  occurrences of `@rust.via`, `bridge.`, `converter`, or `pipeline` in those
  seven modules (`grep` over `stdlib/_sifr/*.sifr`).
- `stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies` now
  covers the 6 completed private modules plus the 8 public modules it already
  covered; `_sifr.crypto` is intentionally excluded because
  `crates/sifr_stdlib_model/src/features.rs:605` still routes it through
  `Rand`/`RandDistr` for the un-migrated stateful random surface — matching
  the audit result bullets in the plan doc.
- `merged_user_and_private_stdlib_interop_keeps_user_trust_separate`
  (`crates/sifr_driver/src/build/sysroot_interop_tests.rs:136-169`) still
  correctly proves sysroot `trusted_no_panic` does not extend to
  `bridge.user_noop` after the user manifest's `TrustPolicy` is reset —
  fails with `SIFR-RUST-TRUST-0001`.
- The doc taxonomy wording change
  (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:127,132`) rewords
  "lower-level backend code" -> "lower-level implementation code" and
  "`sifr_stdlib` own backend-specific adaptation" -> "`sifr_stdlib` own
  implementation adaptation", with `M9-M13 migrations` -> `These migrations`.
  Semantics-preserving; consistent with the surrounding taxonomy sections
  that already talk about `sifr_stdlib` as the owner of implementation
  adaptation.
- Perf-trend deferral (`verification/areas/performance/data/trend/current.json`)
  covers exactly the 65 result IDs (`jq` diff of `.results[].id` vs the new
  deferral's `benchmark_ids` returns identical), expires `2026-07-31`
  (~27 days from today `2026-07-04`), and applies only to freshness — the
  freshness check is the only site where `has_benchmark_deferral` is
  consulted in `check_trend_policy.py:299-304`. Baselines are 48.1 days old
  vs a 45-day window, so the deferral is a bounded catch-up window rather
  than an open-ended bypass.
- `run_self_test` now strips `benchmark_ids` deferrals before both the stale
  and missing negative cases, and additionally refreshes the remaining
  baselines' `baseline_captured_at_unix` in the missing case. Without those
  refreshes, `validate_freshness` would raise
  `stale trend baseline for build-project-001-additional-modules` before
  `validate_results`'s missing-id check runs — so `assert_fails(...,
  "missing current trend baselines")` would fail with the wrong diagnostic.
  The extra timestamp bump correctly isolates the missing-id gate as the
  active negative case.

---

## Findings (actionable)

None that block the audit checkpoint.

---

## Non-blocking suggestions

1. `verification/areas/performance/check_trend_policy.py:322-323,332-333` —
   the "stale" and "missing" self-test cases strip deferrals with
   `"benchmark_ids" not in deferral` and then refresh timestamps to isolate
   each negative gate. The logic is correct but non-obvious to a future
   reader; a one-line comment above each strip explaining "drop the
   freshness deferral so the negative case actually exercises the gate"
   would prevent someone from later "cleaning up" what looks like redundant
   filtering. Not a blocker because the self-test itself passes and would
   loudly fail if the strip were removed.

2. `verification/areas/performance/data/trend/current.json:17-88` — the new
   `approved-reference-baseline-refresh-window` deferral names all 65
   benchmark IDs by hand. Since they were all captured at the same
   `1778968823` timestamp, the deferral scope is effectively "the entire
   trend baseline". This is legitimate given a single-shot capture, but
   means any future benchmark added to the manifest before the deferral
   expires will need to either land with a fresh baseline of its own or be
   appended to this list. Consider recording that maintenance rule in the
   deferral's rationale, or opening a follow-up to replace the checked-in
   baseline before `2026-07-31` so the deferral can be removed entirely
   rather than expanded. Non-blocking because the deferral does expire and
   its rationale already commits to a refresh.

3. Pass 1's non-blocking suggestions still stand and were not addressed in
   pass 2, which is fine (they were explicitly non-blocking):
   - `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:46`
     — `!source.contains("converter") && !source.contains("pipeline")` is a
     bare substring guard; safe today because none of the seven modules
     contain those strings anywhere, but fragile against future function
     names or docstrings.
   - `crates/sifr_stdlib_model/src/features_tests.rs:230-262` — the
     stateless-leaves list still omits `_sifr.crypto` without an inline
     comment; the exclusion reason is only documented in the plan doc.
   - `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:1104-1144`
     — "exact-shape direct binding" vs "`sifr_stdlib` adapter binding" is a
     doc-only taxonomy; not machine-checked.

4. `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:42` —
   `!source.contains("bridge.")` is a bare substring guard against the
   sysroot-adapter target root, but would also flag any future docstring,
   identifier, or comment containing the literal `bridge.` (e.g. a
   docstring saying "does not go through `bridge.foo`"). Today the seven
   completed modules contain zero occurrences, so the assertion is stable.
   Consider tightening to a `@rust(bridge.`-anchored check if you want the
   guard to survive future documentation changes. Non-blocking.

---

## Validation notes

- All required commands listed in the reviewer prompt are recorded as
  passing by the branch author. I did not re-run the full
  `scripts/run_all_tests.sh --profile create-pr` pipeline in this pass
  (review-only, no code changes); I spot-checked structural invariants via
  `git diff`, `grep`, and `jq`.
- `jq` cross-check confirmed the perf-trend deferral IDs equal
  `.results[].id` exactly and that no other deferral references
  `benchmark_ids`.
- No untracked risky files found beyond the two review notes
  (`plans/reviews/active/post-m10-adapter-policy-audit-agent-review-pass1.md`
  and this pass-2 note).

The audit checkpoint remains ready to unblock further M10/M11 waves.
