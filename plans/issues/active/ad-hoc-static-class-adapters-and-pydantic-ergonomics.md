# Ad Hoc Phase: Static Class Adapters and Pydantic Ergonomics

## Status

Status: active on 2026-08-18. M0 is complete; M1 is next.

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

These exclusions are terminal for this phase. Their reasons are architectural,
not missing milestone dependencies:

| Excluded feature | Terminal reason |
| --- | --- |
| Python metaclasses or runtime class mutation | Classes and schema programs are finalized statically; runtime mutation would invalidate checked layout and identity. |
| Dynamic `create_model` | Runtime type and schema creation conflicts with concrete compile-time type identity and sealed programs. |
| Arbitrary syntax-tree macros | The adapter is deliberately limited to typed declarations and a bounded plan so packages cannot rewrite language semantics. |
| Runtime schema construction | One build-time canonicalizer and sealed static program remain the only schema authority. |
| Python plugins or custom Core Schema hooks | Published binaries contain no Python runtime, and open runtime hooks would bypass the checked schema contract. |
| Pydantic dataclasses | Dataclass field discovery and construction are a separate authoring model; this phase standardizes the adapted-class model only. |
| Private attributes | An adapter cannot add hidden per-instance storage or fields outside the declared structural layout. |
| `validate_call` | Intercepting arbitrary function calls is a separate function-adaptation mechanism, not class declaration adaptation. |
| `model_construct` | Construction that bypasses validation conflicts with the sealed validate-and-construct boundary. |
| Dynamic `model_copy` updates | Dynamic field updates conflict with static field typing and ownership; ordinary Sifr construction or explicit cloning is used instead. |
| Runtime `model_fields` and `model_rebuild` | Runtime reflection and schema rebuilding conflict with immutable compile-time shapes and programs. |
| ORM `from_attributes` | Arbitrary attribute probing has no place in Sifr's typed structural-input contract. |
| Arbitrary runtime types | Values require a statically checked type and structural or declared nominal mapping; there is no unchecked runtime-type escape. |
| Multiple data inheritance | One data parent preserves deterministic layout, constructor synthesis, and field identity. |
| Mixed class-adapter providers | Two providers would create ambiguous declaration-plan, ordering, and cache authorities. |
| Assignment-validation interception | Intercepting ordinary field mutation would change core assignment semantics and ownership rather than adapt a declaration. |
| Python-compatible frozen-model emulation | Sifr uses ordinary static immutability and ownership contracts instead of a Python runtime flag. |
| Public wrap-handler continuations | A user-visible continuation would require a new ownership, lifetime, and effect contract; internal engine wrap nodes do not expose that continuation. |
| Wildcard `field_validator("*")` targeting | Explicit field identities keep target checking, diagnostics, inheritance, and ordering static and deterministic. |
| Schema generation for an unbound generic model | A schema program requires a concrete owner type, substituted fields, and a complete cache identity. |

No milestone in this phase implements or depends on an excluded feature.

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

## M0 Coverage and Ownership Inventory

The implementation baseline for this inventory is compiler merge
`afc87ef9dbe669ced9eca1b2fa57a9eeef809ffb` and `pydantic-sifr` main
`0c643a676d821b92ce4dfa824a8f6a5b98073d4c` on 2026-08-18. The completed
native engine remains the sole execution path. The table identifies the
existing path, the package-neutral compiler mechanism still required, and the
one package or engine owner for every selected feature family.

| Selected feature family | Current implementation support | Compiler mechanism owner | Package/engine owner and delivery |
| --- | --- | --- | --- |
| `BaseModel`, adapted inheritance, and concrete generic models | Ordinary data inheritance and raw `@const_specialize` exist; no erased marker facade exists. | M1-M4 declaration collection, marker, defaults, inheritance, and identity | `pydantic-sifr` M8 declares the marker and adapter; the existing Core Schema engine is reused. |
| `Field`, `ConfigDict`, and `Annotated` | Raw `@metadata` strings and a scalar-declared `ShapeMetadata.value` feed the current specializer. | M1-M4 spanned typed descriptors, typed `D`, normalized defaults, and ordered annotation descriptors | `pydantic-sifr` M8 owns descriptor values, merge rules, configuration, and Core Schema derivation. |
| Required, constant-default, and factory-default fields | Ordinary defaults and engine default nodes exist, but descriptor calls cannot finalize constructor requiredness and factories have no sealed callable identity. | M2 and M4 own `CallableIdentity`, field states, type checking, constructor synthesis, and identity. | `pydantic-sifr` M8 owns default validation policy and schema nodes. |
| Aliases, alias paths/choices, generators, constraints, extras, strictness, field schema annotations, and unconditional field exclusion | The engine and raw-metadata specializer already implement the selected policies. | M1-M4 provide precise origins and typed declaration delivery only. | `pydantic-sifr` M8 replaces raw metadata and remains the sole policy/schema owner. |
| Nested, optional, union, tagged-union, recursive, enum, literal, and concrete generic model derivation | The existing Core Schema path supports these shapes through raw specialization. | M2-M4 preserve typed recursive identity, descriptors, inheritance, and concrete substitution. | `pydantic-sifr` M8 derives the same existing schema nodes from the new facade. |
| URL, multi-host URL, and compiled-pattern public values | The native core validates specialized payloads, but safe Sifr-visible nominal construction/projection is missing. | M6 owns package-neutral nominal mapping and structural output. | `pydantic-sifr` M8 owns public wrapper mappings and schema behavior; issue #27 transfers when M6 changes its substrate. |
| Field and model validators | Checked method-slot prototypes and engine validator stages exist; declaration-bound handler selection and dispatch are incomplete. | M5 owns checked method descriptors, `Self`, owned receivers, handler identity, dispatch, and cleanup. | `pydantic-sifr` M9 owns modes, targets, ordering, context, errors, and schema placement; issue #10 transfers when its owned handler substrate is changed. |
| Field/model serializers and computed fields | Serializer plans exist, but checked user serializer/computed method dispatch is blocked. | M5 supplies handler slots; M6 supplies structural output; M7 supplies dump attachment. | `pydantic-sifr` M10 owns handler policy, `when_used`, computed fields, selection, serialization mode, and corpus; issues #14 and the selected rows of #17 transfer here. |
| Structural, JSON, and strings-profile validation | Free functions use the sealed engine; strings input is currently `bytes`, not generic structural `S`. | M7 owns attached type methods; M11 owns the generic all-string structural-input check. | `pydantic-sifr` M11 owns public signatures and keeps the existing validator engine. |
| Structural and JSON serialization, typed include/exclude, alias/default/`None` policies | `model_dump_json` and the serializer engine exist; no `dict[str, JsonValue]` structural dump exists. | M6 owns structural dictionary output; M7 owns attached instance methods. | `pydantic-sifr` M10 owns dump signatures and serialization policy over the existing engine. |
| Validation-mode and serialization-mode JSON Schema | The existing free function emits from the sealed program but requires a dummy typed value. | M7 owns type-directed attached APIs with no owner value. | `pydantic-sifr` M11 owns the type method and configuration over the existing JSON Schema engine. |
| Structured validation and serialization errors | The native core has structured validation details; facade coverage and complete serialization error exposure remain. | M1 supplies descriptor-origin diagnostics; M5-M7 preserve checked callback/API boundaries. | `pydantic-sifr` M9-M11 own public error types, context, and aggregation. |
| `RootModel[T]` | Non-model roots work in the engine; no familiar declared generic facade exists. | M3-M4 and M7 provide marker adaptation, concrete generic identity, and APIs. | `pydantic-sifr` M8 declares the stored `root` field and M11 completes operations. |
| `TypeAdapter[T]` | The Rust core has a reusable typed adapter and target-inferred functional calls; no selected public class facade exists. | M7 provides concrete owner/API substitution and rejects unbound generic programs. | `pydantic-sifr` M11 owns the facade and routes it to the same sealed program. |

### Public operation replacement inventory

| Public operation | Baseline path | Classification | Final owner |
| --- | --- | --- | --- |
| `Model.model_validate` | `model_validate[T, Input](input)` free function | Existing path to attach and retain only as a thin functional view | Compiler M7 attachment; package M11 signature and engine call |
| `Model.model_validate_json` | `model_validate_json[T](bytes)` free function | Existing path to attach and retain only as a thin functional view | Compiler M7; package M11 |
| `Model.model_validate_strings` | `model_validate_strings[T](bytes)` free function | Existing path to replace with generic structural `S`, attach, and retain as a thin view | Compiler M7/M11 checking; package M11 |
| `model.model_dump` | No public structural-dictionary operation | Net-new facade over the existing serializer plan and M6 structural output | Compiler M6/M7; package M10 |
| `model.model_dump_json` | `model_dump_json[T](value)` free function | Existing path to attach and retain only as a thin functional view | Compiler M7; package M10 |
| `Model.model_json_schema` | `model_json_schema[T](target: T)` free function whose value is ignored | Existing path to replace with a type-directed call and retain only as a thin functional view without a dummy value | Compiler M7; package M11 |
| `TypeAdapter[T]` validation, dump, and schema methods | Reusable native typed adapter plus target-inferred free functions | Net-new public facade over existing sealed-program operations | Compiler M7; package M11 |
| `RootModel[T]` validation, dump, and schema methods | Engine supports non-model roots; no declared facade | Net-new public facade using the same attached operation sets | Compiler M3/M4/M7; package M8/M11 |

All operation facades above call the existing Core Schema program. None owns a
second validator, serializer, schema generator, or configuration authority.

## Existing Dependency Reconciliation

M0 reconciles this phase with the existing owners as follows:

| Existing issue | Current state and ownership disposition |
| --- | --- |
| [`sifr-lang/sifr#3233`](https://github.com/sifr-lang/sifr/issues/3233) | Closed before M0. Its self-contained installed-sysroot result is a consumed M7 publication prerequisite, not active duplicate work. |
| [`pydantic-sifr#10`](https://github.com/sifr-lang/pydantic-sifr/issues/10) | Remains the owner of modular const-graph and handler-slot gaps until M5 or M8 first changes that code. That merge unit must transfer the selected substrate into this phase, update or close #10, and leave public wrap continuations terminally excluded. |
| [`pydantic-sifr#14`](https://github.com/sifr-lang/pydantic-sifr/issues/14) | Remains the owner of serializer/computed callback execution and typed context until M10 changes the owned package code. M10 must transfer and update or close #14 in the same merge unit. |
| [`pydantic-sifr#17`](https://github.com/sifr-lang/pydantic-sifr/issues/17) | Remains the serializer-corpus owner. M10 transfers only rows selected by this phase when their typed surfaces land; out-of-scope temporal, Decimal, Complex, and UUID work remains with its existing owners. |
| [`pydantic-sifr#27`](https://github.com/sifr-lang/pydantic-sifr/issues/27) | Remains the owner of specialized public-value mapping gaps until M6 changes the substrate. M6 must transfer and update or close #27 in the same merge unit. |

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

State: complete
PR: [`sifr-lang/sifr#3240`](https://github.com/sifr-lang/sifr/pull/3240),
building on the design foundation merged by
[`sifr-lang/sifr#3239`](https://github.com/sifr-lang/sifr/pull/3239)
Base SHA: `afc87ef9dbe669ced9eca1b2fa57a9eeef809ffb`
Candidate SHA: `89b9f226371e14aa12d66a14503e8c97117e22a0`
Merge SHA: `2a7b7d68438d9410650a432dd2c0859c70152d07`
Changed paths: `internal_docs/native_pydantic_sifr_architecture.md`;
`plans/issues/active/ad-hoc-static-class-adapters-and-pydantic-ergonomics.md`
Validation: `git diff --check`; `python3
scripts/check_docs_error_code_links.py`; `python3
scripts/check_file_size_guardrails.py` (`PASS`, 3149 files, 900-line limit).
Documentation-only item, so Sifr create-PR and merge gates were not run.
Review evidence: initial exact-SHA Opus review of
`b3c757e8bdd22917501a1bf485522c96f9e41464` reported two blocking omissions
([evidence](https://github.com/sifr-lang/sifr/pull/3239#issuecomment-5326156228));
the one remediation review of final candidate
`89b9f226371e14aa12d66a14503e8c97117e22a0` returned `SATISFIED` with no
blocking finding
([evidence](https://github.com/sifr-lang/sifr/pull/3240#issuecomment-5326151512)).
Deferred follow-up: M1 must use one canonical name for the provider declaration
input (`ClassDeclaration[D]` or `DeclarationInput[D]`); M2 must update
`internal_docs/const_specialization.md` and cache-identity evidence with
`CallableIdentity`; M9-M11 must classify public structured-error accessors when
their signatures are fixed. Existing issue transfers remain governed by the
reconciliation table above.
Next action: implement M1 spanned class declarations and package issues.

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

State: complete
PR: [`sifr-lang/sifr#3243`](https://github.com/sifr-lang/sifr/pull/3243)
Base SHA: `b80a7059a6f15171803b9f32a38aa5eed2ada6ac`
Candidate SHA: `613464a9229981459b32bbd61a9f93c6ee9d69b6`
Merge SHA: `991323596c40a8def31dd3868764eec096e0e01c`
Changed paths: `crates/sifr_frontend/src/{class_declarations.rs,
package_issues.rs,const_evaluator.rs,const_specialization.rs,
graph_cache_and_queries.rs,lib.rs,query_diagnostic_rendering.rs,
specialization_runner.rs,specialization_support.rs,structural_shape.rs,
warning_diagnostics.rs}`; `crates/sifr_ir/src/diagnostic_types.rs`;
`crates/sifr_lsp/src/{conversion.rs,diagnostics.rs}`;
`stdlib/sifr/meta.sifr`; `internal_docs/const_specialization.md`; and the
const-specialization and meta-diagnostic fixtures and compact baselines.
Validation: `cargo check -p sifr_frontend -p sifr_lsp`; all 9 focused
specialization-runner tests; both class-declaration origin tests; all 5 LSP
conversion tests; `cargo clippy -p sifr_frontend -p sifr_lsp -- -D warnings`;
`cargo fmt --check`; HIR maintainability and file-size guardrails (`PASS`,
3151 files, 900-line limit). Direct human CLI reproduction selected the
metadata argument as primary and rendered class, field, and method related
spans plus the package note. The diagnostics-area attempt reached both M1
fixtures, while 142 unrelated cases were replaced by `SIFR-WORKSPACE-0003`
because the separately owned LeetCode corpus was not initialized in this
worktree.
Review evidence: the one exact-SHA Opus review returned `SATISFIED` with no
blocking findings
([evidence](https://github.com/sifr-lang/sifr/pull/3243#issuecomment-5326628275)).
The create-PR and merge gates were each run once on the unchanged candidate.
Both stopped in the same inherited global verification-taxonomy check before
M1-specific lanes; the separately running pre-v1 compatibility phase replayed
that failure at its own base and owns the offending inventory/checker paths.
The exact merge-gate evidence is recorded on the PR
([evidence](https://github.com/sifr-lang/sifr/pull/3243#issuecomment-5326716293)).
Deferred follow-up: M2 must carry keyword names with argument origins when it
adds typed descriptor arguments, rather than exposing only positional origin
order. M12 certification should add human/JSON fixture baselines for related
spans and re-evaluate the eight-label bound, lazy origin collection, origin
kind enforcement, and spanless label degradation. Cross-file LSP span URI
handling is pre-existing and separately owned; M1 origins are same-file.
The canonical public declaration-input name is now `ClassDeclaration`, closing
M0's naming follow-up.
Next action: implement M2 typed declaration descriptors.

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

Current state: M0 contract lock and coverage inventory merged and recorded.

Next action: complete M1. Add spanned pre-finalization declarations and
source-origin package issues through one CLI/LSP diagnostic path.
