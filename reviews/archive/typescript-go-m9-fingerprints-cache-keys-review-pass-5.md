## Verdict: **NOT SATISFIED**

M9 introduces a clean deterministic fingerprint helper and a complete family-level key taxonomy, but the diff fails two explicit requirements (negative tests, typed enums for free-form style/scope strings) and leaves at least one deterministic-identity concern in `HirLoweringCacheKey`.

---

## Findings

### 1. Missing negative tests for intentionally omitted cache-key inputs (HIGH — explicit requirement)

The M9 brief says: *"add negative tests for intentionally omitted cache-key inputs."* Every test in `cache_keys.rs::tests` is a *positive* assertion that mutating an included field changes the fingerprint (`parse_key_includes_source_compiler_workspace_package_and_policy`, `source_map_key_includes_line_map_algorithm`, etc.). There is no test that demonstrates a field which is intentionally NOT part of a given key actually leaves the fingerprint unchanged.

Examples of the negative cases that should be exercised:
- `DocumentVersion` must not affect parse/source-map/HIR/diagnostics/lint/format/symbol/flow keys.
- `SourceUri` and `FileId` must not affect any content-derived key.
- The package graph manifest path string must not influence parse, source-map, HIR, or symbol-bucket keys.
- `bucket_scope` of a parse/source-map key must not be reused.
- Different `SymbolKind` distributions inside the same module must not influence lint, format, package-graph, or flow-graph keys.
- The compiler fingerprint's `flow_graph_policy` field must not change a parse or source-map key.

Without these, a future contributor cannot tell which fields are *deliberately* excluded and may accidentally fold them in (or fail to fold them in) when M10 lands reuse.

File: `crates/sifr_frontend/src/cache_keys.rs` (tests module, lines ~525-718).

### 2. `diagnostic_style: String` and `bucket_scope: String` should be typed enums (MEDIUM — type-safety / API concern)

- `DiagnosticsCacheKey::diagnostic_style` is a `String` populated with literal strings like `"bare"` / `"module-prefixed"` in tests. The codebase already exposes `FrontendDiagnosticStyle` (used in `source_maps.rs` and `query_diagnostics.rs`); the key should hold a `FrontendDiagnosticStyle` so callers can't typo a style.
- `SymbolBucketsCacheKey::bucket_scope: String` is also free-form. Bucket scope is exactly the kind of value that should be a closed enum (`SymbolBucketScope::Workspace`, `SymbolBucketScope::Package`, `SymbolBucketScope::Module`, …) so a misspelling silently becomes a cache key collision rather than a compile error.

File: `crates/sifr_frontend/src/cache_keys.rs` lines defining `DiagnosticsCacheKey` and `SymbolBucketsCacheKey` (around the `pub struct ...` blocks). Test fixture uses `"bare".to_string()` and `"workspace".to_string()` which would not compile against a typed enum — this is precisely the regression risk the M9 contract should prevent.

### 3. `ParseCacheKey` and `FormatCacheKey` cannot carry user-specified options (MEDIUM — key-input gap)

- `ParseCacheKey::new` always sets `parser_options` to `QueryPolicyFingerprint::default_for_cache_family(CacheFamily::Parse)`. There is no constructor that accepts a caller-supplied parser policy fingerprint, so any future experimental parser option (e.g., a per-project `parser_pragma`) cannot influence parse-cache identity.
- `FormatCacheKey` is even more exposed: it has no `formatter_options` field at all — only `formatter_policy`, which is also pinned to the default. Real formatter output depends on the user's `sifr.toml` formatter config (line width, indent style, quote style). Two different `sifr.toml` configurations sharing a project will therefore collide on the format key.

For HIR and diagnostics, the analogous fields (`compiler_options`, `diagnostic_style`) are public and overridable. The asymmetry will bite when M10 turns these keys into live cache lookups.

File: `crates/sifr_frontend/src/cache_keys.rs` `ParseCacheKey::new` / `FormatCacheKey` struct definitions.

### 4. `HirLoweringCacheKey` fingerprints `compiler_options` via `Debug` (MEDIUM — deterministic-identity concern)

```rust
builder.field("compiler_options", format!("{:?}", self.compiler_options));
```

`WorkspaceCompilerOptions` is a Rust struct whose `Debug` output is sensitive to field order, generic parameter formatting, and any added/renamed fields. Two semantically equivalent configurations that serialize differently through `Debug` (e.g., one with a future `bool` defaulting to `false`, another with the field absent) will produce different HIR keys, and reordering the struct will invalidate every persisted HIR cache.

Either:
- derive a stable `Serialize` view of `WorkspaceCompilerOptions` and feed that into `FingerprintBuilder`, or
- pass the relevant `QueryPolicyFingerprint` explicitly and drop the `Debug` formatting.

File: `crates/sifr_frontend/src/cache_keys.rs` `HirLoweringCacheKey::fingerprint`.

### 5. Public `for_testing` constructors leak into the production API (LOW)

`CompilerFingerprint::for_testing`, `WorkspaceContextFingerprint::for_testing`, and `PackageContextFingerprint::for_testing` are all `pub fn` with no `#[cfg(test)]` guard. They produce a value indistinguishable from a real fingerprint, so a downstream crate can ship a "test" fingerprint into production. These should be `#[cfg(test)]` only, or live in a `test_utils` module behind a feature flag.

File: `crates/sifr_frontend/src/cache_keys.rs` (three `for_testing` definitions).

### 6. `PackageContextFingerprint` may be under-specified (MEDIUM — depends on `WorkspacePackageConfigIdentity`)

`PackageContextFingerprint::from_identity` only fingerprints `workspace_root` and `entrypoint` paths. If `WorkspacePackageConfigIdentity` carries any other config fields that affect query results (e.g., `tool.sifr` settings, package name, version constraints, custom source roots), they will not be part of the cache key. The diff only exercises the two-path variant, so this cannot be confirmed from the diff alone — but the docstring claim that "every cache key includes … package/config context fingerprint" is not currently justified by what is hashed.

File: `crates/sifr_frontend/src/cache_keys.rs` `PackageContextFingerprint::from_identity`; cross-check against the real definition of `WorkspacePackageConfigIdentity` in `workspace_session`/related module.

### 7. Circular module dependency between `cache_keys` and `source_maps` (LOW — works, but ugly)

`cache_keys.rs` uses `SourcePath` from `super` (i.e., `source_maps`), and `source_maps.rs::SourceHash::from_source_text` calls `crate::stable_source_hash` from `cache_keys`. This compiles because Rust resolves `use` lazily, but it is a layering inversion: a "leaf" type-identity helper is now reachable through the cache-key module, while the leaf module reaches back into it. Move `stable_source_hash` (or at least the FNV-1a helper) into a small `source_identity` / `fingerprint` module that both `source_maps` and `cache_keys` consume, so the dependency graph is a DAG.

File: `crates/sifr_frontend/src/lib.rs` (mod ordering), `crates/sifr_frontend/src/source_maps.rs` (`SourceHash::from_source_text`), `crates/sifr_frontend/src/cache_keys.rs` (`pub(crate) fn stable_source_hash`).

### 8. `CacheKeyContext::new` requires the caller to redundantly pass `CacheFamily` (LOW — API concern)

Every key type already implies a `CacheFamily`. Yet the supported context constructor forces callers to repeat it:

```rust
CacheKeyContext::new(CacheFamily::Parse, compiler, workspace, package);
```

followed by:

```rust
ParseCacheKey::new(source_hash, context);
```

The `ParseCacheKey::new` then sets `parser_options` to the same default. Prefer either a per-key `context_for(...)` constructor or hide the family in `CacheKeyContext::new` and let the key wrappers set their own policy field.

File: `crates/sifr_frontend/src/cache_keys.rs` `CacheKeyContext::new`, `ParseCacheKey::new`, `FormatCacheKey::new`.

### 9. `SourceHash` migration is correct, but the regression-risk note is missing (LOW — docs)

The switch from `DefaultHasher` to FNV-1a-64 is the right call — `DefaultHasher` is not stable across Rust versions, and the old code was a latent determinism bug. However:

- No comment in `source_maps.rs` flags the behavioral change for any pre-M9 persisted cache.
- The M9 doc file does not call out that pre-existing `SourceHash` values are no longer comparable across this boundary.

If anything is persisting `SourceHash` between M8 and M10 (M8 closeout introduced the flow graph fingerprints, which are also string-form), M9 invalidates them silently.

File: `crates/sifr_frontend/src/source_maps.rs` `impl SourceHash`, `internal_docs/typescript_go_architecture_transfer_m9_fingerprints_cache_keys.md`.

### 10. `QueryPolicyFingerprint` redundancy in lint/format keys (LOW)

`LintCacheKey` carries both `context.query_policy` (default for the family, set by `CacheKeyContext::new`) and `lint_policy` (also defaulted in `new`). For a caller that does not call `with_query_policy`, the context's `query_policy` and `lint_policy` are equal strings, and both are hashed — wasting a field in the fingerprint and making the meaning of "policy" ambiguous. Decide whether `query_policy` is the unified policy slot (and lint/format stop carrying their own), or whether it is package-wide and the per-key field is the family override (and the context should not default it).

File: `crates/sifr_frontend/src/cache_keys.rs` `LintCacheKey`, `FormatCacheKey`, `CacheKeyContext::new`, `CacheKeyContext::with_query_policy`.

---

## Test Gaps

- **No negative tests** (see Finding 1) — this is the only explicitly required test class in the M9 brief, and it is absent.
- No test for `SourceHash::from_source_text` cross-process stability (a single FNV-1a known-vector test would lock the contract in).
- No test for `WorkspaceContextFingerprint::project` (only `single_file` is exercised).
- No test that `from_workspace` is sensitive to `WorkspaceSessionTarget::Project` differing from two `single_file` targets with the same path.
- No test that `PackageContextFingerprint` differs between `None` and `Some` for `workspace_root` / `entrypoint` (the `optional_path_field` uses `"<none>"`, but the contract isn't pinned by a test).
- No test that `CacheFamily::default_policy` values are stable across runs (the constant strings in `cache_keys.rs` are the contract).
- No test for ordering/equality of `CacheKeyFingerprint` / `CompilerFingerprint` (both derive `PartialOrd, Ord`).
- No test that `FlowGraphCacheKey` differs when only `hir_fingerprint` changes (the test only varies `control_flow_fingerprint`).
- No fuzz / collision test (acceptable for FNV-1a 64-bit, but a property test for the length-prefix discipline would prevent regressions in the framing logic).

## Doc Gaps

- The M9 doc file does not explain the **negative-test policy**: which fields are deliberately *not* part of which keys, and why. The brief explicitly asks for those tests, and the contract that justifies them belongs in the doc.
- The doc does not describe the **contracts** for opaque inputs: `parse_fingerprint`, `hir_fingerprint`, `manifest_fingerprint`, `module_graph_fingerprint`, `control_flow_fingerprint`. Callers in M10 will need to know which fingerprint producer to consult for each slot.
- The doc does not call out the **`SourceHash` determinism change** and any pre-M9 cache invalidation risk (Finding 9).
- `internal_docs/architecture.md`, `internal_docs/frontend_cache_invalidation.md`, and `internal_docs/frontend_query_architecture.md` were updated with an M9 note, but the note is too short to convey the typed-key taxonomy or the M10 hand-off. Consider linking to a table in the M9 doc that lists each `CacheFamily` with its key type and the policy field name.
- The issue tracker row for M9 is correct ("Adds deterministic compiler/cache fingerprints and typed key identities…") but does not name the negative-test gap; readers of the issue alone won't know that requirement is at risk.

## Verdict Rationale

The taxonomy, the FNV-1a helper, and the cross-key context model are well structured and the deterministic migration of `SourceHash` is an improvement. However, two M9 requirements are not met (negative tests, and the broader "type-safety / option-capture" intent of the key-input contract), one deterministic-identity concern is live (`Debug`-formatted `WorkspaceCompilerOptions`), and several free-form strings invite cache-key collisions once M10 turns these into live lookups. **NOT SATISFIED** pending fixes for Findings 1, 2, and 4 at minimum, and ideally 3, 5, and 6.
