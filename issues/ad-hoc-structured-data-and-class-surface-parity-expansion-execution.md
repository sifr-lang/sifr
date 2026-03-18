# Ad Hoc Phase Execution Checklist (Structured Data and Class-Surface Parity Expansion)

Status: in_progress (started 2026-03-18)
Owner: ad_hoc_structured_class_surface execution loop
Reference planning doc:
- `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 0
- [ ] Scope remains constrained to active wave
- [ ] Root cause is fixed without compatibility shims
- [ ] Positive-path and negative-path validation recorded for each wave
- [ ] Demo runs before opening each wave PR
- [ ] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [ ] PR opened/reviewed/merged before next wave starts
- [ ] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [ ] `wave_psp_struct_0`: architecture lock and explicit waiver-boundary enforcement
2. [ ] `wave_psp_struct_1`: parser and serialization surface expansion (`json`, `configparser`, `csv`)
3. [ ] `wave_psp_struct_2`: collections and CLI class-surface expansion (`collections`, `argparse`)
4. [ ] `wave_psp_struct_3`: `uuid` and `datetime` expansion under fixed-offset timezone contract
5. [ ] `wave_psp_struct_4`: text-surface polish (`textwrap`, `html`) and governance closure
6. [ ] wave-level extra completion review cycle done
7. [ ] wave-level extra production-grade review cycle done
8. [ ] milestone-level completion review cycle done
9. [ ] milestone-level production-grade review cycle done
10. [ ] phase-level completion review cycle done
11. [ ] phase-level production-grade review cycle done
12. [ ] closure telegram notification sent

## Entry Baseline Evidence (2026-03-18)

Baseline command:
- `scripts/run_all_tests.sh --profile quick`

Observed baseline result before wave edits:
- HIR maintainability guardrails: PASS
- `sifr_driver` maintainability guardrails: PASS
- `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
- e2e fail/runtime/corpus lane: PASS (`25` tests)
- validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`): PASS (`7` rows)
- e2e pass suite quick profile: PASS (`24` fixtures, report signature `e1bf653aaa770517`)
- quick lane report: PASS (wall `58.37s`, max RSS `377.3MiB`, swaps `0`)

Required entry records:
- architecture lock must pin fixed public contracts for `json`, `datetime`, `uuid`, `csv`, `argparse`, and `collections` before feature expansion waves
- permanent Sifr-safe diffs in this phase must have explicit negative-path enforcement fixtures
- CPython-family mapping must classify adopted/adapted/waived direction per owned module family

## Wave Progress

### wave_psp_struct_0: Architecture Lock
- Status: merged (implementation) + review pass 1 in progress
- Implementation PR:
  - `https://github.com/yaseralnajjar/sifr/pull/1269` (merged)
- Scope:
  - contract lock for `json`, `datetime`, `uuid`, `csv`, `argparse`, `collections`
  - permanent-diff classification and enforcement fixtures
  - architecture-lock demos and mapping ledgers
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_struct_0_architecture_lock.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_struct_wave0_json_wrapper_model_demo.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_struct_wave0_fixed_offset_datetime_model_demo.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_json_dynamic_hooks_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_datetime_tzinfo_zoneinfo_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_counter_kwargs_constructor_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_csv_dynamic_registry_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_argparse_formatter_class_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_psp_struct_1: Parser and Serialization Surface Expansion
- Status: pending

### wave_psp_struct_2: Collections and CLI Class-Surface Expansion
- Status: pending

### wave_psp_struct_3: UUID and Datetime Expansion
- Status: pending

### wave_psp_struct_4: Text-Surface Polish and Governance Closure
- Status: pending

## External Review Passes

### wave_psp_struct_0 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-0-review-pass-1.md`
- Status: completed (review note about swapped fixture content was validated and found non-reproducible in `main`; fixture contents match their filenames and both expected-fail checks remain enforced)

### wave_psp_struct_0 review_pass_2 (production-grade)
- Reviewer artifact: pending
- Status: pending

### closure review cycles
- wave closure completion review: pending
- wave closure production-grade review: pending
- milestone closure completion review: pending
- milestone closure production-grade review: pending
- phase closure completion review: pending
- phase closure production-grade review: pending
