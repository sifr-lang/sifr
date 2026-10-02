# SQL runtime and tooling execution coverage

The SQL verification area owns the offline execution routes for these seven
packages. Their coverage-registry classification remains runtime or tooling,
with merge assignment. The strict compiler-only membership rule does not imply
that these packages lack execution: the merge profile selects the SQL adapter
suites below.

| Package | Selected suite / adapter command | Cargo test selection |
| --- | --- | --- |
| `sifr_sql_mysql_runtime` | `mysql-provider` / `sql-mysql-runtime-tests` | `--lib --test runtime_types` |
| `sifr_sql_postgresql_runtime` | `postgresql-runtime` / `sql-postgresql-runtime-tests` | `--lib --test runtime_types` |
| `sifr_sql_sqlite_runtime` | `sqlite-provider` / `sql-sqlite-runtime-tests` | Default package-test selection |
| `sifr_sql_mysql_tools` | `schema-tools` / `sql-schema-tool-tests` | Default package-test selection |
| `sifr_sql_postgresql_tools` | `schema-tools` / `sql-schema-tool-tests` | Default package-test selection |
| `sifr_sql_sqlite_tools` | `schema-tools` / `sql-schema-tool-tests` | Default package-test selection |
| `sifr_sql_tool` | `schema-tools` / `sql-schema-tool-tests` | Default package-test selection |

Every command uses `cargo test --locked` and selects its package explicitly.
The schema-tools command selects all four tooling packages together. Additional
MySQL migration and SQLite tool commands exercise narrower targets as well.
The adapter command definitions are authoritative in
[`runner.py`](../verification/areas/sql_platform/runner.py); the suites and
cases are declared in [`manifest.json`](../verification/areas/sql_platform/manifest.json).

The MySQL adapter previously selected only `runtime_types`, which excluded the
source-unit tests in `codec`, `control`, and `error`. Adding `--lib` restores
their execution while retaining the runtime type-contract test. This repairs
the bounded omission; no broad compiler/runtime classification or scheduler
policy change is needed.

## Live-server policy

Cargo's default test selection leaves ignored live-server cases unexecuted.
Offline profile coverage must not count those cases as passes. MySQL and
PostgreSQL runtime adapters explicitly select their lib/type targets; tooling
and SQLite adapters use default package selection, which still skips ignored
cases.

The separate MySQL live matrix is
[`run_mysql_server_matrix.py`](../verification/areas/sql_platform/tools/run_mysql_server_matrix.py).
It provisions the supported server series and passes `--include-ignored
--test-threads=1` to Cargo for its runtime and tooling surfaces. Running
`python3 verification/areas/sql_platform/tools/run_mysql_server_matrix.py
--surface runtime` selects the real runtime matrix. Its live prerequisites and
results are separate from this offline execution audit.

The PostgreSQL runtime matrix similarly selects its exact ignored live test
against PostgreSQL 13 through 18. SQLite runtime tests use local SQLite
databases; their execution does not establish MySQL or PostgreSQL live coverage.

## Item 5 qualification

The phase qualification runs the named seven-package Cargo test command and
the changed MySQL adapter command, records actual executed and ignored cases,
and validates profile selection and coverage readiness. Candidate-specific
results belong in the canonical SQL phase delivery receipt and evidence
outside the reviewed Git tree. No live-matrix pass is inferred from offline
tests or from a registry assignment.
