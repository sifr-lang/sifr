# Deterministic Const Specialization

Status: active compiler contract.

Sifr packages can derive immutable static descriptions from concrete source types without
runtime reflection or compiler knowledge of package names. The frontend owns structural shape
extraction, bounded evaluation, diagnostic validation, and cross-mode transport. Packages own
their specialization functions and the meaning of the static values they produce.

## Source surface

- `@const_eval` marks a pure package function that the deterministic frontend evaluator may run.
- `@const_specialize("package.module", "function")` on a concrete class requests that function
  with the class's closed structural shape. The package module must also be imported so ordinary
  package dependency ordering applies.
- `@metadata("package.key", const_value)` attaches typed metadata to a type, function, or method.
- `@metadata("field", "name", "package.key", const_value)`, `enum_variant`, and `parameter`
  target declaration children. Values must lower to closed const data.
- `@metadata("sifr.meta.issue_template", ("reason_code", ["argument", ...]))` on a
  specializer declares its bounded package-issue arguments.
- `@json_integer_boundary(field, profile, representation, minimum, maximum)` declares a
  package-neutral integer JSON boundary. `profile` is `exact`, `web`, `string_ints`, or `None`;
  representation is `default`, `number`, or `decimal_string`.
- `@class_adapter_provider("descriptor.module", "Descriptor")` marks an `@const_eval` function
  whose exact signature is `(DeclarationInput[D]) -> DeclarationPlan[D]`.
- `@class_adapter_marker("provider.module", "provider_function")` marks a field-less, erased
  class that selects one canonical provider when used as a class base.

The public `sifr.meta` module contains `SourceOrigin`, `ClassDeclaration`,
`ClassDecoratorDeclaration`, `ClassParameterDeclaration`, `ClassDeclarationItem`,
`ConstIssueLabel`, `ConstIssueTemplate`, generic `ConstPackageIssue[A]`, generic
`ConstSpecializationOutcome[T, A]`, and `JsonIntegerBoundaryDescriptor` value types. Package
argument records remain statically typed package code; the compiler converts them to a closed
const record only at the specialization boundary.

Every specialization input also contains one `declaration: ClassDeclaration`. The compiler
collects this declaration before class lowering finalizes storage. It preserves class decorators,
fields, annotated values, class items, methods, method decorators and parameters, and source
order. Declaration and decorator-argument locations are represented only by `SourceOrigin`.
`SourceOrigin` is an opaque compiler value: package code can forward an origin supplied in the
current declaration, but cannot construct, inspect, serialize, return, or select an origin from a
different class declaration.

## Structural shape

The compiler description preserves primitive and fixed-integer kinds, exact nominal identity,
generic arguments, declaration-order fields, required/defaulted state and values, typed metadata,
tuples, collections, optional and union members, enum variants, newtype bases, and recursive
references. Its canonical identity includes defaults and metadata and is independent of hash-map
iteration order. Recursive occurrences are identity references rather than expanded copies.

Structural descriptions are compile-time data only. They do not create a runtime reflection API.

## Early class adapters

An adapter marker is a compile-time base, not a data parent. It contributes no stored fields,
constructor parameters, runtime class value, or `super()` work. An adapted class can select only
one canonical provider and can also have one ordinary data parent. Conflicting providers and a
second data parent fail at the base declaration.

The frontend first builds a provisional typed declaration, evaluates its descriptor calls, and
runs the selected adapter with `DeclarationInput[D]`. It validates the returned bounded plan
before it rebuilds the finalized class HIR. The adapter plan must echo every stored field identity,
order, and canonical type exactly. It can add typed metadata for the class or its existing fields,
emit bounded package issues, and request at most one package specialization. Unknown plan fields,
field additions or removals, type changes, method-body output, forged origins, and descriptor-type
mismatches fail closed.

The provider descriptor type `D` is also the metadata value type in `ShapeInput[D]`. Adapter
metadata therefore reaches specialization as its checked record or closed-union value rather than
as a string. Adapter evaluation uses the normal deterministic const evaluator limits, and its
issues use the same package-owned templates and compiler-owned source origins as specialization.

Descriptor arguments can call a checked `@const_eval` function by its direct imported name.
The call uses the function's checked parameter types, keyword names, and defaults.
An imported or re-exported function keeps its canonical body and package identity.
Lists evaluate each item against the declared item type.
Nested calls use the same deterministic limits as adapter and specialization evaluation.

## Evaluation and outcomes

Const evaluation is deterministic and fails closed. Default limits are 100,000 evaluation steps,
64 nested calls, and 10,000 values in a collection. It accepts the closed pure HIR subset used for
static derivation and rejects runtime effects, unsupported expressions, missing `@const_eval`,
budget exhaustion, and escaping loop control.

The closed evaluator supports `isinstance` checks for primitive const values.
Supported names are `bool`, `int`, `float`, `str`, `bytes`, `tuple`, `list`, and `dict`.
Iteration over a closed record yields its string keys in canonical order.

A package result is a closed record with exactly `status`, `value`, and `issues`. Each issue has
exactly `package`, `reason_code`, `severity`, `arguments`, `primary_origin`, `labels`, and `notes`.
Each label has exactly `origin` and `message`. The compiler resolves the primary and related
origins against the current declaration and owns the resulting source spans. Produced values may
carry only warnings. Failed outcomes contain at least one fatal issue and no value.
Package/reason namespaces, template arguments, text, issue counts, nesting, labels, and notes are
bounded and validated before rendering. A missing, forged, or unrelated origin fails closed with
`SIFR-META-0003` at the specialization request.

The frontend maps fatal, warning, and malformed results to `SIFR-META-0001`,
`SIFR-META-0002`, and `SIFR-META-0003`. Package text never becomes a top-level compiler code or
renderer. `SIFR-META-0002` is a hard compiler warning, not a suppressible lint. The same lowering
and rendered-diagnostic path is used by check, build, tests, and editor analysis.

For a produced outcome, the frontend retains the value together with its target owner, package
module, and specializer function in the lowering result. The retained value uses a deterministic
closed-value encoding; project compilation exports it with the declaring module so later compiler
stages can consume the static result without executing package code again. Failed outcomes never
produce retained static data.

Each retained result also has one static-program identity. The hash includes the declaring module,
concrete owner, package module, specializer function, canonical structural shape, canonical result,
and structural-contract identity. Check and editor analysis retain this identity. Build and run emit
the canonical bytes. For a structural static program, they also emit an allocation-free borrowed
view of the same closed value. The generated-project cache key includes the same identity.
Source origins and byte locations are excluded from both the canonical structural shape and the
retained static value. Moving an otherwise unchanged declaration therefore updates diagnostic
locations without changing the static-program identity.

An adapted shape field records one of four default states: `required`, `const`, `factory`, or
`runtime`. Constant defaults carry their closed value. Factory defaults carry the compiler-sealed
`CallableIdentity`; the callable's canonical identity participates in the structural shape and
static-program identity. `runtime` identifies an ordinary non-constant class default that cannot
participate in a static program. Structural construction accepts named record edges in any order,
rejects unknown or duplicate names, and evaluates the checked class default for each omitted field.
Omitting a required field returns `ArityMismatch`. A factory therefore runs once per constructed
value and mutable factory results are not shared.

For a structural Rust call with the compiler-owned `sifr.meta.StaticProgram` bound, the compiler
also emits a sealed typed `StaticProgram[T]` envelope and implements `StaticProgramType` for the
concrete specialized type. This bound requires a produced specialization result. It has no empty
program, runtime compiler, compatibility path, or fallback to the ordinary `Structural` bound.
`StaticProgramValue` is non-exhaustive for Rust consumers. Its current compiler-owned variant set
is closed for this contract. It includes `CallableIdentity`, a sealed checked target containing the
canonical module, optional owner, symbol, concrete generic arguments, and canonical signature.
All five components participate in the canonical result and therefore the static-program and cache
identity. Packages can carry this value through records and collections but cannot construct it
from strings. A new variant requires a contract and cache-identity review.

The provisional adapter pass accepts `StaticProgram` bounds before attached API selection finishes.
The final pass still requires a produced program and complete structural support.
Descriptor-shaped field defaults do not affect required-field ordering during the provisional pass.
The finalized adapter field plan supplies the required and default states for the final check.

An unbound generic adapted declaration does not request a schema program.
A concrete owner must supply all type arguments before static program generation.
An attached call through a concrete generic type alias uses the resolved class as
that owner. The alias does not create a second schema program. Attached-call
lowering also uses the package function's checked defaults for omitted public
arguments; instance receivers shift default indexes after the hidden owner is
removed.

Project code generation checks structural program owners against the complete module graph.
An imported mapped opaque field uses the generated package type's structural identity.
It does not refer to the dependency's private Rust bridge path from the consumer module.

A produced value can request a method-slot table through the reserved `sifr_method_slots` field.
The field is ordered. An empty list emits no slot table, which lets one typed specialization
payload cover types with and without handlers. Package method descriptors expose compiler-sealed
`CallableIdentity` values in the structural shape. The adapter validates the descriptor value and
source origin, and the specializer selects those identities through `sifr_method_slots`. Legacy
annotated slots can still use exact `module.Type::method` strings.

Each described method contains its declared `result`, its successful `output`, and a `fallible`
flag. For `Result[T, E]`, `output` describes `T` and `fallible` is true. For other results,
`output` equals `result` and `fallible` is false. Packages use these typed facts to reject invalid
handler signatures during specialization. They do not parse a displayed type name.

The compiler accepts synchronous static, class, shared-instance, mutable-instance, and owned
receiver handlers. A checked handler can return an infallible structural value or a typed `Result`.
An instance handler can also declare one owned structural value before its optional borrowed
context. The slot input is then the structural tuple `(receiver, value)`, and generated dispatch
reconstructs both values before the checked method call.
An owned receiver must return the exact current `Self` specialization, directly or as the successful
`Result` value. Duplicate, unqualified, missing, constructor, asynchronous, invalid-context, and
nonstructural contracts fail with `SIFR-RUST-SLOT-####` at the method descriptor when one exists.
The compiler derives one context type and borrow mode from the selected methods. Programs without
caller-owned context use `NoContext`. A nonempty table identity includes slot order, structural
signatures, receiver mode, context shape and borrow mode, descriptor-backed handler identities,
and the static-program identity.

## Integer boundary verification

`JsonIntegerBoundaryDescriptor` verification fails closed when the profile is missing, the static
range is malformed, `json.string_ints` requests numbers, `json.exact` requests decimal strings, or
`json.web` numeric output lacks a JavaScript-safe static range. Failures use registry-owned
`SIFR-INT-0009`; runtime values that exceed an accepted policy continue to use
`JsonIntegerRangeError` through `sifr_runtime::json`.

The package-neutral conformance fixture is
`verification/areas/core_language/fixtures/const_specialization_general`. It includes a separate
specializer module, a concrete consumer, a malformed request, and an unsafe integer descriptor.
