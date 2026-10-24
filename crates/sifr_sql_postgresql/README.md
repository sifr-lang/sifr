# PostgreSQL parser builds

Select a native parser with one Cargo feature: `postgresql-13` through
`postgresql-18`. With no version feature, the default is PostgreSQL 18.
Multiple version features are an error. `SIFR_POSTGRESQL_MAJOR` is no longer
accepted as a build input; it remains compiler-emitted metadata for the adapter.

```sh
cargo test --locked -p sifr_sql_postgresql --features postgresql-13 --tests
cargo test --locked -p sifr_sql_postgresql_tools --features sifr_sql_postgresql/postgresql-13
```

Each feature selection has its own Cargo artifacts in the same target directory.
Compatible Rust dependencies remain shared. The parser matrix retains all six
versions and runs the component host checks separately. Guest producers use the
same explicit version selection and record their source/tool provenance.

Features select mutually exclusive native implementations, so an all-features
build is intentionally rejected. When a dependency graph requests two different
versions, resolve the conflict explicitly instead of choosing one silently.
