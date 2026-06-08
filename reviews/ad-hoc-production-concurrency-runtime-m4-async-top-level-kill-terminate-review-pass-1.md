I've inspected the changed surfaces. Here's the assessment.

## Result: PASS

### Correctness

- `lib/sifr/process.sifr` (lines 442–447): `async_kill(child: AsyncChild)` and `async_terminate(child: AsyncChild)` are defined without `own`, so `AsyncChild` (a class — non-Copy) takes the default `Borrow` convention. The handle survives the call, so the subsequent `await child.wait()` observation in the fixture is well-defined. Both helpers forward to `process_async_kill(child._handle)` / `process_async_terminate(child._handle)`, identical to the corresponding method-form bodies — no divergence between method-form and top-level semantics.

- `leaves_and_plain_calls.rs:419–420` + helper at 699–702: the change correctly gates only the awaited-`HirExpr::Call` branch on a generic `args.iter().any(|arg| !is_copy_type_for_codegen(arg.ty()))` predicate. It is **not** an `async_kill`-only special case; any awaited plain call with at least one non-Copy argument defers to the signature-aware path so borrow/move conventions match the callee signature. `__sifr_task_sleep` is special-cased ahead of the new check, preserving the existing sleep lowering. Pure-Copy-arg awaits (e.g., `await foo(42, true)`) still take the fast path. The single-line `return None` is the standard convention used elsewhere in this file for "let the structured emitter handle it."

- The local-validation evidence already confirms preserved behavior for `process_async_spawn_wait`, `process_async_child_kill_terminate`, and the new fixture — those are exactly the awaited calls with non-Copy args that now route through the structured emitter.

### Tests

- `process_async_top_level_kill_terminate.sifr` covers: top-level borrowed `async_kill(killed_child)` with later `wait` observation (SIGKILL/signal 9); top-level borrowed `async_terminate(terminated_child)` with later `wait` (SIGTERM/signal 15); a closed-handle re-kill case asserting the typed "closed or unknown" `ProcessError`. The 7-element `actual` list matches the 7 expected truths. Fixture shape mirrors the method-form sibling, which keeps coverage parallel and reviewable.

- Both validation manifests (create-pr and merge) are updated. JSON is well-formed (validation log confirms `json.tool` pass).

### Docs honesty

- Traceability surface row collapses method-form + top-level helpers honestly: "Method-form and top-level `async_kill(child)` request forceful termination while preserving the handle for later wait observation" — accurate. The non-Unix `async_terminate` typed-unsupported wording is retained.
- Host matrix row cites both fixtures and keeps Windows host-limited — consistent with the substrate-execution ledger.
- Follow-up boundaries correctly drop "top-level async kill/terminate helper shape" now that this wave delivers it.
- CPython family mapping row picks up the new fixture in the planned-for-M4-follow-up bucket.
- Status line and ledger entry correctly mark this wave as "in progress" pending merge.

### Phase-contract compliance

- No public CPython-shaped adapter; `sifr.process` keeps the canonical surface.
- No new user-triggerable runtime panic paths — Sifr code returns `Result[None, ProcessError]`; underlying intrinsics already produce typed errors (validated by the closed-handle assertion).
- File-size guardrail: `leaves_and_plain_calls.rs` is 823 lines, `process.sifr` is 549 lines — both under 900.
- The fix preserves the structured emitter's ownership/convention model rather than papering over the symptom in `async_kill` alone — root-cause fix.

### Non-blocking polish

1. `lib/sifr/process.sifr` lacks an explicit `own`-marker comparison: `async_wait(own child)` consuming vs `async_kill(child)` / `async_terminate(child)` borrowing is implicit from the missing keyword. A `# borrowed: preserves AsyncChild handle for later wait` one-liner above the new defs would make the contract eye-readable without changing semantics. Optional.
2. The new untracked `reviews/ad-hoc-production-concurrency-runtime-m4-async-top-level-kill-terminate-review-pass-1.md` is currently empty (0 bytes). Per prior-wave precedent (M0a pass-1 explicitly flagged "empty review artifact" as a blocker), make sure this is populated with the actual review content before the PR is opened — otherwise it would be a blocker on the next pass.
3. Host-matrix sentence "Both preserve subsequent async wait observation and closed-handle typed errors" reads slightly ambiguously (which two?). Replacing "Both" with "Both fixtures" clarifies the antecedent. Trivial.
4. The lowering predicate is named `await_call_needs_convention_aware_lowering` but is structurally just `any-non-copy-arg`. A brief doc-comment naming the underlying reason ("Non-Copy args need signature-aware borrow/move conventions; bypassing emits raw idents and breaks borrowed parameters") would help the next reader. Optional.

None of these block the PR.
