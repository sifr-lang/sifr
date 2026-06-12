# Review (Fable Pass 4): Production Network and HTTP Platform Substrate — Final Broad Implementation-Readiness Review

**Reviewer:** Claude Fable 5
**Scope:**
- `issues/ad-hoc-production-network-http-platform-substrate.md` (full doc, all sections — not TLS-only)
- `issues/ad-hoc-production-network-http-platform-substrate-execution.md`
- `reviews/ad-hoc-production-network-http-platform-substrate-fable-review-pass-1.md` / `pass-2.md` / `pass-3.md`
- Full uncommitted working-tree diff against `23235bfc4` (TLS disposition + pass-1 remediation + post-pass-2 and post-pass-3 polish + ledger entries)
- Cross-checked against: provider phase ledgers (text/i18n, concurrency/runtime), `issues/ad-hoc-production-stdlib-platform-contract.md`, `verification/platform/*` artifacts, `lib/sifr/*` baseline, `internal_docs/phases/41_web_framework_and_platform_expansion.md`

**Verdict:** **CONDITIONAL PASS** — the phase is implementation-ready for M0 after two one-line wording fixes. Both blockers are ownership/vocabulary contradictions of exactly the class this review series has consistently treated as blocking (Opus pass-1 B1: body-stream M0-vs-M4; Fable pass-1 B1: TLS M0-vs-M2). Neither is a design gap; both are wording fixes. Everything else — dependency rings, security/resource ownership, concurrency-provider consumption, TLS/TCP contracts, serving-scale boundary, no-legacy policy, milestone graph — checks out, and the cross-phase premises (provider phases closed, platform artifacts present, no forbidden legacy modules in `lib/sifr`) were verified against the repo, not just the doc.

---

## Blocking issues

### B1. HTTP/2 abuse limits: "M4 must define" contradicts M0 ownership of the security/resource model

The Security And Resource Model row at `substrate.md:724` reads: "HTTP/2 abuse | **M4 must define** SETTINGS limits, max concurrent streams, flow-control window defaults, max frame/body buffering, PING handling, RST_STREAM cancellation mapping, GOAWAY graceful shutdown mapping, and malformed-frame typed errors."

This contradicts three other normative statements:

- The section preamble (`:716`): "Network/HTTP **M0 must record** concrete security and resource decisions for the surfaces this phase owns. These decisions are **implementation inputs, not later discovery tasks**."
- M0 scope (`:785`): "Define the network-owned security/resource model for … **HTTP/2 abuse**, body/header limits, …"
- M0 DoD (`:804`): "Security/resource rows are checked into the shared platform artifacts **with concrete limits** … **for every network-owned concern**."

An implementer reading `:724` can legitimately defer all HTTP/2 abuse limits into M4 implementation — decision-by-discovery inside the implementing milestone, the exact structure fixed twice before (body stream → M0 defines/M4 implements; TLS contract → M0 defines/M2 implements). Note the sibling row `:723` (request smuggling) at least states a gate boundary ("before M4 starts"); the HTTP/2 row states none.

**Required edit (wording only):** `:724` → "M0 must define SETTINGS limits, max concurrent streams, flow-control window defaults, max frame/body buffering, PING handling, RST_STREAM cancellation mapping, GOAWAY graceful shutdown mapping, and malformed-frame typed errors before M4 starts; M4 implements and validates them with loopback fixtures." (M4 DoD `:1006` already consumes "the M0 conformance inventory", so this aligns the row with what M4 already assumes.)

### B2. Inventory terminal-state list omits text/i18n states the M0 DoD mandates

The Evidence Sources allowed-state list (`substrate.md:255-271`) enumerates `blocked-on-text-i18n-m1` plus five `blocked-on-concurrency-runtime-*` labels — but omits `blocked-on-text-i18n-m2`, `blocked-on-text-i18n-m2_5`, and `blocked-on-text-i18n-m3`. Those three states are mandated elsewhere:

- Text/I18n Dependency Decisions assignment list (`:193-195`).
- M0 DoD (`:800`): "Every text-dependent surface is classified as … `blocked-on-text-i18n-m2`, `blocked-on-text-i18n-m2_5`, `blocked-on-text-i18n-m3` …"
- M3 scope (`:930`): "Keep Unicode/IDNA host canonicalization `blocked-on-text-i18n-m2`."

Under the natural exhaustive reading of `:255` ("Network-specific provider states are **allowed only** as refinements … :" followed by the list), the M0 DoD is unsatisfiable — the inventory schema cannot legally contain the states M0 must assign. A lenient reading (the preamble permits any refinement of shared `blocked-on-phase-X`) exists, but a final-readiness doc must not depend on the lenient reading; the concurrency labels are enumerated exhaustively, implying the text labels are too.

**Required edit (one line):** add `blocked-on-text-i18n-m2`, `blocked-on-text-i18n-m2_5`, and `blocked-on-text-i18n-m3` to the list at `:264`.

---

## Recommended edits (non-blocking)

1. **No label for provider process-runtime (m4) consumption.** The phase consumes "subprocess-backed demos" (`:41`) and the ledger must record dependency states for "subprocess/process-pool-dependent demos" (`execution.md:241`), but the concurrency vocabulary (`:217-227`) jumps m3 → m5 → m6; the provider's `milestone_concurrency_runtime_4` is Process Runtime. Either add `blocked-on-concurrency-runtime-m4` or state explicitly that surfaces consuming the closed provider process runtime classify as `production-substrate`.
2. **Name the URL typed-error family.** The canonical typed-error list (`:674-685`) has no URL error type, yet M3 DoD requires "Invalid input returns typed errors" (`:952`) and the M0 error-mapping doc must map URL error evidence (`:689`). Add `UrlError` (or the chosen name) to the list so M3 doesn't invent one ad hoc.
3. **Stale baseline: `sifr.asyncio` no longer exists.** `:278` says "`sifr.asyncio` is a veneer over the canonical task model…", but the concurrency phase's legacy-surface-removal gate (m0a) removed it — `lib/sifr/asyncio.sifr` is absent and the concurrency ledger records legacy modules verified unreachable with `SIFR-IMPORT-0009` replacement diagnostics. Update the Current Sifr Baseline bullet; under this phase's own no-legacy policy the doc should not describe a removed module as present.
4. **Tighten request-smuggling row ownership.** `:723` ("Header parsing must define canonical validation … before M4 starts") states the right gate but no owner; per `:716` the owner is M0 (with M3 implementing header primitives). Same edit direction as B1, lower severity because the boundary is stated.
5. **Phase 41 backlink missing.** `internal_docs/phases/41_web_framework_and_platform_expansion.md:9-11` lists "Depends on: Phase 40, Phase 32" with no reference to this substrate phase, the serving-scale boundary, or the deferred capabilities (multipart, WebSocket/upgrade, compression, HTTP/3) that M5 DoD (`:1063-1064`) says Phase 41 handoff documentation must state. M0's "Define Phase 41 handoff contract" owns this, but adding the dependency line now prevents Phase 41 planning against an unstated prerequisite. This makes the carried "handoff contracts remain one-sided" risk concrete.
6. **Platform-contract status header.** `ad-hoc-production-stdlib-platform-contract.md:3` still says `Status: draft` although its own ordering rule required an external review `PASS` before text/i18n M1 opened — and both provider phases have since closed against it. Record the review result and refresh the status, or have network M0 note the discrepancy when verifying inventories against the contract.
7. **Cosmetic:** `listen_tcp(address, *, backlog=None, reuse_addr=false)` (`:445`) uses Rust-style `false` in an otherwise Python-syntax signature (`None` elsewhere); Sifr spelling is `False`.

## Areas verified clean

- **Post-pass-3 polish.** M2 DoD (`:907`) now echoes repeated `close_notify` and empty-flush-after-`close_notify` coverage exactly as the pass-3 ledger entry claims; it is verification language, not a second contract definition. The pass-3 ledger entry and checklist items (`execution.md:126-129`, `:179-183`) match the diff.
- **TLS ownership chain intact.** Greps for `M2 must define`, `M0/M2`, and `before HTTPS transport implementation starts` return zero hits; the only remaining "must define" defect anywhere in the doc is B1 (an HTTP/2 row, outside the TLS chain). M0-defines → M2-implements → M2-DoD-verifies holds for TLS stream, TLS full-duplex, and `close_notify` semantics; TCP (M0→M1) and body stream (M0→M4) chains unchanged.
- **Phase-order premise verified against the repo.** Text/i18n milestones 0–5 and concurrency/runtime milestones 0–7 are all checked complete in their ledgers; network genuinely runs third with closed providers, matching the `:206` preamble. The provider modules the doc requires consumers to call all exist (`lib/sifr/`: `task`, `sync`, `runtime`, `process`, `signal`, `parallel`, `ipc`; `encoding`, `unicode`, `io`, `i18n`).
- **No forbidden or premature modules.** `socket`, `ssl`, `select`, `selectors`, `urllib`, `socketserver` are absent from `lib/sifr` as required; `net`, `tls`, `url`, `http` do not exist yet (created/confirmed in M0+), so no namespace collision.
- **Shared platform artifacts exist.** `verification/platform/platform_contract.{md,json}`, `supported_host_matrix.md`, `golden/`, and `scripts/run_platform_golden.sh` are all present; the network doc's required-artifact list is satisfiable. Network's terminal states are a coherent subset/refinement of the platform contract vocabulary, with `compat-adapter` deliberately unused — consistent with the no-legacy policy, not a conflict.
- **Dependency rings.** Locked versions, feature boundaries, rejected rows, conditional rules (`tokio-util`, `hyper-util`, `metrics`), the `h2` direct/transitive lockfile coherence requirement, and the defer-with-evidence/no-bespoke rule are internally consistent; the TLS disposition still requires no new crate or feature. Tokio feature set is identical in Ring 2 and Architecture Principles; `rt-multi-thread` rejection is consistent with the serving-scale boundary at all six anchor points (`:4`, `:321-333`, `:404`, M0 DoD `:807`, M4 DoD `:1008`, M5 DoD `:1063`, decision 8 `:1126`).
- **Security/resource ownership otherwise sound.** TLS defaults/no-silent-downgrade, root strategy (platform verifier + `rcgen`, no `webpki-roots` fallback), body/header limits with `TooLargeError`, redaction-before-observability, loopback-only validation, and cookie-header scope are all M0-owned with implementing milestones downstream; B1 is the single ownership defect in the table.
- **Text/i18n and concurrency matrices.** No local codec/Unicode/locale/fallback-decoder path; no local cancellation/shutdown/offload/diagnostics substitute; URL/IDNA guard with text-provider-owner sign-off; matrices cover Phase 41 and HTTP-client handoff rows. The only defects are the vocabulary omissions (B2, recommended edit 1).
- **No-toy/no-legacy policy.** Gate and Maintenance Burden Test present; partial-public ban; all CPython-shaped surfaces resolve to rejected/unsupported/internal/test-only; the harness promotion path requires gate approval plus Phase 41 reviewer sign-off; `open` state explicitly forbidden at phase exit.
- **Milestone graph and ledger.** M0→M5 ordering with parser-only parallelism; M3 owns canonical URL/header/cookie primitives with M4 as consumer; execution ledger review history, remediation checklist, pending PR sections, and evidence-state vocabulary all consistent with the phase doc and the working-tree diff.

## Residual risks (carried; unchanged)

- `TlsReadHalf.close()` / `TcpReadHalf.close()` disposition while the sibling half is live remains an M0 day-one decision under "final affine-handle rules".
- Pass-3 recommendation to record the flush/repeated-`close_notify` coupling as a single M0 decision-index entry remains process guidance for M0, not doc text.
- Metrics schema can still slip to deferred at M5; `tracing` baseline ships regardless.
- `aws-lc-rs` build-tooling burden across supported hosts remains an M2 evidence item, not a pre-proven fact.

---

## Bottom line

**CONDITIONAL PASS.** Fix B1 (one table-cell reword giving M0 ownership of HTTP/2 abuse limits with M4 implementing) and B2 (add the three missing `blocked-on-text-i18n-m{2,2_5,3}` states to the allowed-state list), and the phase is implementation-ready for M0. The TLS work that occupied passes 1–3 is fully coherent in the current tree, the cross-phase premises hold against the actual repo state (providers closed, platform artifacts present, legacy modules gone), and the seven recommended edits are accuracy/coordination polish — worth folding in with the blocker fixes, but not gates.
