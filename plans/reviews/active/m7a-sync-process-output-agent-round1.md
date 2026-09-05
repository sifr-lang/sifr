I've completed the review. Let me write it up.

## Code Review — M7a stdlib-native-boundary: sync process migration

### Scope verification

- Sync surfaces `process_run`, `process_output`, `process_output_text`, `process_output_timeout`, `process_shell_run`, `process_shell_output`, `process_shell_output_text`, `process_shell_output_timeout` are removed from the compiler intrinsic dispatch (`registry.rs`), the retained catalog (`sifr_retained_intrinsics/src/process.rs`), and the allowlist (`stdlib_retained_compiler_intrinsics.toml`). Child, pipe, async surfaces remain intact.
- New Rust implementations live in `crates/sifr_stdlib/src/process.rs`; the `process` feature now correctly depends on `sifr_runtime` (Cargo.toml:86) for `sifr_runtime::encoding::decode_text`.
- `_sifr/process.sifr` declarations return handle-based getters (`process_output_stdout/stderr/status/timed_out/close`), consistent with the milestone rationale on tuple/record shapes.
- `stdlib/sifr/process.sifr` uses `_status_from_parts([code, signal, has_signal], timed_out)`; the parts layout in `status_tuple` (process.rs:313-320) matches: `parts[0]=code`, `parts[1]=signal`, `parts[2]=has_signal`. Status semantics (`success`, `timed_out`, `signal`, `kind`) preserved.
- `rust_interop_direct.rs:205` maps `Type::Alias { name: "ProcessError", .. }` to `ProcessError { message: err.to_string() }`, plus the resolved class arm at line 212 for the pre-resolved alias case. Codegen test `process_sync_private_declarations_codegen_through_sifr_stdlib` asserts the emitted map_err shape.

### Findings

1. **Handle-map leak on error paths (minor, hardening only)** — `stdlib/sifr/process.sifr:502-508, 524-530, 546-552, 667-679, 697-714, 726-738`. The pattern is `handle = process_output(...)` → getters → `process_output_close(handle)` → return. If any getter raises `ProcessError`, `process_output_close(handle)` is skipped and the entry stays in the static `PROCESS_OUTPUTS` map. In practice the getters can only fail on "unknown handle" (impossible right after a successful store) or on text decoding (only when `store_text_output` failed before storing, so no handle to leak). No user-triggerable trigger today, but the `try/except` re-raise misses the closure step, which is worth revisiting when adding new failure modes.

2. **`process_output_timed_out` swallows unknown handles** — `crates/sifr_stdlib/src/process.rs:246-250`. All other getters return `Result<T, io::Error>` and surface "unknown process output handle"; this one silently returns `false`. Not a bug (callers only invoke it on live handles), but the API inconsistency is worth noting so it doesn't mask a future race by returning a plausible false-y default.

3. **`terminate_process_group_or_child` can leak child on spawn failure** — `crates/sifr_stdlib/src/process.rs:349-372`. The two `Command::new("kill").status()?` calls propagate any spawn/wait IO error via `?`. If the `kill` binary spawn ever fails, we return before `wait_with_output`, leaving a zombie. Realistically only triggers on truly broken environments, but the pre-existing timeout-cleanup contract deserves a comment or best-effort ignore for launch failures.

4. **`bridge_error_expr` name-based ProcessError match** — `crates/sifr_codegen/src/rust_interop_direct.rs:205-210`. The early return fires for any `Type::Alias { name: "ProcessError", .. }` without checking fields or parent class. In this migration ProcessError has only `message: str`, so the output is well-formed; if a user ever defines a shadowing class named `ProcessError` with different fields, the emitted `ProcessError { message: ... }` would refer to that user class and fail rustc typecheck. Consistent with the existing name-based JSON error arms below it, so keep as-is unless we hit a shadowing collision.

### Non-issues verified

- Direct interop conversion of `Result[list[int], ProcessError]` correctly maps `Vec<SifrIntBridge>` → `Vec<i64>` via `bridge_int_vec_to_i64_vec_expr`.
- Timeout preconditions reject `NaN`, infinity, and negatives (process.rs:169-176); host-clock overflow surfaces "process timeout is too large" (process.rs:174-176).
- `write_child_stdin` no-op when `has_stdin` is false; `wait_with_output` still closes the piped stdin, so children see EOF (matches previous behavior).
- Handle IDs are unique via `AtomicU64::fetch_add(Relaxed)` (process.rs:302-307); the format `process-output-<n>` is opaque to callers.
- Retained catalog for `_sifr.process` still routes children/pipes/async via `process_status_result`, `process_output_object_result`, etc. — unchanged and consistent with the retained wave.
- Guardrail scripts (`check_stdlib_migration_closure.py`, `check_stdlib_native_intrinsic_allowlist.py`) match the reduced allowlist. Legacy `subprocess_run*` removal test in `lib.rs:151-164` still passes.
- E2E fixtures (`process_sync_output_text`, `process_text_explicit_encoding`, `process_sync_bytes_env_cwd_stdin`, `process_shell_exec_output`, `process_timeout_status`, `process_timeout_group_cleanup`) exercise the migrated surfaces including bytes/text, env+cwd+stdin, shell, timeout status, and process-group cleanup.

### Verdict

The four findings above are hardening/consistency notes, not blockers — no correctness regression, no closure gap, and the milestone's stated behavior preservation holds.

READY
