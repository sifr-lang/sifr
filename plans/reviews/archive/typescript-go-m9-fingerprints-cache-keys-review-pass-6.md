# M9 Fingerprints And Cache Keys — Pass 6 Review

## Verdict: **SATISFIED**

All pass-5 specific fixes have been applied correctly. M9 scope is fulfilled: no cache-reuse behavior is introduced, identity is exhaustive, and the documented boundary is honored. A few small findings and test/doc gaps remain.

---

## Findings

### 1. `compiler_options_fingerprint` exhaustive destructuring (cache_keys.rs, `fn compiler_options_fingerprint`)
```rust
let WorkspaceCompilerOptions { mode } = options;
```
This is the right pattern (exhaustive, forces a compile error on struct growth), but the body only fingerprints `mode`. If `WorkspaceCompilerOptions` adds even a derived field later, the destructure will force the maintainer to think about it — which is desired — but a brief `// add field to fingerprint here` guard comment would make the intent explicit. **Minor.**

### 2. Mode label inconsistency in workspace context (cache_keys.rs, `WorkspaceContextFingerprint::single_file`)
```rust
builder.field("mode", format!("{mode:?}"));
```
The same `FrontendMode` enum is now labeled by a typed helper (`frontend_mode_label`) in `compiler_options_fingerprint`, but `WorkspaceContextFingerprint::single_file` still Debug-formats it. The Debug output (`"SingleFile"` / `"ProjectEntrypoint"`) is stable, so this isn't a correctness bug, but it is an inconsistency vs. the new typed-label pattern introduced by this pass. **Minor — likely acceptable since the labels are stable and the field is consumed in the same `FingerprintBuilder` flow.**

### 3. Magic default string in `FormatCacheKey::new` (cache_keys.rs)
```rust
formatter_options: QueryPolicyFingerprint::new("default-format-options"),
```
All other families use a module-level constant for their default policy/version (`PARSER_OPTIONS_VERSION`, `LOWERING_OPTIONS_VERSION`, etc.). `FormatCacheKey` is the only family that hard-codes its default. There is no `FORMAT_OPTIONS_VERSION` constant. Either introduce a constant or accept the magic string with a `// M9 sentinel` comment. **Minor consistency issue.**

### 4. Duplicate `new file` diff entry for `cache_keys.rs`
The diff contains both a modification hunk and a `new file mode 100644` hunk for `crates/sifr_frontend/src/cache_keys.rs` with identical content. This looks like a diff generation artifact (perhaps the file was re-added during a rebase or the diff was produced with a wrong option). The final file content is correct, but reviewers and patch-apply tools will be confused. **Verify the diff generation path; not a content issue.**

### 5. `WorkspaceContextFingerprint::project` not directly tested
Not a regression in this pass — but worth noting: only `single_file` is exercised in tests, while `project` is only exercised indirectly via `CacheKeyContext::from_workspace` (and even that path uses `SingleFile` in the existing test). Adding a dedicated `project` path-sensitivity test would be a low-cost coverage improvement. **Pre-existing gap.**

---

## Test Gaps

### T1. `ParseCacheKey::with_parser_options` is not directly tested (cache_keys.rs, tests module)
The new override surface is added but no test exercises it. The existing `parse_key_includes_source_compiler_workspace_package_and_policy` only verifies the default policy path. A test like:
```rust
let custom = ParseCacheKey::with_parser_options(
    source("x = 1"),
    QueryPolicyFingerprint::new("experimental-parser-v2"),
    context(CacheFamily::Parse),
);
let default = ParseCacheKey::new(source("x = 1"), context(CacheFamily::Parse));
assert_ne!(custom.fingerprint(), default.fingerprint());
```
would close the loop. **Should add.**

### T2. `document_identity_inputs_are_intentionally_omitted_from_content_keys` only covers `ParseCacheKey`
The negative test is well-formed, but only one of the nine key families is exercised. Given the M9 spec explicitly enumerates parse, source-map, HIR, diagnostics, lint, format, symbol buckets, and flow graph as "content-derived", the omission contract should be pinned for at least one more family (HIR or diagnostics is the natural second target). **Should extend or add a sibling test.**

### T3. `FormatCacheKey::new` defaults are not asserted (cache_keys.rs, tests module)
`FormatCacheKey::new` sets `formatter_options = "default-format-options"`, but no test asserts the default value or that the constructor wires the right `formatter_policy`. The `formatter_options` field is mutated in `format_options_changed`, but the "default" path is not pinned. **Minor — low priority.**

### T4. `package_identity_none_and_some_are_distinct_and_exhaustive` name oversells coverage (cache_keys.rs, tests module)
The test compares `(None, None)` vs `(Some, None)`. It does not exercise `(None, Some)` or the `(Some, Some) → (Some, Some with different entrypoint)` case explicitly — the latter is in `workspace_and_package_contexts_are_path_sensitive`, but the `None, Some` arm is uncovered. Either rename the test or add the missing case. **Minor.**

---

## Doc Gaps

### D1. MD validation section still cites "agent reviewer pass 4" (internal_docs/typescript_go_architecture_transfer_m9_fingerprints_cache_keys.md)
The `## Validation` block ends with:
> agent reviewer pass 4 -> SATISFIED (`reviews/typescript-go-m9-fingerprints-cache-keys-review-pass-4.md`)

This is pass 6. Either the validation list should be updated to reference the current pass (and any newly run validation commands), or the pass-N line should be moved out of `## Validation` into a separate `## Review History` section so it can grow without polluting the "what we ran" list. **Update before merge.**

### D2. MD doesn't mention the new override surfaces by name (internal_docs/typescript_go_architecture_transfer_m9_fingerprints_cache_keys.md)
The "Family-specific inputs" line now lists "formatter options", but the existence of `ParseCacheKey::with_parser_options` (a public override constructor distinct from the default `new`) and the separate `formatter_options` slot on `FormatCacheKey` is not called out. A short paragraph or bullet explaining that M9 keys expose default-and-override construction (no live reuse) would make the override surfaces easier to discover for M10. **Should add.**

### D3. MD doesn't cross-reference the SourceHash boundary with `SourceHash::from_source_text` behavior
The new paragraph states the boundary correctly ("Pre-M9 `SourceHash` strings are intentionally not comparable to M9 hashes"), but does not name the function whose behavior changed. A one-line mention of `SourceHash::from_source_text` switching off `DefaultHasher` (which the section already alludes to) would let a future reader find the diff in the public API. **Minor wording fix.**

---

## Pass-5 Fix Audit

| Pass-5 fix | Status | Evidence |
|---|---|---|
| Negative test for omitted document identity | ✅ Applied | `document_identity_inputs_are_intentionally_omitted_from_content_keys` (cache_keys.rs tests) |
| Diagnostics style → typed enum | ✅ Applied | `diagnostic_style: FrontendDiagnosticStyle` + `diagnostic_style_label` |
| Symbol bucket scope → typed enum | ✅ Applied | `SymbolBucketScope` enum + `.label()` |
| Parser option override surface | ✅ Applied | `ParseCacheKey::with_parser_options` |
| Formatter option override surface | ✅ Applied | `FormatCacheKey.formatter_options` field |
| Stop Debug-formatting `WorkspaceCompilerOptions` | ✅ Applied | `compiler_options_fingerprint` helper |
| `#[cfg(test)]` on testing constructors | ✅ Applied | `for_testing` is gated on `#[cfg(test)]` for all three types |
| Exhaustive `WorkspacePackageConfigIdentity` destructure | ✅ Applied | `let WorkspacePackageConfigIdentity { workspace_root, entrypoint } = identity;` |
| Document omitted inputs | ✅ Applied | New "Intentionally Omitted Inputs" section |
| Document `SourceHash` boundary | ✅ Applied | New paragraph under "Fingerprint Model" |

All ten pass-5 fixes are addressed. M10 reuse still does not exist (no map, no LRU, no eviction, no entry construction). M9 contract is correctly scoped.

---

## Recommendation

**Accept for merge** with the following non-blocking follow-ups:

1. Add a direct test for `ParseCacheKey::with_parser_options` (T1).
2. Extend or duplicate the document-identity-omitted test for at least one non-parse family (T2).
3. Update the MD validation section to reference the current pass review (D1) and call out the override constructors (D2).
4. Promote the magic `"default-format-options"` string in `FormatCacheKey::new` to a module-level constant for parity with other families (Finding 3).
