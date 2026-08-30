# SQL Query and Fragment Contracts

## Scope

This document defines the common query substrate for Sifr SQL. Dialect packages
still own parsing and SQL semantics.

`sifr_sql_contract` owns immutable query and fragment records. It also owns the
generated-name codec and the profile-module registry.

`sifr_frontend::SqlQueryCompiler` consumes the registry and provider analysis.
It lowers each accepted query to closed SQL HIR records.

`sifr_sql_runtime` owns the two runtime query states. It has no database driver
and no dependency on the compiler contract.

## Public query states

Sifr exposes only these query states:

- `QueryTemplate[Profile, Params, Row, Cardinality, Effect]`
- `BoundQuery[Profile, Row, Cardinality, Effect]`

`@profile.query` gives a top-level function one stable template identity. The
identity includes the module, symbol, source range, profile, and normalized SQL.

Calling the template captures the parameter expressions. The compiler keeps
the expressions in source order and evaluates each expression one time.

A bound query owns all captured values. Execution consumes the bound query.
The bound query supports `Clone` only when its generated capture type supports
`Clone`.

There is no public prepared-query state. A provider can prepare and cache a
statement inside a connection.

## Result types and `RowOf`

The frontend lowers a query result to a normal immutable structural record.
The field order does not change the record identity.

`RowOf[query]` accepts a top-level reusable query symbol. It rejects a local
function, a closure, or a runtime query value.

An exported query stores its canonical structural result type. A consumer can
use that type without another analysis of the query body.

## Binding

An ordinary interpolation value always creates a parameter slot. A runtime
`str` cannot create SQL syntax or a database identifier.

The provider supplies the exact database type and codec for each slot. The
common bind table decides whether the Sifr value is exact, fallible, or rejected.

The runtime can retain a typed capture until execution. Fallible encoding then
keeps the original error order.

The runtime also provides an ordered encoder for generated code. Each
`capture` call evaluates and owns one value before the next call starts.

## Fragment contract

The common contract has these fragment categories:

- expression, predicate, identifier, relation, order-by, and join
- select-list, assignment-list, values, and returning-list
- query and command

Each fragment records the profile, dialect, query identity, and syntax category.
It also records the input and output relation scopes.

Each relation alias has a hygienic identity. Textual equality does not give a
fragment access to an alias from a different query site.

A fragment also records these values:

- required and introduced aliases
- free schema identifiers
- ordered parameter slots
- result and effect transformations
- SQL precedence and canonical syntax
- an unsafe-syntax audit, when applicable

The compiler creates aliases and fragment identities only at static sites in a
query definition. Runtime branches, loops, containers, and returns cannot create
or retain these identities.

Fragment insertion requires an exact category, profile, dialect, and query
identity. The required scope must be a subset of the available scope.

## Predicate operations

The canonical predicate operations are `all`, `any`, and `not`.

`all([])` produces `TRUE`. `any([])` produces `FALSE`.

The operations preserve the source parameter order. They add parentheses only
when the child precedence requires them.

These operations support optional filters without SQL string assembly. Normal
Sifr branches can select typed fragments before the final composition.

## Cardinality operations

`expect_at_most_one()` keeps the SQL text. It adds a runtime check and narrows
the upper bound to one.

`first()` requires a provider-produced one-row plan. The frontend reports a
warning when the query has no deterministic order.

Cardinality never selects a result container. The caller selects `fetch_one`,
`fetch_optional`, bounded `fetch_all`, `stream`, or `execute`.

The HIR stores the cardinality and effect records. Execution lowering copies
the exact records into the runtime request.

## Unsafe syntax

Unsafe syntax requires the `sql.unsafe-syntax` security capability. It also
requires a package identity, an audit reason, and a non-deny lint policy.

The root package grants the capability with
`trust.security-capabilities = ["sql.unsafe-syntax"]`. `sifr_package` resolves
the grant for the exact package identity. The fragment constructor consumes
that resolver directly.

The unsafe path still creates a typed, static fragment. It cannot turn runtime
text into syntax through an ordinary interpolation.

The default lint policy is `deny`. A qualified package can select `warn` for an
explicit audited site.

## Generated names

Generated schema names use one reversible codec. Safe names stay readable.
The codec escapes a segment that starts or ends with an underscore. Thus, a
boundary underscore cannot combine with the double-underscore path separator.

The codec escapes these names with a reserved prefix:

- a Sifr keyword
- a name that contains the path separator `__`
- a name that starts with an underscore
- a name that ends with an underscore
- a name that starts with the reserved generated prefix

The escape contains the exact UTF-8 bytes in lowercase hexadecimal form. The
decoder rejects a noncanonical spelling.

This rule prevents collisions between schema paths, keywords, enum variants,
and composite fields. The result does not depend on schema order.

## Profile registry

`ProfileModuleRegistry` is the production authority for generated profile
modules. It indexes each entry by profile name, module path, and nominal identity.

Registration compares the complete module metadata with the profile authority.
It rejects a duplicate name, path, or nominal identity.

The driver builds this registry during package preparation. The frontend query
compiler resolves every query profile through the same registry.

The build cache uses the registry fingerprints. It does not parse a cache string
to reconstruct query authority.

## Verification

The `query-fragments` SQL verification suite covers these mechanisms:

- generated-name collision and round-trip properties
- profile-registry identity checks and a production frontend consumer
- top-level `RowOf` acceptance and local-symbol rejection
- fragment category, profile, scope, alias, and precedence checks
- unsafe-capability and runtime-text rejection
- source-order evaluation, owned captures, and failure timing
- conditional `Clone` and consuming execution lowering
- cardinality and effect round trips from compiler HIR to runtime requests

The qualification record is
`verification/areas/sql_platform/data/query_substrate_qualification.json`.
