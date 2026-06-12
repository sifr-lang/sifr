Reviewing the wave by examining diffs, generated code emission, and verification artifacts. I'll write the review now.

PASS

## Verification summary

**Generated code (Unix host, this checkout):**
- `process_signal_status.sifr` runs and asserts SIGTERM produces `Status(kind="signal", signal=15, success=False, exited()==False)` for `run`, `output`, and `spawn`+`wait`. Confirmed `cargo run` returns success (cache hit).
- `cargo check -p sifr_codegen -p sifr_stdlib -p sifr_driver -p sifr --quiet` → clean.
- `cargo fmt --check` → clean.
- Emit on `process_sync_output_text.sifr`: emits both `#[cfg(unix)]`/`#[cfg(not(unix))]` `__sifr_process_exit_signal` definitions and no `__SIFR_PROCESS_CHILDREN`/`__sifr_next_process_child_id` references. Prelude gating is doing the right thing.
- Emit on `process_spawn_wait_status.sifr`: emits both the signal helper *and* the child-handle table, as expected.

**Signal-status path correctness:**
- `crates/sifr_codegen/src/preamble/process_runtime.rs:5-72` builds two cfg-gated `fn __sifr_process_exit_signal` items via the IR `Attr` mechanism. `render_core.rs:22,300` honors tight spacing for `Attr` so the cfg attaches to the next `fn` when re-parsed by syn — both `is_shared_prelude_item` and `SharedNeedsCollector::visit_path` correctly find them.
- The Unix arm uses `std::os::unix::process::ExitStatusExt::signal(status)` returning `Option<i32>` and maps via `|x: _| x as i64`. No `.unwrap()` / `.expect()` / panic path. Non-Unix arm returns `None`. Covers every Rust target since `cfg(unix)` and `cfg(not(unix))` partition cleanly.
- `status_tuple` in `crates/sifr_codegen/src/intrinsics/registry/process.rs:168-198` builds `(code, signal)` from a single status expression. Callsites use either a local `__status` (own ExitStatus) or `__output.status` (Copy field on `std::process::Output`); both are safe to read twice (`.code()` and `&...`), including after the partial moves of `__output.stdout`/`__output.stderr` in `output_text_tuple_expr`.

**Sifr status contract:**
- `lib/sifr/process.sifr:144-161`: `_status_from_exit` upgrades to `kind="signal"`, `success=False`, `signal=N` whenever the helper returns Some, otherwise falls through to `_status` (success/nonzero). `_status_from` preserves timeout precedence: when `timed_out=True`, returns `kind="timeout"` and leaves `signal=None`, even though the SIGKILL we issued would otherwise surface as a signal — this matches the wave's stated semantics.
- `Status.__init__` leaves `signal=None`; the mutation pattern after construction is consistent with the existing `_status_from` design. `Status.exited()` returns False when signal is set.
- On non-Unix the helper returns None, so observed exits remain `success`/`nonzero` (or `timeout`); the platform matrix row in `verification/platform/supported_host_matrix.md:19` honestly tags Windows as host-limited rather than claiming portable signal status.

**Intrinsic registry coherence:**
- `crates/sifr_stdlib/src/process.rs` now returns `process_status_tuple()` (`tuple[int, int|None]`) from `process_run`, `process_wait`, `process_shell_run`; `process_bytes_output_tuple`, `process_bytes_timeout_output_tuple`, `process_text_output_tuple` carry the same nested status tuple. Each lowering in `intrinsics/registry/process.rs` mirrors the new shape (`status_tuple` for direct returns, `command_status_tuple("__output")` for output-bearing returns), and the timeout poll loop drops the obsolete `__status_code` placeholder in favor of letting `wait_with_output()` produce the final status. Type shapes match call-site annotations in `lib/sifr/process.sifr` exactly.

**Prelude gating:**
- `crates/sifr_codegen/src/stdlib_filter/implementation.rs` adds `SharedPreludeProcessStatusNeeds` and the matching text-scan / `Visit` path. `is_shared_prelude_item` includes `__sifr_process_exit_signal` so the helper is stripped from per-module output and emitted exactly once from `build_process_status_items` in `lib_modules_and_codegen.rs:576-578`. `needs_process_status` is independent of `needs_process_children`, so ordinary output paths do not pull in the child-handle table. Verified by emission above.

**Manifests / docs:**
- Both `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` add `process_signal_status` next to the existing process fixtures (json.tool parseable).
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md` adds `process_signal_status` to the Status row and to both lane lists; updates the `kill` follow-up wording to "non-Unix signal-status evidence remain later M4 work" and the follow-up boundary to "non-Unix signal status evidence". Phrasing matches what was actually implemented (Unix-only). Cancellation, owned pipes, async, graceful terminate, escalation are all still listed as deferred — no overclaim.
- Platform matrix row honestly tags Windows as `host-limited` with a concrete note about `ExitStatusExt`.
- Issue ledger gains a "pending PR" entry and a validation-evidence section that quotes the create-pr report signature (`f84374f7aa32a96e`) and pass counts.

No blockers found. Wave is scoped to signal-status evidence; it does not silently claim graceful terminate, escalation, cancellation, owned pipes, or async APIs.

## Non-blocking follow-ups (do not gate this PR)

- `crates/sifr_codegen/src/intrinsics/registry/process.rs:543` generates `if let Some(_) = (__child.try_wait()?) { break; }`. Functionally correct; if the embedded user-crate clippy profile ever turns on `redundant_pattern_matching`, this would warn. Consider switching to `if (__child.try_wait()?).is_some() { break; }` next time the timeout lowering is touched.
- `Status.code` on signal-killed children is the `-1` sentinel from `unwrap_or(-1) as i64`. Honest under the current contract since `kind=="signal"` and `signal` carry the meaning, but it is not documented in either the traceability doc or `Status` itself. A future doc pass could note "when `kind == 'signal'`, `code` is an unspecified sentinel and callers should consult `signal`."
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:416-419` lists the M4 waves but still omits a header bullet for the merged "M4 sync child kill" (PR #2337) — a pre-existing gap from the prior wave that this wave doesn't have to fix, but worth catching the next time the list is touched.
