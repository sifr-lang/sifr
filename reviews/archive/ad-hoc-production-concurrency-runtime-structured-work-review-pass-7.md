Now I'll synthesize the findings across all three documents.

---

**VERDICT: FAIL**

One material blocker. Three non-blocking polish items follow.

---

## Material Blocker

### 1. `JoinSet.join_all()` return type hard-coded in M3 without cross-reference to M0's TaskGroup offload error binding gate

**Files/sections:**
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md` — M3 scope, `JoinSet` API line; Resolved Decisions table, "TaskGroup offload error binding" row
- `internal_docs/structured_runtime_work_model.md` — Work Kinds table (JoinSet listed without error-wrapper discussion)

**Problem:**

M3 scope specifies `JoinSet.join_all().await -> list[Result[T, WorkerError[E]]]` — hard-coding a `WorkerError[E]` wrapper. M0 simultaneously owns the TaskGroup offload error binding decision with three options:

1. `Result[T, E]` where runtime offload errors are absorbed into `E` (user-defined variant)
2. `TaskGroup[WorkerError[E]]`
3. explicit mapper closure

If M0 chooses option 1 (E absorbs runtime errors), TaskGroup and JoinSet use inconsistent wrapping for the same underlying operations (`spawn_blocking`, `spawn_cpu`): TaskGroup has no wrapper, JoinSet wraps in `WorkerError[E]`. The M0 gate text for this decision does not mention JoinSet's already-specified return type, so the author of M0 can resolve the TaskGroup binding without realizing they need to reconcile JoinSet.

**Required fix:**

Add a cross-reference note to the Resolved Decisions "TaskGroup offload error binding" row:

> *This decision must also be reconciled with `JoinSet.join_all().await -> list[Result[T, WorkerError[E]]]` specified in M3. If M0 chooses `Result[T, E]` with E absorbing runtime errors, M0 must either update M3's JoinSet return type to `list[Result[T, E]]` or record an explicit rationale for why JoinSet and TaskGroup use different error-wrapping patterns for the same offload operations.*

No other text needs to change. The gate in M0 scope should also reference this: "Record the exact error-type binding for `TaskGroup[E].spawn_blocking` and `TaskGroup[E].spawn_cpu`, and confirm alignment or record a rationale difference against `JoinSet.join_all() -> list[Result[T, WorkerError[E]]]` (M3 scope)."

---

## Non-Blocking Polish

### P1. `CancelScope` type name retains hedged wording in `structured_runtime_work_model.md`

**File/section:** `internal_docs/structured_runtime_work_model.md`, Cancellation And Failure section (line 151)

Current: *"a Sifr-owned `CancelScope` or cancellation handle is a settled stable API"*

The "or cancellation handle" hedge is vestigial. The function `cancel_scope` is settled; the type name is intentionally left to M0. The wording conflates the two. Suggested: *"a Sifr-owned cancellation scope handle (named `CancelScope` or another M0-recorded name) is a settled stable API."*

### P2. Canonical example leaves `child` handle uninteracted — no pipe-access pattern shown

**Files/sections:** Both the substrate doc (Structured Runtime Work Model section) and `structured_runtime_work_model.md` (Product Decision section)

Both show:
```sifr
child = group.spawn_process(process.Command("worker"))  # supervised by group scope
```

`child` is assigned but never used. Since the scoped process spawn return type (M0 gate) is specifically motivated by pipe-ownership, not showing any pipe interaction makes the example incomplete as a directional signal. Consider adding a one-line comment showing intent: `# use child.stdin / child.stdout for pipe access (type settled in M0)`.

### P3. Resolved Decisions "JoinSet submission API" row gives `cancel_all()` return type but omits `join_all()`'s

**File/section:** `issues/ad-hoc-production-concurrency-runtime-platform-substrate.md`, Resolved Decisions table, "JoinSet submission API" row

The row ends: *"`join_all().await` and `cancel_all().await -> list[CancelOutcome]` consume the set."* This reads as if `cancel_all`'s type annotates both, which is asymmetric with M3's explicit `list[Result[T, WorkerError[E]]]` for `join_all`. Suggested: *"`join_all().await -> list[Result[T, WorkerError[E]]]` and `cancel_all().await -> list[CancelOutcome]` consume the set."*

---

**Summary:** One binding dependency is missing from the M0 TaskGroup error-binding gate — it does not require the implementer to reconcile or explicitly diverge from JoinSet's already-specified `WorkerError[E]` return type. This gap can produce an incoherent error-wrapping pattern across two APIs that share the same underlying offload operations. All other pass-6 remediations (`cancel_scope` settled stable, supervised process examples, `spawn_scoped` orientation, `race`/`select` loser evidence tied to `CancelOutcome`) are correctly reflected across the three documents.
