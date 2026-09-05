# TypeScript-Go Architecture Transfer: Source Provider

The source-provider boundary is the only semantic source access boundary in the
compiler. A source operation must use one active `SourceProvider`. The operation
must not create a new disk provider in a lower compiler layer.

## Provider Model

`sifr_frontend` exposes these provider types:

- `SourceProvider` supplies file reads, directory reads, path probes, and path
  canonicalization.
- `DiskSourceProvider` is the canonical provider for disk access.
- `OverlaySourceProvider` replaces selected disk files with editor text. It does
  not change disk state.
- `TrackingSourceProvider` records reads, probes, canonicalization, and failed
  lookups as `SourceDependency` values.
- `OverlayDocument` contains the URI, path, document version, source text,
  source hash, and disk-match state.

The provider returns `SourceText`. Thus, the `sifr_source` text and line-map
model remains the single source-text authority.

## Composition Roots

Only a composition root creates `DiskSourceProvider`. The approved roots are:

- a `sifr` CLI command;
- an LSP request or LSP session operation;
- `WorkspaceSession` construction;
- a standalone compiler binary or benchmark.

The composition root passes the same provider through the full source
operation. Frontend, driver, formatter, linter, and package APIs receive that
provider. They do not provide a second API that creates a disk provider.

Tests create an explicit provider and call the production API. Tests do not use
a disk-backed compatibility wrapper.

## Provider-Backed Operations

The following operations use the active provider:

- frontend project loading and overlay loading;
- driver workspace discovery, module resolution, project discovery, package
  discovery, and test discovery;
- formatter configuration, file discovery, source reads, checks, and writes;
- linter configuration, file discovery, source reads, and fix planning;
- package manifest loading, source-root validation, graph derivation, source-map
  construction, offline validation, projection checks, and session discovery;
- CLI and LSP operations that compose these lower-layer operations.

`PackageSession` captures manifest and application-target discovery results.
Later target queries use the captured session state. They do not repeat disk
discovery through a hidden provider.

When the active provider is a `TrackingSourceProvider`, each semantic read or
probe records a dependency. When it contains an overlay, all lower layers see
the same overlay state. This rule makes overlay and snapshot results
deterministic.

## Package Import Results

`PackageSourceMap::resolve_import_result` is the only package import query API.
It returns one of these states:

- `Resolved`;
- `Ambiguous`;
- `Unresolved`;
- `PrivateAccess`;
- `FatalPackageMapFailure`.

Callers must handle the applicable states. They must not convert the result to
`Result` or `Option` before they select the diagnostic for the source location.
Package tests call this production API and match its variants directly.

## Dependency-Direction Guard

`scripts/check_source_crate_dependency_direction.py` rejects
`DiskSourceProvider` construction in lower compiler layers. It permits the
`WorkspaceSession` composition root and standalone binaries. It ignores test
modules and test source files.

The guard has a mutation self-test. The self-test inserts a disk-provider
construction in `sifr_package` and requires the guard to reject it.

## Direct Filesystem Access

The provider boundary controls semantic source access. Direct filesystem access
can remain for these non-semantic operations:

- generated build artifacts and cache metadata;
- Cargo execution and package archive materialization;
- formatter writes and formatter cache artifacts;
- package projection writes and repair-state changes;
- temporary test and benchmark setup.

A direct filesystem read must move behind `SourceProvider` if its result can
change parsing, lowering, type checking, import resolution, formatting, linting,
package selection, or editor analysis.
