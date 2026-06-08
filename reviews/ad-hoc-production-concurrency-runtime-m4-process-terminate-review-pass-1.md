RESULT: PASS

What I verified:

**Public API surface (`lib/sifr/process.sifr:121-127`, `lib/sifr/process.sifr:281-283`)**
- Top-level `terminate(child)` and `Child.terminate()` both carry `@blocking_io`, both return `Result[None, ProcessError]`, and both bottom out in `process_terminate(handle)` — symmetric with the existing `kill` shape.
- `_sifr.process.process_terminate` metadata declared in `crates/sifr_stdlib/src/process.rs:168-174` matches the public callers' signature exactly (one `int` handle in, `Result[None, ProcessError]` out).

**Intrinsic lowering (`crates/sifr_codegen/src/intrinsics/registry/process_child_lifecycle.rs:254-262`, `registry.rs:599-602`)**
- `lower_process_terminate` correctly rejects non-1-arg call shapes and forwards to the generated helper. Spawn/kill/wait have been moved into this new file as well; arity checks and table-lock semantics for them match the prior `process.rs` implementation that was removed.

**Generated helper (`crates/sifr_codegen/src/preamble/process_runtime.rs:407-532`, registered at 688-691)**
- Unix branch: locks the child table, looks up `__child` via `get_mut`, captures `__child.id().to_string()`, runs `/bin/kill -TERM <pid>` via `std::process::Command`, maps spawn/`status()` failure to `ProcessError`, returns `Err` on non-success exit, returns `Ok(())` on success. Crucially, it does **not** remove the handle from the table, so `wait` can still observe the SIGTERM signal status afterward — consistent with the pass fixture's `status.signal == 15` assertion.
- Non-Unix branch: returns a typed `ProcessError` (`"process terminate is unsupported on this host; use kill for forceful termination"`) — host-limited honestly with a useful escape hatch.
- No `.unwrap()` / `.expect()` on data-dependent values, no `assert!`. Lock poisoning falls back to `into_inner` (same pattern as kill/wait). No user-triggerable panic.

**Prelude filtering (`crates/sifr_codegen/src/stdlib_filter/implementation.rs`)**
- `__sifr_process_terminate` added in all three required places (text scan ~line 322, `SharedNeedsCollector` match arm ~line 367, `is_shared_prelude_item` predicate ~line 416). Matches the kill/spawn/pipe pattern so the helper is preserved in shared prelude when reachable and stripped otherwise.

**Fixtures**
- `pass/process_child_terminate_wait.sifr`: covers free-function form, method form, signal-status observation (`signal == 15`), and post-wait "closed or unknown" recovery. Ran locally → exit 0.
- `fail/process_terminate_direct_async_rejected.sifr` and `fail/process_child_terminate_method_direct_async_rejected.sifr`: both fire `SIFR-ASYNC-0003` with the expected `terminate` / `Child.terminate` identifier in the message. Verified locally.

**File-size guardrail**
- `intrinsics/registry/process.rs`: 692 lines (down from over the cap after extraction).
- `preamble/process_runtime.rs`: 699 lines (close to the cap; flagged below).
- `intrinsics/registry/process_child_lifecycle.rs`: 262 lines.
- `stdlib_filter/implementation.rs`: 753 lines.
- All under 900.

**Docs / manifests**
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md:22` adds a sync terminate row, honest about Unix-only signal evidence and Windows host-limited mapping.
- `verification/platform/supported_host_matrix.md:21` adds "Sync subprocess graceful terminate" with Windows = host-limited.
- Both create-pr and merge manifests pick up `process_child_terminate_wait`.

**Local sanity checks I ran**
- `cargo clippy -p sifr_codegen -p sifr_stdlib --no-deps`: no new warnings (only the pre-existing `fn_params_excessive_bools` in `process_async_runtime.rs`).
- `cargo run -q -p sifr -- run pass/process_child_terminate_wait.sifr`: exit 0.
- `cargo run -q -p sifr -- check` on both fail fixtures: SIFR-ASYNC-0003, correct callee names.
- `cargo run -q -p sifr -- emit` confirmed the `#[cfg(unix)]` / `#[cfg(not(unix))]` pair, the `kill -TERM` invocation, and the typed-error non-Unix stub.

Non-blocking follow-ups:

1. **`__SIFR_PROCESS_CHILDREN` mutex is held across the entire `kill` subprocess fork/exec/wait** in `process_runtime.rs:418-489`. `__pid` is a `String` (independent of the lock-guard'd `&mut Child`), so the guard could be dropped after `__pid` is captured to avoid blocking other process operations during the (typically short) shell-out. Worth tightening when this code is next touched.

2. **Shelling out to `/bin/kill` to send a signal** is an unusual choice for a compiled language — it pays a fork/exec per terminate. Once `nix` (or a small `libc::kill` shim) clears the dependency-policy review, swapping the implementation will be cheaper and avoids depending on the host having `kill` on `PATH`. Acceptable for the current wave since it works on POSIX and avoids `unsafe_code`.

3. **Validation evidence in the execution ledger is lighter than prior M4 waves** (`issues/...-execution.md:929-941`): targeted `cargo check`, `cargo fmt`, file-size, fixture runs, and the fail suite are recorded, but the authoritative `scripts/run_all_tests.sh --profile create-pr` and a full `cargo clippy --workspace -- -D warnings` are not. Every previous M4 wave (async output timeout, stdin guardrails, pipe writer, etc.) records the create-pr lane PASS. Recommend running it and appending the evidence before opening the PR — that is the project's stated gate per AGENTS.md.

4. **Unrelated changes mixed into this branch**: `issues/ad-hoc-production-network-http-platform-substrate{.md,-execution.md}` modifications and two untracked `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-{1,2}.md` files come from a different network/http effort. Per AGENTS.md "Keep changes focused on the requested milestone/issue," these should land in their own PR rather than ride along with the terminate change.
