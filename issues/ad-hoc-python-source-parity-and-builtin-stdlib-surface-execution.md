# Ad Hoc Phase Execution: Python Source Parity and Builtin Stdlib Surface Closure

Status: in_progress
Started: 2026-03-14
Phase owner: Codex (GPT-5)
Source phase: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
Current active wave: `wave_psp_c2`

## Execution Rules

- Follow the phase in strict sequence.
- Only one wave is active at a time.
- Each wave must complete: CPython test inventory, implementation, demo validation, local validation, PR, review, merge, and doc updates before the next wave starts.
- No fallback-only APIs, no workaround-first closures, and no undocumented parity gaps.

## Phase Todo

- [x] `milestone_psp_1` / `wave_psp_a1`: builtin constructors and callable surface
- [x] `milestone_psp_2` / `wave_psp_a2`: core object models and builtin semantics
- [x] `milestone_psp_3` / `wave_psp_b1`: collections objects and ordered helpers
- [x] `milestone_psp_3` / `wave_psp_b2`: iterators, functional helpers, and randomness
- [x] `milestone_psp_4` / `wave_psp_c1`: structured parsing and serialization
- [ ] `milestone_psp_4` / `wave_psp_c2`: text, pattern, and formatting modules
- [ ] `milestone_psp_5` / `wave_psp_d1`: filesystem, paths, and archive surfaces
- [ ] `milestone_psp_5` / `wave_psp_d2`: process, runtime, and platform surfaces
- [ ] `milestone_psp_6` / `wave_psp_e1`: strong-but-incomplete core modules
- [ ] `milestone_psp_6` / `wave_psp_e2`: class-heavy and custom cleanup
- [ ] `milestone_psp_7`: parity governance and exit closure

## Wave Ledger

### `wave_psp_a1` Builtin Constructors and Callable Surface

Status: done

- [x] Baseline current builtin gaps against the phase spec and CPython test inputs:
  - `Lib/test/test_list.py`
  - `Lib/test/test_dict.py`
  - `Lib/test/test_set.py`
  - `Lib/test/test_tuple.py`
  - `Lib/test/test_str.py`
- [x] Close builtin constructor parity for `list(...)`, `tuple(...)`, `dict(...)`, `ord(...)`, and `chr(...)`.
- [x] Expand builtin call-shape parity for `sorted`, `reversed`, `enumerate`, `zip`, `map`, and `range`.
- [x] Add CPython-derived adopt/adapt/waive matrix and traceability rows for this wave.
- [x] Add a wave demo that proves the milestone surface works naturally from Python-shaped source.
- [x] Run local validation and record evidence.
- [x] Open PR, review, merge, and update this ledger with PR links and outcomes.

### `wave_psp_a2` Core Object Models and Builtin Semantics

Status: done

- [x] Harvest the required CPython test inputs for container and string object-model behavior.
- [x] Close `list`, `dict`, `set`, `tuple`, `str`, and `bytes` classification/object-model gaps.
- [x] Add adapted regression coverage and update parity ledgers.
- [x] Demo, validate, PR, review, merge.

### `wave_psp_b1` Collections Objects and Ordered Helpers

Status: done

- [x] Harvest `Lib/test/test_collections.py`, `Lib/test/test_bisect.py`, and `Lib/test/test_heapq.py`.
- [x] Close `collections`, `bisect`, and `heapq` constructor/object/call-shape gaps.
- [x] Add traceable regressions, demo, and local validation coverage for the closed surface.
- [x] Open PR, review, merge, and update this ledger with PR links and outcomes.

### `wave_psp_b2` Iterators, Functional Helpers, and Randomness

Status: done

- [x] Harvest `Lib/test/test_itertools.py`, `Lib/test/test_functools.py`, `Lib/test/test_operator.py`, `Lib/test/test_random.py`, and `Lib/test/test_secrets.py`.
- [x] Close iterator/object/callable parity for `itertools`, `functools`, `operator`, `random`, and `secrets`.
- [x] Add traceable regressions, demo, and traceability coverage for the closed/adapted surface.
- [x] Run local validation and record evidence.
- [x] Open implementation PR and merge the wave body.
- [x] Complete external review passes and update this ledger with outcomes.

### `wave_psp_c1` Structured Parsing and Serialization

Status: done

- [x] Harvest `Lib/test/test_json/`, `Lib/test/test_tomllib/`, `Lib/test/test_csv.py`, and `Lib/test/test_configparser.py`.
- [x] Close structured-return and class/export gaps for `json`, `tomllib`, `csv`, and `configparser`.
- [x] Add traceable regressions (including CPython subset fixtures), demo, and local validation evidence.
- [x] Open implementation PR and merge the wave body.
- [x] Complete external review passes and finalize ledger links.

### `wave_psp_c2` Text, Pattern, and Formatting Modules

Status: in_progress

- [x] Harvest the required CPython text-formatting test families.
- [x] Close `string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, and `calendar`.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_d1` Filesystem, Paths, and Archive Surfaces

Status: pending

- [ ] Harvest the required CPython filesystem/archive test families.
- [ ] Close `io`, `pathlib`, `glob`, `shutil`, `tempfile`, `gzip`, and `zipfile`.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_d2` Process, Runtime, and Platform Surfaces

Status: pending

- [ ] Harvest the required CPython runtime/platform test families.
- [ ] Close `os`, `env`, `sys`, `subprocess`, `logging`, `platform`, `time`, and `timeit`.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_e1` Strong-But-Incomplete Core Modules

Status: pending

- [ ] Harvest `Lib/test/test_datetime.py`, `Lib/test/test_re.py`, `Lib/test/test_math.py`, `Lib/test/test_statistics.py`, and `Lib/test/test_hashlib.py`.
- [ ] Close remaining parity gaps for `datetime`, `re`, `math`, `statistics`, and `hashlib`.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_e2` Class-Heavy and Custom Cleanup

Status: pending

- [ ] Harvest `Lib/test/test_argparse.py`, `Lib/test/test_ipaddress.py`, `Lib/test/test_uuid.py`, and `Lib/test/test_graphlib.py`.
- [ ] Close or explicitly classify final gaps for `argparse`, `ipaddress`, `uuid`, `graphlib`, and `test`.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `milestone_psp_7` Parity Governance and Exit Closure

Status: pending

- [ ] Publish canonical builtin parity inventory.
- [ ] Publish canonical core object-model parity inventory.
- [ ] Publish per-module closure inventory for all shipped `lib/sifr` modules.
- [ ] Publish CPython adopt/adapt/waive ledger and traceability matrix for every wave.
- [ ] Publish waiver index and final exit-gate closure summary.
- [ ] Align `internal_docs/architecture.md`, `internal_docs/roadmap.md`, phase docs, and public claims to the closed state.
- [ ] Run full validation, external reviewer passes, remediation loops, and closure notifications.

## Validation Evidence

### `wave_psp_a1`

- Implemented builtin constructor/call-shape closure in:
  - `crates/sifr_hir/src/lower/builtin_calls.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
- Added wave-specific regression/demo/traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a1_range_duplicate_stop_keyword.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a1_sorted_unexpected_keyword.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a1_map_callable_arity_mismatch.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a1_tuple_dynamic_list_shape.sifr`
  - `demos/wave_psp_a1_builtin_callable_surface_demo.sifr`
  - `verification/stdlib/wave_psp_a1_cpython_traceability.md`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_a1_builtin_callable_surface_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a1_range_duplicate_stop_keyword.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a1_sorted_unexpected_keyword.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a1_map_callable_arity_mismatch.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a1_tuple_dynamic_list_shape.sifr`
  - `cargo test -p sifr_hir test_sorted_accepts_iterable_keyword_and_key_none`
- Maintainability/lint validation:
  - `python3 scripts/check_hir_maintainability_guardrails.py`
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`
- Note:
  - The external script path requested in the phase instructions (`/Users/yaseralnajjar/work/sifr/codebase/scripts/run_all_tests.sh`) changes into a different checkout before running. The equivalent script inside this worktree was used for authoritative validation of the actual code under change.
- PR / merge:
  - Merged PR: `#1142` `Close wave_psp_a1 builtin constructor and call-shape parity`
  - Merge commit: `2879fcaa844367b4e4521f1daa68793292c28b76`

### `wave_psp_a2`

- Implemented core object-model argument normalization and parity closure in:
  - `crates/sifr_hir/src/lower/method_call_args.rs`
  - `crates/sifr_hir/src/lower/builtin_calls.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_hir/src/lower/mutating_methods.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/methods/common.rs`
  - `crates/sifr_codegen/src/methods/dict.rs`
  - `crates/sifr_codegen/src/methods/list.rs`
  - `crates/sifr_codegen/src/methods/mod.rs`
  - `crates/sifr_codegen/src/methods/string.rs`
- Added wave-specific regression, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_list_unexpected_keyword.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_dict_update_invalid_pairs.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_dict_get_duplicate_default.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_set_update_non_iterable.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_str_replace_invalid_count.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_tuple_index_invalid_bound.sifr`
  - `demos/wave_psp_a2_core_object_models_demo.sifr`
  - `verification/stdlib/wave_psp_a2_cpython_traceability.md`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_list_unexpected_keyword.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_dict_update_invalid_pairs.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_dict_get_duplicate_default.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_set_update_non_iterable.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_str_replace_invalid_count.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_tuple_index_invalid_bound.sifr`
  - `cargo test -p sifr_hir expressions_tests -- --nocapture`
  - `cargo test -p sifr_codegen methods -- --nocapture`
- Maintainability/lint validation:
  - `python3 scripts/check_hir_maintainability_guardrails.py`
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`
- CPython parity note:
  - `bytes` / `bytearray` remain explicitly classified as `unsupported` for this wave because Sifr still does not expose a first-class bytes primitive; the traceability ledger records the closure as an intentional object-model classification outcome rather than a fake module-parity claim.
- PR / merge:
  - Merged PR: `#1144` `Close wave_psp_a2 core object-model parity`
  - Merge commit: `d55a79285ee8aaaea8985c6fb1a9d0d0c4737f95`

### `wave_psp_b1`

- Implemented collections/object/call-shape closure in:
  - `lib/sifr/collections.sifr`
  - `lib/sifr/bisect.sifr`
  - `lib/sifr/heapq.sifr`
  - `crates/sifr_hir/src/lower/classes.rs`
  - `crates/sifr_hir/src/lower/compat_imports.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_hir/src/lower/expressions_tests.rs`
  - `crates/sifr_hir/src/lower/imported_defaults.rs`
  - `crates/sifr_hir/src/lower/method_call_args.rs`
  - `crates/sifr_hir/src/lower/mod.rs`
- Added wave-specific regression, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b1_bisect_key_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b1_counter_iterable_constructor_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b1_deque_index_invalid_bound.sifr`
  - `demos/wave_psp_b1_collections_ordered_helpers_demo.sifr`
  - `verification/stdlib/wave_psp_b1_cpython_traceability.md`
- Compatibility regressions updated to the new b1 semantics:
  - `crates/sifr/tests/e2e/pass/cpython_collections.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_collections_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_heapq_subset.sifr`
  - `crates/sifr/tests/e2e/pass/counter_dict_native.sifr`
  - `crates/sifr/tests/e2e/pass/generic_counter_custom_class.sifr`
  - `crates/sifr/tests/e2e/pass/phase31_constructor_compat.sifr`
  - `crates/sifr/tests/e2e/pass/phase31_defaultdict_len_deque_compat.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_collections_consolidated.sifr`
  - `crates/sifr/tests/e2e/fail/stdlib_counter_wrong_type.sifr`
  - `demos/m30_1d_collections_parity_demo/main.sifr`
  - `demos/milestone_borrow_stdlib_demo/borrow_stdlib_demo.sifr`
  - `demos/milestone_stdlib_generic_rewrite_demo.sifr`
  - `demos/phase31_defaultdict_compat_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_b1_collections_ordered_helpers_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_bisect.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_collections.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_heapq.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_heapq_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/counter_dict_native.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_counter_custom_class.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase31_constructor_compat.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase31_defaultdict_len_deque_compat.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_bisect_key_unsupported.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_counter_iterable_constructor_unsupported.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b1_deque_index_invalid_bound.sifr`
  - `cargo test -p sifr_hir expressions_tests -- --nocapture`
- Maintainability/lint validation:
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`
- CPython parity note:
  - The traceability ledger closes `Counter(dict)`, `Counter.most_common([n])`, deque ordered helpers, `bisect` / `insort` optional `lo` / `hi`, and mutating `heapreplace` / `heappushpop`; it explicitly records `bisect(key=...)`, iterable/kwargs `Counter(...)` constructors, and non-exported/private max-heap helper gaps as unsupported for this wave rather than pretending they were closed.
- PR / merge:
  - Merged PR: `#1149` `Close wave_psp_b1 collections ordered-helper parity`
  - Merge commit: `e9c051bcf7aca5dc005a2d8ecce9e9341fe002dd`

### `wave_psp_b2`

- Implemented iterator/function/randomness parity closure in:
  - `lib/sifr/itertools.sifr`
  - `lib/sifr/functools.sifr`
  - `lib/sifr/operator.sifr`
  - `lib/sifr/random.sifr`
  - `lib/sifr/secrets.sifr`
  - `crates/sifr_hir/src/lower/compat_imports.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_hir/src/lower/generic_inference.rs`
  - `crates/sifr_hir/src/lower/imported_defaults.rs`
  - `crates/sifr_hir/src/lower/imports.rs`
  - `crates/sifr_hir/src/lower/method_call_args.rs`
  - `crates/sifr_hir/src/lower/mod.rs`
  - `crates/sifr_hir/src/lower/typing_and_functions.rs`
  - `crates/sifr_codegen/src/function_emitter.rs`
  - `crates/sifr_codegen/src/lib.rs`
  - `crates/sifr_codegen/src/lib_support.rs`
  - `crates/sifr_codegen/src/lower_stmt.rs`
  - `crates/sifr_codegen/src/operator_protocol_emitters.rs`
  - `crates/sifr_driver/src/build/entrypoint.rs`
  - `crates/sifr_driver/src/project/exports.rs`
  - `crates/sifr_driver/src/project/frontend.rs`
  - `crates/sifr_driver/src/stdlib/bootstrap.rs`
- Added wave-specific regression, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`
  - `demos/wave_psp_b2_iterators_functional_randomness_demo.sifr`
  - `verification/stdlib/wave_psp_b2_cpython_traceability.md`
- Compatibility regressions updated to the new b2 semantics:
  - `crates/sifr/tests/e2e/pass/cpython_itertools.sifr`
  - `crates/sifr/tests/e2e/pass/generic_shuffle_str.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_functools.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_operator.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_random_new.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_secrets.sifr`
  - `demos/milestone_stdlib_generic_rewrite_demo.sifr`
  - `demos/milestone_stdlib_pure_expansion_demo.sifr`
  - `demos/milestone_stdlib_remediation_demo.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_b2_iterators_functional_randomness_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_itertools.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_chain_float.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_chain_str.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/generic_shuffle_str.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/itertools_chain_own.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_functools.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_operator.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_random_new.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_secrets.sifr`
  - `cargo test -p sifr_hir expressions_tests -- --nocapture`
- Maintainability/lint validation:
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
  - `python3 scripts/check_hir_maintainability_guardrails.py`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`
- CPython parity note:
  - The traceability ledger records eager list-backed iterator helpers as `adapted`, preserves the new direct `__call__`/imported-vararg root-cause compiler closure, and explicitly classifies `functools.partial`, cache/decorator families, and callable-object use inside higher-order stdlib helpers as unsupported rather than claiming fake parity.
- PR / merge:
  - Merged PR: `#1160` `Close wave_psp_b2 iterator and randomness parity`
  - Merge commit: `07a4c6f9d61d6aadc89b2873fff57444c4738a12`

### `wave_psp_c1`

- Implemented structured parsing/serialization closure in:
  - `lib/sifr/json.sifr`
  - `lib/sifr/tomllib.sifr`
  - `lib/sifr/csv.sifr`
  - `lib/sifr/configparser.sifr`
  - `crates/sifr_hir/src/stdlib/io_json.rs`
  - `crates/sifr_hir/src/stdlib/platform_misc.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_codegen/src/intrinsics/json.rs`
  - `crates/sifr_codegen/src/intrinsics/toml.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/expr_render_helpers.rs`
  - `crates/sifr_codegen/src/stmt_support_emitter.rs`
  - `crates/sifr_codegen/src/helpers.rs`
  - `crates/sifr_codegen/src/hir_analysis/queries.rs`
  - `crates/sifr_codegen/src/lib.rs`
- Added wave-specific regression/demo/traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_c1_structured_parsing_serialization.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr`
  - `demos/wave_psp_c1_structured_parsing_serialization_demo.sifr`
  - `verification/stdlib/wave_psp_c1_cpython_traceability.md`
- Targeted validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_c1_structured_parsing_serialization_demo.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_c1_structured_parsing_serialization.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_configparser.sifr`
  - `cargo test -p sifr_codegen test_class_method_mutable_self_propagates_through_delegation`
- Authoritative local gate:
  - `SIFR_E2E_DISABLE_CACHE=1 scripts/run_all_tests.sh --profile quick`
  - Includes phase-29 hardening summary: `verification ok: variants=64, failures=0, blocking_failures=0, non_blocking_failures=0`
- PR / merge:
  - Merged PR: `#1168` `Close wave_psp_c1 structured parsing and serialization parity`
  - Merge commit: `582697f0b869e07bbaf3ac672e3f1dd87ddd04e6`
  - Reviewer remediation PR: `#1174` `wave_psp_c1: fix reviewer pass1 configparser read + delegated mut self`
  - Merge commit: `a5c6fb7ec1916795dc9408d00cd4b32ff9308271`

### `wave_psp_c2`

- Implemented text/pattern/formatting parity closure in:
  - `lib/sifr/string.sifr`
  - `lib/sifr/textwrap.sifr`
  - `lib/sifr/fnmatch.sifr`
  - `lib/sifr/difflib.sifr`
  - `lib/sifr/calendar.sifr`
  - `lib/sifr/base64.sifr`
- Added wave-specific regression, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_c2_text_pattern_formatting.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_string_template_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_fnmatch_translate_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_difflib_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_calendar_subset.sifr`
  - `demos/wave_psp_c2_text_pattern_formatting_demo.sifr`
  - `verification/stdlib/wave_psp_c2_cpython_traceability.md`
- Targeted validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_c2_text_pattern_formatting_demo.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_c2_text_pattern_formatting.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_string_template_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_fnmatch_translate_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_difflib_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_calendar_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_string.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_string_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_textwrap.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_textwrap_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_base64_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_base64_strictness_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_fnmatch.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_fnmatch_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_fnmatch_consolidated.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_difflib.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_calendar.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_html.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_base64_intrinsics.sifr`
- Authoritative local gate:
  - `SIFR_E2E_DISABLE_CACHE=1 scripts/run_all_tests.sh --profile quick`

### `wave_psp_d1`

- Pending.

### `wave_psp_d2`

- Pending.

### `wave_psp_e1`

- Pending.

### `wave_psp_e2`

- Pending.

### `milestone_psp_7`

- Pending.

## PR Ledger

- `wave_psp_a1`: PR `#1142` merged at `2026-03-14T17:28:40Z`
- `wave_psp_a2`: PR `#1144` merged at `2026-03-14T18:24:24Z`
- `wave_psp_b1`: PR `#1149` merged at `2026-03-15T02:23:59Z`
- `wave_psp_b2`: PR `#1160` merged at `2026-03-15T11:41:51Z`
- `wave_psp_c1`: PR `#1168` merged at `2026-03-16T01:34:09Z`
- `wave_psp_c1-review-pass1`: PR `#1174` merged at `2026-03-16T02:26:34Z`

## External Review Ledger

- `wave_psp_a1` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a1-review-pass1.md`
  - Validated finding: duplicate `range(stop=...)` positional/keyword collision was accepted when the one-positional form normalized too late in builtin lowering.
  - Fix status: merged via PR `#1150`.
- `wave_psp_a1` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a1-review-pass2.md`
  - Validation result: no new actionable finding. The repeated `range(10, stop=20)` bug claim was invalid on the post-`#1150` mainline, and the recommendation to reject all `range(...)` keywords conflicts with the wave's documented `adapted` parity contract in `verification/stdlib/wave_psp_a1_cpython_traceability.md`.
  - Fix status: no code changes required.
- `wave_psp_a2` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a2-review-pass1.md`
  - Validation result: approved with no actionable implementation issue. The only noted verification-hardening interruption was an environment-level disk-space concern, not a wave-specific regression.
  - Fix status: no code changes required.
- `wave_psp_a2` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a2-review-pass2.md`
  - Validated finding: the wave traceability doc did not explicitly call out that `list.index(start=/stop=)`, `tuple.index(start=)`, `dict.pop(default=)`, and `dict.get(default=)` are intentional keyword-binding adaptations over CPython's positional-only API.
  - Fix status: documentation tightened in `verification/stdlib/wave_psp_a2_cpython_traceability.md`; code behavior unchanged.
- `wave_psp_b1` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b1-review-pass1.md`
  - Validation result: approved with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_b1` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b1-review-pass2.md`
  - Validation result: approved as production-ready with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_b2` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-pass1.md`
  - Validation result: non-actionable stale review. The notes described the pre-implementation state and incorrectly claimed the merged b2 artifacts, traceability ledger, demo, and PR were absent from current `main`.
  - Fix status: no code changes required.
- `wave_psp_b2` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-pass2.md`
  - Validated finding: borrowed aggregate arguments were still packing move-only names into temporary collection literals, which regressed `chain(a, b)` / `chain(a, b, c)` style ownership semantics after the b2 vararg expansion.
  - Fix status: remediated in PR `#1162`.
- `wave_psp_c1` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c1-review-pass1b.md`
  - Validated findings: `ConfigParser.read(path)` only read bytes and skipped parser population, `has_option()` default fallback semantics regressed for existing sections, and class-method mutability inference missed delegated `self.read_string(...)` mutation.
  - Fix status: remediated in PR `#1174`.
- `wave_psp_c1` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c1-review-pass2.md`
  - Validation result: approved as production-ready with no actionable implementation issue.
  - Non-actionable stale note: the review text still mentions `ConfigParser.read()` not populating parser state, which was true before PR `#1174` and is now invalid on current mainline.
  - Fix status: no code changes required.
