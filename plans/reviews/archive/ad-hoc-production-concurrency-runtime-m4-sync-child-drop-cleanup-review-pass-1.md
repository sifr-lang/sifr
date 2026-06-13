RESULT: PASS

Code review summary:

**Verified correct:**
- `crates/sifr_codegen/src/class_emitter.rs:10-22` — `is_current_process_resource_class` is gated on `current_module_name == "sifr.process"`, so user classes named `Child`/`PipeReader`/etc. are unaffected. The codegen test (`process_child_resource_derives_are_module_scoped`) explicitly exercises both branches and passes.
- `class_emitter.rs:462` — Drop emission is further narrowed to `class.name == "Child"`, leaving the other resource wrappers as non-clone but Drop-less, matching the wave's stated scope ("sync Child drop cleanup").
- `class_emitter.rs:24-85` — Generated Drop:
  - Guards on `!self._waited`, so method-form `Child.wait()` (which sets `_waited = True` before `process_wait` in `lib/sifr/process.sifr:170`) correctly skips the redundant remove.
  - Uses `unwrap_or_else(|err| err.into_inner())` for panic-free Mutex-poison recovery — same pattern used elsewhere in `preamble/process_runtime.rs:114`.
  - `let _ = …remove(&self._handle)` discards the result; since handle IDs come from a monotonic atomic (`__sifr_next_process_child_id` via `fetch_add(SeqCst)` in `preamble/process_runtime.rs:664`), there is no recycle hazard if the entry was already removed by a prior `process_wait`.
  - References `_handle` (`i64`) and `_waited` (`bool`) match the actual `Child` declaration at `lib/sifr/process.sifr:150-156`.
- `lib_emitter_state.rs:38-39, 210, 307-334` — `current_module_name` is saved/restored around `emit_named_module`, so nested module emission (e.g., tests calling `emit_module` directly) does not leak the sentinel.
- `lib_modules_and_codegen.rs:225-228` — Old `generate_rust_with_stdlib(...)` is preserved as a wrapper, so the user-code emission path in `lib_project_codegen.rs:61` still works with `module_name=None`, leaving user classes derivation-unchanged.
- `sifr_driver/src/stdlib/bootstrap.rs:223-229` — Stdlib bootstrap now feeds each module name into the new entry point, which is what makes the gate fire for `sifr.process`.
- Generated Rust observed via `cargo run -- emit` matches the spec exactly: `#[derive(Debug)]` only on `Child`, and an `impl Drop for Child` that does the guarded `remove(&self._handle)`. `process_spawn_wait_status` and `process_scoped_spawn_handle` fixtures run clean.
- Docs (`verification/platform/supported_host_matrix.md`, `verification/stdlib/concurrency_runtime_m4_process_traceability.md`, `issues/...-execution.md`) are scoped to "drops the host child handle without claiming kill, wait, process-group cleanup, descendant supervision, or non-Unix status mapping" — they do not overclaim what Drop actually does.
- File-size guardrail OK (largest touched file: 857 lines).

**Residual non-blocking risks:**
- Top-level `wait(child)` in `lib/sifr/process.sifr:427` does not flip `_waited`, so after a successful top-level wait the Drop still does a redundant `HashMap::remove` lookup. Harmless (idempotent + monotonic IDs), but it makes the two wait surfaces behave slightly differently — worth a follow-up to keep wrapper state consistent.
- Clone was removed from the entire `sifr.process` resource family (`AsyncChild`, `ProcessHandle`, `PipeReader`/`Writer`, `AsyncPipeReader`/`Writer`) but Drop is only added for sync `Child`. These wrappers now move rather than clone — a safer default, but the underlying handles still leak in the runtime tables on drop. Explicitly out of scope for this wave per the docs, but the next M4 follow-up should cover them.
- The unrelated bulk deletions under `audits/`, `third_party/ruff`, and `verification/package_management/demo_repositories/` appear in the diff (submodule pointers and audit reports). They are orthogonal to the Drop wave; if they weren't intended to be part of this PR they should be split out, but they don't touch the changes under review.
