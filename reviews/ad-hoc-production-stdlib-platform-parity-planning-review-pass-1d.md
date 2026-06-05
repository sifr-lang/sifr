## Review Findings

### Critical gaps (block implementation)

**1. `socketserver` missing from scope**
M3 targets `HTTPServer`/`BaseHTTPRequestHandler`/`SimpleHTTPRequestHandler`, all of which are built on `socketserver`. CPython evidence confirms `socketserver` exists as a separate module with its own test file. It must be listed as an M3 prerequisite or a separate sub-item, not left as an implicit dependency.

**2. Socket C module reexports unaccounted for**
M1 lists `socket.socket`, TCP/UDP basics, and four name-resolution functions. The CPython evidence lists 15+ additional APIs reexported from the C module (`inet_aton/inet_ntoa/inet_pton/inet_ntop`, `gethostbyname/gethostbyaddr`, `getservbyname/getservbyport`, `getprotobyname`, `socketpair`, `fromfd`, `dup`, `close`, `defaulttimeout`, plus constants). Each needs an adopt/adapt/waive decision before M1 implementation starts. Without this, M1 scope is undefined.

**3. Async SSL design absent**
M2 says "async TLS handshake/read/write" but CPython's non-blocking SSL is driven by `WANT_READ`/`WANT_WRITE` retry loops tied to the event loop. Sifr bars raw event-loop public API and requires real async suspension. There is no design for how `SSLSocket` integrates with the async runtime — is there a distinct `AsyncSSLStream` type? Is `wrap_socket` sync-only? Without this, M2 cannot be implemented.

**4. M3 "async HTTP APIs" is a placeholder, not a spec**
No deliverable is defined. Does this mean an async version of `HTTPConnection`? An `asyncio`-backed `urlopen`? A thin wrapper over async streams from M1? This needs concrete API names or an explicit waive.

**5. M6 generator dependency is unresolved**
`contextmanager`/`asynccontextmanager` are gated on "if generator gaps solved," but generator support is neither scoped in this plan nor tracked as a prerequisite task. This makes M6 a potential no-op with no fallback path defined. Either add a generator-support prerequisite milestone or explicitly waive generator-based context managers and document the class-based-only alternative (ExitStack/AsyncExitStack remain, decorator forms do not).

**6. M5 multiprocessing "if safe" is not actionable**
"If safe" is undefined. Ownership semantics across process boundaries, fork vs. spawn vs. forkserver start methods, and shared memory safety in Sifr's type system are non-trivial design questions. Pre-flag which start methods are in scope, what "safe" means (no shared mutable references across process boundary?), or waive multiprocessing entirely with rationale. The current wording blocks the implementor.

**7. Signal handler design unspecified**
"Cautious handler registration" is not defined. Signal handlers in CPython are process-global, fire on arbitrary threads, and interact with the event loop. In Sifr's ownership model, a closure capturing state in a signal handler raises lifetime questions. The plan needs either a concrete design (handler signatures, allowed captures, interaction with the async runtime) or a narrowed scope (signal constants + `raise_signal` only, no custom handlers).

**8. M8 has no acceptance criteria**
"Production gate docs/demos/validation" is a milestone with zero defined deliverables. What demos are required? What validation suite must pass? What documentation must exist? Without this, M8 cannot be declared done.

---

### Significant gaps (will cause mid-milestone rework)

**9. No milestone dependency graph**
M3 depends on M1 (sockets) and M2 (TLS). M4 async subprocess depends on M1's async runtime. M5 `ThreadPoolExecutor`/`ProcessPoolExecutor` depends on M4 subprocess. M6 `asynccontextmanager` depends on async from M1. None of these are stated. Teams starting M3 or M5 in parallel with M1 will hit blockers.

**10. No cross-cutting error type mapping decision**
CPython raises `OSError` subclasses across all of M1–M4 (`socket.error`, `ssl.SSLError`, `ssl.CertificateError`, `subprocess.CalledProcessError`, `TimeoutExpired`, etc.). Sifr uses `Result`/`Option` instead of exceptions. How each error type maps to Sifr `Result` variants is a design decision that cuts across every milestone. If left until implementation, it will produce inconsistent APIs. This needs a single design note or a shared error-type spec before M1 begins.

**11. `concurrent.futures` test package complexity understated**
The CPython evidence shows `test_concurrent_futures` is a package (multiple files), indicating broad test coverage. The plan should explicitly state that the test suite will be mirrored for all executor types, not just the happy-path APIs, and that fork/spawn/forkserver variants in `multiprocessing` tests are accounted for.

**12. `warnings` global-state concurrency design missing**
M6 mentions "concurrency-safe state" for `warnings` but gives no mechanism. CPython's `warnings` module uses process-global mutable filter lists and per-module `__warningregistry__` dicts. Sifr's ownership model requires an explicit choice (e.g., `Mutex`-guarded global, thread-local registries). This needs a design note.

**13. `locale` process-global hazard unaddressed**
M7 acknowledges locale is process-global but does not address how Sifr exposes it in a concurrent/async context. At minimum, locale-mutating functions must be flagged as unsafe to call concurrently, or the scope must be narrowed to read-only locale queries.

---

### Minor gaps (can be resolved during implementation)

- `urllib.response` vs. `urllib.request`'s response object are conflated in the target list — clarify which is in scope.
- M0 "adopted/adapted/waived matrix" needs a defined output format and owner before M1 starts, otherwise M1 begins without the inventory.
- `asyncio.Queue` is placed in M5 but depends on the async runtime established in M1; note the dependency explicitly.

---

**FAIL**

The plan cannot be handed to an implementor as written. Seven items are blocking: missing `socketserver` scope, undefined socket C module waive decisions, absent async SSL design, undefined async HTTP deliverables, unresolved generator prerequisite for M6, vague multiprocessing safety criteria, and absent M8 acceptance criteria. These must be resolved before M1 begins.
