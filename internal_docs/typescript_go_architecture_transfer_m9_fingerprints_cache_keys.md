# TypeScript-Go Architecture Transfer M9 Fingerprints And Cache Keys

status: M9 implementation review

M9 makes cache identity explicit before Sifr starts reusing compiler-service
entries across snapshots. It does not introduce parse, HIR, diagnostic, or index
reuse; M10 owns reuse and reference-counted cache storage.

## Fingerprint Model

`sifr_frontend::CompilerFingerprint` records the cache-key schema, frontend
crate version, parser version, lowering policy, source-map algorithm, diagnostic
policy, lint policy, format policy, package-graph policy, symbol-bucket policy,
and flow-graph policy.

`CacheKeyFingerprint` values are deterministic process-independent FNV-1a
fingerprints built from length-delimited fields. They are compiler cache
identities, not security hashes.

`SourceHash::from_source_text` now uses the same deterministic fingerprint
helper instead of `DefaultHasher`, so source identity is stable across
processes and public cache-key constructors can be built without internal
frontend access.
Pre-M9 `SourceHash` strings are intentionally not comparable to M9 hashes; no
snapshot cache reuse exists yet, so M10 starts from this identity contract.

## Common Inputs

Every cache key includes:

- source content hash or manifest/source hash for package graph keys;
- compiler fingerprint;
- workspace context fingerprint;
- package/config context fingerprint;
- query-policy fingerprint for cache families where policy affects results.

`CacheKeyContext::from_workspace` derives the common compiler, workspace, and
package fingerprints from `WorkspaceSessionTarget` and
`WorkspacePackageConfigIdentity`.

## Key Families

M9 defines typed keys for:

- parse;
- source-map;
- HIR/lowering;
- diagnostics;
- lint;
- format;
- package graph;
- symbol buckets;
- flow graph.

Family-specific inputs include parser options, line-map algorithm, parse/HIR
fingerprints, compiler options, diagnostic rendering style, lint/format policy,
formatter options, manifest fingerprint, module graph fingerprint, typed bucket
scope, and control-flow fingerprint where relevant.

Default constructors use the current M9 policy constants. `ParseCacheKey` also
exposes `with_parser_options`, and `FormatCacheKey` carries a separate
`formatter_options` fingerprint so M10 can key user-configured parser and
formatter behavior without changing the key shape.

## Intentionally Omitted Inputs

Content-derived keys intentionally omit transient editor identity:

- document version;
- file id;
- URI/display path.

Those values affect stale-result publication and editor routing, but they do not
change parse, source-map, HIR, diagnostic, lint, format, symbol, or flow results
when source content, workspace/package context, compiler fingerprint, and query
policy are unchanged. M9 tests pin this omission before M10 introduces reuse.

## Validation

M9 focused validation so far:

- `cargo fmt --check`
- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 verification/areas/package_management/tools/check_package_manager_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_m1_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_m1_guardrails.py --self-test`
- `cargo test -p sifr_frontend cache_key -- --nocapture`
- `cargo test -p sifr_frontend`
- `cargo test -p sifr_analysis -p sifr_lsp`
- `cargo test -p sifr_driver`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy -p sifr_frontend -- -D warnings`
- `cargo clippy --workspace -- -D warnings`
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report
  `target/validation_lane_reports/create-pr.latest.json`, wall time 338.91s,
  advisories: warm wall-time budget exceeded; group skew is high
- Claude reviewer pass 7 -> SATISFIED
