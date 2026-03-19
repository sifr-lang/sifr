# Ad Hoc Phase Execution Checklist (First-Class Bytes and Binary Surface Foundation)

Status: in_progress (started 2026-03-19)
Owner: ad_hoc_first_class_bytes execution loop
Reference planning doc:
- `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`

Loop per wave: Plan -> Implement -> Validate -> Demo -> PR -> External completion review -> Fix -> PR -> Merge -> External production-grade review -> Fix -> PR -> Merge -> Update docs -> Next wave

## Global Gates
- [x] Entry baseline validated before wave 0
- [x] Scope remains constrained to active wave
- [ ] Root cause is fixed without compatibility shims
- [x] Positive-path and negative-path validation recorded for each wave
- [x] Demo runs before opening each wave PR
- [x] `$(pwd)/scripts/run_all_tests.sh` run before each wave PR
- [x] PR opened/reviewed/merged before next wave starts
- [x] Docs + traceability + waiver state updated before moving on

## Full Phase To-Do Plan
1. [x] `wave_psp_bytes_0`: architecture lock, explicit permanent-diff fixtures, and CPython family mapping
2. [x] `wave_psp_bytes_1`: first-class `bytes` type-system/HIR/lowering/codegen foundation
3. [x] `wave_psp_bytes_2`: UTF-8/hex conversion surfaces and `sifr.bytes` compatibility delegation
4. [x] `wave_psp_bytes_3`: downstream contract adoption and governance closeout
5. [ ] wave-level extra completion review cycle done
6. [ ] wave-level extra production-grade review cycle done
7. [ ] milestone-level completion review cycle done
8. [ ] milestone-level production-grade review cycle done
9. [ ] phase-level completion review cycle done
10. [ ] phase-level production-grade review cycle done
11. [ ] closure telegram notification sent

## Entry Baseline Evidence (2026-03-19)

Baseline command:
- `scripts/run_all_tests.sh --profile quick`

Observed baseline result before wave edits:
- HIR maintainability guardrails: PASS
- `sifr_driver` maintainability guardrails: PASS
- `cargo test -p sifr -- --skip test_e2e_pass`: PASS (`37` tests)
- e2e fail/runtime/corpus lane: PASS (`25` tests)
- validation contract matrix (`frontend_mode_parity`, `phase23_graph_isolation`): PASS (`7` rows)
- e2e pass suite quick profile: PASS (`24` fixtures, report signature `e1bf653aaa770517`)
- quick lane report: PASS (wall `124.65s`, max RSS `364.4MiB`, swaps `0`)

Required entry records:
- bytes literal parser/AST support is confirmed as already shipped; wave implementation starts at type-system/HIR/lowering/codegen migration.
- permanent Sifr-safe diffs for mutable/view/buffer families are explicitly classified and enforced with negative fixtures before wave 1.
- CPython-family mapping must classify adopted/adapted/waived direction for `test_bytes`, `test_base64`, `test_hashlib`, and binary `test_io` families.

## Wave Progress

### wave_psp_bytes_0: Architecture Lock
- Status: completed
- Implementation PR:
  - `#1291` (merged): https://github.com/yaseralnajjar/sifr/pull/1291
- Scope:
  - lock first-class immutable `bytes` contract and text/binary boundary for this phase
  - classify permanent diffs (`bytearray`, `memoryview`, buffer protocol, implicit coercions, non-UTF-8 codecs)
  - add wave-0 positive/negative fixtures and demos
  - add CPython-family mapping table for bytes/binary wave ownership
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_0_architecture_lock.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave0_binary_contract_lock_demo.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave0_text_binary_boundary_demo.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_0_bytearray_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_0_memoryview_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_0_buffer_protocol_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_0_implicit_str_bytes_coercion_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_0_non_utf8_codec_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_0_bytes_subclass_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)

### wave_psp_bytes_1: Core `bytes` Type and Compiler Support
- Status: completed
- Implementation PR:
  - `#1294`: https://github.com/yaseralnajjar/sifr/pull/1294
- Scope:
  - add first-class `Type::Bytes` in type-system inference/checking/union ordering
  - lower bytes literals and `bytes()` constructor through HIR/lowering/codegen as immutable sequence values
  - ship bytes indexing/slicing/iteration/concatenation/equality behavior and bytes method typing/lowering (`len`, `count`, `contains`, `index`, `to_ints`)
  - enforce bytes immutability for subscript assignment and unsupported mutating methods
  - migrate existing pass fixtures/demos and `sifr.bytes`/`sifr.base64` signatures from `list[int]` boundaries to first-class `bytes`
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_1_core_type_support.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave1_core_type_demo.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave1_iteration_and_equality_demo.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_1_subscript_assignment_unsupported.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_1_append_unsupported.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-19)

### wave_psp_bytes_2: Conversion Surfaces and Compatibility Migration
- Status: completed
- Implementation PR:
  - `#1297` (merged): https://github.com/yaseralnajjar/sifr/pull/1297
- Scope:
  - lower `bytes(size)` to a typed `Result[bytes, ValueError]` intrinsic path
  - lower `bytes.from_ints(list[int])` and `bytes.from_hex(str)` as first-class bytes factory calls
  - lower `str.encode(encoding?)` and `bytes.decode(encoding?)` through typed intrinsic calls with UTF-8-only enforcement for literal and runtime codec inputs
  - add bytes conversion-surface intrinsics in codegen registry (`bytes_with_size`, `bytes_from_ints`, `str_encode_utf8_result`, codec-aware encode/decode variants)
  - migrate `lib/sifr/bytes.sifr` compatibility exports to delegate decode/from-hex/from-ints/size paths to first-class `bytes` surfaces
  - add wave-2 pass/fail fixtures and wave-2 demos for conversion success/failure semantics
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_2_conversion_surfaces.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_2_conversion_negative_paths.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave2_conversion_surface_demo.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave2_negative_boundary_demo.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_2_constructor_non_int.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_2_from_hex_non_string.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_2_from_ints_non_int_list.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_2_encode_non_string_codec.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_2_decode_non_string_codec.sifr` -> expected compile failure (PASS)
  - targeted unit lane: `cargo test -q -p sifr_codegen lowers_bytes_intrinsics_via_registry` -> PASS
  - targeted unit lane: `cargo test -q -p sifr_hir lower:: -- --nocapture` -> PASS (`105` passed; `1` ignored)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-19)

### wave_psp_bytes_3: Downstream Contract Adoption and Governance Closeout
- Status: completed
- Implementation PR:
  - `#1301` (merged): https://github.com/yaseralnajjar/sifr/pull/1301
- Scope:
  - migrate current shipped `io` binary method boundaries to first-class `bytes` (`FileHandle.read_bytes` / `write_bytes`) while keeping internal intrinsic names stable
  - add explicit wave-3 pass/fail coverage proving downstream binary-carrier contract adoption (`bytes`) and compile-time rejection of stale `list[int]` payload assumptions
  - add wave-3 downstream contract demo for bytes file-handle roundtrip
  - update canonical parity governance and traceability ledgers to remove stale “custom helper over list[int]” wording and record the narrowed remaining binary waiver set
  - re-anchor successor runtime/file-object and RNG/crypto phase readiness text on first-class `bytes` as canonical binary carrier
- Validation:
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_3_downstream_contract_alignment.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave3_downstream_contract_adoption_demo.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/open_binary_read.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/open_binary_write.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_io_subset.sifr` -> PASS
  - positive path: `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr` -> PASS
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_write_bytes_rejects_int_list.sifr` -> expected compile failure (PASS)
  - negative path: `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_read_bytes_not_list.sifr` -> expected compile failure (PASS)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh --profile quick` -> PASS (2026-03-19)
  - wave gate: `$(pwd)/scripts/run_all_tests.sh` -> PASS (2026-03-19)

## External Review Passes

### wave_psp_bytes_0 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-0-review-pass-1.md`
- Status: completed (external review approved; no remediation changes required)

### wave_psp_bytes_0 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-0-review-pass-2.md`
- Status: completed (approved for production-grade readiness; no remediation changes required)

### wave_psp_bytes_1 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-1-review-pass-1.md`
- Status: completed (no blockers; remediated reviewer suggestions on bytes unsupported-method diagnostics and representation documentation)

### wave_psp_bytes_1 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-1-review-pass-2.md`
- Status: completed (approved for production readiness on rerun after prior timeout; no remediation changes required)

### wave_psp_bytes_2 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-2-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_bytes_2 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-2-review-pass-2.md`
- Status: completed (approved for production readiness; no remediation changes required)

### wave_psp_bytes_3 review_pass_1 (completion-gap)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-3-review-pass-1.md`
- Status: completed (approved; no remediation changes required)

### wave_psp_bytes_3 review_pass_2 (production-grade)
- Reviewer artifact: `reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-psp-bytes-3-review-pass-2.md`
- Status: completed (conditional approval remediated by documenting internal `Type::Bytes` backend representation as an explicit intentional-diff governance item and correcting public-phase wording to match shipped implementation)

### closure review cycles
- wave closure completion review: completed (`reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-closure-completion-check-review-pass-1.md`; reviewer gaps remediated via wave-1 pass-2 rerun and explicit wave-1/wave-2 CPython traceability ledgers)
- wave closure production-grade review: completed (`reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-wave-closure-production-grade-check-review-pass-2.md`; approved for wave-closure production readiness)
- milestone closure completion review: completed (`reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-milestone-closure-completion-check-review-pass-1.md`; approved for milestone closure completion after delayed artifact arrival following initial timeout)
- milestone closure production-grade review: completed (`reviews/phase-ad-hoc-first-class-bytes-and-binary-surface-foundation-milestone-closure-production-grade-check-review-pass-2.md`; approved for milestone-closure production readiness)
- phase closure completion review: pending
- phase closure production-grade review: pending
