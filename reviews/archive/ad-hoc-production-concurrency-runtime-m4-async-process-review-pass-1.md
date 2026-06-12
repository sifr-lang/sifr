Based on a thorough inspection of the M4 async process run/output loopback wave, here is my review.

## Implementation Scope Verification

**1. Public Sifr APIs** (`lib/sifr/process.sifr:281-299`)
- `async_run(own command: Command) -> Awaitable[Result[Status, ProcessError]]` — moves Command, returns awaitable, not `@blocking_io`. ✓
- `async_output(own command: Command) -> Awaitable[Result[Output, ProcessError]]` — same shape, threads `has_stdin_data` flag. ✓

**2. Intrinsic registry** (`crates/sifr_stdlib/src/process.rs:167-193`)
- Both async intrinsics return `Type::Awaitable(Box::new(process_status_object_result()))` / `process_output_object_result()` with correct parameter shapes. ✓

**3. Lowering** (`crates/sifr_codegen/src/intrinsics/registry/process_async.rs`)
- Lowers to `Box::pin(__sifr_process_async_run(...))` / `Box::pin(__sifr_process_async_output(...))` with `Clone` of owned args (program/args/env/cwd). ✓
- Registered in `registry.rs:607-614` with `StdlibFeature::Tokio` as `required_feature`. ✓

**4. Runtime preamble** (`crates/sifr_codegen/src/preamble/process_runtime.rs`)
- `__sifr_process_async_run` / `__sifr_process_async_output` are `async fn`s calling `tokio::process::Command::status().await` / `.output().await`. ✓
- I/O errors mapped via `process_map_err` to typed `ProcessError`, no panics. ✓
- `__sifr_process_status_from_exit` reuses existing Sifr `Status` class for nonzero/signal/success kinds. ✓
- `__sifr_process_async_output` short-circuits when `has_stdin=true` with explicit typed `ProcessError("async process stdin bytes require owned pipe support")` BEFORE building the command — stdin data is not silently dropped. ✓

**5. Tokio feature wiring**
- `features.rs:189`: Tokio spec includes `"process"`. ✓
- `lib_modules_and_codegen.rs:781-788`: `stdlib_preamble.contains("tokio::")` forces `StdlibFeature::Tokio` for generated projects. ✓
- `fixture_compilation.rs:481`: harness `tokio_dependency_spec()` mirrors the same `"process"` feature for grouped e2e crates. ✓
- Both new tests (`test_generate_cargo_toml_required_tokio_uses_runtime_features`, `test_generate_project_emits_tokio_dependency_when_required`) assert the `"process"` feature literal. ✓

**6. Stdlib filter wiring** (`crates/sifr_codegen/src/stdlib_filter/implementation.rs`)
- New `SharedPreludeProcessAsyncNeeds` flag (51-54). Detection via both AST visit (350-354) and text scan (320-323). `is_shared_prelude_item` strips `__sifr_process_async_run`/`__sifr_process_async_output` from stdlib bodies (377-380). ✓
- `lib_modules_and_codegen.rs:416`: `needs_process_status = stdlib_needs_process_status || stdlib_needs_process_async` correctly forces the status helper whenever async path is in use. ✓
- Sync-only path: since user never imports `async_run`/`async_output`, `filter_stdlib_ir_to_needed` strips those bodies, so no `__sifr_process_async_run` reference survives the filter and no async preamble is emitted. ✓
- Async-only path: no spawn/wait reference → no `__SIFR_PROCESS_CHILDREN` emitted. ✓

**7. E2E fixture** (`crates/sifr/tests/e2e/pass/process_async_run_output.sifr`)
- Covers success run, nonzero exit (`exit 9` → `kind == "nonzero"`), stdout/stderr capture via `sh -c "printf … >&2"`, and stdin-deferral path catching `ProcessError` with `"owned pipe"` substring. ✓
- Nested try/except is well-formed: inner catches the stdin rejection; outer assertion `e.message == ""` is a no-op safety net. ✓

**8. Validation manifests** (`create_pr_e2e_manifest.json:90`, `merge_e2e_manifest.json:105`) — both include `process_async_run_output`. ✓

**9. Traceability** (`verification/stdlib/concurrency_runtime_m4_process_traceability.md`)
- Status line, output row, Status row, async APIs row, CPython mapping, and validation coverage all correctly identify the wave as run/output loopback only, with stdin-deferral and Windows openness explicit. ✓

**10. Supported host matrix** (`verification/platform/supported_host_matrix.md:19`)
- New row: macOS arm64 / Linux x86_64 `supported`, Windows x86_64 `host-limited` — honest about deferred Windows determinism. ✓

**11. Issue ledger** (`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:421, 692-712`)
- Targeted local validation block matches the provided evidence. PR link marked pending and review file pending. ✓

## Type System / Codegen Correctness Spot Checks

- `Status::new(code, kind)` matches the Sifr `Status.__init__` signature; preamble's field reassignment of `success`/`signal` works because Sifr generates `pub` fields. ✓
- `__output.status` is `std::process::ExitStatus` (`Copy`), so the two `__output.status` accesses (for code and signal) compile fine despite stdout/stderr partial moves. ✓
- `__sifr_process_exit_signal` is cfg-gated on Unix and returns `None` on non-Unix — portable. ✓
- `process_async_owned_args` clones the four `String`/`Vec<String>` args (matching `all_borrow` convention upstream) and passes `has_cwd: bool` (Copy) by value. ✓
- `Awaitable[T]` lowers to a boxed future; `Box::pin(__sifr_process_async_run(...))` matches `Pin<Box<impl Future<Output=Result<Status, ProcessError>>>>`. Awaiting it inside the user's `async def main()` is well-typed; the local validation `cargo run` confirms this. ✓
- `_status_from_exit` (Sifr, sync path) vs `__sifr_process_status_from_exit` (Rust runtime, async path) produce equivalent Status shapes; signal precedence and kind labels match. ✓

## File Size & Guardrails
- All touched files under the 900-line cap (largest: `process.rs` at 887 lines, unchanged async-related). ✓
- `python3 scripts/check_file_size_guardrails.py` and HIR guardrails reported PASS in the evidence. ✓

## Non-Blocking Polish (NOT blockers)
1. `process_status_object_result()` and `process_output_object_result()` reuse the existing synthetic `Type::Class { ..., methods: vec![] }` pattern. This means users awaiting the *intrinsic* directly couldn't call `Status::exited()`, but they only see the Sifr public surface `async_run`/`async_output` which is correctly typed. Same pattern as existing sync intrinsics. No fix needed.
2. `reviews/ad-hoc-production-concurrency-runtime-m4-async-process-review-pass-1.md` is currently a 0-byte placeholder — to be filled by this review pass.
3. `__SIFR_PROCESS_CHILDREN` and `__SIFR_NEXT_PROCESS_CHILD_ID` detection in `derive_shared_needs_text_scan` (313-316) does not include `__sifr_process_async_run` as a substring of `__sifr_next_process_child_id`. The string boundaries are distinct enough that no false positive can fire. OK as-is.

## Result

No blockers found. The implementation is scoped tightly to the stated intent (async run/output loopback, no spawn/wait/communicate/owned-pipes/timeout/cancellation/shell-async/Windows/scoped-supervision), stdin bytes are explicitly deferred with a typed error rather than dropped, sync behavior is untouched, Tokio's `process` feature is wired through both compiler and harness paths, and traceability/host-matrix/manifests honestly reflect the wave.

**RESULT: PASS**
