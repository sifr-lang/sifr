## Pass 2 Decision-Completeness Review

### 1. Verdict: **FAIL**

Three blocking decision gaps remain. None are carry-overs from pass 1 (those remediations hold).

---

### 2. Blocking Decision Gaps

#### Gap B1 — `JoinSet.join_all()` awaitable status unresolved (Severity: HIGH)

**Location:** `ad-hoc-production-concurrency-runtime-stdlib-parity.md` → M3 scope

The M3 API listing is internally inconsistent:

```
JoinSet.join_all() -> list[Result[T, WorkerError[E]]]   ← no .await
JoinSet.cancel_all().await -> list[Cancelled]            ← explicit .await
```

`JoinSet` is explicitly used inside async contexts (collecting results from `spawn_blocking`/`spawn_cpu` called from `async def`). A synchronous `join_all()` would block the Tokio runtime thread, violating the Phase 32 constraint ("async tasks must not run blocking work directly"). If `cancel_all()` requires `.await`, `join_all()` almost certainly does too. The current notation leaves this fork open: synchronous-blocking or async-awaitable.

**Remediation:** Add `.await` to `join_all()` if it suspends (most likely), or add a note to M3 scope explicitly documenting it as synchronous and explaining why it does not violate the blocking-in-async diagnostic for this specific case.

---

#### Gap B2 — JoinSet submission API absent (Severity: HIGH)

**Location:** `ad-hoc-production-concurrency-runtime-stdlib-parity.md` → M3 scope; cross-check `Resolved Decisions` table

M3 scope lists `JoinSet[T, E]`, `join_all()`, and `cancel_all()` with no method for putting work into the set. The description says work is "submitted through `sifr.runtime`/`sifr.parallel`" but no submission API appears anywhere in M3 scope, the Resolved Decisions table, or the API Tier Decision Index.

Candidate shapes are: `JoinSet.spawn_blocking(fn)`, `JoinSet.spawn_cpu(fn)`, `spawn_blocking(fn, into=joinset)`, or `joinset.add(handle)`. All four have different ownership implications and different interactions with the compile-time drop diagnostic. The document cannot remain silent on this because the linear-resource drop enforcement (Gap B1's source of truth) depends on knowing when items enter the set.

**Remediation:** Add the submission API shape to M3 scope. At minimum name the method and its return type (or document that `spawn_blocking`/`spawn_cpu` accept an `into: Option[JoinSet[T,E]]` parameter). Record the decision in the Resolved Decisions table.

---

#### Gap B3 — `Pool` instance API surface undefined (Severity: MEDIUM)

**Location:** `ad-hoc-production-concurrency-runtime-stdlib-parity.md` → M3 scope; `Resolved Decisions` table

M3 introduces `Pool(config: PoolConfig)` as a first-class object backed by a private Rayon pool, but no instance methods are specified. The top-level `parallel.map()` and `parallel.try_map()` use the default pool. It is unresolved whether a configured Pool is used via:

- `pool.map(items, fn)` / `pool.try_map(items, fn)` (instance methods mirroring top-level)
- `parallel.map(items, fn, pool=pool)` (keyword argument)
- some other pattern

Because pool shutdown policy is also listed in M3 scope, the instance API must exist and be designed before M3 can be implemented.

**Remediation:** Add `Pool` instance method signatures to M3 scope (at minimum `map` and `try_map` equivalents) and record the decision in the Resolved Decisions table.

---

### 3. Non-Blocking Polish Items

**P1 — "Pending Reviews" section contains completed items.** In the execution ledger, `Pending Reviews` contains the cross-phase pass-21/22/23 reviews and the Rust ecosystem clarifications, all of which carry final `PASS`/`accepted` results. Only the post-M0 external review is genuinely pending. Reorganizing would prevent future readers from treating settled decisions as open.

**P2 — M2, M4, M5, M7 lack formal entry gates.** M1, M3, and M6 each have an explicit `Entry gate:` clause. M2 (depends on M1), M4 (depends on M1+M2+M3), M5 (depends on M4), and M7 (depends on M6) have no parallel statement. The dependency graph is clear enough to derive ordering, but the asymmetry will create ambiguity at implementation time.

**P3 — M3's entry gate does not cite M2 completion.** The M3 entry gate mentions only the pool-sizing ledger entry. The dependency graph paragraph says M3 runs "after sendability/shareability is accepted" (i.e., M2 complete), but this condition does not appear in the formal entry gate. The two places are not contradictory, but an implementer reading only the M3 entry gate would not know M2 is required.

---

### 4. Why PASS was not returned

Both documents are otherwise thorough. The six pass-1 remediations are solid: `JoinSet` drop is now a compile-time diagnostic, Rayon pool architecture is recorded, M5 context slots are reserved in M1, the post-M0 review gate has a five-day fallback and reviewer-identity DoD, `sifr.asyncio` veneer is frozen as `adapter-later`, and M0 dependency records are gated before M0 closes. The failure is on the M3 `JoinSet`/`Pool` API surface: two concrete API operations (`join_all` awaitable status, submission method) and one instance-method shape (`Pool`) are either contradictory or absent at a level of specificity that the document itself establishes as required (M3 scope already specifies return types for other methods).
