# SQL schema lifecycle tools

This document defines the implemented schema pull, validation, and build tools.
The main SQL architecture defines the language design.

## Ownership

`sifr_sql_tool` owns the provider-neutral lifecycle rules. It owns semantic diff
plans, authority resolution, validation reports, deterministic artifacts, and
atomic artifact replacement.

`sifr_sql_postgresql_tools` owns PostgreSQL live catalog access and the `sql`
tool entry point. It uses `pg_catalog` in one repeatable-read, read-only
transaction. It does not add code to the application dependency graph or the
Sifr standard library.

The compiler remains offline. Only the host tool reads
`SIFR_SQL_DATABASE_URL`. The tool does not serialize that value or include it in
an error.

## Commands

The PostgreSQL tool implements this closed command surface:

```text
sifr sql schema pull --profile <name> [--accept]
sifr sql schema validate --profile <name> [--live]
sifr sql schema build --profile <name>
```

The host-tool manifest must grant `network`, `credentials`, `project-read`, and
`project-write` for `pull`. `validate --live` needs `network`, `credentials`,
and `project-read`. Offline validation needs `project-read`. `build` needs
`project-read` and `project-write`.

An unknown option is an error. A repeated option is an error. The connection
URL has one fixed environment name. A command cannot select another environment
variable from its arguments.

## Canonical artifact directory

Each profile has one artifact directory:

```text
.sifr/sql/<profile>/
```

`schema build` replaces this directory as one transaction. It writes:

- `schema.json`: canonical `SchemaIR`;
- `schema.sha256`: the canonical schema fingerprint;
- `runtime-manifest.json`: runtime evidence and the complete schema slice;
- `schema.sifr`: the generated profile module;
- `schema-module.json`: generated module metadata;
- `dependency-index.json`: forward and reverse object dependencies; and
- `artifact-manifest.json`: the identity, size, and SHA-256 of each payload.

The artifact manifest does not hash itself. All JSON uses ordered maps and sets,
stable pretty serialization, and one final newline. The build reconstructs the
profile authority before it writes. A mismatch is an error.

Schema build does not inspect application query bodies. It does not write
`sifr-query-signatures.json`. Application builds own that artifact.

The writer stages a complete directory beside the destination. It then renames
the old directory to a private backup and renames the staged directory into
place. It restores the backup if the second rename fails. A stale backup,
symbolic-link path, non-absolute path, or unsafe artifact name is an error.

## Pull

Pull opens a repeatable-read, read-only catalog transaction. It verifies the
server major before it reads objects. It records namespaces, relations, columns,
constraints, indexes, sequences, identities, user types, functions, operators,
casts, collations, extensions, triggers, server capabilities, and dialect
metadata.

The adapter excludes PostgreSQL internal namespaces. It keeps provider semantics
as closed `SemanticValue` data. It rejects missing fields, null semantics,
unknown object kinds, invalid object identities, and unresolved dependencies.
The adapter uses the same structured `DatabaseType` values and property names as
the declarative DDL normalizer. Generated domain and composite annotations come
from the provider type registry. An unrepresentable type is an error.

Pull compares the live graph with the checked snapshot. If no snapshot exists,
it compares with the selected source authority. It prints the semantic diff and
flushes standard output before any write. A non-empty diff returns status 2.
It does not write unless the caller supplies `--accept`.

An accepted pull rebuilds all schema artifacts from the live graph. This keeps
the snapshot, fingerprint, manifest, generated module, and dependency index in
one consistent generation.

TLS uses the operating system trust store through Rustls when SSL is enabled.
`sslmode=disable` is the only mode that selects an unencrypted connection.

## Validate

The selected source authority is the validation baseline. Validation always
compares the checked canonical snapshot.

An `introspection` profile also compares live state. The `--live` option adds
that comparison for other evidence policies. A `migration-head` profile must
have `.sifr/sql-migrations/<profile>/schema.json`; the migration compiler owns
that file. Validation compares it with the same baseline.

Build also reads that migration-head file when it exists. The declarative source
and migration result must have equal semantics. A `migration-head` profile
requires the file. The migration result is outside the generated artifact
directory, so atomic artifact replacement cannot delete its own input.

The migration compiler publishes this file with `graph.json`, `impact.json`, and
`artifact-manifest.json`. It replaces the complete profile directory as one
transaction. [`sql_migrations.md`](./sql_migrations.md) defines these artifacts.

Validation is read-only. It reports provider changes, dialect changes, and each
added, removed, or changed object. If the application query signature artifact
exists, the report lists queries whose schema dependency set intersects the
changed objects.

Status 0 means all selected authorities agree. Status 1 means the report has a
semantic difference. Invalid input or missing required evidence is a command
error.

## Authority and credential rules

One source can claim canonical build authority. Multiple sources need an
explicit merge rule. The only common merge rule accepts sources whose normalized
schemas are semantically equal. Duplicate names or unequal schemas are errors.

Before serialization, the builder scans provider metadata and every nested
semantic value. Credential-shaped keys, URL user information, and private-key
material are errors. Error messages identify the object and property but never
include the value.

## Rust tooling

These tools are Rust crates, not Sifr standard-library modules. The root Cargo
workspace and `Cargo.lock` select their exact dependency graph.

The implementation uses the workspace's current stable selections:

- Tokio `1.53.1` for bounded asynchronous I/O;
- tokio-postgres `0.7.18` for the PostgreSQL protocol and typed catalog rows;
- tokio-postgres-rustls `0.14.0` with Rustls `0.23.43` for TLS;
- rustls-platform-verifier `0.7.0` for the operating system trust store;
- Serde `1.0.229` and serde_json `1.0.151` for closed artifacts;
- SHA-2 `0.11.0` for content identities; and
- tempfile `3.27.0` for sibling staging directories.

Update these versions only through a reviewed Cargo dependency change. Run the
complete provider qualification matrix after an update. Do not add a second SQL
parser or an ORM to the schema tool. PostgreSQL remains the semantic authority
for live catalog data.

## Failure boundary

The tool fails closed for a server-major mismatch, connection or TLS failure,
incomplete catalog data, unsafe path, authority conflict, credential disclosure,
invalid artifact, nondeterministic content, or failed atomic replacement.

The tool never falls back to an older snapshot after a requested live check.
It never converts a failed write into a successful report.
