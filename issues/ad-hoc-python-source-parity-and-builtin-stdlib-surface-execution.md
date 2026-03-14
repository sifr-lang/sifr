# Ad Hoc Phase Execution: Python Source Parity and Builtin Stdlib Surface Closure

Status: in_progress
Started: 2026-03-14
Phase owner: Codex (GPT-5)
Source phase: `issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`
Current active wave: `wave_psp_a2`

## Execution Rules

- Follow the phase in strict sequence.
- Only one wave is active at a time.
- Each wave must complete: CPython test inventory, implementation, demo validation, local validation, PR, review, merge, and doc updates before the next wave starts.
- No fallback-only APIs, no workaround-first closures, and no undocumented parity gaps.

## Phase Todo

- [x] `milestone_psp_1` / `wave_psp_a1`: builtin constructors and callable surface
- [ ] `milestone_psp_2` / `wave_psp_a2`: core object models and builtin semantics
- [ ] `milestone_psp_3` / `wave_psp_b1`: collections objects and ordered helpers
- [ ] `milestone_psp_3` / `wave_psp_b2`: iterators, functional helpers, and randomness
- [ ] `milestone_psp_4` / `wave_psp_c1`: structured parsing and serialization
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

Status: in_progress

- [x] Harvest the required CPython test inputs for container and string object-model behavior.
- [x] Close `list`, `dict`, `set`, `tuple`, `str`, and `bytes` classification/object-model gaps.
- [x] Add adapted regression coverage and update parity ledgers.
- [ ] Demo, validate, PR, review, merge.

### `wave_psp_b1` Collections Objects and Ordered Helpers

Status: pending

- [ ] Harvest `Lib/test/test_collections.py`, `Lib/test/test_bisect.py`, and `Lib/test/test_heapq.py`.
- [ ] Close `collections`, `bisect`, and `heapq` constructor/object/call-shape gaps.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_b2` Iterators, Functional Helpers, and Randomness

Status: pending

- [ ] Harvest `Lib/test/test_itertools.py`, `Lib/test/test_functools.py`, `Lib/test/test_operator.py`, `Lib/test/test_random.py`, and `Lib/test/test_secrets.py`.
- [ ] Close iterator/object/callable parity for `itertools`, `functools`, `operator`, `random`, and `secrets`.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_c1` Structured Parsing and Serialization

Status: pending

- [ ] Harvest `Lib/test/test_json/`, `Lib/test/test_tomllib/`, `Lib/test/test_csv.py`, and `Lib/test/test_configparser.py`.
- [ ] Close structured-return and class/export gaps for `json`, `tomllib`, `csv`, and `configparser`.
- [ ] Add traceable regressions, demo, validate, PR, review, merge.

### `wave_psp_c2` Text, Pattern, and Formatting Modules

Status: pending

- [ ] Harvest the required CPython text-formatting test families.
- [ ] Close `string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, and `calendar`.
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
  - `crates/sifr/tests/e2e/fail/phase_psp_a1_sorted_unexpected_keyword.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a1_map_callable_arity_mismatch.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_a1_tuple_dynamic_list_shape.sifr`
  - `demos/wave_psp_a1_builtin_callable_surface_demo.sifr`
  - `verification/stdlib/wave_psp_a1_cpython_traceability.md`
- Demo validation:
  - `cargo run -q -p sifr -- run demos/wave_psp_a1_builtin_callable_surface_demo.sifr`
- Targeted regression validation:
  - `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr`
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

### `wave_psp_b1`

- Pending.

### `wave_psp_b2`

- Pending.

### `wave_psp_c1`

- Pending.

### `wave_psp_c2`

- Pending.

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

## External Review Ledger

- Pending.
