# SQL schema polymorphism

Status: Milestone 15 contract.

This document defines reusable SQL queries that work with structural schema
requirements. It also defines portable provider constraints.

## Design rules

### No second schema language

A requirement is a checked-in provider DDL file. Sifr does not define another
schema language for reusable queries.

The provider normalizes each DDL file into `SchemaIR`. The compiler converts all
declared objects and semantic properties into a `SchemaSlice`.

The slice can contain tables, columns, keys, types, and nullability. It can also
contain other provider objects that the DDL declares.

The requirement slice does not contain absence facts. Extra objects in a
concrete profile do not invalidate a structural requirement. Table membership
properties use containment, so an application can add columns and constraints.
Declared scalar properties and primary keys remain exact.

### Explicit capabilities

Each provider schema normalizer returns a closed capability set. A requirement
must list each capability that portable code can use.

The compiler rejects a requirement when the provider does not support a listed
capability. It also rejects a profile that cannot prove that capability.

Provider query analysis returns the exact capability set that the SQL uses.
The frontend compares this provider-owned set with the requirement. A caller
cannot supply, remove, or narrow capability use.

Provider query analysis also returns every schema object that the SQL reaches.
This set includes columns used only in predicates, joins, groups, and writes.
The frontend rejects each object that is absent from the requirement.

The compiler does not remove syntax or change behavior to satisfy another
provider. There is no silent lowest-common-denominator rewrite.

### Static specialization

Each schema profile exports one compile-time value named `schema`. Its type is
`SqlSchema[Profile]`.

A reusable query accepts this value through a constrained generic parameter.
The compiler proves the requirement against the selected profile.

The compiler then analyzes the SQL with that concrete profile. The result keeps
the concrete profile identity, fingerprint, and schema fingerprint.

### Witness erasure

The `SqlSchema` witness is compile-time-only. Specialization removes the witness
from HIR and generated Rust.

A witness is valid in only these positions:

- the direct `schema` export of a generated profile namespace
- a generic parameter constrained by one schema requirement

The compiler rejects these uses:

- runtime storage
- return values
- closure capture
- field or item selection
- an unconstrained generic parameter

These rules prevent a witness from becoming dynamic provider state.

### Exact execution binding

A specialized query has one concrete profile parameter. Only these verified
handles with the same profile can execute it:

- `Pool[Profile, Verified]`
- `Connection[Profile, Verified]`
- `Transaction[Profile, Verified]`

Another profile is a compile-time error. This rule also applies when another
profile satisfies the same structural requirement.

## Manifest contract

The requirement name is package-local. Each provider family has one checked-in
DDL artifact.

```toml
[sql.requirements.has_users]
capabilities = ["sql.bind.parameters", "sql.expression.equality", "sql.query.select"]

[sql.requirements.has_users.providers.postgresql]
provider = "sifr-sql-postgresql"
source = "db/requirements/has_users.postgresql.sql"
server-version = "13"
extensions = []
sql-modes = []
```

The `provider` value must resolve to one direct locked Sifr dependency. The
source must be a normalized relative path in the declaring package.

`server-version` is the minimum version for that provider artifact. A proving
profile can use the same or a newer version.

Portable libraries add one provider table for each supported family. Milestone
15 qualifies PostgreSQL. Milestones 16 and 17 add MySQL and SQLite evidence.

## Authoring example

```sifr
from sifr.sql.requirements import has_users

def by_email[S: has_users.Schema](schema: SqlSchema[S], email: str):
    return schema.sql(t"""
        SELECT id, email
        FROM users
        WHERE email = {email}
    """)

query = by_email(app.schema, email)
rows = try await db.fetch_all(query, max_rows=100)
```

`app.schema` selects `app.Schema`. It is not a runtime argument.

The query can use only objects in `has_users`. It can use only capabilities in
the requirement manifest.

## Compiler pipeline

1. The package resolver finds requirements in the reachable locked package
   graph.
2. It resolves each provider from the declaring package scope.
3. The provider component normalizes the checked-in DDL.
4. The component returns `SchemaIR` and its capability set.
5. The compiler builds and fingerprints the complete structural slice.
6. A concrete profile proves provider identity, server version, capabilities,
   and the schema subset.
7. The provider analyzes the query and reports its exact capability use.
8. The frontend rejects undeclared objects and provider behavior.
9. The frontend emits a concrete query and removes the witness.
10. Execution accepts only a matching verified handle.

The compiler performs no live database access in this pipeline.

## Fingerprints and cache identity

The requirement artifact fingerprint includes:

- the requirement package and name
- the exact provider package identity
- the provider family and minimum server version
- the source path and source SHA-256
- the required capability set
- every required object, property, and dependency
- the normalized `SchemaIR` fingerprint

Map and set order cannot change the fingerprint. Any semantic change invalidates
the requirement cache entry.

Prepared SQL cache identity includes every provider artifact fingerprint. A
requirement edit cannot reuse a stale prepared profile environment.

## PostgreSQL capability contract

PostgreSQL 13 through 18 publish one capability set for the qualified compiler
surface. It includes binds, reads, writes, joins, subqueries, CTEs, sets,
aggregates, windows, row locking, conflicts, and `RETURNING`.

The exact set is in
`verification/areas/sql_platform/data/schema_polymorphism_qualification.json`.
The qualification checker compares that record with the provider source.

## Diagnostics

The compiler reports separate errors for these failures:

- unknown requirement
- missing provider artifact
- provider package mismatch
- server version below the requirement minimum
- missing capability
- incompatible schema object or property
- undeclared query object
- undeclared provider behavior
- invalid witness use
- execution profile mismatch

The diagnostics name the requirement, provider, object, or capability. They do
not add runtime checks or fallback behavior.

## No runtime provider dispatch

Specialization accepts one statically known profile. It does not create a union
of providers or a provider switch.

Code that supports different providers must specialize and validate each
provider independently. Application code can choose among concrete functions
through normal static control flow.

## Qualification ownership

The provider-neutral harness is in `sifr_sql_contract` and `sifr_frontend`.
PostgreSQL DDL and capability evidence is in `sifr_sql_postgresql`.

The permanent `schema-polymorphism` verification suite runs:

- requirement subset and fingerprint properties
- manifest and locked provider resolution
- witness and execution negative cases
- undeclared object and capability cases
- the production driver preparation path
- PostgreSQL DDL normalization
- qualification mutation checks

MySQL and SQLite must use this same harness. They cannot define another witness,
proof, or runtime dispatch mechanism.
