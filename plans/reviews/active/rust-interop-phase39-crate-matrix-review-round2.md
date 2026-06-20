I have read the round-2 changes, the round-1 findings, and the current state of both docs. Below are findings ordered by severity. No blockers remain; H1–H3 and L1/L2/L4 are fully resolved. There are material gaps left in fixture-to-directory mapping, feature pinning for a few crates, and one implementation hazard around `sqlx macros`.

## Round-1 closeout summary

| Round-1 finding | Status |
| --- | --- |
| H1 — non-fixture rows in phase doc | **Resolved.** Phase tables now contain only production crates; `insta`/`proptest`/`criterion` and `pyo3`/`maturin`/`cbindgen`/`uniffi`/`napi-rs` are gone. |
| H2 — axum/tower-http carve-out | **Resolved.** Architecture line 889 carries the carve-out sentence. |
| H3 — table shape divergence | **Resolved.** Architecture lines 861–887 now mirror the phase doc's three tables and split blocking vs opaque resource and build vs native-link. |
| M1 — `or` choices | **Resolved.** tokio-postgres / candle / flate2 / prost-build all canonical; sqlx correctly moved to ecosystem certification (defended below). |
| M2 — crate→fixture mapping | **Partial.** `*_matrix` directories are annotated; several non-matrix fixtures still have no crate mapping and ecosystem certification has no home directory. (See M-A below.) |
| M3 — Cargo feature pinning | **Partial.** Feature policy section added; rusqlite, tokio-postgres `runtime`, candle, flate2 backends still unpinned. (See M-B, M-C, P-A.) |
| L1 — service execution policy | **Resolved.** Fixture execution policy section in phase doc + architecture line 893. |
| L2 — terminology drift | **Resolved.** Both docs say "Crate Verification Matrix". |
| L3 — `thiserror` classification | **Acceptable.** Verification-purpose text now reads "explicit error types"; row still bundles concerns but the purpose statement covers it. |
| L4 — `bridge_version_mismatch` fixture | **Resolved.** Added at `internal_docs/rust_interop_architecture.md:826`; milestone_39_3 calls it out. |

## Material issues (resolve before fixtures are authored)

### M-A. Ecosystem certification crates have no fixture directory
The verification tree at `internal_docs/rust_interop_architecture.md:808-849` has no directory that owns `axum`/`tower-http`/`sqlx` or `clap`/`tracing`/`tracing-subscriber`/`anyhow`. Both docs *require* these to be certified (architecture lines 884–889, phase lines 274–279), but an implementer reading the tree has nowhere to put the fixtures.

Recommendation: add two leaf directories with explicit annotations, e.g.:

```
ecosystem_backend_certification/   # axum, tower-http, sqlx
ecosystem_cli_certification/       # clap, tracing, tracing-subscriber, anyhow
```

Without these, implementers will either invent ad-hoc directory names (drifting from the normative tree) or quietly fold ecosystem crates into the `*_matrix` fixtures, contaminating the contract isolation the round-1 fix was designed to preserve.

### M-B. `sqlx` with `macros` needs an offline-artifact policy
Phase line 287 pins `sqlx` with `features = ["runtime-tokio-rustls", "postgres", "macros"]`. The `macros` feature enables compile-time SQL checking, which fails the build unless `DATABASE_URL` is set at compile time **or** an offline `.sqlx/query-*.json` cache is checked in (`cargo sqlx prepare`). Neither doc says which path the ecosystem certification fixture uses.

This is a hard build-time hazard, not a runtime one — leaving it unspecified means the first implementer will hit `error: set `DATABASE_URL` to use query macros` and improvise. Recommendation: explicitly require offline mode for ecosystem certification (`.sqlx/` artifacts checked in alongside the fixture), or drop `macros` from the pinned feature set and document that compile-time SQL checking is exercised through a separate, runtime-service-tier fixture.

### M-C. `tokio-postgres` pin is incomplete
Phase line 284 and architecture line 891 both pin `tokio-postgres` with `default-features = false`. `tokio-postgres`'s default features include `runtime`, which is what wires it to a Tokio reactor. With `default-features = false` and nothing else, the crate cannot be used asynchronously — the async opaque-resource fixture will fail to assemble.

Recommendation: change the pin to `default-features = false, features = ["runtime"]` (and add an explicit TLS feature if the fixture exercises TLS — likely `features = ["runtime", "with-uuid-1"]` or a `tokio-postgres-rustls` companion crate if TLS is required). This matters because `tokio-postgres` is the chosen exemplar for the opaque-resource matrix; an underspecified pin blocks the fixture entirely.

### M-D. `rusqlite` features unpinned
`rusqlite` appears in both the opaque-resource matrix and the blocking/CPU-heavy matrix. Its `bundled` feature controls whether sqlite is statically built from vendored C source (no system dep, but exercises `cc` + native-link evidence) or links against system sqlite (different native-links trust requirements). Neither doc names a default.

Recommendation: pin `rusqlite` with `features = ["bundled"]` (matches the intent of `cc`-driven native-link trust evidence in the same fixture group) and call out that the unbundled variant is intentionally not certified in Phase 39. Document the choice in both phase doc and architecture line 891.

## Crate-choice challenges (point 2 of the ask)

- **`tokio-postgres` vs `sqlx`** — accept current split. Putting `tokio-postgres` in opaque resources isolates the opaque-handle/async contract from proc-macro trust; placing `sqlx` in ecosystem certification then independently exercises the macros×async×opaque-resource intersection. This is **stronger** than the round-1 recommendation of consolidating on `sqlx`. Keep, conditional on M-B above.
- **`candle` vs `ort`** — accept. Pure-Rust isolation of DLPack metadata from native-link trust is correct. Worth noting candle has GPU backend feature flags (`cuda`, `mkl`, `accelerate`); the fixture should be explicit that the default Rust-only backend is used (see P-B).
- **`flate2` vs `zstd`** — accept. Avoids double-coverage with `zstd` in native links. But flate2 has three backends (`rust_backend` (default), `zlib`, `zlib-ng`); should be pinned (see P-A).
- **`prost-build` vs `tonic-build`** — accept. Lighter trust surface, narrower dependency tree, sufficient for proc-macro + build-script certification. `tonic-build` can be a later add if gRPC certification becomes part of stable's scope.

## Polish

### P-A. `flate2` backend not pinned
Default is `rust_backend` (pure Rust, no native link). That's the right default for a "blocking/CPU-heavy" matrix entry, but it isn't explicit. Add `flate2 = { features = ["rust_backend"], default-features = false }` to the pinned-feature list in both docs.

### P-B. `candle` backend not pinned
Same reasoning. Add `candle: default features (CPU-only)` and call out that GPU/accelerator backends are explicitly out of scope for Phase 39.

### P-C. Pinned-feature drift between docs
Architecture line 891 omits `tokio-tungstenite`, `axum`, and `tower-http` pins that exist in phase lines 286–288. Either drop them from phase doc or mirror them into architecture so the canonical statement matches the phase enumeration. Architecture is normative; favour mirroring.

### P-D. `thiserror` row labeling
Bridge type row 866/258 still lumps `thiserror` with `serde`-derived records. Optional cleanup: split into a `Generated bridge error types: thiserror-backed Rust errors → Sifr error conversion` purpose row, or keep as-is and accept the current verification-purpose phrasing. Not load-bearing.

### P-E. `tokio` itself has no annotated fixture home
Phase line 261 lists `tokio` in the Async/Tokio ecosystem row, but no fixture annotation names it directly. It's implicit in `async_runtime_reqwest/`, `async_ecosystem_matrix/`, and the tokio-postgres/redis fixtures. Worth one inline sentence in the architecture verification area saying "`tokio` runtime behavior is exercised transitively through async_runtime_reqwest, async_ecosystem_matrix, opaque_resource_matrix, and callback_subscription_matrix." Avoids implementers creating a redundant `tokio_runtime/` fixture.

---

**Bottom line:** the docs are aligned and implementation-ready for the core path. The remaining material issues are all narrow: one missing fixture directory (ecosystem certification), one unresolved sqlx-macros build policy, and three missing feature pins for crates that have build-policy-relevant flags (tokio-postgres `runtime`, rusqlite `bundled`, flate2 backend). None block design lock; all of them will block the first implementer if not resolved before fixtures are authored.
