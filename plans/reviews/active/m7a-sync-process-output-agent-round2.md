Verified round 1's findings hold and confirmed the new E2E Cargo.toml wiring is complete. Writing the review.

## Code Review — M7a sync process output migration (round 2)

### Scope re-verification (delta since round 1)

- **E2E grouped Cargo.toml gap (round 1 fix)**: `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs:329-330` now includes `sifr.process` and `_sifr.process` in `needs_sifr_stdlib_module_dependency`, and `fixture_dependency_paths.rs:129-131` maps both modules to the `process` feature. `stateless_sysroot_cargo_toml_tests.rs:103-112` asserts both flavors emit `features = ["process"]`. Symmetric with the compiler manifest at `crates/sifr_stdlib_manifest/src/features/generated_stdlib_features.rs:51`.
- **Public sifr.process facade**: all 9 sync surfaces (`run`, `run_timeout`, `output`, `output_timeout`, `output_text`, `run_shell`, `output_shell`, `output_shell_timeout`, `output_shell_text`) route through `_sifr.process` private declarations. `_status_from_parts` (`stdlib/sifr/process.sifr:384-390`) reads `[code, signal, has_signal]` matching `status_tuple` (`crates/sifr_stdlib/src/process.rs:313-320`).
- **Compiler registry & retained catalog closure**: `crates/sifr_codegen/src/intrinsics/registry.rs` no longer routes any of the 8 migrated names; the retained catalog (`sifr_retained_intrinsics/src/process.rs`, `internal_docs/stdlib_retained_compiler_intrinsics.toml`) keeps only child/pipe/async surfaces. Handle getters (`process_output_stdout/stderr/status/timed_out/close`) are neither in registry nor retained catalog; they lower purely through the private declaration path. Tests in `registry_core_tests.rs:448-464` and `registry_extended_tests.rs:238-268` assert the sync names now return `None` from lowering.
- **Feature/dependency wiring**: `crates/sifr_stdlib/Cargo.toml:86` `process = ["dep:sifr_runtime"]` correctly pulls the runtime for `sifr_runtime::encoding::decode_text`. `features_tests.rs::process_private_module_emits_process_stdlib_feature` locks the manifest mapping.
- **No user-triggerable panic/error path**: `PROCESS_OUTPUTS.lock()` uses `PoisonError::into_inner` recovery (`process.rs:296-300`); `next_output_id` uses `AtomicU64::fetch_add(Relaxed)`; timeout preconditions surface `io::Error::other` for NaN/negative/overflow (`process.rs:169-176`); every fallible IO uses `?` and lowers to `ProcessError` through `bridge_error_expr`. No `.unwrap()`/`.expect()` in the runtime path.

### Findings against the 5 checkpoints

1. Sync API regression: **none**. Semantics of Status (`success`, `timed_out`, `signal`, `kind`), Output (`stdout`, `stderr`, `status`), TextOutput (`stdout`, `stderr`, `status`, `encoding`) preserved. Text-decoding remains a native precondition (`store_text_output`, `process.rs:260-264`) before any handle is stored.
2. Handle-store leak reachable by user code: **none**. Round 1's four `try/except` sites still skip `process_output_close` on getter failure, but with `store_text_output` decoding *before* insert (no handle exists on decode failure) and all getters cloning stored fields (which are populated for the paths that create the handle), no realistic getter-side failure path leaves the entry orphaned. Behavior unchanged from round 1's assessment.
3. Missing dependency/feature mapping: **none**. Both the compiler manifest and the E2E harness map `sifr.process`/`_sifr.process` → `process`; `sifr_stdlib_manifest` unit test locks the mapping; the E2E stateless-sysroot test locks both flavors.
4. Migrated sync intrinsic still in registry/retained: **none**. Sync `process_run`, `process_output`, `process_output_text`, `process_output_timeout`, `process_shell_run`, `process_shell_output`, `process_shell_output_text`, `process_shell_output_timeout` are removed from `registry.rs` dispatch and the retained TOML/catalog `exact_intrinsics`. Handle getters correctly never appeared in registry or retained catalog.
5. Panic/error handling violations: **none observed** in the runtime path.

### Non-blocking notes (unchanged from round 1)

- Handle-map `try/except` in `sifr.process` skips `process_output_close` on getter failure; presently unreachable but worth revisiting when new fallible getters are added.
- `process_output_timed_out` (`process.rs:246-250`) returns `false` for unknown handles while sibling getters return `io::Error`. Cosmetic API inconsistency; callers today only pass live handles.
- `terminate_process_group_or_child` (`process.rs:349-372`) propagates `kill` spawn/wait failures via `?`; could theoretically leave a child unwaited if the `kill` binary is missing. Existing pre-existing behavior; environment-dependent only.
- `bridge_error_expr` name-based `ProcessError` match (`rust_interop_direct.rs:205-209`) would collide with a user-defined shadowing class of the same name; consistent with the JSON error name-arms.

### File-size guardrail

`crates/sifr_codegen/src/rust_interop_direct.rs` is exactly 900 lines. The guardrail is strictly `>`, so it passes; noted only because it leaves zero headroom for future edits.

### Verdict

READY
