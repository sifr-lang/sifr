# SQL migration compiler and engine

This document defines the provider-neutral migration compiler and execution
engine. Provider packages supply DDL reflection and database operations.

## Ownership

`sifr_sql_contract` owns migration graph compilation. It owns graph identities,
checksums, provider constraints, intermediate schema states, impact reports, and
explicit reverse plans.

`sifr_frontend` lowers each checked step to migration HIR. The HIR contains
compiler-generated nominal state types for `MigrationPlan[S]`. A Sifr data
callback receives `MigrationDb[S]` for the exact input state.

`sifr_sql_runtime` owns execution state. It owns locks, ledger records, step
progress, assertion outcomes, bounded backfill progress, and recovery errors.
Provider tools implement the runtime interface.

`sifr_sql_tool` converts the checked compiler graph to a closed execution plan.
The `MigrationExecutionPlan` contains only runtime-owned types. The
application runtime does not link `sifr_sql_contract` or compiler components.

## Graph contract

Each graph contains one or more named baselines and one explicit head. Every
migration declares these values:

- a stable identity.
- its complete parent set.
- one input fingerprint for each parent.
- one output fingerprint.
- a provider family, minimum server version, and required capabilities.
- ordered steps and a transaction requirement.
- optional explicit reverse steps.
- author and creation metadata.

The compiler uses a deterministic topological order. It rejects unknown parents,
cycles, disconnected baselines, duplicate identities, and multiple terminal
heads. A merge migration compiles once for each parent. All parent paths must
produce the same schema. Thus, a merge joins equivalent parent schemas. It does
not apply a different reconciliation program for each parent.

The canonical checked-in schema is the target. The only graph head must reproduce
its exact semantic graph and fingerprint. A migration history cannot become a
second schema authority.

## Intermediate states

The provider reflects each DDL statement against its input `SchemaIR`. The
compiler creates a nominal state identity from the migration, parent, direction,
step position, and resulting fingerprint.

If a provider cannot reflect a raw DDL statement, the step must declare an
explicit schema effect. An empty effect is an error. If reflection succeeds, a
declared effect must equal the reflected schema.

Each step records its input state, output state, input fingerprint, output
fingerprint, checksum, referenced objects, and affected objects. Data steps do
not change the schema fingerprint.

`MigrationPlan[S]` is affine. A checked transition consumes the old plan and
returns `MigrationPlan[N]`. The plan implements neither `Clone` nor `Copy`.

`MigrationDb[S]` has a private callback lifetime and a nominal state marker.
The compiler accepts only asynchronous, nonescaping callbacks that return a
`Result`. All declared objects must exist in state `S`.

## Checked steps

Typed SQL data steps must have a write or read-write effect. They must return no
rows. Their referenced and affected objects must exist in the input state.

An assertion must return one field named `valid`. This field must have the
non-null Boolean type. The execution engine reports false, zero-row, and
multiple-row results as different errors.

A backfill has a positive maximum row count for each batch. Resumable execution
requires an idempotent replay declaration and a nonempty progress key. The
runtime rejects a batch that exceeds its row bound or does not advance progress.

Explicit transaction boundaries cannot nest. A transaction-required migration
has one outer begin and commit pair. A non-transactional migration cannot contain
transaction boundaries.

Recovery point names are unique within a path. The ledger stores the last point
with the current schema fingerprint.

## Offline compilation

The compiler applies every path from its baseline without a database connection.
It validates these properties:

1. The provider family, server version, and capabilities satisfy each migration.
2. Each parent fingerprint equals the compiled parent schema.
3. Each step uses only objects from its input state.
4. Reflected and declared DDL effects are consistent.
5. Each path produces its declared output fingerprint.
6. An explicit reverse path reproduces its parent fingerprint.
7. The graph head equals the canonical target schema.

The output includes destructive object removals, provider lock risks, and data
rewrite objects. The compiler never creates a reverse plan.

## Execution and recovery

The compiler graph remains in the host tool process. The tool emits the closed
execution plan as `graph.json`. It emits schema and impact evidence in separate
files. This split keeps compiler analysis and Wasmtime out of application
runtime targets.

The engine acquires one provider lock before it reads the migration ledger. It
releases the lock on success and error paths. A provider panic becomes a typed
runtime error.

The ledger stores current heads, the live fingerprint, applied migration
checksums, step checksums, duration, recovery points, and backfill progress. The
engine compares the ledger fingerprint with an independent live observation
before it runs a step.

The engine rejects unknown heads, schema drift, changed applied checksums, and
an in-progress step that does not match its compiled state. It selects the first
valid path in the compiler's stable topological order. It does not guess a path
outside that checked order.

Before each step, the engine stores its in-progress record. After each step, it
stores the checksum, duration, output state, and fingerprint. A bounded run can
pause after the configured number of backfill batches. The next run resumes from
the stored progress key.

During an explicit transaction, the provider stores ledger changes in that same
transaction. An error causes an explicit rollback. The engine rejects a ledger
that claims an uncommitted transaction prefix. An operator must inspect and
repair that state before execution can continue.

The provider runtime returns the observed schema fingerprint for every step.
The engine accepts only the compiled output fingerprint.

## Verification

The permanent `migration-engine` SQL suite contains these checks:

- graph, fingerprint, reflection, rollback, assertion, callback, backfill, and
  transaction contract tests.
- frontend nominal-state HIR tests.
- execution, interruption, resume, drift, checksum, assertion, and panic tests.
- qualification-record mutation tests.
- graph and metadata property or fuzz-smoke tests.

PostgreSQL DDL reflection, advisory locks, live interruption, and server-major
qualification belong to Milestone 14. That milestone also owns execution of
explicit reverse plans. Final source-to-tool command wiring belongs to the
integrated closure milestone.
