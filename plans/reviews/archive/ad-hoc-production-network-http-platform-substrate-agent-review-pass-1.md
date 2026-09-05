# Review (agent Pass 1): Production Network and HTTP Platform Substrate — TLS Full-Duplex Disposition

**Reviewer:** agent
**Scope:**
- `issues/ad-hoc-production-network-http-platform-substrate.md`
- `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-agent-review-pass-1-2.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-agent-review-pass-2-2.md`
- Uncommitted TLS full-duplex disposition diff against `f30e31f9e`

**Verdict:** **CONDITIONAL PASS** — the TLS full-duplex disposition is substantively correct, feasible on the locked `rustls`/`tokio-rustls` stack, and mirrors the accepted TCP contract bullet-for-bullet. One blocking ownership contradiction was introduced in the new text: the TLS contracts are simultaneously assigned to M0 (scope + DoD) and to "M2" / "M0/M2 before HTTPS" (TLS section). This is the same contradiction class as agent pass-1 B1 (M0-vs-M4 body-stream ownership), which this phase's own review standard treats as blocking. It is a two-line reword. After that edit the phase is implementation-ready for M0.

---

## Blocking issue

### B1. TLS contract definition ownership is contradictory (M0 vs M2, and the gate boundary is too late)

Three statements disagree about who defines the TLS stream / TLS full-duplex contracts and when:

- `substrate.md:537` — "**M2 must define** TLS stream semantics for: write, write_all, flush, close, TLS `close_notify`, owned full-duplex split into `TlsReadHalf` and `TlsWriteHalf`, cancellation …" (the new disposition *added* the split bullet to this M2-define list).
- `substrate.md:552` — "**M0/M2 must define** the TLS full-duplex ownership contract **before HTTPS transport implementation starts**" (i.e., as late as pre-M4).
- Versus: M0 scope `:770-771` ("Define TLS stream write/flush/shutdown semantics…", "Define TLS full-duplex ownership semantics…"), M0 DoD `:801` ("Public byte-buffer, DNS, **TLS stream, TLS full-duplex**, `sifr.http` type, HTTP body stream, and URL/IDNA guard contracts are checked in"), and M2 scope `:873` ("Add TLS flush, close, and `close_notify` behavior **according to the M0 TLS stream contract**").

Two concrete problems:

1. **Ownership thrash.** An implementer reading the TLS section can legitimately defer contract definition into M2 implementation; one reading the M0 DoD cannot. This is exactly the structure of pass-1 B1, fixed there by rewording "M4 must define" to "M0 defines, M4 implements." The TLS equivalent was never reworded, and the new disposition strengthened the wrong reading by extending the M2-define list and adding the hedged "M0/M2" phrasing.
2. **Wrong gate boundary.** M2 itself implements the owned TLS split (`:872`) and `close_notify` behavior (`:873`). A contract gate of "before HTTPS transport implementation starts" (M4) is too late — it permits M2 to define-and-implement simultaneously, which is decision-by-discovery inside M2. The TCP analog is correctly gated "before M1 starts" (`:471`).

**Required edits (wording only, no semantic change):**

- `:537`: "M2 must define TLS stream semantics for:" → "The M0 TLS stream contract must define, and M2 must implement, TLS stream semantics for:".
- `:552`: "M0/M2 must define the TLS full-duplex ownership contract before HTTPS transport implementation starts:" → "M0 must define the TLS full-duplex ownership contract before M2 TLS implementation starts:".

---

## Recommended edits (non-blocking; fold into the M0 contract gates)

1. **`close_notify()` ↔ TCP FIN mapping is undecided.** This is the main `tokio-rustls` feasibility boundary. `tokio-rustls`'s shutdown path (`poll_shutdown`) sends the TLS close alert **and** shuts down the underlying transport (TCP FIN); a TLS-only close alert without FIN requires the wrapper to drive rustls `send_close_notify` + flush directly instead. The M0 gate "shared TLS/TCP close/error propagation, `close_notify` behavior on split and unsplit streams" (`:771`) owns this decision, but the doc never names it. Add one bullet under `:563` recording whether `close_notify()` also performs TCP write-side half-close or leaves the TCP write side open until `close()`.
2. **`close_notify()` vs buffered plaintext.** `:550` notes Tokio Rustls buffering and `:548` covers cancellation partial-progress, but the happy path is unstated: does a successful `close_notify()` imply flush of previously accepted plaintext (the `tokio-rustls` behavior), or is unflushed data a typed error? One line in the M0 contract.
3. **TLS-version coverage for read-after-`close_notify`.** "subsequent reads remain usable until peer `close_notify`, EOF, or typed failure" (`:565`) is the TLS 1.3 half-close model; TLS 1.2 peers commonly treat `close_notify` as full close and the read side EOFs immediately. The wording accommodates both, but M2 loopback fixtures should record which protocol versions the full-duplex/half-close tests pin.
4. **Lock-backed owned split expectation.** `tokio-rustls` does not provide a native owned split; the implementation path is `tokio::io::split` (lock-backed owned halves) over the TLS stream — covered by the already-accepted Tokio `io-util` feature, no new crate. Worth one binding-note line in the `tokio-rustls` Ring 4 row so M2 reads "independent read/write APIs" (`:557`) as API-level independence over one synchronized session, not a mandate for a bespoke session-sharing design (which `:438` would reject).

---

## Areas verified clean

- **TLS/TCP contract mirroring.** The TLS full-duplex contract (`:552-561`) mirrors the accepted TCP contract (`:471-479`) bullet-for-bullet: affine unsplit handle, infallible consuming `split()`, owned affine halves, evidence from underlying TLS/TCP state (no local channel/token/diagnostics substitute), compiler-gated sendability, borrowed views rejected, recombine rejected for v1. Replacing TCP-style `shutdown_write()` with `close_notify()` is explicit (`:563`) and correct for a record protocol; write-after-close-notify typed errors (`:568`) parallel the accepted TCP write-after-shutdown rule (`:488`). No silent no-op or panic path.
- **M2 DoD coverage.** `:903` now requires loopback tests for concurrent read/write over split halves, `close_notify`, write-after-close-notify typed errors, and partial-progress evidence. Matches the contract.
- **TCP split/half-close.** Unchanged since agent pass 2; still internally consistent (`split()` infallible, unsplit `shutdown_write()` non-consuming, typed write-after-shutdown, evidence from socket state).
- **Serving-scale boundary.** Anchored by reference to the provider topology ("current Tokio runtime topology from the concurrency/runtime provider", `:327`) rather than restating `current_thread`; `rt-multi-thread` stays rejected; single-runtime-worker-per-process is consistent across phase placement (`:4`), the boundary section (`:321-333`), M0 DoD (`:803`), M4 DoD (`:1003`), M5 DoD (`:1058`), and resolved decision 8 (`:1121`).
- **Closed concurrency-provider consumption.** The preamble at `:206` resolves the `blocked-on-concurrency-runtime-*` labels as dependency-classification labels against *completed* provider milestones and names the closed semantics consumed (abort-backed task-handle cancellation, compiler-recognized same-task `task.timeout(...)` scope, M2 backpressure, M3 offload evidence, M5 shutdown/diagnostics). No statement elsewhere still treats the provider as open; the ledger (`execution.md:15`) is consistent.
- **M0 vs M1/M2/M4 ownership.** Body stream (M0 defines `:620`, M4 implements `:962`, M4 DoD `:1000`) and TCP (M0 before M1, `:471`, `:481`) remain clean. The only ownership defect is blocking issue B1 above.
- **Dependency rings / no-bespoke policy.** The TLS disposition requires no new crate and no new feature: owned split is implementable with already-accepted Tokio `io-util`; write-after-close-notify detection is thin Sifr wrapper state (allowed by `:431-436`). Locked versions, rejected rows, and the defer-with-evidence rule are untouched.
- **Phase 41 / HTTP client handoff.** Capability limits (multipart, WebSocket/upgrade, compression, HTTP/3) remain in M5 DoD (`:1059`); protocol-readiness-vs-throughput split intact; client handoff rows unchanged.
- **Ledger.** The TLS disposition is recorded (`execution.md:115-117`) with a matching remediation checklist entry (`:167`); milestone checklist and PR sections correctly remain pending/draft.

## Residual risks (non-blocking; carried or newly noted)

- `TlsReadHalf.close()` / `TcpReadHalf.close()` disposition while the sibling half is live remains "according to the final affine-handle rules" — a known M0 day-one decision (carried from agent pass 2).
- Phase 41 and HTTP-client handoff contracts remain one-sided until the M5 external-review loop (carried).
- Metrics schema can still slip to deferred at M5; `tracing` baseline ships regardless (carried).
- The `blocked-on-concurrency-runtime-*` vocabulary reads oddly against closed provider milestones; the `:206` preamble disambiguates, so cosmetic only.
- `aws-lc-rs` build-tooling burden across supported hosts is still an M2 evidence item, not a pre-proven fact (carried).

---

## Bottom line

**CONDITIONAL PASS.** Fix B1 — two sentences assigning TLS contract definition to M0 and implementation to M2, with the full-duplex gate moved to "before M2 starts" — and the phase is implementation-ready for M0. The TLS disposition itself is sound: feasible on locked `tokio-rustls` within accepted features, faithfully mirrors the TCP ownership model, closes the last undefined duplex surface, and adds the right M2 loopback gates. The four recommended edits are M0-gate clarifications that prevent discovery work in M2, not contract changes.
