# TypeScript-Go Architecture Transfer: Source Provider

status: source-provider layer implementation review

source-provider layer introduces the first compiler-service filesystem boundary before the
workspace/session layer exists. The new boundary is intentionally small:
callers can read files, enumerate directories, probe files/directories, and
canonicalize paths through `sifr_frontend::SourceProvider`.

## Provider Model

`sifr_frontend` now exposes:

- `SourceProvider`: typed source filesystem interface for semantic reads,
  directory reads, path probes, and canonicalization.
- `DiskSourceProvider`: production disk-backed implementation.
- `OverlaySourceProvider`: read-through provider that can substitute open
  editor buffer text for disk files without mutating disk state.
- `TrackingSourceProvider`: wrapper that records successful reads, path probes,
  canonicalization, and failed lookups as `SourceDependency` records.
- `OverlayDocument`: overlay metadata with URI, path, document version, source
  text, source hash, and disk-match state.

The overlay record stores `SourceText`, so the source text/line-map authority
continues to flow through the source-provider layer `sifr_source` model instead of reintroducing a
second line-map implementation.

## Migrated Reads

The following semantic read paths now use the provider boundary:

- `FrontendContext::load_project_with_provider` reads project entrypoints,
  project directories, and project modules through a supplied provider.
- `FrontendContext::load_project_tracked` returns the dependency records
  captured during project loading.
- Driver module resolution, workspace manifest discovery, project file
  discovery, and package import closure materialization use provider-backed
  reads and probes.
- Package manifest loading, source-root validation, source-map traversal, and
  namespace API extraction accept provider-backed reads.
- Formatter and linter source/config reads use short-lived disk providers
  rather than separate direct source reads.
- Package CLI/session discovery and target selection use provider-backed
  manifest/source-root probes where those probes affect package selection.

The source-provider layer does not consume dependency records for invalidation yet.
Workspace session state, event compaction, dirty-scope classification, and precise
invalidation own that behavior.

## Package Import Ambiguity

`PackageSourceMap` now separates fatal source-map construction failures from
queryable import ambiguity:

- valid duplicate module candidates are retained in `ambiguous_modules`
  instead of turning source-map construction into a package-fatal error;
- `PackageImportResolutionResult` distinguishes `Resolved`, `Ambiguous`,
  `Unresolved`, `PrivateAccess`, and `FatalPackageMapFailure`;
- package import closure diagnostics can map ambiguity back to the import site
  with `SIFR-IMPORT-0005` and candidate paths;
- legacy `resolve_import` remains available and maps ambiguity into a package
  diagnostic for existing non-source-callers.

End-to-end package runtime fixtures remain outside this source-provider layer.
Package unit tests prove the provider state model and import-resolution branch
coverage.

## Direct-Read Exceptions After source-provider layer

The source-provider layer guardrail scanner remains active. After source-provider layer, remaining production direct
filesystem sites are documented exceptions rather than semantic source reads:

- build artifact metadata/cache checks in `crates/sifr_driver/src/build`;
- formatter artifact-cache existence probing in `crates/sifr/src`;
- package projection and repair-state effects in `crates/sifr_package`;
- package lock/source-layout checks that remain package-management or generated
  output validation until package-aware snapshots promote them.

source-provider layer owns persistent build metadata and any future `.sifrbuildinfo` boundary.

## Validation

source-provider layer focused validation:

- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py`
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py --self-test`
- `cargo fmt --check`
- `python3 scripts/check_file_size_guardrails.py`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo test -p sifr_driver -p sifr_package -p sifr_frontend -p sifr_format -p sifr_lint`
- `cargo clippy --workspace -- -D warnings`

The full create-pr validation gate remains required before PR acceptance.
