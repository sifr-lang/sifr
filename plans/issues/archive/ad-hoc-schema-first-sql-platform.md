# Ad hoc phase: Schema-first SQL platform

Status: completed and audited on 2026-09-01

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

The prerequisite merged in [PR #3607](https://github.com/sifr-lang/sifr/pull/3607).
Its final candidate is `46bbd40c8bca7538c8331f2ff3f891a98b2e9c88`, and its
merge commit is `0f01971c4d00cdf7e888360fc79c2703cbafb327`. Milestone 9
can use these contracts.

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
| 1 | completed | Template-string language foundation | Template strings preserve static segments, typed holes, evaluation order, and exact source maps through the full compiler pipeline. |
| 2 | completed | Structural record type system | Immutable records have order-independent canonical identity, width subtyping, deterministic diagnostics, and interned Rust layouts. |
| 3 | completed | Compiler component platform | Resolved packages can provide deterministic sandboxed embedded-language analysis through one closed, versioned, cacheable protocol. |
| 4 | completed | Schema profiles and canonical `SchemaIR` | Configuration sources produce exact provider-owned schema graphs, nominal profile types, fingerprints, dependency slices, diffs, and runtime contracts. |
| 5 | completed | Common SQL contracts | Shared query kinds, complete type and bind mappings, codecs, errors, cardinality, effects, ownership, and provider interfaces have one final contract. |
| 6 | completed | Query and fragment substrate | Query templates, owned bound queries, typed fragments, composition, safe interpolation, and cardinality adapters integrate with Sifr typing and HIR. |
| 7 | completed | PostgreSQL schema and query compiler | PostgreSQL catalogs, grammar, resolution, typing, nullability, result records, writes, dependencies, and diagnostics work offline. |
| 8 | completed | PostgreSQL semantic completion | Advanced PostgreSQL constructs, fragment scope changes, cardinality proofs, custom codecs, and exported-query stability rules are complete. |
| 9 | completed | PostgreSQL runtime | Verified pools, session contracts, transactions, streaming, automatic statement caching, explicit fetch methods, bounded cleanup, tests, and panic-safe protocol handling are complete. |
| 10 | completed | Incremental compiler and editor experience | Fine-grained caching, invalidation, virtual SQL documents, source maps, completion, navigation, rename, formatting, and quick fixes are complete. |
| 11 | completed | Host tool graph and command runner | Cargo-locked host-only tool packages execute direct command namespaces without entering application code generation. |
| 12 | completed | Schema lifecycle tools | Pull, validate, and build commands produce deterministic snapshots, fingerprints, manifests, modules, semantic diffs, and affected-query reports. |
| 13 | completed | Migration compiler and engine | Typed migration DAGs, intermediate schemas, DDL reflection, data steps, assertions, offline validation, recovery, and explicit rollback are complete. |
| 14 | completed | PostgreSQL migration qualification | PostgreSQL DDL, locks, transactional limits, imports, baselines, recovery, and supported-version execution pass full migration qualification. |
| 15 | completed | Schema polymorphism and portable constraints | Structural schema requirements specialize safely, while explicit capability constraints validate portable code for every declared provider. |
| 16 | completed | MySQL provider completion | MySQL query, schema, runtime, tooling, migration, editor, safety, and conformance surfaces satisfy the common and provider-specific contracts. |
| 17 | completed | SQLite provider completion | SQLite query, schema, runtime, tooling, migration, editor, safety, and conformance surfaces satisfy the common and provider-specific contracts. |
| 18 | completed | Integrated qualification and phase closure | All providers, tools, migrations, compiler paths, runtime paths, editor paths, security gates, budgets, examples, and documents are integrated, qualified, reviewed, merged, and archived as one final system. |

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
- [x] The verification inventory names the external issue, owner, and merge
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

- [x] The package graph resolves compiler components by exact identity, version,
  hash, and protocol range.
- [x] Components use the WebAssembly Component Model and the compiler-owned WIT
  interface without default WASI capabilities.
- [x] The protocol accepts static source, typed holes, context, and source maps.
- [x] It returns diagnostics, dependencies, type descriptors, semantic plans,
  and runtime-lowering descriptors from a closed schema.
- [x] The sandbox denies ambient file, network, clock, random, environment,
  process, thread, shared-memory, native-library, linker, Rust-source, and
  arbitrary-HIR access.
- [x] Official and third-party providers use the same component ABI and validation
  path.
- [x] CPU, memory, recursion, input, output, and diagnostic bounds fail with
  structured compiler errors.
- [x] Compiler and provider diagnostic code registries have stable, disjoint
  namespaces with machine-validated uniqueness and lifecycle metadata.
- [x] Compiler and components declare compatible protocol ranges. Incompatible
  ranges fail without protocol downgrade.
- [x] Cache keys include all semantic inputs and component identities.
- [x] A non-SQL fixture proves parsing, typed holes, diagnostics, source maps,
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

- [x] Fixed-width SQL integers map to exact Sifr widths. Generic integer binding
  uses checked narrowing.
- [x] Decimal, floating-point, temporal, text, binary, UUID, JSON, enum, array,
  domain, composite, range, IP, network, MAC, custom, unsigned, and SQLite
  affinity rules have explicit provider contracts.
- [x] Every supported value pair has an exact bind-compatibility rule. Width
  mismatch, nullability, arrays, custom codecs, and generic integers fail or
  convert according to that table.
- [x] Compile-time and runtime error families are structured, stable, redacted,
  and panic-safe.
- [x] Cardinality uses the complete interval lattice and never selects containers.
- [x] Read and write effects identify referenced and affected schema objects.
- [x] Provider interfaces separate shared execution shape from dialect semantics.
- [x] Query, pool, connection, transaction, and stream ownership protocols have
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

- [x] `QueryTemplate` and `BoundQuery` are the only public query states.
- [x] `@profile.query` creates a callable reusable template with a statically
  unique template identity and supports `RowOf`.
- [x] Binding evaluates each expression once, left to right, and owns encoded
  values after construction.
- [x] Execution consumes a bound query. Clone support depends on every captured
  value.
- [x] Ordinary values always become parameters with exact provider type checks.
- [x] Fragments carry profile, dialect, category, relation scope, aliases,
  parameters, result transformation, effect transformation, and precedence.
- [x] Relation aliases and fragment identities are static inside a query
  definition. They cannot escape, depend on runtime control flow, or enter a
  runtime container.
- [x] `RowOf` accepts only a top-level reusable query symbol. Exported signatures
  resolve it to a stable structural type alias.
- [x] Canonical predicate combinators cover optional filters without string
  assembly.
- [x] `expect_at_most_one()` narrows cardinality by runtime validation.
- [x] `first()` is explicit and warns when ordering is not deterministic.
- [x] Branching and generic code unify changing query and structural record types
  through normal Sifr typing.
- [x] Unsafe syntax escape requires the complete security capability and lint
  contract.
- [x] Generated schema identifiers use one injective, reversible encoding across
  paths, keywords, enum variants, and composite fields.
- [x] Frontend and query compilation consume a production-queryable profile-module
  registry; qualification invokes that production consumer.
- [x] Query lowering preserves the compile-time cardinality and effect records in
  the runtime request. Executable round-trip tests prove both views are equal.

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

- [x] The provider uses the Milestone 0 `libpg_query` tag for each supported
  PostgreSQL major. The component manifest records every source checksum.
- [x] The provider embeds tagged `libpg_query` source for each supported server
  major and maps its raw tree into provider-owned syntax nodes.
- [x] Catalog ingestion and DDL sources normalize every supported PostgreSQL
  object into `SchemaIR`.
- [x] The provider implements PostgreSQL parsing, name resolution, casts,
  operators, functions, aggregates, aliases, correlations, and set operations.
- [x] Parameter inference produces one exact database type and codec per hole.
- [x] Result inference produces unique named structural fields with conservative
  and PostgreSQL-correct nullability.
- [x] Write analysis covers required values, generated columns, conflict clauses,
  effects, and `RETURNING`.
- [x] Diagnostics map to Sifr, virtual SQL, and schema spans.
- [x] Differential tests validate behavior against every supported PostgreSQL
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

- [x] Arrays, ranges, composite types, domains, enums, JSON operations, windows,
  common table expressions, locking, and PostgreSQL-specific DDL are complete.
- [x] Outer joins, aggregates, `CASE`, scalar subqueries, and provider functions
  produce exact nullability facts.
- [x] Unique predicates, limits, aggregates, writes, and set operations produce
  sound cardinality intervals.
- [x] Join and select-list fragments transform relation scope and result records
  without losing static typing.
- [x] Values and assignment fragments cover bounded batches, dynamic updates,
  conflict behavior, provider parameter limits, and explicit chunking semantics.
- [x] Custom codecs have one declared database identity and checked encode/decode
  behavior.
- [x] Exported `SELECT *`, unstable names, duplicate names, and schema-sensitive
  public types have final lint behavior and machine-applicable fixes.
- [x] The compiler expands accepted private `SELECT *` forms to explicit emitted
  columns. Exported `SELECT *` is always an error.
- [x] Application `sifr build` emits query-signature artifacts for package API
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

- [x] Runtime manifests use the exact Milestone 0 versions of Tokio, Rustls,
  `tokio-postgres`, `postgres-types`, and `tokio-postgres-rustls`.
- [x] The runtime uses raw `tokio-postgres`, `postgres-types`, and
  `tokio-postgres-rustls` clients with an explicit feature allowlist.
- [x] `sifr_sql_runtime` owns pooling, verified leases, session reset,
  statement-cache policy, cancellation budgets, and resource accounting.
- [x] Pool verification combines one evidence mode with one strictness mode.
- [x] Compatible verification compares every recorded property and absence fact
  in the referenced schema slice.
- [x] Unverified handles cannot execute queries.
- [x] Session state is typed, verified, and re-applied on every acquisition and
  reset. Unsupported transaction-pooler settings fail before execution.
- [x] `execute`, `fetch_one`, `fetch_optional`, `expect_at_most_one`, `first`,
  bounded `fetch_all`, `stream`, and one-field `.scalar()` implement exact
  result contracts.
- [x] Connections, transactions, savepoints, cleanup, commit, rollback, and live
  streams obey static ownership and fallible cleanup rules.
- [x] The transaction runtime implements and tests an explicit live, committed,
  rolled-back, poisoned, and dropped transition matrix. Commit, rollback, and
  cleanup cannot leave a handle reusable or report success after an invalid
  transition.
- [x] Pools are `ShareSafe` cloneable handles. Connections, transactions, and
  streams cannot cross task boundaries.
- [x] Context transactions never retry automatically. The separate replay API
  admits only compiler-validated `@retry_safe` callbacks and creates a fresh
  transaction per attempt.
- [x] Cancellation gives resource cleanup one bounded shielded interval.
  Timeout closes or discards the resource and records a secondary cleanup error.
- [x] Connections use bounded least-recently-used statement caches with complete
  semantic identity and explicit `warm` support. Preparation is not a public type.
- [x] Deadlines, cancellation, backpressure, row-byte bounds, row-count bounds,
  statement-cache bounds, and connection bounds return structured errors.
- [x] `ExecutionResult` has exact rows-affected and provider-metadata contracts.
- [x] Runtime qualification provisions the exact PostgreSQL provider through its
  harness and rolls back test transactions. It does not depend on the later
  public tool namespace. No fake database API exists.
- [x] The external async prerequisite is merged. Transactions and streams pass
  abnormal-exit, cancellation-cause, secondary-cleanup, and early-close tests.
- [x] Malformed protocol and database data cannot reach a user-triggered panic.

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

- [x] Cache identity includes template, hole types, fragments, schema slice,
  provider, compatibility settings, component protocol, and compiler semantics.
- [x] Dependency-level invalidation preserves unaffected query results and always
  invalidates semantic changes.
- [x] `sifr_frontend::cache_keys` owns cache identities. Cache bounds, pinning,
  and eviction cannot affect diagnostics or generated output.
- [x] Every SQL template has a lossless virtual document and bidirectional source
  map.
- [x] Highlighting, completion, hover, definition, references, rename, parameter
  information, result information, nullability, and cardinality work in templates.
- [x] Formatting preserves holes and source meaning.
- [x] Quick fixes cover aliases, casts, missing columns, unsafe collection, and
  supported migration impact changes.
- [x] Fragment completion respects relation scope and aliases.
- [x] SQL editor operations have named performance budgets and cancellation
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

- inherited Milestone 10 SQL-profile failure isolation before host-tool work
- tools workspace resolution and locked entry-point metadata
- direct command namespaces and explicit host capabilities
- `sifr sql test provision` routing and its structured connection manifest
- host-target graph separation and cross-compilation behavior

Acceptance criteria:

- [x] SQL profile preparation failures preserve all non-SQL analysis, emit an
  explicit SQL initialization diagnostic, and have a live package-host
  regression test.
- [x] Cargo resolves a dedicated tools workspace member separately from
  application and target packages.
- [x] `Cargo.lock`, Cargo metadata, and the tools member configuration record
  exact tool packages, selected entry points, hashes, and capabilities.
- [x] `sifr <tool-namespace>` executes only the selected package entry point.
- [x] Built-in namespaces are reserved. Duplicate namespaces are hard errors.
- [x] `sifr sql test provision --profile <name>` invokes the selected provider
  tool, provisions its canonical schema fingerprint, and returns the common
  structured connection manifest.
- [x] File, network, environment, credential-helper, and subprocess capabilities
  require explicit grants.
- [x] Tool code and dependencies never enter target HIR, generated Rust, linker
  input, sysroot selection, or application artifacts.
- [x] Cross-compilation uses host tools and target application dependencies without
  graph leakage.
- [x] Unknown tools, undeclared capabilities, hash drift, and target contamination
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

- [x] `schema pull` normalizes live provider catalogs and preserves semantic
  provider objects.
- [x] Pull displays a semantic diff before replacement unless an explicit
  non-interactive acceptance flag is present.
- [x] `schema validate` compares sources, canonical snapshots, migrations, and
  optional live state according to profile policy.
- [x] Validation reports object differences and affected queries without silent
  file mutation.
- [x] `schema build` produces deterministic snapshots, fingerprints, runtime
  manifests, generated modules, and schema dependency indexes. It does not emit
  application query signatures.
- [x] Conflicting authorities, credentials in output, nondeterminism, and incomplete
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

- [x] Migrations form a checked DAG with stable identities, parents, checksums,
  provider constraints, input fingerprints, and output fingerprints.
- [x] Every DDL step produces an intermediate typed schema state.
- [x] `MigrationPlan[S]` is affine. Each step consumes its plan and returns a
  compiler-generated nominal state type.
- [x] Typed Sifr and SQL data steps can use only the objects in their declared
  intermediate state.
- [x] Data callbacks receive a contextually typed, nonescaping `MigrationDb[S]`.
- [x] SQL assertions require one non-null Boolean field and distinguish false,
  zero-row, and multiple-row failures.
- [x] Raw DDL is reflected into schema effects or requires an explicit effect that
  validates against the canonical schema.
- [x] Assertions, bounded backfills, progress keys, idempotent replay, and explicit
  transaction boundaries have checked semantics.
- [x] Offline graph validation reproduces the canonical schema from every supported
  baseline and reports destructive changes, lock risk, and data rewrites.
- [x] Rollback is explicit. The engine never synthesizes destructive reversal.
- [x] Execution records locks, step state, recovery points, checksums, duration,
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

- [x] PostgreSQL DDL reflection covers every object in the provider capability
  matrix.
- [x] Transactional and non-transactional operations use correct boundaries and
  explicit recovery points.
- [x] Advisory locking, concurrent-start rejection, checksum drift, head mismatch,
  and schema drift fail closed.
- [x] Import creates a truthful baseline without inventing historical migrations.
- [x] Forward-only and explicitly reversible migrations report exact operator
  actions.
- [x] Fresh creation, multi-step upgrade, merge, interruption, resume, failure,
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

- [x] Provider-normalized checked-in DDL artifacts define requirements through a
  declared `SchemaIR` subset. Sifr does not add a second schema language.
- [x] Requirement artifacts describe tables, columns, keys, types, nullability,
  and required provider capabilities.
- [x] A concrete profile must prove every requirement before specialization.
- [x] Each profile namespace exports one compile-time `SqlSchema[Profile]`
  witness. Specialization erases the witness and gives the query the proving
  profile parameter.
- [x] A witness can occur only as a direct namespace export or a constrained
  generic parameter. Runtime storage, return, capture, and selection are errors.
- [x] Only a verified pool, connection, or transaction with the proving profile
  can execute the specialized query.
- [x] Specialized queries cannot reach undeclared schema objects or provider
  behavior.
- [x] Portable code declares provider capability constraints explicitly.
- [x] The provider-neutral specialization harness and PostgreSQL implementation
  pass in this milestone. Milestones 16 and 17 own the MySQL and SQLite evidence.
- [x] There is no silent lowest-common-denominator rewrite or runtime provider
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

- [x] Component and runtime manifests use the exact Milestone 0 versions of
  LALRPOP, `mysql_async`, `mysql_common`, Tokio, and Rustls.
- [x] A provider-owned LALRPOP grammar, lexer, AST, recovery mode, and version
  gates pass differential parsing against each supported MySQL server.
- [x] The runtime uses raw `mysql_async` connections without constructing its
  pool. Its tracing feature stays disabled. Rustls and minimal features are explicit.
- [x] Cancellation uses a bounded `KILL QUERY` control path, then closes and
  discards the target connection when cancellation cannot complete safely.
- [x] MySQL grammar, name resolution, coercions, collations, unsigned types,
  generated columns, conflict forms, modes, and schema objects are exact.
- [x] SQL mode and collation inputs participate in fingerprints and caches.
- [x] The runtime satisfies the common verification, ownership, execution,
  streaming, statement-cache, cancellation, bound, error, and panic-safety
  contracts.
- [x] Schema tools and migration reflection cover the MySQL capability matrix.
- [x] The MySQL provider tool implements `sifr sql test provision` with the
  common connection-manifest contract.
- [x] MySQL independently normalizes, proves, specializes, and validates portable
  schema requirements through the Milestone 15 harness.
- [x] Language-server features use MySQL semantics and documentation.
- [x] Differential, conformance, migration, recovery, fuzz, property, and performance
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

- [x] Component and runtime manifests use the exact Milestone 0 versions of
  syntaqlite, `rusqlite`, `libsqlite3-sys`, and the bundled SQLite amalgamation.
- [x] The component pins syntaqlite and its SQLite grammar for each supported
  SQLite version and qualified compile-flag set.
- [x] The runtime uses bundled `rusqlite` on dedicated blocking workers. It uses
  `InterruptHandle` for bounded cancellation and Sifr-owned pooling.
- [x] SQLite grammar, affinity, strict tables, rowid, generated columns, conflict
  forms, attached scope, and schema objects are exact.
- [x] Required SQLite features and minimum library version participate in profile
  validation, fingerprints, and caches.
- [x] The file runtime satisfies the common verification, ownership, execution,
  streaming, statement-cache, cancellation, bound, error, and panic-safety
  contracts.
- [x] Schema tools and migration reflection cover the SQLite capability matrix,
  including table-rebuild plans.
- [x] The SQLite provider tool implements `sifr sql test provision` with the
  common connection-manifest contract.
- [x] SQLite independently normalizes, proves, specializes, and validates portable
  schema requirements through the Milestone 15 harness.
- [x] Language-server features use SQLite semantics and documentation.
- [x] Conformance, migration, recovery, fuzz, property, corruption, locking, and
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

- [x] The package graph, component manifests, tool workspace, and root lockfile
  match the approved dependency baseline without version drift.
- [x] Automated checks reject unlocked parser or driver sources, unapproved
  features, missing license records, and parser-runtime version mismatches.
- [x] SQLx, generic SQL parsers, generic pools, and ORM migration engines do not
  appear in first-party provider dependency graphs.
- [x] The non-SQL component fixture and all three SQL providers pass the complete
  compiler component protocol qualification.
- [x] Full clean, incremental, offline, reproducibility, and cross-compilation
  builds pass with locked inputs.
- [x] All provider query, schema, runtime, tool, migration, editor, security,
  interoperability, fuzz, property, and performance suites pass.
- [x] Cross-dialect portable examples validate independently for every declared
  provider.
- [x] Security tests cover injection, unsafe capabilities, secret redaction,
  sandbox escape, malicious metadata, malformed protocols, and resource exhaustion.
- [x] Generated runtime audits find no data-dependent panic, unchecked allocation,
  or unbounded collection path.
- [x] Public and internal documentation describe only the implemented final
  architecture and contain complete runnable examples.
- [x] The repository file-size guardrail, formatting, linting, and final
  whole-phase review pass. The one-shot create-PR and merge-gate outcomes and
  their later guard-classification corrections are recorded without a rerun.
- [x] The capability matrix and verification inventory contain no pending,
  unowned, waived, fallback, or deferred row.
- [x] The roadmap records completion and this phase record moves to the archive.

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

If a second review finds a new mechanism defect, record a later phase item.
Continue the sequence and do not run a third review. Milestone 18 must resolve
the item before the final qualification.

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
| 0 | completed | [#3582](https://github.com/sifr-lang/sifr/pull/3582) | `1a1cef93dc` | SQL 4/4; docs 1/1; dependency and runner checks pass | agent `SATISFIED` on `7f3f6bc2c` | Architecture and dependency lock |
| 1 | completed | [#3585](https://github.com/sifr-lang/sifr/pull/3585) | `1173cd9e20` | type system 140/140; focused template 13/13; property 15/15; fuzz 26/26; SQL and repository checks pass | agent remediation `SATISFIED` on `56f131e1b` | Typed template strings; also corrects the Milestone 0 verification integration |
| 2 | completed | [#3588](https://github.com/sifr-lang/sifr/pull/3588) | `955e97f6db` | affected packages, native fixture, HIR guard, and file-size guard pass | agent round 2 closed both original blockers on `dd7ac3cdc`; one new mechanism defect is deferred | Structural record type system |
| 3 | completed | [#3592](https://github.com/sifr-lang/sifr/pull/3592) | `9badcfc4aa` | component 14/14; diagnostics 32/32; package resolution 4/4; SQL 6/6; coverage 5/5; four-target qualification pass | agent round 2 closed the original sandbox and diagnostics blockers on `3d97d7e35`; two new mechanism defects are deferred | Compiler component platform |
| 4 | completed | [#3595](https://github.com/sifr-lang/sifr/pull/3595) | `40facaf98d` | contract 9/9; component 14/14; driver 2/2; package 149; SQL 10/10; coverage, Clippy, formatting, HIR, and file-size checks pass | agent round 2 verified the original pipeline and credential fixes on `04e00c51b`; two new mechanisms are deferred | Canonical schema profiles, provider authority pipeline, generated modules, fingerprints, slices, diffs, and runtime manifests |
| 5 | completed | [#3597](https://github.com/sifr-lang/sifr/pull/3597) | `7f2382ae68` | contract 9/9; runtime 9/9 plus 2 compile-fail doctests; diagnostics 32/32; SQL common qualification; strict Clippy and guards pass | Final exact-SHA agent `SATISFIED` on `f7a3e5a35` | Provider-neutral type, bind, codec, cardinality, effect, error, runtime, and ownership contracts |
| 6 | completed | [#3599](https://github.com/sifr-lang/sifr/pull/3599) | `9944bdd450` | SQL 19/19; coverage 4/4; contract 23/23; runtime 12/12 plus two doctests; frontend 3/3; driver 2/2; strict Clippy and guards pass | agent remediation `SATISFIED` on `0abd5109f` | Query and fragment substrate, registry, generated identifier codec, HIR, runtime binding, and execution request lowering |
| 7 | completed | [#3602](https://github.com/sifr-lang/sifr/pull/3602) | `46f1d06d8e` | PostgreSQL 13-18 native, component, and live matrices; SQL qualification, mutation, Clippy, and guards pass | agent round 2 verified every original remediation on `6cd745149`; two new mechanisms are deferred | PostgreSQL schema and query compiler |
| 8 | completed | [#3604](https://github.com/sifr-lang/sifr/pull/3604) | `e18e0a92d5` | SQL 4/4; PostgreSQL 13-18 native and component suites; contract, build-output, strict Clippy, and guards pass | agent round 2 closed the original nested-star blocker on `94fbb6e0f`; one new semantic-flag mechanism is deferred | PostgreSQL advanced semantics, stable projections, codecs, fragments, and query-signature artifacts |
| 9 | completed | [#3611](https://github.com/sifr-lang/sifr/pull/3611) | `9dc0e55e09` | common and provider runtime tests; SQL compiler and runtime qualification; strict Clippy and guards; exact PostgreSQL 13-18 live matrix | agent remediation `SATISFIED` on `258f13a9a`; one new malformed-BOOL classification defect is deferred | Verified PostgreSQL runtime, sessions, transactions, streaming, caching, cleanup, and resource bounds |
| 10 | completed | [#3617](https://github.com/sifr-lang/sifr/pull/3617) | `35926da677` | incremental-editor 2/2; SQL mutation 9/9; frontend 134; analysis 51; LSP 81; PostgreSQL component 14/14; strict Clippy and guards pass | agent round 2 closed all four original blockers on `08e864bf6`; one new failure-isolation defect is deferred | Incremental compiler and editor experience |
| 11 | completed | [#3619](https://github.com/sifr-lang/sifr/pull/3619) | `a20219da54` | host-tools 7/7; direct CLI 1/1; provisioning 1/1; analysis isolation 2/2; strict Clippy and guards pass | Two exact-SHA agent reviews; round 2 verified the original blockers and deferred one Linux prerequisite plus new follow-ups under the continuation rule | Locked and confined host tool runner, structured provisioning, and inherited M10 failure isolation |
| 12 | completed | [#3621](https://github.com/sifr-lang/sifr/pull/3621) | `73a69f691d` | schema-tool 6/6; provider 20/20; PostgreSQL 13-18 live catalog, artifact, and parity matrix; strict Clippy and guards pass | agent remediation `SATISFIED` on `ba3629d63` | Deterministic pull, validation, build, canonical catalog normalization, and atomic artifact publication |
| 13 | completed | [#3625](https://github.com/sifr-lang/sifr/pull/3625) | `6d28649ff9` | migration-engine 6/6; affected crates; workspace Clippy; formatting; HIR, file-size, SQL contract, metadata, and dependency guards pass | agent remediation `SATISFIED` on `c6acf3da7`; one new merge mechanism is deferred under the continuation rule | Checked compiler graph, nominal migration HIR, closed runtime plan, recovery engine, and atomic artifacts |
| 14 | completed | [#3627](https://github.com/sifr-lang/sifr/pull/3627) | `5b2aa585f4` | migration and PostgreSQL suites 9/9; live PostgreSQL 13-18 migration matrix; strict Clippy, formatting, guards, qualification, and mutation checks pass | agent remediation `SATISFIED` on `a8f02da62`; two new mechanisms are deferred under the continuation rule | PostgreSQL DDL reflection, runtime plan v2, imports, locking, recovery, rollback, operator commands, and supported-major qualification |
| 15 | completed | [#3630](https://github.com/sifr-lang/sifr/pull/3630) | `d878ecfe1a` | schema-polymorphism 7/7; PostgreSQL 13-18 component sandbox; affected tests, strict Clippy, formatting, and guards pass | Two exact-SHA agent reviews; round 2 verified object accounting and deferred the new positional constraint-identity mechanism under the continuation rule | Structural requirements, compile-time witnesses, specialization, execution binding, and provider-owned capability and object accounts |
| 16 | completed | [#3635](https://github.com/sifr-lang/sifr/pull/3635) | `70de2f82bb` | SQL platform 62/62; MySQL 8.4, 9.7, and 26.7 live matrices; affected tests, strict Clippy, formatting, and guards pass | agent remediation `SATISFIED` on `5d585f4b0`; new mechanism suggestions are deferred under the continuation rule | Complete MySQL compiler, runtime, tools, migrations, provisioning, editor integration, and qualification |
| 17 | completed | [#3640](https://github.com/sifr-lang/sifr/pull/3640) | `4a64e163ae` | SQLite provider 6/6; contract and package tests; strict Clippy; formatting and guards pass | agent remediation `SATISFIED` on `22a87c70d`; new suggestions are deferred to Milestone 18 | Complete SQLite compiler, worker runtime, tools, migrations, provisioning, editor integration, and qualification |
| 18 | completed | [#3645](https://github.com/sifr-lang/sifr/pull/3645) | `83a5f95d14` | integrated, build, provider, PostgreSQL 13-18 live, MySQL live, component, Clippy, formatting, HIR, and file-size evidence passed; one-shot gate dispositions recorded | Whole-phase agent remediation `SATISFIED` on `c0c6ae255`; four new mechanisms deferred under the continuation rule | Final integration, live parity remediation, executable qualification, component rebuilds, and phase closure |

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
| Milestone 2 remediation review | An all-`int` wide record can stay live after projection while Rust partially moves one `SifrInt` field. | Milestone 18 | Align logical-copy projection with physical Rust moves. Add a multi-field projection-and-reuse regression fixture. |
| Milestone 2 remediation review | Record fields with `Callable` or union types can produce invalid generated Rust. | Milestone 18 | Complete these record field layouts before integrated qualification. Add native positive fixtures for both field types. |
| Milestone 3 remediation review | A warm component-cache hit can bypass the current request's `max_input_bytes` limit. | Milestone 18 | Validate the input envelope and its byte limit before cache lookup. Add a warm-cache mutation test with a lower request limit. |
| Milestone 3 remediation review | Component response diagnostics and source maps can name document identities that were not present in the request. | Milestone 18 | Restrict primary, related, and source-map spans to request-owned template documents. Add forged-document rejection tests. |
| Milestone 4 remediation review | Generated schema paths, language-keyword escapes, enum variants, and composite fields can collapse to the same emitted identifier. | Milestone 6 | Replace the flattening and escaping rules with one injective, reversible encoding. Add collision properties before queries consume generated schema types. |
| Milestone 4 remediation review | The qualification record claims a compiler profile registry, but production code only consumes a cache fragment and exposes generated modules to tests. | Milestone 6 | Add one production-queryable profile-module registry for frontend and query compilation. Bind its qualification evidence to an executable consumer. |
| Milestone 5 initial review | The common ownership model cannot prove a provider transaction's commit, rollback, poison, and drop transitions against a real runtime implementation. | Milestone 9 | Implement the explicit transaction state machine in the PostgreSQL runtime and test every terminal, cleanup, cancellation, and invalid-reuse transition. |
| Milestone 5 initial review | Compile-time cardinality and effect analysis has no executable lowering round trip into the runtime request. | Milestone 6 | Lower both records with each bound query and prove compiler/runtime equality for reads, writes, adapters, and fragment transformations. |
| Milestone 6 initial review | Fragment composition renumbers parameter slots without a provider placeholder-rewrite contract. | Milestone 7 | Define the PostgreSQL placeholder representation and prove that composed syntax and parameter metadata stay aligned. |
| Milestone 6 initial review | Unsafe syntax accepts an already-verified grant but does not yet consume the package capability resolver directly. | Milestone 7 | Connect the first production provider query entry point to package capability resolution and test denied, warned, and allowed uses. |
| Milestone 6 initial review | The driver profile cache silently skips a registry entry if its context artifact map is inconsistent. | Milestone 18 | Collapse or explicitly validate the two-map invariant during integrated hardening. |
| Milestone 6 initial review | The reverse generated-identifier codec has no production consumer yet. | Milestone 10 | Use the reverse map in editor and source-display paths and qualify real generated modules. |
| Milestone 6 remediation review | The identifier documentation's trigger bullet list does not name all boundary-underscore escapes. | Milestone 7 | Correct the list when the PostgreSQL generated-schema documentation is extended. |
| Milestone 7 remediation review | An aggregate inside a scalar or predicate subquery marks the enclosing query as aggregate and can narrow its cardinality to exactly one. | Milestone 8 | Keep aggregate detection at one query level. Add a correlated scalar-subquery regression that preserves outer `MANY` cardinality. |
| Milestone 7 remediation review | Explicit `DEFAULT` bypasses the required-value check for a non-null column that has no default. | Milestone 8 | Reject explicit `DEFAULT` in INSERT and UPDATE when the target has no default and cannot accept null. Add positive and negative write fixtures. |
| Milestone 7 remediation review | The live PostgreSQL suite conflicts with the standard profiles' offline-only contract, so both required repository gates stop before tests. | Milestone 18 | Add an explicit live SQL profile or another verification-owned opt-in model. Keep standard profiles offline and make profile assignment validation understand live suites. |
| Milestone 7 remediation review | The live server runner captures fresh facts but its default mode does not compare them with the checked evidence. | Milestone 18 | Make default execution fail on evidence drift. Keep `--write` as the explicit refresh operation. |
| Milestone 7 remediation review | `A_Star` and unary minus produce misleading adapter diagnostics, and mixed aggregate queries lack a local GROUP BY validity check. | Milestone 8 | Add explicit syntax nodes or diagnostics and complete same-query-level grouping validation with positive and negative fixtures. |
| Milestone 8 initial review | The application build creates a query-signature registry, but source-level query lowering does not populate it yet. | Milestone 18 | Wire normal `@profile.query` compilation into the build registry. Prove that a package with exported queries emits non-empty compatibility entries. |
| Milestone 8 initial review | Projection fixes use prose instead of structured source edits. | Milestone 10 | Attach spans and replacements to projection diagnostics and expose them as language-server quick fixes. |
| Milestone 8 remediation review | Set operations inherit operand `deterministic-order` flags without an outer `ORDER BY`. | Milestone 18 | Propagate only inheritable operand flags. Add a regression that rejects operand ordering as a set-result guarantee. |
| Milestone 8 remediation review | `EXISTS (SELECT *)` follows the uniform exported no-star rule, but its diagnostic implies that the exported result projection contains the star. | Milestone 10 | Keep or narrow the policy explicitly, then make the diagnostic and quick fix describe the actual nested location. |
| Milestone 8 remediation review | A locking clause on a set operand is not rejected by the enclosing set-operation check. | Milestone 18 | Reject row locking anywhere under `UNION`, `INTERSECT`, or `EXCEPT`. Add direct and parenthesized operand fixtures. |
| Milestone 9 remediation review | A malformed PostgreSQL `BOOL` payload whose length is not one is preserved as encoded data instead of failing at the fixed-width decode boundary. | Milestone 18 | Return `SqlErrorKind::Decode` for every invalid boolean wire length and add malformed zero-length and multi-byte corpus cases. |
| Milestone 9 remediation review | The PostgreSQL component guest-source identity hashes the complete root manifests, so an unrelated workspace dependency edit invalidates the component record. | Milestone 18 | Narrow the identity to compiler-relevant manifest data and prove unrelated workspace dependency mutations do not cause drift. |
| Milestone 9 remediation review | The common statement-cache schema invalidation policy is correct and tested but has no provider caller. | Milestone 18 | Wire schema-change invalidation into the integrated runtime lifecycle or remove the unused policy after proving connection replacement is sufficient. |
| Milestone 10 remediation review | A SQL profile preparation failure aborts `AnalysisHost` construction and disables otherwise valid non-SQL language features. | Milestone 11 | Make SQL initialization failure explicit but non-fatal to non-SQL analysis. Add a package-host regression for a missing lockfile, malformed schema, or provider failure before host-tool work. |
| Milestone 10 remediation review | Workspace URI changes rebuild the analysis host, discard SQL caches, and recompute package metadata and schema normalization. | Milestone 18 | Give prepared profiles and the SQL runtime a workspace-stable owner. Prove bounded cache reuse and precise invalidation across file-open and schema-edit events. |
| Milestone 10 remediation review | Dependency observation uses the immutable prepared schema snapshot, so within-host invalidation cannot observe changes or newly added objects. | Milestone 18 | Observe current schema state independently of cached inputs and define conservative invalidation for newly introduced objects, including `SELECT *`. |
| Milestone 11 remediation review | Linux host-tool enforcement requires Bubblewrap and `prlimit`, but the Ubuntu validation lanes do not declare, diagnose, or execute this prerequisite. | Milestone 18 | Provision and diagnose the sandbox dependencies. Execute the same Linux capability-denial and output-bound integration evidence before phase closure. |
| Milestone 11 remediation review | A legal declared namespace that is one edit from a built-in is intercepted by typo correction before graph lookup. | Milestone 18 | Resolve the locked graph first. Suggest a built-in only when the namespace is not declared. Add a legal near-name regression. |
| Milestone 11 remediation review | Host-tool output is buffered until exit and discarded when the combined bound is exceeded. | Milestone 18 | Define bounded streaming behavior for long-running tools without weakening the hard output limit. |
| Milestone 11 remediation review | The macOS profile grants broad Mach lookup and `/private/etc` reads. | Milestone 18 | Narrow the mandatory platform bootstrap grants and retain executable denial tests. |
| Milestone 11 remediation review | Cargo build scripts and procedural macros run before runtime confinement, but the host-tool trust document does not state that build-time boundary. | Milestone 18 | Document the reviewed-package build-time trust boundary or add build-time confinement evidence. |
| Milestone 11 remediation review | SQL provision profile scanning counts `--profile` operands after a `--` separator. | Milestone 18 | Stop option recognition at `--` and add operand regressions. |
| Milestone 11 remediation review | Host-tool qualification validates evidence-file presence but not the behavior contained in those files. | Milestone 18 | Bind qualification records to named executable tests and reject evidence removal or replacement. |
| Milestone 12 remediation review | Function catalog keys use the final identity segment while live function, operator, and cast identities include generated overload suffixes. | Milestone 18 | Align lookup keys and canonical identities, then add overloaded function, operator, and cast live regressions. |
| Milestone 12 remediation review | Live function result nullability is inferred from PostgreSQL `proisstrict`, which does not prove a non-null result and diverges from DDL normalization. | Milestone 18 | Use a sound canonical nullability rule and prove live-versus-DDL parity for strict and non-strict functions. |
| Milestone 12 remediation review | Live and declarative view definitions, check expressions, constraint names, and identity-column facts do not yet normalize to one semantic representation. | Milestone 18 | Canonicalize these representations and expand live parity beyond the simple table, column, and primary-key case. |
| Milestone 12 remediation review | Domains are registered before ranges, so a domain over a user-defined range can fail closed despite a representable base type. | Milestone 18 | Order registration by dependency or use a deterministic fixpoint, with domain-over-range live evidence. |
| Milestone 12 remediation review | Generated array forms of domains and composites fail closed even when their canonical element forms are representable. | Milestone 18 | Add canonical generated-array handling and live round-trip evidence without fabricating fallback types. |
| Milestone 12 remediation review | Catalog SQL still selects dead fabricated `sifr_type` and composite-field literals that normalization always replaces. | Milestone 18 | Remove the dead literals and keep one authoritative type-normalization path. |
| Milestone 12 review follow-up | Pull acceptance rebuilds live authority while retaining declarative source paths and fingerprints, which can misstate the resulting profile authority. | Milestone 18 | Define and enforce the accepted pull authority transition, including source and fingerprint metadata. |
| Milestone 13 remediation review | A schema-changing diamond can apply a merge after only one parent because the engine does not require every merge parent in applied history. | Milestone 14 | Reject an incomplete merge or reject the graph at compile time. Add a live schema-changing diamond that proves every parent is applied. |
| Milestone 13 remediation review | The closed runtime plan omits minimum server version, required capabilities, transaction requirement, author, and creation time. | Milestone 14 | Emit the runtime fields needed for server-major and operator qualification. Change the format version if the artifact shape changes. |
| Milestone 13 remediation review | `AmbiguousPath` now reports only the absence of a matching path. | Milestone 14 | Rename or split the error before provider migration diagnostics become public. |
| Milestone 13 initial review | Explicit reverse plans compile but have no runtime execution entry point. | Milestone 14 | Execute only declared reverse plans and report exact operator actions in the PostgreSQL rollback matrix. |
| Milestone 13 initial review | Migration contracts and HIR have no complete `.sifr` source-to-tool command path. | Milestone 18 | Wire migration source discovery, checked lowering, artifact build, and command execution through the integrated compiler and tool path. |
| Milestone 14 initial review | Schema observation excludes `sifr_internal`, which can silently hide a user-owned schema with the same name. | Milestone 18 | Define the reserved namespace authority, reject user collisions, expose the policy through schema lifecycle tools, and add live collision evidence. |
| Milestone 14 remediation review | PostgreSQL escape string constants such as `E'it\\'s'` can end quoted-literal scanning early and re-enter migration keyword classification. | Milestone 18 | Parse escape-string constants without scanning their contents as SQL keywords. Add transactional statements whose escape strings contain every autocommit family. |
| Milestone 15 remediation review | PostgreSQL-generated constraint identities are positional. An application-only UNIQUE, CHECK, or foreign key declared before a required constraint can rename the required object and reject a valid structural superset. | Milestone 18 | Derive stable constraint identities from normalized content or match constraint objects structurally. Add earlier-extra UNIQUE, CHECK, and foreign-key regressions. |
| Milestone 15 review follow-up | The permanent portable `.sifr` fixture and qualification checker prove the harness contract but do not compile a complete source-level `SqlSchema` constrained-generic program. | Milestone 18 | Wire the language surface through parsing, typing, lowering, and application build. Replace token-only evidence with executable positive and negative fixtures. |
| Milestone 15 review follow-up | The PostgreSQL portable-capability vocabulary does not yet account for every supported cast, array, function, `LATERAL`, and aggregate `FILTER` behavior. | Milestone 18 | Complete provider-owned capability classification and add one negative portability case for every capability family. |
| Milestone 15 review follow-up | `ProviderAnalysis::validate` proves that the object account contains effects and result sources, but it cannot independently prove that a provider reported every predicate-only object. | Milestone 18 | Add provider-side statement-to-catalog property tests that require every referenced catalog object in the closed account. |
| Milestone 15 review follow-up | A column-list-free INSERT accounts for every concrete relation column, so specialization against a narrower structural requirement rejects it. | Milestone 18 | Specify and test this intentional strictness, or define a provider-neutral declared-column rule without hiding application-only write behavior. |
| Milestone 15 review follow-up | The compatibility normalization response shim has no production consumer. | Milestone 18 | Remove the unused shim after integrated source-level schema requirement loading uses the authoritative response path. |
| Milestone 16 initial review | MySQL schema validation reads the expected character set from the schema that it validates. | Milestone 18 | Give the parser an owned expected character set. Add a negative schema-input mismatch test. |
| Milestone 16 initial review | A table-level character set without a collation inherits the profile collation instead of the character set default. | Milestone 18 | Model the server collation rule. Add differential DDL cases for character-set-only table declarations. |
| Milestone 16 initial review | An abandoned MySQL savepoint records an unused completion flag and has no release behavior. | Milestone 18 | Define savepoint abandonment semantics. Remove the flag or add bounded release and poison behavior with live tests. |
| Milestone 16 remediation review | Carrier cancellation uses a detached cleanup task. Evidence can arrive after the returned error, and a connection ID can retire first. | Milestone 18 | Join the bounded cleanup or add an explicit completion handoff. Attach cleanup evidence to the returned error and test the retirement race. |
| Milestone 16 remediation review | SQL profile authority derives the expected dialect family from provider output instead of profile configuration. | Milestone 18 | Store the resolved provider family in the profile configuration. Compare it before dialect-mode validation. |
| Milestone 16 remediation review | The `sifr_package` all-target strict Clippy run has existing test lint debt. | Milestone 18 | Remove the lint debt before the final workspace gate. Keep the Milestone 16 library target clean. |
| Milestone 17 remediation review | SQLite migration scope validation applies the exact one-statement guard only to DDL steps. | Milestone 18 | Apply an execution-safe statement policy to SQL data, assertion, and backfill steps, even for a hand-edited `graph.json`. |
| Milestone 17 remediation review | SQLite one-row cardinality probing can be constrained by a profile row limit of one. | Milestone 18 | Separate the two-row cardinality probe from collection limits, matching the common explicit cardinality overflow policy. |
| Milestone 17 remediation review | SQLite `==` is retained as two operator tokens and normalizes differently from `=`. | Milestone 18 | Canonicalize the SQLite equality alias and add equivalent-query normalization evidence. |
| Milestone 17 remediation review | Conflict-form detection scans the whole INSERT and can mistake an `OR replace(...)` expression for an INSERT conflict action. | Milestone 18 | Restrict conflict detection to the statement action prefix and add INSERT-SELECT predicate regressions. |
| Milestone 17 remediation review | Two SQLite helper functions remain behind `#[allow(dead_code)]`. | Milestone 18 | Remove the unused compatibility helpers after the integrated component and editor paths prove they have no consumer. |
| Milestone 17 remediation review | SQLite corruption evidence uses a wholly invalid byte blob rather than a damaged valid database. | Milestone 18 | Add truncated-page or page-corruption cases while retaining the existing malformed-file boundary test. |
| Milestone 17 remediation review | The common migration compiler passes provider-normalized SQL into executable data and assertion steps even when literal normalization is lossy. | Milestone 18 | Separate executable SQL from semantic normalization and add literal-bearing migration cases across providers. |
| Milestone 18 remediation review | Generated modules can emit dotted datetime, UUID, JSON, and network type paths without importing their modules. | [SQL platform review follow-ups](../active/ad-hoc-schema-first-sql-platform-review-follow-ups.md) | Import or alias every generated annotation through one closed, compile-tested mapping. |
| Milestone 18 remediation review | A missing schema-profile import makes an otherwise profile-shaped `.query` decorator a silent non-SQL decorator. | [SQL platform review follow-ups](../active/ad-hoc-schema-first-sql-platform-review-follow-ups.md) | Add a targeted missing-import diagnostic without recapturing unrelated decorators. |
| Milestone 18 remediation review | Live sequence filtering also removes user sequences attached with `ALTER SEQUENCE ... OWNED BY`. | [SQL platform review follow-ups](../active/ad-hoc-schema-first-sql-platform-review-follow-ups.md) | Distinguish implementation-owned identity sequences from explicit user-owned sequences. |
| Milestone 18 remediation review | Native build qualification compares Cargo check plans but does not prove byte-for-byte linked artifact reproducibility. | [SQL platform review follow-ups](../active/ad-hoc-schema-first-sql-platform-review-follow-ups.md) | Add deterministic linked-artifact hashing for the supported native targets. |

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
- SQL validation: four variants passed with no failure. The permanent checker
  covers 19 platform parts, three providers, six domains, and 30 invariants.
  The plan-local checker covers all 19 milestones and both status tables.
- Dependency validation: all 14 crates and six `libpg_query` sources match their
  release authorities. The all-feature qualification package and HTTP-enabled
  shared runtime compile with the locked graph.
- TLS validation: the inverse feature graph selects `rustls/aws_lc_rs` and
  `tokio-rustls/aws_lc_rs`. It contains no `ring` feature edge for either crate.
- Repository validation: documentation structure, verification-runner self-tests,
  Rust formatting, Python compilation, diff hygiene, and file-size checks passed.
- Gate disposition: no compiler source changed. The phase rule therefore skipped
  both Sifr repository gates.
- Review round 1: agent found one shared TLS provider regression on
  `94c2bfb6362bede53f13cefbef92775f29119c94`.
- Remediation: the final candidate preserved the accepted AWS-LC-RS provider and
  hardened the dependency and profile mutation checkers.
- Review round 2: agent returned `SATISFIED` with no blocking finding on
  `7f3f6bc2cb5209b5cae64576cf1f11288be31f75`. The [published review](https://github.com/sifr-lang/sifr/pull/3582#issuecomment-5465914743)
  is keyed by that candidate.
- Evidence scope: positive, negative, mutation, and property checks apply to this
  contract lock. The inventory assigns later fuzz and performance evidence to
  their implementation owners.
- Unrelated failures: none.
- Next action: implement Milestone 1 from the merged and recorded mainline.

### Milestone 1 closure record

- Status: completed and merged.
- Starting commit: `f428c0d2c62a7f7914ebc6424e32ad4ba79fbf36`.
- Initial reviewed candidate: `78bd3e50b103fa71dc2dacbc0daa25c3ad587d5f`.
- Remediation candidate: `56f131e1bd8593527c0425fd6a67508683a861dd`.
- Final candidate: `ae4f04c4153ec4e3c861d893d0484c3ba4e9aa38`.
- Pull request: [#3585](https://github.com/sifr-lang/sifr/pull/3585).
- Merge commit: `1173cd9e20ef68480deb1fa0ed459615e10120d5`.
- Acceptance disposition: all six Milestone 1 criteria are satisfied.
- Owned result: the compiler preserves PEP 750 template segments, typed holes,
  conversions, recursive format specifications, evaluation order, and source
  mappings. The formatter and frontend preserve the same structure. Generated
  Rust uses one private, move-only carrier and evaluates each hole one time.
- Compiler validation: all 140 type-system tests passed on the current-main
  integration. The focused template suites passed 3 lowering, 3 codegen, 3
  frontend, 1 formatter, and 3 syntax tests.
- Native validation: the evaluation-order fixture built and ran. The equality
  fixture failed with `SIFR-TYPE-0002`, as required.
- Hardening validation: all 15 property variants passed, including both template
  runs. All 26 fuzz-smoke variants passed. The two template HIR snapshots have
  active lowering-inventory rows.
- SQL integration: the permanent SQL checker, its 11 mutations, all 8 dependency
  mutations, the coverage matrix, the verification taxonomy check, and both
  plan-record checks passed. The file-size and HIR guardrails also passed.
- Create-PR gate: the one allowed run stopped because the template runtime was
  absent from the retained-preamble allowlist. The specific allowlist guard and
  its self-test passed after the correction. The full gate did not run again.
- Merge gate: the one allowed run stopped on M0 SQL coverage metadata and
  generated-code taxonomy debt. M1 corrected all SQL-owned findings. The
  generated-code owner corrected its findings on `main`. The full gate did not
  run again.
- Review round 1: agent returned `NOT SATISFIED` on
  `78bd3e50b103fa71dc2dacbc0daa25c3ad587d5f`. It found discarded recursive
  format metadata, incomplete snapshot and source-map evidence, and a bad
  negative-fixture marker.
- Remediation: one batch retained recursive metadata and added the required
  snapshot, every-offset, and negative-fixture evidence.
- Review round 2: agent returned `SATISFIED` with no blocking or follow-up finding
  on `56f131e1bd8593527c0425fd6a67508683a861dd`. The [published review](https://github.com/sifr-lang/sifr/pull/3585#issuecomment-5466254530)
  records both rounds. No third review ran.
- Base integration: current `main` changed exact-integer compiler paths after
  the review. The merge kept both architectures, and all affected focused tests
  passed. The review and gate limits prohibited more rounds.
- Unrelated failure: the pre-existing
  `method_receiver_conventions_and_source_ranges` snapshot lacks a
  core-language inventory row. The receiver-semantics owner records this defect
  in `ad-hoc-pre-v1-compatibility-removal.md`.
- Next action: implement Milestone 2 from the merged and recorded mainline.

### Milestone 2 closure record

- Status: completed and merged.
- Starting commit: `30e8bc375e3901e57d285daba9836a0d9e1c1e38`.
- Initial reviewed candidate: `930b1e6dfbd5d63f8766353f5a6307b47d324be8`.
- Final candidate: `dd7ac3cdc83ce7d44a7ac4a527be313e67e1d231`.
- Pull request: [#3588](https://github.com/sifr-lang/sifr/pull/3588).
- Merge commit: `955e97f6db04850cd78767597e12a379a99387f0`.
- Owned result: immutable structural records now have canonical field order and
  order-independent identity. The compiler supports exact field access, named
  construction, named destructuring, generic records, and nested records.
- Type rules: ordinary assignment is exact. Width conversion exists only for a
  shared-borrow call boundary. Owned projection uses an explicit consuming HIR
  node and moves selected fields without implicit clones.
- Rust result: code generation interns canonical generic layouts. The generated
  layouts support equality, hashing, ordering, display, and IPC serialization.
  Physical `Copy` classification follows every nested Rust field type.
- Focused validation: all 1,161 codegen tests passed. All 1,060 lowering tests
  passed, with one ignored test. All 146 type-system tests passed. Frontend tests,
  the workspace check, and the native structural-record fixture also passed.
- Property and guard validation: field-permutation, identity, width, capability,
  layout-reuse, and build-order cases passed. The HIR and file-size guards passed.
- Create-PR gate: the one allowed run used `930b1e6d`. All functional areas
  passed. The cold runtime-platform area exceeded its time budget.
- Merge gate: the one allowed run used `dd7ac3cd`. Rust interop, coverage, core
  language, CPython differential, Python interop, diagnostics, and runtime
  platform passed. Runtime platform passed 30 variants with zero failures.
- External gate failure: algorithmic compatibility stopped because this
  worktree did not initialize the LeetCode corpus gitlink. The corpus path was
  absent. The owning algorithmic issue records this external failure. The gate
  did not run again.
- Review round 1: agent found implicit clones in owned projection. It also found
  a mismatch between Sifr copy rules and physical Rust `Copy` behavior.
- Remediation: the final candidate adds a projection HIR node, direct field
  moves, bounded Rust `Copy` derives, and recursive physical-copy checks.
- Review round 2: agent verified both original mechanisms. It found a new case
  for projection from a multi-field all-`int` record. The [published review](https://github.com/sifr-lang/sifr/pull/3588#issuecomment-5466767448)
  records the exact case. The phase rule assigns this case to Milestone 18 and
  prohibits a third review.
- Deferred record fields: the same review noted invalid generated Rust for
  `Callable` and union fields. Milestone 18 owns both native closure cases.
- Next action: implement Milestone 3 from the merged and recorded mainline.

### Milestone 3 closure record

- Status: completed and merged.
- Starting commit: `aec0452484f43732d46b791f909922c001e5eb8b`.
- Initial reviewed candidate: `3b861261a1b832bfedb107fddd76f782fe071688`.
- Remediation reviewed candidate: `3d97d7e35961ed5efb7203dd0f152cdbb0705992`.
- Final candidate: `9764def9e5a43199d15b83beb1efb5d53112da25`.
- Pull request: [#3592](https://github.com/sifr-lang/sifr/pull/3592).
- Merge commit: `9badcfc4aaaebfcd458d6b16359e4cd425daa6d6`.
- Owned result: resolved packages can register exact compiler-component
  identities and protocol ranges. The host executes the closed WIT contract in
  Wasmtime without WASI or ambient capabilities. It validates manifests,
  package-graph ownership, protocol envelopes, bounds, cache identities,
  diagnostic namespaces, and deterministic output.
- Qualification result: the checked-in non-SQL component is built from Rust
  source and analyzes the request inside WebAssembly. It proves typed-hole
  parsing, diagnostics, source maps, dependencies, semantic plans, runtime
  lowering, caching, determinism, malformed-output rejection, and exact tooling
  provenance. Qualification passes on all four supported native targets.
- Focused validation: all 14 component tests, all 32 diagnostics tests, and all
  four package component-resolution tests passed. All six SQL-platform variants
  and all five coverage-matrix variants passed. Strict Clippy, formatting,
  diagnostic catalog and baseline checks, HIR maintainability, documentation
  generation, and the file-size guard passed.
- Create-PR gate: the one allowed run used
  `8d9f1f06aabe91c22da38d636cebe7fe3af7cee0`. It stopped on missing
  coverage classification for the new crate and feature. The classification was
  fixed and its focused coverage checks passed. The gate did not run again.
- Merge gate: the one allowed run used the exact final candidate
  `9764def9e5a43199d15b83beb1efb5d53112da25`. Rust interop, coverage, core
  language, CPython differential,
  Python interop, diagnostics, and runtime platform passed. The gate then
  stopped because the LeetCode profile manifest pointed to a corpus gitlink that
  was not initialized in this worktree. The owning Phase 31 record tracks this
  external repository-state failure. The gate did not run again.
- Review round 1: agent found that relaxed SIMD was not disabled, component
  diagnostics were outside the canonical global registry, and the non-SQL guest
  returned host-selected canned responses.
- Remediation: the host now disables relaxed SIMD and canonicalizes NaNs. The
  compiler owns `SIFR-COMPONENT-0001` through `0009`, with catalog and baseline
  gates. A real Rust-built component parses each request and derives its result
  inside WebAssembly.
- Review round 2: agent verified the original sandbox and diagnostic mechanisms.
  It found two new mechanisms: warm cache hits can bypass a lower input bound,
  and response spans can name documents outside the request. The [published
  review](https://github.com/sifr-lang/sifr/pull/3592#issuecomment-5467260005)
  records both cases. The phase rule assigns them to Milestone 18 and prohibits
  a third review.
- Next action: implement Milestone 4 from the merged and recorded mainline.

### Milestone 4 closure record

- Status: completed and merged.
- Starting commit: `aedf70bf95698338abaaee6c758d363f8e19db37`.
- Initial reviewed candidate: `cf8cad6b3735a51c922f662993c624ee94f49578`.
- Remediation reviewed candidate:
  `04e00c51b4cc5413bcd10675696455bcaec4cf3a`.
- Final candidate: `99ad2269d9eae60a5319167d033dc86266306c9c`.
- Pull request: [#3595](https://github.com/sifr-lang/sifr/pull/3595).
- Merge commit: `40facaf98de8014e3976f5165db59dd478c8dfb9`.
- Acceptance disposition: all eight Milestone 4 criteria are satisfied for the
  merged authority contract. The two newly found consumer-hardening mechanisms
  are explicitly assigned to Milestone 6.
- Owned result: packages declare named profiles with one exact locked provider,
  checked-in schema sources, schema evidence, strictness, and closed session
  modes. Resolution proves package containment and stable source identity.
- Authority result: package build and check read and hash the sources. They send
  bounded artifacts to the provider's exact `.schema` component without WASI.
  The driver validates the response, builds canonical authority, parses the
  generated Sifr module, and includes its identities in the build cache.
- Contract result: `sifr_sql_contract` owns immutable `SchemaIR`, normalized
  fingerprints, object diffs, minimum compatible slices, profile identities,
  nominal generated modules, static symbol lookup, and runtime manifests.
- Focused validation: all nine contract tests, all 14 component tests, both
  driver schema-profile tests, and 149 package tests passed. The unavailable
  external demo-subrepository test was excluded and remains owned by the
  pre-v1 compatibility-removal issue.
- Verification result: all ten schema-profile variants and the coverage matrix
  passed. Strict contract, driver, and package Clippy passed. Formatting, diff
  hygiene, HIR maintainability, and the file-size guard passed.
- Create-PR gate: the one allowed run used
  `04e00c51b4cc5413bcd10675696455bcaec4cf3a`. It stopped before compilation
  because all four profiles omitted the required `schema-profiles` suite.
  `99ad2269d9eae60a5319167d033dc86266306c9c` added the four assignments, and
  the focused profile-assignment matrix passed. The gate did not run again.
- Merge gate: the one allowed run used the exact final candidate
  `99ad2269d9eae60a5319167d033dc86266306c9c`. SQL and repository static checks,
  core language, CPython differential, Rust interop, and coverage passed. The
  gate reached 29 of 30 Python-interop library examples, then stopped on the
  existing `sqlite-context` typed-continuation compiler failure. The emitted
  Rust excellence issue assigns this exact defect to its active Item 3A. The
  gate did not run again.
- Review round 1: agent found local-name collisions, an arbitrary session-mode
  value channel, no production source-to-provider authority path, and
  unsupported verification claims. The [published review](https://github.com/sifr-lang/sifr/pull/3595#issuecomment-5467626475)
  records the exact evidence.
- Remediation: generated types use qualified paths; session modes are bounded
  identifiers; package build and check execute the offline authority pipeline;
  source identities are portable; verification uses executable component and
  driver cases; slice, overload, static-symbol, and host-limit behavior is
  stricter.
- Review round 2: agent verified the original authority and credential
  mechanisms. It found two new mechanisms: emitted-name encoding is not
  injective, and the claimed compiler profile registry has no production
  queryable consumer. The [published review](https://github.com/sifr-lang/sifr/pull/3595#issuecomment-5467626559)
  records both cases. The phase rule assigns them to Milestone 6 and prohibits
  a third review.
- Architecture update: the architecture overview and SQL documents now name the
  profile authority pipeline, its ownership boundary, and its runtime contract.
  The roadmap status did not change because the phase remains active.
- Next action: implement Milestone 5 from the merged and recorded mainline.

### Milestone 5 closure record

- Status: completed and merged.
- Starting commit: `f83bb85c7b54f618ecf8a17019c942bbd788e59a`.
- Initial reviewed candidate: `f71d1bb5d5f266355402185d9a88f7717f4708ab`.
- Remediation reviewed candidate:
  `f7d729ed6d8828d2907c0016cd1dbd7defd11ca7`.
- Final candidate: `f7a3e5a3523f9cd90309f457a78ba7220fabe6d5`.
- Pull request: [#3597](https://github.com/sifr-lang/sifr/pull/3597).
- Merge commit: `7f2382ae68053ca1cde737c51a687ebd701e2e2d`.
- Acceptance disposition: all eight Milestone 5 criteria are satisfied for the
  common contract layer. Runtime-specific transaction transitions are assigned
  to Milestone 9. Compiler-to-runtime cardinality and effect preservation is
  assigned to Milestone 6.
- Owned result: `sifr_sql_contract` defines provider-neutral database and Sifr
  values, exact, fallible, and rejected bind relations, profile-scoped codecs,
  the complete cardinality lattice, read/write effects, provider analysis, and
  typed common diagnostics.
- Runtime result: the driver-free `sifr_sql_runtime` crate owns redacted encoded
  parameters and requests, checked ownership-state handles, resource bounds,
  structured runtime errors, and panic containment at the provider-future
  boundary.
- Type result: fixed-width integers, generic checked narrowing, decimals with
  precision and scale, floats, temporal values, text, bytes, UUID, JSON, enums,
  arrays, domains, composites, ranges, network families, custom codecs,
  unsigned providers, and SQLite dynamic affinity have explicit tested rules.
  Nullable normalization and custom types nested through arrays and ranges use
  the selected profile's registry.
- Diagnostics result: `SIFR-SQL-0001` through `SIFR-SQL-0008` are active typed
  diagnostic codes with generated catalog, documentation, baselines, and an
  executable qualification fixture.
- Focused validation: 9 contract tests, 9 runtime tests, two compile-fail
  ownership doctests, and all 32 diagnostics tests passed. Strict Clippy passed
  for all three crates. The diagnostics rules area, SQL common qualification,
  formatting, file-size guard, and HIR maintainability guard passed.
- Create-PR gate: the one allowed run used
  `472cd7d0b84484957e3cffc7b9ec1e51289d1fca`. Every preceding check passed,
  then diagnostic coverage rejected string-only use of the eight new codes.
  The final candidate replaced that path with direct `DiagnosticCode::SQL_*`
  mappings and passed the focused coverage and diagnostics checks. The gate did
  not run again.
- Merge gate: the one allowed run used the exact final candidate. Repository
  guardrails, Rust interop 10/10, coverage readiness 4/4, core language, and
  CPython differential passed. Python interop passed 28 of 30 variants. The
  existing `readonly-check-doctor` 300-second and `binding-authoring`
  180-second host timeouts are documented by the archived representative
  performance-budget owner and Phase 40 evidence ledger. The SQL workstream did
  not change a timeout, threshold, or waiver, and the gate did not run again.
- Review rounds 1 and 2 found incomplete bind/codec closure, redaction gaps,
  qualification drift, decimal representation loss, nullable normalization,
  and recursive custom-type handling. The implementation corrected those
  mechanisms in one common model.
- User-authorized exact-SHA adjudication then exposed and corrected the final
  `Range<Custom>` read/bind asymmetry. The [satisfied review](https://github.com/sifr-lang/sifr/pull/3597#issuecomment-5468015187)
  approved that candidate with no blocker.
- The create-PR diagnostic correction changed the exact candidate. A final
  [exact-SHA review](https://github.com/sifr-lang/sifr/pull/3597#issuecomment-5468075122)
  verified the dependency topology and every active diagnostic-code mapping on
  `f7a3e5a3523f9cd90309f457a78ba7220fabe6d5`; it returned `SATISFIED` with no
  blocking finding.
- The initial review's later transaction-transition mechanism is assigned to
  Milestone 9. Its compile-time/runtime cardinality and effect round trip is
  assigned to Milestone 6.
- Architecture update: the SQL architecture documents now name the common
  compiler contract, driver-free runtime boundary, codec registry, error and
  ownership rules, and provider qualification matrix. The roadmap status did
  not change because the phase remains active.
- Next action: implement Milestone 6 from the merged and recorded mainline.

### Milestone 6 closure record

- Status: completed and merged.
- Starting commit: `381e6f454027b74d6f95a5fd74d65868314d4204`.
- Initial reviewed candidate: `9daf4a9c3a88125b94da2819153cdfbc936534f5`.
- Remediation reviewed and final candidate:
  `0abd5109f52d4f0fcf7ce45763d1e11aa95f809c`.
- Pull request: [#3599](https://github.com/sifr-lang/sifr/pull/3599).
- Merge commit: `9944bdd4509fb6a32dd3528c9de93a01509e226f`.
- Acceptance disposition: all 16 Milestone 6 criteria are satisfied. Provider
  placeholder rewriting, package capability lookup, editor reverse-name use,
  and integrated registry hardening are assigned to their first production
  consumers in Milestones 7, 10, and 18.
- Contract result: `sifr_sql_contract` owns stable template identities, callable
  template signatures, `RowOf`, explicit cardinality adapters, full fragment
  context and transformations, scope and alias hygiene, predicate combinators,
  unsafe-syntax audit grants, a production profile registry, and one reversible
  generated-identifier codec.
- Compiler result: `sifr_frontend` consumes the registry and provider analysis,
  validates exact parameter codecs, uses normal nominal and structural Sifr
  types, and lowers templates, ordered captures, adapters, cardinality, and
  effects into closed `sifr_ir` query nodes.
- Runtime result: `sifr_sql_runtime` exposes only `QueryTemplate` and
  `BoundQuery` as query states. Binding evaluates and encodes once in source
  order, stores owned values, conditionally supports `Clone`, and consumes the
  bound query to produce an execution request with the compiler's exact
  cardinality and effect records.
- Generated-name remediation: agent found that boundary underscores could make
  two different generated paths collapse. The final codec escapes leading and
  trailing underscores, and the exact collision pair now has injectivity and
  round-trip regression coverage.
- Focused validation: all 23 contract tests, 12 runtime tests, two ownership
  doctests, three frontend query tests, and two driver profile tests passed. The
  SQL area passed 19 of 19 variants, and coverage readiness passed four of four.
  Strict Clippy passed for every changed Rust target. Formatting, diff hygiene,
  file-size, HIR maintainability, and driver maintainability checks passed.
- Create-PR gate: the one allowed run used the exact final candidate. Every
  reached area passed, but the required cold-cache cleanup made runtime-platform
  take 264 seconds against a 120-second host-time budget. The repository rule
  prohibits using the first cold-cache run as host-sensitive performance
  evidence. The gate did not run again.
- Merge gate: the one allowed run used the exact final candidate. Repository
  guardrails, Rust interop 10/10, coverage 4/4, core language 5/5, CPython
  differential 2/2, Python interop 30/30, diagnostics 184/184, and runtime
  platform all passed. Warm runtime-platform took 30 seconds, confirming the
  cold-cache artifact. The gate then stopped on the externally owned LeetCode
  corpus-root configuration before algorithmic compatibility. This is the same
  uninitialized-corpus repository-state failure already recorded by the owning
  verification phase. The gate did not run again.
- Review round 1: agent returned `NOT SATISFIED` because generated paths ending
  and starting with underscores were not injective.
- Remediation: the codec now escapes every boundary underscore and tests the
  exact colliding paths. The [published exact-SHA review](https://github.com/sifr-lang/sifr/pull/3599#issuecomment-5468661005)
  records the remediation and validation evidence.
- Review round 2: agent returned `SATISFIED` on the exact final candidate with no
  blocking findings. The remaining suggestions are assigned in the deferred
  reviewer table, and the phase rule prohibits a third review.
- Architecture update: the architecture overview and SQL documents now define
  query states, binding and execution ownership, fragment hygiene, profile
  lookup, generated names, and the compiler/runtime cardinality-effect round
  trip. The roadmap status did not change because the phase remains active.
- Next action: implement Milestone 7 from the merged and recorded mainline.

### Milestone 7 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `f7697e147c55657d485548a4a0ffc568a604796c`.
- Initial reviewed candidate: `45065c710e9134bc46cbf59c35b19fb91338264d`.
- Remediation reviewed and final candidate:
  `6cd74514901a66988ddd011f85b5eac7cfaf81e4`.
- Pull request: [#3602](https://github.com/sifr-lang/sifr/pull/3602).
- Merge commit: `46f1d06d8e72da7f068503c6a973d72acca7ccb5`.
- Acceptance disposition: the merged candidate delivers all nine listed
  Milestone 7 surfaces and closes every initial-review defect. The permitted
  second review found two new soundness mechanisms. The user rule prohibits a
  third review and assigns both mechanisms to the next item, Milestone 8. The
  phase cannot use M7 as final cardinality or write-completeness evidence until
  those rows close.
- Parser result: the provider embeds the exact Milestone 0 `libpg_query` source
  for PostgreSQL 13 through 18. The build selects one source, maps its raw tree
  into owned nodes, preserves exact spans and parameter slots, and records
  deterministic source content checksums.
- Component result: six checked-in `wasm32-wasip2` components execute the real
  parser and analyzer. The artifact authority binds their binary hashes to the
  parser commits, Rust and WIT guest inputs, WASI SDK `33.0`, WIT Bindgen
  `0.61.1`, and WASI-Virt `0.2.0`. The empty-linker host grants no import and
  bounds fuel, stack, instances, memories, tables, input, and output.
- Semantic result: catalog and DDL ingestion preserve declaration order, real
  view results, qualified nominal identities, casts, operators, aggregates,
  aliases, correlations, set operations, writes, conflict predicates,
  `RETURNING`, exact codecs, nullability, and provider diagnostics across the
  common boundary.
- Capability result: unsafe SQL authority comes only from the exact root package
  in the resolved package graph. A dependency cannot lend or consume the root
  grant. Production fragment compilation consumes the resolver directly.
- Focused validation: native parser and provider tests passed for PostgreSQL 13,
  14, 15, 16, 17, and 18. All six final components passed exact capability-free
  host execution. The live PostgreSQL 13-18 matrix passed version, parameter,
  result, nullability, conflict-write, and diagnostic comparisons. Component,
  contract, package-capability, and provider tests passed. SQL qualification,
  mutation, dependency, formatting, selected strict Clippy, HIR, file-size, and
  diff checks passed.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the global profile validator requires the new live SQL
  suite in `create-pr`, while that profile forbids live network access. The gate
  did not run again.
- Merge gate: the one allowed run used the same exact final candidate. It stopped
  at the same live-suite and offline-profile contradiction before tests. The gate
  did not run again. Milestone 18 owns the verification-profile model correction.
- Review round 1: agent found gaps in the real server differential, view analysis,
  declaration order, raw syntax shapes, operator and aggregate semantics, codec
  identities, qualification, explicit null writes, executable components,
  provider diagnostics, E strings, source checksums, spans, and unsafe-capability
  authority.
- Remediation: one batch implemented the missing provider mechanisms, regression
  fixtures, exact tooling authority, real component artifact pipeline, and live
  server evidence.
- Review round 2: agent verified every original remediation on the exact final
  candidate. It returned `NOT SATISFIED` for two newly discovered mechanisms:
  scalar-subquery aggregate leakage into outer cardinality and explicit
  `DEFAULT` bypass of required-value checks. The
  [published review](https://github.com/sifr-lang/sifr/pull/3602#issuecomment-5469555240)
  assigns both to Milestone 8 and prohibits a third review.
- Additional follow-up: Milestone 8 also owns explicit `A_Star`, unary-minus,
  and grouping diagnostics. Milestone 18 owns the opt-in live-suite profile and
  default-mode evidence comparison.
- Architecture update: the SQL documents now define the PostgreSQL source and
  artifact authority, parser and analyzer boundary, capability-free component
  host, exact type and codec model, live evidence, and build tooling. The roadmap
  status did not change because the phase remains active.
- Next action: start Milestone 8 with the two deferred soundness mechanisms, then
  complete every advanced PostgreSQL semantic and public-stability criterion.

### Milestone 8 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `8341f37dd9ad4d7bf59fa81801f133a7caf6047b`.
- Initial reviewed candidate: `a568927347837c6e2220583c96c50c87aac057b0`.
- Remediation reviewed and final candidate:
  `94fbb6e0f9abe53af8f48b6e73c4068f20dfc929`.
- Pull request: [#3604](https://github.com/sifr-lang/sifr/pull/3604).
- Merge commit: `e18e0a92d5e2f4b587a850db3991009587a825d2`.
- Acceptance disposition: the merged candidate implements all nine Milestone 8
  criteria and closes the Milestone 7 aggregate, `DEFAULT`, syntax-node, and
  grouping findings. The permitted second review found one new semantic-flag
  mechanism. The user rule prohibits a third review and assigns that mechanism
  to Milestone 18. Final qualification cannot use set-result ordering evidence
  from Milestone 8 until that row closes.
- PostgreSQL result: the provider owns arrays, ranges, composites, domains,
  enums, JSON operations, windows, named-window validation, CTE scope, row
  locking, provider DDL, contextual unknown literals, and exact qualified
  built-in type identities for PostgreSQL 13 through 18.
- Proof result: outer joins, aggregates, `CASE`, `COALESCE`, scalar subqueries,
  provider functions, primary-key grouping, unique predicates, limits, writes,
  and set operations produce checked nullability and sound cardinality
  intervals. Property families cover limits, offsets, filtered singletons,
  grouping, joins, scalar subqueries, and window placement.
- Stability result: private stars expand by exact source span in top-level,
  set, CTE, derived-table, subquery, and `RETURNING` projections. Any star marks
  the enclosing analysis, and exported registration rejects it. Stable query
  signatures include parameters, result records, cardinality, effects, and
  referenced and affected schema objects.
- Contract result: join, select-list, values, returning, and assignment fragments
  preserve or transform scope, results, and effects explicitly. Batches enforce
  provider parameter limits and explicit chunking. Custom codecs bind one exact
  database identity to checked, fallible encode and decode operations.
- Build result: application `sifr build` writes canonical
  `sifr-query-signatures.json` atomically and reports its path. Normal
  source-level query lowering does not populate entries yet; Milestone 18 owns
  the integrated non-empty package proof.
- Focused validation: the milestone SQL suite passed four of four variants.
  PostgreSQL 13, 14, 15, 16, 17, and 18 passed native parser, semantic, and
  regression suites. All six regenerated components passed capability-free host
  execution. Contract semantic completion passed 4/4, fragment contracts passed
  6/6, PostgreSQL regressions passed 6/6, and application build-output behavior
  passed 12/12. Strict focused Clippy, formatting, PostgreSQL qualification,
  file-size, HIR maintainability, and diff checks passed.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the global profile validator requires
  `postgresql-live-differential` in the offline `create-pr` profile. The gate did
  not run again. Milestone 18 already owns this profile-model correction.
- Merge gate: the one allowed run used the same exact final candidate. It stopped
  before tests on the same live-suite and offline-profile contradiction. The
  gate did not run again.
- Review round 1: agent returned `NOT SATISFIED` because set operands, CTEs, and
  derived tables could hide `SELECT *` from expansion and exported-query
  rejection. The [published review](https://github.com/sifr-lang/sifr/pull/3604#issuecomment-5470202950)
  records the exact candidate and findings.
- Remediation: one context-wide span map records every star expansion. The
  analyzer applies replacements in reverse source order, marks the enclosing
  analysis, preserves nested set flags, and tests direct, set, CTE, and derived
  forms. Distinct unions also cap duplicate-sensitive positive lower bounds at
  one.
- Review round 2: agent verified that the original star blocker is resolved. It
  returned `NOT SATISFIED` for a new mechanism: set results can inherit an
  operand-only `deterministic-order` flag. The
  [published review](https://github.com/sifr-lang/sifr/pull/3604#issuecomment-5470203053)
  assigns it to Milestone 18 and prohibits a third review.
- Additional follow-up: Milestone 10 owns structured projection quick fixes and
  nested-star diagnostic wording. Milestone 18 owns non-empty application
  signature integration and locking rejection below set operands.
- Architecture update: the PostgreSQL compiler document now defines advanced
  types, queries, nullability, cardinality, star expansion, fragments, custom
  codecs, and query-signature behavior. The roadmap status did not change
  because the phase remains active.
- Next action: implement Milestone 9 from the merged and recorded mainline.

### Milestone 9 closure record

- Status: complete and merged.
- Starting commit: `1bbcbb7feae75c917d1e0ebf5df472dd5660a455`.
- Initial reviewed candidate: `518244bc2f42a68b01d13adb76c776e14db8b6c0`.
- Final reviewed candidate: `258f13a9a77a88ea5dceae2628b811d3593e5330`.
- Pull request: [#3611](https://github.com/sifr-lang/sifr/pull/3611).
- Merge commit: `9dc0e55e09f455e737647417bcb4c584f1e35391`.
- Runtime foundation: the workspace locks the Milestone 0 Tokio, Rustls,
  `tokio-rustls`, `tokio-postgres`, `postgres-types`, and
  `tokio-postgres-rustls` versions and feature allowlists. The provider uses the
  raw drivers. Common pooling, verification, session, cache, transaction,
  cleanup, and resource policies remain in `sifr_sql_runtime`.
- Verification and execution: an unverified pool has no execution surface.
  Exact and compatible schema verification preserve expected and observed
  evidence, including absence facts. Every connection acquisition and reset
  applies and verifies typed session state before execution.
- Runtime behavior: explicit execute, fetch, scalar, warm, cardinality, stream,
  transaction, savepoint, retry, cancellation, and cleanup paths are bounded.
  Terminal operations consume their handles. Retry creates a fresh transaction.
  Errors preserve primary failure and bounded secondary cleanup evidence without
  exposing credentials or permitting user-triggered panics.
- Qualification: common and provider Rust tests passed. Strict focused Clippy,
  formatting, file-size, HIR maintainability, mutation, qualification, and diff
  checks passed. The SQL compiler and PostgreSQL runtime area suites passed all
  six variants, including capability-free component execution. The live runtime
  matrix passed exact PostgreSQL 13, 14, 15, 16, 17, and 18 images with session,
  transaction, stream, cache, retry, cancellation, cleanup, load, and malformed
  data coverage.
- Create-PR and merge gates: not run. This milestone changed no compiler source,
  so the user-approved phase rule makes both Sifr repository gates inapplicable.
- Review round 1: agent returned `NOT SATISFIED` because the ordinary PostgreSQL
  compiler qualification still used a stale guest-source identity after the root
  manifests changed. The
  [published review](https://github.com/sifr-lang/sifr/pull/3611#issuecomment-5471125620)
  records the exact candidate and evidence.
- Remediation: the component record now contains the deterministic identity for
  the final manifests. The ordinary compiler qualification and its SQL profile
  suite pass again. The same batch corrects inverted named-schema cache
  invalidation and adds a regression test.
- Review round 2: agent returned `SATISFIED` on the final candidate. The
  [published review](https://github.com/sifr-lang/sifr/pull/3611#issuecomment-5471125721)
  verifies the original blocker, remediation, and full milestone contract.
- Deferred follow-up: Milestone 18 owns malformed PostgreSQL boolean wire-length
  classification, narrower component identity inputs, and the final disposition
  of provider-level schema-cache invalidation. No third review is permitted.
- Architecture update: the common and PostgreSQL runtime ownership, tooling,
  verification, session, transaction, cleanup, and resource contracts are
  documented. The roadmap status did not change because the phase remains active.
- Next action: implement Milestone 10 from the merged and recorded mainline.

### Milestone 10 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `1a9886c300d612dec54b55dad4cef878372b6ca5`.
- Initial reviewed candidate: `8b25b010f0a7e4ee4e9bd5b91c4ba6476c516752`.
- Remediation reviewed and final candidate:
  `08e864bf6f16fdf2fc0569690dbcfcc8587f3677`.
- Pull request: [#3617](https://github.com/sifr-lang/sifr/pull/3617).
- Merge commit: `35926da67775004d555b862596bd04c991620852`.
- Incremental result: frontend-owned identities include templates, typed holes,
  fragments, provider settings, component and compiler semantics, and exact
  schema dependency slices. The bounded runtime cache preserves deterministic
  results, aliases base requests to slice-qualified keys, and supports precise
  dependency invalidation.
- Editor result: lossless virtual SQL documents provide bidirectional source
  maps. Exact static-SQL routing preserves ordinary Sifr behavior inside holes.
  Completion, hover, definition, references, rename, parameter and result
  information, formatting, structured fixes, generated-identifier display, and
  relation-aware fragment completion use live provider and schema contracts.
- Runtime result: package profiles load canonical `SchemaIR`, execute the frozen
  provider component, decode the common analysis envelope, surface provider
  diagnostics, and propagate LSP cancellation before and during provider work.
- Focused validation: the incremental-editor suite passed 2/2, including 51
  analysis tests, 134 frontend tests, and 81 LSP tests. SQL mutation passed 9/9.
  Common contracts passed 27/27, driver profile tests passed 2/2, and the
  PostgreSQL compiler passed 14/14, including all six regenerated components.
  Strict workspace Clippy, formatting, diff, HIR maintainability, driver
  maintainability, file-size, qualification, and qualification self-tests passed.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the global profile validator requires the PostgreSQL live
  differential and runtime suites in the offline `create-pr` profile. It did not
  run again. Milestone 18 owns the profile-model correction.
- Merge gate: the one allowed run used the same exact final candidate. It stopped
  before tests on the same live-suite and offline-profile contradiction. It did
  not run again.
- Review round 1: agent returned `NOT SATISFIED` because interpolation holes lost
  Sifr routing, schema and provider analysis were not on the production path,
  incremental identities were test-only, and LSP cancellation was not wired to
  live requests.
- Remediation: one batch connected exact routing, frozen package profiles,
  provider execution and diagnostics, slice-qualified caching, current-profile
  dependency fingerprints, relation scope, reverse identifier display, and
  request cancellation. It regenerated all six provider components and added
  focused regression evidence.
- Review round 2: agent verified the four original mechanisms and returned
  `NOT SATISFIED` for one new regression: SQL profile preparation can abort the
  complete analysis host. The
  [published review](https://github.com/sifr-lang/sifr/pull/3617#issuecomment-5471683360)
  records the exact final candidate. The user rule prohibits a third review and
  assigns failure isolation to Milestone 11.
- Additional follow-up: Milestone 18 owns workspace-stable cache lifetime,
  independently observed schema changes, newly added object invalidation, and
  the existing live-suite profile contradiction.
- Architecture update: the compiler and language-server documents define live
  SQL profile loading, virtual documents, source routing, provider diagnostics,
  cache identities, dependency slices, invalidation, budgets, and cancellation.
  The roadmap status did not change because the phase remains active.
- Next action: close the inherited SQL-profile failure-isolation regression,
  then implement Milestone 11 from the merged and recorded mainline.

### Milestone 11 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `7e8da6870b50e157b51b4f326a5b8ac2c42d11a3`.
- Initial reviewed candidate: `6375936723697e982776e9701e488214d9813e18`.
- Remediation reviewed and final candidate:
  `f0790104720bfc943d420c484a1325e25a1d1f53`.
- Pull request: [#3619](https://github.com/sifr-lang/sifr/pull/3619).
- Merge commit: `a20219da54e36d73baf6893d0da0a6e7958c51e6`.
- Graph result: one exact tools workspace member exports direct, normal,
  workspace-member binary entries. Application graph derivation rejects every
  dependency closure that reaches the tools member or a declared tool package.
- Identity result: `sifr tools lock` writes the committed
  `sifr-tools.lock.json`. It binds the tools manifest, Cargo lock, entry points,
  capabilities, and registry or path package identities. Invocation verifies
  the record before build and again before direct execution.
- Execution result: Sifr resolves Cargo and Rustc to absolute programs, removes
  runner and wrapper substitutions, builds the exact host-target binary, hashes
  it, and executes it directly. The native sandbox fails closed and enforces
  environment, credential, project-read, project-write, network, and subprocess
  grants. Combined output is limited to 10 MiB and an over-limit tool is killed.
- Provisioning result: `sifr sql test provision --profile <name>` validates and
  canonicalizes the common connection manifest. Credential helpers use an
  executable plus argument array. Cleanup has one typed namespace, resource,
  and command route.
- Inherited result: SQL profile initialization failures preserve normal package
  analysis in both overlay and direct `AnalysisHost::open_project` paths while
  surfacing an explicit diagnostic.
- Focused validation: package host-tool tests passed 7/7. The direct CLI
  integration passed 1/1 and covered missing and checked locks, direct provision,
  capability denial, self-subprocess denial, combined-output termination,
  unknown namespaces, built-in typo behavior, and persisted source drift.
  Provisioning passed 1/1. The full analysis suite passed 53/53 before the final
  host-tool-only hardening, and the final isolation rerun passed 2/2. Strict
  workspace Clippy, formatting, HIR maintainability, the 3,510-file size guard,
  and host-tool qualification mutations passed.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because `create-pr` omits the required `host-tools`,
  `postgresql-live-differential`, and `postgresql-live-runtime` suites. It did
  not run again. Milestone 18 owns the profile-model correction.
- Merge gate: the one allowed run used the same exact final candidate. It stopped
  before tests because `merge` omits the same three required suites. It did not
  run again.
- Review round 1: agent returned `NOT SATISFIED` for advisory capabilities,
  unusable external-tool topology, non-persisted hashes, Cargo execution
  substitution, invocation-only contamination checks, and source-grep evidence.
  The [published review](https://github.com/sifr-lang/sifr/pull/3619#issuecomment-5472000009)
  records the exact initial candidate.
- Remediation: one batch added the committed tool lock, workspace-member
  topology, build-then-direct execution, OS confinement, bounded output,
  application-time contamination checks, configuration hardening, structured
  provisioning commands, and behavioral evidence.
- Review round 2: agent verified the original corrections and returned
  `NOT SATISFIED` because Linux validation does not provision or diagnose the
  Bubblewrap prerequisite. It also found a new declared-namespace typo-routing
  defect. The
  [published review](https://github.com/sifr-lang/sifr/pull/3619#issuecomment-5472000301)
  records the exact final candidate. The user rule prohibits a third review and
  assigns both findings and the other follow-ups to Milestone 18.
- Documentation result: the architecture records workspace-member tool
  topology, committed identity, direct execution, sandbox capabilities,
  structured provisioning, and standard-library separation. The roadmap status
  did not change because the phase remains active.
- Exact next action: implement Milestone 12 schema lifecycle tools from the
  merged and recorded mainline.

### Milestone 12 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `c560ae5da1e85f433df7e5bb6a034cd8e75c1f54`.
- Initial reviewed candidate: `bd8b04dd28faab312d8527979a3e1096f72255d2`.
- Remediation reviewed and final candidate:
  `ba3629d634b15c192af46b66a4972c77683fd194`.
- Pull request: [#3621](https://github.com/sifr-lang/sifr/pull/3621).
- Merge commit: `73a69f691d3e6bbf49342f1901dfedf55782312a`.
- Command result: the PostgreSQL host tool implements `schema pull`,
  `schema validate`, and `schema build`. Pull shows the semantic diff before a
  replacement. Non-interactive writes require `--accept`.
  Validation is read-only and reports object-level changes plus affected query
  identities. Build verifies deterministic double production before publication.
- Authority result: source, snapshot, migration, and optional live authorities
  resolve through one common policy and reject conflicts. Credentials are
  rejected from inputs and generated values. Incomplete provider facts fail
  closed instead of receiving fallback Sifr types.
- Artifact result: one atomic directory transaction publishes the canonical
  snapshot, fingerprint, runtime manifest, generated Sifr module and metadata,
  dependency index, and artifact manifest. Schema tools never emit application
  query signatures.
- PostgreSQL result: catalog introspection covers namespaces, relations,
  columns, constraints, indexes, sequences, identity metadata, views, enums,
  domains, composites, ranges, and multiranges. It also covers functions,
  operators, casts, collations, extensions, triggers, capabilities, and dialect
  facts. Catalog type names pass through the provider parser and type registry.
  This process produces the same canonical `SchemaIR` vocabulary as declarative
  DDL.
- Focused validation: common schema-tool tests passed 6/6. The complete
  PostgreSQL provider suite passed 20/20. Strict focused Clippy, formatting,
  diff hygiene, HIR maintainability, and the 3,522-file size guard passed.
  The live matrix passed the exact PostgreSQL 13, 14, 15, 16, 17, and 18 image
  digests. Every pull reloaded through `PostgresCatalog`. Every major built the
  complete artifact set. The matrix asserted domain and composite Sifr types.
  PostgreSQL 18 produced an empty live-versus-DDL semantic diff for a simple
  table with a primary key.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the offline `create-pr` profile omits the required
  `host-tools`, `schema-tools`, `postgresql-live-schema-tools`,
  `postgresql-live-differential`, and `postgresql-live-runtime` suites. It did
  not run again. Milestone 18 owns the profile-model correction.
- Merge gate: the one allowed run used the same exact final candidate. It
  stopped before tests on the same required-suite contradiction and did not run
  again. The [published gate evidence](https://github.com/sifr-lang/sifr/pull/3621#issuecomment-5472348573)
  records both exact invocations.
- Review round 1: agent returned `NOT SATISFIED` because live catalogs used a
  parallel vocabulary instead of compiler-canonical `SchemaIR` types. They also
  fabricated `str` fallbacks for domains and composites. The
  [published review](https://github.com/sifr-lang/sifr/pull/3621#issuecomment-5472301449)
  records the exact initial candidate.
- Remediation: one batch added provider-owned catalog type normalization,
  canonical enum, domain, composite, range, and multirange data, separate
  PostgreSQL 14-or-newer multirange introspection, fail-closed unsupported-type
  behavior, full graph reload evidence, and live-versus-DDL parity evidence.
- Review round 2: agent verified both original blockers and returned `SATISFIED`
  on the final candidate. The
  [published review](https://github.com/sifr-lang/sifr/pull/3621#issuecomment-5472337308)
  records the result. Its new mechanism findings and suggestions are assigned
  to Milestone 18 in the deferred-review ledger. No third review ran.
- Documentation result: the schema-tool document and architecture index define
  authority resolution, command ergonomics, atomic artifacts, provider catalog
  ownership, deterministic behavior, secret rejection, and qualification. The
  roadmap now records Milestones 0-12 complete while the phase remains active.
- Exact next action: implement Milestone 13 migration compiler and engine from
  the merged and recorded mainline.

### Milestone 13 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `22a205317448696306cd34631fe88ccb16876f8e`.
- Initial reviewed candidate: `8e51ecb0bd56be522e9b7530feae7566c0f8b8ff`.
- Remediation reviewed and final candidate:
  `c6acf3da7518c2c67a844036c183f45e91440a15`.
- Pull request: [#3625](https://github.com/sifr-lang/sifr/pull/3625).
- Merge commit: `6d28649ff935d880516b7cf5fabe03a9c4e85a39`.
- Acceptance disposition: all 11 Milestone 13 criteria are satisfied.
- Compiler result: the compiler validates stable DAG identities, parents,
  checksums, provider constraints, and input and output fingerprints. It proves
  every baseline path reaches the canonical head.
- State result: each DDL step produces an exact intermediate `SchemaIR` state.
  Nominal `MigrationPlan[S]` transitions are affine. Data callbacks receive a
  scoped `MigrationDb[S]` and can use only objects in state `S`.
- Step result: SQL and Sifr data steps, Boolean assertions, bounded resumable
  backfills, progress keys, recovery points, and transaction boundaries have
  checked contracts. Raw DDL requires reflection or an explicit verified effect.
- Recovery result: the engine records locks, heads, checksums, fingerprints,
  step states, duration, recovery points, and backfill progress. It rejects
  drift, changed checksums, invalid assertion results, provider panics, and
  ambiguous recovery.
- Rollback result: reverse plans exist only when the migration declares them.
  The compiler proves that each reverse path returns to its parent fingerprint.
  The engine never synthesizes a destructive reverse operation.
- Boundary result: the host tool lowers the compiler graph to a closed,
  runtime-owned `MigrationExecutionPlan`. The application runtime does not link
  `sifr_sql_contract`, `sifr_compiler_component`, or Wasmtime.
- Artifact result: the host tool emits deterministic `graph.json`,
  `schema.json`, `impact.json`, and `artifact-manifest.json` files. It publishes
  the complete set with one atomic directory replacement.
- Focused validation: the permanent migration suite passed 6/6 variants. All
  tests passed in `sifr_sql_contract`, `sifr_sql_runtime`, `sifr_sql_tool`, and
  `sifr_frontend`. This includes six compiler contracts, one frontend test, six
  engine tests, two artifact tests, graph properties, and identity fuzz smoke.
- Repository validation: workspace Clippy, strict changed-crate Clippy,
  formatting, locked metadata, SQL contracts, qualification mutation tests,
  HIR maintainability, diff hygiene, and the 3,543-file size guard passed.
  Runtime dependency inspection found only Serde, `sifr_runtime`, and Tokio in
  the normal closure.
- Review round 1: agent returned `NOT SATISFIED` because the application runtime
  linked the compiler-only SQL contract and pulled Wasmtime into its dependency
  closure. The [published review](https://github.com/sifr-lang/sifr/pull/3625#issuecomment-5472602197)
  records the exact initial candidate.
- Remediation: one batch added the closed runtime plan, host-side lowering,
  closed artifact serialization, a dependency regression guard, and stable
  topological branch execution evidence.
- Review round 2: agent verified the original blocker and returned `SATISFIED`
  with no blocking finding. The
  [published review](https://github.com/sifr-lang/sifr/pull/3625#issuecomment-5472663451)
  records the exact final candidate. It found one new schema-changing merge
  mechanism and three suggestions. They are assigned to Milestone 14 under the
  continuation rule. No third review ran.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the profile omits six required SQL suites, including the
  new `migration-engine` suite. It did not run again.
- Merge gate: the one allowed run used the same exact final candidate. It
  stopped before tests on the same profile contradiction and did not run again.
  The [published gate evidence](https://github.com/sifr-lang/sifr/pull/3625#issuecomment-5472666358)
  records both commands. Milestone 18 owns the profile-model correction.
- Documentation result: the migration document defines graph, type, step,
  recovery, artifact, and dependency boundaries. It also records equivalent
  merge schemas and fail-closed transaction recovery.
- Exact next action: implement Milestone 14 PostgreSQL migration qualification
  from the merged and recorded mainline. Start with the deferred merge-parent
  mechanism, then complete explicit reverse execution and runtime provider
  metadata before the live matrix.

### Milestone 14 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `d65ce110dedefc2128857756eed1e9d46ffffdad`.
- Initial reviewed candidate: `e3d90614fe3149b1f49e24f1743089c1b9a3af55`.
- Remediation reviewed and final candidate:
  `a8f02da6258f2938697cc696e316467c8a970902`.
- Pull request: [#3627](https://github.com/sifr-lang/sifr/pull/3627).
- Merge commit: `5b2aa585f4e52f400e73e6932d35140266a17948`.
- Acceptance disposition: all six Milestone 14 criteria are satisfied.
- Reflection result: PostgreSQL DDL reflection covers the provider capability
  inventory. Statements that cannot add a new reflected object retain an exact
  declared effect or become explicit opaque effects.
- Plan result: the closed runtime plan v2 records provider constraints,
  capabilities, minimum server version, transaction requirements, author,
  creation time, checksums, parents, fingerprints, and declared reverse steps.
  Runtime validation rejects plan tampering before database execution.
- Execution result: transactional and autocommit-required DDL use separate
  boundaries and exact recovery points. Forward execution rejects rollback
  progress. Explicit rollback executes only declared reverse plans.
- Safety result: profile and bootstrap advisory locks, incomplete merge parents,
  concurrent starts, checksums, heads, schema fingerprints, and recovery state
  fail closed. Import records a truthful baseline without fabricating history.
- Operator result: `sifr sql migration plan`, `import`, `apply`, and `rollback`
  report exact actions through the host-tool boundary. PostgreSQL migration
  support remains outside the generated application runtime dependency closure.
- Focused validation: the registered migration-engine and PostgreSQL migration
  suites passed 9/9 variants. The pinned live matrix passed fresh creation,
  multi-step upgrade, merge, interruption, resume, failure, rollback, import,
  drift, locking, and cross-profile bootstrap cases on PostgreSQL 13 through 18.
- Repository validation: workspace and changed-target Clippy with warnings
  denied, formatting, HIR maintainability, the 900-line source guard,
  qualification checks, and mutation checks passed.
- Review round 1: agent returned `SATISFIED` with no blocking finding on the
  initial candidate. The
  [published review](https://github.com/sifr-lang/sifr/pull/3627#issuecomment-5475270496)
  records the exact SHA. One remediation batch hardened rollback direction,
  quoted-literal classification, additional autocommit families, plan lowering,
  exact qualification inventories, bootstrap locking, and dead reflection code.
- Review round 2: agent verified every remediation and returned `SATISFIED` with
  no blocking finding. The
  [published review](https://github.com/sifr-lang/sifr/pull/3627#issuecomment-5475270707)
  records the exact final candidate. It found one new escape-string classifier
  mechanism. The initial review also identified reserved-schema ownership. Both
  are assigned to Milestone 18. No third review ran.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the profile omits eight required SQL platform suites. It
  did not run again.
- Merge gate: the one allowed run used the same exact final candidate. It
  stopped before tests on the same profile contradiction and did not run again.
  The [published gate evidence](https://github.com/sifr-lang/sifr/pull/3627#issuecomment-5475279278)
  records both commands. Milestone 18 owns the profile-model correction.
- Documentation result: the PostgreSQL migration document defines reflection,
  plan, transaction, recovery, import, rollback, supported-major, tooling, and
  operator contracts. It pins latest stable tooling through the phase records.
- Exact next action: implement Milestone 15 schema polymorphism and portable
  constraints from the merged and recorded mainline.

### Milestone 15 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `868e86f27c77456d0c91a3678b2213d725c0836b`.
- Initial reviewed candidate: `26cac063c4ec85a31daa0cfd56bdfc797ed6c7bd`.
- Remediation reviewed and final candidate:
  `31eb20ff2e634bd299bc620b76705a141f702244`.
- Pull request: [#3630](https://github.com/sifr-lang/sifr/pull/3630).
- Merge commit: `d878ecfe1a39befe49106d1018b93ac6fcb07610`.
- Requirement result: provider-normalized DDL produces one canonical structural
  requirement. Proofs bind provider identity, minimum server version, declared
  objects, exact scalar semantics, table-membership containment, and explicit
  capabilities without adding a second schema language.
- Specialization result: every profile exports a compile-time-only schema
  witness. Constrained generic specialization erases the witness, selects one
  concrete profile, and binds execution to a verified pool, connection, or
  transaction with the same nominal profile identity.
- Envelope result: PostgreSQL analysis owns the closed capability and schema
  object accounts. Predicate-only, join-only, conflict-target, assignment, and
  result objects participate in undeclared-object rejection. Callers cannot
  narrow the provider account or request runtime provider dispatch.
- Integration result: package manifests resolve requirement artifacts through
  the locked provider component. The driver prepares proofs and generated
  profile modules offline. The provider-neutral harness and PostgreSQL 13-18
  components use the same common contracts.
- Focused validation: the permanent schema-polymorphism suite passed 7/7. The
  affected contract, frontend, package, driver, and PostgreSQL tests passed.
  Every regenerated PostgreSQL 13-18 component executed in the
  capability-free host. PostgreSQL qualification and its mutation check passed.
- Repository validation: workspace Clippy and PostgreSQL all-target Clippy with
  warnings denied, formatting, HIR maintainability, the 900-line source guard,
  and diff hygiene passed.
- Review round 1: agent returned `NOT SATISFIED` on the initial candidate. It
  found exact table-membership comparison and an incomplete effect-derived
  object envelope. The
  [published review](https://github.com/sifr-lang/sifr/pull/3630#issuecomment-5476737851)
  records the two blockers and non-blocking language-integration follow-ups.
- Remediation: table columns, constraints, and unique sets use containment while
  scalar and primary-key properties stay exact. Provider analysis now returns a
  complete accessed-object account, with normalized PostgreSQL and undeclared
  predicate/assignment regression evidence. All six checked-in components were
  regenerated from the pinned toolchain.
- Review round 2: agent verified the provider-owned object account and returned
  `NOT SATISFIED` for a newly discovered positional constraint-identity
  mechanism. The
  [published review](https://github.com/sifr-lang/sifr/pull/3630#issuecomment-5477033586)
  assigns stable constraint identity and the remaining qualification hardening
  to Milestone 18. The user rule prohibits a third review.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the profile omits nine required SQL platform suites. It
  did not run again.
- Merge gate: the one allowed run used the same exact final candidate. It
  stopped before tests on the same profile-composition defect and did not run
  again. The
  [published gate evidence](https://github.com/sifr-lang/sifr/pull/3630#issuecomment-5477046866)
  records both commands. Milestone 18 owns the existing profile-model repair.
- Documentation result: the SQL architecture and schema-polymorphism document
  define the normalized requirement, structural proof, witness, specialization,
  capability, object-account, and execution contracts in direct language.
- Exact next action: implement Milestone 16 MySQL provider completion from the
  merged and recorded mainline.

### Milestone 16 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `247b46580756ec92cafa4d13dff6a503835b7156`.
- Initial reviewed candidate: `67360fcbcc79b57cff1ab5f235537d60bc01643b`.
- Remediation reviewed and final candidate:
  `5d585f4b0b05d3572a1cededcc81c0fc1823b40f`.
- Pull request: [#3635](https://github.com/sifr-lang/sifr/pull/3635).
- Merge commit: `70de2f82bb501c52443055933d1c2ba4ba53e05e`.
- Compiler result: the MySQL package owns its LALRPOP grammar, lexer, AST,
  recovery mode, semantic analysis, schema normalization, and diagnostics.
- Runtime result: the runtime uses raw `mysql_async` connections and a
  Sifr-owned pool. It supports verified sessions, streams, transactions,
  statement caching, and bounded cancellation.
- Tool result: the host package supports schema lifecycle commands, migration
  commands, live catalog normalization, and safe test provisioning.
- Dependency result: the manifests use LALRPOP 0.23.1, `mysql_async` 0.37.0,
  `mysql_common` 0.37.3, Tokio 1.53.1, and Rustls 0.23.43.
- Provider result: MySQL 8.4, 9.7, and 26.7 have separate component artifacts.
  Each supported line passed the complete live provider matrix.
- Focused validation: parser differential, compiler, property, runtime,
  cancellation, catalog, migration, provisioning, and performance tests passed.
- Integrated validation: the SQL platform adapter passed 62/62 variants.
  Component qualification and mutation tests passed after each component rebuild.
- Repository validation: strict Clippy passed for all M16 targets. Formatting,
  diff hygiene, HIR maintainability, and the 900-line source guard passed.
- Review round 1: agent returned `NOT SATISFIED` on the initial candidate. It
  found a non-MySQL session-mode regression and an unbounded cancellation path.
  The [published review](https://github.com/sifr-lang/sifr/pull/3635#issuecomment-5480215882)
  records both blockers.
- Remediation: dialect-mode comparison now limits character inputs to MySQL.
  Both cancellation paths use the same cleanup budget and evidence model.
  New tests cover PostgreSQL profile authority and live carrier cancellation.
- Review round 2: agent verified both corrections and returned `SATISFIED`.
  The [published review](https://github.com/sifr-lang/sifr/pull/3635#issuecomment-5480280529)
  records the exact final candidate. New suggestions belong to Milestone 18.
  No third review ran.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the profile omits eleven required SQL platform suites.
  It did not run again.
- Merge gate: the one allowed run used the same exact final candidate. It
  stopped before tests on the same profile error and did not run again. The
  [published gate evidence](https://github.com/sifr-lang/sifr/pull/3635#issuecomment-5480373044)
  records both commands. Milestone 18 owns the profile correction.
- Documentation result: the MySQL provider document defines compiler, runtime,
  tool, migration, editor, safety, version, and qualification contracts.
- Exact next action: implement Milestone 17 SQLite provider completion from the
  merged and recorded mainline.

### Milestone 17 closure record

- Status: completed and merged under the phase continuation rule.
- Starting commit: `02842e49020146f1b39a415f8b35347bd17c51e7`.
- Initial reviewed candidate: `b76cdf19404c1e04e966243f813d497d95dc6fb2`.
- Remediation reviewed and final candidate:
  `22a87c70d2b41bdb76ec4dc2ebc51ae13f17e25e`.
- Pull request: [#3640](https://github.com/sifr-lang/sifr/pull/3640).
- Merge commit: `4a64e163aecf371738535294fe3daf855a2c9e99`.
- Compiler result: the SQLite component owns pinned grammar validation, exact
  lexical and semantic lowering, affinity, rowid, strict-table, attached-scope,
  query, write, schema normalization, editor, and portable-requirement behavior.
- Runtime result: bundled `rusqlite` connections stay on dedicated joined
  workers. Sifr owns pooling, verification, limits, streaming, transactions,
  statement caching, attached files, bounded interruption, and error mapping.
- Tool result: catalog pull reuses the exact DDL normalizer. Lifecycle commands,
  confined test provisioning, common `graph.json` migration execution, truthful
  baseline import, ledger, drift checks, atomic recovery, and explicit rollback
  are implemented.
- Dependency result: Syntaqlite 0.9.0, `rusqlite` 0.40.2,
  `libsqlite3-sys` 0.38.2, SQLite 3.53.2, Tokio 1.53.1, WASI SDK 33, and
  `wit-bindgen` 0.61.1 are locked. The grammar pin is forced and the actual
  bundled runtime reports version number 3053002 with 51 compile options.
- Focused validation: the SQLite provider area passed 6/6 variants. Compiler,
  property, runtime, catalog, tool, migration, and real bundled-library tests
  passed. The common contract and 160 package tests also passed.
- Repository validation: strict Clippy passed for SQLite compiler, runtime,
  tools, common contracts, and the package library. Formatting, diff hygiene,
  HIR maintainability, and the 900-line source guard passed. The checked-in
  WebAssembly component was rebuilt and its digest, size, target, and identity
  passed qualification.
- Review round 1: agent returned `NOT SATISFIED` on the initial candidate. It
  found compiler/catalog divergence, missing common migration-engine behavior,
  attached-scope, grammar, semantic, runtime-lifecycle, requirement-harness,
  evidence, and documentation gaps.
- Remediation: catalog and DDL use one normalizer; attached files are explicit;
  SQLite parameters, operators, limits, returning, conflicts, DDL, types, and
  rowid rules are exact; provider metadata is separate; open is non-blocking;
  workers join; one-row reads report cardinality; migrations use the common
  engine; and qualification measures the actual bundled library.
- Review round 2: agent verified every original correction and returned
  `SATISFIED` with no blockers on the exact final candidate. The
  [published review](https://github.com/sifr-lang/sifr/pull/3640#issuecomment-5482865565)
  records the verdict. Its new suggestions and one pre-existing common migration
  issue are assigned to Milestone 18. No third review ran.
- Create-PR gate: the one allowed run used the exact final candidate. It stopped
  before tests because the profile omits twelve required SQL platform suites.
  It did not run again.
- Merge gate: the one allowed run used the same exact final candidate. It
  stopped before tests on the same profile error and did not run again. The
  [published gate evidence](https://github.com/sifr-lang/sifr/pull/3640#issuecomment-5482888123)
  records both commands. Milestone 18 owns the integrated profile correction.
- Documentation result: the SQLite provider document defines package ownership,
  locked tools, grammar, schema, runtime, catalog, migrations, editor behavior,
  safety, and executable qualification evidence.
- Exact next action: implement Milestone 18 integrated qualification and phase
  closure from the merged and recorded mainline.

### Milestone 18 closure record

- Status: completed and merged.
- Starting commit: `7ddab7def1bd2c69b9c7e3b1b030c2488ff39973`.
- Initial reviewed candidate: `066300ff185f38b425a884a2225b72990a194e58`.
- Remediation reviewed candidate:
  `c0c6ae255fc605fc58a24d93a15d5a08b8126121`.
- Final candidate: `dce729a051e82a8b11c13d829d82cec434114b6b`.
- Pull request: [#3645](https://github.com/sifr-lang/sifr/pull/3645).
- Merge commit: `83a5f95d143a0f0bd633a01835023ecd16b93ce3`.
- Acceptance disposition: all 13 Milestone 18 criteria are satisfied under the
  session's one-shot gate rule. The two gate failures and their targeted
  corrections are recorded below.
- Integration result: source-level profile queries and migrations use explicit
  imported profile namespaces. The driver no longer captures unrelated `.query`
  decorators or injects every configured profile name into each module.
- Build result: the executable build qualification proves clean, incremental,
  locked, offline, and reproducible Cargo plans. The native run produced 456
  artifacts. The WASI contract run produced 40 artifacts.
- Provider result: the MySQL live matrix passed on 8.4.11, 9.7.2, and 26.7.0.
  PostgreSQL schema-tool, migration, differential, and runtime matrices passed
  on server majors 13 through 18.
- PostgreSQL parity result: explicit numeric casts, domains over ranges,
  generated arrays of domains and composites, column checks, identity columns,
  identity-owned sequences, and view columns and dependencies normalize through
  one DDL and live-catalog contract.
- Component result: all six PostgreSQL components were rebuilt with WASI SDK 33.
  Their recorded hashes and sizes passed qualification. The exact bytes executed
  successfully in the capability-free component host.
- Focused validation: the affected contract, integrated qualification, schema
  profile, common SQL, schema polymorphism, schema tool, PostgreSQL compiler,
  component, and live suites passed. Contract and qualification mutation tests
  also passed.
- Repository validation: workspace Clippy passed with warnings denied. Rust
  formatting, Python compilation, diff hygiene, HIR maintainability, and the
  900-line file-size guard passed.
- Review round 1: agent returned `NOT SATISFIED` on the initial candidate. It
  found unsafe decorator capture, global profile-name injection, declaration-only
  build claims, and missing exact-candidate live evidence.
- Remediation: declaration discovery now requires explicit profile imports. The
  build modes are executable and profile-bound. The complete live lane exposed
  and closed the PostgreSQL type-generation and catalog-parity defects.
- Review round 2: agent returned `SATISFIED` for all four original blockers on
  `c0c6ae255fc605fc58a24d93a15d5a08b8126121`. The final two commits only update
  repository guard classifications. No third review ran.
- Deferred review work: four new non-blocking mechanisms are owned by
  `plans/issues/active/ad-hoc-schema-first-sql-platform-review-follow-ups.md`.
- Create-PR gate: the one allowed run used `c0c6ae255fc605fc58a24d93a15d5a08b8126121`.
  It stopped at a stale source-provider composition-root classification. Commit
  `0af6ef741133b2aec10dfefdf226ffb8a4cf9293` records the exact SQL profile
  composition root. The positive guard and its self-test pass. The gate did not
  run again.
- Merge gate: the one allowed run used `0af6ef741133b2aec10dfefdf226ffb8a4cf9293`.
  It stopped because the six pinned `libpg_query` submodules and pinned WASI-Virt
  submodule were absent from the ownership classifier. Final commit
  `dce729a051e82a8b11c13d829d82cec434114b6b` classifies branch-tracking and
  commit-pinned submodules separately. The positive guard and its mutation
  self-test pass. The gate did not run again.
- Documentation result: the final internal SQL architecture names the compiler,
  provider, runtime, tool, dependency, component, schema, migration, editor,
  build, safety, and qualification contracts that the merged implementation uses.
- Exact next action: work on the separate non-blocking SQL review follow-up issue
  when it is selected. This closed phase has no unfinished milestone.

### External async prerequisite closure record

- Status: complete and merged before Milestone 9.
- Owning issue:
  `plans/issues/archive/ad-hoc-async-cleanup-completion.md`.
- Final candidate: `46bbd40c8bca7538c8331f2ff3f891a98b2e9c88`.
- Pull request: [#3607](https://github.com/sifr-lang/sifr/pull/3607).
- Merge commit: `0f01971c4d00cdf7e888360fc79c2703cbafb327`.
- Runtime, codegen, and lowering suites passed on the reconciled candidate.
- All 19 native cleanup fixtures passed on the reconciled candidate.
- Both exact-SHA agent reviews returned `SATISFIED` with no blocking findings.
- SQL Milestone 18 owns the repository profile contradiction that stopped both
  allowed gates before tests.
- Non-blocking hardening work remains in
  `plans/issues/active/ad-hoc-async-cleanup-review-follow-ups.md`.
- Exact next action: implement Milestone 9 from this recorded mainline.

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

Milestone 18 merged and closed the phase:

- Final status: completed and audited on 2026-09-01.
- Final merge commit: `83a5f95d143a0f0bd633a01835023ecd16b93ce3`.
- Final create-PR profile: ran once and stopped on a stale composition-root
  classification. The classifier correction and its self-test pass. The profile
  did not run again.
- Final merge gate: ran once and stopped on missing ownership rows for seven
  pinned submodules. The ownership correction and its self-test pass. The gate
  did not run again.
- Final whole-phase review: agent remediation verdict `SATISFIED` on
  `c0c6ae255fc605fc58a24d93a15d5a08b8126121` for all original blockers.
- Final capability and verification inventory: complete, owned, executable, and
  free of pending or waived phase rows.
- Deferred out-of-scope work:
  `plans/issues/active/ad-hoc-schema-first-sql-platform-review-follow-ups.md`.
- Archive destination: `plans/issues/archive/ad-hoc-schema-first-sql-platform.md`.
- Exact next action: none for this phase.
