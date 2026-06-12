I have everything I need. Here is my assessment.

---

**VERDICT: FAIL**

One material blocker remains. All nine pass-5 claimed remediations are in place, but a contradiction introduced by including `cancel_scope` in the Resolved Decisions table was not propagated back into M1's scope description or the model doc.

---

## Material Blocker

### `cancel_scope` three-way contradiction

**Location 1 — substrate.md, Resolved Decisions table, line 833:**
> Stable production APIs are `TaskHandle`, `TaskGroup[E]`, `spawn_scoped`, `sleep`, `timeout`, `deadline`, **`cancel_scope`**, `join_all`, `race`, and `select`.

The Resolved Decisions preface states these are pre-settled ("M0 records evidence for these decisions; it does not reopen them without a new issue"). `cancel_scope` is therefore a **settled stable API**.

**Location 2 — substrate.md, M1 scope, line 431:**
> `cancel_scope` groups multiple child operations under a Sifr-owned cancellation scope **if M0 accepts that public surface**.

**Location 3 — structured_runtime_work_model.md, line 151:**
> Tokio or tokio-util token types must not leak publicly; a Sifr-owned `CancelScope` or cancellation handle **may be exposed if the language model needs it**.

Locations 2 and 3 treat `cancel_scope`/`CancelScope` as pending M0 approval. Location 1 has already settled it. An M1 implementer cannot determine whether to implement `cancel_scope` unconditionally or wait for M0's formal acceptance, which is a direct implementation-inconsistency risk.

**Suggested fix:**

In `substrate.md`, line 431, replace:
> `cancel_scope` groups multiple child operations under a Sifr-owned cancellation scope if M0 accepts that public surface.

With:
> `cancel_scope` groups multiple child operations under a Sifr-owned cancellation scope; it is a settled stable API per the Resolved Decisions table. M0 records the concrete public type name and Sifr ownership boundary.

In `structured_runtime_work_model.md`, line 151, replace:
> a Sifr-owned `CancelScope` or cancellation handle may be exposed if the language model needs it.

With:
> a Sifr-owned `CancelScope` or cancellation handle is a settled stable API; M0 records its concrete public type name and Rust implementation boundary.

---

## Non-Blocking Polish

1. **Unawaited `child` handle in canonical examples** (model doc lines 39–46, substrate doc lines 163–167): The canonical `async with TaskGroup` snippet assigns `child = group.spawn_process(...)` but never awaits or observes `child`, while `users`, `config`, and `index` are all awaited. Since `TaskGroup` auto-supervises scope-bound work, this may be intentional, but readers unfamiliar with the policy will infer the example is incomplete. A brief inline comment or note such as "— supervised by group scope" would eliminate the ambiguity without changing any contract.

2. **`spawn_scoped` location ambiguity** (substrate.md M1 surface list, line 417; Resolved Decisions line 848): `spawn_scoped(fn, *, ctx=None)` appears without a class qualifier in the M1 `sifr.task` surface list alongside `TaskGroup[E]`, while canonical examples show `group.spawn(coroutine)` as the group method. Whether `spawn_scoped` is a module-level function that requires a live scope in scope-context or a `TaskGroup` method alias is not stated. M0 is correctly charged with the public API boundary artifact, but adding one orientation sentence ("module-level `sifr.task.spawn_scoped` is distinct from `TaskGroup.spawn`; M0 records the calling convention") would remove M0 ambiguity.

3. **Loser-evidence type unspecified in `race`/`select` container descriptions** (model doc line 155, substrate doc line 850): Both docs say the container carries "loser cancellation evidence" but neither names the type. Given that `CancelOutcome` is defined with minimum states including `Cancelled`, `AlreadyCompleted`, etc., a sentence confirming loser evidence is `list[CancelOutcome]` would connect the cancellation contract to the result container description. M0 is correctly the binding gate for concrete signatures, but the disconnect is avoidable.
