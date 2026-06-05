## Remaining Blockers

**1. Async runtime unnamed (M2, M3, any async module)**
Neither M2 nor M3 names the backing async runtime or Rust crate (tokio, async-std, custom). Sifr requires "real async suspension" and the codegen/driver crates need a concrete decision before any async module can be implemented. This is architecturally prior to both milestones.

**2. `contextmanager` class-based fallback conflicts with Sifr's no-fallback rule**
AGENTS.md states "Do NOT create fallback paths or solutions unless explicitly requested." The remediation proposes a class-based fallback if generators aren't ready. Either the fallback is removed (contextmanager is simply unsupported until generators ship) or an explicit owner exception must appear in the plan. As written it violates a hard constraint.

**3. `signal.signal` decision deferred, not made**
"Defaults to intentional-diff unless a design note proves..." is not a decision — it is a conditional that leaves the milestone scope open at implementation start. The plan must state: supported with these semantics, or unsupported. Deferring to a future design note is not implementation-ready.

**4. M3 HTTP ↔ M2 TLS dependency not declared**
HTTPS in M3 depends on `AsyncTlsStream` from M2. The milestone dependency graph should make this explicit; without it, M3 cannot be scheduled and the graph is incomplete.

**5. `socketserver` mixin safety unaddressed**
`ThreadingMixIn` and `ForkingMixIn` are the primary composition points for socketserver. `ForkingMixIn` intersects M5's fork safety rules directly. Neither mixin's status (supported/unsupported/scoped) is named, leaving a surface area gap in the module scope.

**6. `wrap_socket` async-overload approval path is undefined**
"Sync-only unless typed async overload is approved" — approved by whom, in what process, before or during M2? For implementation-ready, this must be a committed decision, not a gate to an undefined process.

**7. `multiprocessing.Pool` absent from typed-IPC scope**
`Pool` is the primary high-level multiprocessing API. Its work-item and result channels need typed IPC, but M5 only addresses `Value`/`Array`/`shared_memory`. Pool's scope (in or out, and under what constraints) must be stated.

---

FAIL
