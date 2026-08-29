# Sifr SQL architecture

Status: accepted design

This document defines the complete Sifr SQL architecture. It is the authority
for compiler, package, runtime, tooling, migration, and editor behavior.

## Purpose

Sifr SQL uses native SQL for schemas and queries. Sifr provides static types,
safe composition, ownership, typed errors, and native execution.

The core product rule is:

> Write the schema in SQL. Write queries in SQL. Compose safe structure with
> Sifr. The compiler derives the result types.

Sifr SQL does not add an ORM or a second model layer. Domain classes remain
independent from database rows.

## Permanent rules

1. A checked-in SQL schema is the compile-time schema authority.
2. Normal compilation does not connect to a database.
3. Application queries use Python template strings.
4. Ordinary interpolation always produces bound values.
5. A `str` value never becomes SQL syntax or an identifier.
6. Structural SQL fragments carry a checked syntax category and scope.
7. SQL storage widths remain visible in Sifr types.
8. Query execution always returns a typed `Result`.
9. User data cannot cause a Rust panic in generated SQL paths.
10. Compiler components cannot add arbitrary HIR, types, or Rust source.
11. CLI and editor analysis use one compiler query authority.
12. Schema snapshots and migration histories cannot become two authorities.

## Terms

| Term | Meaning |
| --- | --- |
| template string | A Python `t"..."` value with static text and typed expression holes. |
| template processor | A package function that asks a compiler component to analyze a template string. |
| schema profile | A named database contract with a dialect, server profile, schema source, and stable identity. |
| schema IR | The normalized, dialect-aware compiler description of database objects and SQL semantics. |
| query template | A static SQL plan with parameter slots and a result contract. |
| bound query | A query template with owned runtime parameter values. |
| fragment | A checked SQL AST value for one syntax category. |
| structural record | An immutable Sifr value whose type is its field structure. |
| schema contract | The reachable database objects and capabilities that an application requires. |
| compiler component | A deterministic package artifact that implements a bounded compiler protocol. |

## User surface

### Schema profiles

The project declares each schema profile in `sifr.toml`:

```toml
[sql.profiles.app]
provider = "sifr-sql-postgresql"
source = "db/schema.postgresql.sql"
server-version = "18"
search-path = ["app", "public"]
extensions = ["citext"]
schema-evidence = "migration-head"
schema-strictness = "compatible"

[sql.profiles.analytics]
provider = "sifr-sql-postgresql"
source = "db/analytics.postgresql.sql"
server-version = "18"
search-path = ["analytics", "public"]
schema-evidence = "introspection"
schema-strictness = "exact"
```

The profile name is a canonical type identity. Two profiles remain distinct
when they use the same dialect and equal schema text.

The compiler exposes configured profiles through generated namespaces:

```sifr
from sifr.sql.schemas import app, analytics
```

Each profile namespace owns one nominal zero-sized schema type. `app` is a
compile-time namespace, not a runtime value. Its `sql`, `query`, `connect`, and
schema-symbol operations are compiler-known namespace exports.

Type positions use `app.Schema`. For example, a verified pool has type
`Pool[app.Schema, Verified]`.

The module does not contain generated table classes or database models. It can
contain generated enum, domain, and composite types from the SQL schema.

### Query literals

The canonical query form is a template-processor call:

```sifr
query = app.sql(t"""
    SELECT id, email
    FROM users
    WHERE id = {user_id}
""")
```

The parser treats the template string as ordinary Python syntax. The SQL
provider receives static text, typed holes, and exact source ranges.

SQL processors reject template conversions and format specifications:

```sifr
app.sql(t"SELECT {value!r}")
app.sql(t"SELECT {value:10}")
```

These forms have no safe bound-parameter meaning. A bare template string is
inert and cannot execute as SQL.

### Reusable queries

The query decorator defines a reusable typed template:

```sifr
@app.query
def find_user(user_id: int64):
    return app.sql(t"""
        SELECT id, email, display_name
        FROM users
        WHERE id = {user_id}
    """)
```

The compiler validates that every path produces one template identity. The
decorated symbol has a `QueryTemplate` type and remains callable. Each call
creates a bound query with owned parameter values.

An ordinary function can also return a bound query. Use `@app.query` when code
needs the template as a value, exports `RowOf`, or binds it many times.

The Sifr module system is also the query module system. Queries use normal
imports, privacy, generics, and control flow.

### Execution

The application selects the runtime database:

```sifr
db = try await app.connect(database_url)
```

The pool result carries the exact `app` profile identity. It cannot
execute a query for `analytics`.

Execution intent is explicit:

```sifr
result = try await db.execute(command)
row = try await db.fetch_one(query)
row = try await db.fetch_optional(query)
rows = try await db.fetch_all(query, max_rows=10_000)
stream = try await db.stream(query)
```

Scalar intent is also explicit:

```sifr
query = app.sql(t"SELECT COUNT(*) AS count FROM users")
count: int64 = try await db.fetch_one(query.scalar())
```

The SQL analyzer proves cardinality facts. These facts do not select the
public result container.

### Transactions

Transactions use the async context contract:

```sifr
async with db.transaction() as tx:
    _ = try await tx.execute(debit)
    _ = try await tx.execute(credit)
```

The context manager owns commit, rollback, cancellation, and cleanup. A
transaction carries the schema profile and runtime connection identity.

The language does not add `async with try` syntax.

## Template strings

### Core representation

Sifr core owns a package-neutral template representation:

```text
Template
├── static text parts
├── expression holes
├── conversion and format metadata
├── source ranges
└── evaluation order
```

The parser does not know SQL. Lowering preserves each hole as ordinary Sifr
HIR until the processor requests typed hole data.

### Evaluation

The compiler evaluates runtime holes once from left to right. It preserves
normal Sifr side effects and failure timing.

A processor declares the accepted hole categories. SQL supports value holes
and sealed fragment holes. It rejects all other values.

The compiler owns template construction, capture analysis, ownership checks,
and source maps. A provider cannot replace these rules.

## Compiler component platform

### Boundary

Packages can provide embedded-language analysis through compiler components.
SQL dialect providers use this platform.

The platform is not a syntax macro system. It does not permit arbitrary AST
rewrites, HIR injection, type variants, or Rust source generation.

Registration uses the locked package identity and exported symbol identity.
The compiler resolves processors through normal imports.

Conceptually, a provider declares:

```text
package: sifr-sql-postgresql
component-kind: embedded-language-provider
processor: sifr.sql.postgresql.sql
fragment-processors: predicate, expression, order_by, identifier
schema-dialect: postgresql
migration-dialect: postgresql
protocol-version: 1
```

A local variable with the same spelling does not gain provider authority.

### Component execution

Compiler components use a deterministic sandboxed protocol. `Cargo.lock` and
the resolved package metadata bind the component bytes and protocol version.

Components use the WebAssembly Component Model and a compiler-owned WIT
interface. They receive no WASI interfaces by default.

The sandbox denies undeclared access to the filesystem, network, environment,
clocks, random sources, host processes, threads, shared memory, and native
dynamic libraries.

The compiler supplies declared source inputs through immutable handles. The
component receives time, memory, recursion, output, and diagnostic limits.

The compiler supports cancellation between component operations. A timeout or
crash produces a compiler diagnostic, not a compiler panic.

Official and third-party components execute through the same component ABI.
There is no privileged in-process provider path with different semantics.

### Provider protocol

An embedded-language request contains:

```text
provider protocol version
processor identity
template parts and source maps
typed hole descriptions
schema profile and normalized SchemaIR
server semantic profile
imported fragment and template signatures
requested plan kind
compiler contract versions
```

The provider returns a closed `EmbeddedPlan`:

```text
EmbeddedPlan
├── provider and protocol identity
├── plan kind and schema identity
├── canonical provider payload
├── parameter slots
├── compiler-owned result type description
├── cardinality facts and effect contract
├── error additions and runtime declarations
├── schema dependencies and source maps
├── diagnostics
└── stable fingerprint
```

The compiler validates each output field. Type descriptions use only supported
Sifr types.

The provider payload is opaque to SQL-independent code. Its envelope remains
visible to ownership, hashing, effects, code generation, and diagnostics.

Compiler components do not execute application SQL. Runtime behavior uses
ordinary Sifr declarations and Rust interop.

## Structural record types

### Type model

Sifr has first-class immutable structural records:

```sifr
type UserSummary = {
    id: int64,
    email: str,
}
```

A structural record supports field access, named destructuring, containers,
unions, `None`, functions, generics, and control-flow narrowing.

Structural records do not support mutable fields. A mutable or identity-based
domain value uses a class.

### Identity and subtyping

The canonical identity contains field names and canonical field types. Field
order does not change type identity.

The compiler retains source order for diagnostics. SQL decoding retains
projection order separately. Positional record destructuring is unsupported.

Two records with equal canonical fields have equal types. Their source modules
and query origins do not affect equality.

An immutable record with extra fields is a subtype of a matching narrower
record:

```text
{id: int64, email: str, active: bool}
    <: {id: int64, email: str}
```

Borrowed parameters accept a wider record through a compiler-managed view. The
view exists only for the call and cannot escape it.

An owned narrowing conversion is explicit. `.project[Narrow]()` consumes the
wider record and moves the selected fields without a hidden clone.

Structural equality requires equal canonical shapes. Containers remain invariant
in their structural record element type.

A union cannot contain two structural records when one is a width subtype of
the other. The diagnostic asks the user to project or add a nominal tag.

Width subtyping never applies to classes or mutable storage.

Structural records extend the existing `ShapeIdentity` authority in
`sifr_structural_identity`. The compiler does not create a second shape identity.

### Rust representation

Code generation interns each concrete shape by a stable fingerprint. Generated
Rust uses a hidden struct for each demanded shape.

The SQL decoder uses a separate projection plan. It maps column ordinal to
field identity without changing record identity.

Generated equality, ordering, hashing, display, sendability, and clone support
depend on the capabilities of all fields.

### Naming inferred records

`RowOf` gives a stable source name to an inferred query result without creating
a model layer:

```sifr
type UserRow = RowOf[find_user]
```

`RowOf` is a compiler-known type operator over a top-level `@profile.query`
symbol path. It does not accept a runtime query value, closure, or local function.

The compiler resolves the exported query signature and produces an ordinary type
alias. No runtime value enters a type argument.

An exported query signature stores the canonical structural type. A downstream
module does not need to analyze the query body again to use `UserRow`.

## Schema profiles and schema IR

### Profile contents

A schema profile contains:

```text
profile name and package identity
dialect provider identity
server family and version
SQL modes and feature flags
search path or namespace rules
enabled extensions
schema source files
normalized SchemaIR
runtime validation policy
stable profile identity
```

Every semantic input contributes to the profile fingerprint.

### Schema sources

A profile can use one canonical SQL file or deterministic included SQL files.
It can also use a canonical artifact from schema tools.

Normal compilation never uses live introspection.

The common schema IR describes:

- catalogs, schemas, and namespaces,
- tables, columns, storage types, and type parameters,
- nullability, defaults, and generated expressions,
- primary, unique, foreign-key, and check constraints,
- indexes and partial-index predicates,
- sequences and identity columns,
- views and materialized views,
- enums, domains, composites, arrays, and ranges,
- functions, operators, casts, and volatility,
- collations and character sets,
- extensions and server capabilities,
- triggers that affect returned values, and
- dialect metadata.

The IR preserves source locations for each declared object.

### Normalization

Schema normalization removes textual differences that do not change
semantics. It preserves each difference that can change resolution, typing,
encoding, cardinality, or execution.

The schema digest uses normalized IR, provider identity, and server profile.
It does not use absolute paths or diagnostic rendering.

## SQL and Sifr types

### General rule

Database storage types map to representation-aware Sifr types. Reading and
writing use separate compatibility relations.

The compiler does not infer storage width from a domain class or exact Sifr
`int`. The SQL schema remains the storage authority.

### Integer types

Reading preserves each storage width:

| SQL type | Sifr type |
| --- | --- |
| `SMALLINT` | `int16` |
| `INTEGER` | `int32` |
| `BIGINT` | `int64` |
| an unsigned integer | the matching `uint*` type |

Writing exact `int` to fixed-width storage uses a fallible encoder. The compiler
can remove the range test when it proves that the value fits.

### Decimal and floating types

The provider maps each finite `DECIMAL(p,s)` to `decimal` when its complete range
fits. It maps every larger finite contract to `bigdecimal`.

`sifr.sql.Numeric` represents a database numeric type that permits special values.
It has `Finite(bigdecimal)`, `NaN`, `PositiveInfinity`, and `NegativeInfinity`.

A provider rejects unsupported `Numeric` variants during encoding. A schema
constraint can prove a finite contract and remove the wrapper.

An SQL 32-bit or 64-bit floating type maps to Sifr `float` on read. Encoding a
32-bit value is fallible when it loses range or precision.

### Common type mapping

Each provider maps its database types to these canonical Sifr types:

| SQL semantic type | Sifr type | Rule |
| --- | --- | --- |
| boolean | `bool` | Exact. |
| variable or fixed text | `str` | Fixed length is a fallible encode constraint. |
| binary | `bytes` | Exact byte sequence. |
| date | `sifr.datetime.Date` | Exact calendar date. |
| time without zone | `sifr.datetime.LocalTime` | No offset or zone. |
| time with offset | `sifr.datetime.OffsetTime` | Preserves the numeric offset. |
| timestamp without zone | `sifr.datetime.LocalDateTime` | No inferred zone. |
| timestamp with zone | `sifr.datetime.Instant` | Preserves one absolute instant. |
| calendar interval | `sifr.datetime.CalendarInterval` | Preserves months, days, and sub-day units. |
| UUID | `sifr.uuid.UUID` | Exact 128-bit identity. |
| JSON or JSONB | `sifr.json.JsonValue` | The plan retains the database type identity. |
| SQL array | `sifr.sql.Array[T]` | Preserves dimensions and lower bounds. |
| enum | `app.enums.<Name>` | Generated nominal enum. |
| domain | `app.domains.<Name>` | Generated nominal constrained type. |
| composite | `app.composites.<Name>` | Generated nominal immutable record. |
| range | `sifr.sql.Range[T]` | Preserves empty, infinite, and inclusive bounds. |
| multirange | `sifr.sql.MultiRange[T]` | Preserves normalized range members. |
| IP address | `sifr.net.IpAddress` | Exact address family and bytes. |
| IP network | `sifr.net.IpNetwork` | Preserves the prefix. |
| MAC address | `sifr.net.MacAddress` | Exact address bytes. |

The language provides every canonical temporal and SQL container type in this
table. A provider cannot collapse two rows into one weaker type.

SQL `NULL` adds `None` to the mapped result type. Array elements include `None`
unless the provider schema proves element non-nullability.

### Bind compatibility

All providers use this input relation:

| Sifr input | SQL parameter | Result |
| --- | --- | --- |
| exact mapped type | matching non-null type | accepted without conversion |
| `T | None` | nullable `T` | accepted |
| `T | None` | non-null `T` | rejected with a nullability diagnostic |
| exact `int` | fixed-width integer | accepted with a fallible range encoder |
| fixed-width integer | a different width | rejected and suggests the target constructor |
| `float` | 64-bit float | accepted |
| `float` | 32-bit float | accepted with a fallible range encoder |
| `str` | fixed-length text | accepted with a fallible length encoder |
| `list[T]` | one-dimensional SQL array | accepted with lower bound one |
| `sifr.sql.Array[T]` | SQL array | accepted with dimensions and lower bounds preserved |
| generated enum, domain, or composite | its exact database identity | accepted |
| a custom codec type | its registered database identity | accepted |

All other pairs are compile errors. A provider can add stricter requirements,
but it cannot add implicit numeric width conversions.

### Custom codecs

A package can register a checked codec for a database type. The codec contract
contains:

- the canonical database type identity,
- the Sifr type identity,
- accepted server profiles,
- owned encode and decode functions,
- fallible error types,
- null behavior,
- wire-format identity, and
- panic containment evidence.

An unknown database type is a compile-time error. The compiler does not use
`Any`, `str`, or `bytes` as a fallback.

### SQLite storage classes

SQLite `STRICT` tables use checked declared types. A non-strict column can
contain multiple storage classes.

If constraints do not prove a narrower type, a non-strict result uses
`int64 | float | str | bytes | None`.

Validated `typeof` conditions narrow this union through the canonical Sifr
flow-fact system. SQLite does not add a separate narrowing mechanism.

## Query model

### Query template

A query template contains:

```text
schema identity or schema requirements
dialect provider identity
canonical SQL AST and emitted statement
parameter slots
result record and decoder plan
cardinality facts and effect contract
error additions
schema dependencies and source maps
template digest
```

The template is immutable and shareable.

### Bound query

A bound query contains a template identity and owned parameter values. It does
not borrow a local Sifr value.

The compiler converts each hole to a closed SQL parameter carrier. The carrier
retains data for fallible encoding at execution time.

The conversion order follows template-hole order. Fragment insertion retains
the same deterministic order.

Execution consumes a bound query by default. A bound query implements `Clone`
only when every captured value implements `Clone`.

### Query effects

Every plan has one closed effect contract:

```text
Read
Write
ReadWrite
SchemaChange
SessionChange
TransactionControl
```

Application processors reject schema, session, and transaction changes unless
a specific API accepts them.

Application templates contain exactly one SQL statement. Migration processors
accept one DDL statement per explicit step.

## Interpolation

### Value holes

An ordinary hole produces a bound parameter:

```sifr
app.sql(t"SELECT id FROM users WHERE email = {email}")
```

The provider selects the dialect placeholder. The runtime keeps SQL text and
parameter values separate.

### Null values

`T | None` binds SQL `NULL`. The compiler does not rewrite equality.

If an equality operand can be `None`, the provider emits a diagnostic. The
program must use an explicit null-safe operator or conditional fragment.

For PostgreSQL, the null-safe form is:

```sifr
app.sql(t"SELECT id FROM users WHERE email IS NOT DISTINCT FROM {email}")
```

### Collections

Collection binding is explicit and dialect-aware. PostgreSQL arrays can use:

```sifr
app.sql(t"SELECT id FROM users WHERE id = ANY({ids})")
```

Providers define empty-collection behavior. An expansion fragment defines its
parameter count, cache identity, and empty semantics.

The compiler rejects an ordinary list in a scalar SQL position.

### Identifiers

A runtime `str` cannot become an identifier. Checked identifiers originate
from schema symbols or exhaustive Sifr control flow:

```sifr
order_column = match sort_key:
    case "email":
        app.users.email
    case "created":
        app.users.created_at
```

The provider inserts identifier AST. It does not quote runtime text.

## Typed fragments

The common contract supports these fragment categories:

```text
SqlExpression
SqlPredicate
SqlIdentifier
SqlRelation
SqlOrderBy
SqlJoin
SqlSelectList
SqlAssignmentList
SqlValues
SqlReturningList
SqlQuery
SqlCommand
```

Each fragment carries its schema profile and dialect identity.

A fragment also records:

```text
input and output relation scopes
required tables and aliases
introduced aliases and free identifiers
parameter slots
result-shape transformation
effect transformation
source maps
```

Insertion validates each scope requirement. A predicate for alias `users`
cannot enter a query that does not define this alias.

Alias names use hygienic identities. Textual equality does not grant access
across fragment boundaries.

A relation role creates one hygienic alias environment:

```sifr
@app.query
def active_users():
    u = app.users.as_("u")
    active = u.predicate(t"{u.active} = TRUE")

    return app.sql(t"""
        SELECT {u.id}, {u.email}
        FROM {u}
        WHERE {active}
    """)
```

Inserting `u` establishes the role. Its column and predicate fragments require
that exact role.

`as_` is a compiler-recognized static operation inside `@profile.query`. Its
hygienic identity comes from its syntax location.

A relation role cannot escape its query template. The compiler rejects role
creation in runtime loops, runtime branches, stored values, and returned fragments.

The canonical predicate operations are:

```sifr
predicate = app.all(filters)
predicate = app.any(filters)
predicate = app.not_(filter)
```

The identity rules are `all([]) = TRUE` and `any([]) = FALSE`.

Bitwise operators can exist as general language sugar. They are not the
canonical SQL API because Python precedence is easy to misread.

A select-list or join fragment can change the result record. Sifr control flow
must unify the resulting query types. No dynamic composition becomes untyped.

## SQL semantic analysis

### Name resolution

The analyzer resolves every database object against the bound `SchemaIR`:

- catalogs, schemas, tables, views, and columns
- aliases and correlated references
- functions, aggregates, operators, collations, and casts
- constraints, indexes, generated columns, and database-specific objects

Resolution follows the target database rules. The analyzer does not approximate
PostgreSQL, MySQL, or SQLite name resolution with one shared rule set.

The compiler reports an ambiguity when more than one object can satisfy a name.
The diagnostic includes the candidates and the source spans that introduced them.

### Parameter inference

The analyzer derives each parameter type from its SQL context. It then validates
the interpolated Sifr value against the derived type.

One parameter can appear in several positions. All positions must produce one
compatible type. The compiler rejects unresolved or conflicting parameter types.

Explicit casts can resolve an otherwise ambiguous parameter:

```sifr
app.sql(t"SELECT {value}::uuid")
```

The query plan stores the final database type and codec for every parameter.

### Result inference

The analyzer derives an ordered structural record from the select list. Every
field has these properties:

- projection name
- Sifr type
- database type identity
- nullability
- source expression span
- source relation and column when applicable
- decoder identity

Nullability analysis includes:

- schema column constraints
- outer joins
- aggregate behavior
- `CASE`, `COALESCE`, and null tests
- set operations
- scalar subqueries
- database-specific function strictness

The analyzer uses conservative nullability when a database rule does not prove a
non-null result.

### Projection names

Every result field must have a stable unique name. A simple column reference uses
its column name. Every other expression needs an explicit alias unless the dialect
defines a stable portable name.

Duplicate result names are compile errors. The diagnostic proposes aliases.

`SELECT *` is valid in a private query. The compiler emits an explicit column
list, so runtime column order cannot drift.

An exported query cannot use `SELECT *`. This rule is not configurable because a
schema addition changes its public result type.

### Cardinality facts

The analyzer records a cardinality interval:

```text
zero              0..0
at_most_one       0..1
exactly_one       1..1
one_or_more       1..N
many              0..N
```

Cardinality inference uses schema constraints and relational semantics. Examples
include unique predicates, aggregate queries without grouping, `LIMIT`, set
operations, and write statements with `RETURNING`.

Cardinality facts improve diagnostics and optimization. They never choose the
public result container. The caller always selects `fetch_one`, `fetch_optional`,
`fetch_all`, `stream`, or `execute`.

### Write statements

The analyzer validates `INSERT`, `UPDATE`, `DELETE`, `MERGE`, and dialect-specific
write forms against the schema contract.

It tracks:

- target columns and assignment types
- required columns without defaults
- generated and identity columns
- conflict clauses
- affected relation identities
- `RETURNING` or equivalent result records
- required privileges when the schema source includes them

A write without a result clause has no row result. Its execution result reports
the affected-row count and database-specific metadata.

### Batch writes and dynamic assignments

`SqlValues` represents rows with one checked structural shape. It records column
order, row count bounds, parameter count, and per-field codecs.

An empty values collection requires an explicit empty policy. The provider does
not invent a valid `INSERT` statement from an empty collection.

`SqlAssignmentList` represents checked `UPDATE SET` entries. Duplicate columns,
generated columns, missing required values, and incompatible assignments are
compile errors.

Provider APIs expose checked upsert and conflict fragments. They preserve exact
provider behavior instead of translating through a common approximate form.

A batch whose parameters exceed a provider limit is a compile-time error when
the size is static. A dynamic batch uses a bounded batch executor. Chunking is
explicit because it can change atomicity, lock duration, and returned row order.

## Runtime architecture

The SQL runtime exposes these principal types:

```text
Pool[Profile, State]
Connection[Profile, State]
Transaction[Profile, State]
QueryTemplate[Profile, Params, Row, Cardinality, Effect]
BoundQuery[Profile, Row, Cardinality, Effect]
RowStream[Profile, Row]
ExecutionResult[Metadata]
```

`State` is `Unverified` or `Verified`. Query execution requires a verified pool,
connection, or transaction.

### Pool creation and schema contracts

Pool creation and verification are explicit and fallible:

```sifr
pool = try await app.open_pool(database_url)
verified = try await pool.verify_schema()
```

`app.connect(url)` combines `open_pool` and `verify_schema` for the normal path.
It returns `Pool[app.Schema, Verified]` and preserves the verification error.

Schema verification has two independent configuration values:

```text
schema-evidence = introspection | migration-head | signed-manifest
schema-strictness = exact | compatible
```

The evidence mode produces a trusted observed `SchemaIR` identity. The strictness
mode compares that observation with the compile-time schema.

`exact` requires the observed fingerprint to match the compile-time fingerprint.

`compatible` compares the runtime schema with the recorded dependency slice. The
following properties must remain equal:

- each referenced object identity and namespace
- each referenced column type, nullability, collation, and generated behavior
- each default fact used by a write
- the complete set of required columns for each write target
- each constraint identity used by a cardinality proof
- each resolved function signature, volatility fact, and overload candidate set
- each trigger set that can change a returned value
- each enum, domain, composite, array, range, and codec identity in the plan

Objects outside the dependency slice are unconstrained. A changed property in
the slice is incompatible, even when the database describes it as additive.

`migration-head` validates the applied heads. It maps them to a checked-in graph
state and its `SchemaIR` fingerprint. Unknown heads fail verification.

`signed-manifest` validates a deployment-produced `SchemaIR` identity without
broad runtime introspection privileges. The profile pins accepted signer
identities.

`introspection` validates referenced objects through live metadata queries.

The compiler records the minimum referenced schema slice for every executable.
The slice includes absence facts, such as the overload candidates for a function.

Verification returns a new typed handle. An unverified handle has no query
execution methods. This makes accidental execution before validation impossible.

### Session contract

The profile records each session value that can change SQL semantics. This
includes the search path, SQL mode, collation, time zone, role, and isolation
defaults.

The provider applies the session contract on every connection acquisition. It
applies the contract again after a connection reset.

Verification reads the effective session values and rejects drift. A transaction
pooler must support per-acquisition setup.

The profile declares `pooling = "session"` or `pooling = "transaction"`.
Transaction pooling rejects a setting that cannot be applied per acquisition.

Application code can change session state only through typed options:

```sifr
async with db.session(
    SessionOptions(role="reporter", statement_timeout=seconds(5)),
) as session:
    rows = try await session.fetch_all(query, max_rows=100)
```

The session restores the profile contract before it returns the connection. A raw
`SET` statement remains a rejected `SessionChange` effect.

### Connections and transactions

A pool acquires owned connections. A transaction borrows one connection for its
lifetime and controls commit or rollback through `Result`.

```sifr
async with verified.transaction(
    isolation=Isolation.serializable,
    read_only=False,
    statement_timeout=seconds(10),
) as tx:
    _ = try await tx.execute(create_order(...))
    _ = try await tx.execute(add_line(...))
```

Leaving the transaction scope without a successful commit starts rollback. A
rollback failure is preserved with the original error. Generated runtime code does
not panic during cleanup.

Cancellation or timeout starts rollback under the bounded cleanup budget. The
original cancellation remains primary.

If rollback fails or the budget expires, the runtime invalidates and discards the
connection without another awaited step. It adds `SecondaryError.CleanupFailed`
to the cancellation evidence.

Nested transactions use savepoints only when the provider declares exact support.
The API rejects unsupported nesting at compile time when the receiver type proves
the provider. Otherwise, creation returns a structured runtime error.

Normal context-managed transactions never retry automatically. Replayable retry
uses a separate callback API:

```sifr
@retry_safe
async def transfer(
    tx: Transaction[app.Schema, Verified],
) -> Result[None, SqlError]:
    ...

result = try await verified.run_transaction(
    transfer,
    retry=RetryPolicy.serialization(max_attempts=3),
)
```

The callback receives a fresh transaction on each attempt. The compiler applies
the canonical `@retry_safe` rules from the async model.

The callback can use transaction SQL and validated replay-safe computation. Each
captured value must be owned and implement `Clone`.

### Task sharing

`Pool[P, Verified]` implements `ShareSafe`. Cloning it clones only a synchronized
handle, not its connections.

A request task can own one cloned pool handle. `Connection`, `Transaction`, and
`RowStream` cannot cross a spawn boundary.

### Execution methods

The runtime has explicit result-shape methods:

```sifr
_ = try await pool.execute(query)
row = try await pool.fetch_one(query)
row = try await pool.fetch_optional(query)
rows = try await pool.fetch_all(query, max_rows=500)
stream = try await pool.stream(query)
```

`execute` accepts statements that do not return rows.

`fetch_one` requires a query whose inferred upper bound is one. It returns
`Result[Row, SqlError]`. Zero rows and a violated upper bound are separate
structured errors.

`fetch_optional` also requires an inferred upper bound of one. It returns
`Result[Option[Row], SqlError]`. A violated upper bound is an error.

The compiler rejects an incompatible execution method and explains the
cardinality cause. The runtime still validates cardinality because the live
database can violate its schema contract.

`query.expect_at_most_one()` changes an unproven upper bound to one. It keeps the
SQL unchanged and reports extra rows as `CardinalityError`.

`query.first()` adds the provider-specific one-row limit. An unordered query gets
a determinism lint because different executions can select different rows.

`fetch_all` requires an explicit maximum or a statically proven numeric bound. A
literal `LIMIT`, a literal limit fragment, or a schema proof can supply the bound.

The runtime returns an error before the result exceeds the selected bound.

`stream` returns a fallible row stream with backpressure. A pool stream owns its
acquired connection until close.

A transaction stream borrows its transaction. The borrow checker rejects commit
or scope exit while that stream remains live.

`async for` early exit calls `aclose` through the canonical async-iterator rule.
Cancellation uses the bounded cleanup budget and discards an unclean connection.

A single-column result remains a one-field record. `.scalar()` converts it to the
field type only when the result record has exactly one field.

### Ownership and value lifetime

`BoundQuery` owns encoded parameter values. It never borrows temporary user data.
This permits safe storage, asynchronous execution, and deferred execution.

The stream ownership rule uses normal Sifr borrow analysis. User code does not
contain a lifetime annotation.

Rows decode into owned Sifr values by default. Providers can expose opt-in borrowed
views only when their lifetime is statically tied to the source row buffer.

### Statement cache

There is no public `PreparedQuery` type. Each connection maintains a bounded
prepared-statement cache. Cache identity
includes the normalized SQL, parameter database types, result database types,
provider version, and schema fingerprint.

Schema contract failure invalidates affected entries before execution.

`connection.warm(template)` prepares one template on a held connection. The
template still binds through its normal callable API.

Profiles define the cache size and eviction policy. The default policy is bounded
least-recently-used eviction.

### Cancellation, timeouts, and resource bounds

Every execution API accepts a shorter deadline than the profile limit. Providers
map cancellation to their native protocol and preserve the reason in `SqlError`.

Profiles use these resource values:

| Configuration value | Type | Default |
| --- | --- | --- |
| `max-connections` | `uint32` | `10` |
| `acquire-timeout` | `Duration` | `30s` |
| `statement-timeout` | `Duration` | `30s` |
| `cleanup-timeout` | `Duration` | `5s` |
| `max-decoded-row-bytes` | `uint64` | `16 MiB` |
| `max-collected-rows` | `uint64` | `10_000` |
| `statement-cache-capacity` | `uint32` | `100` |
| `migration-lock-timeout` | `Duration` | `30s` |

The runtime rejects an exceeded bound with a structured error. It does not allocate
without a checked bound.

### Execution results

`execute` returns `ExecutionResult[M]`:

```sifr
struct ExecutionResult[M]:
    rows_affected: uint64 | None
    metadata: M
```

`rows_affected` is `None` when the provider has no meaningful count for the
command. The provider never substitutes zero for an unknown count.

The provider defines `M` as a structural record. PostgreSQL records the command
tag. MySQL records the optional insert identity and warning count.

SQLite records the optional last row identity and change count. Cross-provider
code can use only `rows_affected` unless it declares a metadata requirement.

### Application testing

`sifr sql test provision --profile app` creates an ephemeral database at the
canonical schema fingerprint. It returns a structured connection manifest.

`db.test_transaction()` requires a test build. It returns a verified transaction
that always rolls back under the cleanup budget.

Sifr SQL does not provide an in-memory fake database. Tests use the exact provider
because a fake SQL engine cannot preserve provider semantics.

## Error model

The compiler owns these diagnostic families:

```text
SIFR-COMPONENT-*    component loading, protocol, sandbox, and resource errors
SIFR-SQL-*          common query and type errors
SIFR-SQL-SCHEMA-*   schema normalization and dependency errors
SIFR-SQL-FRAGMENT-* fragment category, role, and scope errors
SIFR-SQL-PROFILE-*  profile and session contract errors
SIFR-SQL-TOOL-*     tool graph and schema-tool errors
SIFR-MIGRATE-*      migration graph and step errors
```

The compiler registry in `sifr_diagnostics` owns these families. It reserves
code identities before their implementation starts.

Each provider ships a registry for `SIFR-SQL-<PROVIDER-ID>-NNNN`. Component
loading validates the provider identity, nonzero code numbers, unique codes,
documentation links, severity, and message arguments.

A provider cannot replace a compiler-owned code. Provider lint diagnostics use
the existing `LINT` rule lifecycle for configuration and suppression.

Every diagnostic contains the Sifr source span. When applicable, it also contains
the SQL virtual-document span, schema declaration span, provider note, and a
machine-applicable edit.

Runtime operations return `SqlError`. Its principal variants are:

```text
ConfigurationError
SchemaContractError
ConnectionError
AuthenticationError
TimeoutError
CancelledError
ConstraintError
SerializationError
DeadlockError
DecodeError
EncodeError
CardinalityError
ResourceLimitError
ProviderError
MigrationError
```

Errors preserve safe database metadata such as SQLSTATE, vendor code, constraint
identity, and retry classification. They do not expose credentials or parameter
values through default display text.

`ConstraintError` has closed kinds for unique, foreign-key, check, not-null,
exclusion, and provider-specific constraints. It carries the resolved constraint,
table, and columns when the provider reports them safely.

`SerializationError` and `DeadlockError` retain provider retry classifications.
Applications never need to match display text or vendor message wording.

Retry classification is data, not automatic behavior. The runtime never retries a
statement unless application code or an explicit transaction policy requests it.

## Schema-polymorphic queries

Ordinary queries bind to one concrete schema profile at definition. This is the
default because it gives exact name resolution and stable public types.

Reusable libraries declare schema requirements in provider-owned DDL artifacts:

```toml
[sql.requirements.has_users]
provider = "sifr-sql-postgresql"
source = "db/requirements/has_users.postgresql.sql"
```

The provider normalizes this artifact into a `SchemaIR` subset. It does not become
an application schema authority.

The compiler generates a nominal requirement type:

```sifr
from sifr.sql.requirements import has_users

def by_email[S: has_users.Schema](db: SqlSchema[S], email: str):
    return db.sql(t"""
        SELECT id, email
        FROM users
        WHERE email = {email}
    """)
```

A requirement is a structural contract over normalized database objects. The
compiler proves the `SchemaIR` subset relation before specialization.

A portable requirement supplies one DDL artifact for each declared provider. Each
provider normalizes and validates its own artifact.

Schema-polymorphic queries cannot depend on undeclared objects or provider behavior.
The compiler specializes and validates the SQL for every concrete use.

## Dialect packages

### Common package

`sifr.sql` owns:

- query, fragment, and execution protocols
- structural result conventions
- common codecs and errors
- schema contract interfaces
- transaction and streaming interfaces
- compiler component interfaces

It does not define one universal SQL grammar.

### PostgreSQL package

`sifr.sql.postgresql` owns PostgreSQL grammar, catalogs, operator resolution, casts,
functions, arrays, ranges, composite types, `RETURNING`, conflict clauses, locking,
and protocol behavior.

The Cargo distribution name is `sifr-sql-postgresql`. The Sifr module path is
`sifr.sql.postgresql`. All providers use this hyphen-to-dot naming rule.

### MySQL package

`sifr.sql.mysql` owns MySQL grammar, coercions, collations, unsigned numeric types,
generated columns, conflict behavior, SQL modes, and protocol behavior.

The selected SQL mode is part of the schema and query cache identity.

### SQLite package

`sifr.sql.sqlite` owns SQLite grammar, affinity rules, strict tables, rowid behavior,
generated columns, conflict behavior, attached-database scope, and file runtime.

The profile records required SQLite features and minimum library version.

### Cross-dialect code

Portable code uses an explicit dialect capability constraint. The compiler validates
the query independently for every declared provider.

There is no silent lowest-common-denominator rewrite. A provider-specific construct
requires a provider-specific branch or an explicit abstraction supplied by the
application.

## Compiler pipeline

The frontend preserves template strings and embedded source spans during parsing.
It does not lower SQL to ordinary string concatenation.

Compilation follows this order:

1. Parse the Sifr module and preserve each template-string segment.
2. Resolve imports, schema profiles, and compiler component registrations.
3. Type-check interpolation expressions enough to produce input type descriptors.
4. Invoke the SQL provider with source, schema, dialect, and type descriptors.
5. Receive validated SQL semantics, required coercions, dependencies, and diagnostics.
6. Finish Sifr type checking with the query and structural result types.
7. Lower the query plan and fragment plans into HIR.
8. Intern structural Rust layouts and static SQL metadata.
9. Generate provider runtime calls and safe codecs.

The SQL provider returns typed compiler data. It does not return arbitrary HIR,
Rust source, linker arguments, or executable code.

### Repository ownership

Existing crates keep their current responsibilities:

| Owner | SQL responsibility |
| --- | --- |
| `sifr_syntax` | Preserve Python template-string syntax and spans. |
| `sifr_type_system` | Own structural records and closed embedded-plan host types. |
| `sifr_structural_identity` | Canonicalize record shapes, profile identities, and generated layout keys. |
| `sifr_package` | Resolve component metadata, tool selections, capabilities, and locked package identities. |
| `sifr_frontend` | Orchestrate typed component queries and expose one semantic authority. |
| `sifr_analysis` | Own dependency indexes, incremental invalidation, and semantic query facts. |
| `sifr_source` and `sifr_diagnostics` | Own cross-language spans and stable diagnostic transport. |
| `sifr_ir` and `sifr_lowering` | Carry validated closed plans and ownership without dialect logic. |
| `sifr_codegen` | Emit provider runtime calls, static metadata, codecs, and record layouts. |
| `sifr_driver` | Load checked inputs, host components, assemble builds, and enforce reproducibility. |
| `sifr_lsp` | Present frontend results through embedded virtual documents. |
| `sifr_runtime` | Own package-neutral async, resource, cancellation, and panic-safety substrate. |
| `sifr` | Dispatch built-in and package-provided command namespaces. |

A new `sifr_compiler_component` crate owns the WIT contract, WebAssembly host,
resource limits, protocol validation, and component cache transport.

A new `sifr_sql_contract` crate owns compiler-facing SQL envelopes, `SchemaIR`
interfaces, common type descriptors, capability manifests, and conformance data.
It contains no dialect parser or runtime driver.

Dialect packages own their parser, analyzer, schema normalizer, WebAssembly
component, Sifr API declarations, audited Rust runtime bridge, tools, migration
rules, and provider qualification fixtures.

The driver and language server cannot implement dialect semantics. They consume
the same frontend queries and validated provider plans.

### Cache identity

`sifr_frontend::cache_keys` owns the typed `EmbeddedAnalysisKey` family. It contains:

- normalized template segments
- interpolation type descriptors
- interpolation fragment identities
- schema fingerprint and required schema slice
- dialect provider identity and version
- database compatibility settings
- compiler component protocol version
- Sifr compiler semantic version

The cache value contains the validated query plan, dependencies, diagnostics,
structural result type, parameter codecs, and source-map data.

The component cache uses the existing frontend cache root and atomic entry format.
Its default size limit is `512 MiB` with least-recently-used eviction.

The current compiler session pins its live entries. Eviction changes performance
only and cannot change diagnostics or generated output.

Changes invalidate only entries whose dependency fingerprints changed. A column
comment change does not invalidate a query unless comments participate in the
selected contract. A changed referenced column type always invalidates it.

### Reproducibility

Normal compilation is offline. All schema inputs and compiler components come from
the resolved package graph and checked-in project inputs.

The build record includes component hashes, schema fingerprints, dialect settings,
and migration heads. Two builds with equal records must produce equal query plans.

The compiler and each component declare compatible protocol ranges. A major-range
mismatch is a `SIFR-COMPONENT-*` error and never selects a weaker protocol.

`schema build` emits an exported-query signature artifact. It records query
parameters, result types, cardinality, effects, and schema dependencies.

An exported inferred-row change is a breaking package API change. CI can compare
signature artifacts before publication.

## Editor and language-server support

The language server treats every SQL template as an embedded virtual document.
Source maps connect virtual SQL offsets to Sifr source offsets and interpolation
holes.

The language server provides:

- SQL syntax highlighting
- table, column, function, and operator completion
- hover information with database and Sifr types
- go-to-definition for schema objects
- find-references across queries, schema files, and migrations
- rename for supported schema objects
- parameter and result type display
- nullability and cardinality display
- format support that preserves interpolation holes
- quick fixes for aliases, casts, missing columns, and unsafe collection
- migration impact previews

Completion inside a fragment uses its relation scope and alias environment. A
fragment cannot offer columns that its type does not permit.

Diagnostics from a provider appear at the original Sifr span. Related information
links to the embedded SQL span and schema declaration.

The LSP performance registry adds `perf.lsp.sql.completion`,
`perf.lsp.sql.hover`, `perf.lsp.sql.navigation`, `perf.lsp.sql.diagnostics`, and
`perf.lsp.sql.format`.

Each embedded request uses the existing LSP cancellation, progress, and watchdog
contracts. The frontend checks cancellation before component entry and between
provider operations.

## Tool graph and command runner

Package tools use direct Sifr command namespaces:

```text
sifr <tool-namespace> [arguments]
```

The SQL tool package contributes the `sql` namespace. Users run `sifr sql`, not
`sifrx` or `sifr x sql`.

Cargo owns tool dependency resolution. A project uses a dedicated tools workspace
member:

```toml
# root Cargo.toml
[workspace.metadata.sifr]
tools-package = "project-tools"

# tools/Cargo.toml
[package]
name = "project-tools"

[dependencies]
sifr-sql-postgresql-tools = "..."

[package.metadata.sifr]
manifest = "sifr.toml"
```

The tools member selects exported entry points in its `sifr.toml`:

```toml
[tools.sql]
package = "sifr-sql-postgresql-tools"
entrypoint = "sql"
capabilities = ["network", "credentials", "project-write"]
```

The tools member is not an application dependency. Sifr resolves and builds it for
the host triple through the workspace `Cargo.lock`.

Application builds select only the target application package. Tool code does not
enter target HIR, generated Rust, linker input, or application artifacts.

Tools receive an explicit capability set. Network, environment variables, file
paths, credential helpers, and subprocess access require grants from the command or
project configuration.

Built-in command namespaces are reserved. A tool cannot shadow a built-in command.
Duplicate tool namespaces are hard errors.

Package scripts remain available through `sifr run --script`. A script is not a
tool namespace and cannot receive tool capabilities implicitly.

## Schema tools

Each dialect package provides a tool package with a common command contract.

### Pull

```text
sifr sql schema pull --profile app
```

`pull` connects to the configured database, normalizes its catalog, and writes a
canonical schema snapshot. It preserves provider-specific objects that affect
query semantics.

The command displays a semantic diff before replacement unless the caller passes
an explicit non-interactive acceptance flag.

### Validate

```text
sifr sql schema validate --profile app
```

`validate` compares the selected source, checked-in canonical snapshot, migration
result, and optional live database according to profile policy.

It reports object-level differences and affected queries. It does not silently
rewrite project files.

### Build

```text
sifr sql schema build --profile app
```

`build` compiles declarative schema sources and migrations into the canonical
snapshot, fingerprint, runtime manifest, and generated Sifr schema module.

Generated output is deterministic. The command fails when two inputs claim
authority for the same object without an explicit merge rule.

## Migration architecture

The checked-in canonical schema is the compile-time authority. The migration graph
must reproduce that schema for every supported starting state.

This separates two concerns:

- the canonical schema answers what the database must be
- migrations answer how an existing database reaches that state

### Migration graph

Migrations form a directed acyclic graph. Each migration has:

- stable identity
- parent identities
- provider constraint
- input schema fingerprint
- output schema fingerprint
- ordered steps
- transactional capability requirement
- optional rollback plan
- author and creation metadata

Multiple heads require an explicit merge migration. The tool rejects accidental
branches and cycles.

### Intermediate schema states

Every migration step transforms one typed schema state into the next. Later steps
type-check against the result of earlier steps.

```sifr
@migration(id="2026_08_add_status", parents=["2026_07_previous"])
def add_status[S: MigrationState](plan: MigrationPlan[S]):
    after_add = plan.ddl(
        t"ALTER TABLE orders ADD COLUMN status TEXT NULL"
    )

    after_data = after_add.sql_step(t"""
            UPDATE orders
            SET status = 'pending'
            WHERE status IS NULL
    """)
    after_assert = after_data.assert_sql(t"""
        SELECT NOT EXISTS(
            SELECT 1 FROM orders WHERE status IS NULL
        ) AS valid
    """)
    return after_assert.ddl(
        t"ALTER TABLE orders ALTER COLUMN status SET NOT NULL"
)
```

`MigrationPlan[S]` is affine. Each step consumes its plan and returns a plan with
a compiler-generated nominal state type. A local value never appears in a type
argument.

`plan.data(callback)` contextually types its nonescaping callback as
`async def callback(db: MigrationDb[S]) -> Result[None, SqlError]`. Data code can
refer only to objects available in `S`.

`assert_sql` accepts one non-null `valid: bool` field. `false` returns
`MigrationAssertionError`. Zero or multiple rows return `MigrationStateError`.

Migration definitions use ordinary Sifr decorators, functions, template strings,
and types. They do not add a migration grammar to the Sifr parser.

### Step types

A migration can contain:

- declarative DDL transformations
- checked raw DDL for provider features
- typed Sifr data transformations
- typed SQL data transformations
- assertions
- backfill batches
- explicit transaction boundaries when the provider requires them

Raw DDL is parsed and reflected into the next schema state. An opaque statement
that prevents reflection must declare its schema effect explicitly. The compiler
validates the declared effect against the final canonical schema.

### Data migrations

Typed data migrations use a migration-only database capability. They cannot open
unrelated connections or execute SQL outside the declared profile.

Large transformations use bounded batches and explicit progress keys. The engine
stores resumable progress only for steps that declare idempotent replay semantics.

There is no automatic retry for an unclassified data step.

### Offline validation

The migration engine validates the entire graph without a live database:

1. Load each supported baseline schema.
2. Apply graph transformations in topological order.
3. Type-check every DDL, query, assertion, and data step against its state.
4. Validate every provider capability requirement.
5. Compare each head with the canonical schema fingerprint.
6. Produce destructive-change, lock-risk, and data-rewrite reports.

Provider integration tests also execute migrations against real database versions.
Offline validation remains the normal compiler path.

### Rollback

Rollback is explicit. A migration can provide a checked reverse plan when reversal
is semantically valid.

The tool does not synthesize destructive rollback. A migration without a reverse
plan is forward-only.

### Runtime safety

Migration execution acquires a provider-specific advisory lock or equivalent. It
validates the current heads and schema fingerprint before the first step.

The engine records step start, completion, checksum, duration, and resulting
fingerprint. A changed checksum for an applied migration is an error.

Non-transactional DDL requires an explicit execution plan with recovery points.
The tool stops at the first failed step and reports the exact recoverable state.

### Import and baselines

Projects can import an existing database as a named baseline. The import stores its
canonical fingerprint and provider metadata.

Future migrations start from that baseline. The tool never invents historical
migrations for changes that occurred before import.

## Security and trust

SQL values are parameters by default. Text interpolation is unavailable for SQL
syntax unless the expression is a typed fragment or an explicitly unsafe escape.

An unsafe raw SQL escape requires:

- an `unsafe` block
- a provider-specific capability
- a source annotation with a reason
- a security lint that is an error in release builds by default

Compiler components execute with deterministic inputs and declared resource bounds.
They have no ambient file, network, clock, random, environment, or process access.

Schema snapshots and generated manifests never contain credentials. Tool output
redacts credentials, parameter values, and secrets by default.

Runtime providers use checked length conversions, bounded allocations, and fallible
decoders. Malformed server data returns an error and cannot trigger a user-reachable
panic.

## Verification strategy

### Compiler component protocol

A non-SQL fixture component proves that the platform is genuinely generic. It must
exercise parsing, typed holes, dependencies, diagnostics, caching, and source maps.

Protocol tests validate determinism, malformed output handling, resource limits,
version negotiation, and package isolation.

### Dialect conformance

Each dialect has:

- parser and analyzer snapshots
- catalog normalization fixtures
- compile-pass and compile-fail Sifr fixtures
- live integration suites for supported database versions
- differential tests against database type and nullability behavior
- migration graph and recovery suites
- runtime protocol and cancellation suites

Provider claims are explicit. Unsupported server versions fail during schema build
or pool validation.

### Safety testing

Fuzzing covers SQL parsing, schema ingestion, component protocol decoding, runtime
row decoding, and migration metadata.

Property tests cover structural record canonicalization, schema normalization,
cache invalidation, parameter ordering, and migration graph reproduction.

Generated runtime code is audited for data-dependent panic paths. Error-path tests
exercise malformed rows, truncated protocol frames, integer overflow, invalid text,
resource exhaustion, cancellation, and cleanup failure.

### Performance qualification

Benchmarks track:

- cold and warm query analysis
- incremental schema-change invalidation
- generated code size
- parameter encode and row decode throughput
- statement-cache behavior
- streaming memory bounds
- full migration graph validation

Performance targets belong to provider qualification records. A regression blocks
provider release when it exceeds the recorded budget.

## End-to-end example

```toml
[sql.profiles.app]
provider = "sifr-sql-postgresql"
source = "db/schema.postgresql.sql"
server-version = "18"
search-path = ["app", "public"]
schema-evidence = "migration-head"
schema-strictness = "exact"
```

```sifr
from sifr.env import getenv_opt
from sifr.sql import SqlError
from sifr.sql.schemas import app

@app.query
def find_users(domain: str, limit: int64):
    u = app.users.as_("u")
    active = u.predicate(t"{u.active} = TRUE")
    query = app.sql(t"""
        SELECT {u.id}, {u.email}, {u.created_at}
        FROM {u}
        WHERE {active}
          AND {u.email} LIKE {'%' + domain}
        ORDER BY {u.created_at} DESC
        LIMIT {limit}
    """)

    return query

async def main() -> Result[None, SqlError]:
    url = getenv_opt("DATABASE_URL")
    if url is None:
        print("DATABASE_URL is required")
        return None

    db = try await app.connect(url)
    stream = try await db.stream(find_users("@example.com", 100))

    async for user in stream:
        print(user.email)

    return None
```

The compiler validates all object names and SQL types offline. It infers this row
type:

```text
{id: int64, email: str, created_at: sifr.datetime.Instant}
```

The runtime verifies the migration heads before execution. Interpolated values are
owned parameters. The predicate is a typed SQL fragment with the required relation
scope.

## Rejected alternatives

### Generated model or ORM layer

An ORM makes tables the primary user abstraction and hides SQL behavior. Sifr keeps
SQL explicit and generates only schema identities, structural result types, and
safe compiler metadata.

### Custom tagged-string syntax

A language-specific SQL string prefix duplicates template-string semantics and
creates a second interpolation model. Sifr uses template strings and library entry
points.

### One dialect-neutral analyzer

Database name resolution, casts, nullability, functions, and DDL differ materially.
One approximate analyzer produces false safety. Providers share protocols, not
semantics.

### Cardinality-selected containers

Using inferred cardinality to choose `Row`, `Option[Row]`, or `list[Row]` makes small
SQL changes alter public APIs. Explicit fetch methods keep application intent
stable.

### Arbitrary compiler plugins

Allowing packages to emit HIR, Rust, linker flags, or executable build steps breaks
the compiler safety boundary. Components return validated data through a closed
protocol.

### Live database compilation

Live compilation makes builds non-reproducible and requires credentials in build
environments. Tools update checked-in schema inputs. Normal compilation remains
offline.

### Automatic migration rollback

Many schema and data transformations are not reversible. Generated rollback can
destroy data while appearing safe. Reverse plans remain explicit and checked.

## Final invariant

Every executable SQL statement has one compile-time schema authority and one
exact dialect provider. It has a typed parameter plan, a typed structural result,
a bounded runtime path, and a verified schema contract.

No user-controlled SQL value becomes syntax by default. No compiler component can
inject arbitrary compiler output. No generated runtime SQL path can panic because
of database data, network data, or ordinary application input.
