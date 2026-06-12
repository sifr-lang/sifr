---

## Verdict: FAIL

Six blocking decision gaps remain. Most are cross-milestone sequencing contradictions rather than missing definitions; M0 inventory work cannot resolve them because they constrain what M1–M5 may validly implement.

---

### Blocking Decision Gaps (Severity Order)

---

**1. `JoinSet` drop behaviour directly contradicts the no-silent-failure guarantee**
Severity: Critical

M1 DoD states "task failures cannot be silently dropped." M3 introduces `JoinSet[T, E]` described as a "dynamically-growable collection" with no defined drop contract. If a `JoinSet` is dropped while tasks are in flight, those tasks either run to completion with no one observing the results (silent drop, violates the guarantee) or they are cancelled with no typed evidence surface (also a violation). The phase never decides: does dropping a non-empty `JoinSet` (a) produce a compile-time diagnostic, (b) propagate a `JoinSetAbandoned` typed error, or (c) require explicit `join_all` or `cancel_all` before drop? Without this, M1's guarantee and M3's API are on a collision course.

**Remediation:** Add a `JoinSet` drop contract to M3 scope: either `JoinSet` must be explicitly drained/cancelled before drop (compile-time diagnostic on drop of non-empty `JoinSet`) or dropped work produces a `JoinSetAbandoned` typed evidence path that the compiler enforces an observer for.

---

**2. Rayon pool architecture conflicts with the `PoolConfig` API that M3 commits to shipping**
Severity: Critical

The resolved decisions commit to `sifr.parallel.PoolConfig { workers: PositiveInt }` as an optional configuration API. Rayon's global default thread pool (`rayon::ThreadPoolBuilder::build_global`) can only be configured once; a second attempt is a `ThreadPoolBuildError`. If M3 exposes `PoolConfig`, it must choose one of three incompatible architectures: (a) configure Rayon's global pool once at startup (single call, panics on subsequent calls — contradicts "no user-triggerable panics"), (b) build a private `rayon::ThreadPool` per `PoolConfig` call (expensive, correct), or (c) a lazy-init global pool that is sealed after first use. The phase commits to the API (`PoolConfig`) but leaves the architecture to M0's dependency decision record, which is too late: the architecture determines whether `PoolConfig` is a program-start hint, a per-scope pool builder, or something else, and that shapes what M0 must record and what M3 may implement.

**Remediation:** Add a concrete architecture decision to "Resolved Decisions": state that `sifr.parallel` uses a private Rayon `ThreadPool` built from `PoolConfig` at program start (or lazily on first use), or that `PoolConfig` is a program-startup-only call with a diagnostic on subsequent use. Name the exact Rayon API (`ThreadPoolBuilder`) M3 must use.

---

**3. M5 `sifr.task.Context` propagation requires M1 spawn-API slots that M1 does not know it must reserve**
Severity: High

M1 finalises `spawn_scoped`, `TaskGroup[T, E]`, and all task-spawn signatures. M5 — four milestones later — adds `sifr.task.Context` with "explicit opt-in propagation across task groups." Explicit propagation means the context must be passed at the spawn call site, not injected post-hoc. If M1's spawn signatures do not include an optional `ctx: Context` parameter (or an equivalent extension point), M5 cannot add explicit propagation without breaking M1's already-merged API.

The phase offers no coordination: it does not say "M1 must reserve a context slot" nor "M5 is allowed to modify M1 APIs." This is a sequencing gap, not an M0 inventory gap.

**Remediation:** Add to M1 scope: "spawn_scoped and TaskGroup signatures must accept an optional `ctx: sifr.task.Context` parameter; M1 may leave the propagation implementation as a no-op stub, but the parameter must be present in the stable API." Alternatively, add to Resolved Decisions a statement that M5 context propagation is permitted to extend M1 APIs with a minor-version additive change, naming the extension mechanism.

---

**4. Post-M0 external review gate has no fallback rule, leaving M1 indefinitely blockable**
Severity: High

M7's external review has an explicit fallback: if review output is unavailable for five working days, the phase owner may proceed by recording the attempted review and a conservative self-review. The post-M0 review that gates M1 has no equivalent fallback. If the designated reviewer (who is not yet named and is first recorded in M0) is unavailable after M0 closes, M1 is indefinitely blocked with no defined escape. The M0 DoD also does not list "reviewer identity recorded" as an explicit blocking gate — it only requires the post-M0 review to carry a `PASS` result.

**Remediation:** Add the same five-working-day fallback rule to the post-M0 review gate, identically to M7's rule. Also add "Reviewer identity is recorded in the execution ledger" as an explicit line in M0's Definition of Done (it is in M0 scope but absent from DoD).

---

**5. `sifr.asyncio` existing veneer disposition is unresolved between three incompatible options**
Severity: Medium

M0 scope says: "Classify existing `sifr.asyncio` compatibility veneer cleanup: keep as `adapter-later` with cleanup work, narrow it, or reject it with diagnostics." The API Tier Decision Index records `sifr.asyncio` as `adapter-later / not-implemented-this-phase`. But "keep with cleanup work," "narrow it," and "reject it with diagnostics" are three different outcomes for the code that already exists in the compiler. M1 adds a parallel production `sifr.task` surface — if `sifr.asyncio` still wraps the same scheduler, does M1 build on top of the veneer or replace what it wraps? If `sifr.asyncio` is narrowed, which APIs survive? If rejected, which milestone adds the diagnostics? The waiver index has no entry for `sifr.asyncio`, meaning the disposition is not recorded as a concrete decision anywhere.

**Remediation:** Collapse the three options to one in Resolved Decisions. Either (a) "existing `sifr.asyncio` veneer code is removed in M1 and replaced by `sifr.task`; bare `asyncio.*` imports receive the namespace-contract diagnostic," or (b) "existing veneer code is kept intact but narrowed to only the non-task symbols; M1 does not touch it." Record the chosen option in the waiver index with a regression fixture assignment.

---

**6. M0 DoD does not require the dependency decision records it gates M3 on**
Severity: Medium

M3 has an explicit entry gate: "Pool-sizing policy for `sifr.parallel` is recorded in the execution ledger before M0 closes; M3 must not start until this entry exists." M0 scope says "Create checked-in dependency decision records for every Rust Ecosystem First crate family." But M0's Definition of Done reads: "Every accepted or rejected Rust ecosystem crate family has a checked-in dependency decision record **before M1 starts**" — not before M0 closes. M3's entry gate references M0 close; M0 DoD references M1 start. These are the same moment in the nominal sequence, but they state different events. If M0 is declared closed without dependency decision records, and M1 starts, M3's gate is technically satisfied (records exist before M1 started) even if the pool-sizing record is missing.

**Remediation:** Change the M0 DoD line to read "before M0 closes" to match the M3 entry gate wording, and add an explicit bullet: "All Rust Ecosystem First dependency decision records are checked in and include accepted crate, feature flags, and the architecture decisions listed in the Rust Ecosystem First table."

---

### Non-Blocking Polish Items

1. **`race` and `select` loser-cancellation evidence type** — listed as M1 scope items but their return-type shape (does `race` return `(T, Vec<Cancelled>)` or plain `T`?) is not stated. The governing principles (no silent failure, typed evidence) are present; M1 can derive the concrete shape from CPython `gather`/`wait` evidence and Sifr typed-error policy. Not blocking.

2. **`cancel_scope` vs `deadline` vs `timeout` semantics** — three names are listed together in M1 scope with no description of how they differ. M1 will need to define this from Trio/Tokio evidence. The phase's governing principle (structured cancellation with typed evidence) is adequate. Not blocking, but worth a single sentence in M1 scope distinguishing the three.

3. **`Channel[T]` relation to `BoundedChannel[T]` and `UnboundedChannel[T]`** — whether `Channel[T]` is a trait, an alias for one variant, or a distinct type is unspecified. M2 can design this coherently within its own milestone scope since no other milestone pre-depends on the distinction. Not blocking.

4. **`TaskGroupError[E]` and `Cancelled` interaction** — when a task group has some failing children (`E`) and some cancelled children, the aggregate type is written `TaskGroupError[E]` but `Cancelled` is a separate error type. Whether `E` must be `E | Cancelled` or whether `TaskGroupError` carries a separate cancellation list is not stated. M1 can decide this; the phase's principle (aggregate evidence for all observed failures) is clear enough to drive the design. Not blocking.

5. **`spawn_scoped` vs standalone `TaskHandle` usage** — both appear in M1 scope but their relationship is not described. Whether `spawn_scoped` returns a `TaskHandle` that must be awaited before scope exit, or whether it registers into the enclosing `TaskGroup`, is left implicit. Not blocking; M1 can derive this from the structured-concurrency model.

6. **`strsignal` host matrix** — the phase says "where host-supported" and correctly gates adoption on M0's signal-to-host matrix. No decision gap; M0 owns the matrix.

---

### Why FAIL and Not PASS

Gaps 1–3 are direct contradictions or sequencing incompatibilities between milestones that M0 inventory work cannot retroactively fix: `JoinSet` drop conflicts with the no-silent-drop guarantee already embedded in M1 DoD; Rayon pool architecture constrains the `PoolConfig` API shape that is already committed in the Resolved Decisions table; and context propagation will break M1's finalised spawn signatures unless M1 is told today to reserve extension points. These three require edits to the phase spec and/or Resolved Decisions before M0 work begins. Gaps 4–6 are governance and consistency issues that, if left, will create stuck states or ambiguous milestone exit criteria.
