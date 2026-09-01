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

`sifr_sql_tool` loads source files and converts the checked compiler graph to a
closed execution plan. It receives checked declarations through a callback.
Thus, this crate does not depend on `sifr_driver` in production.

Each provider tool invokes `sifr_driver` for source checking. The provider tool
then passes the checked declarations to `sifr_sql_tool`.

The `MigrationExecutionPlan` contains only runtime-owned types. The application
runtime does not link `sifr_sql_contract` or compiler components.

## Source layout and build command

Migration declarations are ordinary checked Sifr source files. A profile uses
this project layout:

```text
migrations/
  app/
    2026_08_add_status.sifr
    baselines/
      2026_07_previous.json
.sifr/
  sql-migrations/
    app/                         # generated; never an authoring location
```

Each source filename must equal its migration `id`. Each baseline filename must
equal its baseline `id`. Each source file declares exactly one migration.

The profile name is one normalized path segment. Source files, baseline files,
and their directories must be real files or directories. Symlinks fail before
artifact publication.

Duplicate identities, unknown parents, cycles, and disconnected graphs also
fail before artifact publication.

All providers expose the same offline command:

```text
sifr sql migration build --profile app
```

The command loads the selected profile authority, type-checks every `.sifr`
source, uses that provider's exact parser and analyzer for SQL steps, reflects
DDL through the provider migration dialect, compiles the complete graph, and
atomically writes `graph.json`, `schema.json`, `impact.json`, and
`artifact-manifest.json` under `.sifr/sql-migrations/app/`.

The source declaration surface is concise and deterministic:

```sifr
from sifr.sql.migration import MigrationPlan, MigrationState, migration, rollback

@migration(
    id="2026_08_add_status",
    parents=["2026_07_previous"],
    author="Application team",
    created_at="2026-08-31T00:00:00Z",
)
def add_status[S: MigrationState](own plan: MigrationPlan[S]) -> MigrationPlan[S]:
    changed = plan.ddl(t"ALTER TABLE orders ADD COLUMN status TEXT")
    filled = changed.sql_step(
        t"UPDATE orders SET status = 'pending' WHERE status IS NULL"
    )
    checked = filled.assert_sql(
        t"SELECT bool_and(status IS NOT NULL) AS valid FROM orders"
    )
    return checked.ddl(t"ALTER TABLE orders ALTER COLUMN status SET NOT NULL")

@rollback(of="2026_08_add_status")
def remove_status[S: MigrationState](own plan: MigrationPlan[S]) -> MigrationPlan[S]:
    return plan.ddl(t"ALTER TABLE orders DROP COLUMN status")
```

Migration SQL is static at graph-build time. Exact source text is retained for
execution; provider normalization is retained separately for semantics and
fingerprints. Dynamic interpolation is rejected.
`@rollback` is optional. When present, it names exactly one migration and the
compiler proves that its explicit reverse steps reproduce every parent schema.
No reverse SQL is synthesized.

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

Version comparison uses semantic versions. User-facing database versions can
omit trailing zero fields: `18` becomes `18.0.0`, and `8.4` becomes `8.4.0`.
Other malformed versions fail before migration compilation.

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

The source-level `plan.ddl` operation accepts only DDL that the selected
provider can reflect. The compiler rejects opaque DDL. It does not accept an
unchecked schema-effect fallback.

The low-level graph contract can carry a provider-owned declared effect. If
reflection also succeeds, that effect must equal the reflected schema.

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

[`sql_postgresql_migrations.md`](./sql_postgresql_migrations.md) defines the
PostgreSQL DDL, advisory lock, ledger, import, command, recovery, rollback, and
PostgreSQL 13 through 18 qualification contracts. MySQL and SQLite use the same
source loader, graph compiler, artifact transaction, and command surface with
their own exact parser, analyzer, reflection, and runtime implementations.
