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

`component-sources.json` records each tag, commit, tree archive checksum, and
source path. Qualification recalculates each checksum from the checked-in git
object.

The build uses `cc` `1.4.4`. The workspace and dependency qualification records
pin this build-only adapter to its latest stable release.

These tools are not Sifr standard-library modules. They remain private
implementation dependencies of the provider component.

## Parser selection

`SIFR_POSTGRESQL_MAJOR` selects one parser source during the component build.
The default selection is PostgreSQL 18.

The build compiles only files listed by the selected upstream build layout. It
does not link a system PostgreSQL library.

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

Built-in rules cover common arithmetic, comparisons, text concatenation,
`count`, `lower`, `upper`, and `now`. Catalog metadata supplies other overloads.

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

`ON CONFLICT` targets must match a primary or unique key. The `excluded`
pseudo-relation uses the target table column types.

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

`sifr_package` resolves that grant for one exact package identity.
`UnsafeSyntaxGrant` consumes the resolver directly.

An absent capability, empty audit reason, wrong package identity, or `deny`
lint policy rejects the fragment.

## Verification

The `postgresql-compiler` SQL suite owns this component.

It validates source tags, commits, archive checksums, provider registrations,
schema objects, semantics, spans, placeholders, and exact codecs.

The parser matrix rebuilds and runs the same Rust tests for PostgreSQL 13
through 18. This proves that each embedded parser crosses the owned adapter.

The live matrix starts each supported PostgreSQL server. It compares version,
parameter types, result types, nullability, writes, and diagnostic SQLSTATEs.

The checked-in live evidence records exact server image digests and observed
results. Ordinary offline gates validate that evidence without opening a
database connection.
