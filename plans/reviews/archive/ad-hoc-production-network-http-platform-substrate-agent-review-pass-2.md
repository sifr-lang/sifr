# Review (agent Pass 2): Production Network and HTTP Platform Substrate — TLS Contract Ownership Remediation

**Reviewer:** agent
**Scope:**
- `issues/ad-hoc-production-network-http-platform-substrate.md`
- `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-agent-review-pass-1.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-agent-review-pass-1-2.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-agent-review-pass-2-2.md`
- Uncommitted agent pass-1 remediation diff against `23235bfc4`

**Verdict:** **PASS** — the phase is implementation-ready for M0.

The single pass-1 blocker (B1, TLS contract ownership contradiction) is remediated with exactly the required wording, all four pass-1 recommended edits are folded into the M0 contract gates, and a second hard read of the full phase doc, the remediation diff, and the execution ledger found no new blocking contradiction, ownership thrash, decision-by-discovery hole, or dependency-ring violation.

---

## Pass-1 blocker verification

### B1. TLS contract definition ownership (M0 vs M2, gate boundary) — FIXED

Both required edits are present verbatim:

- `substrate.md:537` now reads: "**The M0 TLS stream contract must define, and M2 must implement**, TLS stream semantics for:" — including the owned full-duplex split bullet (`:544`) that pass 1 flagged as wrongly extending an M2-define list.
- `substrate.md:552` now reads: "**M0 must define** the TLS full-duplex ownership contract **before M2 TLS implementation starts**:" — the hedged "M0/M2" phrasing and the too-late "before HTTPS transport implementation starts" gate are gone.

The ownership chain is now consistent end to end:

- M0 scope `:773` ("Define TLS stream write/flush/shutdown semantics, including `close_notify`, cancellation, and partial-progress evidence") and the new `:774` ("Define TLS full-duplex ownership semantics, including owned split halves, task-boundary sendability, shared TLS/TCP close/error propagation, `close_notify` behavior on split and unsplit streams, and recombine rejection or acceptance").
- M0 DoD `:804` ("Public byte-buffer, DNS, TLS stream, **TLS full-duplex**, `sifr.http` type, HTTP body stream, and URL/IDNA guard contracts are checked in with concrete backlog entries").
- M2 scope `:875-876` ("Add owned TLS full-duplex split into read/write halves", "Add TLS `flush`, `close`, and `close_notify` behavior **according to the M0 TLS stream contract**").
- M2 DoD `:906` (loopback coverage for concurrent read/write, `close_notify`, write-after-close-notify typed errors, partial-progress evidence).

A grep for residual ambiguity (`M2 must define`, `M0/M2 must define`, `before HTTPS transport implementation starts`) returns no hits in the substrate doc. The only remaining "M0/M2" strings are in the execution ledger's review-history narrative and the pre-existing mTLS classification checklist item, both of which describe milestone pairs, not contract-definition ownership. The TLS gate boundary now matches the TCP analog ("before M1 starts", `:471`) structurally: contract before the implementing milestone.

## Pass-1 recommended-edit verification

| # | Pass-1 recommendation | Current state |
| --- | --- | --- |
| 1 | Name the `close_notify()` ↔ TCP FIN disposition as an M0 decision. | `:567` — "M0 must record whether `close_notify()` also performs TCP write-side half-close or only sends and flushes the TLS close alert while leaving TCP write-side closure to `close()`." Explicit, placed in the `close_notify` contract list as recommended. |
| 2 | State the happy-path flush guarantee of successful `close_notify()`. | `:568` — "successful `close_notify()` flushes previously accepted plaintext and the TLS close alert before reporting success, or returns typed partial-progress evidence instead of silently discarding accepted plaintext." Explicit; matches `tokio-rustls` shutdown behavior and the buffering note at `:550`. |
| 3 | Record TLS protocol-version coverage for full-duplex/half-close fixtures. | `:572` — "M2 full-duplex and `close_notify` loopback fixtures must record which TLS protocol versions are covered; TLS 1.2 peers that treat `close_notify` as full close and TLS 1.3 peers that allow post-`close_notify` reads both map to the typed EOF/failure evidence defined by M0." Explicit and binds M2. |
| 4 | Bind the lock-backed/synchronized owned-split expectation into the `tokio-rustls` Ring 4 row. | `:364` — "Owned TLS split halves are API-level independent read/write handles over one synchronized TLS session; implementation uses the accepted Tokio I/O utilities or tokio-rustls facilities rather than bespoke TLS session sharing." Explicit; forecloses the bespoke-session-sharing misreading of `:557` and requires no new crate or feature. |

## Second-pass scan for new issues

The remediation diff touches the `tokio-rustls` binding note, the TLS API shape section, the TLS contract bullets, M0 scope/DoD, M2 scope/DoD, and the execution ledger. Verified clean:

- **No new ownership conflict.** The diff adds "define" language only to M0 and "implement" language only to M2. The body-stream (M0→M4), TCP full-duplex/half-close (M0→M1), byte-buffer (M0, before M1), and DNS (M0) contract chains are untouched and remain single-owner.
- **TLS API shape consistency.** The accepted TLS stream API list (`:522-535`) is internally consistent with the contract bullets: `split()` is infallible and consuming (`:556`), every listed half operation (`read_chunk`, `write`, `write_all`, `flush`, `close_notify`, `close`) has a corresponding semantic owner in the M0 contract, and the public-shape bullet "owned TLS full-duplex split halves for custom bidirectional protocols" (`:516`) matches M2 scope `:875`.
- **TCP/TLS mirroring preserved.** The TLS contract still mirrors the accepted TCP contract bullet-for-bullet (affine unsplit handle, infallible consuming split, owned affine halves, evidence from underlying TLS/TCP state, compiler-gated sendability, borrowed views rejected, recombine rejected for v1), with `close_notify()` correctly substituted for TCP `shutdown_write()` and write-after-close-notify (`:570`) paralleling write-after-shutdown (`:488`).
- **Dependency rings unchanged.** No new crate, no new feature flag; the Ring 4 note points implementation at already-accepted Tokio `io-util` / `tokio-rustls` facilities. Locked versions and rejected rows are untouched.
- **Ledger consistency.** The pass-1 review and remediations are recorded (`execution.md:118-121`) with matching checklist entries (`:171-172`); milestone checklist and implementation-PR sections correctly remain pending/draft; the pass-1 review artifact is retained.
- **Serving-scale, concurrency-provider consumption, Phase 41/client handoff, security/resource rows.** Untouched by the diff; re-checked against the pass-1 verified-clean list with no regression.

## Non-blocking observations (do not gate M0)

1. **List punctuation nit.** In the `close_notify` contract list, the cancellation bullet (`:571`) ends with a period while the list continues at `:572`; sibling bullets end with semicolons. Cosmetic only.
2. **Post-`close_notify` `flush` with nothing pending.** `:570` types the error for operations "that would write application data"; combined with `:568`, a post-`close_notify` `flush` can never have pending application data, so by this wording it succeeds vacuously. That reading is deterministic and matches `tokio-rustls`, but M0 may want one line pinning idempotent-flush-after-close-notify explicitly when it writes the contract.
3. **M2 DoD does not echo the TLS-version recording requirement.** The requirement is normative and binds M2 at `:572`, so no gap exists; an echo in the M2 DoD list (`:906`) would make it harder to miss during M2 closure. Optional.

## Residual risks (carried; unchanged from pass 1 / agent pass 2)

- `TlsReadHalf.close()` / `TcpReadHalf.close()` disposition while the sibling half is live remains "according to the final affine-handle rules" — a known M0 day-one decision.
- Phase 41 and HTTP-client handoff contracts remain one-sided until the M5 external-review loop.
- Metrics schema can still slip to deferred at M5; `tracing` baseline ships regardless.
- `aws-lc-rs` build-tooling burden across supported hosts remains an M2 evidence item.

---

## Bottom line

**PASS.** The pass-1 blocker is fixed with the exact required wording, the four recommended M0-gate clarifications are all in the contract text, and no new blocking issue was introduced by the remediation. M0 now owns every pre-implementation contract (byte buffer, DNS, TCP full-duplex/half-close, TLS stream, TLS full-duplex, `sifr.http` types, body stream, URL/IDNA guard) with the implementing milestones unambiguously downstream. The phase is implementation-ready for M0.
