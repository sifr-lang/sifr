# Common SQL contracts

This document records the implemented provider-neutral SQL contracts. The full
platform design remains in [`sql_architecture.md`](./sql_architecture.md).

## Ownership

`sifr_sql_contract` owns compile-time data. It defines database types, Sifr read
types, bind compatibility, codecs, cardinality, effects, and the provider
analysis interface. It contains no parser and no database driver.

`sifr_sql_runtime` owns application runtime shape. It defines verified handle
states, owned parameters, explicit execution modes, structured errors,
cancellation handoff, and panic containment. It depends only on `sifr_runtime`
only. The compiler emits closed runtime metadata; the runtime never links the
compiler-only contract crate.

Provider components own dialect parsing and semantic analysis. Provider runtime
bridges own raw drivers, wire codecs, protocol cancellation, and session reset.
They implement the closed common interfaces. The common crates never choose a
dialect rule or inject a driver.

SQL remains a first-party package family. These crates are not new core standard
library modules.

## Type contract

`DatabaseType` is the complete provider-neutral storage descriptor. It includes:

- signed and unsigned 8-, 16-, 32-, and 64-bit integers;
- finite decimal, arbitrary finite decimal, and numeric special-value contracts;
- 32- and 64-bit floating storage;
- bounded and fixed text, bounded binary, UUID, and JSON identities;
- date, local time, offset time, local date-time, instant, and calendar interval;
- arrays with element nullability, dimensions, and lower-bound behavior;
- nominal enums, domains, and composites;
- ranges and multiranges;
- IP address, IP network, and MAC address values;
- exact custom types with one registered codec; and
- SQLite dynamic storage-class unions.

`canonical_read_type` preserves storage width and nominal identity for common
types. `canonical_read_type_in` uses the selected server-profile codec registry
for custom types. A registry-free custom-type request is an error. Neither path
uses `Any`, `str`, or `bytes` as a fallback. SQLite dynamic columns preserve
each allowed storage class, including `None`.

## Bind contract

`bind_compatibility` is one closed input relation. It returns one of three
results: exact, fallible with a named encoder check, or rejected with a named
reason.

Exact Sifr and database types bind without conversion. Nullable input cannot
bind to a non-null parameter. Exact `int` uses a checked fixed-width encoder.
Different fixed widths or signs do not convert implicitly. A 32-bit float target
checks range and precision. A bounded decimal target checks precision and scale.
Bounded text and binary targets check length. SQLite dynamic parameters accept
only values whose storage classes are in the declared storage-class set.

`InputType.nullability` is canonical. The bind relation also normalizes an
inline `None` or a union that contains `None` before it compares types. Thus,
inline and explicit nullability have the same behavior. A decimal bind must
match the target representation before any precision or scale check applies.

A Sifr `list[T]` binds only to a one-dimensional SQL array with lower bound one.
`SqlArray[T]` preserves dimensions and lower bounds. Array elements use the same
closed bind relation. Nominal and custom values require the exact database and
codec identities.

All pairs outside this relation are compile errors. Providers can reject more
values for an exact dialect rule. They cannot add an implicit width conversion.

## Codec contract

Each `CodecContract` contains one database type, one Sifr type, accepted server
profiles, closed encode and decode errors, null behavior, wire-format identity,
and required panic containment.

`CodecRegistry` selects one server profile. It rejects duplicate codec identities
and duplicate database-type registrations within that profile. Separate
profiles can select different wire codecs for the same common database type.
Unknown custom types are compile errors. Runtime codec operations return
`SqlError`; the common boundary catches and redacts a provider panic.

## Cardinality and effects

`Cardinality` is a complete interval lattice. It has an empty bottom value and
any valid `minimum..maximum` interval, where an absent maximum means unbounded.
The named values are `zero`, `at_most_one`, `exactly_one`, `one_or_more`, and
`many`. Join and meet are commutative and retain bottom.

Cardinality does not select a result container. It only validates the caller's
explicit `execute`, `fetch_one`, `fetch_optional`, `fetch_all`, or `stream`
method.

`EffectContract` uses the closed effects `Read`, `Write`, `ReadWrite`,
`SchemaChange`, `SessionChange`, and `TransactionControl`. It keeps referenced
objects separate from affected objects. Application query APIs accept only
read, write, and read-write effects.

## Runtime ownership

`Pool[Profile, Verified]` is cloneable, sendable, and share-safe. Verification
requires matching profile evidence and retains the schema fingerprint.

`Connection`, `Transaction`, and row streams are not cloneable or sendable.
A transaction exclusively borrows its connection. A transaction stream borrows
the transaction. A pool stream owns its leased connection until close.

Bound parameters contain only owned values. Slots are sorted and unique. No
borrowed value can enter an execution request. Debug output shows value kinds
and sizes but never shows parameter contents, profile contents, or SQL text.

Rust compile-fail documentation tests enforce the task and borrow boundaries.
Static trait tests enforce the pool and connection capabilities.

## Error contract

Compiler diagnostics use `SIFR-SQL-0001` through `SIFR-SQL-0008`. They cover
database types, binds, nullability, codecs, cardinality, effects, provider
analysis, and ownership.

Runtime operations use the closed `SqlErrorKind` variants from the architecture.
Metadata can retain SQLSTATE, vendor code, constraint identity, object identity,
columns, retry classification, and a resource-limit kind. Default display uses
fixed redacted text and never renders provider metadata.

`ProviderFuture` catches a panic during polling and returns a redacted provider
error. `catch_codec_boundary` provides the same rule for synchronous codec
operations. Provider data cannot make the common runtime unwind.

`RuntimeLimits` defines connection, deadline, decoded-byte, collected-row,
statement-cache, and parameter bounds. `ResourceUsage` uses checked arithmetic.
It returns a structured resource-limit error before a count or allocation can
exceed its configured ceiling.

## Verification

The `common-sql` SQL-platform suite executes:

- the complete type and bind matrices;
- codec registration, round-trip, malformed-value, and panic tests;
- cardinality lattice properties and explicit-fetch validation;
- effect-object validation;
- runtime redaction and asynchronous panic containment;
- ownership trait tests and compile-fail documentation tests; and
- a mutation-tested qualification record that rejects driver leakage.
