# SQL schema profiles and canonical schema IR

This document defines the implemented compile-time schema authority. The main
SQL architecture remains authoritative. This document records the concrete
schema-profile boundaries and artifacts.

## Ownership

`sifr_package` parses `[sql.profiles.<name>]`. It resolves the `provider` field
through the direct Cargo-backed Sifr dependency scope. Resolution produces one
exact package identity, package version, package source, package-graph digest,
and set of compiler-component SHA-256 values.

`sifr_sql_contract` owns the provider-neutral schema graph. A dialect component
maps its SQL DDL, provider metadata, or generated definitions to this graph. The
core compiler does not parse dialect DDL and does not connect to a database.
The contract serializes the normalized graph as a fingerprinted generic context
artifact in the compiler-component request. The component host stays independent
of SQL.

`sifr_structural_identity` owns nominal profile identity. The identity uses only
the owning package identity and profile name. Therefore, two profiles do not
become the same type when their schema contents are equal.

## Profile configuration

Each profile must declare these fields:

```toml
[sql.profiles.app]
provider = "sifr-sql-postgresql"
source = "db/schema.postgresql.sql"
source-kind = "sql-ddl"
server-version = "18"
search-path = ["app", "public"]
extensions = ["citext"]
pooling = "session"
schema-evidence = "migration-head"
schema-strictness = "compatible"

[sql.profiles.app.session]
time-zone = "UTC"
```

`source` accepts one relative path or a non-empty list of relative paths. The
package loader requires every path to be a package-local file. It sorts and
deduplicates paths. `source-kind` is `sql-ddl`, `provider-metadata`, or
`generated-definitions`. It defaults to `sql-ddl` for the common case.

The parser rejects unknown profile and session fields. Session configuration has
no arbitrary option map. Thus, a connection URL,
password, credential environment variable, or live-introspection option cannot
enter normal compilation. Schema lifecycle tools can use credentials in their
separate host-only execution path. They write checked-in schema artifacts for
later compilation.

`schema-evidence` is `introspection`, `migration-head`, or `signed-manifest`.
`schema-strictness` is `exact` or `compatible`. Signed manifests require at
least one accepted signer. Transaction pooling rejects a persistent session
role because it cannot guarantee that state across acquisitions.

## Canonical `SchemaIR`

`SchemaIR` is an immutable ordered graph. It contains:

- the exact provider and dialect identities;
- every schema object under one canonical qualified `ObjectId`;
- a closed object-kind value;
- provider-normalized semantic properties;
- object-to-object dependencies; and
- an optional source location for diagnostics.

The object-kind set covers catalogs, namespaces, tables, columns, constraints,
indexes, sequences, identities, views, materialized views, enums, domains,
composites, arrays, ranges, functions, operators, casts, collations, character
sets, extensions, triggers, server capabilities, and dialect metadata.

Semantic values are closed deterministic values. They can contain booleans,
signed or unsigned integers, text, hexadecimal bytes, ordered lists, ordered
sets, and ordered maps. Providers use a list only when order changes meaning.
They use a set when input order is irrelevant.

Normalization rejects duplicate object identities, invalid qualified names,
missing dependency targets, malformed source spans, invalid hexadecimal values,
and incomplete provider or dialect identities. All three schema-source forms
enter the same normalization path.

## Fingerprints and diffs

The schema fingerprint includes the format version, exact provider, complete
dialect profile, object identities, object kinds, semantic properties, and
dependencies. It excludes file paths and source offsets because those values do
not change database semantics.

Canonical maps and sets make the fingerprint independent of input order. Lists
remain ordered. Any change that can affect resolution, typing, encoding,
cardinality, or execution must appear in a semantic property and change the
fingerprint.

The semantic diff reports provider drift, dialect drift, and exact added,
removed, or changed objects. Source-location-only changes do not appear.

## Minimum dependency slices

A query asks for named properties of each directly used object. Slice creation
keeps only those properties. It also closes over every declared object
dependency and records the complete semantics of each transitive dependency.

A slice can contain absence facts. These include a required missing object and
an exact overload candidate set. Compatible runtime verification compares the
observed schema with every required property, dependency, and absence fact.
Objects outside the slice do not affect compatibility.

## Generated profile modules

The generated module path is `sifr.sql.schemas.<profile>`. Its source contains
the nominal zero-field `Schema` type and generated enum, domain, and composite
types. A sidecar `ProfileModuleMetadata` contains the nominal identity, profile
and schema fingerprints, compiler-known exports, generated types, and complete
schema-symbol index.

Compiler-known exports are not emitted as runtime profile values. Their names
are reserved by the sidecar. A database object with a reserved name stays in the
symbol index and does not shadow the compiler export. Static
`profile.symbol("qualified.name")` lookup resolves it through `SchemaIR`.
Unqualified ambiguous lookup fails.

The runtime verification manifest contains only identities, the evidence and
strictness policies, the session contract, signer identities, and the minimum
dependency slice. It cannot contain a connection URL or credential.
