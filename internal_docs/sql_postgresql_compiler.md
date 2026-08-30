# PostgreSQL schema and query compiler

This document defines the offline PostgreSQL compiler component.

The component owns PostgreSQL syntax, schema normalization, semantic analysis,
and compiler diagnostics. It does not open a database connection during normal
compilation.

## Artifact boundary

The workspace package is `sifr_sql_postgresql`. It produces one compiler
component artifact for each supported PostgreSQL major.

The package contains these internal layers:

- `ffi` is the only unsafe Rust layer. It copies and frees `libpg_query` data.
- `raw_adapter` converts parser JSON into provider-owned syntax nodes.
- `catalog` indexes the canonical `SchemaIR` authority.
- `analysis` and `writes` produce closed provider facts.
- `component` converts those facts into the compiler component protocol.

No upstream parser node crosses the package boundary. Public Sifr APIs do not
expose Rust parser, catalog, or driver types.

## Selected tooling

The component uses `libpg_query` for strict PostgreSQL parsing. It embeds one
exact source commit for each supported PostgreSQL major.

| PostgreSQL | Parser tag | Source commit |
| ---: | --- | --- |
| 13 | `13-2.2.0` | `1097b2c33e54a37c0d2c0f2d498c7d1cf967eae9` |
| 14 | `14-3.0.0` | `397cbb9c1b188b8a5c6e1a9633461b2d01903abc` |
| 15 | `15-4.2.4` | `db39825bc7c1ddd45962ec6a626d740b7f8f027a` |
| 16 | `16-5.2.0` | `fce106abf41205e5d0db47bea7de44ad1e36f7a5` |
| 17 | `17-6.2.2` | `7be1aed1f1f968a36cf541319f71e845850f0381` |
| 18 | `18.0.0` | `204fbdbd3ed5f8691ab358e49f1fc5397b4679e2` |

`component-sources.json` records each tag, commit, deterministic tracked-content
checksum, and source path. Qualification hashes each tracked path and file
content. The result does not depend on a Git archive implementation.

The build uses `cc` `1.4.4`, `wit-bindgen` `0.61.1`, WASI SDK `33`, and
WASI-Virt `0.2.0`. The workspace and qualification records pin these latest
stable tools. WASI SDK provides the C headers and Clang target for the checked
`wasm32-wasip2` build. WASI-Virt is pinned at source commit
`448f6df8f688cee5d6995e96b1ffc31f9bf00742` with a tracked-content checksum.

`build_postgresql_components.py` builds all six component artifacts. Each guest
exports the `embedded-language-provider.analyze` WIT function. It reads the
canonical `SchemaIR` context artifact, reconstructs `$n` holes, runs the same
owned parser and semantic analyzer as native qualification, and returns the
closed compiler protocol response. Component registration reads the artifact
and derives its SHA-256; callers cannot supply an unrelated hash.

`component-artifacts.json` is the artifact authority. It records the exact
parser commit, output size, SHA-256, WIT world, protocol, target, and toolchain
for each server major. It also records one deterministic digest of the Cargo
lock, manifests, Rust sources, WIT files, and WASI compatibility sources that
form the guest. Qualification rejects a stale artifact when any input changes.
The build script replaces this manifest only after all six artifacts build
successfully.

The component build requires the Rust `wasm32-wasip2` target and this exact
environment:

```bash
git submodule update --init third_party/wasi-virt
export WASI_SDK_PATH=/absolute/path/to/wasi-sdk-33.0
python3 verification/areas/sql_platform/tools/build_postgresql_components.py
```

The WASI compatibility layer supplies only facilities required by extracted
PostgreSQL parser code. It uses compiler atomics for spinlocks. It uses WASI
SDK libraries for signals, non-local jumps, memory mapping, process identity,
and process clocks. It excludes server, socket, and postmaster units that the
parser archive does not use. WASI-Virt then composes deny-by-default
implementations of every remaining WASI interface into the artifact. The
finished component has no host imports, so the compiler host still grants no
WASI capability. Any unexpected environment, file, network, clock, random,
process, or standard-I/O access traps inside the component.

These tools are not Sifr standard-library modules. `cc` and `wit-bindgen` are
pinned workspace dependencies. WASI SDK and WASI-Virt are private build tools.
`libpg_query` and WASI-Virt are exact checked-in submodule sources. None of
these names enters the Sifr user API.

## Parser selection

`SIFR_POSTGRESQL_MAJOR` selects one parser source during the component build.
The default selection is PostgreSQL 18.

The build follows the selected upstream build layout, with the documented
WASI-only exclusions. It does not link a system PostgreSQL library.

The adapter checks the version number in every returned parse tree. A version
mismatch is a provider diagnostic.

## Provider syntax

The provider owns serializable nodes for queries, writes, expressions, joins,
set operations, and supported DDL.

The strict adapter accepts the supported node set only. An unknown raw node
fails with `SIFR-SQL-POSTGRESQL-0011`.

Advanced PostgreSQL syntax extends this node model. It does not add a second
parser or a second semantic authority.

## Schema authority

DDL and catalog snapshots both normalize into `SchemaIR`. Query analysis reads
only that normalized graph.

The DDL normalizer emits these core object kinds:

- namespaces, tables, columns, and sequences
- primary, unique, foreign-key, and check constraints
- indexes, views, and materialized views
- enums, domains, and functions

Catalog snapshots can also provide composite types, operators, casts, and other
provider metadata. Each object retains its schema source location.

Relations keep their ordered columns, required values, defaults, generated
flags, primary key, and unique sets. Constraint objects keep exact dependency
edges for schema slicing.

The schema fingerprint covers every semantic object. Source paths and offsets do
not change that fingerprint.

## Name and scope resolution

Qualified names resolve by exact object identity. Unqualified relation names
prefer the `public` schema, then require one unique suffix match.

Each query block has one scope frame. Relation aliases replace textual relation
names inside that frame.

Lateral subqueries can read earlier bindings in the current frame. Correlated
subqueries can read outer frames.

Output aliases are visible to PostgreSQL `ORDER BY` and `GROUP BY`. They are not
visible to `WHERE` or `HAVING`.

An ambiguous column is an error. An unknown qualified column includes the
related schema declaration span.

## Types and codecs

The provider maps PostgreSQL types into closed `DatabaseType` values. Each
supported value has one exact codec identity for the selected server profile.

The core registry covers Boolean, fixed integers, numeric values, floats, text,
binary values, temporal values, UUID, JSON, network values, and MAC addresses.
`inet` and `cidr` keep different codec identities. Character and varying
character types use `DatabaseType::Named`. This type preserves the qualified
database identity and all modifiers while mapping the value to Sifr `str`.

Enums, domains, and composites use stable schema object identities. Catalog
metadata can extend functions, operators, and casts without exposing raw server
identifiers.

Every `$n` parameter must receive one exact database type. Repeated uses must
produce the same type constraint.

Slots must form the contiguous sequence `$1`, `$2`, and so on. Provider analysis
returns zero-based slots and exact codec identities.

## Fragment placeholders

PostgreSQL canonical syntax uses one-based `$n` placeholders. Fragment metadata
uses the matching zero-based slot order.

Composition rewrites each inserted fragment by the number of earlier parameters.
The lexical rewriter ignores quoted strings, quoted identifiers, dollar strings,
line comments, and nested block comments.

The rewriter rejects `$0` and slot overflow. Provider parsing validates the
composed result after rewriting.

## Query semantics

The analyzer resolves casts, operators, scalar functions, aggregates, aliases,
correlations, and set operations against the catalog.

Built-in rules cover common arithmetic, modulo, comparisons, `LIKE`, `IN`, text
concatenation, `count`, `sum`, `avg`, `min`, `max`, `lower`, `upper`, and `now`.
Catalog metadata supplies other overloads.

Result fields must have unique names. Unnamed expressions receive deterministic
`column_N` names.

Nullability is conservative. Columns preserve catalog nullability, strict
operators propagate nullable inputs, and unknown function results remain
nullable unless metadata proves otherwise.

Core cardinality recognizes aggregate rows and an exact `LIMIT 1`. Later
semantic rules extend the same cardinality lattice.

The effect contract lists every referenced and affected schema object. Embedded
plans carry those objects as schema-fingerprinted dependencies.

## Write semantics

`INSERT` checks required columns, row widths, assignment casts, generated
columns, and `INSERT SELECT` widths.

`ON CONFLICT` targets must match a primary or unique key. Conflict-target and
update predicates remain distinct. The `excluded` pseudo-relation uses the
target table column types. SQL `NULL` cannot be assigned to a non-nullable
column.

`UPDATE` checks each assignment once and includes `FROM` relations in scope.
`DELETE` includes `USING` relations in scope.

All writes infer `RETURNING` fields through the normal result analyzer. Their
effect contract marks the target relation as affected.

## Diagnostics

Provider diagnostics use the stable namespace `SIFR-SQL-POSTGRESQL-0001`
through `SIFR-SQL-POSTGRESQL-0011`.

The primary span points into the virtual SQL document. Related spans point to
the Sifr template and relevant schema declarations.

Messages do not include credentials, raw FFI memory, or server connection data.
Parser failures copy the upstream message before the FFI result is freed.

## Unsafe syntax authority

The package manifest can grant `sql.unsafe-syntax` through
`trust.security-capabilities`.

`sifr_package` resolves that grant for one exact root package identity in the
resolved package graph. A connected dependency cannot present itself as the
root and consume its capability. `UnsafeSyntaxGrant` consumes the resolver
directly. The production fragment compiler accepts no caller-constructed
grant.

An absent capability, empty audit reason, wrong package identity, or `deny`
lint policy rejects the fragment.

## Verification

The `postgresql-compiler` SQL suite owns this component.

It validates source tags, commits, tracked-content checksums, artifact hashes,
provider registrations, schema objects, semantics, spans, placeholders, and
exact codecs. Snapshot tests preserve provider analysis and diagnostic shape.

The parser matrix rebuilds and runs the same Rust tests for PostgreSQL 13
through 18. This proves that each embedded parser crosses the owned adapter.

The live matrix starts each supported PostgreSQL server. It compares version,
parameter types, result types, nullability, writes, and diagnostic SQLSTATEs.
The checked provider regression also replays those facts through the offline
analyzer and compares its parameter, result, nullability, write, and diagnostic
answers.

The checked-in live evidence records exact server image digests and observed
results. Ordinary offline gates validate that evidence without opening a
database connection. The live differential suite is registered in the SQL
manifest and nightly profile. The evidence cannot remain an unexecuted side
record.
