I've reviewed the validator and a representative sample of the modified fixtures across nearly every fixture family. Here are the actionable findings.

## Actionable findings

### Correctness — non-Sifr type names in `.sifr` fixtures

Sifr's `resolve_type_annotation` (`crates/sifr_type_system/src/infer.rs:27`) recognizes only `uintN`, `intN`, `float`, `bytes`, etc. Several new fuller examples use Rust-style aliases that are not valid Sifr types. Even though these are contract-only / cargo-probe fixtures that don't fully type-check, they are now the canonical "fuller examples" readers will copy from, so they should use the real Sifr type names.

- `verification/areas/rust_interop/fixtures/async_runtime_reqwest/examples/tokio.sifr:14` — `delay_ms: u64` and `tokio_spawn(1)` should be `uint64` (cf. `tokio.sifr:10` which already uses `uint64` for `task_id`).
- `verification/areas/rust_interop/fixtures/direct_crate_crc32/examples/crc32fast.sifr:10,12,13` — `u32` should be `uint32` (and the matching positive fixture `…/positive/crc32fast_hash_uint32.sifr` already uses `uint32`).
- `verification/areas/rust_interop/fixtures/async_ecosystem_matrix/examples/http-body.sifr:10,12,13` — `u64` should be `uint64`.
- `verification/areas/rust_interop/fixtures/blocking_diagnostics/examples/rayon.sifr:10,12,13` — `i32` should be `int32`.
- `verification/areas/rust_interop/fixtures/bridge_type_matrix/examples/indexmap.sifr:10,12,13` — `i32` should be `int32`.
- `verification/areas/rust_interop/fixtures/async_ecosystem_matrix/examples/futures.sifr:10,12,13` — `tuple[i32, i32]` should be `tuple[int32, int32]`.
- `verification/areas/rust_interop/fixtures/advanced_data_matrix/examples/candle.sifr:14,17` — `list[f32]` (parameter annotation) is not a Sifr type. Sifr has only `float`; consider `list[float]`. (The `dtype=f32` inside `@rust.view(...)` kwargs is a decorator argument, not a Sifr type annotation, and is fine.)
- `verification/areas/rust_interop/fixtures/advanced_data_matrix/examples/ndarray.sifr:14,17` and `verification/areas/rust_interop/fixtures/tensor_dlpack_bridge/examples/ndarray.sifr:14,17` — same `list[f32]` issue.

### Correctness — undeclared / forward-referenced types in evidence files

The matrix validator only checks for a `verify_…` function and that it calls every `@rust`-decorated binding — it does not check that referenced classes exist. Multiple new fuller evidence files reference types that are never declared in the file or are declared after first use, which is exactly the "still skeletal" smell the task is trying to remove for readers.

- Never declared in the file:
  - `verification/areas/rust_interop/fixtures/advanced_data_matrix/positive/advanced_data_metadata.sifr:11,13,14` — `DataFrame`.
  - `verification/areas/rust_interop/fixtures/advanced_data_matrix/negative/dtype_shape_mismatch.sifr:9,11,12` — `DataFrame`.
  - `verification/areas/rust_interop/fixtures/arrow_record_batch/positive/arrow_schema_identity.sifr:12,14,15` — `ArrowRecordBatch`.
  - `verification/areas/rust_interop/fixtures/arrow_record_batch/negative/invalid_arrow_metadata.sifr:9,11,12` — `ArrowRecordBatch`.
  - `verification/areas/rust_interop/fixtures/tensor_dlpack_bridge/positive/explicit_tensor_ownership.sifr:12,14,15` — `Tensor`.
  - `verification/areas/rust_interop/fixtures/tensor_dlpack_bridge/negative/implicit_dlpack_ownership_rejected.sifr:9,11,12` — `Tensor`.
  - `verification/areas/rust_interop/fixtures/proc_macro_trust/positive/trusted_proc_macro.sifr:9,14,15` — `GeneratedMessage` is referenced three times but never declared.
- Declared after first use (Sifr scoping aside, this looks bad in a "fuller example" intended to be readable):
  - `verification/areas/rust_interop/fixtures/panic_boundary/negative/panic_payload_not_exposed.sifr:8` uses `UserError` before declaring it on line 10.
  - `verification/areas/rust_interop/fixtures/dotted_path_resolution/positive/valid_structured_paths.sifr:7` uses `HashError` before declaring it on line 9.
  - `verification/areas/rust_interop/fixtures/native_build_script/positive/trusted_build_script_native_evidence.sifr:9` uses `NativeError` before declaring it on line 11.
  - `verification/areas/rust_interop/fixtures/panic_boundary_wrapper_emission/negative/invalid_map_error_signature_rejected.sifr:8` uses `PanicMapped` before declaring it on line 10.
  - `verification/areas/rust_interop/fixtures/proc_macro_trust/positive/trusted_proc_macro.sifr:9` uses `DecodeError` before declaring it on line 11.

### Consistency — async crates with non-async package examples

The fuller-evidence change made the evidence files use `async def` + `await` where appropriate, but the matching package examples are still sync. This is misleading for readers studying the async-runtime/async-ecosystem families.

- `verification/areas/rust_interop/fixtures/async_runtime_reqwest/examples/reqwest.sifr:14,16` — `reqwest.Client.get` is naturally async and the sibling positive evidence (`async_reqwest_loopback.sifr`) uses `async def` + `await`, but the package example declares a plain `def reqwest_get(...)` and a plain `def verify_reqwest_package()`.
- `verification/areas/rust_interop/fixtures/async_runtime_reqwest/examples/tokio.sifr:14,16` — `tokio.spawn` is naturally async, declared as plain `def`.
- `verification/areas/rust_interop/fixtures/async_ecosystem_matrix/examples/futures.sifr:10,12` — `futures::future::join` is async-only; declared sync.

### Verifier-enforcement gaps in `check_fixture_matrix.py`

These are gaps in the new enforcement that allow exactly the kind of skeletal-evidence regressions the task is trying to prevent.

- `verification/areas/rust_interop/checks/check_fixture_matrix.py:368-369` — The `\n    pass\n` placeholder-class-body rejection runs only inside `_validate_package_example_text`. `_validate_evidence_example_text` (the new function added at line 496) does not apply the same check, so a positive/negative evidence file can re-introduce empty `class Foo:\n    pass` bodies without tripping the validator. Easy fix: hoist the `pass` check into a shared helper or duplicate it inside `_validate_evidence_example_text`.
- `verification/areas/rust_interop/checks/check_fixture_matrix.py:383-384` — The "must bind sample inputs before returning" rule rejects only the literal pattern `return {bound_function}(`. `return wrap(bound_function(...))`, `return [bound_function(x) for x in xs]`, or any one-line composition would all pass while still being effectively skeletal. If the goal is "the verifier must contain a `name = bound_function(...)` assignment", check for that positively rather than negatively.
- `verification/areas/rust_interop/checks/check_fixture_matrix.py:387-403` — `_rust_bound_function_name` returns only the first `@rust(<crate>...)` binding it finds. If a package example declares two bindings against the same crate, only the first has to be called by the verifier. Consider returning all bindings and asserting each is invoked, mirroring how `_rust_bound_declaration_names` is used for evidence files.
- `verification/areas/rust_interop/checks/check_fixture_matrix.py:522-543` — `_rust_bound_declaration_names` requires `stripped.endswith(": ...")`. If anyone adds a multi-line stub (e.g., `def foo(\n    x: int,\n) -> int: ...`) the binding will silently be dropped from the "must be called" set. Not present in the current diff, but worth a future tightening.
- `verification/areas/rust_interop/checks/check_fixture_matrix.py:466-468` — The existing "must exercise a Rust interop declaration" check accepts any `@rust*` decorator (e.g., a lone `@rust.view(...)` without an accompanying `@rust(...)` binding). The new evidence-example check happens to also run, so this isn't currently exploitable, but if `_validate_evidence_example_text`'s name extractor returns an empty list (which it will for a file containing only decorators without a `def …: ...` body), the verifier check becomes vacuous. Consider requiring at least one bound name to be collected.

### Lower priority / observations

- `verification/areas/rust_interop/fixtures/direct_crate_matrix/examples/regex.sifr` and `verification/areas/rust_interop/fixtures/direct_crate_negative_type/examples/regex.sifr` are byte-identical except for the `# execution-kind:` header. Not a validator issue, but worth flagging as a copy-paste smell in the new package-example layout.
- `verification/areas/rust_interop/fixtures/opaque_handle_tokenizer/negative/unsatisfied_send_or_copy_rejected.sifr` declares `close=close` in `@rust.opaque(...)` but does not define a `close` method on the class. The matrix validator does not check `close=` is satisfied, so this is silent; if the future intent is for this fixture to demonstrate handle aliasing (per the file name) rather than missing close, the discrepancy is fine, but the diagnostic family for this evidence (`SIFR-RUST-HANDLE-0001`) and the file name make the intent ambiguous.
- README at `verification/areas/rust_interop/README.md:30-37` correctly describes the new rule, but does not mention that the matrix validator only catches `\n    pass\n` (4-space indent). If you adopt 2-space or tab indentation anywhere, the check would silently miss it. Either widen the regex or note the indent assumption.

Nothing in the diff is obviously broken by the matrix validator today — all the items above are either real semantic mistakes in fixture text that the validator does not yet catch, or gaps in the validator that future contributors could exploit.
