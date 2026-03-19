# Ad Hoc Phase Execution Checklist (Runtime and File-Object Parity Expansion)

Status: in_progress (started 2026-03-19; wave `wave_psp_runtime_0` review loop completed; wave `wave_psp_runtime_1` implementation merged and completion review pass approved; production-grade review pass pending)
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
3. [ ] `wave_psp_runtime_2`: tempfile and zipfile object lifecycle expansion
4. [ ] `wave_psp_runtime_3`: logging/time/timeit object-surface expansion
5. [ ] `wave_psp_runtime_4`: synchronous subprocess boundary cleanup and final governance closure
6. [ ] wave-level extra completion review cycle done
7. [ ] wave-level extra production-grade review cycle done
8. [ ] milestone-level completion review cycle done
9. [ ] milestone-level production-grade review cycle done
10. [ ] phase-level completion review cycle done
11. [ ] phase-level production-grade review cycle done
12. [ ] closure telegram notification sent

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
  - `#1317` (merged): https://github.com/yaseralnajjar/sifr/pull/1317
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
- Status: completed (implementation merged; completion review pass approved; production-grade review pass pending)
- Implementation PR:
  - `#1320` (merged): https://github.com/yaseralnajjar/sifr/pull/1320
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
  - negative regression: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_runtime_0_pyio_inheritance_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)

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
- Reviewer artifact: pending
- Status: pending
