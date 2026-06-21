Here is my review of the M39.2 staged diff. Findings ordered by severity, with file/line references.

---

## P0 — Blocking

### 1. `panic=trusted_no_panic` and `panic=abort` trust gates are unreachable
`crates/sifr_driver/src/build/rust_interop.rs:297-316`, `:489-494`

`validate_declaration_trust` calls `has_truthy_argument(declaration, "trusted_no_panic")` and `has_truthy_argument(declaration, "panic_abort")`, where `has_truthy_argument` requires `argument.name == "<x>"` AND `value == Boolean(true)`. The architecture's syntax is:

```
@rust(crc32fast.hash, panic=trusted_no_panic)
@rust(legacy_backend.run, panic=abort)
```

`crates/sifr_lowering/src/lower/rust_interop.rs:201` lowers identifier RHS values into `RustInteropValue::Symbol(...)`. So an actual decorator yields `argument.name = "panic"` and `value = Symbol("trusted_no_panic")`, never `name = "trusted_no_panic"` with `Bool(true)`. Both `rust-no-panic` and `rust-panic-abort` gates therefore never fire — the M39.2 DoD explicitly lists these among the required trust gates. There is also no negative test covering them.

Fix: match against `argument.name == "panic"` with `RustInteropValue::Symbol("trusted_no_panic" | "abort")`, and add fixtures covering both the symbol and untrusted decorator paths.

---

## P1 — Should fix before merging

### 2. `build-env` trust gate is declared but never validated
`crates/sifr_codegen/src/rust_interop_plan.rs:127`, `crates/sifr_driver/src/build/rust_interop.rs:233-317`

`RustInteropTrustRequirementKind::BuildEnv` exists, and `RustInteropCargoInputs.declared_build_env` is plumbed through, but no `require_trust(..., BuildEnv, ...)` call exists. The M39.2 DoD calls this out explicitly. There is also no test for it.

### 3. Cache inputs that M39.2 calls out are hard-coded empty/None
`crates/sifr_driver/src/build/rust_interop.rs:595-614`

`cargo_metadata_digest: None`, `cargo_lock_digest: None`, `profile_codegen_settings: Vec::new()`. M39.2 DoD: "Cache invalidation changes when any … Cargo lock state, target, feature, or trust input changes." Without the lock digest, cache won't invalidate on a `Cargo.lock` change; without profile codegen settings, cache won't invalidate on `opt-level`/`lto`/`codegen-units` changes — both explicitly required by the milestone.

### 4. Target/profile/panic inputs come from build-script env vars unset during compilation
`crates/sifr_driver/src/build/rust_interop.rs:601-607`

```rust
target_triple: std::env::var("TARGET").ok(),
target_features: std::env::var("RUSTFLAGS").ok().into_iter().collect(),
cargo_profile: std::env::var("PROFILE").unwrap_or_else(|_| "dev".to_string()),
panic_strategy: std::env::var("SIFR_RUST_PANIC_STRATEGY").ok(),
```

`TARGET` and `PROFILE` are only injected by Cargo into build scripts — they're unset when a user runs `sifr build`/`sifr run`. So in real builds these are all `None`/`"dev"`/empty, defeating cache invalidation when the user changes target triple or feature set. M39.2 DoD requires invalidation on target/feature change. The values must come from the actual build configuration (e.g., the cargo invocation in `materialize.rs:170-194` always builds `--release`, so the profile is known; the target should be propagated from CLI/env Sifr controls).

### 5. `unsafe-rust-bridges` trust matching uses canonical Sifr path, architecture says bridge file path
`crates/sifr_driver/src/build/rust_interop.rs:287-296`, architecture `internal_docs/rust_interop_architecture.md:253`,`:718`

The architecture's documented entry shape is the bridge file path (`unsafe-rust-bridges = ["src/bridges/tokenizer.rs"]`). The resolver compares against `canonical_target_path` (`app.hash`). The validation evidence in the trust requirement also disagrees with the entry shape. Either the architecture or the implementation must be updated; if the entry is intentionally a canonical target path, update the architecture and `docs/errors/SIFR-RUST-TRUST-0001.md`.

### 6. RustBridgeProbePlan never actually probes
`crates/sifr_driver/src/build/rust_interop.rs:353-378`, M39.2 DoD bullet 4

The DoD says "Probe failures map rustc diagnostics to `SIFR-RUST-RESOLVE-*` or `SIFR-RUST-TYPE-*` diagnostics at the original decorator span." The current code only plans probes; it does not invoke `cargo check` against an isolated probe module. `RUST_TYPE_PROBE_FAILURE` only fires when `probe_kind` returns `None` (e.g., a non-Opaque decorator on a class), so it cannot surface signature/visibility/asyncness mismatches as M39.2 requires.

This is consistent with the test gap: `SIFR-RUST-TYPE-0001`'s `representative_fixture_path` in `registry_entries/rust_interop.rs:46` and in `code_catalog.json` points to `package_rust_interop_records_probe_plan`, which asserts a **successful** plan — not a probe failure. There is no fixture exercising the diagnostic.

### 7. Duplicate trust requirements (and diagnostics) per declaration with multiple target paths
`crates/sifr_driver/src/build/rust_interop.rs:149-231`, `:281-317`

`resolve_path` is called for the decorator's target *and* for every nested `TargetPath` argument value (e.g., `panic=map_error(bridge.hash.map_panic)`). It calls `validate_declaration_trust` on every iteration. `uses_bridge_root` (and once the P0 panic-symbol issue is fixed, the panic checks) are properties of the whole declaration, not of individual paths. Result: declarations with N paths emit N copies of the same trust requirement, and N copies of the diagnostic on failure. Move `validate_declaration_trust` out of the per-path loop, and dedupe `trust_requirements` if a kind/path pair already exists.

---

## P2 — Recommended

### 8. `digest_path` silently swallows fs::read errors and recurses without filter
`crates/sifr_driver/src/build/rust_interop.rs:553-584`

`if let Ok(bytes) = fs::read(path)` silently skips unreadable files, so a transiently locked file or permission flap produces a different (and potentially smaller) digest with no diagnostic — cache poisoning risk. The recursion also follows symlinks (default `fs::read_dir`/`fs::read`) and includes every file with no extension filter, so an editor swap file or `.DS_Store` left in `src/bridges/` shifts the cache key. Surface I/O errors as a `BUILD_MATERIALIZATION_FAILURE`-shaped diagnostic and filter to `.rs` (or otherwise canonicalize what counts as bridge content).

### 9. `bridge_root` digest serialization is platform-dependent
`crates/sifr_driver/src/build/rust_interop.rs:541-549`, `:569-573`

Bridge root and relative file paths are serialized via `Path::display()`. On Windows that emits `\` separators, so cache keys diverge across platforms even when the source tree is identical. Use a forward-slash normalization (e.g., join components with `'/'`).

### 10. `tool_version` invokes cargo/rustc as a side effect of cache-key computation
`crates/sifr_driver/src/build/rust_interop.rs:609-610`, `:651-659`

The cache key contains the live `cargo --version` and `rustc --version` output. Beyond the wall-clock cost (two subprocesses per build), the cache key shifts silently if `cargo` or `rustc` is missing from PATH (returns `None`). Consider memoizing per process and at minimum logging when probing fails, and verify these are actually obtained from the same toolchain that will run the embedded build.

### 11. `emit` path can't reach Rust interop context
`crates/sifr_driver/src/build/entrypoint.rs:191-203`

`emit_project_entrypoint` uses `RootedEntrypoint::Project`, which always sets `rust_interop_context = None` (lines 397). So `sifr emit` on any package source containing `@rust(...)` will fail with RUST-CARGO-0001, even if the package is valid. Either a package-aware emit entrypoint is needed or this should be called out as an intentional M39.2 limitation.

---

## P3 — Minor

### 12. `Self`-on-non-method diagnostic message is wrong wording
`crates/sifr_driver/src/build/rust_interop.rs:177-189`

Template is `"unresolved Rust target root `{root}`"`. The real failure is "invalid context for `Self`" — the root itself is recognized. Consider a dedicated diagnostic message; current wording will confuse users putting `Self.` on a free function.

### 13. `fnv1a64` is duplicated
`crates/sifr_driver/src/build/rust_interop.rs:642-649` vs `crates/sifr_package/src/graph/digest.rs:71-78`

Two identical implementations. Lift into a shared util to avoid drift.

### 14. `apply_package_rust_interop_metadata` early-returns when there are no declarations
`crates/sifr_driver/src/build/rust_interop.rs:53-55`

If the package's trust policy changes but no Rust interop declarations exist yet, `cargo_inputs` and `bridge_sources` aren't written. The wider `package_graph_digest` (graph/digest.rs) does include the new `CanonicalRustTrust` fields, so package-level cache invalidation still picks up trust changes, but `interop.cache_key_fragment` will be empty even when bridge sources exist. Low risk in practice; flag for awareness.

---

## Test gaps that should block merging

- **No fixture for SIFR-RUST-TYPE-0001 actually firing.** The representative fixture in the registry asserts success. Add a probe-failure fixture (or update the registry to point at a real negative test once probe execution lands).
- **No fixture for `bridge` root resolution to `PackageBridge`** — only `DirectCargoDependency` is exercised.
- **No fixture for `Self` root** on a method — neither success nor failure.
- **No fixture for proc-macro/native-links/unsafe-rust-bridges trust gates** firing (only build-script success and failure are covered).
- **No fixture for `panic=trusted_no_panic`/`panic=abort` symbol-style decorator values** (related to P0).
- **No `build-env` fixture** (related to P1 #2).
- **No determinism fixture** asserting that two consecutive `apply_package_rust_interop_metadata` runs produce identical `cache_key_fragment` (especially around `tool_version` and env-var inputs).
- **No fixture covering cache-key invalidation across bridge-source mutation, trust-policy mutation, or backend-crate dependency_name remapping (`package = "..."` rename).**
- **No fixture for transitive Cargo dependency rejected as a Rust target root** (M39.2 DoD: "direct Cargo dependency roots").

---

## Verdict

**Not cleared for M39.2.** The P0 panic-trust gap and the P1 cache-input gaps (lock digest, profile codegen settings, target triple sourcing) materially break milestone DoD bullets ("Add trust gates for … rust-no-panic … rust-panic-abort" and "Cache invalidation changes when any … Cargo lock state, target, feature, or trust input changes"). The probe-plan side (P1 #6) and the unsafe-bridge entry-shape mismatch (P1 #5) also need resolution — either by implementing probes and file-path matching, or by tightening the M39.2 scope statement and architecture doc to match the implementation. Recommend fixing P0/P1 plus the test gaps above before merging.
