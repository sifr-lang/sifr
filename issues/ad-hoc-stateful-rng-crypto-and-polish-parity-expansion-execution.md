# Ad Hoc Phase Execution Checklist (Stateful RNG, Crypto, and Polish Parity Expansion)

Status: in-progress (started 2026-03-21; entry baseline validated; `wave_psp_rng_0` completed; `wave_psp_rng_1` implementation + validation merged via PR #1376; external review pass loop active)
Owner: ad_hoc_stateful_rng_crypto execution loop
Reference planning doc:
- `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 0
- [x] Scope remains constrained to active wave
- [ ] Root cause is fixed without compatibility shims
- [x] Positive-path and negative-path validation recorded for each wave
- [x] Demo runs before opening each wave PR
- [x] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [x] PR opened/reviewed/merged before next wave starts
- [x] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_psp_rng_0`: architecture lock for typed RNG state model, module-global proxy rules, bytes-native crypto boundary, and permanent divergence classification
2. [x] `wave_psp_rng_1`: deterministic RNG state and object model (`RandomState`, `Random`, `SystemRandom`, module-global delegation)
3. [ ] `wave_psp_rng_2`: advanced hash and binary-surface expansion (`hashlib`, `base64`)
4. [ ] `wave_psp_rng_3`: final polish waiver reduction (`statistics`, residual `textwrap`, residual `html`)
5. [ ] wave-level extra completion review cycle done
6. [ ] wave-level extra production-grade review cycle done
7. [ ] milestone-level completion review cycle done
8. [ ] milestone-level production-grade review cycle done
9. [ ] phase-level completion review cycle done
10. [ ] phase-level production-grade review cycle done
11. [ ] closure telegram notification sent

## Detailed Wave To-Do Plan

### `wave_psp_rng_1`
- [x] Add typed deterministic `RandomState`/`Random` object model in `lib/sifr/random.sifr`.
- [x] Add module-global RNG state storage + intrinsic plumbing for `seed/getstate/setstate`.
- [x] Keep `SystemRandom` state export/import explicitly unsupported with typed `Result` boundaries.
- [x] Keep weighted `choices(weights=...)` explicitly unsupported in this wave.
- [x] Add positive fixture + demo covering deterministic replay and state round-trips.
- [x] Add negative fixture for `SystemRandom` state boundary.
- [x] Run `cargo test -p sifr -- --skip test_e2e_pass`.
- [x] Run full gate `$(pwd)/scripts/run_all_tests.sh`.
- [x] Open PR, review, and merge for this wave (`https://github.com/yaseralnajjar/sifr/pull/1376`).

### `wave_psp_rng_2`
- [ ] Expand `hashlib` to bytes-native digest/object APIs (`digest`, `digest_bytes`, `update_bytes`, `new_bytes`).
- [ ] Audit dependency support and close approved SHA3/SHAKE tranche only where runtime support is real.
- [ ] Expand `base64` residual binary-surface parity on first-class `bytes`.
- [ ] Port/adapt relevant CPython tests and lock explicit waivers for anything still deferred.
- [ ] Run demo + full gate, then PR/review/merge.

### `wave_psp_rng_3`
- [ ] Reduce residual `statistics` waivers on deterministic float/int-safe advanced surfaces.
- [ ] Re-triage residual `textwrap`/`html` waivers and close low-risk owned gaps.
- [ ] Port/adapt relevant CPython tests and refresh waiver inventory.
- [ ] Run demo + full gate, then PR/review/merge.

## Entry Baseline Evidence (2026-03-21)

Baseline command:
- `scripts/run_all_tests.sh`

Observed baseline result before rng/crypto wave edits:
- HIR maintainability guardrails: PASS
- `sifr_driver` maintainability guardrails: PASS
- `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
- e2e fail/runtime/corpus lane: PASS (`25` tests)
- validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`, `phase24_hir_analysis`, `phase25_cfg_flow`): PASS (`14` rows)
- e2e pass suite `pr` profile: PASS (`64` fixtures, report signature `2161ea8c3fd4e3df`)
- phase-29 hardening suites: PASS (`18` variants, `0` failures)
- pr lane report: PASS (wall `323.81s`, max RSS `793.4MiB`, swaps `0`)

Required entry records:
- architecture lock must pin one typed deterministic RNG-state contract (`RandomState`) and one module-global proxy behavior before wave 1 implementation begins.
- bytes-native crypto boundaries (`HashObject.digest` bytes carrier, `update_bytes`, `new_bytes`, SHA3/SHAKE eligibility) must be explicitly classified before wave 2 implementation begins.
- residual `textwrap`/`html` waivers must be explicitly classified as either phase-owned closure targets or intentionally carried diffs before wave 3 implementation begins.

## Wave Progress

### wave_psp_rng_0: Architecture Lock
- Status: completed
- Scope:
  - lock `RandomState` object shape and module-global `random` delegation contract
  - lock bytes-native crypto boundary expectations for `hashlib` (including bytes digest/object APIs and SHA3/SHAKE scope gate)
  - classify permanent Sifr-safe diffs and residual `textwrap`/`html` ownership for this phase
  - add wave-0 lock fixture/demo and explicit negative fixtures for deferred families
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_0_architecture_lock.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_rng_wave0_architecture_lock_demo.sifr` -> PASS (`ad_hoc_rng_wave0_architecture_lock_demo: pass`)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_rng_0_hashlib_bytes_digest_api_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_rng_0_textwrap_max_lines_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-21)

### wave_psp_rng_1: Deterministic RNG State and Object Model
- Status: completed (implementation + validation + merge via PR `#1376`)
- Scope:
  - ship typed deterministic `RandomState(version, state_words, index, gauss_next)` with `Random` mutable state ownership
  - ship module-global delegation for `seed/getstate/setstate/randrange/randint/random/choice/choices/sample/shuffle/gauss/uniform/randbytes`
  - keep weighted `choices(weights=...)` unsupported for this wave
  - keep `SystemRandom` state export/import unsupported with explicit typed boundaries
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_1_stateful_random_object_model.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_rng_wave1_stateful_object_model_demo.sifr` -> PASS (`ad_hoc_rng_wave1_stateful_object_model_demo: pass`)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_rng_1_system_random_state_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr` -> expected compile failure (PASS)
  - unit lane: `cargo test -p sifr -- --skip test_e2e_pass` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-21)
- Merge evidence:
  - implementation PR: `https://github.com/yaseralnajjar/sifr/pull/1376` (merged 2026-03-21)
  - external review pass 1 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-review-pass-1.md`
