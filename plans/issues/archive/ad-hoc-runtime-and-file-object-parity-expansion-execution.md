# Ad Hoc Phase Execution Checklist (Runtime and File-Object Parity Expansion)

Status: completed (started 2026-03-19; wave `wave_psp_runtime_0` review loop completed; wave `wave_psp_runtime_1` review loop completed; wave `wave_psp_runtime_2` review loop completed; wave `wave_psp_runtime_3` review loop completed; wave `wave_psp_runtime_4` implementation merged with pass-1 and pass-2 external review approval; wave-level closure review pass-1 and pass-2 approved; milestone-level closure review pass-1 and pass-2 approved; phase-level closure review pass-1 and pass-2 approved; closure telegram notifications sent)
Owner: ad_hoc_runtime_file_object execution loop
Reference planning doc:
- `issues/ad-hoc-runtime-and-file-object-parity-expansion.md`

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
1. [x] `wave_psp_runtime_0`: architecture lock for sealed hierarchy, lifecycle model, and permanent divergence boundaries
2. [x] `wave_psp_runtime_1`: `io` and in-memory stream hierarchy (`BytesIO`, `StringIO`)
3. [x] `wave_psp_runtime_2`: tempfile and zipfile object lifecycle expansion
4. [x] `wave_psp_runtime_3`: logging/time/timeit object-surface expansion
5. [x] `wave_psp_runtime_4`: synchronous subprocess boundary cleanup and final governance closure
6. [x] wave-level extra completion review cycle done
7. [x] wave-level extra production-grade review cycle done
8. [x] milestone-level completion review cycle done
9. [x] milestone-level production-grade review cycle done
10. [x] phase-level completion review cycle done
11. [x] phase-level production-grade review cycle done
12. [x] closure telegram notification sent

## Entry Baseline Evidence (2026-03-19)

Baseline command:
- `scripts/run_all_tests.sh --profile quick`

Observed baseline result before runtime-wave edits:
- HIR maintainability guardrails: PASS
- `sifr_driver` maintainability guardrails: PASS
- `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
- e2e fail/runtime/corpus lane: PASS (`25` tests)
- validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`): PASS (`7` rows)
- e2e pass suite quick profile: PASS (`24` fixtures, report signature `e1bf653aaa770517`)
- quick lane report: PASS (wall `42.45s`, max RSS `105.0MiB`, swaps `0`)

Required entry records:
- architecture lock must pin one sealed stream hierarchy, one lifecycle cleanup model, and one explicit host-limited divergence set before wave 1 implementation begins.
- binary stream and archive surfaces must explicitly consume first-class raw-byte-backed `bytes` and must not reintroduce `list[int]` carrier assumptions.
- CPython-family mapping must classify adopt/adapt/waive direction for `test_io`, `test_tempfile`, `test_zipfile`, `test_logging`, `test_time`, `test_timeit`, and `test_subprocess`.

## Wave Progress

### wave_psp_runtime_0: Architecture Lock
- Status: completed (implementation merged; completion review pass approved)
- Implementation PR:
  - `#1317` (merged): https://github.com/sifr-lang/sifr/pull/1317
- Scope:
  - lock sealed stream hierarchy and lifecycle-cleanup model for this phase
  - lock explicit permanent divergences (`_pyio` full inheritance, async `Popen`, `dictConfig`/dynamic graphs, thread-order claims, `SpooledTemporaryFile`, string-eval `timeit`, timezone mutation helpers)
  - add architecture-lock implementation notes, traceability matrix, demos, and explicit fail fixtures for the locked divergences
  - record explicit CPython family ownership and test-harvest mapping for all runtime/file-object phase families
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_runtime_0_architecture_lock.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_runtime_wave0_stream_hierarchy_contract_demo.sifr` -> PASS (`ad_hoc_runtime_wave0_stream_hierarchy_contract_demo: ok`)
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_runtime_wave0_tempfile_zip_lifecycle_demo.sifr` -> PASS (`ad_hoc_runtime_wave0_tempfile_zip_lifecycle_demo: ok`)
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_runtime_wave0_bytes_binary_io_contract_demo.sifr` -> PASS (`ad_hoc_runtime_wave0_bytes_binary_io_contract_demo: ok`)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_pyio_inheritance_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_async_popen_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_dictconfig_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_loggeradapter_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_spooled_tempfile_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timeit_string_eval_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timezone_mutation_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)

### wave_psp_runtime_1: `io` and In-Memory Stream Hierarchy
- Status: completed (implementation merged; completion and production-grade external review passes approved)
- Implementation PR:
  - `#1320` (merged): https://github.com/sifr-lang/sifr/pull/1320
- Scope:
  - add first-class in-memory stream wrappers (`StringIO`, `BytesIO`) with typed cursor/lifecycle APIs
  - add binary-handle entry `open_binary(...) -> Result[BinaryFileHandle, IOError]` while preserving `open(...)` compatibility
  - extend file-handle and stream surfaces with explicit `closed`/`flush`/`seek`/`tell`/`readable`/`writable`/`seekable` contracts
  - add wave-1 CPython traceability matrix and explicit typed-boundary negative fixtures for in-memory stream misuse
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_runtime_1_io_in_memory_stream_hierarchy.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_runtime_wave1_io_in_memory_hierarchy_demo.sifr` -> PASS (`ad_hoc_runtime_wave1_io_in_memory_hierarchy_demo: ok`)
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_runtime_0_architecture_lock.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run demos/m30_1f_logging_parity_demo/main.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_1_stringio_read_bytes_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_1_bytesio_text_write_unsupported.sifr` -> expected compile failure (PASS)
  - remediation path: `StringIO.seek`/`BytesIO.seek` now reject negative offsets (explicit `IOError` instead of silent clamp), covered by wave-1 fixture/demo assertions (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_pyio_inheritance_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)

### wave_psp_runtime_2: Tempfile and Archive Object Lifecycles
- Status: completed (implementation merged; completion and production-grade review passes closed)
- Implementation PR:
  - `#1323` (merged): https://github.com/sifr-lang/sifr/pull/1323
  - `#1325` (merged): https://github.com/sifr-lang/sifr/pull/1325
- Scope:
  - add deterministic lifecycle wrappers (`NamedTemporaryFile`, `TemporaryDirectory`) with explicit `close()/cleanup()` result surfaces and best-effort scope-exit cleanup
  - extend `zipfile` with bytes-backed write/read helpers plus governance surfaces (`is_zipfile`, `ZIP_STORED`, `ZIP_DEFLATED`, `ZipInfo`, `ZipReadHandle`)
  - land byte-native zip intrinsic contracts (`zip_add_file_bytes`, `zip_read_file_bytes`) so archive payload flow stays on first-class `bytes`
  - keep unsupported boundaries explicit (`ZipExtFile`, `ZIP_BZIP2`, `SpooledTemporaryFile`, and read-handle/extraction advanced methods not yet implemented in this tranche)
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_runtime_2_tempfile_zipfile_lifecycle.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_runtime_wave2_tempfile_zipfile_lifecycle_demo.sifr` -> PASS (`ad_hoc_runtime_wave2_tempfile_zipfile_lifecycle_demo: ok`)
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_tempfile_consolidated.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_zipfile.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_tempfile_subset.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_zipfile_subset.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_2_zip_ext_file_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_2_zip_bzip2_constant_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_spooled_tempfile_unsupported.sifr` -> expected compile failure (PASS)
  - remediation path: `NamedTemporaryFile`/`TemporaryDirectory` cleanup now propagates `remove_file`/`rmdir_all` failures via `Result`; `ZipReadHandle.read_bytes` now handles negative sizes explicitly as read-all and wave fixture/demo cover it; `ZipFile` write gating now accepts only `w`/`a`/`wb`/`ab` (rejects invalid mixed modes such as `rw`) (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)

### wave_psp_runtime_3: Logging, Clock, and Timer Object Expansion
- Status: completed (implementation merged; completion and production-grade review passes approved)
- Implementation PR:
  - `#1327` (merged): https://github.com/sifr-lang/sifr/pull/1327
- Scope:
  - expand `sifr.logging` with deterministic handler family (`Handler`, `StreamHandler`, `FileHandler`, `NullHandler`) and logger wiring (`add_handler`, `set_stream_handler`, `set_null_handler`, `clear_handler`) under single-process deterministic governance
  - expand `sifr.time` with immutable `struct_time`, explicit `gmtime_struct/localtime_struct`, `mktime`, and stable timezone constants (`TIMEZONE`, `ALTZONE`, `DAYLIGHT`, `TZNAME`)
  - expand `sifr.timeit` with callable `Timer` object surface (`timeit`, `repeat`, `__call__`) while preserving callable-only statement model
  - add wave-3 CPython traceability matrix and phase/demo fixtures for logging/time/timeit object surfaces
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_runtime_3_logging_time_timeit_object_surface.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_runtime_wave3_logging_time_timeit_object_surface_demo.sifr` -> PASS (`ad_hoc_runtime_wave3_logging_time_timeit_object_surface_demo: ok`)
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_logging_consolidated.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_logging_subset.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_time_consolidated.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_time_subset.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_timeit_consolidated.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_timeit_subset.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_runtime_0_architecture_lock.sifr` -> PASS
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_dictconfig_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_logging_loggeradapter_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timeit_string_eval_unsupported.sifr` -> expected compile failure (PASS)
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_timezone_mutation_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (profile `pr`, report signature `2161ea8c3fd4e3df`, 2026-03-20)

### wave_psp_runtime_4: Synchronous Subprocess Boundary Cleanup and Governance Closure
- Status: completed (implementation merged; completion and production-grade external review passes approved)
- Implementation PR:
  - `#1330` (merged): https://github.com/sifr-lang/sifr/pull/1330
- Scope:
  - expand `sifr.subprocess` with synchronous subprocess constants (`PIPE`, `STDOUT`, `DEVNULL`) and helper APIs (`check_call`, `check_output`)
  - keep deterministic sync `CompletedProcess` contract (`returncode`, `stdout`, `stderr`) while making non-zero exit behavior explicit through typed `IOError`
  - preserve explicit async waiver boundaries (`Popen` lifecycle/orchestration unsupported) and keep typed non-string command rejection
  - add wave-4 fixture/demo and per-wave subprocess CPython traceability matrix for final subprocess governance closure in this phase
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_runtime_4_subprocess_sync_boundary_governance.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_runtime_wave4_subprocess_sync_boundary_governance_demo.sifr` -> PASS (`ad_hoc_runtime_wave4_subprocess_sync_boundary_governance_demo: ok`)
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_subprocess_subset.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_subprocess.sifr` -> PASS
  - positive regression: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_d2_process_runtime_platform.sifr` -> PASS
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_d2_subprocess_non_string_cmd.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (profile `pr`, report signature `2161ea8c3fd4e3df`, 2026-03-20)

## External Review Passes

### wave_psp_runtime_0 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-0-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_runtime_0 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-0-review-pass-2.md`
- Status: completed (conditional pass accepted after governance clarifications: tempfile class timeline is explicitly anchored to wave 2 while wave 0 remains function-prototype based, and logging fail-soft file-sink behavior is explicitly documented as a host-limited policy to be finalized in wave 3)

### wave_psp_runtime_1 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-1-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_runtime_1 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-1-review-pass-2.md`
- Status: completed (conditional pass accepted after remediation: negative-seek behavior hardened for in-memory streams and remaining `BinaryFileHandle.read_bytes(size)` compatibility limitation documented explicitly in traceability governance)

### wave_psp_runtime_2 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-2-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_runtime_2 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-2-review-pass-2.md`
- Status: completed (conditional pass closed via `#1325` remediation: cleanup error propagation, explicit negative-size semantics, strict write-mode gating; constructor-time `Result` propagation remains constrained by current class-lowering behavior and is documented for wave-3 follow-up)

### wave_psp_runtime_3 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-3-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_runtime_3 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-3-review-pass-2.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_runtime_4 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-4-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_runtime_4 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-psp-runtime-4-review-pass-2.md`
- Status: completed (approved; production ready with full regression compatibility verified)

### wave_closure review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-closure-review-pass-1.md`
- Status: completed (approved; no completion gaps, contract violations, or evidence gaps identified)

### wave_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-wave-closure-review-pass-2.md`
- Status: completed (approved; production-ready wave closure with low regression risk and no evidence gaps)

### milestone_closure review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-milestone-closure-review-pass-1.md`
- Status: completed (approved; milestone-level scope, governance, and evidence closure confirmed)

### milestone_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-milestone-closure-review-pass-2.md`
- Status: completed (approved; production-grade milestone closure with low risk posture and full evidence coverage)

### phase_closure review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-phase-closure-review-pass-1.md`
- Status: completed (approved; phase-level completion, governance consistency, and artifact coverage confirmed)

### phase_closure review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-runtime-and-file-object-parity-expansion-phase-closure-review-pass-2.md`
- Status: completed (approved; production-grade phase closure with full validation evidence and low risk posture)
