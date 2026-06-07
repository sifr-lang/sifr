**VERDICT: FAIL**

Six material gaps would cause inconsistent implementation choices. Listed in order of severity.

---

### Gap 1 — `task.Scope`/`runtime.Scope` vs `TaskGroup` as unified work owner is left open while the canonical example already answers it

**Files**: `ad-hoc-production-concurrency-runtime-platform-substrate.md` §Structured Runtime Work Model; `structured_runtime_work_model.md` §Work Kinds (identical phrasing in both)

Both docs say:
> "`task.Scope` or `runtime.Scope` is the general owner for mixed runtime work. M0 chooses the public name and method placement."

Immediately below, the canonical example shows `spawn_blocking`, `spawn_cpu`, and `spawn_process` on `TaskGroup`. M3's scope says "scoped offload methods on the accepted scope/group API where M0 places them." M4's scope repeats the same deferral. If `TaskGroup` already owns all work kinds in the canonical model, no separate `Scope` type is needed and the "M0 chooses" clause creates a false open question. If a separate `Scope` type IS intended, the canonical example is misleading and M3/M4 implementers will design the wrong surface.

**Suggested fix** (both docs, identical change):
> Replace "`task.Scope` or `runtime.Scope` is the general owner for mixed runtime work. M0 chooses the public name and method placement." with:
>
> "`TaskGroup[E]` is the canonical owner for mixed runtime work under the fail-fast structured-concurrency policy. A distinct `task.Scope` or `runtime.Scope` type is introduced only if M0 identifies a concrete use case `TaskGroup[E]` cannot satisfy; M0 must record that finding before M1 starts."

---

### Gap 2 — M1 Definition of Done does not close the observed-failure → sibling-cancellation question

**File**: `ad-hoc-production-concurrency-runtime-platform-substrate.md` §milestone_concurrency_runtime_1 DoD

The Cancellation Contract and M0 scope both correctly note that M0 must record "whether an observed failure still triggers fail-fast sibling cancellation or only unhandled failures do." M1's DoD closes the exit-reporting side ("explicitly handled child failures are not re-reported as group-exit failures") but says nothing about whether M1 must implement the sibling-cancellation ruling. An implementer reading M1's DoD in isolation would not know this was required, and the decision would remain soft.

**Suggested fix** — add to M1 DoD bullet list:
> - M0's recorded sibling-cancellation policy for observed failures is implemented and has a named representative fixture.

---

### Gap 3 — Process handle shape (Child vs TaskHandle vs ProcessHandle) claimed as recorded but absent from Resolved Decisions

**Files**: `ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` §Planning Reviews ("scoped process handle shape...were recorded in the phase contract"); `ad-hoc-production-concurrency-runtime-platform-substrate.md` §Resolved Decisions table

The execution ledger's review summary for pass-5 says this was "recorded in the phase contract," but the Resolved Decisions table has no row for it. The phase doc §milestone_concurrency_runtime_4 and the structured work model §Process And Worker Policy both still read "M0 decides." M4 implementers have no pre-committed default and no table row to anchor to.

**Suggested fix** — add a row to the Resolved Decisions table in `ad-hoc-production-concurrency-runtime-platform-substrate.md`:

| Decision area | Decision |
|---|---|
| Scoped process spawn return type | M0 is the binding gate; no pre-M0 default. M0 must choose among `Child`, `TaskHandle[Status, SubprocessError]`, and a distinct `ProcessHandle` and record the choice with pipe-ownership semantics before M4 starts. This row must be updated with the M0 outcome before M4's first implementation PR. |

---

### Gap 4 — `sifr.subprocess` freeze status is "may remain" in the model doc but "must" in the phase contract

**File**: `structured_runtime_work_model.md` §Process And Worker Policy

> "Existing `sifr.subprocess` **may remain** frozen or compatibility-only..."

The phase doc §milestone_concurrency_runtime_4 says "Keep existing `sifr.subprocess`...frozen or marked compatibility-only" and the Resolved Decisions table says existing adapters "may remain frozen compatibility-only surfaces." The execution ledger's API Tier Decision Index classifies it as `compat-adapter`. The "may remain" in the model doc implies discretion; an implementer reading only the model doc could interpret this as permission to extend it.

**Suggested fix** — in `structured_runtime_work_model.md` §Process And Worker Policy:
> Change "may remain frozen or compatibility-only" → "must remain frozen and compatibility-only"

---

### Gap 5 — `select` call API syntax is unspecified; M0 scope only asks for result containers

**File**: `ad-hoc-production-concurrency-runtime-platform-substrate.md` §milestone_concurrency_runtime_0 scope and §milestone_concurrency_runtime_1 scope

M1 says `select` is "the named-branch form for a statically known set of awaitable branches" and returns a "winner branch tag." M0 is asked to "Record `race` and `select` result containers" — but not the call syntax. A "statically known, named-branch" form requires a compiler decision: named-kwargs (`select(fetch=coro1, config=coro2)`), a special compiler form / keyword, or an explicit enum-keyed collection. The branch-tag type follows directly from that choice (string literal, generated enum, etc.). Without the call API shape, HIR lowering for `select` in M1 cannot be designed.

**Suggested fix** — add to M0 scope bullet list in `ad-hoc-production-concurrency-runtime-platform-substrate.md`:
> - Record `select` call API syntax: whether it accepts named kwargs, requires a special compiler keyword/macro form, or uses another static-branch mechanism; record the branch-tag type (string literal, generated enum, or other) and how the compiler enforces static-branch identity at compile time. This is a required M0 artifact before M1 starts.

---

### Gap 6 — `TaskGroup[E].spawn_blocking` error-type binding is deferred to M0 but M0 scope does not ask the right question

**Files**: `ad-hoc-production-concurrency-runtime-platform-substrate.md` §Structured Runtime Work Model, §milestone_concurrency_runtime_0, §milestone_concurrency_runtime_3; `structured_runtime_work_model.md` §Work Kinds

Both docs say "Scoped offload inserted into `TaskGroup[E]` maps user errors plus runtime/offload failures into the group's error type or an accepted wrapper such as `WorkerError[E]`; M0 records the exact error shape." But M0's scope only says to record "whether scoped offload/process methods live on `TaskGroup`..." (a placement question). It does not ask M0 to decide the error binding: must `fn` return `Result[T, E]` with a required `E: From<WorkerRuntimeError>` bound? Or does the group's error type become `WorkerError[E]` (forcing all async tasks in the same group to align to that)? Or is there an explicit user-supplied error mapper? The `JoinSet` signatures are fully specified (`JoinSet.join_all() -> list[Result[T, WorkerError[E]]]`), but the `TaskGroup` analog is not. M3 implementers who add `TaskGroup.spawn_blocking` have no contract to implement against.

**Suggested fix** — add to M0 scope bullet list:
> - Record the exact error-type binding for `TaskGroup[E].spawn_blocking` and `TaskGroup[E].spawn_cpu`: whether the callable must return `Result[T, E]` (requiring `E` to be representable from runtime offload errors), whether the group must be typed as `TaskGroup[WorkerError[E]]`, or whether an explicit mapper closure is required. This must be recorded as a Resolved Decision entry before M0 closes.

---

### Non-blocking polish (no file changes required before M0)

**A.** Both docs use "do not cross **unrelated** `await` points" for lock guards. "Unrelated" is non-standard and implies some await points are fine. Simpler: sync guards cannot cross any `await`; async guard await rules are M0-decided and API-marked. (`structured_runtime_work_model.md` §Synchronization Model; `ad-hoc-production-concurrency-runtime-platform-substrate.md` §milestone_concurrency_runtime_2)

**B.** `structured_runtime_work_model.md` has no explicit rule that Tokio/Rayon/Crossbeam/etc types must not appear in public Sifr APIs. The phase doc has this in §Rust Lowering Contract. One sentence in the model doc's §Public Namespace Policy would close the gap without duplication.

**C.** The `Task` vs `TaskHandle` alias question ("M0 records a concrete reason to expose both names") should prompt M0 to audit existing generated-code usages of `Task`/`BlockingTask` first — this is implied but would benefit from an explicit bullet in M0's scope so it isn't skipped.
