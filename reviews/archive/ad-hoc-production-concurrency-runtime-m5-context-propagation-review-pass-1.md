PASS

Findings (none blocking — all reviewer-flagged risks check out):

**Walker/HIR coverage of `TaskGroup { context }` (✓)**
- `crates/sifr_ir/src/hir_nodes.rs:106-110` — variant changed to `TaskGroup { context: Option<HirExpr> }`.
- All consumers descend into `context: Some(_)`:
  - `crates/sifr_codegen/src/hir_analysis/traversal/traversal_impl.rs:605-613` (walks the context expr).
  - `crates/sifr_lowering/src/flow_graph/effects.rs:252-256` (emits effects for the context).
  - `crates/sifr_codegen/src/error_refs.rs:299-310` (collects refs from context).
  - `crates/sifr_codegen/src/lower_stmt/candidate_and_validation.rs:343-349` (validates the context shape).
  - `crates/sifr_lowering/src/lower/nonlocal_support.rs:201-205` (call detection through context).
- `TaskGroup { .. }` match-all (no descent needed) is consistent everywhere else: `lib_runtime_needs.rs:648`, `task_owner_scope_state.rs:40,60,66`, `async_with.rs:843,857`, `with_yield_and_match.rs:124`, `async_with_and_for.rs:199`.

**Duplicate task-local / helper emission (✓)**
- `crates/sifr_codegen/src/preamble/task_context_runtime.rs:19-74` exposes two preamble builders, each gated on `include_task_local`.
- `lib_modules_and_codegen.rs:466-469,656-666` computes `stdlib_emits_task_context` from preamble substring match (`__sifr_task_current_context` or `__SIFR_TASK_CONTEXT_LABEL`) and `uses_task_current_context` excludes the case where stdlib already supplies the helper. Across the realistic matrix (stdlib path always emits both via the no-task-scope branch; user modules then see `stdlib_emits_task_context=true` and skip both emissions), the `tokio::task_local!` declaration is emitted exactly once per generated crate.
- The test entrypoint at `crates/sifr_codegen/src/entrypoints.rs:73` unconditionally passes `include_task_local=true` to the scope-extension builder, but `generate_rust_test` does not link against a stdlib preamble and never calls `build_task_current_context_items`, so no double-declaration arises.

**Context type-check (✓ honest enough for this slice)**
- `crates/sifr_lowering/src/lower/task_context_keywords.rs:42-66` accepts `None` and otherwise lowers the expression, requiring `Type::Class { name == "Context", fields contain ("name", Type::Str) }`. Strings, ints, and arbitrary classes are rejected with `TYPE_MISMATCH`. The match is structural rather than nominal, which is permissive in principle (a user-declared `class Context { name: str }` would pass), but harmless: any class that satisfies this shape also satisfies the runtime `Display` constraint (Sifr emits `impl Display` for classes with `__str__`), and the diagnostic text correctly names the expected surface. Fail fixture exercises the rejection (`crates/sifr/tests/e2e/fail/task_context_propagation_rejected.sifr` with `ctx="not-context"`).

**Runtime spawn override + restoration (✓)**
- `crates/sifr_codegen/src/preamble/task_context_runtime.rs:36-48` — `__sifr_spawn_(infallible|result)_with_context` swaps `self.context_label` via `Option::replace`, delegates to the no-context spawn, then restores the previous label. The no-context path in `preamble/task_runtime.rs:438,567` captures `child_context_label = self.context_label.clone()` *before* `tokio::spawn`, so the swap is observed by the spawned future and the group label is restored synchronously for the next call.
- HIR lowering routes ctx-bearing spawns to the with_context variants: `task_scope_calls.rs:116-131` + `lower_expr/iterators_and_callables.rs:369-376` + `lower/async_with.rs:340-348` (the spawn-detection helpers learned both new methods).

**Tests, docs, manifests (✓)**
- New pass fixture `crates/sifr/tests/e2e/pass/task_context_propagation_basic.sifr` covers default `current_context()`, `TaskGroup(ctx=…)` inheritance, `spawn_scoped(..., ctx=…)` override, and restoration after the group ends.
- Updated fail fixture targets the non-Context branch.
- Lowering unit tests `expressions_tests/task_runtime_m1.rs:14-115` cover Context acceptance for `TaskGroup`, Context acceptance for `spawn_scoped`, and structural rejection.
- `verification/validation_lanes/{create_pr,merge}_e2e_manifest.json` adds `task_context_propagation_basic`.
- `verification/platform/supported_host_matrix.md` and `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md` move propagation from in-progress/planned to supported, name the new fixture, and explicitly disclaim Python `contextvars` mutation — matching the contract.

Nothing in the diff claims Python contextvars semantics; propagation is label-based via `tokio::task_local!` wrapped per spawned future. Lane signals as reported (123/0, advisory warm budget only) match the change footprint.
