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

## Evaluation and outcomes

Const evaluation is deterministic and fails closed. Default limits are 100,000 evaluation steps,
64 nested calls, and 10,000 values in a collection. It accepts the closed pure HIR subset used for
static derivation and rejects runtime effects, unsupported expressions, missing `@const_eval`,
budget exhaustion, and escaping loop control.

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

For a structural Rust call with the compiler-owned `sifr.meta.StaticProgram` bound, the compiler
also emits a sealed typed `StaticProgram[T]` envelope and implements `StaticProgramType` for the
concrete specialized type. This bound requires a produced specialization result. It has no empty
program, runtime compiler, compatibility path, or fallback to the ordinary `Structural` bound.
`StaticProgramValue` is non-exhaustive for Rust consumers. Its current compiler-owned variant set
is closed for this contract. A new variant requires a contract and cache-identity review.

A produced value can request a method-slot table through exactly one reserved entry:
`sifr_method_slots: list[str]`. The field is ordered. An empty list emits no slot table, which
lets one typed specialization payload cover types with and without callbacks. Each value is an exact
`module.Type::method` identity, including for imported or re-exported owners. Duplicate,
unqualified, missing, unannotated, asynchronous, constructor, class-method, non-`Result`, or
nonstructural method contracts fail with `SIFR-RUST-SLOT-####`. The compiler derives one context
type and borrow mode from the selected methods and emits the table only when a Rust declaration
uses the package-neutral `MethodSlots` and `Context` bounds. Programs without caller-owned
context use `NoContext`. A nonempty table identity includes slot order, structural signatures, receiver
mode, context shape and borrow mode, handler shapes, and the static-program identity.

## Integer boundary verification

`JsonIntegerBoundaryDescriptor` verification fails closed when the profile is missing, the static
range is malformed, `json.string_ints` requests numbers, `json.exact` requests decimal strings, or
`json.web` numeric output lacks a JavaScript-safe static range. Failures use registry-owned
`SIFR-INT-0009`; runtime values that exceed an accepted policy continue to use
`JsonIntegerRangeError` through `sifr_runtime::json`.

The package-neutral conformance fixture is
`verification/areas/core_language/fixtures/const_specialization_general`. It includes a separate
specializer module, a concrete consumer, a malformed request, and an unsafe integer descriptor.
