# Ad Hoc Phase Execution: Python Source Parity and Builtin Stdlib Surface Closure

Status: complete
Started: 2026-03-14
Phase owner: Codex (GPT-5)
Source phase: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
Current active wave: `none (phase closed)`

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
- [x] `milestone_psp_4` / `wave_psp_c2`: text, pattern, and formatting modules
- [x] `milestone_psp_5` / `wave_psp_d1`: filesystem, paths, and archive surfaces
- [x] `milestone_psp_5` / `wave_psp_d2`: process, runtime, and platform surfaces
- [x] `milestone_psp_6` / `wave_psp_e1`: strong-but-incomplete core modules
- [x] `milestone_psp_6` / `wave_psp_e2`: class-heavy and custom cleanup
- [x] `milestone_psp_7`: parity governance and exit closure

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

Status: done

- [x] Harvest the required CPython text-formatting test families.
- [x] Close `string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, and `calendar`.
- [x] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_d1` Filesystem, Paths, and Archive Surfaces

Status: done

- [x] Harvest the required CPython filesystem/archive test families.
- [x] Close `io`, `pathlib`, `glob`, `shutil`, `tempfile`, `gzip`, and `zipfile`.
- [x] Add traceable regressions, demo, validate, PR, review, merge.
- [x] Build per-module adopt/adapt/waive map against:
  - `Lib/test/test_io/`
  - `Lib/test/test_pathlib/`
  - `Lib/test/test_glob.py`
  - `Lib/test/test_shutil.py`
  - `Lib/test/test_tempfile.py`
  - `Lib/test/test_gzip.py`
  - `Lib/test/test_zipfile/`
- [x] Close `io` object/call-shape gaps:
  - context-manager lifecycle parity for file handles
  - binary/text mode coverage and CPython-shaped helper entry points
- [x] Close `pathlib` class-family gaps:
  - ensure `Path` methods align with CPython naming and return-shape expectations
  - tighten path-mutation helper semantics (`with_name`, `with_suffix`, join/parent/name/suffix/stem)
- [x] Close `glob` parity gaps:
  - recursive matching behavior and predictable hidden-file handling rules
  - align pattern semantics with shipped `fnmatch` adaptations
- [x] Close `shutil` helper surface gaps:
  - CPython-shaped copy/move/tree helpers and disk/tooling helpers
  - explicit error semantics for missing inputs and tree cleanup
- [x] Close `tempfile` lifecycle and naming gaps:
  - temp path generation semantics and creation helpers (`mkstemp`, `mkdtemp`)
  - collision/error behavior under missing parent and race-like conditions
- [x] Close `gzip` and `zipfile` class/entry gaps:
  - gzip codec surface parity that is compatible with Sifr string/bytes policy
  - zip archive class/object model and helper methods with deterministic behavior

### `wave_psp_d2` Process, Runtime, and Platform Surfaces

Status: done

- [x] Harvest the required CPython runtime/platform test families.
- [x] Close `os`, `env`, `sys`, `subprocess`, `logging`, `platform`, `time`, and `timeit`.
- [x] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_e1` Strong-But-Incomplete Core Modules

Status: done

- [x] Harvest `Lib/test/test_datetime.py`, `Lib/test/test_re.py`, `Lib/test/test_math.py`, `Lib/test/test_statistics.py`, and `Lib/test/test_hashlib.py`.
- [x] Close remaining parity gaps for `datetime`, `re`, `math`, `statistics`, and `hashlib`.
- [x] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_e2` Class-Heavy and Custom Cleanup

Status: done

- [x] Harvest `Lib/test/test_argparse.py`, `Lib/test/test_ipaddress.py`, `Lib/test/test_uuid.py`, and `Lib/test/test_graphlib.py`.
- [x] Close or explicitly classify final gaps for `argparse`, `ipaddress`, `uuid`, `graphlib`, and `test`.
- [x] Add traceable regressions, demo, validate, PR, review, merge.

### `milestone_psp_7` Parity Governance and Exit Closure

Status: done

- [x] Publish canonical builtin parity inventory.
- [x] Publish canonical core object-model parity inventory.
- [x] Publish per-module closure inventory for all shipped `lib/sifr` modules.
- [x] Publish CPython adopt/adapt/waive ledger and traceability matrix for every wave.
- [x] Publish waiver index and final exit-gate closure summary.
- [x] Align `internal_docs/architecture.md`, `internal_docs/roadmap.md`, phase docs, and public claims to the closed state.
- [x] Run full validation, external reviewer passes, remediation loops, and closure notifications.

## Validation Evidence

### `wave_psp_a1`

- Implemented builtin constructor/call-shape closure in:
  - `crates/sifr_hir/src/lower/builtin_calls.rs`
  - `crates/sifr_hir/src/lower/expressions.rs`
  - `crates/sifr_codegen/src/intrinsic_method_emitters.rs`
  - `crates/sifr_codegen/src/lower_expr.rs`
- Added wave-specific regression/demo/traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_builtins_subset.sifr`
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
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_builtins_subset.sifr`
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
  - `crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_list_unexpected_keyword.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_dict_update_invalid_pairs.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_dict_get_duplicate_default.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_dict_setdefault_invalid_default.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_set_update_non_iterable.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_str_replace_invalid_count.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a2_tuple_index_invalid_bound.sifr`
  - `demos/wave_psp_a2_core_object_models_demo.sifr`
  - `verification/stdlib/wave_psp_a2_cpython_traceability.md`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_list_unexpected_keyword.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_dict_update_invalid_pairs.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_dict_get_duplicate_default.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_dict_setdefault_invalid_default.sifr`
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
- Post-closure parity hardening from A/B reviewer cycle:
  - `crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_random_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_secrets_subset.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_functools_partial_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_itertools_starmap_non_binary_callable.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_operator_attrgetter_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_operator_methodcaller_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_random_choices_weights_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_b2_secrets_token_urlsafe_unsupported.sifr`
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
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_itertools_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_functools.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_operator.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_random_new.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_secrets.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_random_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_secrets_subset.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_b2_itertools_starmap_non_binary_callable.sifr`
  - `cargo test -p sifr test_e2e_fail -- --nocapture`
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
  - `crates/sifr/tests/e2e/pass/cpython_json_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_csv_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr`
  - `demos/wave_psp_c1_structured_parsing_serialization_demo.sifr`
  - `verification/stdlib/wave_psp_c1_cpython_traceability.md`
- Targeted validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_c1_structured_parsing_serialization_demo.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_json_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_csv_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_tomllib_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_configparser_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_c1_structured_parsing_serialization.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_configparser.sifr`
  - `cargo test -p sifr_codegen test_class_method_mutable_self_propagates_through_delegation`
- Authoritative local gate:
  - `SIFR_E2E_DISABLE_CACHE=1 scripts/run_all_tests.sh --profile quick`
  - Re-run after reviewer-pass1 remediation merged: `SIFR_E2E_DISABLE_CACHE=1 scripts/run_all_tests.sh --profile quick`
- PR / merge:
  - Merged PR: `#1182` `Close wave_psp_c2 text pattern and formatting parity`
  - Merge commit: `030bc9053b9bfbb598db7267934a567665ba7924`
  - Reviewer remediation PR: `#1187` `wave_psp_c2: fix reviewer-pass parity gaps in difflib/textwrap`
  - Merge commit: `4ad100ee85d32791822dfd48ce09fcdddb04d306`
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
  - `crates/sifr/tests/e2e/fail/phase_psp_c2_difflib_sequence_matcher_isjunk_unsupported.sifr`
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
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_c2_difflib_sequence_matcher_isjunk_unsupported.sifr`
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

- Added wave-specific regression, fail guard, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_d1_filesystem_paths_archives.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_gzip_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d1_io_open_non_string_mode.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d1_glob_non_string_pattern.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d1_shutil_copy_non_string_path.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d1_zipfile_write_non_string_content.sifr`
  - `demos/wave_psp_d1_filesystem_paths_archives_demo.sifr`
  - `verification/stdlib/wave_psp_d1_cpython_traceability.md`
- Post-closure parity hardening from d1 review pass4:
  - `lib/sifr/pathlib.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_pathlib_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_io_subset.sifr`
  - `verification/stdlib/wave_psp_d1_cpython_traceability.md`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_d1_filesystem_paths_archives.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_io_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_pathlib_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_glob_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_shutil_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_tempfile_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_gzip_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr`
  - `cargo run -q -p sifr -- run demos/wave_psp_d1_filesystem_paths_archives_demo.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d1_io_open_non_string_mode.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d1_glob_non_string_pattern.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d1_shutil_copy_non_string_path.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d1_zipfile_write_non_string_content.sifr`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`

### `wave_psp_d2`

- Added wave-specific regression, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_d2_process_runtime_platform.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_sys_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_subprocess_subset.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d2_os_mkdir_non_string_path.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d2_subprocess_non_string_cmd.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d2_sys_exit_non_int_code.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_d2_timeit_non_callable_stmt.sifr`
  - `demos/wave_psp_d2_process_runtime_platform_demo.sifr`
  - `verification/stdlib/wave_psp_d2_cpython_traceability.md`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_d2_process_runtime_platform_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_d2_process_runtime_platform.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_sys_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_subprocess_subset.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d2_os_mkdir_non_string_path.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d2_subprocess_non_string_cmd.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d2_sys_exit_non_int_code.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d2_timeit_non_callable_stmt.sifr`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`

### `wave_psp_e1`

- Added wave-specific regression, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_e1_core_modules_numeric_patterns_crypto.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e1_datetime_from_timestamp_non_float.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e1_re_search_non_string_pattern.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e1_math_isclose_non_float_tol.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e1_statistics_mean_non_float_list.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e1_hashlib_new_non_string_name.sifr`
  - `demos/wave_psp_e1_strong_core_modules_demo.sifr`
  - `verification/stdlib/wave_psp_e1_cpython_traceability.md`
- CPython-derived regression fixtures exercised for this wave:
  - `crates/sifr/tests/e2e/pass/cpython_datetime_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_re_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_math_missing_surface_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_math_semantic_corrections_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_e1_strong_core_modules_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_e1_core_modules_numeric_patterns_crypto.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e1_datetime_from_timestamp_non_float.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e1_re_search_non_string_pattern.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e1_math_isclose_non_float_tol.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e1_statistics_mean_non_float_list.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e1_hashlib_new_non_string_name.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_datetime_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_re_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_math_missing_surface_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_math_semantic_corrections_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_hashlib_object_model_subset.sifr`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`

### `wave_psp_e2`

- Implemented class/object-model parity upgrades in:
  - `lib/sifr/argparse.sifr`
  - `lib/sifr/ipaddress.sifr`
  - `lib/sifr/uuid.sifr`
  - `lib/sifr/graphlib.sifr`
- Post-closure gap hardening (CPython-derived) in this same wave:
  - `argparse`: added `--name=value` token-shape support and `--` positional passthrough in `parse_option`, `parse_positional`, and `ArgumentParser.parse_args`.
  - `argparse`: hardened pending-option parsing so option-like tokens (`--...`) are no longer consumed as values for an option that is missing its argument; defaults now remain stable under this adapted flow.
  - `ipaddress`: added CPython-style leading-zero rejection for IPv4 segments while preserving typed factory errors via `ip_address` / `ipv4_address`.
  - `ipaddress`: aligned IPv4 special-range classification with CPython-adapted semantics for `is_private` / `is_global` (including `100.64.0.0/10` and `192.0.0.9/.10` exception handling) and added `is_link_local` / `is_reserved` coverage on both function and class surfaces.
  - `uuid`: extended `uuid_from_hex` normalization to accept canonical, hyphenated, `urn:uuid:...`, and `{...}` forms.
  - `graphlib`: fixed sparse-node ordering by filtering static/incremental order to explicitly added nodes (no undeclared intermediary node leakage).
- Added wave-specific regression, demo, and traceability artifacts:
  - `crates/sifr/tests/e2e/pass/phase_psp_e2_class_heavy_custom_cleanup.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_argparse_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_ipaddress_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_graphlib_subset.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e2_argparse_parse_args_non_string_list.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e2_ip_address_non_string.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e2_graphlib_add_non_int_predecessor.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_e2_uuid_from_hex_non_string.sifr`
  - `demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr`
  - `verification/stdlib/wave_psp_e2_cpython_traceability.md`
- CPython-derived regression fixtures exercised for this wave:
  - `crates/sifr/tests/e2e/pass/cpython_argparse_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_ipaddress_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_uuid_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_graphlib_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_unittest_assertions_subset.sifr`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_e2_class_heavy_custom_cleanup_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_e2_class_heavy_custom_cleanup.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_argparse_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_ipaddress_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_uuid_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_graphlib_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_unittest_assertions_subset.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_argparse.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_ipaddress.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_ipaddress_extended.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_uuid_consolidated.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_graphlib.sifr`
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_graphlib_class.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e2_argparse_parse_args_non_string_list.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e2_ip_address_non_string.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e2_graphlib_add_non_int_predecessor.sifr`
  - `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_e2_uuid_from_hex_non_string.sifr`
- Authoritative local gate:
  - `scripts/run_all_tests.sh --profile quick`

### `milestone_psp_7`

- Added canonical parity-governance inventory artifact:
  - `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`
- Inventory now covers:
  - canonical builtin parity inventory
  - canonical core object-model inventory
  - per-module closure inventory for all shipped `lib/sifr` modules
  - per-wave CPython adopt/adapt/waive ledger links
  - waiver index and exit-gate closure summary
- Validation evidence:
  - `scripts/run_all_tests.sh --profile quick`
  - `scripts/run_all_tests.sh`

## PR Ledger

- `wave_psp_a1`: PR `#1142` merged at `2026-03-14T17:28:40Z`
- `wave_psp_a2`: PR `#1144` merged at `2026-03-14T18:24:24Z`
- `wave_psp_b1`: PR `#1149` merged at `2026-03-15T02:23:59Z`
- `wave_psp_b2`: PR `#1160` merged at `2026-03-15T11:41:51Z`
- `wave_psp_c1`: PR `#1168` merged at `2026-03-16T01:34:09Z`
- `wave_psp_c1-review-pass1`: PR `#1174` merged at `2026-03-16T02:26:34Z`
- `wave_psp_c2`: PR `#1182` merged at `2026-03-16T03:11:07Z`
- `wave_psp_c2-review-pass1`: PR `#1187` merged at `2026-03-16T03:39:09Z`
- `wave_psp_ab-review-pass1`: PR `#1190` merged at `2026-03-16T04:49:13Z`
- `wave_psp_d1`: PR `#1192` merged at `2026-03-16T05:15:54Z`
- `wave_psp_d1-review-pass1`: PR `#1193` merged at `2026-03-16T05:28:23Z`
- `wave_psp_ab-review-pass3`: PR `#1196` merged at `2026-03-16T06:12:23Z`
- `wave_psp_ab-review-pass4`: PR `#1197` merged at `2026-03-16T06:38:54Z`
- `wave_psp_d2`: PR `#1198` merged at `2026-03-16T08:10:57Z`
- `wave_psp_d2-review-pass1`: PR `#1199` merged at `2026-03-16T08:22:10Z`
- `wave_psp_d2-review-pass2`: PR `#1200` merged at `2026-03-16T08:36:31Z`
- `wave_psp_e1`: PR `#1201` merged at `2026-03-16T08:43:54Z`
- `wave_psp_e1-review-pass1`: PR `#1202` merged at `2026-03-16T09:01:04Z`
- `wave_psp_e2`: PR `#1205` merged at `2026-03-16T09:39:48Z`
- `wave_psp_e2-review-pass1`: PR `#1206` merged at `2026-03-16T09:50:26Z`
- `wave_psp_a2-review-pass3`: PR `#1220` merged at `2026-03-17T11:32:16Z`
- `wave_psp_a2-review-pass4`: PR `#1221` merged at `2026-03-17T12:01:41Z`
- `wave_psp_a2-review-pass5`: PR `#1222` merged at `2026-03-17T13:03:15Z`
- `wave_psp_b2-review-pass6`: PR `#1223` merged at `2026-03-17T13:15:23Z`
- `wave-ledger-sync-recent-review-prs`: PR `#1224` merged at `2026-03-17T13:20:15Z`
- `wave_psp_c1-review-pass5`: PR `#1225` merged at `2026-03-17T13:28:05Z`
- `wave_psp_c2-review-pass5`: PR `#1226` merged at `2026-03-17T13:42:38Z`
- `wave_psp_d1-review-pass5`: PR `#1227` merged at `2026-03-17T13:56:31Z`
- `wave_psp_d2-review-pass4`: PR `#1228` merged at `2026-03-17T14:08:04Z`
- `wave_psp_e1-review-pass4`: PR `#1229` merged at `2026-03-17T14:21:44Z`
- `wave_psp_e2-review-pass8`: PR `#1230` merged at `2026-03-17T14:34:16Z`
- `wave_psp_d1-review-pass6`: PR `#1231` merged at `2026-03-17T14:46:27Z`
- `wave_psp_d2-review-pass5`: PR `#1232` merged at `2026-03-17T14:54:01Z`
- `wave_psp_e2-review-pass9`: PR `#1233` merged at `2026-03-17T15:03:44Z`
- `wave-ledger-sync-through-1233`: PR `#1234` merged at `2026-03-17T15:04:56Z`
- `milestone_psp_7-production-grade-remediation-r2`: PR `#1235` merged at `2026-03-17T16:17:31Z`

## External Review Ledger

- `wave_psp_a1` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a1-review-pass1.md`
  - Validated finding: duplicate `range(stop=...)` positional/keyword collision was accepted when the one-positional form normalized too late in builtin lowering.
  - Fix status: merged via PR `#1150`.
- `wave_psp_a1` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a1-review-pass2.md`
  - Validation result: no new actionable finding. The repeated `range(10, stop=20)` bug claim was invalid on the post-`#1150` mainline, and the recommendation to reject all `range(...)` keywords conflicts with the wave's documented `adapted` parity contract in `verification/stdlib/wave_psp_a1_cpython_traceability.md`.
  - Fix status: no code changes required.
- `wave_psp_a1` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-a1-review-gap-cpython-parity-20260317-r2.md`
  - Validation result: non-actionable stale finding. The report asks to reject all `range(...)` keyword forms, but wave `a1` intentionally classifies keyword `range(start=..., stop=..., step=...)` as `adapted` parity, with executable evidence in `phase_psp_a1_builtin_callable_surface.sifr` and `cpython_builtins_subset.sifr`.
  - Fix status: no code changes required; behavior and traceability remain aligned.
- `wave_psp_a1` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-a1-review-gap-cpython-parity-20260317-r3.md`
  - Validation result: non-actionable stale finding. The report repeats the same recommendation to reject all `range(...)` keyword forms, which conflicts with the wave's explicit `adapted` parity contract and shipped CPython-derived fixtures.
  - Fix status: no code changes required; traceability and executable evidence remain aligned.
- `wave_psp_a1` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a1-review-gap-cpython-parity-20260317-r4.md`
  - Validation result: reviewer satisfied with no actionable findings.
  - Fix status: no code changes required.
- `wave_psp_a2` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a2-review-pass1.md`
  - Validation result: approved with no actionable implementation issue. The only noted verification-hardening interruption was an environment-level disk-space concern, not a wave-specific regression.
  - Fix status: no code changes required.
- `wave_psp_a2` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a2-review-pass2.md`
  - Validated finding: the wave traceability doc did not explicitly call out that `list.index(start=/stop=)`, `tuple.index(start=)`, `dict.pop(default=)`, and `dict.get(default=)` are intentional keyword-binding adaptations over CPython's positional-only API.
  - Fix status: documentation tightened in `verification/stdlib/wave_psp_a2_cpython_traceability.md`; code behavior unchanged.
- `wave_psp_a2` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-a2-review-gap-cpython-parity-20260317-r1.md`
  - Validation result: partially actionable. The reported mutable-set method failures were stale on current mainline, but the `dict.setdefault(key, default)` parity gap was valid for the shipped a2 surface.
  - Fix status: remediated by adding `dict.setdefault` lowering/type checks (`crates/sifr_hir/src/lower/{method_call_args,expressions,mutating_methods}.rs` + `crates/sifr_codegen/src/methods/{dict,mod}.rs`), adding fail guard `phase_psp_a2_dict_setdefault_invalid_default.sifr`, expanding CPython-derived/pass coverage for `setdefault` + set mutating update families in `phase_psp_a2_core_object_model_surface.sifr` and `cpython_core_object_model_subset.sifr`, and tightening `verification/stdlib/wave_psp_a2_cpython_traceability.md`.
- `wave_psp_a2` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-a2-review-gap-cpython-parity-20260317-r2.md`
  - Validated finding: local-variable mutability inference missed `dict.setdefault` and set `_update` mutators when they were the first mutating call on a binding, which produced non-`mut` Rust locals and compile failures.
  - Fix status: remediated at root cause by extending the canonical codegen mutator registry in `crates/sifr_codegen/src/hir_analysis/queries.rs` (which feeds `collect_mutated_vars`) to include `setdefault` and set `_update` mutators, adding unit coverage for the new mutator detection paths, and strengthening CPython-derived pass fixtures with first-mutation local-binding assertions.
- `wave_psp_a2` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/work/sifr/codebase/reviews/wave-psp-a2-review-gap-cpython-parity-20260317-r3.md`
  - Validation result: reviewer satisfied with no actionable findings.
  - Fix status: no code changes required.
- `wave_psp_b1` review pass 1:
  - Reviewer file: historical artifact no longer present in the current workspace (`reviews/wave-psp-b1-review-pass1.md` was removed during later workspace cleanup); validated outcome retained in this ledger.
  - Validation result: approved with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_b1` review pass 2:
  - Reviewer file: historical artifact no longer present in the current workspace (`reviews/wave-psp-b1-review-pass2.md` was removed during later workspace cleanup); validated outcome retained in this ledger.
  - Validation result: approved as production-ready with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_b1` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b1-review-gap-cpython-parity-20260316.md`
  - Validated findings: `heapq.merge` had no regression coverage and compiled in `check` mode but failed in `run` mode due invalid option/comparison lowering in `lib/sifr/heapq.sifr`; parity documentation under-described the intentional `defaultdict` compat-lowering model, and waiver enforcement for `Counter(**kwargs)` was missing.
  - Fix status: remediated by hardening `lib/sifr/heapq.sifr::merge`, adding CPython-derived merge assertions in `crates/sifr/tests/e2e/pass/cpython_heapq*.sifr` and `phase_psp_b1_collections_ordered_helpers.sifr`, adding `phase_psp_b1_defaultdict_keyword_constructor_unsupported.sifr` and `phase_psp_b1_counter_kwargs_constructor_unsupported.sifr`, extending `Counter.get(key[, default])` parity in `lib/sifr/collections.sifr`, and tightening `verification/stdlib/wave_psp_b1_cpython_traceability.md`.
- `wave_psp_b1` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b1-review-gap-cpython-parity-20260316-r2.md`
  - Validation result: approved for closure. The reviewer-confirmed r1 follow-ups (`Counter.get` default parameter and `Counter(**kwargs)` waiver enforcement) are now present, and remaining deque gaps are intentional/adapted differences.
  - Fix status: no additional code changes required beyond pass-3 remediation; documentation remains aligned with current behavior.
- `wave_psp_b2` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-pass1.md`
  - Validation result: non-actionable stale review. The notes described the pre-implementation state and incorrectly claimed the merged b2 artifacts, traceability ledger, demo, and PR were absent from current `main`.
  - Fix status: no code changes required.
- `wave_psp_b2` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-pass2.md`
  - Validated finding: borrowed aggregate arguments were still packing move-only names into temporary collection literals, which regressed `chain(a, b)` / `chain(a, b, c)` style ownership semantics after the b2 vararg expansion.
  - Fix status: remediated in PR `#1162`.
- `wave_psp_b2` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-gap-cpython-parity-20260316.md`
  - Validated findings: CPython-derived coverage for itertools combinator families was too concentrated in the wave fixture, `accumulate(..., initial=...)` parity was missing, operator helper coverage omitted shipped boolean helpers, and `compare_digest` constant-time limitations needed clearer traceability wording.
  - Fix status: remediated by adding `accumulate(initial=...)` in `lib/sifr/itertools.sifr`, expanding `cpython_itertools_subset.sifr` with product/permutations/combinations/combinations_with_replacement/starmap coverage, extending `stdlib_operator.sifr` boolean-helper assertions, and tightening `verification/stdlib/wave_psp_b2_cpython_traceability.md`.
- `wave_psp_b2` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-gap-cpython-parity-20260316-r2.md`
  - Validation result: partially stale. The review repeated already-remediated b2 items (itertools combinator coverage and operator helper tests) but raised two useful follow-ups: remove internal-API dependency in `stdlib_random.sifr` and make `compare_digest` timing-safety non-claim explicit in waivers.
  - Fix status: remediated by rewriting `crates/sifr/tests/e2e/pass/stdlib_random.sifr` against the public `sifr.random` API and tightening `verification/stdlib/wave_psp_b2_cpython_traceability.md` to classify constant-time `compare_digest` guarantees as unsupported for this wave.
- `wave_psp_b2` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-gap-cpython-parity-20260317-r1.md`
  - Validation result: partially actionable. Core implementation was correct, but parity evidence needed stronger coverage for documented edge paths and an explicit compile-time guard around the typed `starmap` arity adaptation.
  - Fix status: remediated by expanding `cpython_itertools_subset.sifr` with negative edge assertions (`repeat<0`, oversized combinator `r`, empty-data replacement combos, and non-positive `islice` step), expanding `cpython_random_subset.sifr` with shipped helper coverage (`random`, `randint`, `uniform`, `gauss`, `sample` + invalid sample guard), adding fail fixture `phase_psp_b2_itertools_starmap_non_binary_callable.sifr`, and tightening `verification/stdlib/wave_psp_b2_cpython_traceability.md` to include explicit `itemgetter`/random surface coverage and the intentional binary-only `starmap` contract.
- `wave_psp_b2` review pass 6:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b2-review-gap-cpython-parity-20260317-r3.md`
  - Validation result: no new implementation gaps; reviewer flagged only low-severity documentation clarity follow-ups for already-shipped intentional adaptations.
  - Fix status: remediated by adding explicit source-level notes in `lib/sifr/operator.sifr` (`getitem` safe-indexing `None` behavior and list-only `truth` surface), clarifying `lib/sifr/secrets.sifr::compare_digest` as non-constant-time, and tightening `verification/stdlib/wave_psp_b2_cpython_traceability.md` wording for these intentional differences.
- `wave_psp_c1` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c1-review-pass1b.md`
  - Validated findings: `ConfigParser.read(path)` only read bytes and skipped parser population, `has_option()` default fallback semantics regressed for existing sections, and class-method mutability inference missed delegated `self.read_string(...)` mutation.
  - Fix status: remediated in PR `#1174`.
- `wave_psp_c1` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c1-review-pass2.md`
  - Validation result: approved as production-ready with no actionable implementation issue.
  - Non-actionable stale note: the review text still mentions `ConfigParser.read()` not populating parser state, which was true before PR `#1174` and is now invalid on current mainline.
  - Fix status: no code changes required.
- `wave_psp_c1` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c1-review-gap-cpython-parity-20260316.md`
  - Validation result: partially actionable. The reported `ConfigParser.has_option()` bug is invalid against CPython behavior (`DEFAULT` options are visible to concrete sections), while the request to explicitly classify `json.dumps` encode-error semantics and clean minor CSV no-op assignment was valid.
  - Fix status: remediated by classifying `json.dumps` encode-error propagation as an intentional C1 divergence in `verification/stdlib/wave_psp_c1_cpython_traceability.md` and removing the redundant `DictWriter.writeheader()` reassignment in `lib/sifr/csv.sifr`.
- `wave_psp_c1` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c1-review-gap-cpython-parity-20260317-r1.md`
  - Validation result: partially actionable on documentation fidelity. Runtime behavior remained correct, but traceability wording needed tighter subset-scoped coverage framing to avoid overstating CPython-family depth.
  - Fix status: remediated by tightening `verification/stdlib/wave_psp_c1_cpython_traceability.md` (explicit subset-coverage scope note, explicit `cpython_json_subset.sifr` evidence mapping, and explicit TOML decode-position `intentional-diff` classification).
- `wave_psp_c1` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c1-review-gap-cpython-parity-20260317-r3.md`
  - Validation result: approved with no actionable implementation or parity gaps; the previously flagged `ConfigParser.has_option()` claim was explicitly corrected as non-bug and aligned with CPython behavior.
  - Fix status: no code changes required.
- `wave_psp_c2` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c2-review-pass1.md`
  - Validated findings: `SequenceMatcher.get_matching_blocks()` only returned the longest substring block, `SequenceMatcher.ratio()` used character-presence instead of block-based matching, and `TextWrapper` width did not account for line indentation width.
  - Fix status: remediated in PR `#1187` with strengthened CPython-derived regression assertions for difflib/textwrap.
- `wave_psp_c2` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c2-review-pass2c.md`
  - Validation result: approved as production-ready with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_c2` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c2-review-gap-cpython-parity-20260316.md`
  - Validation result: approved as production-grade with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_c2` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c2-review-gap-cpython-parity-20260317-r2.md`
  - Validation result: partially actionable on parity-contract clarity. The reported "critical difflib mismatch" reflects CPython's 3-argument `SequenceMatcher(isjunk, a, b)` semantics, while Sifr intentionally ships a simplified 2-argument constructor in this wave.
  - Fix status: remediated by adding compile-time guard fixture `phase_psp_c2_difflib_sequence_matcher_isjunk_unsupported.sifr` and tightening `verification/stdlib/wave_psp_c2_cpython_traceability.md` to explicitly classify the simplified constructor and deterministic non-junk matching as an intentional adaptation.
- `wave_psp_c2` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-c2-review-gap-cpython-parity-20260317-r4.md`
  - Validation result: approved with no actionable parity or CPython-test gaps; reviewer reported only a non-blocking base64 export style consistency suggestion.
  - Fix status: no code changes required.
- `wave_psp_ab` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-ab-review-pass1.md`
  - Validated findings: b2 parity coverage lacked CPython-derived random/secrets subset fixtures and explicit waiver guard fail tests for unsupported callable/factory surfaces.
  - Fix status: remediated in PR `#1190`.
- `wave_psp_ab` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-ab-review-pass2.md`
  - Validation result: no new actionable implementation issue. The repeated recommendation to reject all `range(...)` keywords conflicts with the approved `wave_psp_a1` adapted parity contract, and the "no b2 fail tests" note is stale after PR `#1190`.
  - Fix status: no code changes required.
- `wave_psp_ab` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-ab-review-pass3.md`
  - Validation result: partially actionable. The claimed absence of b2 fail tests was stale (fail fixtures exist since PR `#1190`), but the request for stronger explicit CPython-port evidence on A-wave coverage was valid.
  - Fix status: remediated by adding `crates/sifr/tests/e2e/pass/cpython_builtins_subset.sifr` and `crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr`, then wiring both into A-wave traceability docs.
- `wave_psp_ab` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-ab-review-pass4.md`
  - Validation result: no new actionable implementation issue. The repeated “no b2 fail tests” claim is stale: b2 has explicit fail fixtures (`phase_psp_b2_*_unsupported`) plus runtime negative-path assertions in `cpython_random_subset.sifr`.
  - Fix status: documentation tightened in `verification/stdlib/wave_psp_b2_cpython_traceability.md` to make negative coverage evidence explicit and avoid future stale misreads.
- `wave_psp_d1` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d1-review-pass1.md`
  - Validated finding: `pathlib.Path` transformation methods (`parent`, `joinpath`, `with_name`, `with_suffix`) returned `str` instead of `Path`, which broke Python-shaped chaining semantics.
  - Fix status: remediated in PR `#1193` by returning `Path` from those methods and updating impacted parity tests/demos.
- `wave_psp_d1` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d1-review-pass2.md`
  - Validation result: non-actionable stale review. The notes incorrectly reported `wave_psp_d1` as pending and missing artifacts that are already merged (`#1192`) on current mainline.
  - Fix status: no code changes required.
- `wave_psp_d1` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d1-review-gap-cpython-parity-20260316.md`
  - Validation result: no new actionable implementation issue. Reported gaps are documented adapt/waive scope boundaries already captured in `verification/stdlib/wave_psp_d1_cpython_traceability.md`.
  - Fix status: no code changes required.
- `wave_psp_d1` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d1-review-gap-cpython-parity-20260317-r1.md`
  - Validation result: partially actionable. Public parity behavior was stable, but absolute-path classification and open-mode waiver boundaries needed clearer enforcement and evidence to prevent cross-platform parity ambiguity.
  - Fix status: remediated by broadening `sifr.pathlib.is_absolute()` to recognize drive-prefixed and rooted-backslash absolute forms, extending `cpython_pathlib_subset.sifr` coverage for drive-form absolute semantics, extending `cpython_io_subset.sifr` to assert rejection of unsupported mixed read/write modes (`r+`/`w+`/`a+`), and tightening `verification/stdlib/wave_psp_d1_cpython_traceability.md` with explicit mode-matrix and internal-surface waivers.
- `wave_psp_d1` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d1-review-gap-cpython-parity-20260317-r3.md`
  - Validation result: partially actionable. The only valid gap was missing CPython-style evidence for `ZipFile.read()` on a non-existent entry from an existing archive.
  - Fix status: remediated by extending `crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr` with explicit missing-entry read assertions (`IOError` path).
- `wave_psp_d1` review pass 6:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d1-review-gap-cpython-parity-20260317-r4.md`
  - Validation result: reviewer satisfied with no actionable gaps after pass-5 remediation.
  - Fix status: no code changes required.
- `wave_psp_d2` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d2-review-pass1.md`
  - Validation result: non-actionable stale review. It incorrectly reported d2 as pending and missing artifacts (`wave_psp_d2` traceability/demo/pass/fail fixtures) that are present and merged in PR `#1198`.
  - Fix status: no code changes required.
- `wave_psp_d2` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d2-review-pass2.md`
  - Validation result: approved as production-ready with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_d2` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d2-review-gap-cpython-parity-20260316.md`
  - Validation result: approved with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_d2` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d2-review-gap-cpython-parity-20260317-r3.md`
  - Validation result: partially actionable. The CPython platform subset test used inverted logic for valid `platform.system()` values, so it could pass accidentally instead of asserting correct value-shape semantics.
  - Fix status: remediated by correcting `crates/sifr/tests/e2e/pass/cpython_platform_subset.sifr` to assert positive valid system-name forms.
- `wave_psp_d2` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-d2-review-gap-cpython-parity-20260317-r4.md`
  - Validation result: reviewer satisfied with no actionable gaps after pass-4 remediation.
  - Fix status: no code changes required.
- `wave_psp_e1` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e1-review-pass1.md`
  - Validation result: partially actionable. The review mostly reflected stale pre-`#1201` state (it claimed missing e1 artifacts and pending implementation), but the request to make e1 semantic adaptations explicit in evidence was valid.
  - Fix status: remediated by hardening `phase_psp_e1_core_modules_numeric_patterns_crypto.sifr` assertions (`timedelta.total_seconds()`, invalid combinatorics domains, and `digest()==hexdigest()`) and tightening adaptation notes in `verification/stdlib/wave_psp_e1_cpython_traceability.md`.
- `wave_psp_e1` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e1-review-pass2.md`
  - Validation result: non-actionable stale review. It incorrectly reported wave e1 as pending and missing traceability/demo artifacts that are already merged in `#1201` and `#1202`.
  - Fix status: no code changes required.
- `wave_psp_e1` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e1-review-gap-cpython-parity-20260316.md`
  - Validation result: approved with no actionable implementation issue.
  - Fix status: no code changes required.
- `wave_psp_e1` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e1-review-gap-cpython-parity-20260317-r1.md`
  - Validation result: approved with no actionable implementation, parity, or CPython-test gaps.
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-pass1.md`
  - Validation result: approved with no actionable implementation issue. The only missing-artifact concerns in the report were stale: the e2 wave demo, fail fixtures, and traceability ledger are already present in PR `#1205`.
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-pass2.md`
  - Validation result: approved as production-ready with no actionable implementation issue. The review reiterated stale artifact-missing claims that are invalid after merged PRs `#1205` and `#1206`.
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 3:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-pass3.md`
  - Validation result: approved with no actionable implementation issue after the e2 hardening pass (`argparse` token-shape support, `ipaddress` leading-zero rejection, `uuid` URN/curly parse normalization, and `graphlib` sparse-node filtering).
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 4:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-pass4.md`
  - Validation result: non-actionable stale cross-branch review. The report evaluated an outdated branch state and claimed e2 artifacts were missing; in this active worktree the referenced files and tests are present and validated.
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 5:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-pass5.md`
  - Validation result: approved with no actionable implementation issue after the follow-up e2 gap-closure hardening (argparse pending-option fallback + ipaddress special-range parity alignment + expanded CPython-derived regressions).
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 6:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-pass6.md`
  - Validation result: approved with no actionable implementation issue; the body references an older e2 commit snapshot but reports no correctness regressions against the active wave surface.
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 7:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-gap-cpython-parity-20260316.md`
  - Validation result: non-actionable stale review. It reports `wave_psp_e2` as "in progress / PR pending" even though the wave and follow-up hardening are already merged.
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 8:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-gap-cpython-parity-20260317-r1.md`
  - Validation result: non-actionable stale cross-branch review. The report claims critical regressions from an older branch snapshot, but all cited surfaces are present in the active worktree and the shipped e2 demo executes successfully with inline argparse handling, ipaddress classification helpers, uuid URN/curly parsing, and graphlib sparse-node filtering.
  - Fix status: no code changes required.
- `wave_psp_e2` review pass 9:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-e2-review-gap-cpython-parity-20260317-r2.md`
  - Validation result: reviewer satisfied with no actionable gaps when constrained to current-mainline worktree state.
  - Fix status: no code changes required.
- `milestone_psp_7` completion review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/milestone-psp-7-completion-review-20260317-r1.md`
  - Validation result: reviewer satisfied with milestone completion and no actionable closure gaps.
  - Fix status: no code changes required.
- `milestone_psp_7` production-grade review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/milestone-psp-7-production-grade-review-20260317-r1.md`
  - Validation result: actionable gap. Clippy `-D warnings` failed across HIR/codegen/driver, and the milestone demo had an invalid TOML type annotation.
  - Fix status: remediated by removing recursion-only context plumbing from `crates/sifr_hir/src/lower/expressions.rs`, hardening clippy-clean helper shapes in `crates/sifr_codegen/src/{class_method_emitter.rs,expr_render_helpers.rs,intrinsic_method_emitters.rs,intrinsics/toml.rs,stmt_support_emitter.rs}`, fixing driver lint issues in `crates/sifr_driver/src/build/materialize.rs`, and correcting TOML typing assertions in `demos/milestone_stdlib_parity_demo.sifr`.
- `milestone_psp_7` production-grade review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/milestone-psp-7-production-grade-review-20260317-r2.md`
  - Validation result: reviewer satisfied; milestone marked production-grade ready with no actionable gaps.
  - Fix status: no additional code changes required beyond pass-1 remediation.
- `milestone_psp_7` completion review pass 2:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/milestone-psp-7-completion-review-20260317-r2.md`
  - Validation result: reviewer satisfied; milestone completion closure confirmed with no actionable gaps.
  - Fix status: no code changes required.
- `phase_psp` completion review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/phase-psp-completion-review-20260317-r1.md`
  - Validation result: reviewer marked the phase complete with no actionable closure gaps.
  - Fix status: no code changes required.
- `phase_psp` production-grade review pass 1:
  - Reviewer file: `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/phase-psp-production-grade-review-20260317-r1.md`
  - Validation result: reviewer marked production-grade ready; only minor cosmetic governance status-marker inconsistencies were noted.
  - Fix status: remediated by updating status markers to `complete` in `verification/stdlib/milestone_psp_7_parity_governance_inventory.md` and `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`, and by closing this execution ledger status.
