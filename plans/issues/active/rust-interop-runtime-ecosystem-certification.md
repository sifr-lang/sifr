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

The stdlib native boundary completion phase takes ownership only of the narrow
stdlib-blocking certification mechanics it proves. It must split broad matrix
rows rather than reassigning ecosystem rows wholesale:

- `opaque_resource_matrix` splits into stdlib-owned `opaque_resource_core` and
  certification-owned `opaque_resource_ecosystem`.
- `async_runtime_reqwest` splits into stdlib-owned `async_runtime_core` and the
  certification-owned `async_runtime_reqwest` ecosystem loopback row.
- `callback_subscription_matrix` splits into stdlib-owned
  `callback_subscription_core` and certification-owned
  `callback_subscription_ecosystem`.
- `callbacks_call_scoped` may split into stdlib-owned
  `callbacks_call_scoped_core` if the Python adapter migration proves only the
  core callback lifetime mechanics.
- `panic_boundary_wrapper_emission` remains certification-owned unless a stdlib
  milestone needs generated panic-wrapper evidence; in that case the milestone
  creates a narrow `panic_boundary_stdlib_core` row and leaves package wrapper
  emission and mapper-panic fallback evidence here.

When one of those milestones starts, its PR updates the compatibility matrix,
executable evidence, and milestone evidence table in
`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`. This issue
continues to own the ecosystem portions of split rows plus backend, database,
messaging, CLI, native-link, proc-macro, locked/offline Cargo, and
package-ecosystem certification rows that are not direct stdlib migration
blockers.

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
