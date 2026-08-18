# Ad Hoc Phase: Static Class Adapters and Pydantic Ergonomics

## Status

Status: proposed on 2026-08-18. No milestone is complete.

This phase starts after the completed Native Pydantic-Sifr engine phase. It
does not reopen the validated schema engine, structural bridge, or native core.

The completed predecessor is
[`ad-hoc-native-pydantic-sifr-architecture.md`](../archive/ad-hoc-native-pydantic-sifr-architecture.md).

## Design Reference

The durable design is
[`native_pydantic_sifr_architecture.md`](../../../internal_docs/native_pydantic_sifr_architecture.md).

This phase owns milestone order, status, validation evidence, review evidence,
blockers, and closure. The architecture document owns durable behavior and
compiler contracts.

## Objective

Add a package-neutral static class-adapter contract to Sifr. Use that contract
to give Pydantic-Sifr a familiar model declaration and method API.

This phase has one end state. Milestones define dependency order. They do not
define an MVP, reduced release, or optional continuation.

The complete result must support this model shape:

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

The canonical example uses the `json.exact` integer profile.

Normal users must not write `@metadata`, `@const_specialize`, dummy schema
values, or free-function model operations.

## Current Baseline

The completed engine already provides:

- deterministic structural shapes and static programs,
- native validation, serialization, and JSON Schema,
- field constraints, defaults, aliases, unions, and recursion,
- typed structural construction and projection,
- checked user-method slots,
- `TypeAdapter` engine support, and
- one Python-free native core.

The current authoring surface does not provide the target ergonomics:

- A named base is a data parent. `class User(BaseModel)` therefore enters
  ordinary inheritance rules.
- A field right-hand-side call such as `Field(gt=0)` is not a simple default.
- An unannotated `model_config = ConfigDict(...)` class item is unsupported.
- Structural shape fields do not carry argument-level source origins.
- The compiler carries metadata as `ConstValue`, but the package-facing
  `ShapeMetadata.value` delivery type is declared as `str`.
- Method slots can reference only methods already present in the shape.
- `model_json_schema[T](target: T)` requires a value that the bridge ignores.
- Public model operations are free functions.

This phase corrects these declaration and facade gaps. It does not replace the
existing native execution engine.

The main extension points are close to the 900-line source limit on the
2026-08-18 baseline:

- `crates/sifr_frontend/src/specialization_runner.rs`: 891 lines,
- `crates/sifr_frontend/src/structural_shape.rs`: 844 lines, and
- `pydantic-sifr/src/schema_contract.sifr`: 810 lines.

The first milestone that changes each file must split new responsibility into
a focused module. Do not split a file by alphabet or line count.

## Complete Scope

### Generic Sifr support

The compiler work includes:

- opaque source-origin IDs for class declarations and descriptor arguments,
- typed field, class, method, and type-annotation descriptors,
- erased class-adapter marker bases,
- adapter execution before class finalization,
- required, constant-default, and factory-default field states,
- one bounded declaration plan,
- checked handler references to user-authored methods,
- checked type and instance API attachment,
- owner-type and `Self` substitution,
- adapter-aware single inheritance,
- deterministic identity and cache invalidation, and
- non-Pydantic conformance fixtures.

### Pydantic-Sifr support

The package work includes:

- `BaseModel`, `RootModel[T]`, and concrete generic models,
- `Field`, `ConfigDict`, and `Annotated`,
- required fields, constant defaults, and `default_factory`,
- explicit aliases, alias paths, alias choices, and separate validation or
  serialization aliases,
- deterministic alias generators,
- common numeric, text, bytes, collection, pattern, and discriminator
  constraints,
- title, description, examples, and bounded `json_schema_extra`,
- optional, union, tagged-union, nested, and recursive model declarations,
- existing enum and literal field types through the same schema path,
- public URL, multi-host URL, and compiled-pattern values,
- field validators in `before`, `after`, and `plain` modes,
- field validators with one or more explicit field targets,
- model validators in `before` and `after` modes,
- field and model serializers,
- computed fields,
- structural, JSON, and strings-profile validation,
- structural and JSON serialization,
- typed include and exclude selections,
- unconditional field serialization exclusion,
- alias, default, and `None` serialization policies,
- structured validation and serialization errors,
- `TypeAdapter[T]`, and
- validation-mode and serialization-mode JSON Schema.

### Deliberate exclusions

This phase does not implement:

- Python metaclasses or runtime class mutation,
- dynamic `create_model`,
- arbitrary syntax-tree macros,
- runtime schema construction,
- Python plugins or custom Core Schema hooks,
- Pydantic dataclasses,
- private attributes,
- `validate_call`,
- `model_construct` and dynamic `model_copy` updates,
- runtime `model_fields` and `model_rebuild`,
- ORM `from_attributes`,
- arbitrary runtime types,
- multiple data inheritance,
- mixed class-adapter providers,
- assignment-validation interception,
- Python-compatible frozen-model emulation,
- public wrap-handler continuations,
- wildcard `field_validator("*")` targeting, and
- schema generation for an unbound generic model.

These exclusions are terminal for this phase. They are not hidden later
milestones.

## Architecture Boundaries

### Compiler ownership

The `sifr-lang/sifr` repository owns only package-neutral mechanisms. Compiler
source, diagnostics, fixtures, and documentation must not use Pydantic field,
constraint, validation, or serializer vocabulary.

At least one synthetic non-Pydantic package must use every generic mechanism.
The fixture can model an RPC contract, row mapper, or command-line schema.

### Package ownership

The `sifr-lang/pydantic-sifr` repository owns:

- every Pydantic public name,
- descriptor value types,
- descriptor merge rules,
- model configuration semantics,
- handler kinds and order,
- schema derivation,
- model method bindings,
- public error types, and
- compatibility fixtures and migration documentation.

### No second authority

The ergonomic facade must produce the existing Core Schema program. It must
use the existing validator, serializer, and JSON Schema engine.

The phase must not add:

- a second schema compiler,
- a second validator or serializer,
- bridge-local configuration defaults,
- a runtime reflection fallback, or
- a raw-metadata compatibility path for normal model declarations.

## Delivery Rules

- Complete milestones in order.
- Complete one milestone item in one session and one worktree.
- Use the phase-closure loop for every milestone item.
- Merge compiler prerequisites before dependent package changes.
- Use one production path when a milestone activates behavior.
- Do not expose temporary public syntax.
- Do not keep old Pydantic authoring syntax as a compatibility facade.
- Keep durable design text out of this phase record.
- Update the design only when a milestone changes architecture.
- Update `internal_docs/const_specialization.md` in the same merge unit when a
  milestone changes a const-value, static-program, package-issue, or method-slot
  contract.
- Record status and evidence here after each merged item.
- Record external blockers in their owning issue.
- Stop the current item when an external blocker prevents completion.

## Existing Dependency Reconciliation

M0 must reconcile this phase with existing issue owners:

- [`sifr-lang/sifr#3233`](https://github.com/sifr-lang/sifr/issues/3233) owns the
  missing structural-identity asset in the installed sysroot.
- [`pydantic-sifr#10`](https://github.com/sifr-lang/pydantic-sifr/issues/10)
  owns modular const-graph and handler-slot prerequisites.
- [`pydantic-sifr#14`](https://github.com/sifr-lang/pydantic-sifr/issues/14)
  owns serializer and computed-field callback support.
- [`pydantic-sifr#27`](https://github.com/sifr-lang/pydantic-sifr/issues/27)
  owns structural mapping gaps for public values.

Transfer an issue into this phase before its milestone changes that issue's
owned code. Close or update the old issue in the same merge unit. Do not keep
two active owners for one change.

The compiler release that contains M1 through M7 must include a self-contained
installed sysroot. M8 cannot merge against a source-checkout-only toolchain.

## Dependency Order

```text
M0 contract lock
  -> M1 spanned declarations
  -> M2 typed descriptors
  -> M3 marker bases and early adapters
  -> M4 defaults, Annotated, inheritance, and identity
  -> M5 handler-bearing method descriptors
  -> M6 structural public values and output mappings
  -> M7 attached package APIs
  -> M8 Pydantic model and field declarations
  -> M9 Pydantic validators
  -> M10 Pydantic serializers and computed fields
  -> M11 complete model operations and schema surface
  -> M12 migration, certification, and closure
```

No package milestone can begin before its compiler dependency is merged and
available through the selected Sifr toolchain.

## Ordered Milestones

### M0: Contract Lock and Coverage Inventory

Owner: `sifr-lang/sifr` design, with a companion package inventory.

Scope:

- Accept the static class-adapter architecture.
- Fix the package-author declaration syntax for adapter markers, descriptor
  functions, adapters, handler references, and attached APIs.
- Define the typed declaration input and bounded declaration plan.
- Define source-origin, identity, inheritance, collision, and budget rules.
- Define the selected Pydantic public surface and terminal exclusions.
- Map each selected public feature to a compiler mechanism and engine owner.
- Inventory current implementation support and existing issue ownership.
- Mark each public operation as an existing path to replace or a net-new path.

Acceptance criteria:

- The architecture contains no Pydantic-specific compiler behavior.
- The adapter cannot rewrite arbitrary syntax, types, layout, or method bodies.
- Every selected Pydantic feature has one implementation owner.
- Every excluded feature has a reason and no milestone dependency.
- The inventory identifies all existing paths to replace and all net-new
  operations.
- An independent architecture review returns `SATISFIED` with no blocking
  finding.

Exit gate: the reviewed contract is sufficient to implement all later
milestones without a new language mechanism.

### M1: Spanned Class Declarations and Package Issues

Owner: `sifr-lang/sifr`.

Scope:

- Add a pre-finalization class declaration representation.
- Separate source-origin and package-issue work from the specialization runner
  before that source file grows.
- Preserve fields, annotations, class items, methods, and declaration order.
- Assign opaque source-origin IDs to declarations and descriptor arguments.
- Extend package issues to select valid source-origin IDs.
- Keep source locations out of structural and static-program identities.
- Make CLI, build, tests, and editor analysis use the same diagnostic path.
- Add a synthetic package fixture for argument-level diagnostics.

Acceptance criteria:

- A package issue can identify one descriptor argument and add related labels.
- A package cannot forge a source range or select an unrelated declaration.
- Moving unchanged source changes diagnostics but not semantic identity.
- Malformed origins produce a compiler-owned diagnostic.
- CLI and LSP diagnostics have the same code, reason, location, and notes.

Exit gate: a non-Pydantic fixture emits precise field, class, and method
declaration diagnostics without changing static-program identity.

### M2: Typed Declaration Descriptors

Owner: `sifr-lang/sifr`.

Scope:

- Add package declarations for field, class, method, and type descriptors.
- Separate declaration-shape and canonicalization responsibilities before the
  structural-shape source file grows.
- Add the provider-to-descriptor-type declaration without selecting a provider
  for a class.
- Require its descriptor functions to return that type or an assignable union
  member.
- Resolve descriptor functions by canonical imported identity.
- Evaluate descriptor calls with the bounded const evaluator.
- Add a sealed `CallableIdentity` variant to `ConstValue` and
  `StaticProgramValue`.
- Include callable module, owner, symbol, generic arguments, and signature in
  canonical identity.
- Make the specializer input generic over the provider's descriptor type with
  `ShapeMetadata[D]` and `ShapeInput[D]`, or an equivalent typed contract.
- Thread the declared `D` through descriptor and specialization contracts. M3
  selects that provider for each adapted class.
- Preserve typed structural results instead of one package-declared scalar
  value type.
- Accept a field descriptor on an annotated field right-hand side.
- Accept a class descriptor in a consumed class assignment.
- Accept method descriptors on checked method declarations.
- Represent ordered type descriptors in `Annotated`.
- Reject descriptor calls in invalid locations.
- Resolve built-in method-kind decorators before package method descriptors.

Acceptance criteria:

- Descriptor values preserve booleans, integers, strings, optionals, records,
  lists, and compiler-checked callable identities.
- A descriptor from another provider fails type checking before adapter
  evaluation.
- A consumed class descriptor does not create runtime storage.
- A field descriptor is not treated as an ordinary field default expression.
- An unannotated class assignment whose value is not a class descriptor keeps
  the ordinary unsupported-class-item diagnostic.
- Import aliases work and same-basename unrelated functions do not match.
- Const-evaluation limits and malformed results fail with stable diagnostics.
- Callable const values round-trip through static-program emission and change
  the cache identity when their exact target changes.
- A non-Pydantic fixture uses all four descriptor kinds.

Exit gate: a package can collect typed declaration intent without raw metadata
decorators or package-name compiler branches.

### M3: Erased Marker Bases and Early Adapter Execution

Owner: `sifr-lang/sifr`.

Scope:

- Add field-less erased class-adapter marker declarations.
- Separate the adapter marker from ordinary data inheritance.
- Permit one adapter provider and one optional data parent.
- Run the selected adapter before field and constructor finalization.
- Validate the bounded declaration plan.
- Apply typed metadata and one specialization request.
- Reject field addition, field removal, type changes, and method-body output.
- Add adapter evaluation budgets and deterministic issue limits.

Acceptance criteria:

- An adapted field-bearing class does not require marker initialization.
- The marker adds no layout, value identity, or constructor parameter.
- Conflicting providers fail at the base declarations.
- The adapter sees exact field, method, parent, and provider identities.
- Adapter output cannot change stored field types or add stored fields.
- One non-Pydantic adapter derives and specializes a class successfully.

Exit gate: an external package can adapt a normal class before the compiler
builds its constructor and structural shape.

### M4: Defaults, `Annotated`, Inheritance, and Identity

Owner: `sifr-lang/sifr`.

Scope:

- Implement `Required`, `ConstDefault`, and `FactoryDefault` field states.
- Build constructor parameters from normalized field states.
- Type-check constant values and factory callable signatures.
- Flatten nested `Annotated` descriptor lists in a stable order.
- Preserve right-hand-side descriptor precedence.
- Support one adapted data parent with the same provider.
- Preserve inherited field and method declaration identity.
- Define separate adapter-invocation and post-adapter program cache inputs.
- Exclude source locations and generated facade bindings from shape identity.

Acceptance criteria:

- `Field()`-shaped descriptors can represent required fields.
- A constant default and a factory default have distinct static identities.
- A factory can reference a checked function, static method, or supported type
  constructor.
- Mutable defaults do not share state between instances.
- Inherited fields precede local fields.
- An incompatible inherited field override is rejected.
- A concretely instantiated generic adapted parent preserves its substituted
  field types and provider identity.
- Relevant adapter edits invalidate the schema program.
- Selected handler and attached-API outputs never feed the adapter-invocation
  key.
- Unrelated source movement does not invalidate the schema program.

Exit gate: an adapter can define complete field requiredness, defaults,
annotation metadata, and single-inheritance input without an identity cycle.

### M5: Handler-Bearing Method Descriptors

Owner: `sifr-lang/sifr`.

Scope:

- Resolve method descriptors against user-authored methods.
- Extend checked method slots with descriptor values and source origins.
- Preserve method declaration and inheritance order.
- Check receivers, ownership, input, output, context, and optional `Result`
  contracts.
- Support static, class, immutable-instance, mutable-instance, and owned
  receivers where the declared signature permits them.
- Add `Self` annotation support for ordinary class methods.
- Add the declared `own self` receiver convention.
- Permit `own self` handlers to return `Self` or `Result[Self, E]`.
- Require the package method descriptor outside `@classmethod` in the familiar
  stacked form.
- Keep attached APIs and constructors out of handler targets.
- Preserve panic containment and callback cleanup.
- Add a non-Pydantic handler pipeline fixture.

Acceptance criteria:

- Invalid target names and signatures identify the method descriptor.
- A handler cannot escape its declared owner or context lifetime.
- Handler identities participate in static-program and code-generation caches.
- Inherited and local handler order is deterministic.
- Infallible and typed fallible handlers both use checked static signatures.
- `Self` resolves to the exact current concrete class specialization.
- A declared owned receiver moves once and cannot be reused after the call.
- Success, typed error, panic, and cleanup cases pass.

Exit gate: a package can describe ordered field and class handler pipelines
without compiler knowledge of handler semantics.

### M6: Structural Public Values and Output Mappings

Owner: `sifr-lang/sifr`.

Scope:

- Complete package-neutral structural mappings for native opaque and
  specialized scalar values.
- Support safe construction and projection of Sifr-visible nominal wrappers.
- Add structural output support for `dict[str, JsonValue]` model dumps.
- Preserve URL, multi-host URL, and compiled-pattern value identity.
- Preserve ownership, cleanup, panic containment, and static shape identity.
- Add a synthetic non-Pydantic mapping and structural-output fixture.
- Transfer or close `pydantic-sifr#27` when its owned substrate merges.

Acceptance criteria:

- A synthetic package constructs and projects one specialized nominal value.
- Structural output can produce `dict[str, JsonValue]` without a JSON byte
  round trip.
- Invalid native payloads return typed errors without partial construction.
- Mapped values preserve exact nominal and structural shape identities.
- Success, error, move, cleanup, and panic cases pass.
- Source and installed toolchains pass the same structural mapping fixtures.

Exit gate: an external package can expose safe specialized public values and
structural model output through package-neutral bridge contracts.

### M7: Attached Package APIs and Owner Types

Owner: `sifr-lang/sifr`.

Scope:

- Add package declarations for fixed attached API sets.
- Support type, immutable, mutable, and owned receiver forms.
- Substitute the concrete owner type and `Self` in signatures.
- Retain non-owner type parameters for ordinary call-site inference.
- Include concrete residual generic arguments in code-generation identity.
- Bind existing package functions without synthesized method bodies.
- Define name-collision, visibility, inheritance, and generic rules.
- Keep attached APIs out of the structural shape and handler table.
- Add type-directed operations that require no dummy runtime value.
- Add a non-Pydantic attached-API fixture.
- Publish or select a Sifr toolchain that contains M1 through M7 and its
  self-contained structural dependencies.
- Treat unresolved `sifr-lang/sifr#3233` as a publication blocker. Do not use a
  source-checkout-only package path.

Acceptance criteria:

- Type methods can construct the concrete adapted owner through `Result`.
- Instance methods use normal Sifr borrow and ownership checks.
- A user-method collision is a compile-time error at both declarations.
- Concrete generic owners receive concrete attached signatures.
- A residual input type parameter is inferred from a method argument.
- An unbound generic owner cannot request a static program.
- A type-directed attached method runs without a dummy owner value.
- Attached API edits invalidate code generation without cycling shape identity.
- Source and installed toolchains pass the same adapter and attached-API
  fixtures.
- The installed sysroot contains all structural-identity dependencies.

Exit gate: a package can expose type-directed and instance-directed APIs on an
adapted class without generated HIR or runtime reflection.

### M8: Pydantic Model, Field, and Configuration Declarations

Owner: `sifr-lang/pydantic-sifr`.

Scope:

- Declare `BaseModel` through the generic adapter marker.
- Implement the Pydantic adapter and descriptor value types.
- Split descriptor normalization and Core Schema derivation by responsibility
  before the package schema-contract source file grows.
- Implement `Field`, `ConfigDict`, and Pydantic `Annotated` merge rules.
- Implement required fields, constant defaults, and typed factories.
- Implement explicit aliases, `AliasPath`, `AliasChoices`, and separate
  validation and serialization aliases.
- Implement deterministic alias generators.
- Implement common numeric, text, bytes, collection, pattern, and discriminator
  constraints.
- Implement field title, description, examples, and bounded
  `json_schema_extra`.
- Implement unconditional field serialization exclusion.
- Expose URL, multi-host URL, and compiled-pattern public field values through
  the M6 structural mapping contract.
- Preserve existing enum and literal schema derivation through the new facade.
- Implement common model configuration from the design.
- Derive the existing Core Schema and static program.
- Replace raw metadata in model, field, configuration, and constraint demos.

Acceptance criteria:

- The M8 fields-and-configuration example compiles without raw metadata.
- A milestone-scoped fields-and-configuration fixture compiles before handler
  and attached-method milestones begin.
- Typed descriptor values do not use string conversion.
- Invalid constraints identify the exact descriptor argument.
- Defaults, aliases, extras, strictness, and name population match the selected
  compatibility contract.
- Alias paths and choices select deterministic validation input locations.
- `extra="allow"` without a typed destination reports the `extra` configuration
  argument as a package issue.
- URL, multi-host URL, compiled-pattern, enum, and literal fields construct
  their documented public values.
- Nested, optional, union, tagged-union, recursive, and concrete generic models
  use the same declaration surface.
- Ordinary functional entry points and adapted models use the same engine.

Exit gate: users can declare the selected fields, constraints, defaults,
aliases, configuration, and specialized public values with familiar syntax.

### M9: Pydantic Validator Facade

Owner: `sifr-lang/pydantic-sifr`.

Scope:

- Implement `field_validator` in `before`, `after`, and `plain` modes.
- Support one or more explicit field targets per field validator.
- Reject wildcard field targeting with the documented terminal disposition.
- Implement `model_validator` in `before` and `after` modes.
- Let a field `before` handler declare an input type independent from its field
  type. Validate source input into that type before callback execution.
- Require a model `before` handler to declare one concrete structural input
  type.
- Run `after` model handlers in generated package glue after structural
  construction.
- Feed each returned `Self` to the next handler and return the final value.
- Check field targets, receiver forms, input types, output types, and errors.
- Implement deterministic local and inherited ordering.
- Forward typed validation context where the public signature requests it.
- Integrate callback failures into aggregate validation errors.
- Classify the statically typed before-handler contract as an adapted Pydantic
  behavior in the compatibility matrix.
- Port the selected validator compatibility corpus.

Acceptance criteria:

- Validator declarations require no raw metadata or manual method-slot list.
- Before handlers receive the declared input form.
- A field before handler accepts an input type that differs from the field type
  and returns a value for the normal field schema.
- A model before handler receives its declared concrete structural input type.
- After handlers receive validated field or model values.
- Plain handlers replace the normal field pipeline by explicit package policy.
- Invalid signatures fail during checking.
- Handler errors preserve field or model locations and package context.
- An after model handler consumes the constructed value. Its returned `Self`
  becomes the validation result.
- An after model handler error joins the aggregate error at the model root.
- Validator panics cannot escape the native boundary.

Exit gate: the selected field and model validator API works through the single
schema engine for structural and JSON input.

### M10: Pydantic Serializer and Computed-Field Facade

Owner: `sifr-lang/pydantic-sifr`.

Scope:

- Implement `field_serializer` and `model_serializer`.
- Implement `when_used` policies for always, unless-`None`, JSON, and
  JSON-unless-`None` execution.
- Implement `computed_field` from checked user methods.
- Require `computed_field` on a checked zero-argument instance method. Treat
  Python `@property` stacking as not applicable.
- Attach `model_dump` and `model_dump_json` as instance methods.
- Define their typed selection, exclusion, alias, and mode arguments.
- Define serializer and computed-field declaration order.
- Support aliases and typed include or exclude selections.
- Support `exclude_defaults`, `exclude_none`, and mode selection.
- Forward typed serialization context where declared.
- Make JSON Schema include computed fields in serialization mode only.
- Port the selected serializer and computed-field compatibility corpus.

Acceptance criteria:

- Serializers observe the current model value after mutation.
- Computed fields add no stored field or constructor parameter.
- Serializer `when_used` policies select the documented mode and `None`
  combinations.
- A computed field accepts a checked zero-argument instance method and rejects
  incompatible receiver or parameter forms.
- Invalid serializer and computed-field signatures fail during checking.
- Selection, alias, and exclusion order is deterministic.
- Serializer errors preserve structured context.
- Serializer panics cannot escape the native boundary.
- `model_dump` returns `dict[str, JsonValue]` without a JSON byte round trip.
- Dump methods and serializer handlers use the same sealed static program.

Exit gate: the selected serializer and computed-field API works through the
existing projection and serialization engine.

### M11: Complete Model Operations and Schema Surface

Owner: both repositories, with compiler work merged first.

Scope:

- Attach `model_validate`, `model_validate_json`, and
  `model_validate_strings` as type methods.
- Replace the current `bytes` strings-profile input with a generic structural
  input `S`.
- Accept `S` only for a bare `str` root or a structural input whose scalar
  leaves and mapping keys are strings.
- Attach type-directed `model_json_schema` without a dummy value.
- Expose typed validation, serialization, and JSON Schema configuration.
- Remove hardcoded bridge configuration that competes with the static program.
- Expose complete structured validation and serialization errors.
- Complete `RootModel[T]` and `TypeAdapter[T]` public ergonomics.
- Implement `RootModel[T]` as a declared generic class with one stored `root`
  field. Do not synthesize storage through the adapter.
- Complete inherited and concrete generic model operation behavior.
- Preserve functional APIs only as thin package functions over the same path.

Acceptance criteria:

- Every model operation uses the same sealed static program.
- One integration test records equal static-program identity across model
  validation, dumping, and JSON Schema for one concrete model.
- `model_json_schema` does not accept or ignore a model value.
- Bridge code does not select independent integer or model policy defaults.
- Error details include code, message, location, and context.
- Structural and JSON output use documented typed results.
- `RootModel[T]` and `TypeAdapter[T]` use the same schema authority.
- A bare string root and a nested all-string structural input pass the strings
  profile. An input with a non-string leaf fails during checking.
- Source and installed package operation is equivalent.

Exit gate: the complete selected Pydantic model API is available as familiar
type and instance methods without a second execution path.

### M12: Migration, Certification, and Closure

Owner: both repositories.

Scope:

- Convert all canonical demos and quickstarts to the ergonomic API.
- Remove normal-user documentation for raw metadata and specialization.
- Remove temporary declarations and duplicate configuration authorities.
- Update the compatibility matrix for every included and excluded feature.
- Add migration guidance from Python Pydantic and the old Sifr package surface.
- Document ordinary Sifr construction or cloning as the replacement for the
  excluded dynamic `model_copy` update API.
- Run compiler, package, differential, property, fuzz, and resource tests.
- Run source-package and installed-package certification.
- Publish representative validation and serialization benchmarks.
- Obtain independent implementation and whole-phase review.

Acceptance criteria:

- The canonical demo contains no raw metadata or specialization decorators.
- The package documentation uses the attached API as the primary path.
- Every included feature has positive, negative, and diagnostic evidence.
- Every exclusion is explicit in the compatibility matrix.
- No temporary syntax, fallback, second engine, or duplicate authority remains.
- The compiler fixture proves that the substrate is not Pydantic-specific.
- The Sifr and Pydantic-Sifr authoritative gates pass on reviewed candidates.
- Released artifacts pass the installed-package demo and certification suite.

Exit gate: a user can install Pydantic-Sifr and use the complete selected API
without Python, raw metadata, compiler special cases, or uncaught panics.

## Validation and Review

### Per-item validation

Each item runs focused tests for its changed compiler or package paths. It also
runs documentation checks and the file-size guardrail when applicable.

Do not repeat unchanged validation evidence. Record an unrelated error in its
owning issue.

### Pull-request gate

Run the repository `create-pr` profile on the final item candidate. Run the
companion repository equivalent for package-owned items.

### Review

Use the phase-closure review protocol for every item. The review input must
contain exact base and candidate SHAs, changed paths, item criteria, and local
validation evidence.

Only an in-scope omission or regression can block the item. Record suggestions
as separate follow-up work.

### Merge gate

Run the merge gate once on the final reviewed implementation candidate. Do not
repeat it for a record-only phase update.

## Phase Acceptance Criteria

The phase is complete only when all criteria are true:

- M0 through M12 are merged in order.
- The compiler contract is package-neutral and has a non-Pydantic consumer.
- Pydantic-Sifr implements the complete selected public surface.
- URL, multi-host URL, and compiled-pattern fields use Sifr-visible public
  values.
- `BaseModel` is an erased adapter marker, not a data parent.
- `Field`, `ConfigDict`, and `Annotated` use typed descriptors.
- Requiredness and defaults are final before constructor synthesis.
- Validators, serializers, and computed fields use checked user-method slots.
- Model operations are attached checked package functions.
- `model_dump` returns `dict[str, JsonValue]`.
- The strings-profile API accepts only a bare string or an all-string
  structural input.
- Diagnostics identify declaration and descriptor arguments.
- Validation, serialization, and JSON Schema use one static program.
- The bridge contains no independent model configuration authority.
- Normal users do not write raw metadata or specialization declarations.
- Source and installed package certification passes.
- Independent final review reports no blocking finding.

## Milestone Record Template

Add this record under a milestone after its item merges:

```text
State: complete | blocked
PR:
Base SHA:
Candidate SHA:
Merge SHA:
Changed paths:
Validation:
Review evidence:
Deferred follow-up:
Next action:
```

## Current Handoff

Current state: proposed design and phase plan.

Next action: complete M0. Review the contract, fix blocking design findings,
and record the accepted contract before compiler implementation starts.
