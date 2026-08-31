# PostgreSQL migrations

This document defines the PostgreSQL migration provider. Read
[`sql_migrations.md`](./sql_migrations.md) first for the common graph and engine.

## Ownership

`sifr_sql_postgresql` owns PostgreSQL DDL parsing, reflection, risk data, and
transaction classification. It uses the same versioned `libpg_query` parser as
the PostgreSQL query compiler.

`sifr_sql_postgresql_tools` owns live migration work. It owns the advisory lock,
the PostgreSQL ledger, live schema checks, statement execution, import, and the
operator command surface.

`sifr_sql_runtime` owns the provider-neutral engine and closed execution-plan
types. The PostgreSQL tool implements its `MigrationRuntime` interface.

These crates are host tools. They are not Sifr standard-library modules. They do
not enter an application dependency graph.

## Locked Rust tooling

The workspace uses Rust 1.98. Cargo and `Cargo.lock` select the implementation.
The PostgreSQL migration path uses these workspace dependencies:

| Purpose | Tool | Selected version |
| --- | --- | --- |
| Server protocol | `tokio-postgres` | 0.7.18 |
| TLS bridge | `tokio-postgres-rustls` | 0.14.0 |
| TLS | `rustls` | 0.23.43 |
| Platform trust | `rustls-platform-verifier` | 0.7.0 |
| Async I/O | `tokio` | 1.53.1 |
| Provider-version checks | `semver` | 1.0.28 |
| Stable advisory-lock key | `sha2` | 0.11.0 |
| Closed records | `serde` and `serde_json` | 1.0.229 and 1.0.151 |

The root manifest is the version authority. The SQL dependency checker and
`Cargo.lock` detect drift. Provider crates use workspace dependencies. They do
not select a second driver, parser, TLS stack, or async runtime.

The compiler embeds exact `libpg_query` sources for PostgreSQL 13 through 18.
The selected profile major chooses one parser source. The live matrix uses the
same major when it builds the provider.

## DDL reflection

The migration compiler parses one statement for each DDL step. It rejects a
parser and profile major mismatch.

The provider directly reflects these canonical object kinds:

- namespaces, tables, and columns;
- primary, unique, foreign-key, and check constraints;
- indexes and sequences;
- views and materialized views; and
- enums, domains, composites, ranges, and functions.

The provider returns an opaque result for valid DDL that its static normalizer
cannot prove. The migration must then declare the complete output `SchemaIR`.
The common compiler validates that effect against the provider identity and all
later intermediate states. This rule covers arrays, multiranges, casts,
operators, collations, extensions, triggers, generated and identity columns,
privileges, server capabilities, and dialect metadata.

The provider never guesses the effect of unhandled DDL. A reflected statement
cannot replace an existing object identity. A declared effect cannot be empty.

Reflection also reports schema-lock and data-rewrite risks. The operator plan
keeps those compile-time reports separate from live execution.

## Transaction classes

Most PostgreSQL DDL is transactional. The provider requires autocommit for these
operation families:

- `CREATE` or `DROP DATABASE`;
- `CREATE` or `DROP TABLESPACE`;
- `CREATE`, `ALTER`, or `DROP SUBSCRIPTION`;
- `CREATE INDEX CONCURRENTLY` and `DROP INDEX CONCURRENTLY`;
- `REINDEX ... CONCURRENTLY`;
- `REFRESH MATERIALIZED VIEW CONCURRENTLY`;
- `ALTER SYSTEM`;
- `VACUUM`; and
- `CLUSTER`.

The classifier ignores whitespace and SQL comments between leading keywords. It
also accepts `UNIQUE` before `INDEX`.

An autocommit operation must be outside a transaction. A named recovery point
must occur before it. The plan validator rejects a missing recovery point.

A transaction-required path has one outer `BEGIN` and `COMMIT` pair. The first
step is `BEGIN`. The last step is `COMMIT`. A transaction-forbidden path has no
transaction boundary. The provider validates these rules again after it reads
the serialized execution plan.

## Live runtime

The runtime opens one raw `tokio-postgres` client. TLS uses Rustls and the
platform trust store unless the connection profile disables TLS.

The runtime reads `server_version_num`. The actual major must equal the profile
major. The runtime identity also contains the profile modes and features. The
engine rejects a migration that needs another family, a newer server, or a
missing capability.

The runtime observes the schema through the same client that executes the step.
An open transaction therefore sees its own DDL. The catalog excludes the
`sifr_internal` namespace. The migration ledger cannot change the application
schema fingerprint.

SQL assertions must return exactly one row and one Boolean value. The engine
reports zero rows, multiple rows, false, and null as distinct failures.

A backfill statement returns its affected-row count. The count cannot exceed the
compiled batch bound. An incomplete batch stores increasing progress. The
engine can pause only outside a transaction.

The graph can name a compiled Sifr callback. Live callback binding belongs to the
integrated source-to-tool build. A host tool that has no bound callback fails the
step. It does not run another implementation.

## Advisory lock and ledger

The provider derives one signed 64-bit advisory-lock key from the database OID
and profile ledger identity. It calls `pg_try_advisory_lock`. A second process
for the same database and profile fails before it reads or changes the ledger.

The provider stores one JSON ledger row in
`sifr_internal.migration_ledger`. The row identity is the profile name. The
ledger contains:

- provider family, server version, modes, and features;
- current graph heads and live schema fingerprint;
- each applied migration checksum, selected parent, prior heads, duration, and
  output fingerprint; and
- the current direction, migration, parent, completed step checksums, recovery
  point, backfill progress, duration, and transaction state.

The engine checks the live fingerprint before it selects a path. It rejects an
unknown head, an absent merge parent, a changed migration or step checksum, a
provider mismatch, and schema drift.

Ledger writes inside a database transaction use that transaction. If a step
fails, the provider rolls back the transaction before it releases the advisory
lock. The pre-transaction progress record remains. It identifies the exact path
that a later run can retry.

## Import

Import is explicit. The command names one baseline that exists in `graph.json`.
The provider acquires the advisory lock and reads the live schema. The live
fingerprint must equal the compiled baseline fingerprint.

Import writes the named baseline as the only head. Its applied-migration map is
empty. It does not create records for changes that happened before import.

Import refuses an existing ledger row. It also refuses a missing baseline or a
fingerprint mismatch.

## Operator commands

The PostgreSQL host tool has this closed command surface:

```text
sifr sql migration plan --profile <name>
sifr sql migration import --profile <name> --baseline <id>
sifr sql migration apply --profile <name>
sifr sql migration rollback --profile <name>
```

The commands read `.sifr/sql-migrations/<profile>/graph.json`. Live commands
also read the profile authority and `SIFR_SQL_DATABASE_URL`.

`plan` is offline. It reports every forward and reverse action. Each action has
the migration, selected parent, direction, step identity, step checksum, action
kind, transaction state, and active recovery point. It lists forward-only and
reversible migrations separately. It does not print SQL text or credentials.

`apply` runs paths in the stable compiler order. A merge runs only after all its
non-baseline parents are current or applied. A bounded backfill can return a
paused report. Running `apply` again resumes the checked prefix.

`rollback` reverses the one current migration head. It uses only the reverse
steps that the compiler checked for the selected parent path. It restores the
recorded prior heads. A baseline, multiple current heads, or a migration without
a reverse path is an error.

The tool never creates a reverse statement. It never chooses a different
migration path after an error.

## Qualification

The offline suite checks reflection, opaque effects, transaction classes,
serialized-plan tampering, and exact operator actions.

The live suite runs against pinned PostgreSQL 13, 14, 15, 16, 17, and 18 images.
For each server, it covers:

- truthful and false baseline imports;
- fresh creation and a multi-step upgrade;
- two schema-neutral branches and one merge;
- transactional and concurrent-index DDL;
- advisory-lock contention and concurrent-start rejection;
- bounded backfill interruption and resume;
- failed transaction rollback;
- checksum drift, head mismatch, and live schema drift; and
- an explicit reverse plan and restored prior head.

The test derives every expected fingerprint from the live server before it runs
the migration. It then restores the baseline. This method checks the provider's
catalog normalization and runtime observations together.

The authority files are:

- `verification/areas/sql_platform/data/postgresql_migration_qualification.json`;
- `verification/areas/sql_platform/data/postgresql_migration_matrix.json`; and
- `verification/areas/sql_platform/tools/run_postgresql_migration_matrix.py`.

The qualification checker mutates the server range, execution-plan version,
operator inventory, and fail-closed inventory. Each mutation must fail.
