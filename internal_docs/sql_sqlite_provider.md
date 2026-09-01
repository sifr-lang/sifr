# SQLite provider

This document defines the SQLite compiler, runtime, tools, migrations, test
provisioning, and editor behavior.

The provider is a first-party Cargo package family. It is not part of the Sifr
standard library. A Sifr application selects `sifr.sql.sqlite`. The application
does not select Rust parser or driver crates.

## Package boundaries

| Crate | Responsibility |
| --- | --- |
| `sifr_sql_sqlite` | Exact syntax, semantic analysis, affinity, schema normalization, diagnostics, recovery tokens, migration reflection, and the WebAssembly guest. |
| `sifr_sql_sqlite_runtime` | Bundled SQLite, verified dedicated workers, codecs, pooling, statements, streams, transactions, cancellation, and errors. |
| `sifr_sql_sqlite_tools` | Catalog pull, schema lifecycle commands, rebuild migrations, and disposable database files. |
| `sifr_sql_contract` | Provider-neutral schema, query, capability, and connection-manifest records. |
| `sifr_sql_runtime` | Provider-neutral limits, pool coordination, resource accounting, and verification contracts. |

No `rusqlite` or Syntaqlite type crosses a public Sifr API. Host tools do not
enter an application dependency graph. The compiler component runs in the
capability-free WebAssembly component host.

## Locked tooling

The root Cargo manifest and `Cargo.lock` are the implementation authority. The
provider uses this exact stable set:

| Purpose | Tool | Exact version | Selected features |
| --- | --- | ---: | --- |
| SQLite grammar and AST | Syntaqlite | 0.9.0 | `analysis`, `fmt`, `pin-cflags`, `pin-version`, `serde`, and `sqlite` |
| SQLite driver | `rusqlite` | 0.40.2 | `bundled`, `cache`, `hooks`, `limits`, and `unlock_notify` |
| Native SQLite binding | `libsqlite3-sys` | 0.38.2 | `bundled` |
| Bundled SQLite | SQLite amalgamation | 3.53.2 | default bundled compile flags |
| Component C toolchain | WASI SDK | 33 | WASI Preview 2 target |
| Component ABI generator | `wit-bindgen` | 0.61.1 | macros, reallocation, and standard support |
| Component capability virtualizer | WASI-Virt | 0.2.0 at `448f6df8f688cee5d6995e96b1ffc31f9bf00742` | deny-by-default WASI composition |
| Async coordination | Tokio | 1.53.1 | macros, runtime, synchronization, and time |

Cargo sets `SYNTAQLITE_SQLITE_VERSION=3053002` for all repository builds. No
`SYNTAQLITE_CFLAG_*` value is set. Therefore, the parser grammar and the bundled
runtime use SQLite 3.53.2 with the same selected compile-flag set. The library
matrix executes the bundled library, reads `sqlite3_libversion_number()`, and
records `PRAGMA compile_options`. Compiler-identification strings are observed
but are not portable qualification inputs.

These crates are private Rust implementation tools. Sifr programs cannot import
them. Sifr package metadata selects the provider package only.

The component builder verifies the exact WASI-Virt commit and tracked-source
digest before use. The artifact manifest records both values, the artifact
digest, and its size. The guest uses sorted JSON maps and does not request
ambient randomness. WASI-Virt resolves every remaining WASI interface inside
the artifact, so the host grants no WASI capability.

## Supported library

The provider supports bundled SQLite 3.53.2. A profile must select this exact
version. A component, schema, cache, or runtime from another version is an
error.

```toml
[sql.profiles.app]
provider = "sifr_sql_sqlite"
family = "sqlite"
source = "db/schema.sql"
server-version = "3.53.2"
search-path = ["main"]
compile-flags = []
required-features = ["json"]
pooling = "session"
schema-evidence = "migration-head"
schema-strictness = "exact"
```

`compile-flags` and `required-features` are provider-neutral profile inputs.
Internally, they use the common dialect-mode and feature sets. This mapping
keeps cache and schema fingerprints closed without exposing Rust tooling.

SQLite is an embedded library, so there is no server image matrix. The live
matrix runs against the exact bundled amalgamation. The matrix records
`sqlite3_libversion_number()` and the compile options.

A dependency update is one atomic change. It must update Syntaqlite, the grammar
pin, `rusqlite`, `libsqlite3-sys`, the amalgamation, `Cargo.lock`, and all
qualification evidence.

## Strict compiler component

Syntaqlite is the syntax authority. It uses SQLite's tokenizer and parser rules.
The component checks every document with the pinned grammar before it lowers
provider-owned semantic nodes.

The component does not use a generic SQL parser or a generic SQL AST. It owns
query, write, DDL, expression, conflict, and schema nodes. Syntaqlite types do
not enter the component protocol.

The strict parser is the only compile authority. The recovery tokenizer serves
editor requests. It sets `compile_authority = false`. Recovery output cannot
produce a query plan or schema proof.

The parser supports SQLite query and write forms. These forms include CTEs,
windows, `RETURNING`, `INSERT OR` actions, `REPLACE`, and `ON CONFLICT` upserts.
DDL lowering retains strict tables, `WITHOUT ROWID`, generated columns,
collations, constraints, indexes, views, and attached schema names.

The component rejects an unknown grammar version or compile flag. The version
and complete compile-flag set participate in the profile fingerprint, schema
fingerprint, and component cache key.

## Affinity and schema semantics

The provider implements SQLite's five affinity rules in their required order:

1. A declared type that contains `INT` has INTEGER affinity.
2. A type that contains `CHAR`, `CLOB`, or `TEXT` has TEXT affinity.
3. A type that contains `BLOB`, or an empty type, has BLOB affinity.
4. A type that contains `REAL`, `FLOA`, or `DOUB` has REAL affinity.
5. Every other type has NUMERIC affinity.

A non-strict NUMERIC column uses the common `SqliteDynamic` database type. It
can contain NULL, INTEGER, REAL, TEXT, or BLOB values. The compiler does not
claim a static storage class that SQLite does not enforce.

A `STRICT` table accepts only `ANY`, `BLOB`, `INT`, `INTEGER`, `REAL`, or `TEXT`.
`ANY` remains dynamic. Other strict types have one canonical Sifr read type.

An `INTEGER PRIMARY KEY` column aliases the rowid only when the table has a
rowid and the declaration meets SQLite's exact rule. `WITHOUT ROWID` tables do
not get this alias. Generated columns retain virtual or stored state.

A profile uses `main` as its default schema. It can list explicit attached
schema names. `temp` is reserved. Attached names participate in scope
resolution and fingerprints. The compiler does not inspect or attach a file.

DDL sources and catalog data normalize to the same `SchemaIR`. The normalizer
emits namespaces, tables, views, columns, primary keys, unique constraints,
foreign keys, checks, indexes, triggers, identity columns, generated columns,
and dialect metadata.

## Query semantics and editor behavior

The analyzer reads only canonical `SchemaIR`. It resolves relations and columns
offline. An unknown or ambiguous name is a provider diagnostic.

Provider analysis returns ordered parameters, ordered result fields,
nullability, cardinality, effects, accessed schema objects, and required
capabilities. The component host does not infer dependencies from SQL text.

Portable requirements use the common specialization harness. SQLite
normalizes, proves, specializes, and validates each requirement independently.
It does not reuse PostgreSQL or MySQL evidence.

Editor completion includes SQLite conflict, `RETURNING`, `STRICT`, and
`WITHOUT ROWID` forms. Hover and diagnostics use affinity and dynamic storage
class facts. Documentation links use `https://sqlite.org/lang.html`.

The editor settings fingerprint contains the SQLite version and compile flags.
Incomplete input can produce recovery tokens, but cannot produce compile
authority.

## Runtime ownership

The runtime links bundled `rusqlite`. It does not use an async SQLite wrapper or
an upstream pool.

Each physical `rusqlite::Connection` lives on one dedicated operating-system
thread. It never moves to an async task. An async caller sends one command
through a one-slot channel and waits on a Tokio one-shot response.

Opening a native connection runs through Tokio's blocking pool, so file open,
PRAGMA setup, feature probes, and attached-file setup do not block an async
executor thread. Worker destruction interrupts the current statement, requests
shutdown, and joins the owning thread. A native connection cannot outlive its
pool resource as a detached thread.

An unverified pool cannot acquire or execute. The pool verifies the expected
schema dependency slice by introspection, migration head, or a signed manifest.
Only the resulting `SqlitePool<Verified>` has execution methods. A request must
carry the same profile identity and verified schema fingerprint.

`sifr_sql_runtime::PoolCoordinator` owns connection limits, acquisition
deadlines, idle leases, reset deadlines, and discard accounting. Each pool item
is one worker handle, not one shared native connection.

Worker setup verifies the exact library number. It also verifies every required
feature. Supported feature requirements are JSON, FTS5, RTree, and math
functions. An unknown or missing feature is a configuration error.

Setup enables foreign keys and recursive triggers. It disables trusted schema.
The profile defines a bounded busy timeout, statement timeout, cleanup timeout,
row count, row bytes, parameter count, cache size, and connection count.

## Execution, transactions, and cancellation

Parameters cross the worker boundary as owned common SQL values. The worker
converts them to SQLite values. It rejects an unsigned value larger than
`i64::MAX`, a non-finite float, a sequence, or an unknown encoded value.

Rows return owned NULL, INTEGER, REAL, TEXT, or BLOB values. UTF-8 errors,
non-finite values, oversized rows, and excessive row counts are structured
errors. No value-dependent path can panic.

Prepared statements use the connection-local `rusqlite` cache. The profile
bounds its capacity. Reset rolls back an open transaction, clears the statement
cache, restores session PRAGMAs, and then returns the worker to the pool.

`stream()` sends owned rows through a one-row channel. This channel applies
consumer backpressure without moving the connection from its worker. A stream
owns its pooled connection until exhaustion or `aclose()`. Dropping an
incomplete pool stream discards the worker. Dropping an incomplete transaction
stream poisons the transaction.

Starting a transaction consumes a connection. The runtime uses
`BEGIN IMMEDIATE` to make the writer-lock point explicit. Commit and rollback
reset and release the connection. Savepoint names are generated numeric identifiers. User
text cannot become a savepoint identifier.

Before a query starts, the runtime claims the common cancellation carrier. The
claim owns a clone of `rusqlite::InterruptHandle`. A cancellation request calls
`sqlite3_interrupt()` without crossing the worker channel.

A statement timeout also calls the interrupt handle. After cancellation or a
timeout, the runtime marks the worker as poisoned and discards it. The worker
cannot process a later query. This rule removes the late-interrupt race.

SQLite busy and locked results map to bounded timeout errors. Constraint,
corruption, decode, encode, cancellation, and connection failures have stable
Sifr error kinds. Error display does not contain a database path, statement,
parameter, row value, or native message.

## Catalog and schema tools

The host tool exposes this surface:

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

Live schema commands read `SIFR_SQL_DATABASE_PATH`. The catalog reads
`PRAGMA database_list` and each `<schema>.sqlite_schema`. It excludes the
temporary schema, internal `sqlite_*` objects, and the Sifr migration ledger. It
reads stored DDL in stable order and sends it through the same exact provider
parser and `SchemaIR` normalizer as checked-in sources.

Reflection records declared types, affinity, strictness, rowid aliases,
generated-column storage, defaults, keys, indexes, triggers, views, attached
scope, and object definitions. It disables trusted schema before reflection.

Test provisioning creates one random file below
`.sifr/sql-test/sqlite`. The common manifest uses the file transport and does
not contain credentials. Cleanup accepts only the generated resource identity.
It resolves the owned directory and removes the exact database plus its WAL,
shared-memory, or journal sidecars. It cannot remove an arbitrary path.

## Rebuild migrations

SQLite consumes the provider-neutral, compiler-checked `graph.json`. It does not
have a second runtime plan format. A table rebuild is a checked sequence of DDL
and data steps that names the old table, a generated temporary table, the exact
target columns, source expressions, and object definitions to recreate.

The migration compiler rejects an unowned temporary name, duplicate rebuilds,
empty expressions, and column-count mismatches. The SQLite runtime-plan
validator accepts only one exact statement per DDL step and rejects `ATTACH`,
`DETACH`, `writable_schema`, and `VACUUM INTO` in syntax tokens. Text inside a
literal or comment is not treated as migration syntax.

The tool imports one truthful baseline before it can apply a graph. It stores
the common JSON ledger in `sifr_migration_ledger`, which catalog reflection
excludes from application schema identity. Applied migration and step
checksums, heads, fingerprints, recovery points, and backfill progress use the
shared migration engine. Changed checksums, head mismatch, or live schema drift
fail before an unrelated path can run. Rollback exists only when `graph.json`
contains an explicit compiler-proved reverse path.

The runtime gets a bounded `BEGIN IMMEDIATE` writer lock before it reads the
ledger or changes data. The lock transaction makes a failed or interrupted
SQLite run atomic and safe to retry. For a rebuild it performs this sequence:

1. Disable foreign-key enforcement before the transaction.
2. Create the generated replacement table.
3. Copy the named columns with the compiled expressions.
4. Drop the old table.
5. Rename the replacement table.
6. Recreate the compiled indexes, triggers, and views.
7. Run `pragma_foreign_key_check`.
8. Verify the live schema fingerprint after every checked step.
9. Commit the schema and ledger only when the foreign-key check has no row.
10. Restore foreign-key enforcement after success or failure.

Any failure rolls back the transaction. There is no best-effort continuation.
The plan carries before and after schema fingerprints. Live drift prevents the
tool from selecting an unrelated plan.

## Qualification

`sqlite_qualification.json` is the offline inventory. Its checker validates
the exact dependency versions, features, supported library, owners, evidence,
and required contracts. Mutation mode proves that a changed version, missing
surface, missing evidence, or missing contract fails.

The bundled matrix covers compiler conformance, runtime execution, cancellation,
locking, corruption, catalog parity, common-engine rebuilds, drift, checksums,
explicit rollback, provisioning, portable requirements, editor behavior,
properties, malformed input, and a warmed statement-cache performance budget.
It records the observed library number and runtime compile options.
