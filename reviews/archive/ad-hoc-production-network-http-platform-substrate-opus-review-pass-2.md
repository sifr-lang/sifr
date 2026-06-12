# Review (Pass 2): Production Network and HTTP Platform Substrate

**Reviewer:** Claude Opus 4.7
**Scope:**
- `issues/ad-hoc-production-network-http-platform-substrate.md`
- `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-opus-review-pass-1.md`

**Verdict:** **PASS** — the phase is implementation-ready for M0.

The four pass-1 blockers (B1-B4) and all seven recommended edits are remediated in the current docs, and no new blocking contradictions, decision-by-discovery holes, or cross-milestone ownership conflicts were found on a second hard read.

---

## Pass-1 blocker verification

### B1. M0 vs M4 body-stream contract ownership — FIXED
- `substrate.md:580` now reads: "M0 must define the public body stream contract. M4 implements that M0 contract and must not redefine it" with the full sub-list (chunk type, EOF, trailers, max chunk/body size, collect helper, cancellation, HTTP/2 reset, partial progress).
- M0 scope `:732` defines the contract; M4 scope `:920` says "Implement body streaming without unbounded buffering according to the M0 body stream contract"; M4 DoD `:958` says "The M0 `sifr.http` substrate type table and body stream contract are implemented…". Consistent.

### B2. `TcpStream.split()` failure shape — FIXED
- API shape `:449` is now `TcpStream.split() -> (TcpReadHalf, TcpWriteHalf)` — infallible.
- Contract `:474`: "`split()` consumes a live `TcpStream` and is infallible; closed or moved streams cannot be split because the affine handle is no longer available." This makes the affine-consume the only legal path; there is no failure mode to invent.

### B3. Write-after-`shutdown_write` — FIXED
- `:486`: "after successful `shutdown_write()`, subsequent `write` or `write_all` on the unsplit stream or split write half returns a stable typed write-after-shutdown error; silent no-op and panic are rejected." Covers both unsplit and split write half.

### B4. Unsplit `TcpStream.shutdown_write()` handle disposition — FIXED
- `:482`: "unsplit `TcpStream.shutdown_write()` does not consume the `TcpStream`; subsequent reads remain usable until EOF or typed failure." Paired with the B3 sentence, the unsplit half-close path is fully defined.

## Recommended-edit verification

| # | Pass-1 recommendation | Current state |
| --- | --- | --- |
| 1 | Specify the mechanism for shared close/error evidence across split halves; do not tempt M1 into a sync channel. | `:474` — "peer close, reset, local close, and shutdown outcomes must surface as typed `NetError`/EOF evidence from the underlying socket rather than through a local channel, cancellation token, or diagnostics substitute." Explicit. |
| 2 | Multi-core serving follow-up creation belongs in M0 DoD with a recorded identifier. | DoD `:762` — "M0 has created or linked the multi-core serving follow-up issue with a stable identifier recorded in this phase doc." Explicit. |
| 3 | `listen_tcp` / `SO_REUSEPORT` must be flagged in the follow-up scope. | Scope `:738` — "Record whether the serving-scale follow-up will extend `listen_tcp` with host-limited `SO_REUSEPORT`, add a separate host-limited listener constructor, or defer `SO_REUSEPORT` from public API entirely." Explicit. |
| 4 | IDNA approved-backend path must be co-signed by the text/i18n owner. | `:683` — "The `url` crate's IDNA behavior may become the approved backend only after explicit text/i18n provider owner sign-off…". Explicit. |
| 5 | State that M3 publishes into two namespaces. | M3 scope `:878` — "Publish M3 primitives into two namespaces: URL and query APIs under `sifr.url`, and header/cookie-header protocol primitives under `sifr.http`." Explicit. |
| 6 | Name the gate for promoting the server accept/dispatch harness from internal-only to substrate. | `:577` — "may be promoted from internal-only to production-substrate only through M0 No-Toy-Module Gate approval and Phase 41 reviewer sign-off." Explicit. |
| 7 | Record the default Sifr-owned graceful-shutdown baseline for `hyper-util/server-graceful`. | `:369` — "default to a Sifr-owned graceful-shutdown loop over provider shutdown primitives and avoid `server-graceful` unless M4 proves it composes with provider-owned shutdown." Echoed in the rejected-features row at `:390`. Explicit. |

## Phase 41 capability-gap risk (pass-1 residual) — also addressed
M5 DoD `:1017` now reads: "Phase 41 handoff documentation states that multipart/form parsing, WebSocket/upgrade products, Content-Encoding compression, and HTTP/3/QUIC are outside this substrate and require separate accepted product/transport phases before Phase 41 may claim those capabilities." Pass-1 risk closed.

---

## Second-pass scan for new blockers

I read the full `substrate.md` (1081 lines) and the execution ledger against the M0/M1/M4 critical path. No new blockers found. Specifically:

- **Cross-namespace ownership.** M0 owns the `sifr.http` substrate type table (`:559`, `:731`); M3 implements the canonical header/cookie primitives (`:878-880`); M4 consumes M3 without defining duplicates (`:919`). Header-validation policy (obs-fold, CL/chunked conflict) is anchored to the security/resource row at `:679` as a pre-M4 requirement. No circular ownership.
- **API surface internal consistency.** `TcpStream`, `TcpReadHalf`, `TcpWriteHalf` (`:443-457`) compose cleanly with the affine-consume `split()`, the unsplit `shutdown_write()` non-consume rule, the typed write-after-shutdown error, and the public Sifr byte-buffer type used uniformly across `read_chunk`, TLS read, and HTTP body chunks (`:489`).
- **Cross-phase blockers.** Text/i18n (M1/M2/M2.5/M3) and concurrency/runtime (M1/M2/M3/M5/M6) dependency matrices are exhaustive and use the shared blocked-on-* states. No surface is left in a hand-wavy state.
- **Ecosystem decisions.** Ring 2-6 crate decisions are pinned; M0 DoD `:758` requires per-family dependency-decision records; conditional crates (`tokio-util`, `hyper-util`, `metrics`) are gated on specific M0/M4/M5 proofs with a recorded default baseline.
- **TLS contract.** `:517-528` covers write/flush/close/`close_notify`/cancellation/partial-progress. mTLS is in M2 scope (`:828`) and has a DoD gate (`:857`). Generated-build requirements and platform-verifier host matrix are M2 DoD items.
- **HTTP/2 conformance.** M0 conformance inventory + M4 DoD `:959` (SETTINGS, RST_STREAM, GOAWAY, HPACK) + direct/transitive `h2` lockfile coherence in M0 (`:370`). Consistent.
- **Milestone ordering.** Dependency graph `:290-298` and the M0-gates-everything rule are intact. M3 → M4 (header/cookie consumption) and M2 → M4 (HTTPS) are explicit.

## Minor observations (non-blocking, do not gate M0)

Three items that M0 will naturally resolve as part of its existing definition gates — not edits to the planning docs:

1. **`close()` disposition on a split half** (`:484`) is described as "according to the final affine-handle rules." That phrase defers the precise semantics (does `TcpReadHalf.close()` affect the still-live `TcpWriteHalf`?) to M0. M0 scope `:726` explicitly owns this under "shared close/error propagation," so it is a known open decision rather than a hidden one — but M1 implementers will look here for guidance. Worth M0 picking the policy on day one.

2. **`Method` and `Status` stability classification** (`:561-562`) is listed as "production-substrate or stable-public utility." M0 picks one. Minor.

3. **`UdpSocket` split / half-close** is not mentioned. UDP is datagram, so split/half-close don't apply semantically — but if M0 accepts UDP, it should explicitly say so to forestall a later "what about UDP split?" question. Trivial.

## Residual cross-phase risks (acknowledged in pass 1; still acceptable)

- Concurrency/runtime cancellation-model choice (abort vs cooperative token) is still owned by that provider phase. Network-side surfaces are correctly tagged `blocked-on-concurrency-runtime-m1`/`-m2`, so amendments propagate cleanly.
- Phase 41 and HTTP-client handoff contracts remain one-sided (no future-phase reviewer signed off). Unavoidable; M5 external-review loop is the mitigation.
- Metrics schema can still slip to deferred if M5 reviewer rejection happens; `tracing` baseline ships regardless.

---

## Bottom line

**PASS.** All four pass-1 blockers and all seven recommended edits are now in the docs with concrete, quotable contract language. No new blocking contradictions surfaced on a second hard read. M0 has a clear, exhaustive set of definition gates; M1 and M4 have unambiguous "implement the M0 contract" mandates; cross-phase blockers and ecosystem dependency records are tight. The phase is implementation-ready.
