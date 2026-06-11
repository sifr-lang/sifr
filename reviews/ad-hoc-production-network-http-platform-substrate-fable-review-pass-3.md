# Review (Fable Pass 3): Production Network and HTTP Platform Substrate — TLS Disposition And Post-Pass-2 Polish

**Reviewer:** Claude Fable 5
**Scope:**
- `issues/ad-hoc-production-network-http-platform-substrate.md`
- `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-1.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-2.md`
- Full uncommitted diff against `23235bfc4` (TLS full-duplex disposition + pass-1 remediation + post-pass-2 polish, layered in one working-tree diff)

**Verdict:** **PASS** — the phase remains implementation-ready for M0.

The post-pass-2 polish (deterministic `flush` after successful `close_notify()` with nothing pending, and the M2 DoD echo of the TLS-version fixture requirement) is exactly what pass-2 observations 2 and 3 asked for, introduces no contradiction with the existing TLS or TCP contracts, and closes the last stated wording gap. The pass-2 punctuation nit was also fixed. No blockers; the recommended edits below are optional record-keeping, not contract changes.

---

## Post-pass-2 polish verification

### P1. Deterministic `flush` after successful `close_notify()` — VERIFIED, NO CONTRADICTION

New bullet at `substrate.md:571`: "`flush` after successful `close_notify()` with no pending application data is deterministic and either succeeds as an idempotent flush or returns the same typed already-closed outcome chosen by M0 for repeated `close_notify()`."

Checked against the adjacent contract bullets:

- **Partition is complete and non-overlapping.** `:570` governs `write`/`write_all`/`flush` "that would write application data"; `:571` governs `flush` "with no pending application data". A post-`close_notify` flush has pending application data or it does not — every case is owned by exactly one bullet. In practice the `:570` flush clause is unreachable (a successful `close_notify()` flushed all accepted plaintext per `:568`, and post-`close_notify` writes error without buffering per `:570`), so `:571` resolves the previously vacuous case pass-2 flagged, rather than contradicting it.
- **Consistent with `:569`.** The flush outcome is tied to the M0 choice for repeated `close_notify()`: if M0 picks idempotent success there, no "typed already-closed outcome" exists and flush succeeds idempotently; if M0 picks the typed outcome, flush may return the same one. Both readings are deterministic, which is the contract's stated bar.
- **Consistent with `tokio-rustls` behavior and the buffering note at `:550`.** No new crate or feature is implied.

### P2. M2 DoD echo of the TLS-version fixture requirement — VERIFIED

`substrate.md:908`: "TLS full-duplex and `close_notify` loopback fixtures record the TLS protocol versions covered and validate the EOF/failure evidence required by the M0 TLS stream contract." This echoes the normative requirement at `:573` without restating different semantics — the DoD line defers to "the M0 TLS stream contract" rather than duplicating the TLS 1.2/1.3 mapping, so the contract has a single owner. The sibling DoD line at `:907` (concurrent read/write, `close_notify`, write-after-close-notify typed errors, partial-progress evidence) is unchanged from what pass 2 verified.

### P3. Punctuation nit — FIXED

The cancellation bullet (`:572`) now ends with a semicolon and the list terminates at `:573` with a period. Cosmetic, but the pass-2 nit is gone.

## Contradiction and gap scan (full doc + ledger)

- **TLS contract ownership chain intact.** A grep for `M2 must define`, `M0/M2 must define`, and `before HTTPS transport implementation starts` returns zero hits. The chain remains: M0 defines (`:537`, `:552`, scope `:774-775`, DoD `:805`) → M2 implements (`:876-877`) → M2 DoD verifies (`:907-908`). The polish added "define" language to neither milestone — `:571` and `:573` are contract content inside the M0-owned list, and `:908` is verification, not definition.
- **TCP/TLS mirroring preserved.** The new flush bullet has no TCP analog and needs none: TCP `shutdown_write()` has no buffered-plaintext flush concern, and TCP `write`-after-shutdown (`:488`) deliberately omits `flush` from its typed-error list because the TCP API shape (`:444-458`) exposes no `flush`. The asymmetry is protocol-correct, not a mirroring defect.
- **API shape ↔ contract coverage.** Every operation in the accepted TLS stream API list (`:522-535`) — `read_chunk`, `write`, `write_all`, `flush`, `close_notify`, `split`, half `close`, stream `close` — has a semantic owner in the M0 contract lists (`:539-548`, `:554-561`, `:565-573`). No orphan operations.
- **Dependency rings untouched by the polish.** The `tokio-rustls` Ring 4 binding note (`:364`) still points at accepted Tokio I/O utilities / tokio-rustls facilities; no new crate, feature, or bespoke-session-sharing reading. Locked versions and rejected rows unchanged.
- **Ledger consistency.** The pass-2 entry and follow-up polish are recorded (`execution.md:122-125`) and match the diff exactly; the remediation checklist carries the four TLS items (`execution.md:175-178`); milestone checklist and implementation-PR sections correctly remain pending/draft; both pass-1 and pass-2 review artifacts are retained as untracked files alongside this one.
- **Out-of-diff surfaces.** Serving-scale boundary, concurrency-provider consumption preamble (`:206`), body-stream ownership (M0→M4), TCP full-duplex/half-close (M0→M1), byte-buffer/DNS/`sifr.http`/URL-IDNA gates, security/resource rows, and Phase 41 / HTTP-client handoff limits were re-checked against the pass-1/pass-2 verified-clean lists; no regression and no edit touched them.

## Recommended edits (non-blocking)

1. **Record the flush/repeated-`close_notify` coupling as one M0 decision entry.** `:571` deliberately binds the post-`close_notify` flush outcome to the M0 choice for repeated `close_notify()` (`:569`) — a mixed choice (repeated `close_notify` succeeds idempotently while empty flush errors) is foreclosed. That coupling is good for API coherence, but M0 should record both outcomes in a single decision-index entry so the constraint is visible when the contract is written, not rediscovered.
2. **Optional M2 DoD echo for the determinism fixtures.** `:907` names concurrent read/write, `close_notify`, write-after-close-notify, and partial-progress coverage; repeated-`close_notify` determinism (`:569`) and empty-flush determinism (`:571`) are bound on M2 only transitively through `:877` ("according to the M0 TLS stream contract"). One added clause in `:907` would make them harder to miss at M2 closure. Same shape as the pass-2 echo recommendation; optional.
3. **Ledger entry for this pass.** When this review is committed, add the pass-3 artifact and result to the execution ledger's review history, matching the pattern of passes 1 and 2.

## Residual risks (carried; unchanged)

- `TlsReadHalf.close()` / `TcpReadHalf.close()` disposition while the sibling half is live remains "according to the final affine-handle rules" — a known M0 day-one decision.
- Phase 41 and HTTP-client handoff contracts remain one-sided until the M5 external-review loop.
- Metrics schema can still slip to deferred at M5; `tracing` baseline ships regardless.
- `aws-lc-rs` build-tooling burden across supported hosts remains an M2 evidence item.

---

## Bottom line

**PASS.** The TLS full-duplex disposition plus the pass-1 remediation plus the post-pass-2 polish form a coherent, single-owner contract: M0 defines every TLS stream/full-duplex semantic — including the close-notify/TCP-FIN disposition, happy-path flush guarantee, repeated-`close_notify` determinism, and now empty-flush determinism — and M2 implements and proves them with TLS-version-recorded loopback fixtures. No contradiction, ownership thrash, or decision-by-discovery hole was introduced, and no important contract gap remains open beyond the carried M0-day-one residuals. The uncommitted changes are ready to commit; the three recommended edits are record-keeping polish, not gates.
