# Ad Hoc Phase: Native Pydantic-Sifr Architecture

## Status

Architecture proper was approved on draft PR
[#3014](https://github.com/sifr-lang/sifr/pull/3014). Opus 5 pass 17 returned
`SATISFIED` and approved `milestone_ps_0`. The architecture, conformance
inventory, repository boundary, and demo ownership are approved.
`milestone_ps_1` and `milestone_ps_2` are implemented and merged. The required
`certification_pkg_resource_core` item is complete through merged
[PR #3123](https://github.com/sifr-lang/sifr/pull/3123). `milestone_ps_3` is
implemented and merged through [PR #3138](https://github.com/sifr-lang/sifr/pull/3138).
`milestone_ps_4` is the next sequential work.

Review artifacts:

- [`native-pydantic-sifr-architecture-opus5-review-pass-1.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-1.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-2.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-2.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-3.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-3.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-4.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-4.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-5.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-5.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-6.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-6.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-7.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-7.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-8.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-8.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-9.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-9.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-10.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-10.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-11.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-11.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-12.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-12.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-13.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-13.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-14.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-14.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-15.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-15.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-16.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-16.md)
- [`native-pydantic-sifr-architecture-opus5-review-pass-17.md`](../../reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-17.md)
- [`native-pydantic-sifr-ps2-claude-opus-review-pass-1.md`](../../reviews/active/native-pydantic-sifr-ps2-claude-opus-review-pass-1.md)
- [`native-pydantic-sifr-ps2-claude-opus-review-pass-2.md`](../../reviews/active/native-pydantic-sifr-ps2-claude-opus-review-pass-2.md)
- [`native-pydantic-sifr-ps2-claude-opus-review-pass-3.md`](../../reviews/active/native-pydantic-sifr-ps2-claude-opus-review-pass-3.md)
- [`native-pydantic-sifr-ps3-claude-opus-review-pass-1.md`](../../reviews/active/native-pydantic-sifr-ps3-claude-opus-review-pass-1.md)
- [`native-pydantic-sifr-ps3-claude-opus-review-pass-2.md`](../../reviews/active/native-pydantic-sifr-ps3-claude-opus-review-pass-2.md)
- [`native-pydantic-sifr-ps3-claude-opus-review-pass-3.md`](../../reviews/active/native-pydantic-sifr-ps3-claude-opus-review-pass-3.md)

Milestone delivery records:

- `milestone_ps_1` merged in [PR #3104](https://github.com/sifr-lang/sifr/pull/3104)
  at merge commit `7cc7f714e844e851a0ffe665fd18c669e0e21b99`; the reviewed candidate was
  `e23b80d94f67de3d3ced7dbcca7394efdf5ab6c1`.
- Validation passed workspace formatting/clippy, diagnostics/frontend/lowering/stdlib
  suites, the non-E2E CLI suite, diagnostics rules and baselines, file-size and HIR
  guardrails, two selected native E2E fixtures, and the positive/negative external
  specialization fixtures. The one merge gate reached the CPython differential lane;
  its first Sifr process timed out at 240 seconds under host-wide Cargo contention,
  while the remaining cases passed. The exact timed-out fixture then passed directly
  in 29.6 seconds with canonical output `[7,3,2,4,20]`.
- Opus review of candidate `5b1601ded66556fe04b9674916153726b341b2c6`
  found augmented-assignment token handling and negative integer floor arithmetic
  omissions. Candidate `e23b80d94f67de3d3ced7dbcca7394efdf5ab6c1` corrected both;
  remediation review returned `SATISFIED` with no blocking findings.
- The `milestone_ps_2` contract candidate also repairs a PS1 test-adapter escape:
  the `cfg(test)` driver frontend adapter omitted five `LoweringResult` metadata
  fields added by PS1. The adapter now propagates the complete result, and all
  19 targeted `sifr_driver` Python-interop tests pass.
- The `milestone_ps_2` contract wave merged in
  [PR #3107](https://github.com/sifr-lang/sifr/pull/3107) at merge commit
  `44571561309afaabb7afb419804aa2cc00362193`; the reviewed candidate was
  `77442349c745ae1ad6ad1592be129572e37fb65c`.
- That wave accepted one unversioned structural Rust bridge contract and a
  repository-wide atomic removal plan for `[rust] bridge-version`, with no
  compatibility mode, rewrite, shim, fallback, or active `v2` name. It added
  separately gated future-owned evidence for structural calls and removed-field
  rejection without claiming implementation support.
- Contract validation passed formatting, 19 focused driver tests, all 10
  registered Rust-interop cases and their self-tests, the active-source naming
  sweep, and file-size/lowering maintainability guardrails. Earlier create-PR
  execution passed core, diagnostics, package, and stdlib lanes; its aggregate
  Python lane reproduced a host-contention timeout for a scenario that passed
  standalone.
- Opus reviewed eight published remediation candidates. The final exact-SHA
  round confirmed all prior construction, identity, callback, projection,
  cutover-inventory, ownership, and current-state findings closed and returned
  `SATISFIED` with no blocking findings.
- The contract record accidentally leaked phase-delivery taxonomy into active
  Rust-interop fixtures, compatibility rows, and durable architecture surfaces,
  blocking the repository taxonomy guard on current `main`. The narrow repair
  merged in [PR #3112](https://github.com/sifr-lang/sifr/pull/3112) at merge
  commit `2452dca175d7b6b01068c40b3853c1d7b6d251f6`; its reviewed candidate was
  `087a399fd0d89f3066af04a26d4185c03497283f`.
- That repair moved the two future-owned compatibility rows to the exact durable
  Rust-interop architecture owner without weakening concrete-path or existence
  validation. Taxonomy and compatibility checks/self-tests, all 10 registered
  Rust-interop cases, stable claims, stale drafts, tiers, and file-size/HIR
  guardrails passed. Opus review round 3 returned `SATISFIED` with no actionable
  findings. The create-PR profile then reached the externally owned Python
  readonly/doctor timeout tracked by PR #3110; #3110 was blocked on this taxonomy
  repair, so the fix merged to close the dependency cycle without absorbing its
  code.
- `milestone_ps_2` merged in [PR #3114](https://github.com/sifr-lang/sifr/pull/3114)
  at merge commit `fc2e7b29244e08399f6b59b60a927ea740af5c5b`; the reviewed candidate was
  `7c0aaebdb3c0c752a513c8d21735e67dd453678a`.
- The milestone completed the unversioned structural Rust bridge cutover,
  `sifr.meta.Structural` marker validation, safe structural construction and
  projection, typed callback generation, package-resource support, and the
  native-backed `re.Pattern` contract. It retained class generic bounds and
  excluded nominal newtypes from implicit structural eligibility. No legacy
  bridge mode, compatibility path, fallback, or versioned active name remains.
- Focused validation passed 972 codegen tests; 971 lowering tests with one
  ignored test; generated-runtime, abort-profile, generic-bound, newtype,
  collection, regex, formatting, HIR, file-size, taxonomy, coverage, readiness,
  profile, stable-claim, and all registered Rust-interop checks. The final merge
  lane passed every PS2-owned check and 23 of 25 Python-interop variants. The two
  failures were the externally owned PR #3110 readonly/doctor 120-second timeout
  and the host-sensitive binding-authoring 180-second timeout; the same binding
  path passed in 36.431 seconds with the controlled six-worker allocation.
- Opus remediation passes closed recursive identity/cache and panic-contract
  gaps, restored generic class bounds, and removed duplicate stable-support
  claims. The final exact-candidate review returned `SATISFIED` with no blocking
  findings; per the reviewer skill, that final response remains outside the Git
  tree.
- `certification_pkg_resource_core` merged in
  [PR #3123](https://github.com/sifr-lang/sifr/pull/3123) at merge commit
  `c6f5748870471ba6baa7dcddbbadc1b627576140`. The reviewed candidate was
  `ad9e73fc6c589c9d25a8177b734b6d820ef63d9a`.
- The item certifies `opaque_resource_package_core` through a synthetic
  external package. It covers construction, lifecycle, panic redaction,
  direct-construction rejection, alias invalidation, and use after close.
- The exact-SHA Opus review returned `SATISFIED` with no blocking findings.
  The one merge gate passed Python 25/25, Rust interop 10/10, generated builds
  70/70, E2E 694/694, and 268 hardening variants with zero failures.
- `milestone_ps_3` merged in
  [PR #3138](https://github.com/sifr-lang/sifr/pull/3138) at merge commit
  `2174be634d7d3f9e0053ff893dc9a1d2fc34f64d`. The exact reviewed and gated
  candidate was `1b1955f681ccce33b2ca65b130f381ef3e9f27ca`.
- The milestone adds deterministic compiler-owned static-program identities and
  bytes, a sealed typed program envelope, checked compact node indices, and a
  move-only structural arena. The synthetic package uses one generic structural
  signature probe and one monomorphized executor call. It proves exact and fixed
  integers, bytes, collections, records, moved payloads, typed errors, corrupt
  envelopes, invalid indices, cleanup, cache invalidation, and source/installed
  parity. No compatibility layer, fallback, or legacy static-program path exists.
- Three recorded Opus review passes found and closed unsupported-owner emission,
  duplicate decorator handling, cache-order and cleanup evidence, and nested bytes
  encoding. A fresh exact-integration review then returned `SATISFIED` with no
  blocking or actionable milestone-owned findings; per the reviewer skill, that
  final response remains outside the Git tree.
- The unchanged warm create-PR gate passed. Its preceding cold run was
  functionally green and exceeded only the known external cold-artifact budget
  tracked by issue #3134. The warm receipt SHA-256 is
  `270742a07e27aa2cc704dd25693f93b64ef29b47e10f058037664b608f6db0c5`.
- The one authoritative merge gate exited zero. It passed Python interop 25/25,
  Rust interop 10/10, generated builds 72/72, E2E 694/694, hardening 268 variants
  with zero failures, distribution 66/66, sysroot 2/2, generated-code quality
  7/7, and all representative performance checks. The merge receipt SHA-256 is
  `caa0dc08aa5d6c855fcd49edd5516cc33b004d59d2f879e7a9443815b83b267e`.
- `milestone_ps_4` is the next sequential work.
- Deferred follow-up work: align registry representative-fixture paths with diagnostic
  baselines; align the pre-existing lowering and codegen structural-eligibility
  predicates for fixed-width platform integers, metadata, and imported classes;
  audit pre-epoch fractional timestamp reconstruction; disambiguate imported
  structural metadata/default lookup from colliding local names; accept large negative
  const bounds in integer-boundary decorators; and keep shared floor-arithmetic semantics
  from drifting between frontend const evaluation and runtime execution.

This document is the single planning source of truth for:

- the general compiler capabilities required in `sifr-lang/sifr`,
- the separate `sifr-lang/pydantic-sifr` repository,
- the Sifr `pydantic` package,
- the native `pydantic_sifr_core` Rust crate,
- the Pydantic/Pydantic Core compatibility corpus, and
- the ordered delivery and acceptance gates.

After implementation stabilizes, durable compiler contracts belong in the
corresponding `internal_docs` architecture documents in `sifr-lang/sifr`, while
package/core contracts belong in the `pydantic-sifr` repository. This issue
remains the phase history and decision record.

## Objective

Provide a complete, native, Pydantic-like data contract API for Sifr with:

- statically derived schemas,
- validation and coercion from untrusted inputs,
- typed model construction,
- structured aggregate errors,
- serialization profiles,
- custom validators and serializers,
- type adapters,
- JSON Schema generation,
- Pydantic-familiar APIs where they fit Sifr,
- native performance with no Python runtime, and
- behavior grounded in the battle-tested Pydantic and Pydantic Core corpus.

The end state must preserve Sifr's guarantees:

- fallibility is expressed through `Result`, not exceptions,
- user-triggerable input cannot panic the process,
- exact Sifr integers are not silently narrowed,
- ownership and callback effects remain statically visible,
- package behavior does not require compiler special cases, and
- invalid static schemas fail during checking/build rather than on the first
  production request.

## Problem

Sifr can represent typed classes and compile them to native Rust, but a full
data-contract system requires more than JSON parsing:

- structural schema derivation,
- field metadata and defaults,
- strict and lax conversion policies,
- recursive validation,
- alias selection,
- union ranking and tagged unions,
- complete error locations and aggregation,
- custom validator execution,
- profile-aware serialization,
- schema description, and
- a safe native boundary capable of returning arbitrary validated Sifr types.

Implementing these behaviors directly in the compiler would make validation a
language special case and would force package policy into the compiler release
cycle. Implementing them all as ordinary Sifr code over copied JSON values
would move performance-sensitive recursive execution across the Rust bridge and
would fail to reuse the strongest part of Pydantic's architecture.

Directly depending on or lightly wrapping `pydantic-core` also does not solve
the problem. At the researched revision, its central input, validation,
serialization, error, and result representations are shaped around PyO3 and
Python objects. Removing that coupling would be a permanent high-drift fork,
not a small native adapter.

The required solution is a Sifr-native form of Pydantic's proven frontend/core
split, with a smaller schema designed around static Sifr types rather than
Python's dynamic object model.

## Research Baseline

The architecture was derived from complete local checkouts outside the Sifr
codebase:

| Repository | Researched revision | Role |
| --- | --- | --- |
| `pydantic/pydantic` | `f59e929c999e8b2efc7b12fd0bc1685c1a186be3` | Sole compatibility pin for Pydantic and its in-tree Pydantic Core 2.47.0 |
| `pydantic/pydantic-core` | `383eb95a19433754c0cecf7025b50c26b6d97a36` | Historical architecture/reuse research reference at 2.41.5; not a parity oracle |

Both upstream repositories are MIT licensed. Copied implementation fragments or
test data must retain the required notice and provenance.

The Pydantic checkout's tracked `pydantic-core/` component is the engine version
required by that same Pydantic revision and is therefore the sole engine
semantic oracle. The older standalone checkout informed architectural reuse
decisions only; its tests never enter the compatibility ledger and a behavior
present only there cannot define parity.

Existing Sifr contracts: [`rust_interop_architecture.md`](../../../internal_docs/rust_interop_architecture.md),
[`sifr_sysroot_and_stdlib_architecture.md`](../../../internal_docs/sifr_sysroot_and_stdlib_architecture.md),
[`integer_model.md`](../../../internal_docs/integer_model.md), and
[`architecture.md`](../../../internal_docs/architecture.md).

The compiler substrate also depends on the core rows tracked by
[`rust-interop-runtime-ecosystem-certification.md`](../archive/rust-interop-runtime-ecosystem-certification.md).

### Cross-document authority

This accepted ad hoc phase supersedes the draft implementation scope in
`plans/phases/41_typed_data_model_and_validation.md`; that file becomes a
redirect/history note, and the roadmap's Phase 41 row points here. Phase 42's
typed-extractor dependency is the released `pydantic-sifr` public model/error
contract through certified release `milestone_ps_11`, not a separate
in-compiler validator. Phase 42 remains blocked if that external release does
not occur; it may not add a fallback.

This design does not supersede `internal_docs/integer_model.md` or its locked
serialization-boundary artifact: all integer JSON, schema, and diagnostic
behavior defers to them. `ps_1` owns the accepted const-specialization
diagnostic contract, activates `SIFR-INT-0009`, publishes its error page, and
changes `integer_model.md` from Reserved to Active. `ps_9` owns the later
consumer/schema integration update to the locked boundary artifact before the
external package releases; it does not redefine the code. The
former Phase 41 `Serialize`/`Deserialize` and stdlib `dumps`/`loads` proposal is
intentionally subsumed by the one `TypeAdapter[T]`/`BaseModel` Core Schema
path, not retained as a second compiler or stdlib serialization authority.
`sifr-lang/sifr` deliberately keeps only its general `JsonValue` JSON API;
typed model JSON is owned exclusively by the external package. Phase 42 waits
for `milestone_ps_11` certification and remains blocked rather than adding a
fallback if that release is unavailable.
Rust bridge certification rows remain owned by
`rust-interop-runtime-ecosystem-certification.md`; this phase may consume only
passing rows or an explicitly transferred narrow row recorded in that issue
and the compatibility matrix.

## End-State Decisions

1. `pydantic-sifr` is an external Sifr package in a standalone public GitHub
   repository owned by the `sifr-lang` organization, with the planned location
   [`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr). It is
   not a directory, workspace member, vendored subtree, or submodule of
   `sifr-lang/sifr`.
2. The Sifr package and `pydantic_sifr_core` are separate components in that
   repository and normally release together.
3. The design reuses Pydantic's high-level architecture:
   public package -> declarative Core Schema -> compiled native execution plan.
4. The design does not fork, embed, link, or require Pydantic Core in
   production.
5. The Sifr compiler gains only general structural metaprogramming,
   specialization, callback, and native-package capabilities.
6. The compiler contains no Pydantic, validation, field, model, JSON, or schema
   special cases.
7. `pydantic-sifr` owns the public API, schema derivation, configuration, and
   typed Sifr integration.
8. `pydantic-sifr` owns schema canonicalization and semantic verification in
   deterministic compile-time Sifr code; `pydantic_sifr_core` owns execution,
   input adapters, aggregate errors, and performance-sensitive algorithms.
9. Derived static schemas become immutable schema programs during build.
   There is no runtime schema-compilation path.
10. Core Schema is the sole package authority for validation, serialization,
    and description, while embedding and obeying accepted language-wide
    contracts such as Sifr's locked integer JSON profiles. Serde, Schemars, and
    another validator are not parallel authorities.
11. The structural Rust bridge contract replaces the current versioned schema
    and adds one general, trait-bounded structural call contract. The
    implementation removes the `[rust] bridge-version` manifest field and all
    version-specific compiler/tooling paths; there is no compatibility mode,
    rewrite, shim, or fallback. The structural contract does not add
    Pydantic-specific bridge types or container exceptions.
12. Native decoding returns a validated value arena. The JSON parse tree and
    normalized arena are expected; no third copied bridge-object tree exists.
13. Compiler-generated structural traits materialize a validated source into
    the requested Sifr type and project typed Sifr values to native consumers.
14. `pydantic_sifr_core` invokes those traits through one monomorphized native
    call. It never imports package-generated bridge types.
15. `jiter`, without its Python feature, is the canonical JSON parser.
16. `speedate` is a temporal parsing mechanism where its behavior matches the
    selected Sifr contract; it is not a public temporal representation.
17. `sifr_runtime::json` supplies the authoritative integer-profile helpers;
    Serde and `serde_json` may provide other format/writer mechanisms but do
    not redefine validation, coercion, errors, or schemas.
18. Focused Rust crates are reused for regex, URL, UUID, Base64, IDNA, and
    arbitrary-precision numeric mechanisms.
19. Pydantic and Pydantic Core are development oracles and provenance sources,
    never dependencies of published artifacts.
20. Compatibility means equivalent behavior where Python and Sifr correspond,
    with every divergence documented.
21. The canonical Pydantic-Sifr demo is owned, tested, and released by the
    external `sifr-lang/pydantic-sifr` repository. No package-specific demo is
    added to `sifr-lang/sifr`.
22. Delivery is sequential: implement, validate, review, merge, and release a
    milestone before starting the next.

## Repository Ownership

This repository boundary is part of the architecture, not merely source-tree
organization. `sifr-lang/sifr` supplies and releases the general compiler,
sysroot, package, and native-interop capabilities first. The resulting
`pydantic` Sifr package is then developed, tested, reviewed, released, and
consumed as an external package from its own
[`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr) GitHub
repository.

After `milestone_ps_3`, production implementation work for the package and
native core is tracked and merged in that external repository. This planning
issue records cross-repository dependencies and links the resulting external
issues, pull requests, and releases; it does not move their source into the
Sifr compiler repository.

### `sifr-lang/sifr`

Owns only general language and package substrate:

- compile-time type shape inspection,
- compile-time declaration metadata,
- safe structural construction,
- safe structural projection/visitation,
- specialization of generic package code,
- typed package callback adapters,
- bounded const-specialization issues mapped to compiler-owned diagnostics,
- package-neutral integer-boundary descriptor verification,
- static data emission,
- Rust bridge support required by general native packages,
- package/compiler compatibility declarations, and
- compiler conformance fixtures.

The compiler must be able to explain these features without mentioning
Pydantic. Database mappers, RPC systems, command-line parsers, encoders,
decoders, and other packages must be able to consume the same substrate.

### External package repository: `sifr-lang/pydantic-sifr`

Owns:

- the public Sifr package,
- the native core crate,
- the versioned Core Schema contract,
- compatibility and differential tests,
- fuzz targets,
- benchmarks,
- upstream provenance,
- runnable package demos,
- package documentation, and
- releases.

Recommended layout:

```text
pydantic-sifr/
  Cargo.toml
  Cargo.lock
  sifr.toml
  src/
    __init__.sifr
    model.sifr
    fields.sifr
    adapters.sifr
    validators.sifr
    serializers.sifr
    errors.sifr
    json_schema.sifr
    bridges/
      mod.rs
      core.rs
  backend/
    pydantic_sifr_core/
      Cargo.toml
      src/
        lib.rs
        schema/
        input/
        validators/
        serializers/
        errors/
        arena/
  tests/
    native/
    compatibility/
    differential/
    fuzz/
    provenance/
      upstream_manifest.toml
      core_schema_kinds.toml
  benchmarks/
  demos/
    pydantic_sifr_demo.sifr
    README.md
  docs/
  LICENSE
```

The backend is a normal statically linked Rust package dependency under the
existing Rust interop architecture. Published artifacts do not contain or load
a `cdylib`, Python extension, CPython library, or runtime plugin.

## High-Level Architecture

```text
Sifr type T + package metadata
              |
              v
      pydantic-sifr frontend
              |
              v
      Sifr Core Schema graph
              |
 package const canonicalize/verify
              |
              v
    immutable Schema Program
              |
              v
      pydantic_sifr_core
       /              \
      /                \
 decode plan        serialize plan
    |                    |
external input      structural view of T
    |                    |
validated arena     format writer
    |
structural Construct[T]
    |
typed Sifr value T
```

The schema graph is the architectural boundary corresponding to Pydantic's
`CoreSchema`. The representation and node set are Sifr-owned and deliberately
exclude Python-only behavior.

## Compiler Substrate

### Compiler prerequisites

Milestones `ps_1` through `ps_3` create new compiler and sysroot capabilities;
they are not small extensions of the current generics, stdlib value types, or
Rust bridge implementation. Their gated prerequisites are:

- compile-time specialization of package generics for a concrete `T`,
- deterministic compile-time evaluation sufficient to derive and emit static
  data,
- `ConstSpecializationOutcome[T]` and bounded package issues for const specialization in
  `check`, build, tests, and editor analysis,
- a package-neutral `JsonIntegerBoundaryDescriptor` verifier,
- first-class field required/defaulted metadata rather than reconstruction from
  an `__init__` signature,
- exact recursive nominal identity, and
- general stdlib value types that losslessly support microsecond temporal
  precision, timezone-aware `time`, and immutable `frozenset[T]`,
- a native-backed compiled `re.Pattern` that preserves source and flags after
  the opaque-resource substrate exists, and
- the structural Rust bridge call contract and certified ecosystem-owned
  opaque-resource support described below.

C-like enums remain simple constants. In accordance with the accepted Sifr
decision in `internal_docs/architecture.md`, data-carrying variants use ordinary
unions of records. Core Schema tagged unions specialize that existing type
model; they do not require associated-data enums or create a second permanent
sum representation.

### Structural Rust bridge calls

The existing bridge-compatible value table remains closed. The structural contract
does not make tuple, set, arbitrary mapping, union payload, or specialized
scalar values directly cross the boundary as ad hoc bridge types.

Instead, `sifr_runtime` owns three stable, language-general traits. Native
producers implement `StructuralSource`; the compiler generates
`StructuralConstruct` and `StructuralProject` implementations for concrete Sifr
types:

```text
StructuralSource
    shape_identity() -> ShapeIdentity
    root() -> NodeId
    take/read nodes through a sealed stable interface

StructuralConstruct
    construct[S: StructuralSource](source: own S) -> Result[Self, ContractError]

StructuralProject
    project(self: &Self, visitor: StructuralVisitor) -> Result[None, VisitorError]
```

The names above are conceptual; the accepted structural bridge design fixes the
actual Rust/Sifr surface. The essential contract is:

- a native backend may call a generic function bounded by these compiler-owned
  traits,
- the call is monomorphized in the generated package crate for the concrete
  Sifr `T`,
- the backend crate depends only on the stable traits and its own stable opaque
  resources,
- package-local generated glue implements the traits for generated Sifr types,
- the backend crate never imports `crate::__sifr_bridge` types,
- construction consumes a sealed `StructuralSource` carrying a declared
  structural-shape identity,
- projection borrows the current typed value and emits a call-scoped visitor
  event stream,
  and
- the existing bridge rejects all unsupported ordinary direct crossings as
  before.

`StructuralSource` is language-neutral. Pydantic-Sifr's validated arena
implements it, but an RPC decoder or database row mapper can implement the same
trait for its own native resource and use the same construction path. Core
Schema identity is checked by Pydantic-Sifr before construction; it is not part
of the compiler trait contract.

Decoding uses one native generic call:

```text
pydantic_sifr_core::validate_and_construct[T: StructuralConstruct](...)
    -> Result[T, ValidationError]
```

The core owns an opaque `ValidatedArena` implementing `StructuralSource`;
package-local generated glue consumes its stable nodes while constructing `T`.
Strings, bytes, exact integers, and specialized scalar components move out
where ownership permits. Containers are constructed recursively inside the
monomorphized package crate rather than crossing the public bridge wholesale.

Serialization also uses one native generic call. The core serializer pulls
from `T: StructuralProject` through the call-scoped view and remains the sole
driver of alias, exclusion, representation, and writer policy. This avoids
per-field Sifr/Rust bridge calls and avoids a second generic output tree.

The structural bridge contract must specify:

- trait and opaque-resource ownership,
- generated implementation placement,
- lifetime and call-scoped view rules,
- generic signature probing and monomorphization,
- move-out and partial-failure cleanup,
- recursion and callback interaction,
- panic containment,
- cache/build identity, and
- installed/source package certification.

The contract is incomplete until it is merged into
`internal_docs/rust_interop_architecture.md` and its core certification rows
pass. Pydantic-Sifr cannot privately invent an alternate structural bridge.

### Structural shape

Package code must be able to inspect a statically known `T` during
specialization:

- primitive kind,
- exact nominal identity,
- type arguments,
- record/class fields in declaration order,
- field names and declared types,
- required versus defaulted fields,
- enum variants and package-declared scalar value metadata,
- tuple and collection elements,
- optional and union members,
- refined/newtype base type,
- recursive references, and
- package-defined declaration metadata.

The shape is compile-time information. It is not a public runtime reflection
object and does not make arbitrary type mutation possible.

### Declarative metadata

Packages need one general mechanism for typed, compile-time declaration
metadata. It must support metadata attached to:

- a type,
- a field,
- an enum variant,
- a function,
- a method, and
- a parameter.

Metadata values must be statically typed and compile-time evaluable. The
compiler preserves and exposes them to specializing package code; it does not
interpret `Field`, validator, serializer, or model configuration semantics.

This mechanism is the substrate for Pydantic-familiar `Field`,
`field_validator`, `model_validator`, `field_serializer`, `computed_field`,
and configuration declarations without hard-coding their names.

### Const-specialization diagnostics

A specializing package function returns
`ConstSpecializationOutcome[T]`, containing either a value plus zero or more
warnings, or one or more fatal `ConstPackageIssue` values and no value. These
are new frontend contracts; they neither reuse nor alter
`sifr_package::PackageDiagnostic` or the driver's existing `CompileResult`.
Those existing types remain confined to package-manager and driver
diagnostics.

`ConstPackageIssue` carries a package-qualified stable `reason_code`, static
package-template arguments, one primary source span supplied by the
specialization API, additional labels, and notes. Values must be
const-evaluable and bounded. Package argument names are checked against the
package's statically declared template and cannot use compiler/LSP-reserved
names such as `rule`.
The package reason is diagnostic context, not a new top-level compiler code.
The frontend maps a fatal issue to built-in `SIFR-META-0001`, a warning to
`SIFR-META-0002`, and a malformed declaration to `SIFR-META-0003`. These three
general metaprogramming codes are added to the closed Sifr diagnostic registry
in `ps_1`, so normal documentation URLs remain
`https://docs.sifr.sh/errors/<CODE>`. A warning may accompany a produced value
and does not make checking fail; a fatal issue cannot accompany a value. The
compiler, never the package, owns severity and top-level rendering.

The compiler diagnostic arguments remain closed: `SIFR-META-0001` and
`SIFR-META-0002` declare exactly `package` and `reason_code`;
`SIFR-META-0003` declares `package`, `reason_code`, and `declaration_problem`.
Package template arguments are rendered only into a bounded structured note
after static template validation; they are never forwarded as open arguments
to a registry entry. `SIFR-META-0002` is intentionally an unsuppressible
`hard` LSP warning because it arose during deterministic specialization, not a
lint rule; this classification is documented with the code in `ps_1`.

The frontend converts the outcome into the same structured CLI/LSP diagnostic
stream in `check`, build, tests, and editor analysis. It validates the package
namespace, reason code, spans, and template arguments and never executes a
package renderer or accepts arbitrary terminal text. A non-Pydantic fixture
package must prove fatal and warning emission, invalid-issue rejection,
source/installed parity, and identical CLI/LSP identity before
Pydantic-Sifr may depend on the channel.

### Structural construction

Specialized package code must be able to construct `T` from validated
components without:

- bypassing ownership checks,
- invoking user-visible validation a second time,
- creating an observably partially initialized value,
- using reflection at runtime, or
- cloning every field.

Construction succeeds only from a sealed `StructuralSource` whose declared
structural-shape identity matches `T`. Compiler-generated code moves owned
values where possible and rejects source/type mismatches as internal contract
errors. It has no knowledge of Core Schema or Pydantic.

### Structural projection

Specialized package code must expose an immutable typed value to native
serialization as a structural reader:

- record field enumeration,
- enum variant access,
- primitive borrowing,
- collection iteration,
- optional/union discrimination, and
- declaration-order field visitation.

Projection is pulled by the native consumer during one monomorphized call and
does not first allocate a second generic tree. The facility is general
structural visitation; Pydantic-specific alias and serialization policy remains
in the schema program.

### Typed callbacks

Custom validators and serializers are ordinary typed Sifr functions. Generated
adapters must preserve:

- input and output types,
- ownership and borrowing,
- `Result` error types,
- declared ordering,
- callback identity in the schema program,
- one optional concrete context type and immutable/mutable borrow mode,
- panic containment at the Rust boundary, and
- non-send/send restrictions.

There is no universal untyped callback receiving an arbitrary runtime object.

Context-aware entry points are specialized over one concrete caller-owned
context type `C`. Callbacks receive `&C` or `&mut C` according to their declared
effect; mutation is visible to the caller and follows ordinary Sifr borrowing.
Callbacks in one specialized schema must agree on `C`, or use an explicit typed
aggregate context. Calls without context specialize on `NoContext`. The native
core carries only a call-scoped opaque handle plus type identity and forwards
it through generated typed adapters; it never interprets, stores, erases, or
constructs context values.

### Static schema emission

For derived and otherwise statically declared schemas, specialization produces
a schema graph during `check`, `build`, editor analysis, and any other
specializing frontend mode. Package-owned deterministic Sifr const code
canonicalizes and semantically verifies that graph exactly once and asks the
compiler's general static-data facility to materialize the resulting immutable
schema program. `check` and editor analysis retain its identity and
diagnostics; build-like modes additionally embed it in the generated artifact.
The program contains stable node arrays, string tables, references,
constraints, policies, and typed callback slots.

The same schema program must have the same identity across `check`, `build`,
`run`, tests, cache keys, and editor analysis.

Const canonicalization and verification are incremental frontend queries keyed
by package/core-schema version, compiler structural-contract version, concrete
type identity, declaration metadata/configuration, and callback identities.
Editing an unrelated declaration reuses verified programs; dependency changes
invalidate only affected schemas. Check and editor execution must remain
within the repository's accepted frontend median/p95 budgets.

The only native entry point accepts a sealed compiler-emitted
`VerifiedSchemaProgram[T]`; package code cannot construct or mutate one at
runtime. The core borrows it directly and checks only its
header/version/hash/shape-identity envelope. It does not repeat semantic schema
verification, parse a graph, compile a schema, construct validators, or
populate a process/per-call cache. Corrupt artifact envelopes return an
internal load error before user data is processed.

## Public Package Model

The public surface should be familiar to Pydantic users while respecting
Sifr's static and fallible semantics.

Representative shape:

```sifr
from pydantic import BaseModel, Field, ValidationError

class User(BaseModel):
    id: int = Field(gt=0)
    name: str = Field(min_length=1, max_length=100)
    active: bool = True

def parse_user(payload: bytes) -> Result[User, ValidationError]:
    return User.model_validate_json(payload)
```

Sifr does not turn validation failures into exceptions. Familiar operations
therefore return `Result` where user input or custom behavior can fail.

The canonical capabilities are:

- validate a Sifr structural input as `T`,
- validate JSON bytes/text as `T`,
- validate a bare `str` or statically verified strings-leaf structural input
  as `T`,
- serialize `T` to a structural value,
- serialize `T` to JSON,
- obtain a reusable `TypeAdapter[T]`,
- obtain JSON Schema for the selected serialization/validation mode, and
- customize validation/serialization through typed declarations and optional
  caller-owned typed contexts.

Pydantic-style methods and a smaller functional API may coexist only as thin
views over the same Core Schema and execution engine. There is no second
functional validator implementation underneath convenience functions.

## Core Schema Contract

### Role

Core Schema is a declarative, versioned internal contract between
`pydantic-sifr` and `pydantic_sifr_core`.

It describes:

- accepted input forms,
- strict/lax conversions,
- output value shape,
- constraints,
- defaults,
- aliases,
- extra-field policy,
- union selection,
- recursion,
- validation callback positions,
- serialization behavior,
- description behavior, and
- stable error codes.

### Node families

The complete node algebra must cover:

| Family | Required nodes |
| --- | --- |
| Scalars | none, bool, exact integer, fixed integer, float, decimal, string, bytes |
| Specialized scalars | date, time, datetime, duration, UUID, URL, multi-host URL/DSN, pattern, exact rational `Fraction`, `Complex`, and package-provided scalar adapters |
| Constraints | numeric bounds/multiples, decimal total/fractional digit bounds, length bounds, pattern, finite and clock-relative temporal bounds |
| Products | record/model, tuple, typed mapping |
| Collections | list, set, frozen set, typed sequence policies, and lazy `ValidatedIterator[T]` |
| Sums | optional, literal, enum, smart/left-to-right ordinary union with explicit auto-collapse policy, field/path-discriminated tagged union and typed-callback-discriminated tagged union |
| Control | default, nullable, definitions, reference, recursion guard, strict/lax branch, JSON/structural-input branch, embedded-JSON child decoder, typed sequential chain and compositional error override |
| Transforms | before, after, wrap and plain typed validators; built-in string normalization |
| Serialization | alias, inclusion/exclusion, computed field, typed serializer and representation override |

### Pin-derived node-kind disposition

The following table is total over the 53 literals in the sole oracle's
`CoreSchemaType` at
`pydantic-core/python/pydantic_core/core_schema.py:4247-4301`. The external
repository generates `tests/provenance/core_schema_kinds.toml` directly from
that literal and fails exact-set equality unless every kind has exactly one
row, compatibility class, normal form, and primary implementation owner, plus
a non-empty set of evidence families or the `ps_0` disposition audit.
Several evidence families may support one owner; they do not create a second
implementation owner. The same audit covers all four `CoreSchemaFieldType`
literals. A pin update regenerates the universe before any hand-authored
disposition changes, making a newly added kind a blocking review item rather
than a silent omission.

| Pydantic Core kind | Class | Sifr normal form or explicit disposition | Owner/evidence |
| --- | --- | --- | --- |
| `invalid` | `rejected` | Static schema construction fails; no executable invalid node exists | `ps_4` / `core/schema_contract` |
| `any` | `not-applicable` | No untyped runtime node exists; harness occurrences normalize to the smallest concrete child schema | `ps_0` disposition audit |
| `none` | `same` | none scalar | `ps_5` / `validators/none` |
| `bool` | `same` | bool scalar | `ps_5` / `validators/numeric` |
| `int` | `adapted` | exact/fixed integer plus locked Sifr JSON profile | `ps_5` / `validators/numeric`, `core/fixed_integer` |
| `float` | `same` | float scalar | `ps_5` / `validators/numeric` |
| `decimal` | `adapted` | finite exact `bigdecimal` scalar | `ps_5` / `validators/numeric`, `core/decimal_digit_counting` |
| `fraction` | `adapted` | package-owned exact `Fraction` scalar | `ps_5` / `core/fraction` |
| `str` | `adapted` | native string node and Rust-regex policy | `ps_5` / `validators/text_bytes`, `core/string_pipeline_order` |
| `bytes` | `same` | bytes scalar | `ps_5` / `validators/text_bytes` |
| `date` | `same` | date scalar | `ps_5` / `validators/temporal` |
| `time` | `adapted` | lossless Sifr time with declared offset policy | `ps_5` / `validators/temporal` |
| `datetime` | `adapted` | lossless Sifr datetime with declared offset policy | `ps_5` / `validators/temporal` |
| `timedelta` | `adapted` | Sifr duration | `ps_5` / `validators/temporal` |
| `literal` | `same` | literal sum node | `ps_7` / `validators/literal` |
| `missing-sentinel` | `not-applicable` | Missing-input state normalizes to defaults/`Option`; no identity value enters `T` | `ps_0` disposition audit |
| `enum` | `adapted` | enum variant tag plus declared scalar-value metadata | `ps_7` / `validators/enum` |
| `is-instance` | `not-applicable` | No runtime Python class identity or arbitrary object input | `ps_0` disposition audit |
| `is-subclass` | `not-applicable` | No runtime subclass reflection | `ps_0` disposition audit |
| `callable` | `not-applicable` | Sifr functions are statically typed callbacks, not user-data values | `ps_0` disposition audit |
| `list` | `same` | list collection | `ps_5` / `validators/collections` |
| `tuple` | `same` | tuple product | `ps_5` / `validators/collections` |
| `set` | `same` | set collection | `ps_5` / `validators/collections` |
| `frozenset` | `adapted` | immutable Sifr `frozenset[T]` | `ps_5` / `validators/collections` |
| `generator` | `adapted` | `ValidatedIterator[T]`; `next()` returns `Result[Option[T], ValidationError]` and never hides deferred failure | `ps_5` / `validators/generator` |
| `dict` | `same` | typed mapping | `ps_5` / `validators/collections` |
| `function-after` | `adapted` | typed after callback | `ps_7` / `validators/callbacks_context` |
| `function-before` | `adapted` | typed before callback | `ps_7` / `validators/callbacks_context` |
| `function-wrap` | `adapted` | typed wrap callback | `ps_7` / `validators/callbacks_context` |
| `function-plain` | `adapted` | typed plain callback | `ps_7` / `validators/callbacks_context` |
| `default` | `adapted` | statically typed const or validated factory default | `ps_6` / `validators/defaults` |
| `nullable` | `same` | nullable/optional control | `ps_6` / `validators/nullable` |
| `union` | `adapted` | smart or left-to-right ordinary union | `ps_7` / `validators/unions`, `core/smart_union_ranking` |
| `tagged-union` | `adapted` | field/path or typed-callback discriminator | `ps_7` / `validators/tagged_unions` |
| `chain` | `adapted` | flattened typed sequential chain | `ps_7` / `validators/control_composition` |
| `lax-or-strict` | `same` | strict/lax branch selected by static default or call override | `ps_7` / `validators/control_composition` |
| `json-or-python` | `adapted` | JSON/structural-input branch; no Python branch | `ps_7` / `validators/control_composition` |
| `typed-dict` | `adapted` | fixed-layout Sifr record | `ps_6` / `validators/typed_dict` |
| `model-fields` | `adapted` | record/model field plan | `ps_6` / `validators/model_fields` |
| `model` | `adapted` | ordinary Sifr class plus structural construction | `ps_6` / `core/json_models`, `api/base_model` |
| `dataclass-args` | `adapted` | record input/field plan; no separate constructor-argument object | `ps_6` / `validators/dataclasses` |
| `dataclass` | `adapted` | ordinary Sifr class; normalizes to the same model/record node | `ps_6` / `validators/dataclasses` |
| `arguments` | `not-applicable` | Python call-signature validation is static Sifr call checking | `ps_0` disposition audit |
| `arguments-v3` | `not-applicable` | Python call-signature validation is static Sifr call checking | `ps_0` disposition audit |
| `call` | `not-applicable` | `validate_call` is replaced by ordinary typed Sifr calls | `ps_0` disposition audit |
| `custom-error` | `adapted` | compositional `ErrorOverride` with static package code/message | `ps_4` / `core/schema_contract` |
| `json` | `adapted` | embedded-JSON decoder wrapping a typed child schema | `ps_5` / `validators/embedded_json` |
| `url` | `adapted` | package-owned URL scalar | `ps_5` / `validators/url` |
| `multi-host-url` | `adapted` | package-owned ordered multi-host URL/DSN scalar | `ps_10` / `core/multi_host_url_serialization`, `api/networks` |
| `definitions` | `same` | definitions table | `ps_7` / `validators/definitions_recursion` |
| `definition-ref` | `same` | typed definition reference | `ps_7` / `validators/definitions_recursion` |
| `uuid` | `adapted` | package-owned UUID scalar | `ps_5` / `validators/uuid` |
| `complex` | `adapted` | package-owned `Complex` scalar | `ps_5` / `validators/complex` |

The four field kinds normalize as follows:

| Pydantic Core field kind | Class | Sifr normal form | Owner/evidence |
| --- | --- | --- | --- |
| `model-field` | `adapted` | declared model field metadata | `ps_6` / `validators/model_fields` |
| `dataclass-field` | `adapted` | the same declared record field metadata | `ps_6` / `validators/dataclasses` |
| `typed-dict-field` | `adapted` | the same declared record field metadata | `ps_6` / `validators/typed_dict` |
| `computed-field` | `adapted` | serialization-only computed field | `ps_8` / `api/serialization` |

Nodes are orthogonal and compositional. For example, constrained integers are
an integer node plus constraints rather than independent positive-int,
negative-int, bounded-int, and strict-int validator implementations.
Set uniqueness follows from the set collection node rather than a second
constraint. User refinements compose a base node with existing constraints or
a typed validator; there is no redundant catch-all "typed refinement" node.

The pinned `lax-or-strict`, `json-or-python`, and `chain` kinds become general
control nodes instead of Python special cases. Sifr names the second
`json-or-structural`: JSON entry points select its JSON branch and native
structural entry points select its structural branch. Canonicalization flattens
nested typed chains and rejects an empty chain; it may erase a one-step chain.
Strict/lax selection obeys the adapter/model default plus the explicit per-call
override.

The package supplies ordinary typed `Fraction` and `Complex` values backed by
focused Rust numeric crates; they are not compiler primitives. Their scalar
nodes own strict/lax parsing, exact rational constraints, finite/non-finite
complex policy, and serialization through the same schema program.

Two pinned experimental capabilities deliberately do not survive as public
Sifr validation behavior. `missing-sentinel` is a real Core Schema node but is
`not-applicable`: missing input is validation state before construction and
public Sifr values use defaults or `Option[T]`, so no singleton identity leaks
into a model. `allow_partial` is not a Core Schema node; it is a validation-call
mode and is `rejected` for model/adapter validation because silently discarding
an invalid tail cannot produce the promised complete `T`. Incremental framing
is a separate streaming-input concern and never a fallback validation mode.

The enum node maps declared exact-integer or string input literals to ordinary
payload-free Sifr enum variant tags. The mapping is package declaration
metadata, not a Rust/Sifr enum discriminant, so string-valued and
arbitrary-precision integer-valued Pydantic enums do not require associated
data or an expanded compiler enum representation. Serialization consults the
same mapping; construction and projection carry only the Sifr variant tag.

The string node has built-in `strip_whitespace`, `to_upper`, `to_lower`,
`ascii_only`, and `coerce_numbers_to_str` policies rather than synthesizing
user callbacks. Its fixed pipeline, matching the pinned `string.rs` validator,
is input conversion, whitespace stripping, ASCII restriction, Unicode-scalar
length checks, pattern check, then case conversion
(`pydantic-core/src/validators/string.rs:110-178`). Byte lengths count bytes.
A Sifr-native discriminating fixture in `core/string_pipeline_order` contains
pairwise-conflicting inputs that prove strip before ASCII, ASCII before length
and pattern, length before pattern, and all checks before case conversion; it
also proves Unicode-scalar versus byte length. Rust `regex` syntax and flags
are the sole regular-expression engine. Explicit Python-regex mode is
`not-applicable`; a portable precompiled Python pattern is `adapted` by
compile-time translation of supported source and flags to native `re.Pattern`,
while Python-only syntax or flags are rejected at pattern construction.

Requesting both `to_upper` and `to_lower` is rejected as an intentional
stricter Sifr schema rule. Decimal constraints retain both raw and normalized
digit counts. `decimal_max_digits` and `decimal_max_places` are emitted only
when both the raw and normalized forms exceed their respective limit.
`decimal_whole_digits` applies only when both `max_digits` and
`decimal_places` are present: the allowed whole digits are
`max_digits.saturating_sub(decimal_places)`, each observed whole count is
`digits.saturating_sub(decimal_digits)`, and the error is emitted only when
both raw and normalized observed counts exceed the allowance. The native
`core/decimal_digit_counting` family fixes zero, trailing-zero, fractional,
and saturating-subtraction edge cases; it asserts all three exact error codes.
Constraint emission is first-match-wins in `max_digits`,
`decimal_places`, then `decimal_whole_digits` order, and each error context
carries the configured allowance rather than the observed count
(`pydantic-core/src/validators/decimal.rs:152-197`).

Clock-relative date/time constraints carry `past` or `future` plus the declared
UTC-offset policy. Validation captures one call-scoped UTC instant before
executing the plan. Production uses the package's `SystemClock`; tests and
differential fixtures inject a fixed typed clock through the same internal
capability. Every comparison in one validation call uses that snapshot, so the
result is deterministic for `(schema, input, clock)`.

### Program invariants

Core Schema verification rejects:

- dangling references,
- duplicate definition identities,
- impossible type/output relationships,
- callback signature mismatches,
- invalid constraint combinations,
- a missing or unsafe Sifr integer JSON boundary descriptor,
- serialization nodes incompatible with validation output,
- unbounded recursive entry,
- ambiguous discriminator maps or invalid typed discriminator callbacks,
- an error override whose custom code lacks a static message, whose code
  collides with a built-in code while changing that built-in message, or whose
  message/context is not statically serializable,
- defaults that do not validate under their declared policy, and
- unknown schema versions or node kinds.

An unvalidated default expression or factory must produce the declared field
output type. A validated default may instead produce any statically known
input type accepted by the field schema. A const-evaluable validated default
is executed during schema verification and a failure is a package/compiler
diagnostic. A non-const factory result is run through the same validator plan
at use time and either constructs the declared output type or returns its
ordinary validation errors. A configuration cannot defer this typing choice
to runtime.

Verification failures are package/compiler diagnostics. All public adapters
are specialized for a statically known `T`; no runtime schema builder or
alternate validator path exists.

### Versioning

The schema program begins with:

- schema-program format version,
- compiler structural-contract version,
- Rust bridge structural-call version,
- callback ABI version,
- feature bitmap, and
- payload identity hash.

`pydantic-sifr` and `pydantic_sifr_core` release together and require an exact
supported contract tuple. Core Schema is an internal build artifact, not a
cross-release wire protocol. A contract change increments the relevant version
and rebuilds dependents; the core does not carry backward interpreters.
Unknown or mismatched contracts fail during build before user data is
processed.

## Native Core

### Responsibilities

`pydantic_sifr_core` owns:

- runtime validation and serializer program execution without recompilation,
- verified-program envelope and structural-shape identity checks,
- JSON and structural input adapters,
- strict/lax scalar conversion,
- constraint execution,
- record and collection validation,
- aliases and extra-field handling,
- union ranking and discriminator dispatch,
- recursion guards,
- default handling,
- callback scheduling,
- aggregate errors,
- validated value storage,
- serializer-plan execution,
- JSON writing, and
- source positions needed by diagnostics.

The Sifr package owns the one semantic Core Schema canonicalizer and verifier,
implemented as deterministic const-evaluable Sifr code. The native core never
implements a second verifier; its envelope checks protect artifact integrity,
not schema semantics.

It does not own Sifr syntax, class lookup, package imports, compiler type
resolution, or user-facing declaration analysis.

### Input abstraction

The core uses one input abstraction over supported data sources. Input adapters
provide:

- kind inspection,
- exact primitive access,
- sequence iteration,
- mapping lookup and ordered iteration,
- source location,
- strictness-relevant origin, and
- safe replay required by unions.

Validation selects one of three input profiles over that abstraction:

- `native` preserves the source's native primitive kinds,
- `json` applies JSON-origin conversion and strictness rules, and
- `strings` accepts either a bare `str` root or a structural value whose scalar
  leaves are strings and applies the documented Pydantic-style strings-input
  conversions.

The strings-profile public entry point is generic over an input `S`.
Compile-time shape verification accepts `S` when it is `str`, or when every
terminal scalar in its structural projection, including mapping keys, is
`str`; nested records, mappings, and sequences are allowed. A bare `str` uses
the normal scalar projection. The compiler-generated projection for `S` is
therefore the input type—there is no `Any` or package-owned recursive value
tree. The profile reuses the native structural adapter with a leaf-kind
restriction and different conversion rules; it is not a third schema compiler,
value representation, or validation engine.

JSON validation uses a `jiter::JsonValue` document. The value-tree form is the
canonical path because aggregate errors, aliases, recursive records, and union
candidate evaluation require replay and random access. A second streaming
semantic engine is not maintained. The JSON document and normalized validated
arena are two intentional representations with different jobs; the rejected
representation is an additional copied bridge-object tree between the arena
and `T`.

Native Sifr structural inputs use a compiler-generated structural projection
rather than converting through JSON.

### Validated value arena

Python provides one universal object representation; Sifr does not. Successful
decode execution therefore produces a per-call arena containing normalized
validated values:

```text
ValidatedValue =
    None
  | Bool
  | ExactInt
  | FixedInt
  | Float
  | Decimal
  | String
  | Bytes
  | Sequence(range)
  | Mapping(range)
  | Record(range)
  | Variant(tag, child)
  | SpecializedScalar(kind, payload)
```

The arena:

- has one root value,
- uses compact indices rather than recursive bridge allocations,
- owns converted strings/bytes/numbers exactly once,
- supports move-out during `Construct[T]`,
- records schema identity,
- is invalidated after successful consuming construction, and
- has bounded recursion and collection limits.

The Sifr bridge exposes the arena as a sealed opaque resource. Package code
cannot forge nodes or reinterpret one schema's output as another type.

`SpecializedScalar` payloads are crate-neutral normalized components, never
public `speedate`, `chrono`, `uuid`, `url`, `rust_decimal`, or `bigdecimal`
crate values. Examples include calendar/time components plus offset and
microsecond precision, UUID bytes, normalized URL text/components,
compiled-pattern source and flags, and exact decimal coefficient and scale.
`StructuralConstruct` reconstructs the canonical Sifr stdlib type:

- `datetime`, `date`, `time`, and duration use the extended stdlib value types
  that preserve the validated microsecond and timezone-offset components,
- UUID and URL use the existing stdlib-backed types, and
- compiled string patterns use the native-backed `re.Pattern`, and
- the Core Schema `decimal` node uses Sifr `bigdecimal`, backed by
  `bigdecimal::BigDecimal`, because Pydantic decimal precision is unbounded;
  fixed-precision Sifr `decimal` is a separate package-provided scalar adapter
  with its own range and precision contract.

`jiter`, `speedate`, and focused crates parse or normalize these components;
they do not define the Sifr-facing type or schema contract.

### Validation state

One validation state carries:

- strict/lax mode,
- current error path,
- recursion stack,
- exactness class,
- internal successfully-validated-field count,
- partial error accumulator,
- input source kind and profile,
- optional call-scoped typed context handle, type identity, and borrow mode,
- one call-scoped UTC clock snapshot for relative temporal constraints,
- resource limits, and
- callback state.

Tagged and ordinary unions are separate schema nodes and algorithms. An
ordinary union declares `mode = smart | left_to_right` and defaults to
`smart`. A one-choice union collapses to that choice by default; disabling
auto-collapse preserves the union branch label and aggregate-error boundary.

A tagged union either reads a declared field/path or invokes one statically
typed discriminator callback exactly once. The result is a declared tag used
to select a branch from the same indexed discriminator map or return a
discriminator error. Map lookup remains indexed; the callback form adds only
the typed tag computation. A smart ordinary union follows pinned
`pydantic-core/src/validators/union.rs:117-191`, with exactness order from
`pydantic-core/src/validators/validation_state.rs:15-19`:

1. an exact successful candidate with no field-count data short-circuits;
2. otherwise every candidate is evaluated;
3. field counts decide only when both candidates carry a count and those
   counts differ, with the larger count winning;
4. otherwise exactness ranks `Lax < Strict < Exact`;
5. a remaining tie selects the earliest declared candidate;
6. the selected candidate bubbles its exactness floor and adds its successful
   field count to enclosing validation state, so nested record/model counts
   participate additively in an outer union; and
7. an internal `Omit` seen before any successful candidate is remembered, but
   ignored after a best match exists; if no candidate succeeds and any
   candidate omitted, the union omits, while other non-line internal errors
   propagate.

The smart algorithm is a declaration-order left fold against the current best,
not a sort or an order-independent ranking key: its mixed
counted/uncounted comparison is intentionally non-transitive. When all
candidates fail with ordinary line errors, the aggregate retains declaration
order. Each candidate uses its declared choice label when present and otherwise
falls back to the validator/schema name. In `left_to_right` mode
(`pydantic-core/src/validators/union.rs:194-212`) the first result other than
ordinary line errors wins
immediately; total line-error failure uses the same ordered, labelled
aggregate.

The internal field count is ephemeral validation state used only for ranking.
It is not a public `__pydantic_fields_set__` attribute and is not retained on
the constructed Sifr model.

The Sifr-native `core/smart_union_ranking` family discriminates mixed
counted/uncounted candidates, exactness ordering, declaration-order ties,
additive nested bubbling, `Omit`, choice labels, both modes, and auto-collapse.

Any intentional difference from the pinned Pydantic behavior is recorded in
the compatibility manifest.

### Serialization

The serializer plan drives one monomorphized native call over
`T: StructuralProject`. It pulls from the compiler-generated call-scoped view;
it does not read private generated-Rust fields by layout assumption, issue
per-field Sifr/Rust calls, or rely on a Sifr equivalent of Python `__dict__`.

The plan owns:

- aliases,
- validation versus serialization representation,
- inclusion/exclusion,
- default/none policies,
- computed fields,
- tagged-union representation,
- custom serializers,
- typed caller-owned serialization-context forwarding,
- exact integer output policy, and
- target-format constraints.

Include and exclude arguments use one package-owned recursive value language:

```text
Selection =
    All
  | Fields[ordered map[field name, Selection]]
  | Elements {
        default: Option[Selection],
        indices: ordered map[signed index, Selection]
    }
  | Entries {
        default: Option[Selection],
        keys: ordered map[declared key type, Selection]
    }
```

`All` selects or removes the current node; `Fields` recurses by field name.
For `Elements`, `default` applies to every element and a matching `indices`
entry overlays it for that element. Signed indices are normalized against the
node's pre-filter sequence length before lookup; when two declared indices
normalize to the same index, the later declaration wins. `Entries` applies the
same default/override model to typed
mappings, matching entries by validated key value without positional
normalization. A record-wide default desugars to its statically enumerated
`Fields`.

Overlay is recursive and deterministic: missing entries inherit the base;
branch maps merge by key; an explicit branch replaces a base `All`; and an
explicit `All` replaces a base branch. A `default` present on both sides
overlays recursively; a `default` present on only one side is inherited. This
makes an index-specific nested selection refine or replace the default while
unrelated nested keys remain combined.

Element-default overlay and schema/call composition are distinct operations.
Following pinned
`pydantic-core/src/serializers/filter.rs:150-257`, sequence and mapping
filters are resolved per node against
every original mapping key or pre-filter sequence index, in this precedence
order:

1. a call-time exclusion that terminally selects the entry removes it;
2. otherwise, when a call-time inclusion exists, the entry is emitted when
   that inclusion selects it, forwarding any nested selection it carries
   together with a nested call-time exclusion carried from clause 1, and is
   otherwise removed unless the schema-declared inclusion selects it, in which
   case clauses 3 and 4 decide;
3. otherwise, a call-time exclusion that selects the entry only with a nested
   selection emits it and forwards that nested selection; and
4. otherwise the schema-declared filter decides: the entry is emitted when its
   inclusion is absent or selects the entry and its exclusion does not.

At the same sequence/mapping node, schema and call-time inclusions combine by
union, while call-time inclusion can re-include an item removed only by a
schema exclusion. Schema-filter inclusion and exclusion combine as
intersection. The Sifr-native `core/selection_precedence` family discriminates
schema-include plus call-include union, schema include/exclude intersection,
call-include over schema-exclude, negative and positive-out-of-range index
normalization, and empty nested selections. For records, a
statically declared field-level serialization exclusion is unconditional: the
field is removed before call-time selection and cannot be re-included.
Remaining record fields use the call-time parts of the same rules with no
schema filter. Signed indices normalize against the pre-filter sequence
length. Every signed index is normalized by Euclidean modulo for a non-empty
sequence, including positive out-of-range indices; no index matches an empty
sequence
(`pydantic-core/src/serializers/filter.rs:20-56,102-103,282-283`).
`Elements` applies only to statically sized-at-serialization
collections such as lists and tuples. An index selection on
`ValidatedIterator[T]` is rejected because negative/modulo normalization would
require consuming an unsized iterator; iterator filtering uses an explicitly
streaming predicate callback instead. A nested selection beneath a scalar
leaf, including a structurally
incompatible `Fields`, `Elements`, or `Entries` value, is accepted and ignored
rather than rejected; shape checking applies only where the declared type has
fields, elements, or entries. An empty nested exclusion emits the composite
value and excludes no children. An empty nested inclusion on a composite
selects no children and therefore empties that subtree unless a
schema-declared inclusion independently selects a child under clause 2; it is
inert below a scalar leaf.

This replaces Python's overlapping
set/dict/list/dict-view/`True`/ellipsis/`__all__` spellings with one typed
representation while preserving portable default, signed-index, override, and
composition behavior. A Python `None`-valued entry desugars to `All` under an
inclusion and to an empty nested selection under an exclusion. Python
custom-membership and duck-typed `__contains__` precedence are
`not-applicable`; only their explicitly selected key/index results may be
represented as an adapted typed `Selection`.

The pinned filter first normalizes Python `dict`/`set` spellings and has a
separate unsized-iterable path that rejects negative indices. Those are oracle
harness mechanics, not public Sifr forms: typed `Selection` is the sole
representation, and `Elements` is rejected altogether for
`ValidatedIterator[T]` as specified above.

Serializer input/schema type mismatches are statically impossible through
`T: StructuralProject`. Pydantic Core's runtime warning-and-passthrough cases
are `not-applicable`; they are never a fallback behavior. A custom serializer
with an incompatible declared input or output type is a compile-time schema
diagnostic.

JSON output is streamed to a writer. It does not allocate a complete
`serde_json::Value` first. `serde_json` mechanisms may be reused for escaping
and scalar formatting, while Sifr's schema program remains the semantic
authority.

### Integer JSON profiles

Every model or adapter that can serialize an integer selects exactly one
accepted Sifr profile in its static configuration: `json.exact`, `json.web`, or
`json.string_ints`. Nested fields inherit the containing profile unless a field
declares a supported override. The schema program stores that selection and
the native core routes integer reading/writing through
`sifr_runtime::json`; it never reimplements or weakens those helpers.

The package's deterministic const schema emits a general compiler-owned
`JsonIntegerBoundaryDescriptor` containing the selected profile, declared
integer kind, static range if bounded, and source path. The compiler's
package-neutral boundary verifier checks that descriptor before sealing the
schema program. Missing or unsafe information activates the reserved built-in
diagnostic `SIFR-INT-0009`; `ps_1` owns its registry entry, documentation, and
CLI/LSP tests. Pydantic-Sifr supplies data to this general verifier but neither
owns nor emits the top-level code.

- `json.exact` emits canonical base-10 JSON numbers without precision loss.
- `json.web` emits numbers only within JavaScript's safe range; `int64`,
  `uint64`, and unbounded `int` default to decimal strings unless a static safe
  range authorizes numeric output. A violating runtime value returns
  `JsonIntegerRangeError` with the model path.
- `json.string_ints` emits every integer as a canonical decimal string.

JSON Schema generation consumes the same profile. Under `json.web`,
`int8/16/32` and `uint8/16/32` are JSON integers with their exact
`minimum`/`maximum`; `int64`, `uint64`, and unbounded `int` are decimal strings
unless a statically proven JavaScript-safe range authorizes the same bounded
integer form. Under `json.string_ints`, every integer is a decimal string.
String representations use the locked decimal-string pattern and
`x-sifr-format`. Under `json.exact`, numbers use `type: integer`,
`x-sifr-integer-profile: exact`, exact bounds where available, and a client
warning unless the declared schema target supports exact integer parsing.
Browser-facing schema must never claim an unbounded numeric integer.

An absent or insufficient profile fails at compile time with `SIFR-INT-0009`,
including path, boundary, selected-or-missing profile, static range when
known, and suggested policy.
Pydantic oracle expectations that assume an unbounded JSON integer are
`adapted` to this language-wide contract rather than treated as a competing
package policy.

## Error Contract

All user-data failures return one `ValidationError` containing an ordered list
of `ErrorDetail` values.

Each detail contains:

- stable machine-readable code,
- ordered location segments,
- human-readable message,
- expected contract summary,
- optional safe input summary, controlled by the error-disclosure policy,
- optional context,
- optional JSON byte/line/column position, and
- originating schema node identity for diagnostics and testing.

An `ErrorOverride` wraps any validation subgraph and replaces that subgraph's
failure aggregate with one error at the wrapper location. It may reference a
built-in Sifr error code and its canonical message, or declare a package-owned
custom code plus a required static message and typed static context. Built-in
codes and meanings remain Sifr-owned; custom codes occupy a distinct
package-qualified namespace and cannot redefine a built-in code.

The public `ErrorDisclosure` policy has `IncludeSafeInput` and `OmitInput`
modes, selected by static model/adapter configuration with a per-call
override. `IncludeSafeInput` uses bounded, redacted summaries that never invoke
user formatting code; `OmitInput` removes the field from every detail. The
choice does not change validation, ordering, codes, or locations.

Locations support:

- field names,
- aliases,
- sequence indices,
- mapping keys,
- union branches,
- validator stages, and
- root/model positions.

Syntax errors, validation errors, serialization errors, callback errors,
resource-limit errors, and contained Rust panics remain distinct typed errors.
Static schema verification failures are compiler/package diagnostics. Raw Rust
errors, PyO3 errors, and `serde_json::Error` values do not leak into the public
Sifr API.

Error collection is bounded by an explicit policy to prevent adversarial inputs
from allocating unbounded error lists. Reaching the limit produces a stable
truncation fact rather than panicking or silently claiming complete coverage.
Sifr's locked JSON boundary enforces the integer-digit budget before allocating
an unbounded value and returns the existing
`JsonLimitError { message: str, limit: int }` from `sifr_runtime::json`.
Separately, `pydantic_sifr_core` owns explicit input-byte, nesting,
collection-size, string-size, recursion, and accumulated-error limits and
returns package-owned `ResourceLimitError { kind, limit, location }`. These are
distinct authorities: the package reuses the language integer parser/error and
does not pretend the other package limits are locked Sifr runtime behavior.

## Reuse Policy

### Production dependencies

| Component | Decision | Boundary |
| --- | --- | --- |
| `jiter` | Reuse directly | JSON parsing, exact/lossless numbers and locations; Python feature disabled |
| `speedate` | Reuse directly | Temporal parsing into crate-neutral components reconstructed as canonical Sifr stdlib types |
| `serde` | Reuse selectively | Format interoperability and writer mechanisms, never schema authority |
| `serde_json` | Reuse selectively | JSON escaping/formatting or adapters, never canonical validation semantics |
| `regex` | Reuse directly | Pattern compilation/matching with bounded policy |
| `url` and IDNA crates | Reuse directly | URL/IDNA mechanism behind Sifr types and errors |
| `uuid` | Reuse directly | UUID parsing/formatting behind Sifr policy |
| `base64` | Reuse directly | Binary-text codecs behind schema policy |
| `num-bigint` | Reuse directly | Exact integer mechanism compatible with Sifr's integer model |
| `num-rational` | Reuse directly | Canonical exact rational mechanism behind package-owned `Fraction` |
| `num-complex` | Reuse directly | Complex-number representation and arithmetic behind package-owned `Complex` |

Dependency features must be minimal. Python, extension-module, dynamic loading,
and unused default features are disabled.

### Selective algorithm ports

Small algorithms may be ported or behaviorally reimplemented when their
semantics are selected:

- boolean and numeric conversion tables,
- integer-string normalization,
- exactness scoring,
- internal successfully-validated-field union scoring,
- alias-path lookup,
- recursion detection,
- constraint ordering,
- error-location construction, and
- serializer include/exclude decisions.

Whole Pydantic Core validator or serializer modules are not copied. A port must
be small enough to state its Sifr contract independently and must carry source
revision and license provenance.

### Rejected dependencies and approaches

| Approach | Decision | Reason |
| --- | --- | --- |
| Embed CPython/Pydantic Core | Reject | Violates native deployment and imports Python identity, GIL and packaging |
| Link `pydantic-core` as a Rust library | Reject | Its central interfaces and outputs are Python-shaped |
| Fork and remove PyO3 | Reject | Near-total rewrite plus permanent upstream drift |
| Serde derive as validation engine | Reject | Fail-fast format decoding cannot express the complete aggregate/coercive contract |
| Schemars as schema authority | Reject | Creates a second schema model and unstable output ownership |
| Garde/`validator` as core | Reject | Post-construction Rust validation duplicates schema and cannot own decoding |
| Per-model compiler validation lowering | Reject | Makes package behavior a compiler special case |
| Copied arena-to-model bridge tree | Reject | JSON already has a parse tree and normalized arena; a third recursive bridge-object tree adds no semantic value |
| Parallel streaming and tree validators | Reject | Duplicates semantics and substantially increases maintenance |

## Pydantic Compatibility and Test Reuse

### Compatibility classes

Every relevant upstream behavior is classified as:

- `same`: Sifr intentionally matches normalized Pydantic behavior,
- `adapted`: equivalent capability with a documented Sifr-safe difference,
- `not-applicable`: behavior depends on Python-only semantics, or
- `rejected`: behavior conflicts with Sifr's guarantees.

No test is silently omitted because it is inconvenient.

### Portable test categories

Port extensively:

- JSON primitive conversions,
- strict/lax matrices,
- exact and boundary numbers,
- string and byte constraints,
- date/time/datetime/duration cases,
- required, defaulted and nullable fields,
- aliases and alias paths,
- extra-field policies,
- lists, tuples, mappings and sets,
- literals and enums,
- tagged and untagged unions,
- nested and recursive models,
- aggregate locations and messages,
- custom validator ordering,
- serialization aliases and exclusion policies,
- JSON Schema examples,
- malformed/adversarial JSON,
- fuzz seeds, and
- portable benchmarks.

Fixed-width integer schemas have no Python/Pydantic oracle because Python
integers are arbitrary precision. The Sifr-native `core/fixed_integer` contract
covers `int8/16/32/64` and `uint8/16/32/64`: strict mode
accepts only integer-kind inputs; lax mode additionally accepts exactly the
lossless bool, canonical integer-string, and finite integral-float conversions
of the exact-integer node. Conversion first produces a mathematical exact
integer and then performs one target-bound check. Underflow or overflow returns
stable `integer_overflow` detail containing the target type and inclusive
bounds; it never wraps, truncates, saturates, or routes through float. Native
serialization preserves the fixed type; JSON representation follows the
selected Sifr integer profile.

Before the public-model facade exists, `core/pattern_value` is a Sifr-native
package contract for the Core Schema compiled-pattern node: construct a
native-backed `re.Pattern` from source and flags, preserve both components,
match without recompiling per call, return a stable invalid-pattern error, and
serialize the source form. The later `api/pattern` family differentially checks
the public field and JSON Schema behavior against Pydantic.

Five additional native contracts close places where the upstream fixture
does not by itself discriminate Sifr's complete rule:

- `core/string_pipeline_order` proves Unicode-scalar versus byte length and
  every stated normalization/check boundary;
- `core/decimal_digit_counting` proves raw/normalized, zero, trailing-zero,
  fractional, and saturating whole-digit cases;
- `core/fraction` proves normalized numerator/positive-denominator identity,
  exact integer/decimal/rational parsing, zero-denominator rejection,
  strict/lax and JSON/strings profiles, constraints, and canonical
  serialization before the public adapter exists;
- `core/smart_union_ranking` proves counted/uncounted comparison, exactness,
  stable ties and labels, nested additive bubbling, `Omit`, both union modes,
  and optional auto-collapse; and
- `core/selection_precedence` proves schema/call inclusion and exclusion,
  index normalization, and empty nested selection semantics.

Owned Sifr structural inputs cannot contain Python-style identity cycles.
Recursive-schema success behavior is grounded in portable upstream acyclic
cases, while recursion/resource guards use the Sifr-native
`core/recursion_limit` contract: generated acyclic inputs beyond the configured
depth must return a stable `recursion_limit` error without panic or
exponential work. Cyclic-object identity tests are `not-applicable`.

Do not port as Sifr behavior:

- Python object identity,
- Python subclass and duck-typing behavior,
- metaclass mutation,
- descriptors,
- `__dict__` and `__pydantic_fields_set__`,
- pickle,
- CPython garbage collection/reference counts,
- arbitrary `from_attributes` object access,
- Python exception wrapping, or
- extension-module import behavior.

### Pinned conformance manifest

The following tables are the normative minimum implementation scope at the
researched upstream revisions. Together with the total-set rule below, they
replace an open-ended instruction to "port Pydantic tests" with
milestone-owned evidence.

Immediately after `ps_4` creates the external repository and before any core
implementation begins, that repository stores the executable form as
`tests/provenance/upstream_manifest.toml`. A manifest entry identifies an exact
upstream commit, path, pytest selector, parameter identity, compatibility
class, owning milestone, normalized fixture destination, and disposition
reason. Its upstream file set is computed from the pinned Git trees, not from
the hand-written tables:

- tracked entries are enumerated from the pinned Git tree, not a recursive
  filesystem walk. The tracked `tests/pydantic_core` symlink is classified once
  as infrastructure and never followed; engine files are enumerated only from
  `pydantic-core/tests`;
- API and Core pytest collection run in two isolated processes with the
  Pydantic pin's committed `uv.lock`: pytest 9.1.1 and its locked plugins. API
  collection uses `<pydantic-pin>/tests` plus
  `--ignore=<pydantic-pin>/tests/pydantic_core`; Core collection separately
  uses `<pydantic-pin>/pydantic-core/tests`. Normalized identities are prefixed
  `api::` or `core::`, so duplicate basenames cannot collide or acquire a
  second owner. Neither process uses the repository root; explicit path
  arguments bypass the pin's `testpaths` default and prevent it from expanding
  the node universe;
- the standalone 2.41.5 Pydantic Core research checkout is recorded as an
  excluded research source outside the ledger, while the in-tree 2.47.0
  component and API share one immutable Git commit;
- every file is classified as collected conformance, benchmark, fixture,
  infrastructure, or not applicable;
- test collection runs before file-role classification; any file that yields a
  pytest node cannot be hidden as fixture or infrastructure, and every
  collected node and parameter identity in any file is classified as `same`,
  `adapted`, `not-applicable`, or `rejected`;
- sorted upstream paths, collected node identities, and parameter identities
  must exactly equal the manifest ledger; an added, removed, renamed, skipped,
  or unclassified path, node, or parameter fails the audit; and
- the manifest records a content hash for the complete sorted ledger, making
  silent omission from the hand-authored tables detectable.

A collected parameter case records the ordered identity of every contributing
parametrization source. Literal lists use each parameter's normalized
AST-content hash plus its zero-based occurrence among identical hashes.
Non-literal sources are evaluated through pytest collection and use a hash of
their normalized source-dependency closure plus the collected ordinal and
value fingerprint. The dependency closure recursively includes referenced
module constants, comprehensions, starred sources, factory functions, and
fixture parameter declarations. Value fingerprints recursively encode
primitives and containers; types/functions use module, qualified name, and
source hash. An opaque value with no deterministic source fingerprint makes
the audit fail rather than falling back to `repr` or object identity. The
ledger records multiplicity, so repeated equal parameters remain distinct and
a generated source/value change cannot silently preserve the old identity.

The tables then define required implementation anchors and milestone ownership.
The module column identifies the source of that row's anchors; it does not
assign every test in a broad upstream module to the same milestone. A
non-anchor `same` or `adapted` node is owned by the milestone that implements
its feature. These additional rules apply:

- every test and parameter is classified, including cases that are not ported;
- every `same` or `adapted` behavior maps to an explicit Core Schema node,
  error-contract capability, or named Sifr-native contract with one
  implementation milestone and gate; an uncovered capability fails the audit;
- every ordering, precedence, ranking, or normalization algorithm claimed as
  parity records its pinned implementation source and at least one
  discriminating assertion; a non-discriminating anchor cannot certify it;
- the selectors below are mandatory provenance anchors for portable behavior,
  not instructions to copy an upstream test body;
- an anchor must pass, without `xfail` or `skip`, at the pinned revision and
  must contain at least one observable behavioral assertion relevant to Sifr;
- a truthiness-only assertion, import/build smoke test, Python `repr`,
  reflection invariant, or assertion solely about a rejected/not-applicable
  mechanism cannot be a portable anchor;
- for a mixed upstream test, the manifest identifies each retained assertion
  or parameter by a stable AST-content hash, records its normalized Sifr
  expectation, and separately classifies every omitted assertion/parameter;
- a forbidden Python mechanism may appear only as replaceable test harness
  setup around a retained portable assertion, with the Sifr replacement and
  adaptation reason recorded; it cannot be the behavior being asserted;
- every `same` or `adapted` parameterization of an anchor is retained;
  omission is allowed only for an individually justified `not-applicable` or
  `rejected` parameter;
- `py_and_json` cases become both native structural-input and JSON-input
  fixtures where both modes have Sifr meaning;
- Python exception classes, object representations, and versioned message URLs
  are normalized to the selected Sifr error contract; and
- upstream arbitrary custom-error strings normalize to package-qualified Sifr
  custom codes while retaining the declared static message and wrapper
  location; and
- a missing, renamed, ambiguously qualified, or unclassified selector fails the
  upstream audit and the owning milestone gate.

Within a row, a bare filename inherits the directory of the preceding full
path. A bare selector is allowed only when it resolves in exactly one module in
that row; collisions use the full `path::selector`. Repeating an anchor source
module in a later milestone is allowed when different anchors become
executable later, but each retained assertion/parameter has exactly one owning
milestone.

#### Pydantic Core engine baseline

All paths in this table are relative to the authoritative in-tree engine root
`<pydantic checkout at f59e929c999e8b2efc7b12fd0bc1685c1a186be3>/pydantic-core`.
They never resolve against the standalone 2.41.5 research checkout.

| Milestone | Anchor source module | Mandatory portable selector anchors | Fixture family |
| --- | --- | --- | --- |
| `ps_4` | `tests/test_schema_functions.py` | `test_invalid_custom_error`, `test_invalid_custom_error_type`, `test_err_on_invalid` | `core/schema_contract` |
| `ps_4` | `tests/test_json.py` | `test_json_invalid` | `core/json_foundation` |
| `ps_5` | `tests/test_json.py` | `test_input_types`, `test_bool`, `test_int`, `test_float`, `test_json_bytes_base64_round_trip`, `test_json_bytes_base64_invalid` | `core/json_values` |
| `ps_5` | `tests/test_errors.py` | `test_error_json`, `test_error_json_loc`, `test_hide_input_in_error`, `test_error_type` | `core/validation_errors` |
| `ps_5` | `tests/validators/test_bool.py`, `test_int.py`, `test_float.py`, `test_decimal.py` | `test_bool`, `test_bool_strict`, `test_int_py_and_json`, `test_int_strict`, `test_int_kwargs`, `test_float`, `test_float_strict`, `test_float_kwargs`, `test_decimal`, `test_decimal_strict_json`, `test_decimal_kwargs`, `test_validate_max_digits_and_decimal_places` | `validators/numeric` |
| `ps_5` | `tests/validators/test_complex.py` | `test_complex_cases`, `test_complex_strict`, `test_json_complex`, `test_string_complex` | `validators/complex` |
| `ps_5` | `tests/validators/test_string.py`, `test_bytes.py` | `test_str`, `test_constrained_str`, `test_coerce_numbers_to_str`, `test_strict_bytes_validator`, `test_lax_bytes_validator`, `test_constrained_bytes` | `validators/text_bytes` |
| `ps_5` | `tests/validators/test_date.py`, `test_datetime.py`, `test_time.py`, `test_timedelta.py` | `test_date_json`, `test_date_strict_json`, `test_date_kwargs`, `test_datetime_json`, `test_datetime_strict_json`, `test_datetime_past`, `test_datetime_future`, `test_time_json`, `test_time_strict_json`, `test_timedelta_json`, `test_timedelta_strict_json`, `test_timedelta_kwargs` | `validators/temporal` |
| `ps_5` | `tests/validators/test_list.py`, `test_tuple.py`, `test_dict.py`, `test_set.py`, `test_frozenset.py` | `test_list_py_or_json`, `test_list_error`, `test_list_length_constraints`, `test_tuple_json`, `test_tuple_validate`, `test_multiple_missing`, `test_dict`, `test_dict_value_error`, `test_dict_length_constraints`, `test_set_ints_both`, `test_set_multiple_errors`, `test_frozenset_ints_both`, `test_frozenset_multiple_errors` | `validators/collections` |
| `ps_5` | `tests/validators/test_generator.py` | `test_generator_json_int`, `test_error_index`, `test_too_long`, `test_too_short` | `validators/generator` |
| `ps_5` | `tests/validators/test_json.py` | `test_any`, `test_list_int`, `test_dict_key` | `validators/embedded_json` |
| `ps_5` | `tests/validators/test_none.py` | `test_python_none`, `test_json_none` | `validators/none` |
| `ps_5` | `tests/validators/test_url.py` | `test_url_ok`, `test_url_error`, `test_max_length`, `test_allowed_schemes_ok`, `test_allowed_schemes_error`, `test_url_vulnerabilities` | `validators/url` |
| `ps_5` | `tests/validators/test_uuid.py` | `test_uuid`, `test_uuid_strict`, `test_uuid_version`, `test_uuid_json` | `validators/uuid` |
| `ps_5` | `tests/test_validate_strings.py` | `test_bool`, `test_validate_strings`, `test_dict` | `core/strings_profile` |
| `ps_6` | `tests/test_json.py` | `test_typed_dict`, `test_error_loc` | `core/json_models` |
| `ps_6` | `tests/test_errors.py` | `test_loc_with_dots` | `core/model_error_locations` |
| `ps_6` | `tests/test_validate_strings.py` | `test_typed_dict` | `core/strings_profile_models` |
| `ps_6` | `tests/validators/test_model_fields.py` | `test_simple`, `test_with_default`, `test_missing_error`, `test_fields_required_by_default`, `test_alias`, `test_alias_path`, `test_alias_error_loc_alias`, `test_ignore_extra`, `test_forbid_extra` | `validators/model_fields` |
| `ps_6` | `tests/validators/test_typed_dict.py` | `test_simple`, `test_with_default`, `test_missing_error`, `test_fields_required_by_default`, `test_model_deep`, `test_alias`, `test_alias_path`, `test_alias_error_loc_alias`, `test_ignore_extra`, `test_forbid_extra` | `validators/typed_dict` |
| `ps_6` | `tests/validators/test_dataclasses.py` | `test_dataclass_args`, `test_aliases`, `test_dataclass`, `test_dataclass_field_after_validator`, `test_dataclass_json` | `validators/dataclasses` |
| `ps_6` | `tests/validators/test_with_default.py` | `test_typed_dict_default` | `validators/defaults` |
| `ps_6` | `tests/validators/test_nullable.py` | `test_nullable` | `validators/nullable` |
| `ps_7` | `tests/validators/test_literal.py` | `test_literal_py_and_json`, `test_big_int` | `validators/literal` |
| `ps_7` | `tests/validators/test_enums.py` | `test_plain_enum`, `test_int_enum`, `test_str_enum`, `test_enum_exactness`, `test_big_int` | `validators/enum` |
| `ps_7` | `tests/validators/test_union.py` | `test_union_bool_int`, `test_int_float`, `test_left_to_right_union`, `test_smart_union_json_string_types`, `test_smart_union_model_field`, `test_td_smart_union_by_fields_set`, `test_smart_union_does_nested_model_field_counting`, `test_nested_unions_bubble_up_field_count`, `test_smart_union_validator_function`, `test_case_labels`, `test_custom_error` | `validators/unions` |
| `ps_7` | `tests/validators/test_lax_or_strict.py`, `test_json_or_python.py`, `test_chain.py` | `test_lax_or_strict`, `test_lax_or_strict_default_strict`, `test_json_or_python`, `test_chain`, `test_chain_many`, `test_chain_error`, `test_flatten`, `test_chain_empty`, `test_chain_one` | `validators/control_composition` |
| `ps_7` | `tests/validators/test_tagged_union.py` | `test_simple_tagged_union`, `test_discriminator_path`, `test_discriminator_function`, `test_custom_error` | `validators/tagged_unions` |
| `ps_7` | `tests/validators/test_nullable.py` | `test_union_nullable_bool_int` | `validators/nullable_union` |
| `ps_7` | `tests/validators/test_definitions.py`, `test_definitions_recursive.py` | `test_repeated_ref`, `test_deep`, `test_branch_nullable`, `test_recursion_branch`, `test_complex_recursive_type` | `validators/definitions_recursion` |
| `ps_7` | `tests/validators/test_function.py`, `tests/test_validation_context.py`, `tests/validators/test_with_default.py` | `test_function_before`, `test_function_wrap`, `test_function_after`, `test_function_plain`, `test_model_field_before_validator`, `test_model_field_after_validator`, `test_model_field_wrap_validator`, `test_after`, `test_mutable_context`, `test_typed_dict`, `test_wrap`, `test_validate_default_factory`, `test_default_value_validate_default_fail` | `validators/callbacks_context` |
| `ps_8` | `tests/serializers/test_simple.py`, `test_bytes.py`, `test_datetime.py`, `test_decimal.py`, `test_complex.py` | `test_simple_serializers`, `test_float_inf_and_nan_serializers`, `test_bytes`, `test_bytes_base64`, `test_bytes_hex`, `test_datetime`, `test_datetime_json`, `test_date`, `test_time`, `test_decimal`, `test_decimal_json`, `test_complex_json` | `serializers/scalars` |
| `ps_8` | `tests/serializers/test_timedelta.py` | `test_timedelta`, `test_timedelta_float`, `test_config_timedelta`, `test_timedelta_key` | `serializers/duration` |
| `ps_8` | `tests/serializers/test_list_tuple.py` | `test_list_any`, `test_include`, `test_exclude`, `test_filter`, `test_filter_runtime`, `test_filter_runtime_more`, `test_positional_tuple`, `test_filter_args_nested` | `serializers/sequences` |
| `ps_8` | `tests/serializers/test_dict.py` | `test_dict_str_int`, `test_include`, `test_exclude`, `test_filter`, `test_filter_runtime`, `test_filter_args_nested` | `serializers/mappings` |
| `ps_8` | `tests/serializers/test_set_frozenset.py` | `test_set_any`, `test_frozenset_any` | `serializers/sets` |
| `ps_8` | `tests/serializers/test_typed_dict.py` | `test_typed_dict`, `test_include_exclude_args`, `test_alias`, `test_exclude_none`, `test_exclude_default` | `serializers/typed_dict` |
| `ps_8` | `tests/serializers/test_model.py` | `test_model`, `test_include_exclude_args`, `test_alias`, `test_exclude_none`, `test_advanced_exclude_nested_lists`, `test_computed_field_exclude_none` | `serializers/models` |
| `ps_8` | `tests/serializers/test_enum.py`, `test_literal.py`, `test_none.py`, `test_nullable.py` | `test_plain_enum`, `test_int_enum`, `test_str_enum`, `test_int_literal`, `test_str_literal`, `test_none_fallback`, `test_nullable` | `serializers/sums` |
| `ps_8` | `tests/serializers/test_union.py`, `test_definitions_recursive.py`, `test_functions.py` | `test_union_bool_int`, `test_typed_dict_literal`, `test_tagged_union`, `test_tagged_union_with_aliases`, `test_branch_nullable`, `test_function_known_type`, `test_function_only_json` | `serializers/unions_callbacks_recursion` |
| `ps_8` | `tests/serializers/test_definitions.py` | `test_custom_ser`, `test_repeated_ref`, `test_deep`, `test_use_after` | `serializers/definitions` |
| `ps_8` | `tests/serializers/test_url.py`, `test_uuid.py` | `test_url`, `test_url_dict_keys`, `test_uuid`, `test_uuid_key`, `test_uuid_json` | `serializers/url_uuid` |
| `ps_10` | `tests/serializers/test_url.py` | `test_multi_host_url` | `core/multi_host_url_serialization` |

Tests whose Python harness uses `@property` or another descriptor to expose a
computed value are `adapted`: Sifr ports the computed-field serialization
behavior through its statically typed computed-field declaration, not the
descriptor mechanism. In
`serializers/test_model.py::test_computed_field_exclude_none`, the upstream
computed field is normalized to a declared nullable-integer computed field so
both the `None` value and `exclude_none` behavior remain observable without a
wrong-type serializer path.

Upstream `any_schema()`, untyped-container, and `Any`-annotated harnesses are
adapted per retained assertion to the smallest concrete Sifr structural type
that contains that assertion's values. A heterogeneous upstream mapping is
normalized to a declared union key type and concrete value type when those
types represent the retained values; otherwise that parameter is
`not-applicable` with its reason recorded. Neither adaptation introduces
`Any`, an untyped callback, or a recursive dynamic value tree.

Serializer anchors that mix correct typed values with wrong-type
warning/passthrough assertions retain only the correct-value assertions. The
wrong-type assertions are individually `not-applicable`. Likewise,
`test_simple_serializers` excludes subclass-identity parameters, and
`test_none_fallback` retains only parameters where `None` matches the declared
schema. These are assertion/parameter classifications in the manifest, not
alternate runtime paths.

`serializers/sequences::test_positional_tuple` is a mixed anchor: only its
correct declared-tuple serialization assertions are retained; warning,
passthrough, and Python-object `Any` cases are individually
`not-applicable`. `core/validation_errors::test_error_type` anchors canonical
error-code/message availability only; its direct constructor `.type` and
`.context` round-trips do not claim decimal-validator execution. Exact decimal
emission is instead required by `core/decimal_digit_counting` and
`api/constraints::test_decimal_validation`.

`validators/dataclasses` adapts declared field input, aliases, construction,
typed callbacks, and JSON behavior to ordinary Sifr classes. Python dataclass
subclass identity, generated Python initializer signatures, slots,
descriptors, `ArgsKwargs` positional-call harness cases, and object-layout
assertions are individually `not-applicable`; mapping/JSON input cases and
their normalized errors remain.

Enum-member identity assertions that distinguish an enum branch from its
underlying scalar branch are adapted to the selected union variant tag plus
value. The Python `is` relation itself remains `not-applicable`.
Python `str`-enum and beyond-`i64` integer-enum values become package-declared
variant value metadata; retained validation, error alternatives, union tags,
and serialization use those exact literals without changing Sifr's
payload-free enum representation.

Callable-form-only parameterizations, such as
`api/serialization::test_serialize_partial`, collapse to one typed callback
fixture when the retained input, callback effect, and output are identical.
Every collapsed Python callable-form parameter is still individually
classified as `adapted` and points to that shared neutral fixture.

Pydantic public-API anchors may use Python models as their harness, but the
neutral expectation retains only declared field values, serialized output, or
normalized errors. For context-aware anchors it may also retain typed callback
outputs, ordered callback traces, and caller-visible mutations of the declared
context. Assertions about `__dict__`, field-set state, runtime model rebuilds,
Python signatures, internal metadata/reprs, subclass identity, or exception
objects are excluded and classified separately. Heterogeneous Python contexts
become separate statically specialized fixtures with `NoContext` or one
concrete Sifr context type each.

The same projection rule applies to Pydantic Core model-field anchors whose
asserted tuple combines declared values, typed allowed extras, and Python
field-set state: tuple element 0 is retained, element 1 is retained only for a
declared typed `extra_behavior='allow'` destination, and element 2 is
`not-applicable`. The neutral fixture stores those retained observations
separately rather than preserving the Python harness tuple.

`validators/callbacks_context::test_default_value_validate_default_fail` is
adapted from its upstream runtime exception to the build-time diagnostic
required for its const-evaluable invalid default. Its error code and element
location remain provenance inputs; it does not introduce a runtime static
schema failure path.

The Core Schema decimal node is always finite because its Sifr output is
`bigdecimal`. The four `allow_inf_nan` parameters in
`validators/numeric::test_decimal_kwargs` are individually
`not-applicable`; non-finite numeric contracts use the float node rather than
a second decimal representation.

In smart-union anchors, an upstream `isinstance` assertion used only to
identify the chosen model arm is adapted to the corresponding Sifr union
variant tag. Python subclass/reflection behavior remains `not-applicable`; the
selected-arm observation is retained.

`validators/control_composition::test_json_or_python` adapts the Python branch
to Sifr's native structural-input branch. Its observable source-dependent
branch selection and output are retained; Python subclass mechanics and
runtime class reflection are not.

#### Pydantic Sifr-API baseline

All paths in this table are relative to the authoritative repository root
`<pydantic checkout at f59e929c999e8b2efc7b12fd0bc1685c1a186be3>`.

| Milestone | Anchor source module | Mandatory portable selector anchors | Fixture family |
| --- | --- | --- | --- |
| `ps_6` | `tests/test_types.py` | `test_constrained_bytes_good`, `test_constrained_bytes_too_long`, `test_constrained_list_good`, `test_constrained_list_too_long`, `test_constrained_set_good`, `test_constrained_set_too_short`, `test_constrained_str_good`, `test_constrained_str_too_long`, `test_string_too_long`, `test_string_too_short`, `test_string_constraints_ascii_only`, `test_decimal_validation` | `api/constraints` |
| `ps_6` | `tests/test_main.py` | `test_success`, `test_ultra_simple_missing`, `test_ultra_simple_failed`, `test_nullable_strings_success`, `test_parent_sub_model`, `test_required`, `test_default_factory_called_once_2`, `test_allow_extra`, `test_forbidden_extra_fails`, `test_model_validate_strict`, `test_model_validate_json_strict` | `api/base_model` |
| `ps_6` | `tests/test_aliases.py` | `test_basic_alias`, `test_alias_error_loc_by_alias`, `test_pop_by_field_name`, `test_validation_alias`, `test_validation_alias_parse_data`, `test_validation_alias_priority_json` | `api/aliases` |
| `ps_6` | `tests/test_config.py`, `tests/test_fields.py` | `test_config_inf_nan_disabled`, `test_hide_input_in_errors`, `test_populate_by_name_still_effective`, `test_coerce_numbers_to_str_field_option` | `api/config_fields` |
| `ps_7` | `tests/test_validators.py`, `tests/test_model_validator.py` | `test_annotated_validator_before`, `test_annotated_validator_after`, `test_annotated_validator_plain`, `test_annotated_validator_wrap`, `test_annotated_validator_runs_before_field_validators`, `test_validate_multiple`, `test_field_validator_validate_default`, `test_model_validator_before`, `test_model_validator_after`, `test_model_validator_wrap` | `api/validators` |
| `ps_7` | `tests/test_discriminated_union.py` | `test_discriminated_union_validation`, `test_discriminated_annotated_union`, `test_discriminated_union_int`, `test_nested_discriminated_union`, `test_callable_discriminated_union_recursive` | `api/discriminated_unions` |
| `ps_7` | `tests/test_generics.py`, `tests/test_forward_ref.py` | `test_value_validation`, `test_alongside_concrete_generics`, `test_complex_nesting`, `test_required_value`, `test_self_forward_ref_collection`, `test_recursive_model` | `api/generics_recursion` |
| `ps_8` | `tests/test_serialize.py`, `tests/test_computed_fields.py`, `tests/test_aliases.py` | `test_serializer_annotated_plain_json`, `test_serializer_annotated_wrap_json`, `test_model_serializer_plain`, `test_model_serializer_wrap`, `test_serialize_partial`, `test_serialize_json_context`, `test_computed_fields_get`, `test_include_exclude`, `test_exclude_none`, `test_computed_field_with_field_serializer`, `test_serialization_alias` | `api/serialization` |
| `ps_9` | `tests/test_type_adapter.py`, `tests/test_types.py`, `tests/test_serialize.py` | `test_types`, `test_validate_python_strict`, `test_validate_python_context`, `test_validate_json_context`, `test_validate_strings_dict`, `test_decimal_precision`, `test_type_adapter_dump_json` | `api/type_adapter` |
| `ps_9` | `tests/types/test_fraction.py`, `tests/test_types.py` | `test_fraction`, `test_fraction_validate_json`, `test_fraction_validate_strings`, `test_fraction_validation_error_strict`, `test_fraction_dump_json`, `test_strict_complex_field` | `api/specialized_numeric` |
| `ps_9` | `tests/test_json_schema.py`, `tests/test_aliases.py` | `test_by_alias`, `test_sub_model`, `test_optional`, `test_list_union_dict`, `test_constraints_schema_validation`, `test_constraints_schema_serialization`, `test_literal_schema`, `test_new_type_schema`, `test_schema_with_refs`, `test_nested_generic`, `test_discriminated_union`, `test_computed_field`, `test_type_adapter_json_schemas_title_description`, `test_aliases_json_schema` | `api/json_schema` |
| `ps_10` | `tests/test_networks.py` | `test_http_url_success`, `test_http_url_invalid`, `test_any_url_parts`, `test_postgres_dsns`, `test_multihost_postgres_dsns`, `test_json_schema`, `test_url_ser` | `api/networks` |
| `ps_10` | `tests/test_types.py` | `test_pattern` | `api/pattern` |
| `ps_10` | `tests/test_root_model.py` | `test_root_model_validation_error`, `test_root_model_nested`, `test_model_validator_before`, `test_model_validator_after`, `test_root_model_json_schema_meta` | `api/root_model` |
| `ps_10` | `tests/test_annotated.py` | `test_compatible_metadata_raises_correct_validation_error`, `test_decimal_constraints_after_annotation` | `api/field_metadata` |

For `api/pattern`, the string-pattern parameters retain compilation,
match/non-match behavior, and the regex JSON Schema format. Python class-name
reflection and object identity are `not-applicable`; the bytes-pattern
parameter is also `not-applicable` because the canonical Sifr `re.Pattern`
accepts strings.

The tables intentionally name engine and public-API anchors rather than copying
Python scaffolding, while the total-set ledger still enumerates every upstream
test file and collected node. Pydantic modules for pickle, private attributes,
metaclass dynamics, runtime model creation, Python call signatures, import
behavior, mypy plugins, and CPython object lifecycle therefore receive explicit
file/node-level `not-applicable` classifications; absence from the anchor tables
does not remove them from the audited universe.

### Neutral fixtures

Portable cases are stored in a language-neutral fixture format. Each fixture
records:

- an origin variant: `Upstream { repository, commit, test_identifier }` or
  `Native { contract_id, contract_version }`,
- normalized schema,
- input source and value,
- an optional fixed UTC clock instant for clock-relative validation,
- validation/serialization mode,
- expected normalized value or error list,
- compatibility class,
- reason for adaptation/rejection, and
- license/provenance notice when the origin is upstream.

Committed fixtures, not the layout of upstream pytest files, are the stable CI
input. Native-origin fixtures do not enter the upstream exact-set ledger.

### Differential oracle

A development-only differential runner executes the neutral corpus against:

1. pinned Pydantic/Pydantic Core, and
2. the native Sifr implementation.

It normalizes values, locations, codes, and intentional Result-versus-exception
differences before comparison. Published package builds do not invoke Python,
download Pydantic, or require the oracle.

An upstream-audit tool reports newly added or changed relevant upstream cases.
It never changes Sifr behavior or fixtures automatically.

### Upstream pin updates

An upstream revision changes only in a dedicated reviewed compatibility PR:

1. update the sole Pydantic compatibility commit and regenerate the complete
   sorted ledgers for its API and in-tree Core test roots;
2. fail on every added, removed, renamed, skipped, or newly `xfail` node until
   its manifest disposition and owning milestone are reviewed;
3. regenerate only the affected neutral fixtures and differential snapshots;
4. review semantic deltas, provenance, licenses, and benchmark/fuzz seeds;
5. reject automatic behavior changes—an intentional contract change requires
   its own design decision and public compatibility entry; and
6. merge the new pin only when the ledger has exact set equality, every
   retained anchor passes upstream, and both the native corpus and differential
   oracle pass.

Historical manifest revisions remain available so a dependency update cannot
erase why a case was adapted, rejected, or declared not applicable.

## Public Compatibility Policy

The package aims for Pydantic-familiar capability and naming, not Python runtime
emulation.

Permanent Sifr-safe differences include:

- validation and serialization failures return `Result`,
- schemas for statically known types are checked and emitted at build time,
- validators and serializers are statically typed,
- exact Sifr integer behavior is preserved,
- Pydantic's unbounded numeric JSON and JSON Schema expectations are adapted
  to Sifr's locked `json.exact`, `json.web`, or `json.string_ints` profile;
  browser-facing schemas never advertise an unsafe unbounded number,
- ownership and mutation effects remain visible,
- arbitrary runtime class monkey-patching is unsupported,
- Python object identity and attribute probing are unsupported,
- `extra='allow'` is adapted: it is available only when the model declares a
  typed extra-field mapping destination; otherwise extra fields are ignored or
  rejected according to the static model policy,
- `from_attributes`, ORM-style arbitrary attribute probing,
  `revalidate_instances`, and `arbitrary_types_allowed` are not applicable to
  fixed-layout Sifr values,
- `exclude_unset` is not applicable because Sifr models do not retain a
  Python-style per-instance field-set side channel; `exclude_defaults`,
  `exclude_none`, and explicit typed include/exclude selections remain,
- Python `TypedDict` optional-without-default key semantics (`NotRequired` and
  `total=False`) are not applicable to fixed-layout Sifr records: every
  declared field has a value or an explicit default/`Option` representation,
  and upstream assertions whose result omits such a declared key are
  classified `not-applicable`,
- cyclic runtime input objects are not representable by owned Sifr structural
  values; recursive schemas and arbitrarily deep acyclic inputs remain fully
  supported within resource limits,
- serializer wrong-type warnings and passthrough are statically impossible and
  therefore not applicable,
- Pydantic `Decimal` infinity/NaN and `allow_inf_nan` behavior is not
  applicable because Sifr `bigdecimal` is finite by definition,
- simultaneous lower/upper string case conversion is rejected as an
  incompatible static schema instead of inheriting Pydantic's silent
  lower-case precedence,
- `regex_engine='python-re'` is not applicable without a Python runtime;
  portable patterns use the single native Rust-regex contract, and
  `coerce_numbers_to_str` remains a first-class string-node policy,
- a precompiled Python pattern under Pydantic's default engine is adapted only
  when its source and flags translate exactly to Rust `regex`; Python-only
  constructs are rejected at compile-time pattern construction,
- experimental `allow_partial` validation is rejected because it can silently
  discard invalid input while claiming a complete `T`; `missing-sentinel`
  identity is normalized away into ordinary missing-input/default/`Option`
  semantics,
- include/exclude index selection is defined only for sized collections;
  lazy `ValidatedIterator[T]` values require a typed streaming predicate and
  never buffer solely to normalize negative indices,
- Python reflection, internal `repr`, `__dict__`, field-set, subclass-identity,
  and exception-construction assertions may be provenance scaffolding only and
  never define a retained neutral expectation,
- unsupported dynamic behavior fails explicitly rather than falling back, and
- error codes are Sifr-owned even when initially mapped from a Pydantic case.

The compatibility documentation includes a searchable API/behavior matrix.

## Safety and Resource Contract

The native core must:

- contain panics at every package-authored Rust boundary,
- use no data-dependent `unwrap`/`expect`,
- accept only sealed compiler-emitted verified programs and reject a corrupt or
  contract-mismatched envelope before execution,
- guard recursive input and recursive schemas,
- bound input bytes, nesting, collection size, string size, integer digits and
  accumulated errors through explicit policies,
- preserve exact integers without float round trips,
- avoid unsafe code unless separately justified, audited and fuzzed,
- never expose borrowed data beyond its document/arena lifetime,
- never construct partially valid Sifr models,
- avoid quadratic union/alias behavior where an indexed plan is possible, and
- produce deterministic results independent of hash iteration order.

## Performance and Maintainability Contract

- Static schema programs are not rebuilt for every validation call.
- Record field and alias lookup tables are compiled once.
- Tagged-union branch lookup is indexed after direct field/path extraction or
  one typed discriminator callback.
- Validated strings, bytes and big integers are allocated at most once before
  typed construction where ownership permits.
- JSON serialization streams output rather than building a second value tree.
- There is no process, dynamic-library or Python boundary.
- Schema and callback identities participate in build/cache keys.
- Incremental frontend queries cache verified schema programs at the same
  dependency granularity and obey the accepted edit-loop median/p95 budgets.
- Benchmarks separate parse, validate, construct, project and write costs.
- Representative comparisons against pinned Pydantic Core are published, but
  semantic correctness and Sifr safety are never weakened to win a benchmark.
- Once a milestone establishes its performance baseline, unexplained material
  regressions block subsequent milestone closure.
- Rust modules remain responsibility-oriented and below the repository's file
  size guardrail.
- Every schema node has one primary implementation owner, one specification
  table, and one or more focused supporting test families.

## Non-Goals

- Exact source or binary compatibility with Python Pydantic.
- A Python runtime, PyO3 extension, or Python object bridge.
- Supporting Pydantic plugins by executing Python.
- Reusing Pydantic's Python-specific Core Schema nodes.
- Making arbitrary Sifr values dynamically introspectable at runtime.
- Making Core Schema the normal beginner-facing API.
- Runtime model/schema construction or a runtime schema compiler.
- Adding JSON-specific rules to the Sifr compiler.
- Replacing Sifr's ordinary type checker with validation schemas.
- Implementing Pydantic Settings, web-framework integration, ORM behavior or
  unrelated ecosystem packages inside the core architecture. Those may be
  separate packages consuming the completed public contract.
- Supporting a temporary reduced public architecture that later requires a
  second validation engine or compatibility fallback.

## Prerequisites and Dependency Order

Phase 27's no-user-panic, stable diagnostic code/severity/span/URL/schema,
deterministic recovery, and CLI exit-code invariants bind every compiler
milestone. Each compiler PR must pass
`scripts/run_all_tests.sh --profile create-pr`; merge readiness requires
`scripts/run_all_tests.sh`. The external repository establishes equivalent
checked-in create-PR and merge gates before its first implementation commit.

The following Sifr capabilities must be merged and certified before the
companion repository depends on them:

| Required by | Prerequisite |
| --- | --- |
| `ps_1` | approved `ps_0` architecture and compatibility-inventory contract, plus released Phase 40 compiler/tooling foundations |
| `ps_2` | released `ps_1` compiler/sysroot containing compile-time specialization, deterministic const evaluation, `ConstSpecializationOutcome`/`ConstPackageIssue`, registry-owned `SIFR-META-*` and `SIFR-INT-0009`, field required/default metadata, recursive nominal shape identity, structural shape inspection, lossless microsecond/timezone temporal value types, and `frozenset[T]` |
| `ps_3` | released `ps_2` compiler/sysroot containing the merged structural Rust bridge call contract, the already-passing stdlib `opaque_resource_core`, plus completed certification item `certification_pkg_resource_core` with passing `opaque_resource_package_core`, `callbacks_call_scoped`, `panic_boundary_wrapper_emission`, typed construction/projection, callback adapters, and native-backed compiled `re.Pattern` |
| `ps_4` and later | released Sifr compiler/sysroot containing the certified `ps_1` through `ps_3` contracts |

The certification work is tracked by
[`rust-interop-runtime-ecosystem-certification.md`](../archive/rust-interop-runtime-ecosystem-certification.md).
Callback invocation, cleanup, and panic mapping are blocking prerequisites, not
assumed capabilities. No Pydantic-Sifr milestone privately implements or
bypasses an uncertified bridge row.

## Ordered Milestones

Each milestone follows the project workflow:

1. define the complete milestone checklist,
2. implement and validate locally,
3. open its PR,
4. review to satisfaction and merge,
5. release a compiler/package version when the next repository depends on it,
6. update this issue and durable documentation, and
7. only then begin the next milestone.

### milestone_ps_0: Architecture Lock and Compatibility Inventory

- Approve this architecture.
- Freeze the researched upstream revisions.
- Approve the pinned module and selector baseline in this document.
- Define the executable `upstream_manifest.toml` schema and total-set audit
  that expand the API and in-tree Core test roots at the one Pydantic pin into individually classified
  files, collected nodes, and parameters.
- Approve the upstream pin-update procedure.
- Approve the pin-derived 53-kind Core Schema and four-kind field disposition
  table plus its exact-set generated-manifest rule.
- Classify Python-only and portable behavior.
- Define initial compatibility, error-code and provenance tables.
- Define compiler/package/core version relationships.
- Add no production implementation.

Exit gate: independent architecture review finds no unresolved ownership,
semantic-authority, bridge, safety, or sequencing ambiguity; every required
feature family with a meaningful Pydantic oracle has pinned selector anchors,
Sifr-native families such as fixed-width integer overflow have explicit native
contracts, and the design makes an omitted upstream file or collected node
mechanically detectable.

### milestone_ps_1: Compile-Time Shape and Metadata

- Implement the prerequisite compile-time specialization and deterministic
  const-evaluation subsystems.
- Complete field required/default metadata.
- Implement general compile-time structural shape inspection.
- Implement typed declaration metadata.
- Implement `ConstSpecializationOutcome[T]` and bounded
  `ConstPackageIssue`, activating general built-in `SIFR-META-0001`,
  `SIFR-META-0002`, and `SIFR-META-0003` with CLI/LSP parity.
- Implement the package-neutral `JsonIntegerBoundaryDescriptor` verifier and
  activate reserved `SIFR-INT-0009` with registry tests, generated error page,
  and the corresponding `integer_model.md` status update.
- Repair the diagnostics code-coverage guard to validate the canonical `.mdx`
  error pages before adding the new active codes; the ps_1 gate must not inherit
  or mask a pre-existing red diagnostics surface.
- Cover fields, defaults, generics, unions, enums, newtypes/refinements and
  recursive identity.
- Extend the general stdlib with lossless microsecond and timezone-aware
  temporal value types and immutable `frozenset[T]`.
- Add a compiler conformance fixture that is not Pydantic-specific.
- Document the durable general compiler contract.

Exit gate: an external fixture package derives a deterministic static
description of representative types without compiler-known package names,
emits identical fatal/warning structured diagnostics in `check`, build, and
editor analysis, and proves invalid issue declarations are rejected. A second
non-Pydantic fixture proves missing/unsafe integer-boundary descriptors emit
registry-owned `SIFR-INT-0009`.

### milestone_ps_2: Construction, Projection and Typed Callbacks

- Specify and merge the monomorphized structural Rust bridge call contract
  into `internal_docs/rust_interop_architecture.md` before implementation.
- Consume the already-passing stdlib `opaque_resource_core`. This milestone
  creates general package-resource support but no Pydantic resource; after its
  release, the certification issue's sequential
  `certification_pkg_resource_core` item must create and pass
  `opaque_resource_package_core` before `ps_3` begins. Service-specific
  `opaque_resource_matrix` evidence remains out of scope.
- Block on the certification issue's passing `callbacks_call_scoped`, including
  callback-invocation panic mapping, and `panic_boundary_wrapper_emission`
  rows; this phase does not privately take their ownership.
- Implement the accepted structural Rust bridge contract.
- Atomically remove `[rust] bridge-version` from the manifest schema, every
  in-repository bridge manifest and fixture, managed projections, archive
  expectations, cache records, diagnostics, and generated-build assertions.
  Remove the Rust-interop fixture matrix's top-level `bridge_version` marker,
  the `check_fixture_matrix.py` and `_scenario_checks.py` assertions that
  require it, `_scenario_registry.py`'s literal token,
  `_matrix_inventory.py`'s required-fixture entry, and
  `runner/bridge_check.py`'s version parameter/default. Remove the
  `rust_interop_plan.rs` module/cache fields, package-graph digest field, and
  sysroot's synthesized `Some(1)`. Delete the complete `bridge_version_mismatch`
  fixture/scenario and its matrix, tier, and stable-claim entries rather than
  retaining legacy acceptance evidence.
- Delete the `bridge-version = 1` subsection and rewrite every remaining
  version-keyed statement in `internal_docs/**`, `docs/**`, active issues,
  `plans/phases/**`, and the roadmap. This explicitly includes
  `docs/packages/manifest.mdx`, `docs/rust-interop.mdx`, the Blake3 and Reqwest
  interop guides, and
  `internal_docs/sifr_sysroot_and_stdlib_architecture.md`. Dated reviews, issue
  archives, and frozen release-candidate evidence remain immutable history.
- Reject the removed field through an explicit diagnostic rather than relying
  on the manifest parser's unknown-key behavior, and provide no compatibility
  path, rewrite, shim, or fallback. Replace the current
  `bridge_version_mismatch` evidence with passing
  `bridge_version_field_removal` evidence in the same implementation PR. Bind
  its positive side to a driver contract test and prove the repo-wide cutover
  through the now-unversioned package/scenario examples.
- Promote `structural_bridge_calls` from future-owned to supported-through-bridge
  only when both directions pass; remove it from the stable-support runtime
  deferrals and update the public stable-claims documentation atomically.
- Add the compiler-owned `sifr.meta.Structural` marker, recognize
  `@rust.structural` as the sole bare Rust marker, and diagnose marker arguments,
  missing targets, duplicate markers, and invalid generic placement through
  `SIFR-RUST-CONFIG-0001` / `SIFR-RUST-TYPE-*` with targeted tests.
- Implement safe structural `Construct[T]`.
- Implement allocation-free structural projection/visitation.
- Implement typed callback adapter generation.
- Upgrade stdlib `re.Pattern` to use the general opaque-resource substrate,
  compiling once while preserving readable source and flags. Because Sifr has
  no overloads, retain both public names but correct both signatures
  atomically: `compile(pattern) -> Result[Pattern, RegexError]` and
  `compile_flags(pattern, flags) -> Result[Pattern, RegexError]`. Update all
  stdlib/compiler call sites and docs in the same pre-stable contract PR;
  invalid patterns fail at construction and no infallible compatibility path
  remains.
- Prove ownership, move, borrow, error and panic behavior.
- Extend non-Pydantic compiler conformance fixtures.

Exit gate: a fixture package round-trips nested generic/recursive values through
a native opaque resource without dynamic reflection, layout assumptions, or
untyped callbacks.

### milestone_ps_3: Static Program and Native Bridge Contract

- Implement deterministic static schema-program emission support.
- Implement sealed arena/document opaque resources and compact node indices.
- Add generic signature probes, installed/source parity, cleanup, and cache-key
  contracts while consuming the already-passing unversioned-manifest contract.
- Prove exact integer, bytes, collection and error crossings.
- Update Rust interop architecture and verification.

Implementation checklist:

- [x] Give each retained const-specialization result one deterministic program
  identity. Include its owner, package function, canonical value, structural
  contract, and declaration metadata in that identity.
- [x] Emit immutable program bytes and a sealed typed program envelope during
  build. Retain the same identity during check and editor analysis.
- [x] Include the program identity in generated-project cache keys. Prove that
  relevant program inputs invalidate the key and unrelated declarations do not.
- [x] Add reusable sealed document and validated-arena runtime types. Use
  checked compact node indices and one move-only structural source.
- [x] Add one synthetic external package that consumes the unversioned Rust
  manifest contract. It must use a generic structural signature probe and one
  monomorphized executor call.
- [x] Prove exact integers, fixed integers, bytes, lists, mappings, records,
  moved scalar payloads, typed errors, corrupt envelopes, and invalid indices.
- [x] Prove source and installed package parity, deterministic generated output,
  cache-key behavior, and cleanup after success and failure.
- [x] Register passing positive and negative Rust-interop evidence. Update the
  compatibility matrix, fixture matrix, tiers, and durable architecture.
- [x] Run targeted tests, open the milestone pull request, and obtain an exact
  candidate `SATISFIED` Opus review before the create-PR and merge gates.

Exit gate: the merged and certified structural Rust bridge contract lets a
synthetic schema executor return a validated arena, construct a typed Sifr
value, and pull a structural view for output through one monomorphized call.

### milestone_ps_4: Companion Repository and Core Foundation

- Require the released Sifr compiler/sysroot containing certified `ps_1`
  through `ps_3` contracts.
- Create the standalone public GitHub repository
  [`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr) under
  the `sifr-lang` organization.
- Establish the external Sifr package and Rust backend layouts there.
- Track, review, merge, and release all package/core implementation from that
  repository from this milestone onward.
- Materialize the total-set `upstream_manifest.toml` before core
  implementation; prove exact equality with both test roots at the sole
  Pydantic pin and explicitly exclude the historical standalone Core checkout.
- Generate `tests/provenance/core_schema_kinds.toml` from the pinned
  `CoreSchemaType`/`CoreSchemaFieldType` literals and prove exact equality with
  the accepted disposition table before defining format version 1.
- Define Core Schema/program format version 1.
- Implement that canonicalizer/verifier once as deterministic Sifr package
  code and emit sealed `VerifiedSchemaProgram[T]` static data in every
  specializing frontend mode.
- Define the built-in/custom error-code registry and verify compositional
  `ErrorOverride` declarations.
- Add error, input, arena and plan foundations.
- Integrate Python-free `jiter`.
- Establish licenses, provenance, fuzzing and benchmark harnesses.

Exit gate: `core/schema_contract` and `core/json_foundation` pass; malformed
schemas and malformed JSON return stable typed errors with zero panics under
unit, property and fuzz tests; the upstream ledger has no missing path/node or
unclassified entry; and `core_schema_kinds.toml` is exact-set-equal to all
pinned Core Schema and field kinds with one accepted primary owner and evidence
set/disposition audit per row.

### milestone_ps_5: Scalar and Collection Validation

- Implement scalar schema nodes and strict/lax conversion.
- Implement exact/fixed integers, floats, decimals, exact rational fractions,
  complex values, strings and bytes.
- Integrate temporal and focused scalar libraries, including the Core Schema
  compiled-pattern value node over stdlib `re.Pattern`.
- Implement numeric/decimal, string-normalization, pattern, length, and
  call-scoped clock-relative temporal constraints with the specified ordering.
- Implement lists, tuples, mappings, sets and frozen sets.
- Implement lazy `ValidatedIterator[T]` with fallible `next`, stable deferred
  error indices, and length/resource limits; it is not silently collected.
- Implement the embedded-JSON decoder after manifest adaptation supplies an
  explicit statically known child schema.
- Implement native, JSON, and strings input profiles over one validation
  engine.
- Port the corresponding neutral Pydantic Core corpus.

Exit gate: `core/json_values`, `validators/numeric`,
`validators/complex`, `validators/text_bytes`, `validators/temporal`, `validators/collections`,
`validators/generator`, `validators/embedded_json`, `validators/none`,
`validators/url`, `validators/uuid`,
`core/fixed_integer`, `core/pattern_value`, `core/string_pipeline_order`,
`core/decimal_digit_counting`, `core/fraction`, `core/strings_profile`, and
`core/validation_errors` pass; all intentional differences are recorded,
`JsonLimitError` covers integer-digit exhaustion before allocation, and all
resource limits are enforced.

### milestone_ps_6: Models, Fields, Defaults and Aliases

- Implement model/record schemas.
- Implement required/defaulted/nullable distinctions.
- Implement field metadata, aliases and alias paths.
- Implement extra-field policies and ephemeral validated-field-count tracking.
- Implement typed construction into ordinary Sifr classes.
- Expose the first complete `BaseModel` validation API, including JSON,
  structural, and strings-profile entry points.

Exit gate: nested models validate JSON and native structural inputs into typed
Sifr values with aggregate stable errors and no third arena-to-model bridge
tree; `core/json_models`, `validators/model_fields`,
`core/model_error_locations`, `core/strings_profile_models`,
`validators/typed_dict`, `validators/dataclasses`, `validators/defaults`, `validators/nullable`,
`api/base_model`, `api/aliases`, `api/config_fields`, and `api/constraints`
pass.

### milestone_ps_7: Unions, Recursion and Custom Validation

- Implement literals, enums, ordinary unions, field/path-discriminated tagged
  unions, and typed-callback-discriminated tagged unions.
- Implement deterministic `smart` and `left_to_right` ordinary-union modes,
  labelled aggregate errors, nested ranking-state bubbling, `Omit` handling,
  and the declared auto-collapse policy.
- Execute compositional error overrides, including union and tagged-union
  custom errors, through the single aggregate-error path.
- Implement definitions, references and recursion guards.
- Implement strict/lax, JSON/structural-input, and flattened typed-chain
  control composition.
- Implement before/after/wrap/plain typed validators.
- Implement field/model validator ordering and caller-owned typed validation
  context, including immutable and mutable borrow modes.
- Port the corresponding upstream behavior corpus.

Exit gate: ambiguous, recursive and callback-heavy cases have deterministic
success/error behavior, bounded execution and complete ownership coverage;
`validators/literal`, `validators/enum`, `validators/unions`,
`validators/control_composition`,
`validators/tagged_unions`, `validators/nullable_union`,
`validators/definitions_recursion`, `validators/callbacks_context`,
`core/recursion_limit`, `core/smart_union_ranking`, `api/validators`,
`api/discriminated_unions`, and `api/generics_recursion` pass.

### milestone_ps_8: Serialization

- Implement serializer plans over structural projections.
- Implement structural and streaming JSON outputs.
- Implement aliases, typed recursive include/exclude selections, and
  default/none policies.
- Implement custom field/model serializers and computed fields.
- Implement caller-owned typed serialization context forwarding.
- Preserve temporal output policies and implement Sifr's selected integer JSON
  profile through `sifr_runtime::json`, including typed range errors.
- Port serialization tests and benchmarks.

Exit gate: mutated typed models serialize from their current values, not a
retained validation arena, and no full generic output tree is required for
JSON; every `serializers/*` fixture family named in the baseline and
`core/selection_precedence` and `api/serialization` pass.

### milestone_ps_9: TypeAdapter and JSON Schema

- Implement reusable `TypeAdapter[T]`.
- Implement native, JSON, and strings-profile validation plus serialization
  modes.
- Generate JSON Schema from the same Core Schema.
- Reflect the selected Sifr integer JSON profile and static range in every
  integer schema, failing closed with `SIFR-INT-0009` when ambiguous.
- Before the external package release, merge a coordinated `sifr-lang/sifr`
  documentation/verification PR updating
  `verification/areas/core_language/data/integer_model/serialization_boundary_rules.md`
  with the implemented descriptor consumer, generated-client warning
  ownership, and exact bounded JSON Schema snapshots; update
  `internal_docs/integer_model.md` to name
  `x-sifr-integer-profile: exact` as the implemented exact-profile schema
  marker. `ps_1` already owns the diagnostic page and Reserved-to-Active
  diagnostic status change.
- Support definitions, recursion, aliases, constraints and mode-specific
  representations.
- Complete public `Fraction` and `Complex` adapter/schema representations.
- Add deterministic schema snapshots and dialect conformance.

Exit gate: validation, serialization and description agree for every supported
schema node, with no Schemars or alternate metadata authority;
the coordinated Sifr boundary-artifact PR is merged and its snapshots pass;
`api/type_adapter`, `api/specialized_numeric`, and `api/json_schema` pass.

### milestone_ps_10: Full Pydantic-Familiar Surface

- Complete the selected `BaseModel`, `Field`, configuration, validator,
  serializer, computed-field and adapter APIs.
- Complete the selected root-model, specialized network type, field-metadata,
  compiled-pattern field/API, and public error surfaces.
- Publish the API/behavior compatibility matrix.
- Add migration documentation for Pydantic users.
- Prove ordinary Sifr classes and the familiar facade use the same engine.
- Remove any temporary internal API exposed during construction.

Exit gate: the documented end-state public API is complete; no public fallback,
temporary schema form or second validation path remains; `api/networks`,
`core/multi_host_url_serialization`, `api/pattern`, `api/root_model` and
`api/field_metadata` pass.

### milestone_ps_11: Certification and Release

- Re-audit the already-complete manifest against its pinned revisions and the
  documented update-pin procedure; no compatibility coverage is deferred to
  this milestone.
- Run differential validation against the pinned oracle.
- Complete fuzz, property, adversarial resource and panic testing.
- Publish parse/validate/construct/serialize benchmarks.
- Certify supported compiler/core/package version combinations.
- Add end-to-end demos and package documentation.
- Add and snapshot-test the canonical
  `demos/pydantic_sifr_demo.sifr` in the external `pydantic-sifr` repository.
- Perform independent whole-architecture and implementation review.

Exit gate: all acceptance criteria pass using released Sifr and
`pydantic-sifr` artifacts without access to the source checkout, Python, or the
upstream repositories, and the canonical demo runs from an installed package
without a Sifr compiler source checkout.

## Acceptance Criteria

### Architecture

- `sifr-lang/sifr` contains no Pydantic-specific compiler branch, type, schema
  node, decorator name or JSON validation policy.
- The external
  [`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr)
  repository contains the Sifr package and native core as separately owned
  components with one versioned schema contract.
- `sifr-lang/sifr` contains no production `pydantic` package or
  `pydantic_sifr_core` source as a workspace member, vendored subtree, or
  submodule.
- Validation, serialization and JSON Schema generation consume one Core Schema
  authority.
- Static schemas are verified and deterministically materialized during
  `check`, build, editor analysis, and every specializing frontend mode by the
  same package const implementation; build-like modes embed the result.
- There is no runtime schema compiler or alternate dynamic adapter path.
- The structural Rust bridge is a merged, certified general contract with a
  non-Pydantic conformance consumer.
- Package const specialization can emit bounded issues whose package reason is
  mapped to registry-owned `SIFR-META-*` diagnostics with identical CLI/LSP
  identity through a non-Pydantic conformance consumer.

### Native execution

- Published artifacts contain no Pydantic, Pydantic Core, PyO3, CPython, GIL or
  dynamic extension dependency.
- JSON input uses Python-free `jiter`.
- Exact Sifr integers survive parse, validation, construction and serialization.
- Integer JSON and generated-schema behavior route through one explicitly
  selected locked Sifr profile; missing/unsafe policy fails with
  `SIFR-INT-0009` or `JsonIntegerRangeError` as appropriate.
- Successful validation constructs the requested native Sifr type.
- Serialization observes the current typed value after mutation.
- Construction and serialization each use one monomorphized structural native
  call; the core never imports generated package bridge types.
- User-controlled data and callbacks cannot produce an uncaught Rust panic.

### Behavior

- Required Pydantic-equivalent features have neutral fixtures and provenance.
- Every relevant upstream case is classified as same, adapted, not applicable
  or rejected.
- The manifest's sorted file/node ledger exactly equals the API and in-tree
  Core test roots at the sole Pydantic pin; no upstream path, collected
  selector, or parameter can disappear without failing the audit.
- The generated Core Schema kind ledger exactly equals every pinned
  `CoreSchemaType` and `CoreSchemaFieldType` literal, with one accepted
  primary implementation/disposition owner and either a non-empty
  evidence-family set or the explicit `ps_0` disposition audit per kind.
- Every fixture family assigned to a milestone passes before that milestone
  releases; `ps_11` performs re-audit and certification, not deferred behavior
  implementation.
- Strict/lax behavior, union ranking, error ordering and serializer profiles are
  deterministic and documented.
- Validation returns aggregate typed errors with stable codes and locations.
- Intentional Sifr differences are public and tested.

### Maintainability

- No permanent fork of Pydantic or Pydantic Core exists.
- Mature focused Rust dependencies are reused at their natural boundary.
- No schema behavior is implemented independently in both Sifr and Rust.
- No third arena-to-model bridge tree or per-call schema rebuild exists.
- Dependency features, licenses and provenance are audited.
- Fuzzing covers schema verification, JSON input, validation plans and writers.
- Benchmarks and regression gates cover each execution stage.
- Compiler and package conformance tests prevent accidental coupling.

### Delivery

- Milestones are delivered sequentially through reviewed PRs.
- Corresponding durable architecture and status documents are updated after
  every milestone.
- The canonical `demos/pydantic_sifr_demo.sifr` lives in
  `sifr-lang/pydantic-sifr`, is covered by that repository's local validation,
  and builds and runs against installed/released artifacts.
- `sifr-lang/sifr` contains only package-neutral compiler conformance demos and
  fixtures; it contains no Pydantic-Sifr product demo.
- Authoritative local validation passes in both repositories.
- Independent final review confirms the implementation matches this end state
  without fallback paths or split semantic authority.

## Exit Gate

This ad hoc phase is complete only when a Sifr user can install the released
`pydantic-sifr` package and use a Pydantic-familiar, fully native API to:

1. derive a schema for a typed model,
2. validate hostile JSON into that model,
3. receive deterministic aggregate `Result` errors,
4. run typed custom validators,
5. mutate the resulting native model,
6. serialize its current state through a selected profile,
7. generate matching JSON Schema, and
8. do all of the above without Python, a Pydantic Core fork, compiler package
   special cases, duplicated schema authorities, or user-triggerable panics.
