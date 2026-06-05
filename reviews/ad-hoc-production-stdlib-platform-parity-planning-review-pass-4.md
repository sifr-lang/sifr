Reviewing the stated resolutions against the phase scope:

**contextmanager/asynccontextmanager** — Clean. Unsupported + revisit rule, no partial generator path, class-based APIs stand independently. No leakage between the two tracks.

**Tokio named backing runtime** — Clean. Naming resolves the async executor identity/lifecycle ambiguity.

**M3 HTTPS / M2 AsyncTlsStream dependency** — Clean. Explicit milestone sequencing, no premature stub.

**signal.signal custom handlers** — Clean. Unsupported with no fallback; default-signal behavior remains in scope if implemented separately.

**socketserver ThreadingMixIn/ForkingMixIn** — Clean. These require OS-level fork/thread dispatch semantics that belong outside this phase.

**SSLContext.wrap_socket sync-only** — Clean. Sync constraint is consistent with no async TLS path in this phase.

**multiprocessing.Pool / ProcessPoolExecutor typed IPC gate** — Clean. Sharing the gate avoids duplicated typed-channel infrastructure without coupling semantics.

No gaps are left unresolved or partially resolved. All unsupported items have explicit revisit rules rather than stub implementations, which is consistent with the no-fallback-path requirement.

PASS
