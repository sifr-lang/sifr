I've inspected the plan, the normative protocol spec (§Opaque Lifecycle And Context Managers), and current main. Here is a concrete, repository-specific review.

## Where the plan lands cleanly

- **M4 already ships everything the plan depends on**: `PythonCleanupPolicy`, `python_opaque_classes` registry in `LowerCtx`, `must_use_obligation_for_type`, `ForeignObject` lock discipline, `enter_context`/`exit_context`/`exit_context_with_error` glue in `_sifr.python`, and the `SIFR-PYRES-0002` guard around `cleanup=context` (crates/sifr_lowering/src/lower/python_interop.rs:336-340). Activation is a matter of flipping that reserved arm.
- `SIFR-PYCTX-0000` family and `SIFR-PYCTX-0001` slot are already reserved (crates/sifr_diagnostics/src/codes/registry/registry_entries/reserved.rs:18,52) — allocation only.
- `PythonError` today is `String`-only (crates/sifr_runtime/src/python/object_ops.rs:29) — already `Send + Clone + Eq`, so adding a replay slot behind an `Arc` keeps it sendable if you keep only the store-slot id in the arc, not `Py<PyAny>`.

## Concrete flaws / gaps in the current decomposition

1. **`cleanup=context` obligation kind is currently indistinguishable from `close`.** `must_use_obligation_for_type` at crates/sifr_lowering/src/lower/mod_context.rs:216-234 returns `Some` for any non-Drop policy and today's discharge points (returns, owning aggregates) apply uniformly. The doc requires `cleanup=context` to be dischargeable **only** by a `with` block — a plain `return` must **not** discharge it. The plan's item 1 doesn't call this out. Fix: split the obligation into an enum (`CloseLike`, `ContextOnly`, `AsyncContextOnly`) stored in `live_must_use_bindings` and reject non-with discharge of `ContextOnly`.

2. **`PythonError` Eq/PartialEq semantics silently change.** Adding `replay: Option<Arc<ReplayCapability>>` to the struct at crates/sifr_runtime/src/python/object_ops.rs:29-35 will make `derive(PartialEq, Eq)` compare pointer identity, breaking every existing runtime test (e.g. crates/sifr_runtime/src/python/object_ops_tests.rs:143). Fix must be explicit: hand-implement `PartialEq`/`Eq` over public fields only. The plan should state this.

3. **`SifrBoundaryError` registration is unowned.** The runtime needs to install a stable Python exception class once at CPython/bridge init before any user code runs and cache the type object. The plan says "create a SifrBoundaryError" but doesn't say where the class comes from or when it's registered. Suggested owner: a new `crates/sifr_runtime/src/python/context_ops.rs` module invoked from `crates/sifr_runtime/src/python.rs` initialization (the same place M4's runtime init lives).

4. **`python.ExitCause`/`python.ExitDecision` visibility.** The doc requires that user source cannot construct `ExitCause` and cannot directly call `__exit__` (protocol doc lines 262-263), yet `__exit__` in the example takes `cause: python.ExitCause` in the Sifr signature. The plan says "compiler-known ExitCause/ExitDecision" but doesn't specify enforcement. Recommendation: declare `class ExitCause`, `enum ExitCauseKind`, `enum ExitDecision`, `class SifrBoundaryError(PythonError)` in `stdlib/_sifr/python.sifr` and add a sysroot-only construction guard driven by the existing `LoweringSourceOrigin::SysrootPrivateDeclaration` flag already used for private stdlib declarations. Also reject any HIR call to a `@python.context.exit`-marked method outside `PythonWith` lowering (this rejection is missing from Task 1).

5. **`file-size guardrail pressure` is worse than "likely":**
   - `crates/sifr_runtime/src/python/object_ops.rs` is at 877 lines. Adding `PythonError.replay`, its accessors, and the from_pyerr triple capture will exceed 900. Split `PythonError` into `python_error.rs` **as part of PR 2**, don't leave for post-hoc refactor.
   - `crates/sifr_lowering/src/lower/async_with.rs` is at 874 lines. Do **not** put sync Python-with lowering here — create `crates/sifr_lowering/src/lower/python_with.rs`.
   - `crates/sifr_lowering/src/lower/python_interop.rs` at 758 lines gets context decorator validation + signature checks + entered-type checks; likely to blow the cap. Extract `python_interop/context.rs`.

6. **`HirStmt::PythonWith` variant vs. flag on `HirStmt::With`.** A dedicated variant is correct (as the plan proposes) because `nonlocal_support`, `cfg`, `flow_graph/effects`, `nested_function_inference/state_collection`, `container_literal_specialization`, `numeric_sentinels`, `for_loop_safety`, `sequence_guard_detection`, `class_body_lowering`, and every codegen descent in crates/sifr_codegen/src/stmt_support_emitter must know the `as` binding is a **borrow that cannot escape**. A silent extra flag on `With` would leave 15+ passes reasoning about `as x` as ordinary and produce ownership false-negatives. Enumerated consumers to update:
   ```
   sifr_ir/src/hir_nodes.rs
   sifr_lowering/src/lower/statements/statement_dispatch.rs
   sifr_lowering/src/lower/{async_with,for_loop_safety,container_literal_specialization,
                            numeric_sentinels,sequence_guard_detection,function_flow,
                            nonlocal_support,async_for,name_resolution_snapshot_tests,
                            hir_snapshot_tests,cfg,flow_graph,flow_graph/effects,
                            classes/class_body_lowering,nested_function_inference/state_collection}.rs
   sifr_codegen/src/{lib_emitter_state,lib_runtime_needs,error_refs}.rs
   sifr_codegen/src/stmt_support_emitter/{stmt_block,stmt_block_helpers,loops_try_finally,
                                          try_error_helpers}.rs
   sifr_codegen/src/lower_stmt/candidate_and_validation.rs
   ```
   All of these currently match `HirStmt::With` / `AsyncWith` explicitly. Missing one will let the "context-scoped borrow" leak.

7. **Body-outcome codegen.** RAII / Drop-guard on the manager is unsafe because `__exit__` is fallible and its result must be observed — Drop can't return `Result<PythonError>`. The plan doesn't spell this out, but every codegen author will reach for a guard first. State the closure-with-outcome-enum approach explicitly:
   ```
   enum SifrPyCtxOutcome<T, E> {
       Normal, Return(T), Break, Continue,
       PythonErr(PythonError), OrdinaryErr(E),
   }
   ```
   `return`/`break`/`continue`/`raise`/`?` inside the body must be rewritten to `Outcome::…` constructors before invoking the exit, then rematerialized after exit runs. This is a nontrivial rewrite already present in shape in `try_lower_try_except_stmt_for_ir` (crates/sifr_codegen/src/stmt_support_emitter/loops_try_finally.rs:189) — reuse the closure-body scaffolding.

8. **Secondary cleanup evidence in Sifr's Result machinery.** The doc requires exit's own `Err(PythonError)` to attach as "secondary evidence" for unsuppressible causes, without altering primary error semantics. Sifr `Result` doesn't carry a secondary slot. The smallest correct move:
   - When primary is `PythonError`, append cleanup exit's `exception_type`/`message` into the primary's `context` string (already a free-form redacted field).
   - When primary is an ordinary Sifr error, route the secondary `PythonError` through a runtime diagnostics sink (a `tracing::warn!` or the existing outstanding-resource diagnostic channel) so tests can observe it. **Do not** change the primary Result payload. Add this to Task 2 explicitly.

## Suggested implementation order (5 PRs, each independently reviewable)

**PR-1 — Types, diagnostics, and decorator validation**
- Declare `ExitCauseKind`, `ExitCause`, `ExitDecision`, `SifrBoundaryError` in `stdlib/_sifr/python.sifr` with sysroot-only construction.
- Split `python_interop.rs` → add `lower/python_interop/context.rs`; replace `reserved_cleanup(ctx, "context", …)` with real acceptance producing `PythonCleanupPolicy::Context`.
- Recognize `@python.context.enter`/`@python.context.exit` on methods; validate signatures + entered-type policy (Same-identity | non-opaque | distinct opaque with `cleanup=drop`; reject others with `SIFR-PYCTX-0003`).
- Reserve `SIFR-PYCTX-0002…0008` slots.

**PR-2 — Runtime `PythonError` replay + `SifrBoundaryError` + exit API**
- Split `object_ops.rs` → `python_error.rs`; extend `PythonError` with `replay: Option<Arc<ReplayCapability>>`; hand-implement `PartialEq/Eq/Hash` over public fields only.
- New `crates/sifr_runtime/src/python/context_ops.rs`:
  - `register_boundary_error()` (called from init).
  - `context_exit_normal(&ObjectHandle) -> Result<ExitDecisionRaw, PythonError>`
  - `context_exit_python_error(&ObjectHandle, &mut PythonError) -> Result<ExitDecisionRaw, PythonError>`
  - `context_exit_sifr_cause(&ObjectHandle, cause: SifrExitCauseMeta) -> Result<ExitDecisionRaw, PythonError>`
- `ReplayCapability` = `Arc<Mutex<StoredTriple>>` holding three `ForeignObject` store slots; drop schedules through `PENDING_RELEASES`; `resolve_under_gil` re-materializes `(type, value, tb)` without consuming.

**PR-3 — HIR variant + ownership analysis**
- Add `HirStmt::PythonWith { manager, manager_ty, entered: Option<(String, EnteredValueKind)>, body, span }` and `EnteredValueKind::{SameIdentityBorrow, DistinctOpaqueDrop(Type), Converted(Type)}`.
- Split `MustUseKind::{CloseLike, ContextOnly, AsyncContextOnly}`; discharge `ContextOnly` only through `PythonWith`.
- Statement dispatch: at `Stmt::With(..)` if the item's type resolves to a `@python.opaque(cleanup=context)` class, emit `PythonWith` instead of `With`.
- Ownership pass: mark entered borrow non-escaping, non-moveable, non-consumable.
- Update every `HirStmt::With | AsyncWith` matcher listed above.

**PR-4 — Codegen dedicated lowering**
- New `crates/sifr_codegen/src/stmt_support_emitter/python_context.rs`:
  - Body rewrite to `SifrPyCtxOutcome` closure.
  - Post-body dispatch to `context_exit_normal/python_error/sifr_cause`.
  - Suppression only for `PythonException` cause.
  - Ensure entered distinct-drop handle drops before exit runs.
- Wire dispatcher in `stmt_support_emitter/stmt_block.rs` **before** the `HirStmt::With` arm.
- Snapshot tests: all 6 body outcomes × normal/exit-fail combinations.

**PR-5 — Demo and end-to-end evidence**
- `demos/python_context_transaction.sifr` running a real sqlite3 transaction, plus a sqlite3-based negative fixture asserting rollback + zero outstanding.
- `verification/areas/python_interop/context_sync/` positive/negative/replay-nested/exit-failure/suppression fixtures.
- Update `plans/issues/active/ad-hoc-declaration-first-python-interop.md` M5 checkbox.

## Must-have tests

Declaration:
- `__enter__` returns `Self` → accepted; distinct opaque `cleanup=drop` → accepted; distinct opaque `cleanup=close/context/async_close/async_context` → `SIFR-PYCTX-*`; non-opaque converted → accepted.
- `__exit__` signature deviations (missing `own self`, wrong cause type, wrong return) → each a distinct diagnostic.
- Direct source call to `@python.context.exit` method → rejected.

Ownership / must-use:
- `cleanup=context` value dropped without with → rejected.
- `cleanup=context` value returned → rejected (context obligation does not transfer).
- `with T() as t: return t` → rejected (escape).
- `with T() as t: t.close()` → rejected (independent close).
- `with T() as t: consume(t)` where `consume(own T)` → rejected.

Codegen/runtime end-to-end matrix (per outcome):
- Normal → exit called with `(None,None,None)`; count == 1.
- Return, Break, Continue → exit called once; outer control-flow resumes.
- Python exception → live triple replayed to `__exit__`; truthy suppresses; falsy propagates.
- Ordinary Sifr error → `SifrBoundaryError` passed; truthy result ignored; ordinary error still propagates.
- Body Normal + exit `Err` → exit error is primary.
- Body Ordinary/Python/Return + exit `Err` → primary preserved, cleanup attached as secondary evidence.

Replay:
- Nested `with A: with B: raise` — both exits see the same `type/value/traceback` identity; final drop releases exactly once via pending-release queue; assert `outstanding == 0`.
- `PythonError` clone shares replay; last drop releases.

Backward compat:
- Native Sifr `class C: def __enter__/def __exit__` still lowers through `HirStmt::With` with argless drop-style exit — no `PythonWith`, no ExitCause.

Live evidence:
- Compiled `python_context_transaction.sifr` binary runs sqlite3 commit + rollback paths with zero outstanding foreign objects on both success and failure paths.

NOT SATISFIED — the plan is close to implementable but omits the following decisions that must be settled before implementation starts:
1. `MustUseKind` split so `cleanup=context` cannot be discharged by return.
2. `PythonError` manual `PartialEq/Eq`.
3. `SifrBoundaryError` install site (init hook in `sifr_runtime::python`).
4. Compiler-known sysroot-only construction of `ExitCause`/`ExitDecision` + direct-`__exit__`-call rejection.
5. Body-outcome enum + closure rewrite as the exclusive codegen shape (no Drop-guard).
6. Secondary cleanup evidence routing (append-to-`context` for Python primary; runtime diagnostic sink for ordinary primary) — no change to Sifr `Result`.
7. Module splits committed in the same PRs that would exceed 900 lines: `object_ops.rs → python_error.rs` (PR-2), `python_interop.rs → python_interop/context.rs` (PR-1), `stmt_support_emitter/python_context.rs` (PR-4). Do not defer.
