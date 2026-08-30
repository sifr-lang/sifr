# PostgreSQL runtime

This document defines the implemented PostgreSQL execution runtime for Sifr SQL.
The architecture contract remains in [sql_architecture.md](./sql_architecture.md).

## Package boundary

The runtime uses two first-party crates.

| Crate | Responsibility |
| --- | --- |
| `sifr_sql_runtime` | Pool coordination, limits, verification comparison, session policy, statement-cache policy, and transaction states. |
| `sifr_sql_postgresql_runtime` | PostgreSQL connections, TLS adaptation, wire codecs, protocol cancellation, reset commands, and SQLSTATE mapping. |

`sifr_sql_runtime` links no database driver. It depends on `sifr_runtime` and
Tokio synchronization and timing primitives.

`sifr_sql_postgresql_runtime` uses the raw PostgreSQL driver. Public APIs do not
expose types from that driver.

The provider is a Cargo-backed first-party package. It is not part of the Sifr
standard library. Generated applications include it only when they select the
PostgreSQL provider.

## Locked foundations

The workspace manifest and `Cargo.lock` are the release authority. The runtime
uses these exact stable releases from the SQL dependency baseline:

| Tool | Exact release | Enabled features |
| --- | --- | --- |
| Tokio | `1.53.1` | Workspace async, network, synchronization, and time features |
| Rustls | `0.23.43` | AWS-LC-RS, logging, post-quantum preference, standard library, and TLS 1.2 |
| Tokio Rustls | `0.26.4` | AWS-LC-RS, logging, and TLS 1.2 |
| `postgres-types` | `0.2.14` | `derive` |
| `tokio-postgres` | `0.7.18` | `runtime` |
| `tokio-postgres-rustls` | `0.14.0` | `aws-lc-rs` |

All driver crates disable default features. A dependency change requires a new
qualification baseline and a PostgreSQL 13 through 18 runtime matrix run.

## Verified pool states

`open_pool` returns `PostgresPool[Unverified]`. This type has no execution
methods. `verify_schema` consumes it and returns `PostgresPool[Verified]`.

`connect` performs both operations for the normal path.

One profile selects one evidence mode: introspection, migration-head mapping, or
a signed manifest with a panic-contained verifier. The same profile selects
exact or compatible comparison.

Exact comparison requires the schema fingerprint. Compatible comparison checks
every recorded property in the dependency slice. A recorded missing value is an
absence fact, and the runtime compares it too.

Verification creates immutable evidence with the profile and schema
fingerprints. Every request must carry the same schema identity.

## Pool and task ownership

The common coordinator owns the connection count, acquisition deadline, idle
queue, cleanup deadline, and discard accounting.

A verified pool is cloneable, `Send`, and `Sync`. A clone shares only the pool
coordinator and immutable profile.

Connections, transactions, savepoints, and streams are task-local. They are not
cloneable, `Send`, or `Sync`.

A checked-out connection owns one pool permit. A normal release resets and
verifies the session before it returns the connection. A failed reset discards
the connection.

## Session contract

The profile records the search path, time zone, role, default isolation,
read-only mode, and pooling mode.

The provider applies and verifies these values after connection creation. It
also applies and verifies them on each acquisition and after each reset.

Transaction pooling rejects a profile that requires session affinity. Raw
queries with a session-change effect remain invalid.

## Execution and caching

The provider exposes `execute`, `fetch_one`, `fetch_optional`, bounded
`fetch_all`, `stream`, `warm`, and one-field scalar extraction.

The compiler implements `expect_at_most_one` by keeping the SQL and narrowing
the runtime cardinality contract. The provider reads one extra row and reports a
cardinality error when the live database violates that contract.

The compiler implements `first` with a checked PostgreSQL one-row limit. The
normal fetch methods enforce the narrowed contract.

Parameters are owned values. The runtime checks the parameter count before a
driver call. It converts values through the prepared statement's exact types.

Rows are decoded from raw wire bytes. The decoder checks each fixed-width value,
UTF-8 text, row byte count, and collected row count. Malformed data returns a
typed error.

`ExecutionResult` contains the exact affected-row count and redacted provider
metadata. PostgreSQL metadata includes the server version and cache-hit status.

Each physical connection owns one bounded least-recently-used statement cache.
Preparation is never a public type.

The cache key includes statement, parameter, result, server-version, and schema
fingerprints. `warm` uses the same key and path as execution.

## Transactions and savepoints

Starting a transaction consumes its connection. Commit and rollback consume the
transaction handle.

The common state machine has five states: live, committed, rolled back,
poisoned, and dropped. Only a live transaction can execute or change state.

Commit cannot run while a savepoint is live. Savepoint release and rollback
consume the savepoint. Dropping an unfinished savepoint poisons the transaction.

Dropping a live transaction discards the connection. PostgreSQL then rolls back
the transaction when it closes the connection.

Context completion commits a successful body and rolls back a failed body. The
body error remains primary. A rollback failure or timeout becomes secondary
cleanup evidence.

Normal transactions never retry. `run_transaction` accepts only the generated
wrapper for a compiler-validated `@retry_safe` callback. Each allowed retry
starts a fresh transaction.

The retry policy bounds attempts and exponential backoff. Only errors classified
for transaction replay can trigger it.

## Streams

A pool stream owns its connection until `aclose`. A transaction stream borrows
its transaction until `close` or exhaustion.

Each `next` call uses the statement deadline and one cancellation claim. The
stream decodes one row at a time and provides driver backpressure.

An early pool-stream close drops the portal and resets the session. An unclosed
pool stream discards its connection.

An unclosed transaction stream poisons its transaction. This rule stops commit
after an abnormal stream exit.

## Cancellation, cleanup, and errors

Each operation has a deadline no longer than the profile statement limit.
Cancellation uses PostgreSQL's protocol cancellation token and poisons the
current connection.

After a timeout, cancellation gets one shielded cleanup interval. Failure adds
`CleanupFailed` evidence. Budget expiry adds `CleanupTimedOut` evidence.

Pool reset and transaction rollback use the same bounded cleanup rule. A primary
body, timeout, or cancellation error is never replaced by cleanup evidence.

The bridge maps stable PostgreSQL SQLSTATE classes to Sifr SQL errors. It records
safe constraint, table, and column identities when PostgreSQL supplies them.

Serialization and deadlock errors permit transaction retry. Connection-class
errors permit connection retry. Other errors do not retry automatically.

Display and debug output never includes URLs, statements, parameters, wire
values, or provider messages. Malformed data and verifier panics cannot reach a
user-triggered panic.

## Qualification

Offline qualification checks topology, exact versions, the feature allowlist,
ownership, panic scans, policy tests, and malformed codec data.

The live harness starts the exact locked PostgreSQL images for majors 13 through
18. It runs the real runtime crate against every server.

The live case covers verification, fetch shapes, affected rows, scalar values,
statement warming, early stream close, commit, rollback, savepoints, fresh retry
attempts, deadlines, and pool bounds.

The harness uses a disposable database container. It explicitly tests rollback
and removes the complete database after each server run. It uses no fake
database API or later public SQL tool namespace.
