# Rust Interop Runtime and Ecosystem Certification Follow-Up

## Status

Active follow-up created by Phase 39 closeout.

## Objective

Convert every `future-owned-by-separate-phase` row in the Rust interop
compatibility matrix into either passing support evidence or an explicit
unsupported-by-design decision before any stable release claims that surface.

## Scope

- Full bridge type value roundtrips for `serde`, `serde_json`, `thiserror`,
  `bytes`, and `indexmap`.
- Resource-shaped opaque handles for `reqwest`, `rusqlite`, `tokio-postgres`,
  and `redis`.
- Generated panic wrapper emission and mapper-panic fallback behavior.
- Loopback `reqwest` async runtime behavior.
- Call-scoped callbacks and subscription callbacks over `tokio-tungstenite`,
  Redis pub/sub, and `notify`.
- Backend/service package certification for `axum`, `tower-http`, and `sqlx`.
- CLI/tooling package certification for `clap`, `tracing`,
  `tracing-subscriber`, and `anyhow`.
- Native-link/build-script certification for `cc`, `bindgen`, `cxx`, and
  `zstd`.
- Proc-macro/build-script certification for `serde_derive` and `prost-build`.
- Locked, offline, and frozen Cargo certification.

## Handoff to Stdlib Native Boundary Completion

The stdlib native boundary completion phase takes ownership of the
stdlib-blocking certification rows it consumes:

- `opaque_resource_matrix` when the file/resource migration lands.
- `async_runtime_reqwest` when the async runtime pilot lands.
- `callback_subscription_matrix` when the signal subscription pilot lands.
- `callbacks_call_scoped` when the Python adapter migration lands.

When one of those milestones starts, its PR updates the compatibility matrix
`future_owner`, executable evidence, and milestone evidence table in
`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`. This issue
continues to own backend, database, messaging, CLI, native-link, proc-macro,
locked/offline Cargo, and package-ecosystem certification rows that are not
direct stdlib migration blockers.

## Required Evidence

Each row must land with:

- executable positive and negative fixtures under
  `verification/areas/rust_interop/fixtures`,
- runner wiring that executes the fixture rather than relying on README-only
  evidence,
- compatibility-matrix status updates that move the row out of
  `future-owned-by-separate-phase`, and
- reviewer sign-off that the public docs do not overclaim the surface.

## Stable Release Constraint

Phase 40 stable promotion must not claim support for any Rust interop surface
that remains `future-owned-by-separate-phase` in
`verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`.
