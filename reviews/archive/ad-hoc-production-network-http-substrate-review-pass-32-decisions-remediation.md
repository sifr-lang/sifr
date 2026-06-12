Reading the phase doc, execution ledger, and pass-31 review simultaneously before forming any judgment.

---

## Review: Pass 32 — Decision Clarity Verification

### All 14 Pass-31 Decisions: Status

| # | Pass-31 Gap | Resolved in patch? | Evidence (line) |
|---|---|---|---|
| 1 | rustls crypto provider | **YES** | "Use rustls's default `aws-lc-rs` provider..." |
| 2 | TLS root store | **YES** | "`rustls-platform-verifier`...`webpki-roots` is not a fallback" |
| 3 | DNS strategy | **YES** | "`tokio::net::lookup_host`...`hickory-resolver` is deferred to an explicit `sifr.net.resolve_*` API" |
| 4 | Stream I/O ownership | **YES** | `read_chunk(max_bytes) -> Result[Option[bytes], NetError]` owned-buffer model, `None`=EOF |
| 5 | `sifr.http` path | **YES** | "`sifr.http.core` is rejected as an extra stable namespace layer"; "Stable HTTP substrate types live under `sifr.http`" |
| 6 | UDP | **YES** | "M1 includes a constrained `UdpSocket` with `bind`, `send_to`, `recv_from`, `connect`, `send`, `recv`, `local_addr`, and `close`" |
| 7 | Tower acceptance | **YES** | "Minimal `tower-service` / `tower::Service` shape is internal-only...no public `tower` or `Layer` types" |
| 8 | OpenTelemetry bridge | **YES** | "OTel exporter/bridge crates are deferred...`tracing` spans/events and `metrics` counters/histograms only" |
| 9 | `x509-parser` | **YES** | "deferred; `x509-parser` only in a future certificate-inspection phase...raw DER fingerprints only" |
| 10 | `socket2` options | **YES** | "`SO_REUSEADDR`, host-limited `SO_REUSEPORT`, `TCP_NODELAY`, `SO_KEEPALIVE`, and `IPV6_V6ONLY`...Other options are not public" |
| 11 | mTLS | **YES** | "M2 includes client certificate authentication configuration and deterministic `rcgen` client/server certificate fixtures" |
| 12 | Multipart/form | **YES** | "Multipart parsing is rejected for this phase...No `multipart` crate is accepted here" |
| 13 | Upgrade hooks | **YES** | "`internal-test` only for transport validation. Public WebSocket, CONNECT tunneling, and upgrade APIs are deferred" |
| 14 | Tokio features | **YES** | "`macros`, `rt-multi-thread`, `sync`, `time`, `net`, and `io-util`. `tokio/full` is rejected" |

All 14 pass-31 decisions are now explicitly resolved. No reversions.

---

### New Contradictions Introduced By The Patch

None found. The eight resolved decisions in the ecosystem table are internally consistent with:
- M1/M2/M3/M4 Rust/runtime stacks
- TLS API shape section
- Sifr-Native Network API shape section
- Resolved Planning Decisions for M0 (item 7)
- Architecture Principles section

The "M0 must expand the Tokio dependency feature plan" instruction (Architecture Principles) is now redundant rather than contradictory — the expansion is pre-decided and M0 validates it.

---

### Remaining Unmade Decisions That Would Cause M0 Implementers To Guess

**Finding 1 — `tower-service` vs `tower` crate identity (concrete guessing point)**

The ecosystem table (Preferred crates column) says:

> `tower-service` / minimal `tower` traits internally

The resolved decision for the same row says:

> Minimal `tower-service` / `tower::Service` shape is internal-only

The `/` notation is genuinely ambiguous. It could mean:
- depend on the `tower-service` crate (a ~100-line crate containing only the `Service` trait), **or**
- depend on the `tower` crate with `default-features = false` plus whatever features are needed

These are different Cargo entries, different transitive-dependency graphs, and different binary-size impacts. An M0 implementer writing the dependency decision record per the DoD requirement ("accepted crate and feature flags") would have to guess which to write. The rest of the document resolved 13 peer decisions at exactly this level of precision; this one should too.

The correct answer given the context is `tower-service` only — `tower::Layer` is explicitly excluded, and `tower-service` provides exactly `tower::Service<Request>` with no additional weight. But the doc should say so.

**Minimal fix:** In the ecosystem table's Preferred crates cell, replace:

```
`tower-service` / minimal `tower` traits internally
```

with:

```
`tower-service`
```

And in the resolved decisions table, replace:

```
Minimal `tower-service` / `tower::Service` shape is internal-only for Phase 41 handoff.
```

with:

```
Use `tower-service` crate only (not the full `tower` crate). The `Service` trait is internal. No `tower::Layer`, `tower::util`, or `tower` features are pulled. Public Sifr APIs hide this behind Sifr request/response types.
```

---

**Finding 2 — M2 definition of done does not gate mTLS**

The M2 scope explicitly says:

> Add client certificate authentication with deterministic fixtures.

The M2 definition of done has five bullets, none of which require mTLS to pass:

> - Local self-signed and CA-backed handshake fixtures are deterministic.
> - HTTPS-ready TLS loopback tests pass.
> - Invalid certificate tests produce typed errors.
> - Safe verification is default.
> - TLS verification failures never panic and never silently downgrade verification.

An M2 PR can pass every DoD bullet without implementing client certificate authentication. The scope says it's in-scope; the DoD does not verify it. This is a direct scope-to-DoD contradiction.

**Minimal fix:** Add one bullet to the M2 definition of done:

```
- Client certificate authentication loopback tests pass with `rcgen`-generated client and server certificate fixtures; mTLS handshake rejection and typed `CertificateError` are covered.
```

---

### Verdict

**FAIL**

Two issues:
1. `tower-service` vs `tower` crate identity — leaves M0's dependency decision record ambiguous at exactly the level of precision the phase requires for all other ecosystem entries.
2. M2 DoD is missing the mTLS gate — scope says add it, DoD does not require it, so a compliant M2 PR could omit client certificate authentication.

Both fixes are one-line edits. No new contradictions were introduced by the patch. All 14 pass-31 decisions are resolved clearly.
