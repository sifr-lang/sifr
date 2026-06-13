# Ad Hoc Phase Execution Checklist (Stateful RNG, Crypto, and Polish Parity Expansion)

Status: closed (started 2026-03-21; entry baseline validated; `wave_psp_rng_0` completed; `wave_psp_rng_1` merged and review-closed; `wave_psp_rng_2` merged and review-closed; `wave_psp_rng_3` merged via PR `#1382` and review-closed via PR `#1383` + external production-grade pass-2 approval; milestone-closure review passes 1/2 completed and approved; phase-closure review passes 1/2 approved and production-grade closure confirmed; closure telegram sent on 2026-03-21)
Owner: ad_hoc_stateful_rng_crypto execution loop
Reference planning doc:
- `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 0
- [x] Scope remains constrained to active wave
- [x] Root cause is fixed without compatibility shims
- [x] Positive-path and negative-path validation recorded for each wave
- [x] Demo runs before opening each wave PR
- [x] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [x] PR opened/reviewed/merged before next wave starts
- [x] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_psp_rng_0`: architecture lock for typed RNG state model, module-global proxy rules, bytes-native crypto boundary, and permanent divergence classification
2. [x] `wave_psp_rng_1`: deterministic RNG state and object model (`RandomState`, `Random`, `SystemRandom`, module-global delegation)
3. [x] `wave_psp_rng_2`: advanced hash and binary-surface expansion (`hashlib`, `base64`)
4. [x] `wave_psp_rng_3`: final polish waiver reduction (`statistics`, residual `textwrap`, residual `html`)
5. [x] wave-level extra completion review cycle done
6. [x] wave-level extra production-grade review cycle done
7. [x] milestone-level completion review cycle done
8. [x] milestone-level production-grade review cycle done
9. [x] phase-level completion review cycle done
10. [x] phase-level production-grade review cycle done
11. [x] closure telegram notification sent

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
- [x] Open PR, review, and merge for this wave (`https://github.com/sifr-lang/sifr/pull/1376`).

### `wave_psp_rng_2`
- [x] Expand `hashlib` to bytes-native digest/object APIs (`digest`, `digest_bytes`, `update_bytes`, `new_bytes`).
- [x] Audit dependency support and close approved SHA3/SHAKE tranche only where runtime support is real.
- [x] Expand `base64` residual binary-surface parity on first-class `bytes`.
- [x] Port/adapt relevant CPython tests and lock explicit waivers for anything still deferred.
- [x] Run demo + full gate, then PR/review/merge.

### `wave_psp_rng_3`
- [x] Reduce residual `statistics` waivers on deterministic float/int-safe advanced surfaces.
- [x] Re-triage residual `textwrap`/`html` waivers and close low-risk owned gaps.
- [x] Port/adapt relevant CPython tests and refresh waiver inventory.
- [x] Run demo + full gate.
- [x] Open PR, review, and merge.

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
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-21)
  - historical note: `phase_psp_rng_0_hashlib_bytes_digest_api_unsupported.sifr` was retired after wave-2 bytes-native hashlib/base64 closure shipped.
  - historical note: `phase_psp_rng_0_textwrap_max_lines_unsupported.sifr` was retired after wave-3 formatter-option closure shipped.

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
  - implementation PR: `https://github.com/sifr-lang/sifr/pull/1376` (merged 2026-03-21)
  - external review pass 1 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-review-pass-1.md`
  - external review pass 1 fixes PR: `https://github.com/sifr-lang/sifr/pull/1377` (merged 2026-03-21)
  - external review pass 2 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-review-pass-2.md`
  - pass 2 validation result: reviewer output was stale (claimed wave 1 absent) and contradicted merged code/docs in PR `#1376`; no additional code fix was valid from that report

### wave_psp_rng_2: Advanced Hash and Binary Surface Expansion
- Status: completed (implementation merged + external completion/production-grade review passes closed)
- Scope:
  - ship bytes-native `hashlib` object model (`digest() -> bytes`, `digest_bytes()`, `update_bytes()`, `new_bytes()`)
  - keep str-facing compatibility surfaces (`update(str)`, `hexdigest()`, existing constructors) on top of bytes-native internal state
  - ship bytes-native `base64` API surfaces (`b64encode_bytes`, `b64decode_bytes`, standard/urlsafe bytes variants)
  - keep SHA3/SHAKE constructor families explicitly unsupported in this wave with typed boundary tests
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_rng_wave2_hashlib_base64_bytes_demo.sifr` -> PASS (`ad_hoc_rng_wave2_hashlib_base64_bytes_demo: pass`)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_rng_2_sha3_object_model_unsupported.sifr` -> expected compile failure (PASS)
  - unit/non-pass lane: `cargo test -p sifr -- --skip test_e2e_pass` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-21)
- Merge evidence:
  - implementation PR: `https://github.com/sifr-lang/sifr/pull/1379` (merged 2026-03-21)
  - external review pass 1 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-wave-psp-rng-2-review-pass-1.md`
  - external review pass 1 fixes PR: `https://github.com/sifr-lang/sifr/pull/1380` (merged 2026-03-21)
  - external review pass 2 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-wave-psp-rng-2-review-pass-2.md`
  - pass 2 validation result: reviewer marked wave as production-grade with no additional code changes required

### wave_psp_rng_3: Final Polish Waiver Reduction
- Status: completed (implementation merged + external completion/production-grade review passes closed)
- Scope:
  - ship deterministic `statistics.median_grouped(data, interval)` with typed boundaries
  - close residual `textwrap` formatter-option waivers (`fix_sentence_endings`, `max_lines`, `placeholder`)
  - re-triage `html` residual scope and keep package-wide `html.parser` ecosystem explicitly unsupported
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr` -> PASS (`ad_hoc_rng_wave3_polish_waiver_reduction_demo: pass`)
  - negative boundary: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-21)
- Merge evidence:
  - implementation PR: `https://github.com/sifr-lang/sifr/pull/1382` (merged 2026-03-21)
  - external review pass 1 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-wave-psp-rng-3-review-pass-1.md`
  - pass 1 status PR: `https://github.com/sifr-lang/sifr/pull/1383` (merged 2026-03-21)
  - pass 1 validation result: reviewer approved wave scope with no additional code changes required
  - external review pass 2 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-wave-psp-rng-3-review-pass-2.md`
  - pass 2 validation result: reviewer marked wave as production-grade with no additional code changes required

## Milestone and Phase Closure Progress

- milestone closure review pass 1 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-milestone-closure-review-pass-1.md`
- milestone closure pass 1 remediation scope: roadmap/architecture/inventory status alignment plus checklist drift cleanup
- milestone closure review pass 2 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-milestone-closure-review-pass-2.md`
- phase closure review pass 1 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-phase-closure-review-pass-1.md`
- phase closure review pass 2 artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-phase-closure-review-pass-2.md`

## Post-Closure CPython Adaptation Pass

- status: completed (post-closure add-on requested after phase closure to increase direct CPython test adaptation depth for shipped wave scope)
- added fixture: `crates/sifr/tests/e2e/pass/cpython_rng_phase_additional_subset.sifr`
- focus: additional adopted/adapted cases from `test_random` setstate-domain validation, `test_hashlib` constructor/case/vector coverage, `test_statistics` grouped interval/error boundaries, `test_textwrap` sentence-ending matrix cases, and `test_html` top-level escape coverage
- traceability updates:
  - `verification/stdlib/wave_psp_rng_1_cpython_traceability.md`
  - `verification/stdlib/wave_psp_rng_2_cpython_traceability.md`
  - `verification/stdlib/wave_psp_rng_3_cpython_traceability.md`
- validation:
  - targeted fixtures: PASS (`cpython_rng_phase_additional_subset`, `phase_psp_rng_1_stateful_random_object_model`, `phase_psp_rng_2_hashlib_base64_bytes_native_surface`, `phase_psp_rng_3_textwrap_formatter_options`)
  - full gate: `scripts/run_all_tests.sh` PASS (profile `pr`, 2026-03-21)

## Post-Closure External Review Remediation (Pass 1)

- review artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-post-closure-cpython-review-pass-1.md`
- status: completed (reviewer-validated gaps acted on with shipped-surface coverage + explicit unsupported boundaries)
- accepted remediation scope:
  - add CPython-adapted coverage for seeded `random.choices` deterministic path and `Random.gauss` cached-state behavior
  - add CPython-adapted `hashlib` incremental large-update equivalence coverage
  - add CPython-adapted `base64` lowercase Base32 decode (`b32decode` casefold-style) coverage
  - extend `html.unescape` intrinsic/test coverage for numeric references (`&#60;`, `&#x3C;`, `&#62;`, `&#x3E;`)
  - formalize unsupported boundaries with negative fixtures/docs for `statistics.NormalDist`, `hashlib.pbkdf2_hmac`, and `hashlib.scrypt`
- validation:
  - targeted: PASS (`cpython_rng_phase_additional_subset`, `phase_psp_rng_2_hashlib_pbkdf2_hmac_unsupported`, `phase_psp_rng_2_hashlib_scrypt_unsupported`, `phase_psp_rng_3_statistics_normaldist_unsupported`)
  - full gate: `scripts/run_all_tests.sh` PASS (profile `pr`, 2026-03-21)

## Post-Closure External Review Remediation (Pass 2)

- review artifact: `reviews/phase-ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-post-closure-cpython-review-pass-2.md`
- status: completed (acted on reviewer low-severity opportunities for stronger CPython parity depth)
- accepted remediation scope:
  - add uppercase-hex numeric reference support for `html.unescape` (`&#X27;`, `&#X3C;`, `&#X3E;`) and corresponding CPython-adapted fixture coverage
  - strengthen `random.choices` adaptation depth with 2000-draw seeded frequency bounds to detect distribution regressions
- validation:
  - targeted: PASS (`cpython_rng_phase_additional_subset`)
  - full gate: `scripts/run_all_tests.sh` PASS (profile `pr`, 2026-03-21)
