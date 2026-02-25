## [Task] Replace Sifr file `# expect-stdout` checks with native Sifr assertions

## Goal
Replace stdout-based behavioral checks in `.sifr` files (`# expect-stdout`) with Sifr assertions so tests are expressed in language-native form
and stop depending on output matching via harness comments.

## Scope (current observed state)
- Total files with `# expect-stdout`: **631**
- Total expected lines: **2049**
- Location split:
  - `audit/leetcode`: **208** files / **450** expectations
  - `crates/sifr/tests/e2e/pass`: **387** files / **1138** expectations
  - `demos`: **36** files / **461** expectations
- Exclusions:
  - `# expect-error` only files (error-mode checks)
  - `# expect-stderr` files
  - Non-target runtime-fail fixture behavior unless clearly stdout-based

## Observations (important for migration safety)
- `# expect-stdout` files already importing `sifr.test`: **47**
- Files with `# expect-stdout` and no `print(...)`:
  - `crates/sifr/tests/e2e/pass/stdlib_logging.sifr`
  - `crates/sifr/tests/e2e/pass/stdlib_logging_class.sifr`
  - `crates/sifr/tests/e2e/pass/logging_basic_config.sifr`
- Bucket by print-vs-expected line count:
  - exact match (`print == expect`): **513** files
  - mismatch requiring manual intervention: **118** files

## Execution Plan

### 1) Preparation
- Create migration log in this issue with three buckets:
  - `simple` (exact 1:1 conversion)
  - `manual-extra-prints` (more prints than expects)
  - `manual-extra-expects` (more expects than prints)
- Freeze conversion rules:
  - `print(expr)` + `# expect-stdout: X` → `assert expr == X`
  - Keep `true/false` expectations as direct bool asserts where natural.
  - Preserve side-effect/logging behavior tests as-is if they are explicitly validating output channels.

### 2) Automated conversion for `simple` bucket
- Convert each `# expect-stdout` line to an assertion tied to the immediately related `print` sequence.
- Remove the consumed `# expect-stdout` comment lines.
- Validate no new semantic statements are introduced; keep existing control flow and variable names unchanged.
- Prioritize by smallest risk:
  1. `audit/leetcode` (mostly algorithmic I/O assertions)
  2. `crates/sifr/tests/e2e/pass`
  3. `demos`

### 3) Manual conversion for mismatch bucket
- `print > expect`:
  - remove/logical-noise prints OR keep prints and add asserts for expected lines where behavior is asserted.
- `expect > print`:
  - find missing assertions or implicit behavior and convert to explicit assertions.
- `0 print` files:
  - rewrite with capture-based assertions or replace with direct non-stdout validation where possible.

### 4) Post-migration consistency pass
- Remove all remaining `# expect-stdout` comments in converted files.
- Keep `# expect-error` and `# expect-stderr` untouched unless explicitly out-of-scope and approved.
- Deduplicate and normalize assertion style:
  - Prefer existing `sifr.test` helpers when file already imports them and behavior matches.
  - Prefer native `assert` form when replacing pure debug prints.

### 5) Validation
- Add/track checks in issue:
  - no `# expect-stdout` remains in converted files.
  - no behavior regressions observed in existing e2e command flow
  - per-folder conversion counts match target:
    - `audit/leetcode`
    - `crates/sifr/tests/e2e/pass`
    - `demos`

## Acceptance Criteria
- All targeted `# expect-stdout` usage removed.
- All previously validated outputs are represented via assertions in Sifr.
- No expected-output checks accidentally moved out of test intent (especially in exception/safety/demo paths).
- No regressions in existing `e2e` pass collection behavior.

## Rollout Strategy
- Do by folder batches to keep review easy:
  1. `demos` (36 files)
  2. `crates/sifr/tests/e2e/pass` (387 files)
  3. `audit/leetcode` (208 files)
- Commit per batch, with a short checklist for each batch:
  - converted files
  - skipped/manual exceptions
  - risky/owner-review items

## Notes
- This is a large mechanical migration and should be done in scripted passes with explicit exceptions list so intent is auditable.
- Any file requiring semantic interpretation beyond straightforward print replacement should be documented in this issue before changing.

## Migration To-Do (Execution Tracking)

### Rollout Checklist

- [x] Part 1 - demos: convert all `simple` files and review demos for behavior retention
- [x] Part 2 - crates/sifr/tests/e2e/pass: convert all `simple` files and review demos for behavior retention
- [x] Part 3 - audit/leetcode: convert all `simple` files and review semantic edge cases
- [ ] Manual bucket audit: resolve `manual_*` files with explicit assertions or owner review

### Current Mechanical Conversion Status

- `demos`: 11 simple converted, 25 remaining files requiring manual review
- `crates/sifr/tests/e2e/pass`: 294 simple converted, 93 remaining files requiring manual review
- `audit/leetcode`: 208 simple converted, 0 remaining files requiring manual review

### Pending Manual Buckets

### manual_print>expect (92)
- `crates/sifr/tests/e2e/pass/bigint_overflow_conversion.sifr`
- `crates/sifr/tests/e2e/pass/bigint_to_int.sifr`
- `crates/sifr/tests/e2e/pass/chained_comparison.sifr`
- `crates/sifr/tests/e2e/pass/collection_safety_error_paths.sifr`
- `crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr`
- `crates/sifr/tests/e2e/pass/cpython_base64_strictness_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_base64_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_bytes_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_hashlib_api_subset.sifr`
- `crates/sifr/tests/e2e/pass/cpython_itertools.sifr`
- `crates/sifr/tests/e2e/pass/cpython_json.sifr`
- `crates/sifr/tests/e2e/pass/cpython_re.sifr`
- `crates/sifr/tests/e2e/pass/cpython_statistics.sifr`
- `crates/sifr/tests/e2e/pass/cpython_textwrap.sifr`
- `crates/sifr/tests/e2e/pass/csv_reader_file.sifr`
- `crates/sifr/tests/e2e/pass/custom_error.sifr`
- `crates/sifr/tests/e2e/pass/del_statement.sifr`
- `crates/sifr/tests/e2e/pass/dict_get_option.sifr`
- `crates/sifr/tests/e2e/pass/edge_case_safety.sifr`
- `crates/sifr/tests/e2e/pass/error_custom_class.sifr`
- `crates/sifr/tests/e2e/pass/error_propagation.sifr`
- `crates/sifr/tests/e2e/pass/error_subclass_handling.sifr`
- `crates/sifr/tests/e2e/pass/intrinsics_block_test.sifr`
- `crates/sifr/tests/e2e/pass/io_safety_error_paths.sifr`
- `crates/sifr/tests/e2e/pass/list_pop_option.sifr`
- `crates/sifr/tests/e2e/pass/loop_else.sifr`
- `crates/sifr/tests/e2e/pass/open_binary_read.sifr`
- `crates/sifr/tests/e2e/pass/open_binary_write.sifr`
- `crates/sifr/tests/e2e/pass/open_context_manager.sifr`
- `crates/sifr/tests/e2e/pass/open_read.sifr`
- `crates/sifr/tests/e2e/pass/open_readline.sifr`
- `crates/sifr/tests/e2e/pass/open_write.sifr`
- `crates/sifr/tests/e2e/pass/parse_safety_error_paths.sifr`
- `crates/sifr/tests/e2e/pass/path_glob.sifr`
- `crates/sifr/tests/e2e/pass/re_flags_ignorecase.sifr`
- `crates/sifr/tests/e2e/pass/result_basic.sifr`
- `crates/sifr/tests/e2e/pass/safe_dict_key.sifr`
- `crates/sifr/tests/e2e/pass/safe_list_index.sifr`
- `crates/sifr/tests/e2e/pass/safe_string_index.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_base64_intrinsics.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_bytes.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_bytes_safety.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_datetime.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_encoding.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_glob.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_graphlib.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_graphlib_class.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_gzip.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_itertools_extended.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_json.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_os.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_os_expanded.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_pathlib_additions.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_random_new.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_re.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_re_expanded.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_re_pattern.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_secrets.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_shutil.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_statistics.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_statistics_expanded.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_statistics_extended.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_statistics_new.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_subprocess.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_tempfile.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_time_intrinsics.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_tomllib.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_zipfile.sifr`
- `crates/sifr/tests/e2e/pass/string_find_option.sifr`
- `crates/sifr/tests/e2e/pass/subprocess_completed_process.sifr`
- `demos/m0_parity_foundation_demo.sifr`
- `demos/m2_bytes_demo.sifr`
- `demos/m3_base64_demo.sifr`
- `demos/m5_hashlib_demo.sifr`
- `demos/milestone_codegen_preamble_migration_demo.sifr`
- `demos/milestone_codegen_structural_passes_demo.sifr`
- `demos/milestone_cpython_tests_demo.sifr`
- `demos/milestone_edge_case_safety_demo.sifr`
- `demos/milestone_error_handling_demo.sifr`
- `demos/milestone_error_safety_demo.sifr`
- `demos/milestone_io_safety_demo.sifr`
- `demos/milestone_new_modules_demo.sifr`
- `demos/milestone_parse_safety_demo.sifr`
- `demos/milestone_safe_indexing_demo.sifr`
- `demos/milestone_stdlib_expansion_demo.sifr`
- `demos/milestone_stdlib_functions_demo.sifr`
- `demos/milestone_stdlib_intrinsic_expansion_demo.sifr`
- `demos/milestone_stdlib_migration_demo.sifr`
- `demos/milestone_stdlib_naming_demo.sifr`
- `demos/milestone_stdlib_parity_demo.sifr`
- `demos/milestone_stdlib_pure_expansion_demo.sifr`
- `demos/milestone_test_infra_demo.sifr`


### manual_expect>print (23)
- `crates/sifr/tests/e2e/pass/builtins_range_3arg.sifr`
- `crates/sifr/tests/e2e/pass/comp_range.sifr`
- `crates/sifr/tests/e2e/pass/for_tuple_unpack.sifr`
- `crates/sifr/tests/e2e/pass/generator_expr.sifr`
- `crates/sifr/tests/e2e/pass/generic_accumulate_float.sifr`
- `crates/sifr/tests/e2e/pass/generic_chain_float.sifr`
- `crates/sifr/tests/e2e/pass/generic_chain_str.sifr`
- `crates/sifr/tests/e2e/pass/generic_heapq_bigint.sifr`
- `crates/sifr/tests/e2e/pass/generic_heapq_float.sifr`
- `crates/sifr/tests/e2e/pass/generic_zip_longest_str.sifr`
- `crates/sifr/tests/e2e/pass/iteration_string.sifr`
- `crates/sifr/tests/e2e/pass/lazy_conditional_yield.sifr`
- `crates/sifr/tests/e2e/pass/lazy_for_loop.sifr`
- `crates/sifr/tests/e2e/pass/lazy_generator.sifr`
- `crates/sifr/tests/e2e/pass/protocol_dispatch.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_csv.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_logging_enhanced.sifr`
- `crates/sifr/tests/e2e/pass/union_ops_list_concat.sifr`
- `crates/sifr/tests/e2e/pass/union_ops_list_remove.sifr`
- `crates/sifr/tests/e2e/pass/with_break.sifr`
- `demos/milestone_compiler_hardening_demo.sifr`
- `demos/milestone_iteration_v2_demo.sifr`
- `demos/milestone_lazy_iterators_demo.sifr`


### manual_no_print (3)
- `crates/sifr/tests/e2e/pass/logging_basic_config.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_logging.sifr`
- `crates/sifr/tests/e2e/pass/stdlib_logging_class.sifr`



### Note

Manual files should be individually documented with root-cause and intended assertion strategy before any scripted changes.
