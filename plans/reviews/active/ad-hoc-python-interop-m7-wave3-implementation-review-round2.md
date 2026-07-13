# Implementation Review — M7 Wave 3 Cancellation Carrier (Round 2)

## Scope of round 2

Only one implementation-tree change since round 1 (which returned SATISFIED): the retained-manifest edit in `internal_docs/stdlib_retained_compiler_intrinsics.toml` — the `_sifr.task::language_runtime_glue` surface gained
`task_cancellation_runtime.rs` + `task_supervisor_runtime.rs` in `preamble_files` and `direct_runtime_roots = ["sifr_runtime::cancellation"]`.

## Ownership placement — verified

| Item | Observed at | Allowlist owner |
|---|---|---|
| `task_cancellation_runtime.rs` | `crates/sifr_codegen/src/preamble/` | `_sifr.task::language_runtime_glue` (only) |
| `task_supervisor_runtime.rs` | `crates/sifr_codegen/src/preamble/` | `_sifr.task::language_runtime_glue` (only) |
| `sifr_runtime::cancellation` | `preamble/task_runtime.rs`, `preamble/task_cancellation_runtime.rs`, `preamble/task_scope_offload_runtime.rs` (non-test) | `_sifr.task::language_runtime_glue` (only) |

The two new preamble files sit next to `task_runtime.rs`, `task_scope_offload_runtime.rs`, `join_set_runtime.rs`, `task_context_runtime.rs`, `cpu_offload_runtime.rs`, `parallel_runtime.rs` under the same surface — the shape matches how existing task-runtime glue is owned. The surface's reason string already lists "cancellation" as a covered concern, so no reason text is stale.

`preamble_files` and `direct_runtime_roots` are in the validator's `unique_owner_keys` — an entry may appear in only one surface. Both new entries appear only in `_sifr.task::language_runtime_glue`; no duplicate ownership.

Enumerated `sifr_runtime::*` roots referenced from non-test codegen files: `DEFAULT_MAX_INTEGER_DIGITS` (shared-language-preamble), `cancellation` (task), `interop` (shared-language-preamble), `python` (declaration-first-python-runtime-glue). All four match `direct_runtime_roots` in the manifest — no missing/stale.

Enumerated preamble files (11 total: `cpu_offload_runtime`, `io_bytes_methods`, `io_file_handles`, `join_set_runtime`, `parallel_runtime`, `task_cancellation_runtime`, `task_context_runtime`, `task_runtime`, `task_scope_offload_runtime`, `task_supervisor_runtime`, `types_and_errors`) — every one has exactly one owner in the manifest.

## Gate results (read-only, run just now)

- `scripts/check_stdlib_manifest_schema.py --self-test` PASS
- `scripts/check_stdlib_manifest_schema.py` PASS (surfaces=11, schema_version=2)
- `scripts/check_stdlib_native_intrinsic_allowlist.py --self-test` PASS
- `scripts/check_stdlib_native_intrinsic_allowlist.py` PASS (preamble_files=11, direct_runtime_roots=4)
- `scripts/check_sysroot_stdlib_resource_certification_gate.py --self-test` PASS
- `scripts/check_sysroot_stdlib_resource_certification_gate.py` PASS
- `scripts/check_hir_maintainability_guardrails.py` PASS

The manifest edit does not opt any new surface into `certification_rows`, so the resource certification gate remains bound to the pre-existing `_sifr.fs::opaque_resource_core` row — no new certification obligation is created or missed.

## Findings

No blocker introduced by the manifest edit. Round-1's non-blocking observations (debug double-lock in `__SifrCancellationCarrier::abort`, unused `fallback_hook` getter, missing `debug_assert!(Handle::try_current().is_err())` on `run_coroutine_blocking`, less granular file split than design §H's five-file sketch, per-submission `loop_object` clone, `__sifr_cancel_all` fallback branch note) still stand and remain non-blocking; none are affected by this edit.

## Validation-evidence check

The reported "final gate green (all lanes, 130/130 e2e) + earlier transient LSP-executeCommand 90 s timeout, followed by an isolated smoke pass and a subsequent full-gate LSP smoke pass in 11 s" reads as environmental transient, not a wave-3 regression signal: the timeout was on `executeCommand` and the same lane passed cleanly on rerun both in isolation and inside the full gate.

## Wave-boundary integrity

Const gate `__SIFR_COOPERATIVE_SUPERVISORS_READY = false` is still emitted; supervisors still route through `cancellation.abort_handle()`; `__SifrBlockingTask` remains carrier-free; the manifest edit adds no new codegen or runtime code path — it only records ownership of files that already existed in round 1.

VERDICT: SATISFIED
