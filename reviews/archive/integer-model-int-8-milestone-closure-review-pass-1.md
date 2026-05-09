

Now I have all the evidence needed. Let me write the milestone closure review.

---

## INT-8 Milestone Closure Review — Pass 1

### Verdict

**SATISFIED**

### Evidence Summary

| Check | Result | Details |
|---|---|---|
| `cargo clippy --workspace -- -D warnings` | PASS | No warnings |
| `cargo fmt --check` | PASS | No diff |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | No guardrail violations |
| `python3 scripts/run_integer_model_closure_perf.py` | PASS | Zero allocations; 2.38x slowdown |
| JSON manifests (`-m json.tool`) | PASS | Both `property_manifest.json` and `fuzz_smoke_manifest.json` valid |
| Sifr perf fixture run | PASS | "sifr_int_loop: passed" |
| Property/fuzz-smoke seed runs | PASS | Both `integer_external_boundaries_seed.sifr` and `integer_fixed_width_helpers_seed.sifr` pass |
| Hardening (`--profile nightly --suite property --suite fuzz-smoke`) | PASS | variants=45, failures=0, blocking_failures=0 |
| `scripts/run_all_tests.sh --profile quick` | PASS | wall_time=55.28s, e2e 23 pass tests, budget_ok=yes |
| `scripts/run_all_tests.sh --profile pr` | PASS | wall_time=110.69s, e2e 63 pass tests, hardening variants=28, budget_ok=yes |

### Strictness Checks

**1. Small-int allocation and throughput closure evidence**

Allocation check: `sifr_loop_allocs=0`, `counter_allocs=0`, `hash_allocs=0` — all three probes confirmed zero heap allocations in the generated code path.

Throughput check: 2.38x slowdown measured under `GlobalAlloc` instrumentation with 200,000 iterations and 5 repeats best-of-5 timing. This is better than the 3.19x observed in prior runs due to system variance; the gating threshold remains 10x.

The evidence is concrete: instrumentation reads actual `GlobalAlloc` counts, not an assumption. **Acceptable.**

**2. 10x threshold is properly ratified and documented; long-term 2x Phase 35 target preserved**

`verification/integer_model_closure_hardening.md` (lines 17–18) explicitly documents:

> "INT-8 records a local ratified threshold here rather than wiring a broader benchmark governance lane. The long-term target remains within `2x` of an equivalent optimized Rust `i64` loop once Phase 35 owns statistically governed performance budgets. Until that tooling exists, the INT-8 blocking threshold is `10x`; the stricter `2x` target is tracked as the future Phase 35 budget target, not an unratified blocker for this closure phase."

The design doc (`internal_docs/integer_model.md` line 544) also anchors this: "small `int` loops stay on `SifrInt::Small` without per-iteration heap allocation."

Phase 35 exists as `internal_docs/phases/35_performance_benchmarking_and_budgets.md` and is referenced on the roadmap as owning "performance thresholds and benchmark governance." The threshold separation is explicit and traceable. **Acceptable.**

**3. Fuzz/property coverage addresses high-risk external-input integer paths**

- `integer_external_boundaries_seed.sifr`: exercises `from_array`, `from_int`, `dumps_web` (JS-safe rejection with `JsonIntegerRangeError`), `dumps_string_ints`, `validate_integer_digit_limits` (4096-digit limit rejection with `JsonLimitError`), and `loads` with oversized integer. All paths terminate deterministically with typed errors, no panics.
- `integer_fixed_width_helpers_seed.sifr`: exercises `uint8` wrapping, saturating, overflowing, and checked overflow semantics with inline assertions. All paths terminate with correct results.

Both seeds have `assert_no_panic: true` and are registered in `property_manifest.json` (PROP-INT-0001, PROP-INT-0002) and `fuzz_smoke_manifest.json` (FUZZ-SMOKE-LOCAL-0001 seeds list). **Acceptable.**

**4. Generated-code panic-shape and full validation closure evidence**

`assert_no_panic: true` is set in both manifests. The hardening runner reports 45 variants (nightly) and 28 variants (pr) with zero failures and zero blocking failures across property and fuzz-smoke suites.

Full validation lanes (`quick`: 55.28s; `pr`: 110.69s) report budget_ok=yes with no e2e failures. **Acceptable.**

**5. Tracker/doc updates**

The checklist already marks INT-8 items [x]:
- [x] PR #1901 (fixture + runner + seeds + hardening artifact)
- [x] PR #1902 (clippy cleanup)

The review history and final checklist item for INT-8 milestone closure need to be added, then the INT-8 row itself marked [x].

---

### Blockers

**None.**

---

### Non-Blocking Notes

1. **Format string hygiene**: The probe script uses `f"..."` and `f"""..."""` for format strings with no bare `{` in output text. `print(f"  {key}={values[key]}")` on line 274 is correct — `key` and `values[key]` are local variables, not format specs. No format string risks.
2. **Observed slowdown variance**: 2.38x this run vs 3.19x previously is within normal variance for microbenchmarks. The gating threshold is 10x; no action needed.
3. **Hardening coverage composition**: The pr profile runs 28 hardening variants (diagnostics, project, fixedbugs, crashes, oss-curated). The nightly profile additionally runs property (4 entries) and fuzz-smoke (1 corpus with 8 seeds). Both profiles are clean. The INT-8 seeds are exercised in both profiles.
4. **Probe artifact location**: `target/integer_model_closure_perf_probe/` is a transient build directory. The runner creates and manages it; no cleanup needed since it's under `target/`.

---

### Tracker Update (suggested additions to `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`)

**In Review History, add:**
```
- [x] INT-8 closure hardening gates review pass 1 satisfied: `verification/perf/sifr_int_loop.sifr` fixture, `scripts/run_integer_model_closure_perf.py` runner (zero allocations, 2.38x observed slowdown under ratified 10x threshold), integer external boundaries and fixed-width helpers property seeds registered in `property_manifest.json` and `fuzz_smoke_manifest.json`, `verification/integer_model_closure_hardening.md` artifact documents gate, threshold rationale, fuzz/property coverage, and panic-shape sweep ownership; `scripts/run_all_tests.sh --profile quick` (55.28s, budget_ok=yes) and `scripts/run_all_tests.sh --profile pr` (110.69s, e2e 63 pass, hardening variants=28, budget_ok=yes) pass locally; `scripts/run_verification_hardening.py --profile nightly --suite property --suite fuzz-smoke` reports variants=45, failures=0; PR #1901, PR #1902: `reviews/integer-model-int-8-milestone-closure-review-pass-1.md`.
```

**In Implementation Checklist, update the INT-8 row:**
```
- [x] INT-8 closure hardening and performance gates
  - [x] `verification/perf/sifr_int_loop.sifr` fixture for small-int accumulation/counter loops with inline assertions.
  - [x] `scripts/run_integer_model_closure_perf.py` runner with zero-allocation probe and 10x throughput gate; 2.38x observed, zero allocations.
  - [x] `integer_external_boundaries_seed.sifr` fuzz/property seed for JSON profile and digit-limit boundary behavior.
  - [x] `integer_fixed_width_helpers_seed.sifr` fuzz/property seed for fixed-width checked/wrapping/saturating/overflowing helper surfaces.
  - [x] Seeds registered in `property_manifest.json` (PROP-INT-0001, PROP-INT-0002) and `fuzz_smoke_manifest.json` (FUZZ-SMOKE-LOCAL-0001 seeds list).
  - [x] `verification/integer_model_closure_hardening.md` artifact documenting gate, threshold rationale, fuzz/property coverage, and panic-shape sweep ownership.
  - [x] Review passes satisfied: `reviews/integer-model-int-8-closure-hardening-gates-review-pass-1.md` (PR #1901), `reviews/integer-model-int-8-clippy-closure-cleanup-review-pass-1.md` (PR #1902), `reviews/integer-model-int-8-milestone-closure-review-pass-1.md`.
```
