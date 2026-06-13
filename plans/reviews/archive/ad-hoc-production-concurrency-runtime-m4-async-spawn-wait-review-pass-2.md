# Ad Hoc M4 Async Process Spawn/Wait Review — Pass 2

Scope: re-review of the M4 async process spawn/wait wave after the pass-1 post-cleanup pass.

Reviewer date: 2026-06-08.

## Verdict

`PASS` — pass-1 residual note #1 (`AsyncChild._waited`) is fully resolved, the new `stdout(Stdio("pipe"))` deferral assertion is correct, no new blockers were introduced, and the refreshed ledger validation evidence is consistent with the on-disk artifacts. The wave remains ready to proceed to PR.

## What changed since pass 1

The current working tree diff (`git diff --stat HEAD`) shows the same 12 files touched by the spawn/wait wave; the only post-pass-1 deltas relative to the pass-1 audit are the unused-field removal on `AsyncChild`, the additional stdout-mode rejection branch in the fixture, and a refreshed ledger validation block.

## Pass-1 residual note #1 — resolved

- `lib/sifr/process.sifr:162-169` now declares `AsyncChild` with only `_handle: int`, `__init__(self, handle: int)`, and `wait(self) -> Awaitable[Result[Status, ProcessError]]`. The `_waited: bool` field and its initialization are gone.
- `crates/sifr_stdlib/src/process.rs:63-72` matches: `process_async_child_class()` now declares `fields: vec![("_handle".to_string(), Type::Int)]` only. No stale field metadata survives.
- A repository-wide `grep -n _waited` confirms `_waited` now appears only on the sync `Child` class (`lib/sifr/process.sifr:116, 120, 128, 130`) — the deliberate existing pattern where the sync side uses an instance flag to fail fast — plus an unrelated local Rust binding `__waited` inside the async run-timeout helper (`crates/sifr_codegen/src/preamble/process_async_runtime.rs:413-414`) and historical references in older review documents. None of those collide with the async wave.

The async child no longer carries dead state on its public surface; the runtime handle table remains the single source of truth for double-wait detection, exactly as pass 1 recommended.

## Added stdout-mode rejection coverage — correct

- Fixture (`crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr:43-49`) constructs `Command("true")`, sets `stdout(Stdio("pipe"))`, awaits `async_spawn`, and asserts the resulting `ProcessError.message` contains the substring `"owned pipe"`.
- Generated runtime guard (`crates/sifr_codegen/src/preamble/process_async_runtime.rs:586-594`) is symmetric: `if stdin_mode != "inherit" || stdout_mode != "inherit" || stderr_mode != "inherit" { return Err(ProcessError { message: "async process spawn stdio modes require async owned pipe support".to_string() }); }`. The literal `"owned pipe"` substring is in fact emitted as part of `"async owned pipe support"`, so the fixture assertion matches the generated text exactly.
- The new stdout case sits next to the existing stdin-pipe-mode case (`crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr:35-41`) and the `stdin_bytes(...)` rejection (`:27-33`). Together these now exercise the `has_stdin` guard, the explicit `stdin` stdio-mode guard, and the explicit `stdout` stdio-mode guard. The `stderr` arm of the symmetric runtime guard is the only branch not directly fired by the fixture; the rejection is still a single boolean covering all three, so this is residual coverage, not a correctness gap.
- Result count was updated to match: `assert actual == [True, True, True, True, True, True, True, True]` (`:53`), 8 booleans for the 8 appends across the 5 try blocks (1+1+1+1 from the spawn/wait/method/second-wait flow, 1 each for stdin_bytes/pipe-stdin/pipe-stdout). The arithmetic lines up.

## Other consistency checks executed

- `process_async_child_class()` field shape matches `lib/sifr/process.sifr` 1:1 (`_handle: int` only). The HIR-visible class therefore matches what intrinsic metadata advertises; nothing else lowering through the type checker can see a non-existent field.
- Registry wiring (`crates/sifr_codegen/src/intrinsics/registry.rs:626-633`) and lowerers (`crates/sifr_codegen/src/intrinsics/registry/process_async.rs:82-104`) are unchanged from pass 1. `process_async_spawn` still takes 9 args and clones `stdout_mode`/`stderr_mode`; `process_async_wait` still takes 1 arg. Both gate on `Some(StdlibFeature::Tokio)`.
- Shared-prelude classification (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:335-403, 421-444`) still distinguishes `__sifr_process_async_spawn(` from `__sifr_process_async_run(` via paren-suffix scanning, and the AST collector and `is_shared_prelude_item` branches for the new statics and functions are unchanged.
- Driver aggregation (`crates/sifr_codegen/src/lib_modules_and_codegen.rs:389-400, 427-432, 599-611`) still OR-aggregates `needs_spawn`/`needs_wait` across modules and continues to gate emission on per-module use rather than blanket-emitting whenever any async helper is needed.
- Manifests (`verification/validation_lanes/create_pr_e2e_manifest.json:90-93`, `verification/validation_lanes/merge_e2e_manifest.json:105-108`) still list `process_async_spawn_wait` in lexicographic position immediately after `process_async_output_timeout` for both lanes.
- Traceability (`verification/stdlib/concurrency_runtime_m4_process_traceability.md:19, 33, 39-40`) keeps the dedicated `AsyncChild`/`async_spawn`/`async_wait`/`AsyncChild.wait` row with the documented deferrals, and continues to reference `process_async_spawn_wait` from the CPython family mapping and validation-coverage tables without overclaiming.
- Host matrix (`verification/platform/supported_host_matrix.md:23`) keeps the `Async subprocess spawn/wait` row as `supported` (Linux/macOS) and `host-limited` (Windows), with notes that explicitly reference inherited-stdio scope and the deferred owned-pipe/Windows work.
- Phase ledger (`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:425-427, 953-977`) lists the wave as `in progress`, records the targeted local checks (including the new `cargo run` on `process_async_spawn_wait`), and quotes a fresh full create-pr lane PASS at `wall_time=282.19s`, `pass=102`, `cache_hits=22/26`, `report_signature=5e93ca9f74a9781c`. No premature merge claim.

## Spot checks against the pass-2 brief

- Pass-1 residual note #1 fully resolved: ✔ (see above).
- Added stdout-mode assertion correct: ✔ (substring matches generated message; guard is symmetric).
- New blocker or mismatch introduced: none found. Generated helper emission still independently gated for spawn vs wait vs the existing run/output/timeout helpers, so existing async users do not accidentally pull in the spawn/wait state. The phase ledger evidence matches the manifests and fixtures actually present in the tree.

## Residual non-blocking notes carried forward

Notes 2–5 from pass 1 remain accurate and non-blocking:

- AST-collector branches for the spawn statics/`__sifr_next_process_async_child_id` remain effectively dead under current codegen (matching the sync-child precedent).
- The fixture still does not exercise Unix signal-status flow through async wait; `process_signal_status` continues to carry that evidence.
- `crates/sifr_codegen/src/preamble/process_async_runtime.rs` is 798 lines — unchanged from pass 1, still under the 900-line cap, but the responsibility-based split should be planned before the next async-process slice rather than after.
- Explicit `stderr(Stdio("pipe"))` rejection is still not directly fixtured. The symmetric runtime guard makes this residual coverage rather than correctness.

## Bottom line

The two pass-1 deltas (drop `AsyncChild._waited`, add stdout-pipe-mode rejection coverage) are implemented correctly, do not introduce new regressions, and leave the rest of the wave artifacts (intrinsic metadata, lowerers, generated runtime, prelude classification, manifests, traceability, host matrix, phase ledger) self-consistent. PASS.
