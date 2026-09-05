# Review (agent Pass 5): Production Network and HTTP Platform Substrate — Pass-4 Remediation Verification And Final Readiness Sweep

**Reviewer:** agent
**Scope:**
- `issues/ad-hoc-production-network-http-platform-substrate.md` (full doc)
- `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
- `issues/ad-hoc-production-stdlib-platform-contract.md`
- `internal_docs/phases/41_web_framework_and_platform_expansion.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-agent-review-pass-4.md`
- Full uncommitted working-tree diff against `23235bfc4` (pass-4 blocker fixes B1/B2 plus all seven recommended polish edits)
- Repo verification: `lib/sifr/*` baseline, concurrency provider phase doc/ledger, platform-contract review artifacts

**Verdict:** **PASS** — both pass-4 blockers are fully remediated, all seven recommended polish edits landed without introducing a new contradiction, and a fresh sweep of the whole phase found no remaining implementation-readiness blockers. The phase is implementation-ready for M0.

---

## Blocker verification

### B1. HTTP/2 abuse ownership — FIXED

The Security And Resource Model row (`substrate.md:730`) now reads: "M0 must define SETTINGS limits, max concurrent streams, flow-control window defaults, max frame/body buffering, PING handling, RST_STREAM cancellation mapping, GOAWAY graceful shutdown mapping, and malformed-frame typed errors before M4 starts; M4 implements and validates them with loopback fixtures." This is exactly the required edit and now agrees with all three statements it previously contradicted:

- Section preamble (`:722`): security/resource decisions are M0-recorded implementation inputs, not later discovery tasks.
- M0 scope (`:791`): the network-owned security/resource model M0 defines explicitly includes HTTP/2 abuse.
- M0 DoD (`:810`): security/resource rows are checked in with concrete limits for every network-owned concern.
- M4 DoD (`:1012`): HTTP/2 protocol behaviors come from "the M0 conformance inventory" — the row now matches what M4 already assumed.

A full `must define` sweep of the doc confirms every remaining contract-definition obligation is M0-owned: TCP full-duplex (`:476`), byte buffer (`:496`), TLS stream M0-defines/M2-implements (`:542`), TLS full-duplex (`:557`), `sifr.http` type table (`:603`), body stream M0-defines/M4-implements (`:629`), request smuggling (`:729`), HTTP/2 abuse (`:730`). The only other "M4 must" hit is the M3 ownership rule at `:932` ("M4 must not define duplicate header... representations"), which is a prohibition, not a definition obligation. The decision-by-discovery class of defect is now zero-hit across the doc.

### B2. Terminal-state vocabulary — FIXED

The Evidence Sources allowed-state list (`substrate.md:256-277`) now enumerates `blocked-on-text-i18n-m1`, `blocked-on-text-i18n-m2`, `blocked-on-text-i18n-m2_5`, and `blocked-on-text-i18n-m3` (`:265-268`) alongside the six concurrency labels including the new `blocked-on-concurrency-runtime-m4` (`:269-274`). Cross-checked exhaustively: every `blocked-on-*` label used anywhere in the doc — the text/i18n assignment list (`:191-198`), the concurrency assignment list (`:219-228`), the M0 DoD classification sentences (`:806`, `:807`), and the M3 scope (`:936` keeps Unicode/IDNA `blocked-on-text-i18n-m2`) — now appears in the allowed-state list. The M0 DoD is satisfiable under the strict exhaustive reading; no reliance on the lenient "refinement preamble" reading remains for blocked states. The named-phase deferral labels (`deferred-to-http-client-phase`, `deferred-to-phase-41`, `deferred-to-transport-phase`) remain instantiations of the listed `deferred-to-phase-X` pattern, which the preamble (`:256`) explicitly sanctions as refinements — consistent with how all prior passes read that pattern.

---

## Polish edit verification (all seven landed; no new contradiction)

1. **`blocked-on-concurrency-runtime-m4`** — added to the executor/process matrix row (`:215`), the assignment list (`:223`), the terminal-state list (`:272`), and the M0 DoD (`:807`). Verified against the provider: `milestone_concurrency_runtime_4: Process Runtime` exists in the concurrency phase doc and is checked complete in its ledger, so the label is a coherent refinement of a closed provider milestone. The updated row decision text ("consume the closed provider offload/process/IPC gates or remain deferred. No local worker pool or process supervisor is allowed.") is consistent with the no-substitute rule at `:230` and the Quality Contract at `:1112`.
2. **`UrlError`** — added to the canonical typed-error list (`:685`). Repo-wide grep shows no competing URL error name anywhere; M3 DoD's "Invalid input returns typed errors" (`:956`) and the M0 error-mapping requirement (`:695`) now have a named family to target.
3. **`sifr.asyncio` baseline** — `:283` now states the canonical async surface is `sifr.task` and legacy `sifr.asyncio` veneer surfaces are absent and must remain unsupported diagnostics, not adapter targets. Verified against the repo: `lib/sifr/task.sifr` exists, `lib/sifr/asyncio.sifr` does not. Remaining `asyncio` mentions in the doc are all CPython evidence-source references (`:242-243`, `:673`, `:847-850`, `:893-894`), which are correct usage.
4. **Request-smuggling ownership** — `:729` now reads "M0 must define canonical validation ... before M4 starts; M3/M4 implement and validate the accepted header and HTTP transport behavior." M0-owned, implementers named. One pedantic residual noted below (gate phrase weaker than the M3 implementer implies) — harmless, see non-blocking notes.
5. **Phase 41 backlink** — `41_web_framework_and_platform_expansion.md:11` now lists "Ad Hoc Production Network and HTTP Platform Substrate for `sifr.net`/`sifr.tls`/`sifr.http` protocol substrate, with multi-core serving throughput still owned by the substrate phase's serving-scale follow-up." This matches the substrate's serving-scale boundary (`:336`, `:1069`) exactly — Phase 41 may consume protocol/runtime readiness but not claim multi-core throughput. No contradiction with Phase 41's existing entry/exit criteria.
6. **Platform-contract status** — `ad-hoc-production-stdlib-platform-contract.md:3` now reads "approved shared baseline; provider phases have closed against this contract, and network/HTTP M0 must verify inventories against it." Verified against the repo: the contract's own ordering rule (`:24`, external review PASS before text/i18n M1) is factually satisfied — `reviews/ad-hoc-production-platform-contract-review-pass-3a` through `pass-3d` all record `PASS` (after pass-1/pass-2 FAILs were remediated), and both provider ledgers record closure against the contract. The status is now accurate rather than stale; the M0-verification clause matches the contract's M0 Acceptance section.
7. **`reuse_addr=False`** — `:450` fixed; a doc-wide grep finds no remaining Rust-style lowercase booleans in any signature.

**Ledger coherence:** the execution ledger's pass-4 entry (`execution.md:130-133`) and the two new checklist items (`:188-189`) describe exactly the edits present in the working-tree diff — blockers and polish enumerated accurately, no overclaim. The review-history chain (agent, agent, now 5) remains in order.

## Whole-phase readiness sweep (fresh, not delta-only)

- **No remaining ownership defects.** The `must define` sweep above is the strongest signal: every contract definition is M0-owned with a named implementing milestone. No "M1/M2/M3/M4/M5 must define" survives anywhere.
- **Vocabulary closure.** Every classification label used in any matrix, scope, or DoD appears in the allowed-state list; `open` remains implementation-only and forbidden at phase exit (`:277`). The old `m3\` or \`...m6` row pattern is gone.
- **TLS chain unchanged and intact.** The pass 1-3 TLS work (API shape `:525-540`, M0-defines/M2-implements `:542`, full-duplex contract `:557-566`, `close_notify` semantics `:568-578`, M2 scope `:882-883`, M2 DoD `:913-914`) is byte-identical to the pass-3/pass-4-verified state except for the verified additions; the tokio-rustls binding note (`:369`) requires no new crate or feature — `tokio::io` split utilities are already inside the accepted Tokio `io-util` feature set (`:360`).
- **No stale references.** No document in `issues/` or `internal_docs/` still describes the platform contract as draft; no TODO/TBD/FIXME placeholders in any of the three phase documents.
- **Repo premises re-verified.** Provider phases closed (text/i18n M0-M5, concurrency/runtime M0-M7 checked complete in their ledgers); `lib/sifr` contains the provider modules consumers must call (`task`, `sync`, `runtime`, `process`, `signal`, `parallel`, `ipc`, `encoding`, `unicode`, `io`, `i18n`) and none of the forbidden network-shaped modules (`socket`, `ssl`, `select`, `selectors`, `urllib`, `socketserver`, `asyncio`); `net`/`tls`/`url`/`http` remain absent pending M0.
- **Milestone graph, dependency rings, security table, serving-scale boundary** — unchanged from the pass-4-verified state; spot checks confirm no drift.

## Non-blocking notes (polish only; none gate M0)

1. **Request-smuggling gate phrase.** `:729` says "before M4 starts" while naming M3 as a co-implementer of header validation. Harmless in practice — milestone graph rule 1 (`:299`) forces M0 closure before any implementation milestone, including parallel parser work — but "before M3 starts" would be the tighter phrase if the row is ever touched again.
2. **Phase 41 backlink omits `sifr.url`.** The new Depends-on line names `sifr.net`/`sifr.tls`/`sifr.http` but not `sifr.url`, which Phase 41 Path/Query extractors will also consume. The dependency is on the phase as a whole, so this is cosmetic; the M0 Phase 41 handoff contract will enumerate the full surface anyway.
3. **Platform-contract status could cite its evidence.** The status line asserts approval without naming the review artifacts; appending a pointer to `reviews/ad-hoc-production-platform-contract-review-pass-3a..3d` would make it self-evidencing for future readers.
4. **Historical checklist wording.** `execution.md:163` describes the concurrency dependency matrix as covering "M1...M2...M3...M5...M6" — written before the m4 label existed. It is a historical record of an earlier edit and the m4 addition is separately recorded at `:189`, so no action is required.

## Residual risks (carried; unchanged from pass 4)

- `TlsReadHalf.close()` / `TcpReadHalf.close()` disposition while the sibling half is live remains an M0 day-one decision under "final affine-handle rules".
- Metrics schema can still slip to deferred at M5; `tracing` baseline ships regardless.
- `aws-lc-rs` build-tooling burden across supported hosts remains an M2 evidence item, not a pre-proven fact.

---

## Bottom line

**PASS.** B1 and B2 are fixed exactly as specified, with the fixes verified in full document context rather than diff-only: every contract definition in the phase is now M0-owned with named implementing milestones, and the classification vocabulary is exhaustively closed over every label the phase uses. The seven polish edits are all accurate against the actual repo state (provider milestones, `lib/sifr` contents, platform-contract review artifacts) and introduce no new contradiction. The four notes above are cosmetic. No implementation-readiness blocker remains; M0 may start.
