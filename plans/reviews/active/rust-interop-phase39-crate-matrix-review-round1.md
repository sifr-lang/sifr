I have read both documents. Below are findings ordered by severity, with concrete edit recommendations. No blockers — the contracts are coherent and implementation-ready. The findings are alignment, scoping, and ambiguity polish.

## High-severity (resolve before `milestone_39_0` closes)

### H1. Phase doc adds crate matrix rows that don't exist in the (normative) architecture doc
`plans/phases/39_rust_interop.md:280-281` introduces two rows under "Ecosystem certification fixtures":
- "Verification tooling references" with `insta`, `proptest`, `criterion`
- "Interop design references" with `pyo3`, `maturin`, `cbindgen`, `uniffi`, `napi-rs`

The architecture doc (`internal_docs/rust_interop_architecture.md:858-868`) lists only the production matrix and explicitly omits these. Since the phase doc itself declares (line 239) that the architecture document is normative for the area, the architecture must own these entries or the phase doc must drop them.

Recommendation: **drop both rows from the phase doc**. They aren't verification fixtures — `insta`/`proptest`/`criterion` are the verification runner's *internal* tools (they belong in the runner section of the architecture doc, not in a fixture matrix), and `pyo3`/`maturin`/`cbindgen`/`uniffi`/`napi-rs` are study material (they belong in an "Interop design references" appendix in the architecture doc, not in any matrix table). Putting them under "Ecosystem certification fixtures" misclassifies non-fixtures as fixtures.

If they must stay, rename the row labels to something other than "fixtures" and mirror the same entries into `internal_docs/rust_interop_architecture.md` so the docs agree.

### H2. The axum/tower-http carve-out conflicts with the architecture's "no product-level web frameworks"
`plans/phases/39_rust_interop.md:287` says Phase 39 *may* certify `axum`/`tower-http` "package compilation and probing", and lines 277-278 makes them required ecosystem-certification crates. The architecture doc (`internal_docs/rust_interop_architecture.md:868-870`) lists `axum`, `tower-http`, `sqlx` in the ecosystem-certification line and then says the matrix "does not move ... product-level web framework work into Phase 39."

The architecture doesn't actually carve out the same "compilation and probing only" exception that the phase doc relies on. A reader of the architecture alone would not know that axum/tower-http compile/probe is in-scope but web-framework workflows are not.

Recommendation: in `internal_docs/rust_interop_architecture.md`, immediately after the ecosystem certification line, insert a one-sentence clarification matching the phase doc — e.g. "Ecosystem certification for `axum`, `tower-http`, and `sqlx` is limited to canonical-package compilation and probe coverage; product-level web framework workflows remain out of Phase 39."

### H3. Architecture's production matrix and phase's "required" tables disagree in shape
Architecture (`internal_docs/rust_interop_architecture.md:858-868`) presents a single flat production matrix; the phase doc (`plans/phases/39_rust_interop.md:255-281`) splits it into "Required core", "Required advanced", and "Ecosystem certification" tables with `Verification purpose` columns. The crates mostly match, but:

- Architecture lumps "opaque resources and blocking work" into one line with `reqwest::Client, rusqlite, tokio-postgres or sqlx, redis, rayon, flate2 or zstd`. Phase doc splits this into two rows (Opaque resources vs. Blocking and CPU-heavy) with `rusqlite` repeated in both.
- Architecture has `serde_derive, prost-build or tonic-build, cc, bindgen, cxx, zstd` as a single trust/build/native line. Phase doc splits into "Build and proc-macro trust" and "Native/build links".

The split is reasonable and more informative; the problem is that the architecture's prose line still reads as the canonical statement. Since the architecture is the normative source for matrix shape, it should match the phase doc's structure.

Recommendation: replace the bullet list at `internal_docs/rust_interop_architecture.md:858-868` with the same three tables (Required core / Required advanced / Ecosystem certification) that exist in the phase doc, or reduce the phase doc to a pointer that says "see architecture doc for the canonical matrix" and put all the structure in the architecture doc.

## Medium-severity (close before fixtures are authored)

### M1. `or` choices remain unresolved across both docs
Multiple matrix entries use "X or Y" without selecting one:

- `tokio-postgres or sqlx` (architecture line 864, phase line 262, milestone scope line 140) — these have very different surfaces (sqlx uses compile-time-checked SQL via proc macros, which couples it to the proc-macro trust gate; tokio-postgres has a plain async API). Picking changes which trust-policy fixture row applies.
- `candle or ort` (architecture line 866, phase line 271) — `ort` needs the ONNX Runtime native library (a `native-links` fixture) and `candle` is pure Rust. The choice changes whether the tensor fixture also exercises native-link trust.
- `flate2 or zstd` (architecture line 864, phase line 263) — `zstd` already appears in "Native/build links". Picking `zstd` here doubles up and reduces coverage; `flate2` is a cleaner blocking-IO/CPU exemplar without re-exercising native-link evidence.
- `prost-build or tonic-build` (architecture line 862, phase line 259) — both run build scripts, but `tonic-build` pulls more dependencies. The choice affects build-time cost and proc-macro trust scope.

Recommendation: pick one canonical crate per pair and record the choice in both docs. Suggested resolutions (defaults, not mandates):

- opaque resources: `sqlx` (covers both async DB and proc-macro trust intersection), drop the `or`.
- tensors: `candle` (pure Rust, isolates DLPack metadata behavior from native-link trust).
- blocking/CPU-heavy: `flate2` (avoids double-coverage with `zstd` in native links).
- build trust: `prost-build` (lighter, narrower trust surface; `tonic-build` then becomes a follow-up if needed).

### M2. Crate categories aren't mapped onto fixture directory names
The architecture's verification tree (`internal_docs/rust_interop_architecture.md:808-848`) names fixtures (`direct_crate_matrix`, `bridge_type_matrix`, `opaque_resource_matrix`, `async_ecosystem_matrix`, `callback_subscription_matrix`, `zero_copy_view_matrix`, `advanced_data_matrix`, ...) but never says which matrix crates land in which matrix fixture. The phase doc tables enumerate crates by purpose but don't bind them to fixture directories either.

For implementation, this leaves ambiguity. Example: do `sha2`/`uuid`/`regex` live under `direct_crate_matrix`? Do `serde`/`serde_json`/`thiserror`/`bytes`/`indexmap` all live under `bridge_type_matrix`?

Recommendation: in the architecture doc's verification area section, annotate each `_matrix` fixture with the crate list it covers. Example:

```
direct_crate_matrix/      # sha2, uuid, regex
bridge_type_matrix/       # serde, serde_json, thiserror, bytes, indexmap
opaque_resource_matrix/   # reqwest::Client, sqlx, redis
async_ecosystem_matrix/   # futures, tower, http, http-body
zero_copy_view_matrix/    # memmap2, bytemuck, zerocopy
advanced_data_matrix/     # datafusion, polars, ndarray, candle
callback_subscription_matrix/ # tokio-tungstenite, redis pub/sub, notify
```

`direct_crate_crc32`, `local_bridge_blake3`, `async_runtime_reqwest`, `arrow_record_batch`, `tensor_dlpack_bridge`, `zero_copy_bytes` already isolate their headline crates.

### M3. No Cargo feature set is pinned for any matrix crate
The phase doc intro (`plans/phases/39_rust_interop.md:251`) requires each fixture to record "the Cargo feature set, target triple, lock state, and any trust policy required" — good — but neither doc pins the actual features for the matrix crates. For example:

- `reqwest` behaves very differently under `default`, `rustls-tls`, `native-tls`, or `blocking`. Async, native-link, and trust evidence all shift with the feature choice.
- `sqlx` requires explicit runtime + TLS features.
- `tracing-subscriber` requires `env-filter` to exercise common interop paths.

Without pinning, fixture authors will make incompatible choices and cache keys will differ across machines.

Recommendation: add a `Cargo features` column to each phase-doc table (or an inline note per crate), at least for the network/TLS/runtime-sensitive ones (`reqwest`, `sqlx`, `tokio-postgres`, `tracing-subscriber`, `axum`, `tower-http`, `tonic-build`/`prost-build`).

## Low-severity (polish)

### L1. Service-dependent fixtures have no recorded execution policy
Crates like `reqwest`, `tokio-postgres`, `sqlx`, `redis`, `redis pub/sub`, and `tokio-tungstenite` require live services to actually exercise opaque-resource and callback contracts. Neither doc says how those fixtures run in CI — real service, mock loopback, recorded protocol, or compile/probe-only. The runner inventory (`cargo_probe.py`, `bridge_check.py`, `trust_check.py`, `native_probe.py`, `report.py`) suggests probe-only is the default, but that wouldn't cover state-transition coverage for `opaque_resource_matrix`, `close_after_use`, or `callbacks_threadsafe`.

Recommendation: add a short subsection to the architecture doc verification area explicitly stating which fixtures are compile/probe-only vs. require a runtime service, and how runtime-service fixtures are expected to run (loopback container, recorded transcript, in-process stub). This will prevent ambiguity once implementation begins.

### L2. Terminology drift between docs
Architecture says "production crate matrix" (line 858); phase doc says "Crate Verification Matrix" (line 249). They refer to the same artifact.

Recommendation: pick one name (suggest "Crate Verification Matrix") and use it in both docs.

### L3. `thiserror` classification reads ambiguously
Phase doc places `thiserror` under "Bridge type generation and conversion" (line 258). `thiserror` is a derive macro for *Rust user error types* — Sifr's bridge generates its own error structs. The verification value is likely "Rust crates that expose `thiserror`-based errors can be projected through the bridge", which is conversion, not generation. The current label conflates two concerns.

Recommendation: either split into two purpose entries (e.g., "Generated bridge error types: `serde`-derived records and `thiserror`-based Rust errors") or move `thiserror` to its own row labeled "Rust user error → Sifr error conversion".

### L4. `bridge-version` change semantics aren't in the matrix
The architecture doc defines `bridge-version = 1` (lines 259, 261-268) and the cache key (line 751). But neither doc names a fixture that exercises a `bridge-version` mismatch — that's the only way to validate the schema gate at archive validation time.

Recommendation: add a `bridge_version_mismatch` fixture (or include it under `cargo_locked_offline`/`shared_bridge_crate`) and call it out in the milestone scope under `milestone_39_3` or `milestone_39_13`.

---

Summary: no blockers. H1–H3 are doc-alignment fixes the user should make before fixtures land. M1–M3 are the only things that could materially slow implementation if left ambiguous — pinning `or` choices, mapping crates to `*_matrix` fixture directories, and recording Cargo feature sets. The rest is polish.
