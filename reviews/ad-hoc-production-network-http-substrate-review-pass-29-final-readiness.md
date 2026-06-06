I now have the complete picture. Let me do the comprehensive final-readiness review.

---

## Pass-29 Final Readiness Review

**Files reviewed:** `issues/ad-hoc-production-network-http-platform-substrate.md` · `issues/ad-hoc-production-network-http-platform-substrate-execution.md`

**Prior review chain:** Pass 23 PASS (ecosystem) → Pass 24 FAIL (M0 DoD gaps, x509-parser bypass) → Pass 25 PASS (remediations confirmed) → Pass 26 PASS (text/i18n matrix) → Pass 27 PASS (cross-phase final) → Pass 28 PASS (Phase 41 + HTTP client handoff rows)

---

### 1. Phase Ordering and Cross-Phase Dependency Consistency

Phase doc header, recommended-order section, and execution ledger all agree: text/i18n → concurrency/runtime → this phase, third. The M0-early-prototype carve-out ("M0 design, dependency selection, and binary/protocol-only prototypes may happen earlier") is worded identically in both documents. **Clean.**

### 2. Milestone Dependency Graph and M0/M1/M2/M3/M4/M5 Handoffs

M0 → M1 (async streams) → M2 (TLS, gated on M1) → M3 (URL/header/cookie, may parallel with pure-parser work) → M4 (HTTP transport, gated on M1 + M2 + M3) → M5 (handoff). All milestone slots in the execution ledger are `pending`, consistent with no implementation having started. Parallel-only-for-pure-parsers carve-out is explicitly stated. **Clean.**

### 3. Rust Crate Decision and Audit Requirements

All crate families are in the table; `x509-parser` has its own conditional row with explicit audit language; Quality Contract backstop covers future unlisted crates. M0 DoD fifth item requires all eight audit components for every table entry. **Clean.**

### 4. HTTP/2 Requirements and Conformance Expectations — GAP FOUND

**Scope is correct:** M4 scope enumerates HPACK, stream state machine, SETTINGS negotiation, flow control, PING, RST_STREAM, GOAWAY, multiplexing with backpressure, and ALPN-driven protocol selection.

**DoD is incomplete:** M4's Definition of Done says only:

> - HTTP/1.1 and HTTP/2 loopback client/server transport tests pass without external network.
> - HTTPS transport works through M2 TLS, including ALPN selection for HTTP/2.
> - Body streaming and HTTP/2 multiplexing work without unbounded buffering.
> - Malformed HTTP tests produce typed protocol errors.

SETTINGS negotiation round-trip, RST_STREAM stream cancellation, GOAWAY graceful shutdown, HPACK correctness edge cases, and connection preface validation are all listed in scope but are not named in the DoD. An implementer using `h2`/`hyper` will get correct protocol behavior from the library, but there is no DoD gate requiring test coverage for these behaviors. A team can satisfy M4's DoD with happy-path HTTP/2 tests only and close the milestone.

The M4 scope section does say "HTTP/2 and HPACK protocol conformance cases selected during M0" — but that phrase appears under the CPython evidence-mining list, not under the DoD. M0's conformance-evidence requirement for the `hyper`/`h2` dependency record partially covers this, but that record only proves that the crate itself is conformant, not that the Sifr wrapping exercises the conformance scenarios.

**Severity:** Non-blocking (h2/hyper handle the state machine; wrong protocol behavior is unlikely from a library wrapper), but materially weaker than the rest of the DoD given that HTTP/2 conformance was explicitly brought into scope.

**Surgical fix:** Add one bullet to `milestone_network_http_4` Definition of Done:

> - HTTP/2 protocol-level behaviors in the M0 conformance inventory — SETTINGS negotiation, RST_STREAM stream cancellation, GOAWAY graceful shutdown, and HPACK correctness edge cases — have loopback test coverage.

### 5. Text/i18n Blocked States

All 15 matrix rows are present (TCP/UDP/DNS, HTTP bodies, headers, Content-Type/charset, URL, percent encoding, query/form, cookies, TLS cert verification, TLS cert display, diagnostics, observability, demos, Phase 41 handoff, HTTP client handoff). All eight valid states are defined. M1/M2/M2.5/M3 assignments are consistent across the matrix, the cross-phase contract prose, and all prior review confirmations. **Clean.**

### 6. Concurrency/Runtime Dependency States

The cross-phase contract names `blocked-on-concurrency-runtime` as a valid state and says "executor-backed serving APIs" require the concurrency/runtime phase. Unlike the text/i18n matrix (which pre-classifies every surface row-by-row), there is no equivalent enumeration of which specific surfaces are expected to be concurrency-blocked.

This is intentionally asymmetric: the text/i18n phase may still be in progress when M0 starts (the early-prototype carve-out permits this), so a pre-classification matrix is load-bearing. The concurrency/runtime phase is sequenced fully before this phase, so at M0 time all concurrency surfaces should be available and the implementer can inspect the delivered substrate directly. The surface state `blocked-on-concurrency-runtime` is a valid escape hatch if anything is unexpectedly missing.

M4's "async server accept/dispatch/shutdown substrate" implicitly uses `tokio::spawn` (already in the existing task model) rather than executor-backed offload. That boundary is not stated, but it is inferrable from M4 being substrate rather than framework and from Phase 41 owning lifecycle. **Non-blocking residual risk.**

### 7. Public/Internal/Deferred/Rejected Surface Closure States

All valid exit states defined. `open` forbidden at phase exit. Product Boundary, Public Surfaces, and Deferred/Rejected tables cover the space without overlap or omission. Non-Goals list is explicit. CPython-shaped surfaces (`sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib.*`, `sifr.http.client`, `sifr.http.server`, `sifr.socketserver`) are enumerated and rejected or deferred. **Clean.**

### 8. Tracking Artifact Schema and Readiness

Phase doc lists four required artifacts; execution ledger has unchecked boxes for all four; M0 PR is gated on artifact creation; decision-index schema is defined (surface, terminal state, rationale, revisit rule, CPython evidence, regression fixture). **Clean.**

### 9. Validation Commands and Implementation PR Readiness

Execution ledger provides baseline commands (fmt, clippy, HIR guardrails, `--profile create-pr`) and merge-gate command for all milestones. M0 and M5 have explicit `Validation:` sections in the phase doc; M1-M4 rely on the execution ledger baseline, consistent with other phase docs. File-size guardrail is in M5 DoD and Quality Contract. **Clean.**

### 10. No Backward Compatibility / No Fallback / No Toy Module Governance

Compatibility Policy (5 conditions), Quality Contract (explicit prohibitions on shims/aliases/bridges/fallbacks), No-Toy-Module Gate (5 acceptance criteria, 5 insufficient criteria), and Maintenance Burden Test (8 questions) are all present and concrete. **Clean.**

### 11. M3/M4 Header Type Ownership Boundary (minor ambiguity)

M3 scope: "Implement HTTP header representation and validation primitives." M4 scope: "Implement method, status, version, headers, and body types." Both milestones reference the `http` crate. The word "consumed" in the milestone graph ("M3 before M4 HTTP integration where URL/header/cookie parsing is consumed") implies M3 owns the types and M4 uses them, but neither milestone's scope text states this explicitly. An implementer could define Sifr header types in both milestones. M0 open planning question #1 and the "Define HTTP client/server substrate boundaries" M0 scope item are the correct gate for this. **Non-blocking: M0 resolves it.**

### 12. M5 External Review — No Reviewer Assignment in Execution Ledger

M5 references "the designated compiler/runtime reviewer recorded in the execution ledger" for the five-day fallback rule. The execution ledger has no reviewer assignment section. An implementer reaching M5 and needing the fallback must infer the reviewer from context. **Non-blocking residual risk.**

---

## Verdict: **PASS**

No finding rises to the level of a hard blocker. No path exists for an implementation team to accidentally implement from scratch, expose a toy compatibility module, duplicate text/concurrency work, leak crate types, or add fallback behavior — all those paths are closed by explicit policy with concrete enforcement gates.

### Required Edit (surgical, non-blocking elevation)

To remove the HTTP/2 DoD ambiguity before M4 starts, add one bullet to the `milestone_network_http_4` Definition of Done in `issues/ad-hoc-production-network-http-platform-substrate.md`:

**Location:** after "Body streaming and HTTP/2 multiplexing work without unbounded buffering."

```
- HTTP/2 protocol-level behaviors selected in the M0 conformance inventory —
  SETTINGS negotiation, RST_STREAM stream cancellation, GOAWAY graceful shutdown,
  and HPACK correctness edge cases — have loopback test coverage.
```

This closes the gap between scope and DoD for HTTP/2 without changing any policy, adding new scope, or requiring any other document change.

### Non-Blocking Residual Risks (no edits required)

1. **M4 HTTP/2 conformance DoD gap** — partially mitigated by M0 crate conformance-evidence requirement; fully closed by the surgical edit above.
2. **No concurrency-blocked surfaces pre-classification table** — acceptable because the concurrency/runtime phase precedes this one; M0 can classify at design time.
3. **M3/M4 header type ownership boundary** — M0 open planning question #1 is the correct resolution gate.
4. **M5 reviewer assignment** — execution ledger should add an explicit reviewer field before M5 starts; not needed before M0.
