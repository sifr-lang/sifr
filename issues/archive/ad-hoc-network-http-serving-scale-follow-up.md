# Ad Hoc Follow-Up: Network HTTP Serving Scale Strategy

Status: draft
Stable identifier: `ad-hoc-network-http-serving-scale-follow-up`
Owner: concurrency/runtime plus runtime/networking

## Objective

Define the production serving-scale strategy that is intentionally outside the network/HTTP substrate phase.

The network/HTTP substrate phase provides protocol-correct async accept, dispatch, TLS, HTTP/1.1, HTTP/2, backpressure, limits, typed errors, and shutdown semantics for one current-thread runtime worker per Sifr process. It does not claim multi-core serving throughput readiness.

## Scope

- Decide whether Sifr serving scale should use host-limited `SO_REUSEPORT` multi-process serving, provider-owned process workers, a future provider-owned Tokio `rt-multi-thread` topology, or another Sifr-native model.
- Define supervision, shutdown, signal, cancellation, diagnostics, and request-context propagation across workers.
- Define host support for macOS arm64, Linux x86_64, and Windows x86_64.
- Define benchmark and load-test evidence required before Phase 41 can claim multi-core serving throughput readiness.

## Non-Goals

- No change to the network/HTTP substrate phase's current-thread runtime topology.
- No public raw Tokio runtime handles, event-loop policies, or selector APIs.
- No CPython `socketserver.ThreadingMixIn` or `ForkingMixIn` compatibility path.

## Dependency

This follow-up consumes the completed network/HTTP substrate, concurrency/runtime process and shutdown substrate, and Phase 41 framework handoff requirements.
