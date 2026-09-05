I've read both documents and the recent contract-fix commit. Here is the implementation-readiness review.

---

# Review: Production Network and HTTP Platform Substrate

**Verdict:** CONDITIONAL PASS — the conceptual model is sound and the recent fixes (single-runtime-worker boundary, owned split halves, `shutdown_write`) close the structural reviewer gaps. Four spot-fixable contract holes remain that would force discovery during M1 implementation. After those edits the phase is ready.

## Blocking issues

### B1. M0 vs M4 body-stream-contract ownership contradiction
- `issues/ad-hoc-production-network-http-platform-substrate.md:577` says **"M4 must define the public body stream contract"** with a full list (chunk type, EOF, trailers, max chunk size, max collected body, collect helper, cancellation, HTTP/2 reset, partial progress).
- `:729` puts the exact same list in M0 scope: **"Define the HTTP body stream contract, including chunks, EOF, trailers, collect limits, cancellation, HTTP/2 reset mapping, and partial-progress evidence."**
- M0 DoD `:756` further requires "HTTP body stream … contracts are checked in with concrete backlog entries."

This is internally inconsistent and will produce thrash. **Recommend:** M0 *decides* the contract (chunk type, EOF, trailers accept/reject, limits, cancellation policy, reset mapping); M4 *implements* it. Reword the M4 paragraph from "define" to "implement the M0 body stream contract."

### B2. `TcpStream.split() -> Result[..., NetError]` failure mode unspecified
`:449` returns a Result without naming what fails. Either:
- `split` is the canonical affine consume — make it infallible: `TcpStream.split() -> (TcpReadHalf, TcpWriteHalf)`; or
- Document the failure variants (already-closed? mid-cancellation? not-connected?) in the M0 ownership contract so M1 doesn't invent them.

Leaving this as an unjustified `Result` is exactly the kind of API drift M0 is supposed to nail down.

### B3. Write-after-`shutdown_write` on a split write half is undefined
The M0 half-close gate at `:478-484` covers repeated `shutdown_write` (idempotent or typed already-shutdown) and cancellation, but does **not** specify what `TcpWriteHalf.write(data)` / `write_all(data)` return after `shutdown_write()` succeeded. Implementations will diverge (panic vs typed `WriteAfterShutdown` vs silent no-op). Add: "after a successful `shutdown_write()`, further `write`/`write_all` calls on that half return a typed `NetError::WriteAfterShutdown` (or equivalent stable variant); no panic and no silent no-op."

### B4. Unsplit `TcpStream.shutdown_write()` handle disposition is ambiguous
`:481` only specifies the split case: "`shutdown_write()` on a split write half propagates EOF behavior to the peer while preserving local read-half ownership." The unsplit `TcpStream.shutdown_write()` (added at `:448`) has no parallel statement. Reading is silently assumed to remain usable, and the affine handle is presumably *not* consumed (otherwise the rest of the read half is unreachable). Make this explicit in M0:
- Unsplit `shutdown_write()` does not consume the `TcpStream`; the read side remains usable until EOF or typed failure.
- Subsequent unsplit `write`/`write_all` returns the same typed error as B3.

---

## Recommended edits (non-blocking but worth doing in the same pass)

1. **Shared error/close evidence across split halves** (`:474`). "Shared connection error/close evidence" is asserted but not specified. State the mechanism — either "the wrapper surfaces peer close/RST as a typed `Shutdown(reason)` on both halves through the underlying socket state" or downgrade the claim. As written it tempts M1 to either copy state across halves or hand-roll a sync channel (the latter would be a local cancellation/diagnostics substitute, banned by `:1053`).

2. **Multi-core serving follow-up creation is in scope but not in M0 DoD.** Scope at `:734` says "create or link the named follow-up issue"; DoD at `:758` only says a "named follow-up owns multi-core serving throughput." Add to DoD: **"M0 has created or linked the multi-core serving follow-up issue with a stable identifier and recorded the identifier in this phase doc."** Otherwise the requirement evaporates on read.

3. **`listen_tcp` lacks `SO_REUSEPORT`.** The v1 public signature (`:445`) only exposes `backlog` and `reuse_addr`. `SO_REUSEPORT` is host-limited substrate (`socket2` row, `:357`). The multi-core serving follow-up will need it — either flag this explicitly in the follow-up scope ("must extend `listen_tcp` or add `listen_tcp_reuse_port`") or expose a host-limited constructor now. Currently silent.

4. **IDNA "approved backend" loophole** (`:680`): "or explicitly recording that the `url` crate's IDNA behavior is the approved text/i18n backend" lets M0 unilaterally bypass the text/i18n M2 Unicode version choice. Require this path to be co-signed by the text/i18n provider phase owner, not just M0 reviewer.

5. **M3 namespace span is implicit.** M3 owns "URL, Header, And Cookie Primitives" — URL types land in `sifr.url`, but Header/Cookie types land in `sifr.http`. The phase doc never says M3 publishes into two namespaces. Add a one-line note so M3 doesn't try to fold headers under `sifr.url`.

6. **"server accept/dispatch harness unless explicitly marked substrate"** (`:573`) is a back door — M0 can promote it from internal to substrate. Either name the promotion gate (Phase 41 reviewer sign-off + No-Toy-Module gate) or remove the escape clause.

7. **`hyper-util/server-graceful` decision-by-discovery.** `:369` and `:390` say it "may be considered only … after proving it composes with provider-owned shutdown." M0 should record the default expectation (Sifr-owned graceful-shutdown loop over the M1/M2 substrate) so M4 doesn't enter graceful-shutdown work without a baseline.

---

## Residual risks (acceptable, but track them)

- **Concurrency/runtime cancellation model is still open in its own M0** (abort-based vs cooperative cancellation token — concurrency phase `:331`). The network phase consumes whichever wins; backpressure/cancellation semantics may need a small downstream amendment. Network phase already marks the surfaces as `blocked-on-concurrency-runtime-m1`/`-m2`, so this is provider-tracked, not network-owned.
- **Phase 41 handoff and HTTP client handoff contracts are one-sided.** M0 writes them with no Phase 41 / future-client-phase review sign-off recorded. M5 has the external-review loop but the contract is baked at M0. Risk of renegotiation later.
- **Phase 41 capability gap is unstated:** with multipart, WebSocket, compression, and HTTP/3 all deferred, Phase 41's "FastAPI-like" claim is narrower than the comparison implies. Worth one explicit line in the Phase 41 handoff section.
- **Metrics schema can slip to deferred** if M5 doesn't approve the schema. `tracing` baseline still ships, so not a hard gap.
- **Single-runtime-worker phrasing depends on the concurrency/runtime provider's `current_thread` topology choice** (verified). If that provider ever flips to `rt-multi-thread`, this phase's serving claim changes meaning. Anchor by reference rather than restating.

---

## Bottom line

Fix B1–B4 (mostly one paragraph each in M0 / the network API shape / the half-close gate). Apply the recommended edits in the same pass. After that the phase is implementation-ready for M0, dependency rings are coherent, no CPython fallback/compat leakage remains, security/resource ownership is concrete, and milestone ordering is sound.
