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
- A reusable sealed opaque-resource substrate for external Rust-backed Sifr
  packages, to be certified by planned row `opaque_resource_package_core`.
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
  the retained service-specific ecosystem matrix. The separate
  planned `opaque_resource_package_core` row owns the reusable external-package
  resource substrate without pulling service loopbacks into package
  certification.
- `async_runtime_reqwest` split in M6 into stdlib-owned `async_runtime_core`
  and the certification-owned `async_runtime_reqwest` ecosystem loopback row.
  The core row covers async declaration contracts, async-close lifecycle
  validation, current-thread affinity, cancellation/drop task semantics, panic
  conversion through declared stdlib error surfaces, and hidden-blocking
  rejection. This issue still owns `tokio`/`reqwest` loopback behavior evidence.
- `callback_subscription_matrix` splits in M10b into stdlib-owned
  `callback_subscription_core` and certification-owned
  `callback_subscription_ecosystem`. The core row covers signal-style stdlib
  subscription callback policy, async-close cancellation, and shutdown
  contract evidence. This issue still owns runtime-observed ecosystem
  subscription evidence for `tokio-tungstenite`, Redis pub/sub, and `notify`.
- `callbacks_call_scoped` may split into stdlib-owned
  `callbacks_call_scoped_core` if the Python adapter migration proves only the
  core callback lifetime mechanics.
- `panic_boundary_wrapper_emission` remains certification-owned unless a stdlib
  milestone needs generated panic-wrapper evidence; in that case the milestone
  creates a narrow `panic_boundary_stdlib_core` row and leaves package wrapper
  emission and mapper-panic fallback evidence here.

When one of those milestones starts, its PR updates the compatibility matrix,
executable evidence, and the currently owning active plan. This issue continues
to own the ecosystem portions of split rows plus backend, database, messaging,
CLI, native-link, proc-macro, locked/offline Cargo, and package-ecosystem
certification rows that are not direct stdlib migration blockers.

## Handoff to Native Pydantic-Sifr

The Native Pydantic-Sifr architecture consumes two existing
certification-owned rows and one planned row only after this issue lands their
passing evidence:

- `opaque_resource_package_core` for a synthetic external package's sealed
  construct/use/close lifecycle and alias/use-after-close rejection;
- `callbacks_call_scoped` for callback lifetime and storage rejection; and
- `panic_boundary_wrapper_emission` for generated panic mapping and
  mapper-signature rejection.

These rows remain owned here. Pydantic-Sifr milestones may not privately
implement them or claim the service-specific `opaque_resource_matrix`; they
block until the matrix and fixtures are merged through this issue.

### certification_pkg_resource_core

This sequential certification item begins only after Native Pydantic-Sifr
`milestone_ps_2` releases the general package-resource substrate. Its one PR:

- creates the `opaque_resource_package_core` tier-2 fixture and compatibility
  rows (the row is intentionally not pre-created as README-only evidence);
- executes positive construct/use/close and negative alias/use-after-close
  packages through the real compiler/runtime;
- adds the `rust_interop` area to the authoritative legacy profile-runner path
  used by create-PR, merge, nightly, and release profiles, rather than adding
  ignored `selected_areas` data; and
- updates the durable Rust-interop fixture inventory and current matrix counts.

Exit gate: the fixture itself executes, every Rust-interop schema/compatibility
check and the full area runner pass, and
`scripts/run_all_tests.sh --profile create-pr` reports the Rust-interop area as
executed. Only then may `milestone_ps_3` begin.

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
