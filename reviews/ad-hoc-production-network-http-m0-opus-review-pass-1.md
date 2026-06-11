I have enough to deliver the verdict. The M0 artifacts have several explicit contract violations where decisions the phase doc demands of M0 are deferred to M1/M2/M4 by name.

# Verdict: **FAIL**

The M0 PR is not safe to merge as-is and M1 cannot safely start. The artifact stack is substantial and the surface-classification / CPython-evidence / dependency-snapshot / unsupported-import-diagnostic / serving-scale-follow-up tracks are in good shape, but several phase-contract DoD items that the contract explicitly says "M0 must define" are recorded in M0 artifacts as deferred to a later milestone. That directly contradicts the M0 closure gate. Details below.

---

## Blocking findings

### B1. `ByteBuffer` final name/namespace/import path is deferred to M1
- **Contract**: `issues/ad-hoc-production-network-http-platform-substrate.md:496` — *"M0 must define the public Sifr byte-buffer type ... `ByteBuffer` in this phase doc is only a placeholder for the public type name until M0 records the final name, namespace, and import path."* Echoed in DoD `:815`.
- **Current state**: `verification/stdlib/network_http_substrate_inventory.md:43` and `.json` decision `byte_buffer:108` — *"ByteBuffer remains placeholder pending final public name/path before M1 implementation"* and *"M1 must select the final public import path before implementing TCP reads."*
- **Why blocking**: TCP read API, TLS read API, HTTP body chunks, and header/body diagnostics all depend on this type name. M1 cannot implement `read_chunk` without it. The phase doc says M0 picks the name.
- **Remediation**: Record concrete name + namespace + import path in `network_http_substrate_inventory.md`/`.json` and in `network_http_m1_async_network_traceability.md`. Update the phase doc placeholder line.

### B2. `TlsStream.close()` / `TlsWriteHalf.close()` disposition is deferred to M2
- **Contract**: `issues/...substrate.md:557` — *"M0 must define `TlsStream.close()` and `TlsWriteHalf.close()` disposition before M2 starts: whether close consumes the stream or write half and closes the underlying TCP stream directly, whether it first attempts `close_notify()`, how cancellation during close is reported, and how failure during close preserves typed `TlsError`/nested `NetError` evidence."*
- **Current state**: `verification/stdlib/network_http_substrate_inventory.md:48` — *"M0 leaves the exact `close()` consumption/TCP disposition as an M2 contract item before code starts."*
- **Why blocking**: Direct, self-admitted contradiction of M0 DoD. M2 cannot start with concrete backlog entries without these decisions.
- **Remediation**: Decide and record disposition rows for `TlsStream.close()` and `TlsWriteHalf.close()` (consume vs. preserve handle, close_notify-first vs. direct TCP close, cancellation reporting, failure evidence shape).

### B3. HTTP/2 abuse limits not defined; only the priority/extensions decision exists
- **Contract**: `issues/...substrate.md:732` — *"M0 must define SETTINGS limits, max concurrent streams, flow-control window defaults, max frame/body buffering, PING handling, RST_STREAM cancellation mapping, GOAWAY graceful shutdown mapping, and malformed-frame typed errors before M4 starts; M4 implements and validates them with loopback fixtures."*
- **Current state**: Only the priority/extensions decision is recorded (`inventory.md:51`). `m4_http_transport_traceability.md:8` lists *"SETTINGS, flow control, PING, RST_STREAM, GOAWAY, HPACK, malformed frame, priority/extension decision fixtures"* — prose-only, no concrete numeric/policy values. `grep` confirms no concrete `max_concurrent`, window defaults, or PING/RST/GOAWAY mapping decisions anywhere under `verification/`.
- **Why blocking**: This is exactly the "prose-only scope" forbidden by M0 DoD line `:819`. M4 has no concrete backlog.
- **Remediation**: Add an M0 decision row per item (e.g., max concurrent streams: N; initial window: N; max frame size: N; PING flood policy; RST_STREAM→`HttpError::StreamReset(...)`; GOAWAY→graceful drain semantics; malformed-frame→typed `ProtocolError` variant).

### B4. Header normalization / request smuggling rules not defined
- **Contract**: `issues/...substrate.md:731` — *"M0 must define canonical validation for names, obs-fold rejection, duplicate header policy, whitespace normalization, `Content-Length` disagreement handling, and `Content-Length` plus chunked conflict handling before M4 starts; M3/M4 implement and validate the accepted header and HTTP transport behavior."*
- **Current state**: No M0 decision rows. `network_http_m3_url_header_cookie_traceability.md:12` only says *"Header token, obs-fold rejection, duplicate policy, and size-limit fixtures"* — prose only, no rule values. `platform_contract.md:71` just labels the concern as network/HTTP-owned without rules.
- **Why blocking**: M3 owns canonical header primitives (`Inventory.md:18`, `m3_traceability.md:12`) and M4 cannot consume them without concrete rules.
- **Remediation**: Record concrete rules: name token charset, obs-fold = reject, duplicates = list-preserving vs. last-wins per header, whitespace = OWS-trim only, CL≠body-length → typed error, CL+TE:chunked → reject 400.

### B5. `sifr.http` substrate type table is collapsed into one row; Trailers decision is missing
- **Contract**: `issues/...substrate.md:606` — full per-type M0 decision table (`Method`, `Status`, `Version`, `HeaderName`, `HeaderValue`, `HeaderMap`, `RequestHead`, `ResponseHead`, `BodyStream`, `BodyChunk`, `Trailers`, error variants). Specifically: *"`Trailers` | accepted or rejected explicitly before M4"* and DoD `:815`.
- **Current state**: `inventory.md:18` is a single combined row *"`sifr.http` method/status/version/header/head/body/error primitives"* with state `production-substrate`. No per-type decision. No Trailers decision recorded anywhere under `verification/` — `grep` returns only forward-references to M4 fixtures.
- **Why blocking**: M3 owns canonical header types and M4 owns body. They cannot start without per-type M0 contracts, including the explicit Trailers accept/reject decision.
- **Remediation**: Reproduce the phase-doc type table in `network_http_substrate_inventory.md`/`.json` with concrete state per type; make the Trailers decision and record it.

### B6. HTTP body stream contract has no concrete values
- **Contract**: `issues/...substrate.md:631-642` lists nine required fields M0 must record: chunk type, EOF behavior, trailers accept/reject, max chunk size, max collected body size, collect-with-limit helper choice, cancellation while reading/writing, HTTP/2 reset mapping, partial-progress evidence.
- **Current state**: `inventory.md:50` says *"M4 implements the M0 contract: bounded chunks, EOF evidence, explicit trailers accept/reject decision, cancellation and reset mapping, and partial-progress evidence"* — but the *contract* itself is not recorded. Chunk type = ByteBuffer (still placeholder, see B1). No size caps. No collect-with-limit helper accept/reject.
- **Why blocking**: M4 cannot build streaming bodies without the contract; this is a direct DoD violation.
- **Remediation**: Add a body-stream contract section with concrete values for all nine fields.

### B7. Body and header size limits — no concrete caps
- **Contract**: `issues/...substrate.md:734` — *"Every parser/body reader has explicit configured limits. Unbounded buffering is rejected unless an API name explicitly collects and M0 records a size cap and typed `TooLargeError`."*
- **Current state**: No concrete caps recorded anywhere under `verification/`. `TooLargeError` exists in the error taxonomy table (`inventory.md:104`) but no size cap values, defaults, or per-parser limits.
- **Remediation**: Add a size-limits table (header name max, header value max, header total max, body chunk max, body collect max if helper accepted, URL max, query max, etc.).

### B8. SO_REUSEPORT v1 decision is not recorded
- **Contract**: `issues/...substrate.md:793` (M0 Scope) — *"Record whether the serving-scale follow-up will extend `listen_tcp` with host-limited `SO_REUSEPORT`, add a separate host-limited listener constructor, or defer `SO_REUSEPORT` from public API entirely."*
- **Current state**: The follow-up issue mentions the *option* (`ad-hoc-network-http-serving-scale-follow-up.md:15`) but neither it nor the M0 inventory records the v1 decision. `grep` confirms no decision row.
- **Remediation**: Pick one (e.g., "deferred from public API entirely until follow-up closes") and record in the inventory M0 Resolved Decisions + the supported-host matrix.

### B9. Redaction rules are not recorded
- **Contract**: `issues/...substrate.md:739` — *"URLs with credentials, query values classified as sensitive, headers, cookies, bodies, certificate fields, peer addresses where configured, and TLS material must have redaction rules before observability hooks ship."*
- **Current state**: `platform_contract.md:72` only says *"Log redaction for URLs, headers, bodies, cookies, certificates, subprocess commands, env, and catalogs | shared contract with owning phase-specific fields"* — i.e., the *owner* is named but the *rules* are not.
- **Remediation**: Add per-field redaction rules (userinfo redacted, `Authorization`/`Cookie`/`Set-Cookie` redacted, body never logged unless explicit opt-in, certificate display follows text/i18n M2, etc.) into the inventory and/or platform contract.

### B10. URL authority security details missing
- **Contract**: `issues/...substrate.md:736` — *"Userinfo redaction, host/port validation, path normalization semantics, percent-decoding boundaries, and IDNA/Unicode blocking states are recorded before `sifr.url` becomes public."*
- **Current state**: Only the IDNA blocking state and ASCII/punycode-only rule are present. No userinfo redaction rules, no host/port validation rules, no path normalization semantics, no percent-decoding boundary rules recorded.
- **Remediation**: Add concrete rules to `inventory.md` M0 Resolved Decisions and to `network_http_m3_url_header_cookie_traceability.md`.

### B11. Per-crate dependency audit fields are incomplete
- **Contract**: `issues/...substrate.md:813` (M0 DoD) — *"Dependency decision records are present and checked in for every crate family ... covering accepted crate and feature flags, Sifr abstraction that hides the crate from public APIs, panic/unsafe audit for user-controlled data paths, typed error mapping into Sifr variants, license/MSRV/binary-size/platform impact, deterministic local test strategy, conformance evidence for protocol crates, and supply-chain/maintenance signal."*
- **Current state**: `inventory.md:56-69` Dependency Decisions table only has 4 columns (Capability | Crate decision | State | Public hiding rule). Missing: panic/unsafe audit, license/MSRV/binary-size/platform impact, deterministic test strategy, conformance evidence, supply-chain signal. The dependency snapshots JSON covers feature flags and Ring 5 exclusion but not the audit fields.
- **Remediation**: Either extend the inventory table or create a per-crate `network_http_dependency_audit.md` covering all listed fields for every Ring 2/3/4/5 crate family.

---

## Non-blocking findings (should fix before M0 closes but not strictly fatal)

### N1. Ring 5 absence "proof" is structural, not resolver-backed
`crates/sifr_stdlib/tests/network_http_dependency_snapshots.rs` only checks that the JSON document's string fields don't mention Ring 5 crate names — it is a self-consistency check on a planning artifact, not a `cargo metadata --filter-platform=...`-backed proof. Phase doc DoD `:818` says *"Generated release dependency snapshots prove Ring 5 ... are absent from production feature combinations."* Acceptable for M0 *only if* the JSON status flag (`"status": "m0-planned"`) is honest about this and M5 closes the loop with a real generated snapshot. Recommend adding a comment to the test plus a `network_http_m5_handoff_traceability.md` row pointing at the resolver-backed snapshot that M5 must produce.

### N2. Cross-phase golden network fixture covers only one import
`verification/platform/golden/unsupported_cpython_network_imports.sifr` only triggers `from socket import socket`. The manifest expects only `SIFR-IMPORT-0008` and `sifr.net`, so the test passes — but the unsupported product boundary covers `ssl`, `urllib.parse`, `urllib.request`, `http.client`, `http.server`, `socketserver`, `select`, `selectors`. Add at least one block per family to the golden fixture so a regression in any suggestion mapping is caught.

### N3. E2E fail-fixture coverage is asymmetric
`crates/sifr_stdlib/src/lib.rs` adds 10 legacy module mappings (`sifr.socket`, `sifr.ssl`, `sifr.select`, `sifr.selectors`, `sifr.urllib*`, `sifr.http.client`, `sifr.http.server`, `sifr.socketserver`) and 5 bare CPython suggestions. E2E fail fixtures only cover 4 bare imports (`socket`, `ssl`, `urllib.parse`, `http.client`) and 4 `sifr.*` forms (`sifr.socket`, `sifr.ssl`, `sifr.urllib.parse`, `sifr.selectors`). Missing fail fixtures: bare `select`, `selectors`, `socketserver`, `urllib.request`, `http.server`, and `sifr.select`, `sifr.http.client`, `sifr.http.server`, `sifr.socketserver`, `sifr.urllib`, `sifr.urllib.request`. The unit tests in `lib.rs` exercise the mappings, but the e2e diagnostic surface is the user-visible contract. The phase doc Quality Contract `:1124` says *"Every public module ... must have ... negative diagnostics for unsupported bare CPython import forms."* Recommend filling out the remaining e2e fail fixtures.

### N4. Execution-ledger Decision Index is incomplete
`issues/ad-hoc-production-network-http-platform-substrate-execution.md:286-289` lists only 4 decisions but the contract `:291-298` requires every `deferred-to-phase-X`, `rejected`, `host-limited`, `internal-only`, or `unsupported-with-diagnostic` decision to appear. Missing: HTTP/3/QUIC, WebSocket/CONNECT, Multipart, Content-Encoding compression, internal readiness primitives, internal HTTP transport harness, deferred `metrics` schema, `hickory-resolver` deferral, `x509-parser` deferral, etc.

### N5. Reviewer not yet assigned
`execution.md:251` requires *"Designated compiler/runtime reviewer: assign in the M0 implementation PR before the first implementation milestone is marked complete."* Not yet recorded.

### N6. Local validation results not recorded
`execution.md:211` says PR pending; the Validation Evidence section (`:218-235`) requires `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `python3 scripts/check_hir_maintainability_guardrails.py`, and `scripts/run_all_tests.sh --profile create-pr` results to be checked in before closing M0. Not yet present.

---

## What's working

- Surface classification (`sifr.net`, `sifr.tls`, `sifr.url`, `sifr.http`) uses shared-contract terminal states and stability levels with no `open` states — `inventory.md` is clean.
- CPython evidence matrix covers all 7 families from the contract Evidence Sources table with explicit terminal states; no parity backlog smuggled in.
- Workload database covers async-native vs. `@blocking_io` vs. pure vs. blocked-on-provider classifications, with the rejected import family pointed at the golden fixture and e2e fail set.
- Text/i18n and concurrency/runtime dependency matrices are present in both the inventory and JSON; provider milestone labels match the platform contract vocabulary.
- Serving-scale follow-up has the required stable identifier (`ad-hoc-network-http-serving-scale-follow-up`) recorded in `inventory.md:52`, `supported_host_matrix.md:54`, the phase doc, and the follow-up issue itself.
- No toy/CPython-shaped public module sneaks into `STDLIB_SOURCES` — `legacy_network_http_modules_are_not_embedded_public_sources` test in `lib.rs` proves it.
- Unsupported import diagnostic mappings (`SIFR-IMPORT-0008` for bare CPython, `SIFR-IMPORT-0009` for `sifr.*` legacy) are wired with sensible Sifr-native replacement suggestions.
- Platform contract security/resource ownership rows are added for network/HTTP-owned concerns (parser DoS, TLS verification, flow control, smuggling, URL authority, cookie scope, redaction).
- M1–M5 traceability docs exist with table-shaped backlog entries (Work item | M0 decision | Acceptance evidence). M1 entries (TCP, DNS, split, half-close, UDP gating) are concrete enough to start. M2/M3/M4 entries inherit the gaps from B2–B7.

---

## Required path to PASS

1. Make the M0-required decisions in B1–B10 and check them into `network_http_substrate_inventory.md`/`.json` and, where they belong, into the per-milestone traceability docs.
2. Add the per-crate dependency audit (B11), either by extending the inventory table or adding a `network_http_dependency_audit.md`.
3. Backfill the Decision Index (N4) and execution-ledger reviewer assignment (N5).
4. Expand the golden network fixture (N2) and the e2e fail set (N3) to cover the full rejected/unsupported surface.
5. Add a comment or status field clarifying that Ring 5 absence is a planning record until M5 (N1), or run a real `cargo metadata` snapshot now.
6. Record local validation results before opening the M0 PR (N6).

Once B1–B11 are closed, M1 can safely start. B2 must close before M2. B3/B4/B5/B6/B7 must close before M4. Recommended sequence: B1 + B11 + N5 + N6 first (unblock M0 PR), then B2 (unblock M2), then B3/B4/B5/B6/B7 (unblock M4), with B8/B9/B10 batched alongside.
