## M11h Python Context/Coroutine Helpers - Round 1 Review

### Blocking findings
**None.** The migration is mechanically sound and follows the established M11 pattern already merged for zero-copy, buffer, arrow, and dlpack helpers.

### Verification summary

**Correctness - bridge, error, and type mapping**
- `stdlib/_sifr/python.sifr:324-347` declarations mirror the runtime signatures (`ObjectHandle = (i64,i64)` <-> `tuple[int,int]`; `&str` <-> `str`).
- `crates/sifr_stdlib/src/python.rs:549-588` shims use the same `object_handle(...) -> (i64,i64)` bridge helper as the other 30+ Python shims and delegate straight to `sifr_runtime::python::{enter_context,exit_context,exit_context_with_error,run_coroutine_blocking}`; return-mapping via `.map(object_raw)` for tuple-returning ops matches the release_dlpack precedent.
- The emit for `context_manager_body_failure.sifr` confirms the private-decl codegen produces a bridging wrapper that:
  - Converts `i64` -> `SifrIntBridge` via `From`.
  - Passes `&String` args (deref-coerces to `&str` - matches shim signature).
  - Maps `sifr_runtime::python::PythonError` -> local `PythonError` via `.map_err` copying all five fields (`message`, `kind`, `exception_type`, `traceback`, `context`) - semantically identical to the prior inline `map_python_error` in the codegen registry.

**ABI / panic safety**
- No `unsafe`, no unwrap/expect/assert added.
- `to_i64_saturating` on the bridge input is panic-free (same as every prior migrated helper).
- Tuple returns are plain `(i64,i64)` - no repr/layout risk.

**Bookkeeping**
- `internal_docs/stdlib_retained_compiler_intrinsics.toml` correctly trims to the 5 callback-only names: `local_callback`, `threadsafe_callback`, `py_close_callback`, `py_local_callback_echo`, `py_threadsafe_callback_echo` - matches the "remaining retained" list in the task.
- `scripts/check_stdlib_migration_closure.py:207-232` adds the 4 names alphabetically; guard passes at `retired_intrinsics=361`.
- `crates/sifr_codegen/src/intrinsics/registry/python.rs` drops `map`-arm, `object_expr`, `lower_handle_conversion`, and the four dedicated lowerers with no dangling helpers. `crates/sifr_retained_intrinsics/src/python.rs` similarly drops `object_handle()` - no orphans (grep-verified: only the new tests reference these names outside `stdlib/`).
- `stdlib/sifr/python.sifr` imports and callsites use the same symbol names - no public-API contract change.

**Test coverage**
- New codegen assertion `python_context_coroutine_helpers_are_owned_by_compiled_stdlib_declarations` and new driver assertion `python_context_coroutine_helpers_codegen_through_sifr_stdlib` mirror the wave-3 zero-copy tests exactly and cover both directions: registry no longer lowers, and stdlib codegen emits `sifr_stdlib::python::py_*(` calls.
- The three E2E fixtures (`context_manager_success`, `context_manager_body_failure`, `offloaded_python_calls`) type-check and emit via the new path per the emit-grep output attached to the task.

### Non-blocking follow-ups
1. **Wrapper `.map(|x| x)` identity** - every private-decl wrapper emits a redundant `.map(|__sifr_bridge_ok| __sifr_bridge_ok)` (visible in the emit for all four functions). Not specific to this slice - already present for the earlier M11 waves. Worth a cleanup on the codegen bridge template in a separate pass but does not affect correctness.
2. **Shim unit tests** - no crate-local test in `sifr_stdlib` calls the new shims directly; behaviour is validated only via the driver-level codegen assertion and E2E fixtures. Consistent with the existing pattern (`py_release_dlpack`, `py_from_str`, etc. also have no direct crate-local tests), so this is not a regression - noting it as a longer-term gap.
3. **Consider `#[must_use]` on the shim `Result` returns** - again, consistent-with-prior-helpers omission, so not a regression to fix here.

### Final verdict
**PR-ready.** Migration is complete, symmetric with the prior waves, all guardrails pass locally, and the emit output confirms the runtime routing has switched cleanly to `sifr_stdlib::python::py_*` with error semantics preserved.
