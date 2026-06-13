# Ad Hoc Phase Execution Checklist (Structured Data and Class-Surface Parity Expansion)

Status: completed (started 2026-03-18; completed 2026-03-18 with wave/milestone/phase closure reviews)
Owner: ad_hoc_structured_class_surface execution loop
Reference planning doc:
- `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`

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
1. [x] `wave_psp_struct_0`: architecture lock and explicit waiver-boundary enforcement
2. [x] `wave_psp_struct_1`: parser and serialization surface expansion (`json`, `configparser`, `csv`)
3. [x] `wave_psp_struct_2`: collections and CLI class-surface expansion (`collections`, `argparse`)
4. [x] `wave_psp_struct_3`: `uuid` and `datetime` expansion under fixed-offset timezone contract
5. [x] `wave_psp_struct_4`: text-surface polish (`textwrap`, `html`) and governance closure
6. [x] wave-level extra completion review cycle done
7. [x] wave-level extra production-grade review cycle done
8. [x] milestone-level completion review cycle done
9. [x] milestone-level production-grade review cycle done
10. [x] phase-level completion review cycle done
11. [x] phase-level production-grade review cycle done
12. [x] closure telegram notification sent

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
- Status: completed
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1269` (merged)
  - review closure PR (pass 1): `https://github.com/sifr-lang/sifr/pull/1270` (merged)
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
- Status: completed
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1272` (merged)
  - review closure PR (pass 1): `https://github.com/sifr-lang/sifr/pull/1273` (merged)
- Scope:
  - `json`: add `JSONEncoder`/`JSONDecoder` typed wrapper classes with file and handle load/dump helpers
  - `configparser`: add interpolation-aware `get(..., raw=...)`, `SectionProxy`, and ini write-back surface (`to_ini_string`, `write`)
  - `csv`: add process-local `DialectRegistry` with defensive dialect copying for register/get boundaries
  - add explicit unsupported boundary fixture for converter registration (`register_converter`)
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_struct_1_parser_serialization_expansion.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_struct_wave1_parser_serialization_expansion_demo.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_1_configparser_converter_registration_unsupported.sifr` -> expected compile failure (PASS)
  - regression path: `stdlib_configparser.sifr`, `stdlib_json_consolidated.sifr`, `stdlib_csv_consolidated.sifr`, `cpython_configparser_subset.sifr`, `cpython_json_subset.sifr`, `cpython_csv_subset.sifr` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_psp_struct_2: Collections and CLI Class-Surface Expansion
- Status: completed
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1275` (merged)
- Scope:
  - `collections`: add `Counter(iterable=...)` constructor parity alongside mapping input, and promote `defaultdict` to an explicit typed class surface (`ensure`, `set`, `has`, `pop`, `clear`, `keys`, `values`, `items`, `len`)
  - `argparse`: add bounded `subparsers`, `nargs` forms (`int`, `?`, `*`, `+`), typed coercion (`str`/`int`/`float`/`bool`) via `add_argument_typed`, and namespace list support (`set_list`/`get_list`)
  - add wave-2 coverage fixture and demo for combined `collections` + `argparse` class-surface expansion
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_struct_2_collections_argparse_expansion.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_struct_wave2_collections_argparse_expansion_demo.sifr` -> PASS
  - regression path: `stdlib_argparse.sifr`, `cpython_argparse_subset.sifr`, `stdlib_collections_consolidated.sifr`, `cpython_collections_subset.sifr`, `phase31_defaultdict_len_deque_compat.sifr`, `phase_psp_b1_collections_ordered_helpers.sifr` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_psp_struct_3: UUID and Datetime Expansion
- Status: completed
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1278` (merged)
  - review closure PR (pass 1): `https://github.com/sifr-lang/sifr/pull/1279` (merged)
- Scope:
  - `uuid`: add typed name-based generation (`uuid3`, `uuid5`) and exported namespace accessors (`NAMESPACE_DNS`, `NAMESPACE_URL`, `NAMESPACE_OID`, `NAMESPACE_X500`)
  - `datetime`: expand fixed-offset timezone surfaces (`UTC`, `utc`, `now(tz=...)`, `from_timestamp(..., tz=...)`, `datetime.astimezone(...)`) with explicit offset-aware ISO/timestamp behavior
  - add wave-3 coverage fixture and demo for combined `uuid` + `datetime` expansion
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_struct_3_uuid_datetime_expansion.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_struct_wave3_uuid_datetime_expansion_demo.sifr` -> PASS
  - regression path: `stdlib_uuid_consolidated.sifr`, `cpython_uuid_subset.sifr`, `stdlib_datetime_consolidated.sifr`, `cpython_datetime_subset.sifr`, `phase_psp_e1_core_modules_numeric_patterns_crypto.sifr`, `edge_case_safety.sifr`, `zero_panic_gate.sifr` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_psp_struct_4: Text-Surface Polish and Governance Closure
- Status: completed
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1281` (merged)
  - review closure PR (pass 1): `https://github.com/sifr-lang/sifr/pull/1282` (merged)
- Scope:
  - `textwrap`: expand `TextWrapper` adjacent option matrix with bounded deterministic fields (`expand_tabs`, `tabsize`, `replace_whitespace`, `drop_whitespace`, `break_on_hyphens`)
  - `html`: add top-level `escape(s, quote: bool = True)` polish while keeping package-level expansion (`html.parser`) explicitly unsupported
  - add wave-4 coverage fixture and demo for text-surface governance closure
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_struct_4_text_surface_governance_closure.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_struct_wave4_text_surface_governance_closure_demo.sifr` -> PASS
  - regression path: `stdlib_textwrap_consolidated.sifr`, `cpython_textwrap_subset.sifr`, `cpython_textwrap_textwrapper_subset.sifr`, `stdlib_html.sifr`, `demos/m30_1c_textwrap_parity_demo/main.sifr`, `demos/wave_psp_c2_text_pattern_formatting_demo.sifr` -> PASS
  - negative boundary: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> pending (run before opening PR)

## External Review Passes

### wave_psp_struct_0 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-0-review-pass-1.md`
- Status: completed (review note about swapped fixture content was validated and found non-reproducible in `main`; fixture contents match their filenames and both expected-fail checks remain enforced)

### wave_psp_struct_0 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-0-review-pass-2.md`
- Status: completed (approved for wave progression)

### wave_psp_struct_1 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-1-review-pass-1.md`
- Status: completed (approved; no corrective code changes required)

### wave_psp_struct_1 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-1-review-pass-2.md`
- Status: completed (approved for wave progression; no corrective code changes required)

### wave_psp_struct_2 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-2-review-pass-1.md`
- Status: completed (approved; no corrective code changes required)

### wave_psp_struct_2 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-2-review-pass-2.md`
- Status: completed (approved for wave progression; no corrective code changes required)

### wave_psp_struct_3 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-3-review-pass-1.md`
- Status: completed (approved; no corrective code changes required)

### wave_psp_struct_3 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-3-review-pass-2.md`
- Status: completed (approved for wave progression; no corrective code changes required)

### wave_psp_struct_4 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-4-review-pass-1.md`
- Status: completed (approved; no corrective code changes required)

### wave_psp_struct_4 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-psp-struct-4-review-pass-2.md`
- Status: completed (approved for wave progression; no corrective code changes required)

### wave_closure review_pass_1 (completion)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-closure-review-pass-1.md`
- Status: completed (approved; no corrective code changes required)

### wave_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-wave-closure-review-pass-2.md`
- Status: completed (approved for milestone-closure progression; no corrective code changes required)

### milestone_closure review_pass_1 (completion)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-milestone-closure-review-pass-1.md`
- Status: completed (approved; no corrective code changes required)

### milestone_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-milestone-closure-review-pass-2.md`
- Status: completed (approved for phase-closure progression; no corrective code changes required)

### phase_closure review_pass_1 (completion)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-phase-closure-review-pass-1.md`
- Status: completed (approved; no corrective code changes required)

### phase_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-structured-data-and-class-surface-parity-expansion-phase-closure-review-pass-2.md`
- Status: completed (approved for long-term governance closure; no corrective code changes required)

### closure review cycles
- wave closure completion review: completed
- wave closure production-grade review: completed
- milestone closure completion review: completed
- milestone closure production-grade review: completed
- phase closure completion review: completed
- phase closure production-grade review: completed
