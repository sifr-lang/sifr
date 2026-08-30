# Ad hoc phase: Schema-first SQL platform

Status: active

Baseline commit: `90762c7872de0de05f760ac91f95463d9c679d59`

## Objective

Deliver the complete schema-first SQL architecture in
[`internal_docs/sql_architecture.md`](../../../internal_docs/sql_architecture.md).

The result includes the compiler component platform, structural records, schema
profiles, checked SQL, explicit execution, and verified runtime pools. It also
includes editor support, tool packages, migrations, and three qualified dialects.

This phase has no reduced product tier. Milestones provide implementation order,
not permanent scope cuts. The phase closes only after the full design works as
one coherent system.

## Exit state

The phase is complete when all these conditions are true:

- Sifr preserves template strings, typed holes, evaluation order, and source maps
  through parsing, typing, HIR, code generation, formatting, and editor queries.
- Immutable structural records have canonical identity, ownership rules, width
  subtyping, generated layouts, and complete type-system integration.
- Sandboxed compiler components provide deterministic embedded-language analysis
  through one closed protocol.
- Named schema profiles produce canonical provider-owned `SchemaIR`, exact
  fingerprints, generated modules, and runtime verification contracts.
- Typed queries and fragments prevent SQL injection unless code uses the explicit
  unsafe capability.
- Public execution APIs use verified pools, explicit fetch methods, bounded
  resources, safe cancellation, and panic-free protocol handling.
- PostgreSQL, MySQL, and SQLite satisfy their complete compiler, runtime, tool,
  migration, editor, and conformance contracts.
- Host-only SQL tools stay outside target application artifacts.
- Typed migration graphs prove every supported path to the canonical schema.
- Public documentation, examples, language-server behavior, and diagnostics match
  the implemented contracts.
- Every capability and verification row has an owner, executable evidence, and no
  pending waiver or fallback.

## Reason for this phase

Sifr can call database crates through Rust and Python interoperability. Those
paths do not provide schema-first compilation or Sifr-owned SQL semantics.

A generic SQL parser cannot model exact provider grammar, typing, catalogs,
nullability, migrations, and runtime behavior. Separate ad hoc integrations also
cannot enforce one ownership and safety model.

This phase builds one coherent platform. It keeps provider semantics separate
while it shares compiler, package, runtime, tool, editor, and verification
infrastructure.

## Source evidence

The planning audit found these starting conditions:

| Area | Current evidence | Required disposition |
| --- | --- | --- |
| First-party SQL | Only this phase and `internal_docs/sql_architecture.md` define the first-party platform. | Implement every owned compiler, package, runtime, tool, editor, and verification surface. |
| Rust interoperability | `sifr_rust_interop_catalog` certifies SQLx, Tokio PostgreSQL, and Rusqlite use. | Keep interoperability independent from first-party SQL authority. |
| Python interoperability | Python fixtures exercise SQLAlchemy, Psycopg, Aiosqlite, and SQLite contexts. | Keep these external integrations without treating them as Sifr SQL providers. |
| Compiler pipeline | Existing frontend, HIR, lowering, code generation, diagnostics, package, and LSP crates provide integration points. | Extend their owned contracts without placing dialect semantics in compiler core crates. |
| Runtime substrate | `sifr_runtime` owns package-neutral async, cancellation, resource, and TLS behavior. | Add `sifr_sql_runtime` above this substrate and keep raw drivers in provider packages. |
| SQL compiler components | No `sifr_compiler_component` or `sifr_sql_contract` implementation exists. | Build the closed component host and protocol before a dialect provider. |
| Schema authority | No canonical SQL `SchemaIR`, profile, capability matrix, or dependency index exists. | Define and machine-validate one provider-owned schema authority. |
| Tooling and migrations | No first-party SQL tool namespace or typed migration engine exists. | Add host-only tools and a Sifr-owned migration model. |

Primary evidence paths:

- `internal_docs/sql_architecture.md`
- `internal_docs/architecture.md`
- `internal_docs/dependency_policy.md`
- `crates/sifr_rust_interop_catalog/`
- `crates/sifr_frontend/`
- `crates/sifr_package/`
- `crates/sifr_runtime/`
- `crates/sifr_lsp/`
- `verification/areas/rust_interop/`
- `verification/areas/python_interop/`

## Definitions

This phase uses these terms:

- **Schema profile:** A named compile-time identity that selects one provider,
  schema source, verification policy, strictness policy, and session contract.
- **`SchemaIR`:** The immutable provider-owned graph of every database object that
  can affect supported SQL semantics.
- **Compiler component:** A deterministic WebAssembly component that parses and
  analyzes one embedded language through the compiler-owned closed protocol.
- **Provider:** The package family that owns one dialect parser, analyzer, schema
  normalizer, runtime bridge, tools, migrations, and qualification evidence.
- **Capability matrix:** The machine-readable list of supported provider grammar,
  schema, runtime, tool, migration, and editor behavior.
- **Query template:** A reusable typed SQL definition with static identity and
  unbound typed holes.
- **Bound query:** An owned executable query that contains encoded parameter values.
- **Verified pool:** A pool handle that carries current schema evidence and the
  selected strictness contract.
- **Qualification record:** Machine-readable evidence for dependencies, supported
  versions, features, conformance, safety, and performance.
- **Focused validation:** The smallest complete verification set for one changed
  ownership surface.
- **Closure evidence:** The exact commits, commands, results, review, pull request,
  and merge identity that prove one milestone is complete.

## Source of truth

- this phase record for order, ownership, acceptance, and closure
- [`internal_docs/sql_architecture.md`](../../../internal_docs/sql_architecture.md)
  for the permanent architecture and user contract
- [`internal_docs/architecture.md`](../../../internal_docs/architecture.md) for
  repository-wide component ownership and compiler flow
- provider qualification records and machine-validated capability manifests
- compiler, runtime, tool, editor, and verification sources delivered by this
  phase

If this record and the SQL architecture conflict, implementation stops. The
same milestone must update both records and explain the resolved contract.

## Locked delivery contract

1. Sifr uses PEP 750-style template strings through library entry points. It
   does not add custom SQL literal syntax.
2. Normal compilation is offline, deterministic, and schema-first.
3. Ordinary queries bind to one named schema profile at definition.
4. SQL values become parameters by default. Syntax composition requires typed
   fragments or an explicit unsafe capability.
5. Query results use immutable structural records with canonical identity and
   width subtyping.
6. Query templates and bound queries are distinct types. Statement preparation
   stays inside the connection cache.
7. Callers select explicit fetch or execution methods. Inferred cardinality
   does not select public containers.
8. Runtime execution requires verified schema evidence and schema strictness.
9. Compiler components return typed data through a closed protocol. They cannot
   emit arbitrary HIR, Rust, linker arguments, or executable build steps.
10. The canonical schema defines the target state. The migration graph proves
    how supported starting states reach it.
11. PostgreSQL, MySQL, and SQLite use exact provider semantics. There is no
    approximate universal SQL analyzer.
12. Database tools execute through direct package namespaces, such as `sifr sql`.
    Cargo resolves a separate host-only tools workspace member.
13. Generated runtime paths do not panic because of database data, network
    data, malformed metadata, or ordinary application input.
14. No milestone adds backward compatibility, a silent fallback, or a temporary
    public API that contradicts the final architecture.
15. SQL ships as first-party Cargo-backed Sifr packages, not as core standard
    library modules. Provider packages select parsers, drivers, and tools.
16. Sifr owns pooling, schema verification, sessions, statement caches,
    cancellation cleanup, migrations, and public codecs above raw drivers.
17. Compiler and editor semantics use one strict provider parser. A generic SQL
    parser cannot become a second semantic authority.
18. Milestone 0 resolves and locks the latest stable compatible tooling set.
    Every later milestone uses that exact baseline until an approved upgrade.

## Retained external and adjacent contracts

The phase does not replace these supported contracts:

- Rust interoperability for direct SQLx, Tokio PostgreSQL, Rusqlite, and other
  certified crate use
- Python interoperability for SQLAlchemy, Psycopg, Aiosqlite, SQLite, and other
  declared Python packages
- PostgreSQL, MySQL, and SQLite wire, file, catalog, and authentication protocols
- Cargo dependency resolution, semantic-version rules, lockfiles, and metadata
- package-neutral async, TLS, cancellation, time, network, file, and resource APIs
- external database administration tools that users run outside `sifr sql`
- third-party compiler components that obey the same closed protocol and sandbox

These contracts cannot become a fallback for first-party SQL semantics. They
remain separate supported integration paths.

## Scope

### In scope

- template-string parsing, type checking, lowering, source maps, and formatting
- immutable structural record types and Rust layout interning
- deterministic compiler component protocol, sandbox, package registration,
  caching, diagnostics, and qualification
- named SQL profiles, canonical `SchemaIR`, fingerprints, generated schema
  modules, and schema-polymorphic requirements
- typed query templates, owned binding, fragments, cardinality,
  effects, nullability, codecs, and errors
- PostgreSQL, MySQL, and SQLite compiler providers and runtime providers
- verified pools, connections, transactions, streams, cancellation, deadlines,
  resource bounds, and prepared-statement caches
- embedded SQL language-server support
- Cargo-resolved host-only tools and direct command namespaces
- schema pull, validate, and build tools
- migration graph compilation, intermediate schema states, typed data steps,
  assertions, recovery, import, and baselines
- security, conformance, fuzzing, property tests, performance budgets, examples,
  and complete documentation

### Permanent exclusions

- an ORM, active-record layer, or generated table-model API
- live database access during normal compilation
- interpolation that treats ordinary strings as SQL syntax
- arbitrary compiler plugins or package-supplied Rust generation
- inferred fetch-container selection
- unbounded row collection
- synthesized destructive rollback
- a second tool-runner executable
- hand-maintained dialect forks of common compiler or runtime infrastructure
- provider claims without executable conformance evidence

## Ownership and coordination boundaries

Each milestone owns only the surfaces in its owned-scope list. A milestone must
record coordination before it edits a surface with another active owner.

The most likely shared surfaces are:

- template-string syntax, structural records, and type-system rules
- package manifests, Cargo graphs, and compiler component registration
- runtime cancellation, TLS, tasks, resources, and panic-safety infrastructure
- language-server caches, source maps, and virtual documents
- verification profiles, release gates, and dependency policy
- Rust and Python interoperability catalogs and qualification fixtures

An SQL milestone extends these shared contracts only for its approved architecture.
It cannot absorb unrelated cleanup or change another feature family.

## External prerequisites

The SQL runtime depends on the accepted async context-manager and iterator
contracts. Their implementation track remains outside this phase.

Before Milestone 9 starts, that track must merge these capabilities:

- abnormal body-exit cleanup for `async with`
- cancellation-specific `AsyncExitCause` values
- secondary cleanup evidence, including cleanup timeout
- `AsyncClosable.aclose()` on early `async for` exit

Milestone 0 must record the owning issue, owner, and merged evidence in the
verification inventory. This phase does not reimplement those capabilities.

## Execution rules

1. Execute one milestone at a time in the exact order below.
2. Start a milestone only after every earlier milestone is merged and this
   record names its evidence.
3. Rebase the milestone branch point on current `origin/main` before work starts.
4. Use the phase-closure loop for every milestone.
5. Implement the complete acceptance contract before validation begins.
6. Run focused tests during implementation. Then run the milestone qualification
   suite on the final candidate.
7. Run one create-PR gate for a compiler-changing milestone and one merge gate
   for its final candidate. Documentation-only milestones use relevant document
   validation.
8. Obtain one exact-candidate external review. Apply valid blockers in one batch.
   Run one remediation review after relevant implementation changes.
9. Merge the milestone, update this record, update the active issue, and then
   start the next milestone.
10. Record an unrelated failure in its owning issue. Do not broaden this phase
    to absorb it.
11. Update `internal_docs/architecture.md` when component boundaries change.
12. Update `plans/roadmap.md` only when phase or milestone status changes.
13. Regenerate derived schema, protocol, and provider artifacts from their
    authoritative producers. Do not hand-edit derived output.
14. The final milestone runs the only whole-phase review and whole-phase
    repository merge gate. This gate is separate from each milestone merge
    gate in rule 7.

## Sequential milestones

| Milestone | Status | Name | Required outcome |
|---:|---|---|---|
| 0 | completed | Architecture and dependency lock | The final architecture, language dependencies, ownership map, capability matrix, verification inventory, and phase gates are authoritative and machine validated. |
| 1 | in progress | Template-string language foundation | Template strings preserve static segments, typed holes, evaluation order, and exact source maps through the full compiler pipeline. |
| 2 | pending | Structural record type system | Immutable records have order-independent canonical identity, width subtyping, deterministic diagnostics, and interned Rust layouts. |
| 3 | pending | Compiler component platform | Resolved packages can provide deterministic sandboxed embedded-language analysis through one closed, versioned, cacheable protocol. |
| 4 | pending | Schema profiles and canonical `SchemaIR` | Configuration sources produce exact provider-owned schema graphs, nominal profile types, fingerprints, dependency slices, diffs, and runtime contracts. |
| 5 | pending | Common SQL contracts | Shared query kinds, complete type and bind mappings, codecs, errors, cardinality, effects, ownership, and provider interfaces have one final contract. |
| 6 | pending | Query and fragment substrate | Query templates, owned bound queries, typed fragments, composition, safe interpolation, and cardinality adapters integrate with Sifr typing and HIR. |
| 7 | pending | PostgreSQL schema and query compiler | PostgreSQL catalogs, grammar, resolution, typing, nullability, result records, writes, dependencies, and diagnostics work offline. |
| 8 | pending | PostgreSQL semantic completion | Advanced PostgreSQL constructs, fragment scope changes, cardinality proofs, custom codecs, and exported-query stability rules are complete. |
| 9 | pending | PostgreSQL runtime | Verified pools, session contracts, transactions, streaming, automatic statement caching, explicit fetch methods, bounded cleanup, tests, and panic-safe protocol handling are complete. |
| 10 | pending | Incremental compiler and editor experience | Fine-grained caching, invalidation, virtual SQL documents, source maps, completion, navigation, rename, formatting, and quick fixes are complete. |
| 11 | pending | Host tool graph and command runner | Cargo-locked host-only tool packages execute direct command namespaces without entering application code generation. |
| 12 | pending | Schema lifecycle tools | Pull, validate, and build commands produce deterministic snapshots, fingerprints, manifests, modules, semantic diffs, and affected-query reports. |
| 13 | pending | Migration compiler and engine | Typed migration DAGs, intermediate schemas, DDL reflection, data steps, assertions, offline validation, recovery, and explicit rollback are complete. |
| 14 | pending | PostgreSQL migration qualification | PostgreSQL DDL, locks, transactional limits, imports, baselines, recovery, and supported-version execution pass full migration qualification. |
| 15 | pending | Schema polymorphism and portable constraints | Structural schema requirements specialize safely, while explicit capability constraints validate portable code for every declared provider. |
| 16 | pending | MySQL provider completion | MySQL query, schema, runtime, tooling, migration, editor, safety, and conformance surfaces satisfy the common and provider-specific contracts. |
| 17 | pending | SQLite provider completion | SQLite query, schema, runtime, tooling, migration, editor, safety, and conformance surfaces satisfy the common and provider-specific contracts. |
| 18 | pending | Integrated qualification and phase closure | All providers, tools, migrations, compiler paths, runtime paths, editor paths, security gates, budgets, examples, and documents pass as one final system. |

## Milestone acceptance contracts

### Milestone 0: Architecture and dependency lock

ID: `sql_0_contract_lock`

Purpose: Lock every contract, owner, dependency, capability, and gate before
implementation starts.

Owned scope:

- architecture and repository ownership records
- dependency baseline and qualification schema
- provider capability matrix and verification inventory
- machine checks for ownership, identity, acceptance, and gate completeness

Acceptance criteria:

- [ ] The SQL architecture contains every locked delivery contract from this
  record and has no version-scoped deferral.
- [ ] Structural records, fixed-width integers, canonical temporal types, network
  address value types, replay-safe callbacks, bounded cancellation cleanup, and
  diagnostic registries have approved language contracts before SQL work begins.
- [ ] The verification inventory names the external issue, owner, and merge
  evidence for abnormal async cleanup and iterator close behavior before
  Milestone 9 starts.
- [ ] A machine-readable ownership map assigns each architecture surface to one
  milestone and one repository owner.
- [ ] A dependency qualification manifest locks parser sources, runtime crates,
  TLS adapters, feature sets, licenses, checksums, target support, and audits.
- [ ] `verification/areas/sql_platform/dependency_baseline.toml` records the
  latest stable compatible version from every documented release authority.
- [ ] The baseline generator excludes prereleases and yanked releases. It rejects
  incompatible release families, broad version ranges, and unlocked Git sources.
- [ ] The generated baseline matches the dated architecture table or updates it
  in the same change. The root lockfile resolves every selected crate exactly.
- [ ] Package metadata separates public Sifr APIs, WebAssembly components,
  runtime bridges, and host-only tools in the resolved graph.
- [ ] A capability matrix lists required PostgreSQL, MySQL, and SQLite grammar,
  schema, runtime, tool, migration, and editor behavior.
- [ ] A verification inventory maps every locked invariant to positive,
  negative, mutation, integration, fuzz, property, or performance evidence.
- [ ] Checkers reject a missing owner, missing acceptance mapping, duplicate
  identity, invalid milestone, unsupported provider claim, and empty gate.
- [ ] Checkers reject a missing milestone ID, purpose, owned scope, acceptance
  list, focused validation list, progress row, or synchronized status.
- [ ] Repository architecture and roadmap links resolve to this record and the
  SQL architecture.

Focused validation:

- dependency-baseline resolver and mutation self-tests
- ownership, capability, and verification inventory checks
- documentation structure and local-link checks
- `git diff --check` and the file-size guardrail

### Milestone 1: Template-string language foundation

ID: `sql_1_template_strings`

Purpose: Preserve typed template-string structure through the complete compiler
and editor pipeline.

Owned scope:

- parser, AST, spans, escapes, and evaluation order
- frontend typing, HIR, lowering, and code generation
- formatter and source-map behavior

Acceptance criteria:

- [ ] The parser represents static segments and expression holes without
  lowering them to string concatenation.
- [ ] Every hole preserves its Sifr span, virtual-document span, and left-to-right
  single-evaluation order.
- [ ] Type checking supports library APIs that consume typed template strings.
- [ ] HIR and code generation preserve static text and typed-hole metadata.
- [ ] Formatting preserves meaning, indentation, escapes, and hole boundaries.
- [ ] Compile-pass, compile-fail, snapshot, parser fuzz, and source-map property
  tests cover single-line and multiline forms.

Focused validation:

- syntax snapshots and parser recovery tests
- frontend, HIR, lowering, and code-generation tests
- formatter stability and bidirectional source-map properties
- parser fuzzing and focused native e2e fixtures

### Milestone 2: Structural record type system

ID: `sql_2_structural_records`

Purpose: Add one SQL-independent immutable structural record model to the Sifr
type system.

Owned scope:

- type identity, canonicalization, subtyping, and ownership
- HIR representation and generated Rust layout interning
- equality, hashing, display, serialization, and diagnostics

Acceptance criteria:

- [ ] Structural records are immutable, named, and independent of SQL. Field
  order does not affect type identity.
- [ ] Canonical identity is stable across modules and build order.
- [ ] Width subtyping applies only at borrow-only call boundaries.
- [ ] Owned projection consumes its source and never clones fields implicitly.
- [ ] Exact equality, invariant containers, union formation, branching, generics,
  and diagnostics follow one documented rule set.
- [ ] Field access preserves exact types and nullability.
- [ ] Code generation interns one Rust layout for each canonical record identity.
- [ ] ABI, ownership, equality, hashing, display, serialization hooks, and nested
  records have explicit behavior.
- [ ] Property tests cover canonicalization, field order, subtyping, and layout
  reuse.

Focused validation:

- type-system and structural-identity unit tests
- positive and negative ownership and subtyping fixtures
- code-generation layout snapshots
- canonicalization and layout-reuse property tests

### Milestone 3: Compiler component platform

ID: `sql_3_compiler_components`

Purpose: Provide one deterministic and sandboxed protocol for embedded-language
analysis.

Owned scope:

- WIT protocol, component host, package registration, and cache transport
- capability denial, resource bounds, diagnostics, and version negotiation
- malformed component and non-SQL qualification fixtures

Acceptance criteria:

- [ ] The package graph resolves compiler components by exact identity, version,
  hash, and protocol range.
- [ ] Components use the WebAssembly Component Model and the compiler-owned WIT
  interface without default WASI capabilities.
- [ ] The protocol accepts static source, typed holes, context, and source maps.
- [ ] It returns diagnostics, dependencies, type descriptors, semantic plans,
  and runtime-lowering descriptors from a closed schema.
- [ ] The sandbox denies ambient file, network, clock, random, environment,
  process, thread, shared-memory, native-library, linker, Rust-source, and
  arbitrary-HIR access.
- [ ] Official and third-party providers use the same component ABI and validation
  path.
- [ ] CPU, memory, recursion, input, output, and diagnostic bounds fail with
  structured compiler errors.
- [ ] Compiler and provider diagnostic code registries have stable, disjoint
  namespaces with machine-validated uniqueness and lifecycle metadata.
- [ ] Compiler and components declare compatible protocol ranges. Incompatible
  ranges fail without protocol downgrade.
- [ ] Cache keys include all semantic inputs and component identities.
- [ ] A non-SQL fixture proves parsing, typed holes, diagnostics, source maps,
  dependencies, caching, determinism, and malformed-output rejection.

Focused validation:

- protocol round-trip, version, and malformed-envelope tests
- sandbox capability and resource-limit mutation tests
- cache determinism and invalidation properties
- non-SQL component qualification on every supported host target

### Milestone 4: Schema profiles and canonical `SchemaIR`

ID: `sql_4_schema_profiles`

Purpose: Establish one exact compile-time schema authority for each named profile.

Owned scope:

- profile configuration and nominal identities
- provider schema normalization, fingerprints, slices, and semantic diffs
- generated schema modules and runtime verification manifests

Acceptance criteria:

- [ ] `sifr.toml` supports named profiles, exact providers, checked-in sources,
  schema evidence, schema strictness, and session contracts.
- [ ] SQL DDL, provider metadata, and generated definitions normalize into one
  immutable provider-owned `SchemaIR`.
- [ ] The IR represents every object that can affect provider query semantics.
- [ ] Canonical fingerprints are stable across irrelevant input order and reject
  semantic drift.
- [ ] Generated Sifr modules expose profile namespaces, nominal profile types,
  schema identities, and metadata without an ORM or runtime profile value.
- [ ] Compiler-known profile exports cannot collide with generated schema symbols.
  Static `profile.symbol("qualified.name")` lookup reaches a colliding object.
- [ ] Object-level semantic diffs and minimum referenced schema slices are exact.
- [ ] Credentials and live connections are absent from normal compilation.

Focused validation:

- profile parsing and package-resolution tests
- canonical fingerprint and irrelevant-order properties
- provider schema snapshot and semantic-diff tests
- reserved-export collision and static-symbol lookup fixtures
- offline-build and credential-leakage negative tests

### Milestone 5: Common SQL contracts

ID: `sql_5_common_contracts`

Purpose: Define every shared type, codec, error, effect, cardinality, and ownership
contract before query implementation.

Owned scope:

- provider-neutral SQL types and exact bind mappings
- codec, error, cardinality, and effect models
- pool, connection, transaction, query, and stream protocols

Acceptance criteria:

- [ ] Fixed-width SQL integers map to exact Sifr widths. Generic integer binding
  uses checked narrowing.
- [ ] Decimal, floating-point, temporal, text, binary, UUID, JSON, enum, array,
  domain, composite, range, IP, network, MAC, custom, unsigned, and SQLite
  affinity rules have explicit provider contracts.
- [ ] Every supported value pair has an exact bind-compatibility rule. Width
  mismatch, nullability, arrays, custom codecs, and generic integers fail or
  convert according to that table.
- [ ] Compile-time and runtime error families are structured, stable, redacted,
  and panic-safe.
- [ ] Cardinality uses the complete interval lattice and never selects containers.
- [ ] Read and write effects identify referenced and affected schema objects.
- [ ] Provider interfaces separate shared execution shape from dialect semantics.
- [ ] Query, pool, connection, transaction, and stream ownership protocols have
  explicit transfer, sharing, and lifetime rules.

Focused validation:

- machine validation for the complete type and bind matrices
- codec round-trip and malformed-value properties
- error redaction and panic-safety tests
- ownership protocol compile-pass and compile-fail fixtures

### Milestone 6: Query and fragment substrate

ID: `sql_6_queries_fragments`

Purpose: Integrate safe reusable queries and typed syntax fragments with normal
Sifr typing and ownership.

Owned scope:

- `QueryTemplate`, `BoundQuery`, `RowOf`, binding, and execution lowering
- fragment categories, scopes, aliases, parameters, effects, and precedence
- cardinality adapters and unsafe syntax capability

Acceptance criteria:

- [ ] `QueryTemplate` and `BoundQuery` are the only public query states.
- [ ] `@profile.query` creates a callable reusable template with a statically
  unique template identity and supports `RowOf`.
- [ ] Binding evaluates each expression once, left to right, and owns encoded
  values after construction.
- [ ] Execution consumes a bound query. Clone support depends on every captured
  value.
- [ ] Ordinary values always become parameters with exact provider type checks.
- [ ] Fragments carry profile, dialect, category, relation scope, aliases,
  parameters, result transformation, effect transformation, and precedence.
- [ ] Relation aliases and fragment identities are static inside a query
  definition. They cannot escape, depend on runtime control flow, or enter a
  runtime container.
- [ ] `RowOf` accepts only a top-level reusable query symbol. Exported signatures
  resolve it to a stable structural type alias.
- [ ] Canonical predicate combinators cover optional filters without string
  assembly.
- [ ] `expect_at_most_one()` narrows cardinality by runtime validation.
- [ ] `first()` is explicit and warns when ordering is not deterministic.
- [ ] Branching and generic code unify changing query and structural record types
  through normal Sifr typing.
- [ ] Unsafe syntax escape requires the complete security capability and lint
  contract.

Focused validation:

- query-state and ownership compile-pass and compile-fail fixtures
- evaluation-order, binding, and encoded-ownership tests
- fragment scope, precedence, alias, and composition properties
- injection, unsafe-capability, and cardinality diagnostics

### Milestone 7: PostgreSQL schema and query compiler

ID: `sql_7_postgresql_compiler`

Purpose: Implement exact offline PostgreSQL schema and core query semantics.

Owned scope:

- versioned PostgreSQL parser sources and provider-owned syntax nodes
- catalog normalization, resolution, typing, nullability, writes, and diagnostics
- provider dependency tracking and differential server evidence

Acceptance criteria:

- [ ] The provider uses the Milestone 0 `libpg_query` tag for each supported
  PostgreSQL major. The component manifest records every source checksum.
- [ ] The provider embeds tagged `libpg_query` source for each supported server
  major and maps its raw tree into provider-owned syntax nodes.
- [ ] Catalog ingestion and DDL sources normalize every supported PostgreSQL
  object into `SchemaIR`.
- [ ] The provider implements PostgreSQL parsing, name resolution, casts,
  operators, functions, aggregates, aliases, correlations, and set operations.
- [ ] Parameter inference produces one exact database type and codec per hole.
- [ ] Result inference produces unique named structural fields with conservative
  and PostgreSQL-correct nullability.
- [ ] Write analysis covers required values, generated columns, conflict clauses,
  effects, and `RETURNING`.
- [ ] Diagnostics map to Sifr, virtual SQL, and schema spans.
- [ ] Differential tests validate behavior against every supported PostgreSQL
  server version.

Focused validation:

- parser differential suites for every supported PostgreSQL major
- catalog and DDL normalization snapshots
- semantic positive, negative, and source-map fixtures
- live server type, nullability, write, and diagnostic comparisons

### Milestone 8: PostgreSQL semantic completion

ID: `sql_8_postgresql_semantics`

Purpose: Complete advanced PostgreSQL semantics and public query stability rules.

Owned scope:

- advanced types, expressions, queries, DDL, and locking
- cardinality, nullability, fragment scope changes, and custom codecs
- exported query signatures and `SELECT *` policy

Acceptance criteria:

- [ ] Arrays, ranges, composite types, domains, enums, JSON operations, windows,
  common table expressions, locking, and PostgreSQL-specific DDL are complete.
- [ ] Outer joins, aggregates, `CASE`, scalar subqueries, and provider functions
  produce exact nullability facts.
- [ ] Unique predicates, limits, aggregates, writes, and set operations produce
  sound cardinality intervals.
- [ ] Join and select-list fragments transform relation scope and result records
  without losing static typing.
- [ ] Values and assignment fragments cover bounded batches, dynamic updates,
  conflict behavior, provider parameter limits, and explicit chunking semantics.
- [ ] Custom codecs have one declared database identity and checked encode/decode
  behavior.
- [ ] Exported `SELECT *`, unstable names, duplicate names, and schema-sensitive
  public types have final lint behavior and machine-applicable fixes.
- [ ] The compiler expands accepted private `SELECT *` forms to explicit emitted
  columns. Exported `SELECT *` is always an error.
- [ ] Application `sifr build` emits query-signature artifacts for package API
  compatibility checks.

Focused validation:

- advanced PostgreSQL differential and conformance suites
- cardinality and nullability property tests
- fragment scope and custom-codec compile fixtures
- exported-signature snapshots and compatibility comparisons

### Milestone 9: PostgreSQL runtime

ID: `sql_9_postgresql_runtime`

Purpose: Implement verified and bounded PostgreSQL execution without public driver
types or user-triggered panics.

Owned scope:

- raw driver bridge, TLS, codecs, pooling, verification, and session reset
- transactions, streams, statement caches, cancellation, and cleanup
- live runtime qualification, malformed protocol handling, and resource budgets

Acceptance criteria:

- [ ] Runtime manifests use the exact Milestone 0 versions of Tokio, Rustls,
  `tokio-postgres`, `postgres-types`, and `tokio-postgres-rustls`.
- [ ] The runtime uses raw `tokio-postgres`, `postgres-types`, and
  `tokio-postgres-rustls` clients with an explicit feature allowlist.
- [ ] `sifr_sql_runtime` owns pooling, verified leases, session reset,
  statement-cache policy, cancellation budgets, and resource accounting.
- [ ] Pool verification combines one evidence mode with one strictness mode.
- [ ] Compatible verification compares every recorded property and absence fact
  in the referenced schema slice.
- [ ] Unverified handles cannot execute queries.
- [ ] Session state is typed, verified, and re-applied on every acquisition and
  reset. Unsupported transaction-pooler settings fail before execution.
- [ ] `execute`, `fetch_one`, `fetch_optional`, `expect_at_most_one`, `first`,
  bounded `fetch_all`, `stream`, and one-field `.scalar()` implement exact
  result contracts.
- [ ] Connections, transactions, savepoints, cleanup, commit, rollback, and live
  streams obey static ownership and fallible cleanup rules.
- [ ] Pools are `ShareSafe` cloneable handles. Connections, transactions, and
  streams cannot cross task boundaries.
- [ ] Context transactions never retry automatically. The separate replay API
  admits only compiler-validated `@retry_safe` callbacks and creates a fresh
  transaction per attempt.
- [ ] Cancellation gives resource cleanup one bounded shielded interval.
  Timeout closes or discards the resource and records a secondary cleanup error.
- [ ] Connections use bounded least-recently-used statement caches with complete
  semantic identity and explicit `warm` support. Preparation is not a public type.
- [ ] Deadlines, cancellation, backpressure, row-byte bounds, row-count bounds,
  statement-cache bounds, and connection bounds return structured errors.
- [ ] `ExecutionResult` has exact rows-affected and provider-metadata contracts.
- [ ] Runtime qualification provisions the exact PostgreSQL provider through its
  harness and rolls back test transactions. It does not depend on the later
  public tool namespace. No fake database API exists.
- [ ] The external async prerequisite is merged. Transactions and streams pass
  abnormal-exit, cancellation-cause, secondary-cleanup, and early-close tests.
- [ ] Malformed protocol and database data cannot reach a user-triggered panic.

Focused validation:

- runtime bridge, codec, pool, session, and statement-cache unit tests
- live PostgreSQL execution on every supported major
- cancellation, timeout, cleanup, leak, and poisoned-connection tests
- protocol fuzzing, panic scans, load tests, and resource-budget tests

### Milestone 10: Incremental compiler and editor experience

ID: `sql_10_incremental_editor`

Purpose: Provide deterministic incremental compilation and complete embedded SQL
editor behavior.

Owned scope:

- cache keys, dependency invalidation, bounds, and determinism
- virtual SQL documents and bidirectional source maps
- language-server features, fixes, formatting, budgets, and cancellation

Acceptance criteria:

- [ ] Cache identity includes template, hole types, fragments, schema slice,
  provider, compatibility settings, component protocol, and compiler semantics.
- [ ] Dependency-level invalidation preserves unaffected query results and always
  invalidates semantic changes.
- [ ] `sifr_frontend::cache_keys` owns cache identities. Cache bounds, pinning,
  and eviction cannot affect diagnostics or generated output.
- [ ] Every SQL template has a lossless virtual document and bidirectional source
  map.
- [ ] Highlighting, completion, hover, definition, references, rename, parameter
  information, result information, nullability, and cardinality work in templates.
- [ ] Formatting preserves holes and source meaning.
- [ ] Quick fixes cover aliases, casts, missing columns, unsafe collection, and
  supported migration impact changes.
- [ ] Fragment completion respects relation scope and aliases.
- [ ] SQL editor operations have named performance budgets and cancellation
  checkpoints before component entry and between provider operations.

Focused validation:

- incremental cold, warm, edit, and invalidation tests
- virtual-document and source-map round-trip properties
- language-server snapshots for every embedded SQL feature
- stale-result, cancellation, and editor latency budgets

### Milestone 11: Host tool graph and command runner

ID: `sql_11_host_tools`

Purpose: Resolve and execute SQL tools as host-only packages without target graph
contamination.

Owned scope:

- tools workspace resolution and locked entry-point metadata
- direct command namespaces and explicit host capabilities
- `sifr sql test provision` routing and its structured connection manifest
- host-target graph separation and cross-compilation behavior

Acceptance criteria:

- [ ] Cargo resolves a dedicated tools workspace member separately from
  application and target packages.
- [ ] `Cargo.lock`, Cargo metadata, and the tools member configuration record
  exact tool packages, selected entry points, hashes, and capabilities.
- [ ] `sifr <tool-namespace>` executes only the selected package entry point.
- [ ] Built-in namespaces are reserved. Duplicate namespaces are hard errors.
- [ ] `sifr sql test provision --profile <name>` invokes the selected provider
  tool, provisions its canonical schema fingerprint, and returns the common
  structured connection manifest.
- [ ] File, network, environment, credential-helper, and subprocess capabilities
  require explicit grants.
- [ ] Tool code and dependencies never enter target HIR, generated Rust, linker
  input, sysroot selection, or application artifacts.
- [ ] Cross-compilation uses host tools and target application dependencies without
  graph leakage.
- [ ] Unknown tools, undeclared capabilities, hash drift, and target contamination
  fail closed.

Focused validation:

- package and Cargo metadata resolution tests
- command namespace and capability negative tests
- target artifact and linker-input contamination scans
- native and cross-compilation graph-isolation suites

### Milestone 12: Schema lifecycle tools

ID: `sql_12_schema_tools`

Purpose: Provide deterministic schema pull, validation, and build workflows.

Owned scope:

- live catalog pull and semantic diff presentation
- source, snapshot, migration, and optional live-state validation
- deterministic snapshots, manifests, modules, and affected-query reports

Acceptance criteria:

- [ ] `schema pull` normalizes live provider catalogs and preserves semantic
  provider objects.
- [ ] Pull displays a semantic diff before replacement unless an explicit
  non-interactive acceptance flag is present.
- [ ] `schema validate` compares sources, canonical snapshots, migrations, and
  optional live state according to profile policy.
- [ ] Validation reports object differences and affected queries without silent
  file mutation.
- [ ] `schema build` produces deterministic snapshots, fingerprints, runtime
  manifests, generated modules, and schema dependency indexes. It does not emit
  application query signatures.
- [ ] Conflicting authorities, credentials in output, nondeterminism, and incomplete
  provider metadata fail closed.

Focused validation:

- tool command and capability tests
- deterministic output and semantic-diff snapshots
- live catalog round trips for each available provider
- credential redaction, authority conflict, and incomplete-metadata tests

### Milestone 13: Migration compiler and engine

ID: `sql_13_migration_engine`

Purpose: Compile and execute typed migration graphs with exact intermediate
schemas and recovery evidence.

Owned scope:

- migration DAG identities, fingerprints, states, and affine plans
- typed DDL, data steps, assertions, backfills, and transaction boundaries
- offline graph validation, execution records, recovery, and rollback contracts

Acceptance criteria:

- [ ] Migrations form a checked DAG with stable identities, parents, checksums,
  provider constraints, input fingerprints, and output fingerprints.
- [ ] Every DDL step produces an intermediate typed schema state.
- [ ] `MigrationPlan[S]` is affine. Each step consumes its plan and returns a
  compiler-generated nominal state type.
- [ ] Typed Sifr and SQL data steps can use only the objects in their declared
  intermediate state.
- [ ] Data callbacks receive a contextually typed, nonescaping `MigrationDb[S]`.
- [ ] SQL assertions require one non-null Boolean field and distinguish false,
  zero-row, and multiple-row failures.
- [ ] Raw DDL is reflected into schema effects or requires an explicit effect that
  validates against the canonical schema.
- [ ] Assertions, bounded backfills, progress keys, idempotent replay, and explicit
  transaction boundaries have checked semantics.
- [ ] Offline graph validation reproduces the canonical schema from every supported
  baseline and reports destructive changes, lock risk, and data rewrites.
- [ ] Rollback is explicit. The engine never synthesizes destructive reversal.
- [ ] Execution records locks, step state, recovery points, checksums, duration,
  heads, and fingerprints without panic or ambiguous recovery.

Focused validation:

- migration parser, graph, state-type, and fingerprint tests
- compile-pass and compile-fail data-step and assertion fixtures
- interruption, resume, checksum, head, and recovery simulations
- migration fuzzing and intermediate-schema property tests

### Milestone 14: PostgreSQL migration qualification

ID: `sql_14_postgresql_migrations`

Purpose: Qualify PostgreSQL migration semantics on every supported server major.

Owned scope:

- PostgreSQL DDL reflection and transaction boundaries
- locks, imports, baselines, drift, resume, and explicit rollback
- live supported-version migration matrix

Acceptance criteria:

- [ ] PostgreSQL DDL reflection covers every object in the provider capability
  matrix.
- [ ] Transactional and non-transactional operations use correct boundaries and
  explicit recovery points.
- [ ] Advisory locking, concurrent-start rejection, checksum drift, head mismatch,
  and schema drift fail closed.
- [ ] Import creates a truthful baseline without inventing historical migrations.
- [ ] Forward-only and explicitly reversible migrations report exact operator
  actions.
- [ ] Fresh creation, multi-step upgrade, merge, interruption, resume, failure,
  and rollback suites pass on every supported PostgreSQL version.

Focused validation:

- live fresh, upgrade, merge, interruption, and resume suites
- transactional and non-transactional DDL tests
- advisory-lock, drift, checksum, and concurrent-start tests
- import, baseline, forward-only, and explicit rollback evidence

### Milestone 15: Schema polymorphism and portable constraints

ID: `sql_15_schema_polymorphism`

Purpose: Specialize structural schema requirements without hiding provider
differences.

Owned scope:

- provider-normalized requirement artifacts and subset proofs
- profile specialization and undeclared-object rejection
- explicit provider capability constraints for portable code

Acceptance criteria:

- [ ] Provider-normalized checked-in DDL artifacts define requirements through a
  declared `SchemaIR` subset. Sifr does not add a second schema language.
- [ ] Requirement artifacts describe tables, columns, keys, types, nullability,
  and required provider capabilities.
- [ ] A concrete profile must prove every requirement before specialization.
- [ ] Each profile namespace exports one compile-time `SqlSchema[Profile]`
  witness. Specialization erases the witness and gives the query the proving
  profile parameter.
- [ ] A witness can occur only as a direct namespace export or a constrained
  generic parameter. Runtime storage, return, capture, and selection are errors.
- [ ] Only a verified pool, connection, or transaction with the proving profile
  can execute the specialized query.
- [ ] Specialized queries cannot reach undeclared schema objects or provider
  behavior.
- [ ] Portable code declares provider capability constraints explicitly.
- [ ] The provider-neutral specialization harness and PostgreSQL implementation
  pass in this milestone. Milestones 16 and 17 own the MySQL and SQLite evidence.
- [ ] There is no silent lowest-common-denominator rewrite or runtime provider
  dispatch for a statically known profile.

Focused validation:

- requirement normalization and subset property tests
- positive and negative witness, forwarding, specialization, and execution-binding
  fixtures, including every prohibited runtime use
- undeclared-object and missing-capability diagnostics
- PostgreSQL portable examples and the reusable provider-neutral harness

### Milestone 16: MySQL provider completion

ID: `sql_16_mysql_provider`

Purpose: Complete and qualify every MySQL compiler, runtime, tool, migration, and
editor contract.

Owned scope:

- provider-owned parser, semantics, schema normalization, and diagnostics
- raw runtime driver, TLS, codecs, cancellation, sessions, and pooling
- MySQL tools, migrations, editor behavior, and supported-version qualification

Acceptance criteria:

- [ ] Component and runtime manifests use the exact Milestone 0 versions of
  LALRPOP, `mysql_async`, `mysql_common`, Tokio, and Rustls.
- [ ] A provider-owned LALRPOP grammar, lexer, AST, recovery mode, and version
  gates pass differential parsing against each supported MySQL server.
- [ ] The runtime uses raw `mysql_async` connections without constructing its
  pool. Its tracing feature stays disabled. Rustls and minimal features are explicit.
- [ ] Cancellation uses a bounded `KILL QUERY` control path, then closes and
  discards the target connection when cancellation cannot complete safely.
- [ ] MySQL grammar, name resolution, coercions, collations, unsigned types,
  generated columns, conflict forms, modes, and schema objects are exact.
- [ ] SQL mode and collation inputs participate in fingerprints and caches.
- [ ] The runtime satisfies the common verification, ownership, execution,
  streaming, statement-cache, cancellation, bound, error, and panic-safety
  contracts.
- [ ] Schema tools and migration reflection cover the MySQL capability matrix.
- [ ] The MySQL provider tool implements `sifr sql test provision` with the
  common connection-manifest contract.
- [ ] MySQL independently normalizes, proves, specializes, and validates portable
  schema requirements through the Milestone 15 harness.
- [ ] Language-server features use MySQL semantics and documentation.
- [ ] Differential, conformance, migration, recovery, fuzz, property, and performance
  suites pass on every supported MySQL version.

Focused validation:

- MySQL parser and semantic differential suites
- live runtime, cancellation, session, and statement-cache tests
- schema tool and migration matrices for every supported version
- test-provision command and connection-manifest tests
- portable-requirement specialization and capability diagnostics
- editor snapshots, protocol fuzzing, panic scans, and performance budgets

### Milestone 17: SQLite provider completion

ID: `sql_17_sqlite_provider`

Purpose: Complete and qualify every SQLite compiler, runtime, tool, migration, and
editor contract.

Owned scope:

- versioned SQLite grammar, semantics, affinity, schema, and diagnostics
- dedicated-worker runtime, bundled library, codecs, interruption, and pooling
- SQLite tools, rebuild migrations, editor behavior, and corruption handling

Acceptance criteria:

- [ ] Component and runtime manifests use the exact Milestone 0 versions of
  syntaqlite, `rusqlite`, `libsqlite3-sys`, and the bundled SQLite amalgamation.
- [ ] The component pins syntaqlite and its SQLite grammar for each supported
  SQLite version and qualified compile-flag set.
- [ ] The runtime uses bundled `rusqlite` on dedicated blocking workers. It uses
  `InterruptHandle` for bounded cancellation and Sifr-owned pooling.
- [ ] SQLite grammar, affinity, strict tables, rowid, generated columns, conflict
  forms, attached scope, and schema objects are exact.
- [ ] Required SQLite features and minimum library version participate in profile
  validation, fingerprints, and caches.
- [ ] The file runtime satisfies the common verification, ownership, execution,
  streaming, statement-cache, cancellation, bound, error, and panic-safety
  contracts.
- [ ] Schema tools and migration reflection cover the SQLite capability matrix,
  including table-rebuild plans.
- [ ] The SQLite provider tool implements `sifr sql test provision` with the
  common connection-manifest contract.
- [ ] SQLite independently normalizes, proves, specializes, and validates portable
  schema requirements through the Milestone 15 harness.
- [ ] Language-server features use SQLite semantics and documentation.
- [ ] Conformance, migration, recovery, fuzz, property, corruption, locking, and
  performance suites pass on every supported SQLite version.

Focused validation:

- SQLite grammar and semantic conformance suites
- bundled-runtime, worker, interrupt, lock, and statement-cache tests
- schema tools and table-rebuild migration matrices
- test-provision command and connection-manifest tests
- portable-requirement specialization and capability diagnostics
- corruption, fuzz, panic, editor, and performance suites

### Milestone 18: Integrated qualification and phase closure

ID: `sql_18_closure`

Purpose: Prove that the complete SQL platform works as one final system without
unowned gaps or version drift.

Owned scope:

- integrated compiler, provider, runtime, tool, migration, editor, and security
  qualification
- final dependency, capability, verification, documentation, and roadmap records
- whole-phase review, one final merge gate, archive, and completion record

Acceptance criteria:

- [ ] The package graph, component manifests, tool workspace, and root lockfile
  match the approved dependency baseline without version drift.
- [ ] Automated checks reject unlocked parser or driver sources, unapproved
  features, missing license records, and parser-runtime version mismatches.
- [ ] SQLx, generic SQL parsers, generic pools, and ORM migration engines do not
  appear in first-party provider dependency graphs.
- [ ] The non-SQL component fixture and all three SQL providers pass the complete
  compiler component protocol qualification.
- [ ] Full clean, incremental, offline, reproducibility, and cross-compilation
  builds pass with locked inputs.
- [ ] All provider query, schema, runtime, tool, migration, editor, security,
  interoperability, fuzz, property, and performance suites pass.
- [ ] Cross-dialect portable examples validate independently for every declared
  provider.
- [ ] Security tests cover injection, unsafe capabilities, secret redaction,
  sandbox escape, malicious metadata, malformed protocols, and resource exhaustion.
- [ ] Generated runtime audits find no data-dependent panic, unchecked allocation,
  or unbounded collection path.
- [ ] Public and internal documentation describe only the implemented final
  architecture and contain complete runnable examples.
- [ ] The repository file-size guardrail, formatting, linting, create-PR gate,
  merge gate, and final whole-phase review pass on the exact candidate.
- [ ] The capability matrix and verification inventory contain no pending,
  unowned, waived, fallback, or deferred row.
- [ ] The roadmap records completion and this phase record moves to the archive.

Closure validation:

- every milestone qualification suite on its final merged evidence
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py`
- final capability, verification, dependency, panic, and documentation checks

## Dependency sequence

```text
Milestone 0   Architecture and dependency lock
  -> Milestone 1   Template-string language foundation
  -> Milestone 2   Structural record type system
  -> Milestone 3   Compiler component platform
  -> Milestone 4   Schema profiles and canonical `SchemaIR`
  -> Milestone 5   Common SQL contracts
  -> Milestone 6   Query and fragment substrate
  -> Milestone 7   PostgreSQL schema and query compiler
  -> Milestone 8   PostgreSQL semantic completion
  -> Milestone 9   PostgreSQL runtime
  -> Milestone 10  Incremental compiler and editor experience
  -> Milestone 11  Host tool graph and command runner
  -> Milestone 12  Schema lifecycle tools
  -> Milestone 13  Migration compiler and engine
  -> Milestone 14  PostgreSQL migration qualification
  -> Milestone 15  Schema polymorphism and portable constraints
  -> Milestone 16  MySQL provider completion
  -> Milestone 17  SQLite provider completion
  -> Milestone 18  Integrated qualification and phase closure
```

The sequence is strict. A later milestone cannot begin from an unmerged earlier
milestone. A later milestone can consume only recorded merged evidence.

Milestones 16 and 17 reuse the common provider contracts and qualification
harnesses. They cannot fork those contracts to simplify one provider.

## Validation ownership matrix

| Changed surface | Minimum focused validation owner |
| --- | --- |
| Template-string syntax or spans | Syntax, frontend, formatter, source-map properties, parser fuzzing, and affected e2e fixtures |
| Structural records | Type system, structural identity, HIR, code generation, compile fixtures, and property tests |
| Compiler component protocol | Package resolution, component host, WIT round trips, sandbox mutations, resource limits, and non-SQL qualification |
| Schema profiles or `SchemaIR` | Package configuration, provider normalization, fingerprints, semantic diffs, generated modules, and offline-build tests |
| Common query contracts | Type and bind matrices, codecs, ownership compile fixtures, errors, cardinality, and effects |
| Typed queries and fragments | Query-state and ownership compile fixtures, evaluation-order and encoded-ownership tests, fragment scope, precedence, and alias properties, injection tests, unsafe-capability diagnostics, and cardinality diagnostics |
| Provider compiler semantics | Parser differential tests, catalog snapshots, live server comparisons, diagnostics, and source maps |
| Provider runtime | Raw bridge tests, live versions, pooling, sessions, transactions, streams, cancellation, cleanup, fuzzing, and panic scans |
| Incremental compiler or editor | Cache properties, invalidation tests, virtual documents, language-server snapshots, stale results, cancellation, and budgets |
| Host tool graph | Package and Cargo metadata, namespace selection, capabilities, cross-compilation, and target-contamination scans |
| Schema lifecycle tools | Command tests, catalog round trips, semantic diffs, deterministic outputs, redaction, and authority conflicts |
| Migration compiler | DAG and state typing, intermediate schemas, assertions, recovery simulation, fuzzing, and graph properties |
| Provider migrations | Live fresh, upgrade, merge, interruption, resume, drift, locking, import, and rollback matrices |
| Schema requirements and portable constraints | Subset proofs, specialization fixtures, per-provider matrices completed in Milestones 15–17, and undeclared-object and missing-capability diagnostics |
| Dependency metadata | Baseline resolver, checksums, feature allowlist, licenses, advisory audit, compatibility, and mutation self-tests |
| Documentation only | Documentation checks, local links, `git diff --check`, and the file-size guardrail |

The current milestone owns every regression that its changes cause. Pre-existing
or external failures remain with their recorded owner.

## Review contract

Each implementation pull request receives one exact-candidate review through the
phase-closure workflow.

The review request must contain:

- the exact base and candidate commit identities
- every changed path
- the milestone ID, purpose, owned scope, and acceptance criteria
- focused and gate validation evidence
- prior blocking findings for a remediation review

Only an in-scope omission or regression can block a milestone. A suggestion or
pre-existing problem enters the deferred-work ledger with an owner.

Apply valid blocking findings in one batch. If the same finding returns twice,
stop and request adjudication.

If a second review finds a new mechanism defect, stop and revise the milestone
scope. Do not hide the new scope in another remediation round.

Review approval and validation must cover the same final candidate. Record-only
evidence updates do not require another broad implementation review.

## Risk controls

### Cross-layer authority drift

The architecture, WIT protocol, `SchemaIR`, provider manifests, generated plans,
and runtime bridges can disagree. Machine checks bind each artifact to one
protocol version, provider identity, and schema fingerprint.

### Parser and runtime version mismatch

A parser can accept syntax that its runtime server rejects. The dependency
baseline and provider capability manifest bind parser versions, server versions,
driver versions, features, and differential evidence.

### Offline and live schema drift

Offline compilation can use stale schema evidence. Runtime execution requires a
verified handle and the selected strictness policy before it sends a query.

### Embedded source-map drift

One edit can change Sifr spans, virtual SQL spans, and provider spans. Source-map
properties prove round trips across every hole, escape, and multiline segment.

### Unsafe syntax leakage

An ordinary value cannot become SQL syntax. Typed fragments carry static
categories, and unsafe syntax requires one explicit capability and lint contract.

### Cancellation races and resource reuse

A late cancellation can affect a later query. Cancellation poisons the active
connection or worker resource, bounds cleanup, and prevents reuse before
disposal.

### Session and statement-cache leakage

Pooled connections can retain session state or stale statements. Verified leases
apply typed session contracts and complete semantic cache identities.

### Migration partial failure

Provider DDL can commit implicitly or reject rollback. Migration plans record
transaction boundaries, recovery points, locks, checksums, and truthful forward
state.

### Provider divergence

Shared abstractions can erase dialect behavior. Each provider owns parsing and
semantics, while the common layer owns only checked execution shapes.

### Tool graph contamination

Host tools can leak dependencies into target artifacts. Cargo graph checks and
artifact scans prove separation for native and cross-compiled applications.

### Dependency and supply-chain drift

Upstream releases can change APIs, licenses, unsafe code, or bundled native
sources. The exact dependency baseline and qualification records make each change
reviewable.

### Qualification cost growth

Three providers and multiple server versions create a large test matrix. Each
milestone owns focused suites, named budgets, and reusable provider harnesses.

## Progress ledger

| Milestone | Status | Pull request | Merge commit | Validation | Review | Notes |
| ---: | --- | --- | --- | --- | --- | --- |
| 0 | completed | [#3582](https://github.com/sifr-lang/sifr/pull/3582) | `1a1cef93dc` | SQL 4/4; docs 1/1; dependency and runner checks pass | Opus `SATISFIED` on `7f3f6bc2c` | Architecture and dependency lock |
| 1 | in progress | — | — | focused compiler, native, property, fuzz, and SQL contract checks pass; exact gates pending | — | Template-string language foundation |
| 2 | pending | — | — | — | — | Structural record type system |
| 3 | pending | — | — | — | — | Compiler component platform |
| 4 | pending | — | — | — | — | Schema profiles and canonical `SchemaIR` |
| 5 | pending | — | — | — | — | Common SQL contracts |
| 6 | pending | — | — | — | — | Query and fragment substrate |
| 7 | pending | — | — | — | — | PostgreSQL schema and query compiler |
| 8 | pending | — | — | — | — | PostgreSQL semantic completion |
| 9 | pending | — | — | — | — | PostgreSQL runtime |
| 10 | pending | — | — | — | — | Incremental compiler and editor experience |
| 11 | pending | — | — | — | — | Host tool graph and command runner |
| 12 | pending | — | — | — | — | Schema lifecycle tools |
| 13 | pending | — | — | — | — | Migration compiler and engine |
| 14 | pending | — | — | — | — | PostgreSQL migration qualification |
| 15 | pending | — | — | — | — | Schema polymorphism and portable constraints |
| 16 | pending | — | — | — | — | MySQL provider completion |
| 17 | pending | — | — | — | — | SQLite provider completion |
| 18 | pending | — | — | — | — | Integrated qualification and phase closure |

## Deferred reviewer follow-up

| Source | Finding | Owner | Disposition |
| --- | --- | --- | --- |
| SQL design review | Syntaqlite is pre-1.0 and has concentrated upstream ownership. | Milestones 0 and 17 | Pin source and API adaptation. Record maintenance, supply-chain, and fork-readiness evidence. |
| SQL design review | PostgreSQL needs an explicit lossless editor layer around `libpg_query`. | Milestones 7 and 10 | Own the token and trivia layer in the provider. Keep strict parsing as compile authority. |
| SQL design review | Rusqlite and direct Rust interop can compete for one `libsqlite3-sys` link identity. | Milestones 0 and 17 | Lock one compatible version and reject an incompatible package graph before Cargo linking. |
| SQL design review | Shared TLS crates and provider TLS adapters need explicit dependency-ring records. | Milestone 0 | Record the existing shared substrate and each Ring 4 provider adapter. |
| Tooling baseline review | PostgreSQL support policy can change while the phase runs. | Milestones 0 and 18 | Reconcile parser tags with the approved supported-major matrix at lock and closure. |
| Tooling baseline review | Syntaqlite has GitHub source and crates.io release identities. | Milestone 0 | Name crates.io as release authority and GitHub as source authority in the baseline record. |
| Milestone 0 remediation review | Match architecture dependency rows by their exact first table cell. | Milestone 18 | Harden the final dependency revalidation checker before phase closure. |
| Milestone 0 remediation review | Bound GitHub release pagination and remove the unreachable tag-key fallback. | Milestone 18 | Harden the final release-authority refresh before phase closure. |

### Milestone 0 closure record

- Status: completed and merged.
- Starting commit: `90762c7872de0de05f760ac91f95463d9c679d59`.
- Initial reviewed candidate: `94c2bfb6362bede53f13cefbef92775f29119c94`.
- Final candidate: `7f3f6bc2cb5209b5cae64576cf1f11288be31f75`.
- Pull request: [#3582](https://github.com/sifr-lang/sifr/pull/3582).
- Merge commit: `1a1cef93dce74c2c8f56ea0059046c7ca332676a`.
- Acceptance disposition: all 14 Milestone 0 criteria are satisfied.
- Owned result: the architecture, dependency baseline, qualification record,
  ownership map, artifact topology, capability matrix, verification inventory,
  verification area, root lock package, and external async prerequisite are
  authoritative.
- SQL validation: four variants passed with no failure. The contract checker
  covers 19 milestones, three providers, six domains, and 30 invariants.
- Dependency validation: all 14 crates and six `libpg_query` sources match their
  release authorities. The all-feature qualification package and HTTP-enabled
  shared runtime compile with the locked graph.
- TLS validation: the inverse feature graph selects `rustls/aws_lc_rs` and
  `tokio-rustls/aws_lc_rs`. It contains no `ring` feature edge for either crate.
- Repository validation: documentation structure, verification-runner self-tests,
  Rust formatting, Python compilation, diff hygiene, and file-size checks passed.
- Gate disposition: no compiler source changed. The phase rule therefore skipped
  both Sifr repository gates.
- Review round 1: Opus found one shared TLS provider regression on
  `94c2bfb6362bede53f13cefbef92775f29119c94`.
- Remediation: the final candidate preserved the accepted AWS-LC-RS provider and
  hardened the dependency and profile mutation checkers.
- Review round 2: Opus returned `SATISFIED` with no blocking finding on
  `7f3f6bc2cb5209b5cae64576cf1f11288be31f75`. The [published review](https://github.com/sifr-lang/sifr/pull/3582#issuecomment-5465914743)
  is keyed by that candidate.
- Evidence scope: positive, negative, mutation, and property checks apply to this
  contract lock. The inventory assigns later fuzz and performance evidence to
  their implementation owners.
- Unrelated failures: none.
- Next action: implement Milestone 1 from the merged and recorded mainline.

## Closure evidence template

Each milestone appends one progress record with:

- exact starting, reviewed, final, and merge commit identities
- implementation summary and owned files
- each completed acceptance row
- focused commands and exact results
- positive, negative, mutation, fuzz, property, and performance evidence
- create-PR and merge-gate results when required
- external review rounds, findings, and remediations
- pull request and merge identities
- architecture, roadmap, and issue updates
- unrelated failures with their owning issue
- deferred reviewer follow-up with an exact owner

The phase cannot close from narrative confidence. Every locked invariant needs an
executable gate or a precise inspected artifact with named ownership.

## Phase completion record

Complete this section after Milestone 18 merges:

- Final status: pending.
- Final merge commit: pending.
- Final create-PR profile: pending.
- Final merge gate: pending.
- Final whole-phase review: pending.
- Final capability and verification inventory: pending.
- Deferred out-of-scope work: pending.
- Archive destination: `plans/issues/archive/ad-hoc-schema-first-sql-platform.md`.
- Exact next action: implement Milestone 1 in a new session from current
  `origin/main`.
