## Pass 3 Decision-Completeness Review

---

## 1. Verdict: FAIL

One blocking gap remains. Three non-blocking polish items follow.

---

## 2. Blocking Decision Gaps

### B-1 — `JoinItemId` purpose and `join_all()` result ordering are both unresolved (M3 blocker)

**Location:** Phase doc M3 scope (lines 418–419, 421); Resolved Decisions row "JoinSet submission API" (line 714).

**The gap:** Every `JoinSet` submission method returns a `JoinItemId`:
```
JoinSet.spawn_blocking(fn) -> JoinItemId
JoinSet.spawn_cpu(fn)      -> JoinItemId
JoinSet.add(handle)        -> JoinItemId
```
But no `JoinSet` method accepts a `JoinItemId` as input — there is no `JoinSet.cancel(id)`, no `JoinSet.result(id)`, and no `JoinItemId` in the `join_all().await -> list[Result[T, WorkerError[E]]]` return type. The returned value has no documented API role.

Simultaneously, `join_all()` returns a flat `list` without specifying whether the order is submission-order or completion-order. Without that guarantee, a user has no way to correlate results with the work items they submitted.

These two questions are coupled: the answer to result ordering determines whether `JoinItemId` is needed for correlation at all.

An implementer must pick one of these designs but the document provides no basis for the choice:

| Option | `join_all()` return type | `JoinItemId` role |
|---|---|---|
| A | `list[Result[T, WorkerError[E]]]` in **submission order** | Opaque user-side tracking token only; no API use. |
| B | `list[(JoinItemId, Result[T, WorkerError[E]])]` | Correlation key in result; order may be completion-order. |
| C | `list[Result[T, WorkerError[E]]]` in **completion order** | Remove `JoinItemId` from submission returns entirely. |

**Remediation:** Add one row to the Resolved Decisions table:

> **`JoinSet` result ordering and `JoinItemId` role** — `join_all().await` returns results in submission order. `JoinItemId` is an opaque user-side correlation token with no further `JoinSet` API; it is returned so callers can index their own submission records but is not an input to any `JoinSet` method. `cancel_all().await` returns `list[Cancelled]` in submission order.

*(Or commit to option B/C and adjust the return-type signatures accordingly in both the scope and Resolved Decisions.)*

---

## 3. Non-Blocking Polish Items

### P-1 — `race` and `select` Resolved Decisions summaries omit loser cancellation evidence

**Location:** Resolved Decisions table, rows for `race` and `select` (lines 704); M1 scope (lines 329–330, 330–331).

The M1 scope explicitly states that `race` "cancels every still-pending loser and **returns typed cancellation evidence for losers**" and that `select` similarly "cancels every still-pending loser, and **returns typed cancellation evidence for losers**." The Resolved Decisions rows say only "returning index plus typed outcome" / "returning branch tag plus typed outcome" — loser evidence is omitted from both summaries.

The M1 scope is the authoritative spec, so this is not a contradiction between milestones, but an implementer reading only the Resolved Decisions table would miss the loser-evidence return.

**Remediation:** Extend the `race` and `select` Resolved Decisions row bodies to append: "…plus typed cancellation evidence for every loser awaitable."

---

### P-2 — `parallel.map`/`Pool.map` async calling convention never stated

**Location:** M3 scope (lines 422–428); M3 definition of done (lines 453–460).

The docs say to "enforce CPU-heavy diagnostics and explicit offload" and that `parallel.map` / `Pool.map` are CPU-heavy, but never state the pattern for calling them from an async context. The intended pattern — `spawn_cpu(|| parallel.map(items, fn))` — is implied by the effect system but is not written anywhere. The docs also do not state whether these functions block their calling thread (making them sync-only without wrapping) or whether they bridge to Rayon internally.

**Remediation:** Add one sentence to M3 scope: "`parallel.map`, `parallel.try_map`, `Pool.map`, and `Pool.try_map` are synchronous CPU-heavy blocking calls; in an async context they must be wrapped in `spawn_cpu` and the CPU-heavy diagnostic fires if called directly."

---

### P-3 — M4 `shell` effect annotation name is unspecified

**Location:** M4 scope, "explicit `shell` effect classification" (line 480).

The docs require shell invocations to be effect-classified but give no effect annotation name (is it `@shell_exec`? a flag on `@blocking_io`? a distinct effect?). M4 implementers must name and register it without guidance.

**Remediation:** Add a sentence to M4 scope or one row to Resolved Decisions: name the Sifr effect annotation for shell subprocess usage (e.g., `@shell_exec`) and state whether it is a new top-level effect or a sub-classification of `@blocking_io`.

---

## 4. Why a PASS was not issued

Three decisions from Passes 1 and 2 are cleanly resolved in the Resolved Decisions table and no contradiction reappears. The sole new blocker (B-1) is a gap introduced when the `JoinSet` submission API was resolved in Pass 2: the submission shape was fixed, but result ordering and `JoinItemId` semantics — which depend on each other — were not addressed. That gap prevents an implementer from correctly designing either the `JoinItemId` type or the `join_all()` result structure.
