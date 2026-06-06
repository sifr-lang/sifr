**FAIL**

The phase doc is structurally sound and has good negative-decision coverage, but there are 14 decisions that remain genuinely open at the phase level. An M0 implementer would have to guess on each of them.

---

## Unresolved Decisions

### 1. rustls crypto provider — missing entirely
The doc names `rustls` and `tokio-rustls` but never mentions the crypto backend. `rustls` 0.23+ requires an explicit choice:
- `ring` (default, battle-tested, no FIPS)
- `aws-lc-rs` (AWS-maintained, FIPS-capable, larger build)

**Required decision:** name the accepted backend, feature flags, and rationale. Binary-size and platform support differ materially.

---

### 2. TLS root store — "and/or" is not a decision
The table says `rustls-platform-verifier` **and/or** `webpki-roots`. These have incompatible semantics for deterministic testing:
- `rustls-platform-verifier` delegates to OS trust store (non-deterministic in CI, correct for production)
- `webpki-roots` bundles Mozilla roots (deterministic, but ignores system CAs)

**Required decision:** specify which is used in production builds, which in test builds, and whether both can coexist in the same binary.

---

### 3. DNS strategy — "and/or" is not a decision
`tokio::net::lookup_host` **and/or** `hickory-resolver`. These have different ownership over DNS configuration, timeout propagation, and SERVFAIL/NXDOMAIN typed error semantics.

**Required decision:** name the strategy (system resolver only, in-process resolver, or layered fallback), explain what `DnsError` variants map from each, and state the deterministic test strategy (e.g. `127.0.0.1`-local mock resolver or `hickory-resolver` with a test config).

---

### 4. Stream I/O ownership model — four options, no preference
The doc lists four mutually exclusive models and says "M0 must choose" with no guidance:
- mutable-borrow buffer reads
- owned-buffer `read(max_bytes) -> Result[Bytes]`
- async iterator / chunk stream
- combined model

**Required decision:** pick one model and state the rationale. This decision controls generated Rust lifetimes for every subsequent TLS and HTTP body streaming implementation.

---

### 5. `sifr.http` public path — explicitly deferred
> "final path may be `sifr.http.core` if review prefers a narrower public boundary"

**Required decision:** choose `sifr.http` or `sifr.http.core` as the stable substrate import path. Every downstream import in Phase 41 and the HTTP client phase depends on this.

---

### 6. UDP — open question with no lean
Open Planning Question #6 is unanswered. UDP affects the M1 scope, loopback test infrastructure, and `UdpSocket` public surface classification. If deferred, it needs a stated deferral criterion (e.g. "deferred until a concrete Phase 41 workload is filed").

**Required decision:** accept or defer UDP; if deferred, record the revisit rule.

---

### 7. `tower` — conditional with no criteria
> "only if M0 accepts it as internal handoff substrate for Phase 41"

No acceptance criteria are given. `tower`'s `Service` trait is the key question for Phase 41's middleware model.

**Required decision:** accept as `internal` or defer. If accepted, state which `tower` features are pulled (`tower`, `tower-service` only, or `tower` full) and whether `tower::Layer` is exposed to Phase 41 or hidden behind a Sifr wrapper.

---

### 8. OpenTelemetry bridge — optional with no crates named
The table says "optional OpenTelemetry bridge crates" but names none. The decision matters for binary size and dependency surface:
- `opentelemetry` + `opentelemetry-otlp` pulls substantial transitive deps
- `tracing-opentelemetry` is a lighter bridge

**Required decision:** accept a specific crate with feature flags, or defer OTel bridge to a later phase and document `tracing`+`metrics` as the only M0–M4 hooks.

---

### 9. `x509-parser` — conditional with no lean
> "only if M0/M2 accepts a production need; must pass malformed-DER, oversized-field, and hostile-chain panic/unsafe audit before merge"

The audit requirement is good, but the acceptance criteria for the "production need" are absent.

**Required decision:** state whether certificate field inspection is in scope for M2 (e.g. for SAN/expiry display in `CertificateError`), or defer entirely to a future phase with an explicit revisit condition.

---

### 10. `socket2` — conditional with no lean
> "low-level options only when Tokio/std do not expose required production behavior"

No list of which options are needed, and no direction on whether to include it by default.

**Required decision:** enumerate the specific socket options (`SO_REUSEPORT`, `TCP_NODELAY`, `SO_KEEPALIVE` timing, `IPV6_V6ONLY`, etc.) that are not available through `tokio::net` and state whether `socket2` is accepted for M1 or deferred pending a concrete gap.

---

### 11. Client certificate authentication (mTLS) — conditional with no lean
> "optional client certificate authentication when M0 confirms deterministic fixtures and backend support"

No guidance on what "confirms" means or what deterministic fixture would satisfy the condition.

**Required decision:** accept client cert auth for M2 with `rcgen`-generated client fixtures, or defer with a concrete revisit criterion (e.g. "accept when Phase 41 requests mTLS for service-to-service auth").

---

### 12. Multipart/form scope — conditional with no criteria
> "Multipart/form parsing is reserved for the future HTTP client/framework phases unless M0 accepts substrate-only parsing"

**Required decision:** reject multipart/form from this phase or accept it with an explicit `multipart` crate dependency record.

---

### 13. HTTP upgrade hooks scope — open
> "upgrade hooks may be reserved but not exposed as a partial public API unless production use is defined"

No production use is defined, and "reserved" is not a valid terminal state.

**Required decision:** classify upgrade hooks as `internal-test` (reserved for WebSocket/HTTP/2-upgrade loopback tests only) or `rejected` with a revisit condition for a WebSocket phase.

---

### 14. Tokio feature flag enumeration — not stated
The doc says "M0 must expand the Tokio dependency feature plan" but never enumerates even a candidate list. The current baseline is `task`/`sync`/`time`. Adding `tokio::net` requires at minimum `net`, likely `io-util`, possibly `rt-multi-thread`.

**Required decision:** enumerate the minimum accepted tokio feature set for M1 (`net`, `io-util`, `rt`, `rt-multi-thread`, `macros`, etc.) and state whether `full` is rejected or allowed for development-only builds.

---

## Summary Table

| # | Area | Gap type |
|---|------|----------|
| 1 | rustls crypto provider | completely absent |
| 2 | TLS root store | "and/or" not resolved |
| 3 | DNS strategy | "and/or" not resolved |
| 4 | Stream I/O ownership model | 4 options, no preference |
| 5 | `sifr.http` vs `sifr.http.core` path | explicitly open |
| 6 | UDP scope | open question, no lean |
| 7 | `tower` acceptance criteria | conditional, no criteria |
| 8 | OpenTelemetry bridge crate(s) | named optional, no crates |
| 9 | `x509-parser` production need | conditional, no criteria |
| 10 | `socket2` gap list | conditional, no gap list |
| 11 | Client cert auth (mTLS) | conditional, no criteria |
| 12 | Multipart/form scope | conditional, no criteria |
| 13 | HTTP upgrade hooks | not a valid terminal state |
| 14 | Tokio feature flag list | M0 task with no starting list |
