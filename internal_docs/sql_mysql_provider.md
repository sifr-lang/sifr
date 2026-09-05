# MySQL provider

This document defines the MySQL compiler, runtime, schema tools, migrations,
test provisioning, and editor behavior.

The provider is a first-party Cargo package family. It is not part of the Sifr
standard library. A Sifr application selects `sifr.sql.mysql`; it does not
select Rust parser or driver crates.

## Package boundaries

| Crate | Responsibility |
| --- | --- |
| `sifr_sql_mysql` | Strict syntax, semantic analysis, schema normalization, diagnostics, migration reflection, recovery tokens, and WebAssembly component guest. |
| `sifr_sql_mysql_runtime` | Raw connections, TLS policy, codecs, schema verification, Sifr-owned pooling, statements, transactions, streams, cancellation, and errors. |
| `sifr_sql_mysql_tools` | Live catalog pull, schema lifecycle commands, migration execution, and disposable test databases. |
| `sifr_sql_contract` | Provider-neutral schema, query, component, capability, and connection-manifest records. |
| `sifr_sql_runtime` | Provider-neutral limits, verification, pool coordination, resource accounting, and migration engine. |

No upstream MySQL type crosses a public Sifr API. Host tools do not enter an
application dependency graph. The compiler component runs in the existing
capability-free WebAssembly host.

## Locked tooling

The root Cargo manifest and `Cargo.lock` are the implementation authority. The
provider uses this exact stable compatible set, verified on 2026-08-30:

| Purpose | Tool | Exact version | Selected features |
| --- | --- | ---: | --- |
| Parser generator | `lalrpop` | 0.23.1 | lexer and Unicode build support |
| Parser runtime | `lalrpop-util` | 0.23.1 | standard library and Unicode |
| Component ABI generator | `wit-bindgen` | 0.61.1 | macros, reallocation, and standard support |
| Component capability virtualizer | WASI-Virt | 0.2.0 at `448f6df8f688cee5d6995e96b1ffc31f9bf00742` | deny-by-default WASI composition |
| Async MySQL client | `mysql_async` | 0.37.0 | `aws-lc-rs`, `minimal-rust`, `rustls-tls`, and `tls12` |
| Protocol values | `mysql_common` | 0.37.3 | no default features |
| Async runtime | Tokio | 1.53.1 | macros, network, runtime, synchronization, and time |
| TLS | Rustls | 0.23.43 | AWS-LC-RS, standard library, and TLS 1.2 |

`mysql_common` 0.37.3 is the newest release in the family accepted by
`mysql_async` 0.37.0. The provider does not force the newer incompatible
`mysql_common` family.

The driver disables default features. The provider does not enable its
`tracing` feature because upstream events can contain statements or values.
Sifr owns redacted operational metadata.

LALRPOP, the driver, Tokio, and Rustls are private Rust implementation tools.
They are not Sifr modules. Applications do not import them.

The component builder verifies the exact WASI-Virt commit and tracked-source
digest before use. The artifact manifest records both values, each artifact
digest, and each artifact size. The guest uses sorted JSON maps. It does not
enable insertion-order maps that can request ambient randomness. WASI-Virt
then resolves every remaining WASI interface inside the artifact, so the host
grants no WASI capability.

## Supported servers

The provider supports these MySQL server series:

- MySQL 8.4 LTS;
- MySQL 9.7 LTS; and
- MySQL 26.7 Innovation.

These are the production release lines supported by MySQL on 2026-08-31.
MySQL 8.0 entered Sustaining Support on 2026-04-21 and is not a Sifr target.
MySQL changed to calendar versioning after 9.7, so there is no MySQL 9.8 line.
The release authority is the
[MySQL Innovation and LTS policy](https://dev.mysql.com/doc/refman/26.7/en/mysql-releases.html),
the [MySQL 8.0 support notice](https://www.mysql.com/support/eol-notice.html),
and the [official image inventory](https://hub.docker.com/_/mysql).

Each profile selects one exact series. A component, schema, runtime connection,
or migration target from another series is an error.

The live matrix uses the official `mysql:8.4`, `mysql:9.7`, and `mysql:26.7`
image lines. A qualification run records the resolved image digest. A new patch
release in one line needs a new complete matrix run before the evidence changes.

## Strict compiler component

`sifr_sql_mysql` owns its lexer, `mysql.lalrpop` grammar, AST, lowering, and
semantic analyzer. It does not use `sqlparser-rs`, a server parser library, or a
generic SQL AST.

The strict parser is the only compile authority. The recovery tokenizer is for
editor requests. Its output has `compile_authority = false` and cannot emit a
query plan or schema proof.

The lexer handles MySQL comments, backtick identifiers, strings, parameters,
operators, and keywords. `ANSI_QUOTES` changes double quotes from strings to
quoted identifiers. SQL modes are normalized to uppercase before parsing.

The grammar accepts the supported query, write, and DDL envelopes. Provider
lowering owns CTEs, projections, joins, filters, grouping, ordering, limits,
row locking, inserts, replaces, updates, deletes, conflict forms, tables,
views, indexes, generated columns, constraints, character sets, and collations.

Every supported series runs against a live server parser. The differential
suite sends the same statements to the provider and to MySQL `PREPARE`. A
provider-only acceptance or rejection fails qualification.

## Schema normalization

A MySQL profile must declare:

- one default database in `search-path`;
- an exact server series;
- the complete SQL mode set;
- one default character set; and
- one default collation.

The common schema-normalization envelope carries all five values. The
component does not infer a server default. The dialect fingerprint contains
the SQL modes plus `character-set:<name>` and `collation:<name>` entries.
Statement and profile cache identities therefore change when these settings
change.

DDL sources and live catalog data normalize to the same `SchemaIR`. The
normalizer emits databases, tables, views, columns, primary keys, unique keys,
foreign keys, checks, indexes, identity columns, generated columns, character
sets, collations, and dialect metadata.

Integer types retain signed or unsigned width. Temporal precision is bounded by
MySQL. Text types retain length and collation facts. `ENUM` and `SET` use stable
schema object identities. Generated columns retain the expression and whether
storage is virtual or stored.

Unnamed constraint identities use a hash of the table, constraint kind, and
semantic signature. They do not use declaration position. Reordering unrelated
columns cannot silently rename a constraint.

## Query semantics

The analyzer reads only canonical `SchemaIR`. It resolves relations and columns
offline. An unknown or ambiguous name is a provider diagnostic.

Provider analysis returns:

- ordered parameter slots and codec identities;
- ordered result fields, database types, Sifr types, and nullability;
- cardinality;
- read and write effects;
- every accessed schema object; and
- required capabilities.

The accessed-object set is provider owned. The component host does not infer it
from SQL text. Each dependency in the embedded plan carries its schema object
fingerprint.

MySQL-specific capability facts cover collations, unsigned types, generated
columns, SQL modes, `INSERT IGNORE`, `REPLACE`, and
`ON DUPLICATE KEY UPDATE`.

Portable schema requirements use the same MySQL schema component. The common
The DDL qualification harness normalizes the provider DDL, proves the required schema
objects and capabilities, specializes the query, and validates the closed
signature. No PostgreSQL artifact is reused as MySQL evidence.

## Runtime and TLS

The runtime owns raw `mysql_async::Conn` values. It never constructs
`mysql_async::Pool`. `sifr_sql_runtime::PoolCoordinator` owns connection limits,
acquisition deadlines, idle leases, reset deadlines, and discard accounting.

`open_pool` returns `MysqlPool<Unverified>`. This state cannot execute SQL.
Schema verification consumes it and returns `MysqlPool<Verified>`.

Production profiles require Rustls certificate and host-name validation. Plain
connections are allowed only through the explicit local-test policy. Target and
control connections use the same policy.

Each physical connection applies and verifies the database, time zone,
isolation level, read-only flag, and optional role. Reset clears the upstream
session and the Sifr statement cache, then reapplies and verifies the contract.
A failed reset discards the connection.

## Execution, streams, and transactions

Each connection has one Sifr-owned bounded statement cache. The upstream driver
cache must have capacity zero. A cache key includes the normalized statement,
parameter type, result type, schema, server series, SQL mode, and collation
identity supplied by the compiled request and profile.

Parameters are owned values. Codecs reject non-finite floats and malformed
values without a panic. Rows have a decoded-byte bound. Collected operations
also have a row-count bound.

A pool stream transfers its raw connection into the driver's owning result
stream. The caller pulls one row at a time, so unread rows do not enter a Sifr
buffer. The pool slot stays occupied for the complete stream lifetime. A
checked-out connection never crosses a task boundary. The driver does not
provide a safe way to recover a raw connection from this owning stream, so
exhaustion, explicit close, errors, and drop all discard that connection. This
rule favors the ownership and cancellation contracts over connection reuse.

Starting a transaction consumes a connection. Commit and rollback consume the
transaction. A transaction stream borrows the transaction until it drains or
closes. Dropping an incomplete transaction stream poisons the transaction.
Savepoint release and rollback consume the savepoint. Dropping an unfinished
transaction or savepoint discards its connection.

Normal transactions do not retry. `run_transaction` accepts only the generated
wrapper for a compiler-checked retry-safe callback. Each replay starts a fresh
transaction. The policy bounds attempts and exponential backoff. Only an error
classified for transaction replay can retry.

## Cancellation and errors

Before execution, the runtime records the target MySQL connection ID. A timeout
or cancellation poisons that connection. A separate raw control connection
sends `KILL QUERY <numeric-id>` within the cleanup budget.

The target connection is always discarded after cancellation starts. It cannot
return to the pool, even when `KILL QUERY` succeeds. This prevents a late kill
from affecting later work.

If the control connection fails or its budget expires, the primary timeout or
cancellation remains primary. The runtime attaches bounded cleanup evidence and
discards the target.

Stable vendor codes map to Sifr authentication, constraint, timeout, deadlock,
cancellation, and connection errors. Retry classification is explicit. Error
display does not include a URL, statement, parameter, row value, or server
message.

## Schema tools and test provisioning

The host tool exposes this closed surface:

```text
sifr sql schema pull --profile <name> [--accept]
sifr sql schema validate --profile <name> [--live]
sifr sql schema build --profile <name>
sifr sql migration build --profile <name>
sifr sql migration plan --profile <name>
sifr sql migration import --profile <name> --baseline <id>
sifr sql migration apply --profile <name>
sifr sql migration rollback --profile <name>
sifr sql test provision --profile <name>
sifr sql test cleanup --resource-id <id>
```

Catalog pull reads the live version, database, SQL modes, character set, and
collation. It lists tables in stable order and normalizes `SHOW CREATE TABLE`
output through the same compiler. It reads the table inventory before and after
the pull. A changed inventory fails the pull.

Provisioning uses the administrator URL in `SIFR_SQL_DATABASE_URL`. It creates a
random validated database and user. `SIFR_SQL_TEST_PASSWORD` supplies the
credential. The JSON result refers to that environment variable; it never
contains the password or administrator URL.

Cleanup accepts only the generated resource identity shape. It drops the exact
database and user. It cannot accept arbitrary identifiers or SQL syntax.

## Migrations

The common migration engine owns the graph, checksums, heads, path selection,
resume rules, drift checks, and rollback selection. The MySQL tool implements
the provider runtime.

MySQL DDL can commit implicitly. Every DDL step must be outside a transaction
and follow a named recovery point. A transaction-required path cannot contain
DDL. The serialized plan is validated again before live work.

The runtime uses `GET_LOCK` and `RELEASE_LOCK` for one database-and-profile
identity. It stores one JSON ledger row in `sifr_migration_ledger`. The ledger
contains heads, checksums, live fingerprint, completed steps, recovery state,
and backfill progress.

Import names one compiled baseline and requires an equal live fingerprint.
Apply and rollback use only compiled paths. The tool does not synthesize reverse
DDL. A missing callback executor fails a compiled Sifr data step.

## Editor behavior

The editor uses the provider lexer and semantic settings. Completion, hover,
navigation, result facts, parameter facts, and diagnostics therefore agree with
compilation.

The editor settings fingerprint includes the server series, SQL modes, and
collation. Documentation links select the matching MySQL reference manual. An
incomplete document can produce recovery tokens, but cannot produce compile
authority.

## Qualification

`mysql_qualification.json` is the offline authority. Its checker validates the
exact dependency versions, features, supported series, owners, evidence paths,
and required contracts. Its mutation mode proves that missing series, changed
versions, missing evidence, and missing contracts fail.

The live matrix runs compiler differential, runtime, schema-tool, and migration
suites on all supported series. It records the resolved official-image digest,
reported server version, tested surface, and result in checked-in evidence. The
focused suites also cover recovery input,
property stability, malformed codecs, cancellation, stream close, session
reset, statement caching, provisioning shape, migration boundaries, portable
requirements, and editor settings.
