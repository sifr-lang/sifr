# Ad Hoc Phase Execution Checklist (Python Source Parity Extension and Waiver Reduction)

Status: completed (started 2026-03-18, completed 2026-03-18)
Owner: ad_hoc_parity_extension execution loop
Reference planning doc:
- `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 1
- [x] Scope remains constrained to active wave
- [x] Root cause is fixed without compatibility shims
- [x] Positive-path and negative-path validation recorded for each wave
- [x] Demo runs before opening each wave PR
- [x] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [x] PR opened/reviewed/merged before next wave starts
- [x] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_psp_ext_1`: builtin iterator return-shape re-closure (`reversed`, `enumerate`, `zip`, `map`)
2. [x] `wave_psp_ext_2`: `itertools` lazy-surface closure (`accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest`, `cycle`, `starmap`, `product`, `permutations`, `combinations`, `combinations_with_replacement`)
3. [x] `wave_psp_ext_3`: regex/filesystem iterator surfaces (`re.finditer`, `Pattern.finditer`, `glob.iglob`, `Path.iterdir`, `Path.glob`, `Path.rglob`)
4. [x] `wave_psp_ext_4`: waiver-ledger reduction and phase exit-closure governance updates
5. [x] wave-level extra completion review cycle done
6. [x] wave-level extra production-grade review cycle done
7. [x] milestone-level completion review cycle done
8. [x] milestone-level production-grade review cycle done
9. [x] phase-level completion review cycle done
10. [x] phase-level production-grade review cycle done (review artifact: reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-phase-closure-production-grade-review.md)
11. [x] closure telegram notification sent

## Entry Baseline Evidence (2026-03-18)

Baseline command:
- `$(pwd)/scripts/run_all_tests.sh --profile quick`

Observed baseline result before wave edits:
- HIR maintainability guardrails: PASS
- `sifr_driver` maintainability guardrails: PASS
- `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
- e2e fail/runtime/corpus lane: PASS (`25` tests)
- validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`): PASS (`7` rows)
- e2e pass suite quick profile: PASS (`24` fixtures, report signature `e1bf653aaa770517`)
- quick lane report: PASS (wall `36.59s`, max RSS `105.1MiB`, swaps `0`)

Required entry records:
- Current builtin iterator-return mismatch to retire in this phase:
  - `map(...)` remains eager/list-returning while `reversed`/`enumerate`/`zip` are already iterator-returning.
- Current `itertools` lazy-waiver debt to retire in this phase:
  - `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest`, `cycle`, `starmap`, `product`, `permutations`, `combinations`, `combinations_with_replacement` still list-backed.
- Current iterator-returning API gaps to close in this phase:
  - `re.finditer(...)`, `Pattern.finditer(...)`, `glob.iglob(...)`, `Path.iterdir()`, `Path.glob(...)`, `Path.rglob()`.
- Initial CPython test-family inventory for this continuation:
  - `Lib/test/test_builtin.py` (`map` iterator contract, `zip`/`enumerate`/`reversed` iterable protocol consistency)
  - `Lib/test/test_itertools.py` (iterator-returning combinators in approved scope)
  - `Lib/test/test_re.py` (`finditer` behavior)
  - `Lib/test/test_glob.py` (`iglob` behavior)
  - `Lib/test/test_pathlib/` (iterator-returning behavior for `iterdir/glob/rglob`)

## Wave Progress

### wave_psp_ext_1: Builtin Iterator Re-Closure
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1254` (merged)
- Validation:
  - positive path: `cargo test -p sifr_hir -- test_map_is_typed_as_iterator --nocapture` -> PASS
  - negative path: `cargo test -p sifr_hir -- test_map_rejects_plain_list_annotation_without_materialization --nocapture` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_builtins_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/lambda_basic.sifr` -> PASS
  - demo check: `cargo run -q -p sifr -- check demos/ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` -> `no errors found`
  - demo run: `cargo run -q -p sifr -- run demos/ad_hoc_parity_ext_wave1_builtin_iterator_reclosure_demo.sifr` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_psp_ext_2: `itertools` Lazy Surface Closure
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1256` (merged)
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_itertools_consolidated.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_accumulate_float.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_accumulate_bigint.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_accumulate_str.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_callable_typevar.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_dropwhile_predicate.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_zip_longest_str.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_parity_ext_wave2_itertools_lazy_surface_demo.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_ext_2_itertools_materialization_required.sifr` -> PASS (`type mismatch: expected 'list[int]', got 'Iterator[int]'`)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_itertools_starmap_non_binary_callable.sifr` -> PASS (compile-time rejection preserved)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_psp_ext_3: Regex and Filesystem Iterator Surfaces
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1259` (merged)
  - review closure PR (pass 1): `https://github.com/sifr-lang/sifr/pull/1260` (merged)
  - review closure PR (pass 2): `https://github.com/sifr-lang/sifr/pull/1261` (merged)
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_glob_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_pathlib_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_pathlib_consolidated.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/pathlib_glob_semantics.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/path_glob.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_d1_filesystem_paths_archives.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_ext_3_regex_filesystem_iterators.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_parity_ext_wave3_regex_filesystem_iterators_demo.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_ext_3_pathlib_iterator_materialization_required.sifr` -> PASS (`type mismatch: expected 'list[str]', got 'Iterator[str]'`)
  - regression spot-check: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_re_subset.sifr` -> PASS
  - regression spot-check: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_e1_core_modules_numeric_patterns_crypto.sifr` -> PASS
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

### wave_psp_ext_4: Waiver Ledger Reduction and Exit Closure
- Status: merged
- Implementation PR:
  - `https://github.com/sifr-lang/sifr/pull/1262` (merged)
- Validation:
  - governance/doc updates:
    - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
    - `verification/stdlib/wave_psp_b2_cpython_traceability.md`
    - `internal_docs/architecture.md`
    - `internal_docs/phases/07_stdlib_parity.md`
    - `internal_docs/phases/12_stdlib_remediation.md`
    - `internal_docs/roadmap.md`
    - `issues/ad-hoc-python-source-parity-extension-waiver-reduction-execution.md`
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-18)

## External Review Passes

### review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-1-review-pass-1.md`
- Status: completed (validated; no actionable defects)

### review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-1-review-pass-2a.md`
- Status: completed (validated; no actionable defects)

### wave_psp_ext_2 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-2-review-pass-1.md` and `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-2-review-pass-1a.md`
- Status: completed (validated; minor governance-diff documentation updates applied)

### wave_psp_ext_2 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-2-review-pass-2.md`
- Status: completed (validated; no actionable defects)

### wave_psp_ext_3 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-3-review-pass-1.md`
- Status: completed (validated; no actionable defects, informational note only: current iterator surfaces are materialize-then-iterate at intrinsic boundary; closure PR `#1260`)

### wave_psp_ext_3 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-3-review-pass-2.md`
- Status: completed (validated; no actionable defects, production-grade approved; closure PR `#1261`)

### wave_psp_ext_4 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-4-review-pass-1.md`
- Status: completed (validated; applied reviewer-aligned clarity update to explicitly classify `itertools.tee`/`itertools.groupby` in `wave_psp_b2` traceability; closure PR `#1263`)

### wave_psp_ext_4 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-psp-ext-4-review-pass-2.md`
- Status: completed (validated; no actionable defects, production-grade approved)

### closure review cycles
- wave closure completion review: completed (`reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-closure-completion-review.md`)
- wave closure production-grade review: completed (`reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-wave-closure-production-grade-review.md`)
- milestone closure completion review: completed (`reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-milestone-closure-completion-review.md`)
- milestone closure production-grade review: completed (`reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-milestone-closure-production-grade-review.md`)
- phase closure completion review: completed (`reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-phase-closure-completion-review.md`)
- phase closure production-grade review: completed (`reviews/phase-ad-hoc-python-source-parity-extension-waiver-reduction-phase-closure-production-grade-review.md`)
