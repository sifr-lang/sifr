## Verdict: SATISFIED

No new blockers were introduced. All pass-6 follow-up recommendations are implemented, and the cache key module is in good shape.

## Findings

### Pass-6 follow-up resolution (all addressed)

- `crates/sifr_frontend/src/cache_keys.rs:633-637` — `ParseCacheKey::with_parser_options` is constructed directly inside `parse_key_includes_source_compiler_workspace_package_and_policy` and the resulting fingerprint is asserted distinct. This closes the "only `new` is exercised" gap.
- `crates/sifr_frontend/src/cache_keys.rs:797-827` — `document_identity_inputs_are_intentionally_omitted_from_content_keys` now defines a parallel `diagnostics_key_for_document` helper and asserts `diagnostics_first.fingerprint() == diagnostics_second.fingerprint()`. Document identity omission is now pinned for two families.
- `crates/sifr_frontend/src/cache_keys.rs:403` — `FormatCacheKey::new` initializes `formatter_options` with the module-level `FORMAT_OPTIONS_VERSION` constant rather than an inline literal, matching the parser/lowering pattern.
- `crates/sifr_frontend/src/cache_keys.rs:73` and `:516-522` — `WorkspaceContextFingerprint::single_file` and `compiler_options_fingerprint` both funnel through `frontend_mode_label(FrontendMode)`, guaranteeing `single-file` / `project-entrypoint` labels are reused consistently across contexts and per-family options.
- `crates/sifr_frontend/src/cache_keys.rs:875-887` — `project_workspace_context_is_path_sensitive` exercises `WorkspaceContextFingerprint::project` for the first time.
- `crates/sifr_frontend/src/cache_keys.rs:851-873` — `package_identity_optional_paths_are_distinct_and_exhaustive` covers the `None` / root-only / entrypoint-only / both-present quadrants, which is the matrix that the optional-path serialization in `optional_path_field` (`:614-621`) actually produces.
- `internal_docs/typescript_go_architecture_transfer_m9_fingerprints_cache_keys.md:60-63` — doc explicitly names `with_parser_options` and the separate `formatter_options` fingerprint as override surfaces for M10.
- `internal_docs/typescript_go_architecture_transfer_m9_fingerprints_cache_keys.md:95-97` and `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:125` — quick-validation wall time and advisories are updated to 338.91s with `warm wall-time budget exceeded; group skew is high`, and the doc records the final run.

### Code-quality checks (no new issues)

- FNV-1a 64-bit implementation in `FingerprintBuilder` (`:679-707`) is self-contained, deterministic, and uses the standard FNV offset basis `0xcbf2_9ce4_8422_2325` and prime `0x0000_0100_0000_01b3`. Field framing is length-delimited with `0xff` terminator — collision-resistant for the structured inputs used.
- `key_builder` (`:502-514`) standardizes the common five fields, so every family fingerprint sees identical `source_hash / compiler / workspace / package / query_policy` ordering. This matters for cache stability.
- All public constructors are `#[must_use]` and `String`-backed, so the module is `Send + Sync` without further work.
- `SourceMapCacheKey`, `LintCacheKey`, `FlowGraphCacheKey`, etc. all carry an explicit family-policy fingerprint as a field rather than relying solely on `context.query_policy`, which is what allows M10 to override user-configured policy without changing the key shape.

## Test / doc gaps (non-blocking)

1. The document-identity-omission test covers `ParseCacheKey` and `DiagnosticsCacheKey`, but not the remaining seven families. Since all of them take `source_hash` as the only content-derived input and the source hash itself is asserted content-sensitive (`source_hash_is_deterministic_and_content_sensitive`), the omission is structurally guaranteed — but a single test asserting equality across all nine key types under varying `DocumentVersion`/`FileId`/`URI` would make the guarantee explicit and prevent a future regression where someone adds, e.g., a `URI` field to `FormatCacheKey`. Low priority because the structural argument is strong.

2. `compiler_fingerprint_is_deterministic` (`:572-577`) confirms two calls return the same value, but there is no test asserting that `CompilerFingerprint::current()` differs from a hash of the same fields built via the underlying `FingerprintBuilder`. A round-trip test would harden the contract that `CompilerFingerprint::current()` is a pure function of its declared inputs.

3. `CacheKeyContext::with_query_policy` is not directly exercised; it is only reached indirectly through `parse_key_includes_source_compiler_workspace_package_and_policy` (which mutates `context.query_policy` directly). A small dedicated test would document the builder-style API.

4. `compiler_options_fingerprint` (`:516-522`) is exercised by `hir_key_includes_parse_fingerprint_and_compiler_options` switching `FrontendMode`, but there is no test that explicitly asserts the helper is also reused by `WorkspaceContextFingerprint::single_file` (i.e., that both produce the same label for the same `FrontendMode`). A one-line assertion would lock in the "shared typed labels" property called out in the pass-6 follow-up.

5. The M9 doc notes that pre-M9 `SourceHash` strings are not comparable to M9 hashes, but `SourceHash::from_source_text`'s migration from `DefaultHasher` to FNV-1a is mentioned in prose only. A brief sentence in the validation section listing a one-shot migration test (e.g., a `#[test]` that pins the new hash of a known input as a golden string) would make the schema-version bump auditable from the doc.

None of these rise to blocker level; the module passes its targeted `cache_key` suite, the workspace-level `cargo test` and `cargo clippy -D warnings` runs, and the M9 quick-validation lane. Ready to merge.
