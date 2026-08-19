# Native Pydantic-Sifr Architecture

Status: active design contract.

## Document Authority

This document owns the durable Native Pydantic-Sifr design and architecture.
Delivery records preserve implementation history but do not define the current
compiler or package contract.

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
- Pydantic-style model declarations without raw specialization metadata,
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

Existing Sifr contracts: [`rust_interop_architecture.md`](./rust_interop_architecture.md),
[`sifr_sysroot_and_stdlib_architecture.md`](./sifr_sysroot_and_stdlib_architecture.md),
[`integer_model.md`](./integer_model.md), and
[`architecture.md`](./architecture.md).

The compiler substrate also depends on the core rows tracked by
[`rust-interop-runtime-ecosystem-certification.md`](../plans/issues/archive/rust-interop-runtime-ecosystem-certification.md).

### Cross-document authority

This document supersedes the design in
[`41_typed_data_model_and_validation.md`](../plans/phases/41_typed_data_model_and_validation.md).
That file remains a redirect and history note. Downstream work consumes the
released `pydantic-sifr` public model and error contract, not a separate in-compiler
validator. It must not add a fallback contract.

This design does not supersede `internal_docs/integer_model.md` or its locked
serialization-boundary artifact: all integer JSON, schema, and diagnostic
behavior defers to them. The const-specialization contract owns
`SIFR-INT-0009`, its error page, and package-neutral boundary verification. The
schema consumer integrates with that locked boundary artifact; it does not
redefine the code. The
former `Serialize`/`Deserialize` and stdlib `dumps`/`loads` proposal is
intentionally subsumed by the one `TypeAdapter[T]`/`BaseModel` Core Schema
path, not retained as a second compiler or stdlib serialization authority.
`sifr-lang/sifr` deliberately keeps only its general `JsonValue` JSON API;
typed model JSON is owned exclusively by the external package. Downstream
consumers wait for a certified external release rather than adding a fallback.
Rust bridge certification rows remain owned by
`rust-interop-runtime-ecosystem-certification.md`; this architecture consumes
only passing rows or an explicitly transferred narrow row recorded in that
issue and the compatibility matrix.

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
22. Sifr provides one package-neutral static class-adapter contract. The
    contract supports declaration descriptors, adapter markers, typed handler
    references, and attached package APIs.
23. A class adapter cannot rewrite arbitrary syntax, add stored fields, change
    field types, rewrite method bodies, or change ownership and layout.
24. `BaseModel`, `Field`, `ConfigDict`, validators, serializers, computed
    fields, and model method names remain package-owned declarations.
25. The compiler resolves descriptors before it determines field requiredness,
    defaults, constructor parameters, and the structural shape.
26. Attached model APIs bind existing package functions to a concrete owner
    type. The compiler does not generate arbitrary package method bodies.
27. Source origins participate in diagnostics. They do not participate in
    static schema identity or semantic cache identity.
28. The ergonomic API replaces the package's raw metadata syntax as the normal
    public path. It does not create a second schema or execution engine.

## Repository Ownership

This repository boundary is part of the architecture, not merely source-tree
organization. `sifr-lang/sifr` supplies and releases the general compiler,
sysroot, package, and native-interop capabilities first. The resulting
`pydantic` Sifr package is then developed, tested, reviewed, released, and
consumed as an external package from its own
[`sifr-lang/pydantic-sifr`](https://github.com/sifr-lang/pydantic-sifr) GitHub
repository.

Package and native-core implementation is owned by that external repository.
The Sifr compiler repository records only the package-neutral contracts that
it owns; it does not import the product source.

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
spanned class declaration
          |
          v
resolve marker, field, class, type, and method descriptors
          |
          v
package-owned static class adapter
          |
          v
finalized Sifr type T + typed package metadata
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

## Static Class-Adapter Contract

### Purpose and limit

The compiler provides one static class-adapter contract for packages that add
declarative behavior to ordinary classes. Database mappers, RPC packages, and
command-line parsers can use the same contract.

The contract is not a macro system. An adapter receives typed declarations and
returns a bounded declaration plan. It cannot receive raw source text or an
untyped syntax tree.

An adapter can:

- consume typed field, class, type, and method descriptors,
- attach typed metadata to existing declarations,
- select a required, constant-default, or factory-default field state,
- request one existing package const specialization,
- reference statically checked user methods as handlers,
- attach declared package functions as type or instance methods, and
- return bounded package issues at supplied source origins.

An adapter cannot:

- add, remove, rename, or retype stored fields,
- rewrite expressions or method bodies,
- change class layout, ownership, or sendability,
- introduce runtime reflection,
- generate arbitrary HIR or Rust code, or
- select behavior from an unresolved package or decorator basename.

### Package-author declaration syntax

The compiler-owned package-author surface uses declaration decorators. These
decorators are compile-time declarations, have no runtime value, and resolve
every referenced module and symbol by canonical imported identity. String
arguments below identify a module and a symbol separately, as they do for
`@const_specialize("package.module", "function")`; they are not late-bound
qualified-name strings.

An adapter provider is a const function whose decorator binds the provider to
one closed structural descriptor type `D`:

```sifr
@class_adapter_provider("fixture.contract_types", "ContractDescriptor")
@const_eval
def adapt_contract(
    declaration: DeclarationInput[ContractDescriptor],
) -> DeclarationPlan[ContractDescriptor]:
    ...
```

`@class_adapter_provider("module", "descriptor_type")` is valid only on a
const-evaluable function with the exact generic input and output relationship
above. The strings resolve the declared `D`; the annotations must resolve to
that same type. The decorated function's canonical identity is the provider
identity. `D` can be a record or a closed union of records, but it cannot be
`Any`, contain runtime resources, or depend on an unresolved type variable.
Declaring the provider and `D` does not select that provider for any class.

Descriptor functions name their provider with the same two-string module and
symbol form:

```sifr
@field_descriptor("fixture.contract", "adapt_contract")
def option(...) -> ContractDescriptor:
    ...

@class_descriptor("fixture.contract", "adapt_contract")
def contract_config(...) -> ContractDescriptor:
    ...

@method_descriptor("fixture.contract", "adapt_contract")
def before_call(...) -> ContractDescriptor:
    ...

@type_descriptor("fixture.contract", "adapt_contract")
def bounded(...) -> ContractDescriptor:
    ...
```

Each descriptor decorator implies bounded const evaluation and is valid only
when every return member is assignable to the provider's declared `D`. The
decorator kind fixes the declaration locations where calls are accepted. The
compiler rejects a provider mismatch before invoking either descriptor or
adapter code.

A field-less marker class selects the provider explicitly:

```sifr
@class_adapter_marker("fixture.contract", "adapt_contract")
class Contract:
    pass
```

`@class_adapter_marker` is the only declaration that selects a provider for a
class hierarchy. The provider function, its `D`, and the marker are separate
canonical declarations, so descriptor collection can be implemented and
checked before marker-base selection is enabled.

One consumed, field-less declaration fixes an attached-API set identity, and
package functions enter that set through `@attached_api`:

```sifr
@attached_api_set
class ContractApi:
    pass

@attached_api(
    "fixture.api",
    "ContractApi",
    public_name="decode",
    receiver="type",
    owner="T",
)
def decode[T: StaticProgram, Input: Structural](
    input: Input,
) -> Result[T, DecodeError]:
    ...

@attached_api(
    "fixture.api",
    "ContractApi",
    public_name="encode",
    receiver="immutable",
    owner="T",
)
def encode[T: StaticProgram](target: T) -> Result[bytes, EncodeError]:
    ...
```

`@attached_api_set` consumes the class declaration as a compile-time namespace;
the class has no runtime value or storage. The two leading strings on
`@attached_api` resolve that set's canonical module and symbol. The adapter
plan selects the same identity. `public_name` is a literal identifier.
`receiver` is exactly `"type"`, `"immutable"`, `"mutable"`, or `"owned"`.
`owner` names one declared type parameter: a type receiver substitutes it
without passing a dummy value; an instance receiver requires the first
function parameter to use that owner type with the matching default-borrow,
`mut`, or `own` convention. Remaining type parameters stay available for
normal call-site inference. Omitted public arguments use the attached package
function's checked defaults. An instance binding maps those defaults after it
removes the hidden owner parameter. A concrete generic alias such as
`TypeAdapter[Model] = Model` forwards a type call to `Model`; it does not own a
separate static program.

A method descriptor call on a user method produces a sealed method-declaration
identity in `ClassDeclaration[D]`. An adapter can return only that identity in
a handler reference; it cannot construct a target from a method-name string.
The compiler resolves `@classmethod`, `@staticmethod`, and receiver form before
it evaluates the package method descriptor. This is the complete
package-author syntax: normal model authors use the package's descriptor
functions and marker base, not these compiler declaration decorators.

### Marker bases

A package can mark a field-less class as a class-adapter marker. The compiler
resolves the marker through its canonical imported identity.

An adapter marker is not a data parent. It does not add layout, constructor
work, or a `super()` requirement. It also does not consume the one ordinary
data-parent position.

A class can select one adapter provider. Two different providers on one class
are a compile-time error. Repeated references to the same inherited provider
are normalized to one provider.

An adapted class can have one ordinary data parent. If that parent is adapted,
it must use the same provider. The adapter receives the complete inherited
declaration order and exact nominal identities.

### Spanned declaration input

The compiler collects an adapted class before normal class finalization. The
input contains:

- the exact class and provider identities,
- type parameters and the optional data parent,
- fields in declaration order,
- field annotations and right-hand-side expressions,
- class-level descriptor assignments,
- methods and their checked signatures,
- descriptor values and callback identities, and
- source-origin IDs for declarations and descriptor arguments.

The input contains typed values only. A source-origin ID is an opaque compiler
token. Package code can return that token with an issue but cannot create or
alter a source range.

Source-origin IDs are diagnostic data. Canonical shape, schema, program, and
incremental cache identities exclude source locations.

### Declaration descriptors

A package can mark a const function as one descriptor kind:

- field descriptor,
- class descriptor,
- method descriptor, or
- type-annotation descriptor.

The compiler recognizes descriptor calls through canonical imported function
identity. It evaluates each call with the normal bounded const evaluator.

A field descriptor is valid on an annotated field right-hand side. A class
descriptor is valid in a consumed class assignment. A consumed assignment does
not create a stored field or runtime class value.

A method descriptor is valid on a checked method declaration. A type descriptor
is valid in `Annotated[T, descriptor]`. The compiler preserves declaration
order and descriptor source origins.

Descriptor functions return package-owned structural values. The compiler does
not define names such as `gt`, `alias`, `extra`, or `strict`.

Each adapter provider declares one structural descriptor type `D`. Its
declaration input is generic over that type:

```text
DeclarationInput[D]
DescriptorUse[D]
    target_kind
    target_identity
    value: D
    origin
```

All descriptor functions consumed by that provider must return `D` or a union
member assignable to `D`. Pydantic-Sifr can use one union of field,
configuration, handler, and type descriptor records.

The compiler retains each evaluated value as a typed const value. It does not
convert values to strings or expose one untyped metadata map. A descriptor from
another provider produces a type error before adapter evaluation.

The structural shape and specialization input use the same provider type `D`:

```text
ShapeMetadata[D]
    key
    value: D

ShapeInput[D]
    root
```

The compiler binds `D` from the selected adapter provider. It converts each
adapter-produced metadata value to that exact static type before it invokes the
provider's specialization function. The package does not declare one `str`
field that erases all metadata value types.

Descriptor values can contain a compiler-owned `CallableIdentity`. This sealed
const variant names a checked function, static method, or supported type
constructor. Its canonical module, owner, symbol, generic arguments, and
signature digest participate in const and static-program identity.

Adding `CallableIdentity` changes the closed `ConstValue` and
`StaticProgramValue` contracts. The implementation must update
`internal_docs/const_specialization.md` and its cache-identity evidence in the
same merge unit.

### Finalization order

The compiler uses this order for an adapted class:

1. Resolve imports, bases, annotations, and declared method signatures.
2. Collect descriptor calls and source origins.
3. Evaluate descriptor functions.
4. Run the selected adapter with the typed declaration input.
5. Validate the bounded declaration plan.
6. Determine field defaults, requiredness, and constructor parameters.
7. Build the finalized nominal type and structural shape.
8. Run the requested package const specialization.
9. Resolve handler slots against user-authored methods.
10. Attach declared package API bindings.

No later step changes an input of an earlier semantic identity. Attached APIs
do not enter the structural shape that drives schema specialization.

### Declaration plan

The adapter returns one bounded declaration plan. The plan can contain:

- typed metadata for the class and its existing fields,
- one default state for each field,
- typed references to user-authored handler methods,
- one specialization function identity, and
- one declared attached-API set identity.

A field default state is exactly one of:

```text
Required
ConstDefault(value, validation_policy)
FactoryDefault(callable_identity, validation_policy)
```

The callable identity can name a checked function, static method, or supported
type constructor. The compiler checks its parameters and output against the
field contract. The package schema remains the authority for default
validation policy.

The plan cannot contain generated stored fields or generated method bodies.
This rule prevents a transform output from changing its own structural input
or schema cache key.

### Handler references

Method descriptors produce typed references to methods in the declaration
input. Before specialization, the compiler binds each reference to a declared
method and checks its static signature.

The package specialization selects the handler references that its static
program uses. After specialization, the compiler resolves that selected set to
the emitted method-slot table. The selected set contributes to the static
program output and code-generation identity. It does not feed the earlier
adapter-invocation key.

The compiler checks:

- the exact owner and method identity,
- the receiver and ownership mode,
- input and output types,
- an infallible output or a typed `Result` output,
- synchronous execution,
- context type and borrow mode,
- duplicate targets, and
- constructor and attached-API exclusions.

An ordinary class supports `Self` in method annotations. `Self` means the exact
current class specialization. A declared `own self` receiver uses the ordinary
owned parameter convention and can return `Self` or `Result[Self, E]`.

The compiler resolves built-in method-kind decorators before package method
descriptors. For the familiar validator form, the package descriptor is the
outer decorator and `@classmethod` is directly above the method declaration.

The package owns handler kinds and ordering. The compiler preserves declaration
and inheritance order and emits a checked method-slot table.

A field `before` handler declares an input type `I` independently from the
field type. The active input profile validates the source into `I`. The handler
then returns a typed value that the remaining field schema validates. The
pipeline does not enter the same `before` stage again.

A model `before` handler declares one concrete structural input type `I`. The
model input must project to `I` under the active input profile. The handler
returns a concrete structural value accepted by the remaining model field
schema. The pipeline does not enter the same `before` stage again. Neither form
uses `Any` or an untyped input map.

An `after` model handler runs in generated package glue after
`StructuralConstruct` creates the model. Each handler consumes the current
model and its returned `Self` becomes the input to the next handler. The final
returned value is the validation result.

The generated glue keeps this work inside the same fallible
validate-and-construct operation. A typed handler error becomes a validation
detail at the model root and joins the same aggregate `ValidationError`.
Cleanup consumes any constructed or replaced model exactly once after an
error.

Attached package methods cannot become handler targets. This rule keeps the
schema input independent from the attached facade.

### Attached package APIs

A package can declare a fixed set of generic functions as an attached API. A
binding selects a public name and one receiver form:

- type receiver,
- immutable instance receiver,
- mutable instance receiver, or
- owned instance receiver.

The compiler substitutes the adapted owner for the declared owner type or
`Self`. An attached function can retain other type parameters. Normal call-site
inference resolves those parameters, and their concrete arguments participate
in code-generation identity. The compiler applies the declared checked defaults
to omitted arguments. A concrete generic alias can forward the call surface to
its resolved adapted owner. The compiler then applies normal generic,
ownership, effect, and `Result` checks.

The binding refers to an existing package function. The compiler does not copy
or synthesize its body. A collision with a user method or another attached API
is a compile-time error.

Inherited classes reuse bindings from the same provider. The concrete owner
type selects the concrete static program. Attached API signatures participate
in module and code-generation cache identities but not structural shape
identity.

### `Annotated` normalization

`Annotated` is the package-neutral carrier for type descriptors. The compiler
preserves the base type and ordered descriptor list.

The adapter defines descriptor merge rules. Pydantic-Sifr uses these rules:

- Nested `Annotated` layers flatten from inner to outer order.
- Independent constraints compose.
- A repeated descriptor property uses the last explicit value.
- Two incompatible descriptor properties produce a package issue.
- A right-hand-side `Field` descriptor applies after annotation descriptors.
- Field defaults come only from the right-hand-side field descriptor or the
  ordinary field default.
- `Annotated[T, Field(...)] | None` constrains `T` only.
- `Annotated[T | None, Field(...)]` applies metadata to the complete optional
  field contract.

### Inheritance

The compiler preserves inherited and local declaration identity. The package
adapter owns package-specific merge policy.

Pydantic-Sifr uses these rules:

- Base fields precede local fields.
- A local field can override an inherited field only with the same type.
- Local field descriptors replace conflicting inherited field properties.
- Model configuration merges from the oldest base to the most-derived class.
- A derived explicit configuration value wins.
- Handler order follows the documented Pydantic-Sifr order for each handler
  mode.
- A derived declaration with the same handler identity replaces the inherited
  declaration.
- The data parent can be a concretely instantiated generic class that uses the
  same adapter provider.
- Schema generation requires concrete type arguments.

### Identity and bounded evaluation

The adapter-invocation cache input contains:

- the provider package, function, and version identity,
- the exact adapted class and parent identities,
- resolved field and method signatures,
- normalized descriptor values,
- the structural and const-evaluator contract versions.

The post-adapter program key is a content identity computed from completed
adapter and specialization results. It contains the adapter-invocation key
plus:

- normalized adapter output,
- selected handler identities, and
- the selected attached-API set identity.

The module and code-generation keys also contain resolved attached signatures
and concrete residual generic arguments. No post-adapter output feeds the
adapter-invocation key. The post-adapter identity never performs the lookup for
the specialization that produces its inputs.

Source ranges, file paths, and diagnostic rendering do not enter semantic
identity. Descriptor and adapter evaluation use explicit recursion, step,
allocation, output-size, and issue-count limits.

## Compiler Substrate

### Compiler prerequisites

The architecture depends on compiler and sysroot capabilities that are broader
than small extensions to generics, stdlib value types, or Rust interop:

- compile-time specialization of package generics for a concrete `T`,
- package-neutral static class adapters and erased marker bases,
- typed field, class, type, and method descriptors,
- source-origin IDs for declaration and descriptor diagnostics,
- checked package API attachment with owner-type substitution,
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
scalar values directly cross the boundary as one-off bridge types.

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

[`rust_interop_architecture.md`](./rust_interop_architecture.md) owns this
contract, and its certification rows must pass. Pydantic-Sifr cannot privately
invent an alternate structural bridge.

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

Raw declaration metadata is a low-level package-authoring mechanism. Static
class adapters produce this metadata from typed descriptors before structural
shape finalization.

This mechanism supports Pydantic-familiar `Field`, `field_validator`,
`model_validator`, `field_serializer`, `computed_field`, and configuration
declarations without compiler knowledge of these names.

### Const-specialization diagnostics

A specializing package function returns
`ConstSpecializationOutcome[T]`, containing either a value plus zero or more
warnings, or one or more fatal `ConstPackageIssue` values and no value. These
are new frontend contracts; they neither reuse nor alter
`sifr_package::PackageDiagnostic` or the driver's existing `CompileResult`.
Those existing types remain confined to package-manager and driver
diagnostics.

`ConstPackageIssue` carries a package-qualified stable `reason_code`, static
package-template arguments, one primary source-origin ID, additional labels,
and notes. Values must be
const-evaluable and bounded. Package argument names are checked against the
package's statically declared template and cannot use compiler/LSP-reserved
names such as `rule`.
The package reason is diagnostic context, not a new top-level compiler code.
The frontend maps a fatal issue to built-in `SIFR-META-0001`, a warning to
`SIFR-META-0002`, and a malformed declaration to `SIFR-META-0003`. These three
general metaprogramming codes belong to the closed Sifr diagnostic registry, so
normal documentation URLs remain
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
lint rule; this classification is documented with the diagnostic code.

The frontend converts the outcome into the same structured CLI/LSP diagnostic
stream in `check`, build, tests, and editor analysis. It validates the package
namespace, reason code, origins, and template arguments and never executes a
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

The public surface is familiar to Pydantic users and preserves Sifr's static
and fallible semantics.

Representative shape:

```sifr
from pydantic_sifr import (
    BaseModel,
    ConfigDict,
    Field,
    ValidationError,
    computed_field,
    field_serializer,
    field_validator,
    model_validator,
)

class User(BaseModel):
    model_config = ConfigDict(
        extra="forbid",
        strict=True,
        populate_by_name=True,
    )

    id: int64 = Field(alias="user_id", gt=0)
    name: str = Field(min_length=1)
    active: bool = True
    tags: list[str] = Field(default_factory=list)

    @field_validator("name", mode="before")
    @classmethod
    def normalize_name(cls, value: str) -> Result[str, ValueError]:
        ...

    @model_validator(mode="after")
    def validate_user(own self) -> Result[Self, ValueError]:
        ...

    @field_serializer("id")
    def serialize_id(self, value: int64) -> str:
        return str(value)

    @computed_field
    def display_name(self) -> str:
        return self.name

def load_user(payload: bytes) -> Result[User, ValidationError]:
    return User.model_validate_json(payload)
```

The example uses the `json.exact` integer profile. The phase record uses the
same canonical example.

Sifr does not turn validation failures into exceptions. Familiar operations
therefore return `Result` where user input or custom behavior can fail.

The complete selected public surface includes:

- `BaseModel`, `RootModel[T]`, and concrete generic models,
- `Field`, `ConfigDict`, and `Annotated` declarations,
- required fields, constant defaults, and typed default factories,
- explicit aliases, alias paths, alias choices, and separate validation or
  serialization aliases,
- deterministic const alias generators,
- numeric, text, bytes, collection, pattern, and discriminator constraints,
- field schema annotations such as title, description, examples, and bounded
  `json_schema_extra`,
- nested models, recursive models, enums, literals, optionals, and unions,
- public URL, multi-host URL, and compiled-pattern values,
- field validators in `before`, `after`, and `plain` modes,
- field validators with one or more explicit field targets,
- model validators in `before` and `after` modes,
- field and model serializers with typed values and results,
- serializer `when_used` policies,
- computed fields from checked zero-argument instance methods,
- structural, JSON, and strings-profile validation,
- structural and JSON serialization,
- typed include and exclude selections,
- unconditional field serialization exclusion,
- `exclude_defaults`, `exclude_none`, and alias output,
- `TypeAdapter[T]`, and
- JSON Schema in validation and serialization modes.

`RootModel[T]` is a package-owned generic adapted class with one stored
`root: T` field. The adapter does not synthesize this field. Static schema
specialization occurs for a concrete `RootModel[Concrete]` type.

A user generic model keeps its declared type variables during checking. The
compiler instantiates the adapter plan and static program only for a concrete
use. An unbound generic model has no runtime schema program.

The common model methods are:

- `Model.model_validate`,
- `Model.model_validate_json`,
- `Model.model_validate_strings`,
- `model.model_dump`,
- `model.model_dump_json`, and
- `Model.model_json_schema`.

Each fallible method returns `Result`. `Model.model_json_schema` is
type-directed and does not require a dummy model value.

`model_dump` always returns `dict[str, JsonValue]`. A separate typed adapter
entry point can project a value to another declared structural output type.
That typed projection does not change the `model_dump` contract.

The static configuration surface covers the common model policies:

- extra-field policy,
- strict or coercing validation,
- validation by alias or field name,
- serialization aliases,
- default validation,
- enum-value representation,
- common text normalization, and
- deterministic alias generation.

`extra="allow"` requires a declared typed extra-field destination. The
compiler does not add hidden storage to a model.

Pydantic-style methods and a smaller functional API may coexist only as thin
views over the same Core Schema and execution engine. There is no second
functional validator implementation underneath convenience functions.

The package does not expose raw `@metadata` and `@const_specialize` declarations
as the normal model authoring path. These declarations remain compiler
substrate for package authors and conformance fixtures.

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
- callback scheduling before structural construction,
- aggregate errors,
- validated value storage,
- serializer-plan execution,
- JSON writing, and
- source positions needed by diagnostics.

The Sifr package owns the one semantic Core Schema canonicalizer and verifier,
implemented as deterministic const-evaluable Sifr code. The native core never
implements a second verifier; its envelope checks protect artifact integrity,
not schema semantics.

Generated package glue schedules `after` model handlers after structural
construction. This is the only validation handler stage that requires a typed
model value.

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
`str`; nested records, mappings, and sequences are allowed. The package uses
the compiler-owned `sifr.meta.StringStructural` bound for this check. The bound
is a compile-time subset of `Structural`. It emits the same structural Rust
projection contract and retains the owned-value bounds needed by generated
function bodies. It adds no runtime trait or value tree. A bare `str` uses the
normal scalar projection. The compiler-generated projection for `S` is
therefore the input type—there is no `Any` or package-owned recursive value
tree. Rust-opaque package values do not satisfy this bound because the compiler
cannot inspect their mapped leaf types. The profile reuses the native structural adapter with a leaf-kind
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
diagnostic `SIFR-INT-0009`; the integer-boundary contract owns its registry
entry, documentation, and CLI/LSP tests. Pydantic-Sifr supplies data to this
general verifier but neither owns nor emits the top-level code.

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
   its manifest disposition and evidence owner are reviewed;
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

### Terminal ergonomic exclusions

The following exclusions are final architecture decisions, not deferred
implementation dependencies:

| Excluded feature | Reason |
| --- | --- |
| Python metaclasses or runtime class mutation | Static class and schema finalization cannot remain sound if runtime code changes checked declarations or layout. |
| Dynamic `create_model` | Runtime type creation has no concrete compile-time owner or sealed schema-program identity. |
| Arbitrary syntax-tree macros | Packages receive typed declarations and return a bounded plan so they cannot redefine language semantics. |
| Runtime schema construction | Build-time package canonicalization and one sealed program are the sole schema authority. |
| Python plugins or custom Core Schema hooks | Published artifacts contain no Python runtime, and open runtime hooks would bypass static schema verification. |
| Pydantic dataclasses | Dataclass discovery and construction form a separate authoring model from the selected adapted-class contract. |
| Private attributes | Adapters cannot add hidden storage outside the declared structural layout. |
| `validate_call` | General function-call interception requires a function-adapter contract that this class-adapter architecture does not introduce. |
| `model_construct` | Bypassing validation conflicts with the sealed validate-and-construct boundary. |
| Dynamic `model_copy` updates | Dynamic field updates conflict with static typing and ownership; ordinary Sifr construction or explicit cloning is the replacement. |
| Runtime `model_fields` and `model_rebuild` | Runtime reflection and schema rebuilding conflict with immutable compile-time shapes and programs. |
| ORM `from_attributes` | Arbitrary attribute probing is outside the typed structural-input contract. |
| Arbitrary runtime types | Every public value requires a statically checked structural or declared nominal mapping. |
| Multiple data inheritance | One data parent preserves deterministic layout, constructor synthesis, and inherited field identity. |
| Mixed class-adapter providers | Multiple providers would create ambiguous plan, ordering, and cache authorities. |
| Assignment-validation interception | Intercepting ordinary mutation would change Sifr assignment and ownership semantics rather than adapt a declaration. |
| Python-compatible frozen-model emulation | Sifr's ordinary immutability and ownership rules are the static replacement. |
| Public wrap-handler continuations | Exposing a continuation requires a new ownership, lifetime, and effect contract; engine-internal wrap nodes remain non-public. |
| Wildcard `field_validator("*")` targeting | Explicit field identities preserve static target checking, diagnostics, inheritance, and deterministic ordering. |
| Schema generation for an unbound generic model | Program identity and field shapes exist only after concrete owner-type substitution. |

No current implementation depends on these features. A separate proposal would
need its own architecture rather than treating one as unfinished work here.

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
- Once a performance baseline is accepted, unexplained material regressions
  block the next release.
- Rust modules remain responsibility-oriented and below the repository's file
  size guardrail.
- Every schema node has one primary implementation owner, one specification
  table, and one or more focused supporting test families.

## Non-Goals

- Exact source or binary compatibility with Python Pydantic.
- A Python runtime, PyO3 extension, or Python object bridge.
- Supporting Pydantic plugins by executing Python.
- Python metaclasses, runtime class mutation, or dynamic model creation.
- Arbitrary syntax-tree transforms or package-generated method bodies.
- Reusing Pydantic's Python-specific Core Schema nodes.
- Making arbitrary Sifr values dynamically introspectable at runtime.
- Making Core Schema the normal beginner-facing API.
- Runtime model/schema construction or a runtime schema compiler.
- `create_model`, `validate_call`, Pydantic dataclasses, or private attributes.
- `model_construct`, dynamic `model_copy` updates, runtime `model_fields`, or
  runtime `model_rebuild`.
- Python `from_attributes`, ORM attribute probing, or arbitrary runtime types.
- Multiple data inheritance or composition of different class-adapter providers.
- Public wrap-validator and wrap-serializer continuation APIs.
- Wildcard `field_validator("*")` targeting. Validators name one or more
  explicit fields.
- Python `@property` stacking for computed fields. A computed field is a checked
  zero-argument instance method.
- Open generic schema generation before concrete type specialization.
- Assignment-validation hooks that intercept ordinary Sifr field mutation.
- Python-compatible frozen-model emulation. Programs use ordinary Sifr
  immutability contracts instead.
- Adding JSON-specific rules to the Sifr compiler.
- Replacing Sifr's ordinary type checker with validation schemas.
- Implementing Pydantic Settings, web-framework integration, ORM behavior or
  unrelated ecosystem packages inside the core architecture. Those may be
  separate packages consuming the completed public contract.
- Supporting a temporary reduced public architecture that later requires a
  second validation engine or compatibility fallback.

## Design Invariants

### Architecture

- `sifr-lang/sifr` contains no Pydantic-specific compiler branch, type, schema
  node, decorator name or JSON validation policy.
- Static class adapters use canonical package identities. They never use
  unqualified decorator, class, or function names.
- A static class adapter consumes typed declarations and returns one bounded
  declaration plan. It cannot rewrite arbitrary syntax or stored layout.
- Adapter execution completes before requiredness, constructor, and structural
  shape finalization.
- Attached APIs bind declared package functions. They do not add method bodies
  to the schema input.
- Source-origin IDs support package diagnostics but never change semantic
  program identity.
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
- The structural Rust bridge is a certified general contract with a
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

- Normal Pydantic-Sifr model declarations use `BaseModel`, `Field`,
  `ConfigDict`, `Annotated`, typed handlers, and attached model methods.
- Normal users do not write raw specialization or declaration metadata.
- Field, class, type, and method descriptor errors identify the relevant
  declaration or argument.
- One schema program defines validation, serialization, JSON Schema, defaults,
  aliases, handlers, and attached model operations.
- Required Pydantic-equivalent features have neutral fixtures and provenance.
- Every relevant upstream case is classified as same, adapted, not applicable
  or rejected.
- The manifest's sorted file/node ledger exactly equals the API and in-tree
  Core test roots at the sole Pydantic pin; no upstream path, collected
  selector, or parameter can disappear without failing the audit.
- The generated Core Schema kind ledger exactly equals every pinned
  `CoreSchemaType` and `CoreSchemaFieldType` literal, with one accepted
  primary implementation/disposition owner and either a non-empty
  evidence-family set or an explicit architecture disposition audit per kind.
- Every required fixture family passes before release. Final certification
  re-audits the corpus; it does not defer behavior implementation.
- Strict/lax behavior, union ranking, error ordering and serializer profiles are
  deterministic and documented.
- Validation returns aggregate typed errors with stable codes and locations.
- Intentional Sifr differences are public and tested.

### Maintainability

- Compiler conformance uses at least one non-Pydantic class-adapter fixture.
- The compiler API contains no validation, model, field, alias, constraint, or
  Pydantic error vocabulary.
- No permanent fork of Pydantic or Pydantic Core exists.
- Mature focused Rust dependencies are reused at their natural boundary.
- No schema behavior is implemented independently in both Sifr and Rust.
- No third arena-to-model bridge tree or per-call schema rebuild exists.
- Dependency features, licenses and provenance are audited.
- Fuzzing covers schema verification, JSON input, validation plans and writers.
- Benchmarks and regression gates cover each execution stage.
- Compiler and package conformance tests prevent accidental coupling.
