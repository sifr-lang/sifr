# Ad hoc phase: Schema-first SQL platform

Status: planned

Baseline commit: `907fe8e3c2fbe64c5e6afb4ed5fda047b34dc68b`

## Objective

Deliver the complete schema-first SQL architecture in
[`internal_docs/sql_architecture.md`](../../../internal_docs/sql_architecture.md).

The result includes the compiler component platform, structural records, schema
profiles, checked SQL, explicit execution, and verified runtime pools. It also
includes editor support, tool packages, migrations, and three qualified dialects.

This phase has no reduced product tier. Milestones provide implementation order,
not permanent scope cuts. The phase closes only after the full design works as
one coherent system.

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
8. Obtain one exact-candidate external review. Remediate actionable findings and
   repeat review until none remain.
9. Merge the milestone, update this record, update the active issue, and then
   start the next milestone.
10. Record an unrelated failure in its owning issue. Do not broaden this phase
    to absorb it.
11. Update `internal_docs/architecture.md` when component boundaries change.
12. Update `plans/roadmap.md` only when phase or milestone status changes.
13. Regenerate derived schema, protocol, and provider artifacts from their
    authoritative producers. Do not hand-edit derived output.
14. The final milestone runs the only whole-phase review and full repository
    merge gate.

## Sequential milestones

| Milestone | Status | Name | Required outcome |
|---:|---|---|---|
| 0 | pending | Architecture and dependency lock | The final architecture, language dependencies, ownership map, capability matrix, verification inventory, and phase gates are authoritative and machine validated. |
| 1 | pending | Template-string language foundation | Template strings preserve static segments, typed holes, evaluation order, and exact source maps through the full compiler pipeline. |
| 2 | pending | Structural record type system | Immutable records have order-independent canonical identity, width subtyping, deterministic diagnostics, and interned Rust layouts. |
| 3 | pending | Compiler component platform | Resolved packages can provide deterministic sandboxed embedded-language analysis through one closed, versioned, cacheable protocol. |
| 4 | pending | Schema profiles and canonical schema IR | Configuration sources produce exact provider-owned schema graphs, nominal profile types, fingerprints, dependency slices, diffs, and runtime contracts. |
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

- [ ] The SQL architecture contains every locked delivery contract from this
  record and has no version-scoped deferral.
- [ ] Structural records, fixed-width integers, canonical temporal types, network
  address value types, replay-safe callbacks, bounded cancellation cleanup, and
  diagnostic registries have approved language contracts before SQL work begins.
- [ ] A machine-readable ownership map assigns each architecture surface to one
  milestone and one repository owner.
- [ ] A capability matrix lists required PostgreSQL, MySQL, and SQLite grammar,
  schema, runtime, tool, migration, and editor behavior.
- [ ] A verification inventory maps every locked invariant to positive,
  negative, mutation, integration, fuzz, property, or performance evidence.
- [ ] Checkers reject a missing owner, missing acceptance mapping, duplicate
  identity, invalid milestone, unsupported provider claim, and empty gate.
- [ ] Repository architecture and roadmap links resolve to this record and the
  SQL architecture.

### Milestone 1: Template-string language foundation

- [ ] The parser represents static segments and expression holes without
  lowering them to string concatenation.
- [ ] Every hole preserves its Sifr span, virtual-document span, and left-to-right
  single-evaluation order.
- [ ] Type checking supports library APIs that consume typed template strings.
- [ ] HIR and code generation preserve static text and typed-hole metadata.
- [ ] Formatting preserves meaning, indentation, escapes, and hole boundaries.
- [ ] Compile-pass, compile-fail, snapshot, parser fuzz, and source-map property
  tests cover single-line and multiline forms.

### Milestone 2: Structural record type system

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

### Milestone 3: Compiler component platform

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

### Milestone 4: Schema profiles and canonical schema IR

- [ ] `sifr.toml` supports named profiles, exact providers, checked-in sources,
  schema evidence, schema strictness, and session contracts.
- [ ] SQL DDL, provider metadata, and generated definitions normalize into one
  immutable provider-owned `SchemaIR`.
- [ ] The IR represents every object that can affect provider query semantics.
- [ ] Canonical fingerprints are stable across irrelevant input order and reject
  semantic drift.
- [ ] Generated Sifr modules expose profile namespaces, nominal profile types,
  schema identities, and metadata without an ORM or runtime profile value.
- [ ] Object-level semantic diffs and minimum referenced schema slices are exact.
- [ ] Credentials and live connections are absent from normal compilation.

### Milestone 5: Common SQL contracts

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

### Milestone 6: Query and fragment substrate

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

### Milestone 7: PostgreSQL schema and query compiler

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

### Milestone 8: PostgreSQL semantic completion

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

### Milestone 9: PostgreSQL runtime

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
- [ ] Application tests use the exact provider in an ephemeral environment and
  rollback test transactions. No fake database API exists.
- [ ] Malformed protocol and database data cannot reach a user-triggered panic.

### Milestone 10: Incremental compiler and editor experience

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

### Milestone 11: Host tool graph and command runner

- [ ] Cargo resolves a dedicated tools workspace member separately from
  application and target packages.
- [ ] `Cargo.lock`, Cargo metadata, and the tools member configuration record
  exact tool packages, selected entry points, hashes, and capabilities.
- [ ] `sifr <tool-namespace>` executes only the selected package entry point.
- [ ] Built-in namespaces are reserved. Duplicate namespaces are hard errors.
- [ ] File, network, environment, credential-helper, and subprocess capabilities
  require explicit grants.
- [ ] Tool code and dependencies never enter target HIR, generated Rust, linker
  input, sysroot selection, or application artifacts.
- [ ] Cross-compilation uses host tools and target application dependencies without
  graph leakage.
- [ ] Unknown tools, undeclared capabilities, hash drift, and target contamination
  fail closed.

### Milestone 12: Schema lifecycle tools

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

### Milestone 13: Migration compiler and engine

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

### Milestone 14: PostgreSQL migration qualification

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

### Milestone 15: Schema polymorphism and portable constraints

- [ ] Provider-normalized checked-in DDL artifacts define requirements through a
  declared `SchemaIR` subset. Sifr does not add a second schema language.
- [ ] Requirement artifacts describe tables, columns, keys, types, nullability,
  and required provider capabilities.
- [ ] A concrete profile must prove every requirement before specialization.
- [ ] Specialized queries cannot reach undeclared schema objects or provider
  behavior.
- [ ] Portable code declares provider capability constraints explicitly.
- [ ] Each declared provider parses, analyzes, specializes, and validates portable
  SQL independently.
- [ ] There is no silent lowest-common-denominator rewrite or runtime provider
  dispatch for a statically known profile.

### Milestone 16: MySQL provider completion

- [ ] MySQL grammar, name resolution, coercions, collations, unsigned types,
  generated columns, conflict forms, modes, and schema objects are exact.
- [ ] SQL mode and collation inputs participate in fingerprints and caches.
- [ ] The runtime satisfies the common verification, ownership, execution,
  streaming, statement-cache, cancellation, bound, error, and panic-safety
  contracts.
- [ ] Schema tools and migration reflection cover the MySQL capability matrix.
- [ ] Language-server features use MySQL semantics and documentation.
- [ ] Differential, conformance, migration, recovery, fuzz, property, and performance
  suites pass on every supported MySQL version.

### Milestone 17: SQLite provider completion

- [ ] SQLite grammar, affinity, strict tables, rowid, generated columns, conflict
  forms, attached scope, and schema objects are exact.
- [ ] Required SQLite features and minimum library version participate in profile
  validation, fingerprints, and caches.
- [ ] The file runtime satisfies the common verification, ownership, execution,
  streaming, statement-cache, cancellation, bound, error, and panic-safety
  contracts.
- [ ] Schema tools and migration reflection cover the SQLite capability matrix,
  including table-rebuild plans.
- [ ] Language-server features use SQLite semantics and documentation.
- [ ] Conformance, migration, recovery, fuzz, property, corruption, locking, and
  performance suites pass on every supported SQLite version.

### Milestone 18: Integrated qualification and phase closure

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

## Closure evidence template

Each milestone appends one record with:

- exact starting and final commit identities
- implementation summary and owned files
- acceptance rows completed
- focused commands and results
- negative and mutation evidence
- create-PR and merge-gate results when required
- external review rounds and remediations
- pull request and merge identities
- architecture, roadmap, and issue updates
- unrelated failures recorded with their owning issue

The phase cannot close from narrative confidence. Every locked invariant needs an
executable gate or a precise inspected artifact with named ownership.
