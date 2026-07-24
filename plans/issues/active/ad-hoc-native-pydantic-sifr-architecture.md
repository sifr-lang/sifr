# Ad Hoc Phase: Native Pydantic-Sifr Architecture

## Status

Proposed architecture and implementation issue. Research and design are
complete; implementation has not started.

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

| Repository | Researched revision |
| --- | --- |
| `pydantic/pydantic` | `f59e929c999e8b2efc7b12fd0bc1685c1a186be3` |
| `pydantic/pydantic-core` | `383eb95a19433754c0cecf7025b50c26b6d97a36` |

Both upstream repositories are MIT licensed. Copied implementation fragments or
test data must retain the required notice and provenance.

Existing Sifr contracts: [`rust_interop_architecture.md`](../../../internal_docs/rust_interop_architecture.md),
[`sifr_sysroot_and_stdlib_architecture.md`](../../../internal_docs/sifr_sysroot_and_stdlib_architecture.md),
[`integer_model.md`](../../../internal_docs/integer_model.md), and
[`architecture.md`](../../../internal_docs/architecture.md).

The compiler substrate also depends on the core rows tracked by
[`rust-interop-runtime-ecosystem-certification.md`](rust-interop-runtime-ecosystem-certification.md).

## End-State Decisions

1. `pydantic-sifr` is a separate repository in the `sifr-lang` organization.
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
8. `pydantic_sifr_core` owns Core Schema verification, plan execution, input
   adapters, aggregate errors, and performance-sensitive algorithms.
9. Derived static schemas become immutable schema programs during build.
   There is no runtime schema-compilation path.
10. Core Schema is the sole authority for validation, serialization, and
    description. Serde, Schemars, and another validator are not parallel
    authorities.
11. Rust bridge version 2 adds one general, trait-bounded structural call
    contract. It does not add Pydantic-specific bridge types or container
    exceptions.
12. Native decoding returns a validated value arena. The JSON parse tree and
    normalized arena are expected; no third copied bridge-object tree exists.
13. Compiler-generated structural traits materialize a validated source into
    the requested Sifr type and project typed Sifr values to native consumers.
14. `pydantic_sifr_core` invokes those traits through one monomorphized native
    call. It never imports package-generated bridge types.
15. `jiter`, without its Python feature, is the canonical JSON parser.
16. `speedate` is a temporal parsing mechanism where its behavior matches the
    selected Sifr contract; it is not a public temporal representation.
17. Serde and `serde_json` may provide format/writer mechanisms but do not
    define validation, coercion, errors, or schemas.
18. Focused Rust crates are reused for regex, URL, UUID, Base64, IDNA, and
    arbitrary-precision numeric mechanisms.
19. Pydantic and Pydantic Core are development oracles and provenance sources,
    never dependencies of published artifacts.
20. Compatibility means equivalent behavior where Python and Sifr correspond,
    with every divergence documented.
21. Delivery is sequential: implement, validate, review, merge, and release a
    milestone before starting the next.

## Repository Ownership

### `sifr-lang/sifr`

Owns only general language and package substrate:

- compile-time type shape inspection,
- compile-time declaration metadata,
- safe structural construction,
- safe structural projection/visitation,
- specialization of generic package code,
- typed package callback adapters,
- static data emission,
- Rust bridge support required by general native packages,
- package/compiler compatibility declarations, and
- compiler conformance fixtures.

The compiler must be able to explain these features without mentioning
Pydantic. Database mappers, RPC systems, command-line parsers, encoders,
decoders, and other packages must be able to consume the same substrate.

### `sifr-lang/pydantic-sifr`

Owns:

- the public Sifr package,
- the native core crate,
- the versioned Core Schema contract,
- compatibility and differential tests,
- fuzz targets,
- benchmarks,
- upstream provenance,
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
  benchmarks/
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
       canonicalize/verify
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

Milestones `ps_1` through `ps_3` create new compiler subsystems; they are not
small extensions of the current generics or Rust bridge implementation. Their
gated prerequisites are:

- compile-time specialization of package generics for a concrete `T`,
- deterministic compile-time evaluation sufficient to derive and emit static
  data,
- first-class field required/defaulted metadata rather than reconstruction from
  an `__init__` signature,
- payload-bearing enum support, or completion of that language capability
  before shapes may expose enum payloads,
- exact recursive nominal identity, and
- the bridge-version 2 structural call contract described below.

Until payload-bearing enums ship, C-like enums remain supported and tagged
payload sums use ordinary unions of records. That is an implementation
dependency, not a second permanent schema representation.

### Rust bridge version 2: structural calls

The existing bridge-compatible value table remains closed. Bridge version 2
does not make tuple, set, arbitrary mapping, payload enum, or specialized
scalar values directly cross the boundary as ad hoc bridge types.

Instead, the compiler generates implementations of two stable,
language-general traits owned by `sifr_runtime`:

```text
StructuralSource
    shape_identity() -> ShapeIdentity
    root() -> NodeId
    take/read nodes through a sealed stable interface

StructuralConstruct
    construct[S: StructuralSource](source: own S) -> Result[Self, ContractError]

StructuralProject
    expose(self: &Self) -> StructuralView
```

The names above are conceptual; the accepted bridge-version 2 design fixes the
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
- projection borrows the current typed value and exposes a call-scoped view,
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

Bridge version 2 must specify:

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
- enum variants and payloads,
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
- a function,
- a method, and
- a parameter.

Metadata values must be statically typed and compile-time evaluable. The
compiler preserves and exposes them to specializing package code; it does not
interpret `Field`, validator, serializer, or model configuration semantics.

This mechanism is the substrate for Pydantic-familiar `Field`,
`field_validator`, `model_validator`, `field_serializer`, `computed_field`,
and configuration declarations without hard-coding their names.

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
- enum variant and payload access,
- primitive borrowing,
- collection iteration,
- optional/union discrimination, and
- profile-controlled field presence.

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
- panic containment at the Rust boundary, and
- non-send/send restrictions.

There is no universal untyped callback receiving an arbitrary runtime object.

### Static schema emission

For derived and otherwise statically declared schemas, specialization produces
a schema graph during check/build. A build-time
`pydantic_sifr_core` compiler canonicalizes and verifies that graph and emits
the deterministic immutable schema program embedded in the generated binary.
The program contains stable node arrays, string tables, references,
constraints, policies, and typed callback slots.

The same schema program must have the same identity across `check`, `build`,
`run`, tests, cache keys, and editor analysis.

At runtime the core borrows the embedded program directly. Header/version/hash
verification is allowed; graph parsing, schema compilation, validator
construction, and cache population per process or per call are not.

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
- serialize `T` to a structural value,
- serialize `T` to JSON,
- obtain a reusable `TypeAdapter[T]`,
- obtain JSON Schema for the selected serialization/validation mode, and
- customize validation/serialization through typed declarations.

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
| Specialized scalars | date, time, datetime, duration, UUID, URL, pattern and package-provided scalar adapters |
| Constraints | numeric bounds/multiples, length bounds, pattern, finite, uniqueness and typed refinement |
| Products | record/model, tuple, typed mapping |
| Collections | list, set, frozen set and typed sequence policies supported by Sifr |
| Sums | optional, literal, enum, ordinary union and tagged union |
| Control | default, nullable, definitions, reference and recursion guard |
| Transforms | before, after, wrap and plain typed validators |
| Serialization | alias, inclusion/exclusion, computed field, typed serializer and representation override |

Nodes are orthogonal and compositional. For example, constrained integers are
an integer node plus constraints rather than independent positive-int,
negative-int, bounded-int, and strict-int validator implementations.

### Program invariants

Core Schema verification rejects:

- dangling references,
- duplicate definition identities,
- impossible type/output relationships,
- callback signature mismatches,
- invalid constraint combinations,
- serialization nodes incompatible with validation output,
- unbounded recursive entry,
- ambiguous discriminator maps,
- defaults that do not validate under their declared policy, and
- unknown schema versions or node kinds.

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

- build-time schema graph verification and immutable program emission,
- runtime validation and serializer program execution without recompilation,
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
values. Examples include calendar/time components plus offset and precision,
UUID bytes, normalized URL text/components, and exact decimal coefficient and
scale. `StructuralConstruct` reconstructs the existing canonical Sifr stdlib
type:

- `datetime`, `date`, `time`, and duration use the stdlib's chrono-backed
  representation,
- UUID and URL use the existing stdlib-backed types, and
- decimal values use Sifr's selected exact decimal representation.

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
- input source kind,
- context values,
- resource limits, and
- callback state.

Tagged and untagged unions are separate schema nodes and algorithms.

A tagged union uses its discriminator map to select exactly one branch or
return a discriminator error. An untagged smart union follows the pinned
Pydantic Core algorithm where applicable:

1. an exact successful candidate short-circuits,
2. otherwise record-like candidates rank first by the internal count of
   successfully validated declared fields,
3. exactness class breaks equal-count ties,
4. declaration order is the final deterministic tie break, and
5. total failure reports candidate errors in stable declaration order.

The internal field count is ephemeral validation state used only for ranking.
It is not a public `__pydantic_fields_set__` attribute and is not retained on
the constructed Sifr model.

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
- unset/default/none policies,
- computed fields,
- tagged-union representation,
- custom serializers,
- exact integer output policy, and
- target-format constraints.

JSON output is streamed to a writer. It does not allocate a complete
`serde_json::Value` first. `serde_json` mechanisms may be reused for escaping
and scalar formatting, while Sifr's schema program remains the semantic
authority.

## Error Contract

All user-data failures return one `ValidationError` containing an ordered list
of `ErrorDetail` values.

Each detail contains:

- stable machine-readable code,
- ordered location segments,
- human-readable message,
- expected contract summary,
- safe input summary,
- optional context,
- optional JSON byte/line/column position, and
- originating schema node identity for diagnostics and testing.

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
integers are arbitrary precision. Their range, overflow, strict/lax, error, and
serialization behavior is specified and tested as a Sifr-native contract rather
than classified as Pydantic parity.

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

### Neutral fixtures

Portable cases are stored in a language-neutral fixture format. Each fixture
records:

- upstream repository,
- upstream commit,
- original test identifier,
- normalized schema,
- input source and value,
- validation/serialization mode,
- expected normalized value or error list,
- compatibility class,
- reason for adaptation/rejection, and
- license/provenance notice.

Committed fixtures, not the layout of upstream pytest files, are the stable CI
input.

### Differential oracle

A development-only differential runner executes the neutral corpus against:

1. pinned Pydantic/Pydantic Core, and
2. the native Sifr implementation.

It normalizes values, locations, codes, and intentional Result-versus-exception
differences before comparison. Published package builds do not invoke Python,
download Pydantic, or require the oracle.

An upstream-audit tool reports newly added or changed relevant upstream cases.
It never changes Sifr behavior or fixtures automatically.

## Public Compatibility Policy

The package aims for Pydantic-familiar capability and naming, not Python runtime
emulation.

Permanent Sifr-safe differences include:

- validation and serialization failures return `Result`,
- schemas for statically known types are checked and emitted at build time,
- validators and serializers are statically typed,
- exact Sifr integer behavior is preserved,
- ownership and mutation effects remain visible,
- arbitrary runtime class monkey-patching is unsupported,
- Python object identity and attribute probing are unsupported,
- `extra='allow'` is adapted: it is available only when the model declares a
  typed extra-field mapping destination; otherwise extra fields are ignored or
  rejected according to the static model policy,
- `from_attributes`, ORM-style arbitrary attribute probing,
  `revalidate_instances`, and `arbitrary_types_allowed` are not applicable to
  fixed-layout Sifr values,
- unsupported dynamic behavior fails explicitly rather than falling back, and
- error codes are Sifr-owned even when initially mapped from a Pydantic case.

The compatibility documentation includes a searchable API/behavior matrix.

## Safety and Resource Contract

The native core must:

- contain panics at every package-authored Rust boundary,
- use no data-dependent `unwrap`/`expect`,
- reject invalid schema programs before execution,
- guard recursive input and recursive schemas,
- bound input bytes, nesting, collection size, string size and accumulated
  errors through explicit policies,
- preserve exact integers without float round trips,
- avoid unsafe code unless separately justified, audited and fuzzed,
- never expose borrowed data beyond its document/arena lifetime,
- never construct partially valid Sifr models,
- avoid quadratic union/alias behavior where an indexed plan is possible, and
- produce deterministic results independent of hash iteration order.

## Performance and Maintainability Contract

- Static schema programs are not rebuilt for every validation call.
- Record field and alias lookup tables are compiled once.
- Tagged-union dispatch is indexed.
- Validated strings, bytes and big integers are allocated at most once before
  typed construction where ownership permits.
- JSON serialization streams output rather than building a second value tree.
- There is no process, dynamic-library or Python boundary.
- Schema and callback identities participate in build/cache keys.
- Benchmarks separate parse, validate, construct, project and write costs.
- Representative comparisons against pinned Pydantic Core are published, but
  semantic correctness and Sifr safety are never weakened to win a benchmark.
- Once a milestone establishes its performance baseline, unexplained material
  regressions block subsequent milestone closure.
- Rust modules remain responsibility-oriented and below the repository's file
  size guardrail.
- Every schema node has one implementation owner, one specification table and
  one focused test family.

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

The following Sifr capabilities must be merged and certified before the
companion repository depends on them:

| Required by | Prerequisite |
| --- | --- |
| `ps_1` | compile-time specialization, deterministic const evaluation, field required/default metadata, recursive nominal shape identity, and payload-bearing enums |
| `ps_2` | bridge-version 2 structural traits plus `opaque_resource_core`, the required `callbacks_call_scoped_core` split, and `panic_boundary_wrapper_emission` |
| `ps_3` | the complete bridge-version 2 design in `internal_docs/rust_interop_architecture.md`, installed/source parity, generic signature probes, cleanup and cache identity |
| `ps_4` and later | released Sifr compiler/sysroot containing the certified `ps_1` through `ps_3` contracts |

The certification work is tracked by
[`rust-interop-runtime-ecosystem-certification.md`](rust-interop-runtime-ecosystem-certification.md).
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
- Build the Pydantic/Pydantic Core feature and test inventory.
- Classify Python-only and portable behavior.
- Define initial compatibility, error-code and provenance tables.
- Define compiler/package/core version relationships.
- Add no production implementation.

Exit gate: independent architecture review finds no unresolved ownership,
semantic-authority, bridge, safety, or sequencing ambiguity.

### milestone_ps_1: Compile-Time Shape and Metadata

- Implement the prerequisite compile-time specialization and deterministic
  const-evaluation subsystems.
- Complete field required/default metadata and payload-bearing enum support.
- Implement general compile-time structural shape inspection.
- Implement typed declaration metadata.
- Cover fields, defaults, generics, unions, enums, newtypes/refinements and
  recursive identity.
- Add a compiler conformance fixture that is not Pydantic-specific.
- Document the durable general compiler contract.

Exit gate: an external fixture package derives a deterministic static
description of representative types without compiler-known package names.

### milestone_ps_2: Construction, Projection and Typed Callbacks

- Complete the required opaque-resource, call-scoped-callback, and panic
  boundary core certification rows.
- Implement and document bridge version 2's monomorphized structural call
  contract.
- Implement safe structural `Construct[T]`.
- Implement allocation-free structural projection/visitation.
- Implement typed callback adapter generation.
- Prove ownership, move, borrow, error and panic behavior.
- Extend non-Pydantic compiler conformance fixtures.

Exit gate: a fixture package round-trips nested generic/recursive values through
a native opaque resource without dynamic reflection, layout assumptions, or
untyped callbacks.

### milestone_ps_3: Static Program and Native Bridge Contract

- Implement deterministic static schema-program emission support.
- Implement sealed arena/document opaque resources and compact node indices.
- Merge bridge version 2 into `internal_docs/rust_interop_architecture.md`.
- Add generic signature probes, installed/source parity, cleanup,
  bridge-version, and cache-key contracts.
- Prove exact integer, bytes, collection and error crossings.
- Update Rust interop architecture and verification.

Exit gate: the merged and certified bridge-version 2 contract lets a synthetic
schema executor return a validated arena, construct a typed Sifr value, and
pull a structural view for output through one monomorphized call.

### milestone_ps_4: Companion Repository and Core Foundation

- Require the released Sifr compiler/sysroot containing certified `ps_1`
  through `ps_3` contracts.
- Create `sifr-lang/pydantic-sifr`.
- Establish Sifr package and Rust backend layouts.
- Define Core Schema/program format version 1 and its build-time verifier.
- Add error, input, arena and plan foundations.
- Integrate Python-free `jiter`.
- Establish licenses, provenance, fuzzing and benchmark harnesses.

Exit gate: malformed schemas and malformed JSON return stable typed errors with
zero panics under unit, property and fuzz tests.

### milestone_ps_5: Scalar and Collection Validation

- Implement scalar schema nodes and strict/lax conversion.
- Implement exact/fixed integers, floats, decimals, strings and bytes.
- Integrate temporal and focused scalar libraries.
- Implement constraints, lists, tuples, mappings and sets.
- Port the corresponding neutral Pydantic Core corpus.

Exit gate: the classified scalar/collection compatibility corpus passes, all
intentional differences are recorded, and resource limits are enforced.

### milestone_ps_6: Models, Fields, Defaults and Aliases

- Implement model/record schemas.
- Implement required/defaulted/nullable distinctions.
- Implement field metadata, aliases and alias paths.
- Implement extra-field policies and ephemeral validated-field-count tracking.
- Implement typed construction into ordinary Sifr classes.
- Expose the first complete `BaseModel` validation API.

Exit gate: nested models validate JSON and native structural inputs into typed
Sifr values with aggregate stable errors and no third arena-to-model bridge
tree.

### milestone_ps_7: Unions, Recursion and Custom Validation

- Implement literals, enums, ordinary unions and tagged unions.
- Implement deterministic smart-union ranking.
- Implement definitions, references and recursion guards.
- Implement before/after/wrap/plain typed validators.
- Implement field/model validator ordering and context.
- Port the corresponding upstream behavior corpus.

Exit gate: ambiguous, recursive and callback-heavy cases have deterministic
success/error behavior, bounded execution and complete ownership coverage.

### milestone_ps_8: Serialization

- Implement serializer plans over structural projections.
- Implement structural and streaming JSON outputs.
- Implement aliases, include/exclude, unset/default/none policies.
- Implement custom field/model serializers and computed fields.
- Preserve exact numeric and temporal output policies.
- Port serialization tests and benchmarks.

Exit gate: mutated typed models serialize from their current values, not a
retained validation arena, and no full generic output tree is required for JSON.

### milestone_ps_9: TypeAdapter and JSON Schema

- Implement reusable `TypeAdapter[T]`.
- Implement validation and serialization modes.
- Generate JSON Schema from the same Core Schema.
- Support definitions, recursion, aliases, constraints and mode-specific
  representations.
- Add deterministic schema snapshots and dialect conformance.

Exit gate: validation, serialization and description agree for every supported
schema node, with no Schemars or alternate metadata authority.

### milestone_ps_10: Full Pydantic-Familiar Surface

- Complete the selected `BaseModel`, `Field`, configuration, validator,
  serializer, computed-field and adapter APIs.
- Publish the API/behavior compatibility matrix.
- Add migration documentation for Pydantic users.
- Prove ordinary Sifr classes and the familiar facade use the same engine.
- Remove any temporary internal API exposed during construction.

Exit gate: the documented end-state public API is complete; no public fallback,
temporary schema form or second validation path remains.

### milestone_ps_11: Certification and Release

- Complete the portable upstream compatibility inventory.
- Run differential validation against the pinned oracle.
- Complete fuzz, property, adversarial resource and panic testing.
- Publish parse/validate/construct/serialize benchmarks.
- Certify supported compiler/core/package version combinations.
- Add end-to-end demos and package documentation.
- Perform independent whole-architecture and implementation review.

Exit gate: all acceptance criteria pass using released Sifr and
`pydantic-sifr` artifacts without access to the source checkout, Python, or the
upstream repositories.

## Acceptance Criteria

### Architecture

- `sifr-lang/sifr` contains no Pydantic-specific compiler branch, type, schema
  node, decorator name or JSON validation policy.
- `sifr-lang/pydantic-sifr` contains the Sifr package and native core as
  separately owned components with one versioned schema contract.
- Validation, serialization and JSON Schema generation consume one Core Schema
  authority.
- Static schemas are verified and emitted during build.
- There is no runtime schema compiler or alternate dynamic adapter path.
- Bridge version 2 is a merged, certified general structural contract with a
  non-Pydantic conformance consumer.

### Native execution

- Published artifacts contain no Pydantic, Pydantic Core, PyO3, CPython, GIL or
  dynamic extension dependency.
- JSON input uses Python-free `jiter`.
- Exact Sifr integers survive parse, validation, construction and serialization.
- Successful validation constructs the requested native Sifr type.
- Serialization observes the current typed value after mutation.
- Construction and serialization each use one monomorphized structural native
  call; the core never imports generated package bridge types.
- User-controlled data and callbacks cannot produce an uncaught Rust panic.

### Behavior

- Required Pydantic-equivalent features have neutral fixtures and provenance.
- Every relevant upstream case is classified as same, adapted, not applicable
  or rejected.
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
- The final demo builds and runs against installed/released artifacts.
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
