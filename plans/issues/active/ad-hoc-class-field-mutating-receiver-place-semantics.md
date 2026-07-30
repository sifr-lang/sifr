# Ad Hoc Issue: Class-Field Mutating Receiver Place Semantics

## Status

Implementation in progress for the urgent correctness follow-up identified
during the M10 declaration-first Python interop milestone review. Claude Opus
implementation-readiness pass 7 returned `SATISFIED`. Item 1, canonical
receiver metadata and inference, merged in
[#3065](https://github.com/sifr-lang/sifr/pull/3065) after implementation
review pass 2 returned `SATISFIED`. Item 2, checked place semantics and defect
closure, is under PR
[#3082](https://github.com/sifr-lang/sifr/pull/3082), and is undergoing
exact-head remediation after PR review pass 6 returned `NOT SATISFIED`;
implementation review pass 5 returned `SATISFIED` after passes 1 through 4
returned `NOT SATISFIED`. The remediation
restores non-cloning shared field receivers, checks delegated fixed-trait
mutation after receiver convergence, makes owned temporary proof exhaustive,
rejects storage-selecting conditionals and re-materialized module constants,
closes shared-receiver/mutable-argument overlap, restores mutable-borrow flow
effects, applies the protected-root optimizer contract to production codegen,
and migrates the LRU compatibility fixture to snapshot `self.head` before a
mutable `self` call as required by same-call exclusivity.

The defect predates M10 and was not introduced by the buffer implementation,
but it violates Sifr's core guarantee: a program can compile and silently lose
a source-visible mutation. This issue is not implementation-ready until the
review ledger below ends in an independent `SATISFIED` verdict.

## Origin and current behavior

The originating M10 review reproduced the defect on both the then-current tree
and the pre-M10 installed release:

```python
class Helper:
    items: list[int]

    def __init__(self):
        self.items = []

    def bump(self) -> None:
        self.items.append(1)

class Owner:
    helper: Helper

    def run(self) -> None:
        self.helper.bump()
```

The current compiler can emit `self.helper.clone().bump()`. The call mutates a
temporary clone, not `self.helper`, and the program observes no mutation.

The first independent review also reproduced these adjacent failures:

- `self.mid.inner.bump()` clones an intermediate field hop;
- an inherited field receiver can clone after parent-storage re-rooting;
- `box.helper.bump()` silently loses mutation even when `box` is a mutable
  borrowed parameter;
- the immutable-parameter form also compiles and silently loses mutation;
- `self.helper.absorb(self.stock)` preserves the mutable argument mutation but
  loses the receiver mutation.

Current code already contains a partial, ambient workaround:

- `RustEmitter::method_call_needs_field_clone_suppression` recognizes many of
  the affected receivers;
- `pending_self_field_clone_suppression` transports that decision as a
  decrementing emitter counter;
- only selected expression emitters arm the counter;
- `lower_field_access_expr_with_lowered_object` consumes it at the next
  qualifying field read, rather than at an explicitly identified place;
- M10 added the same counter mechanism for mutable field arguments.

This issue replaces that positional state with checked receiver/place metadata.
Adding another counter arm site or another receiver-depth special case is not
an acceptable fix.

## Goals

- Make every accepted mutating method receiver operate on its original storage
  place.
- Make receiver mutability a lowering-owned convention shared by method
  signature emission, call validation, HIR flow effects, local mutability
  analysis, and codegen.
- Use one structural place model for mutable method receivers and `mut`
  arguments.
- Reject unsupported or conflicting mutable places during Sifr checking with
  stable Sifr diagnostics.
- Preserve existing value/clone behavior for ordinary field reads and
  shared-receiver method calls.
- Remove the ambient clone-suppression counter and its duplicated arm/disarm
  logic.

## Non-goals

- Do not add general reference values, user-visible lifetimes, `RefCell`,
  `Mutex`, or hidden runtime borrowing.
- Do not add mutable indexing, mutable slicing, or optional/recursive
  projection support in this fix.
- Do not require users to write `mut self` for ordinary inferred-mutating
  instance methods.
- Do not change ordinary field-read value semantics.
- Do not treat callable fields as instance methods.
- Do not add a fallback that guesses receiver mutability in codegen when
  lowering metadata is absent.

## Semantic decisions

### 1. Canonical receiver convention

Add a dedicated `ReceiverConvention` with exactly these modes:

- `SharedBorrow`
- `MutableBorrow`
- `Owned`

It is distinct from `ParamConvention`: an instance receiver is not an ordinary
positional parameter, and static/class methods have no instance receiver.

The convention is carried by:

- regular instance-method signature metadata in `FunctionType`;
- each regular `HirFunction`;
- each `HirExpr::MethodCall`.

`FunctionType.receiver` is `None` for free functions, static methods, class
methods, constructors, and callable fields. It is `Some(...)` for regular
instance methods. Declaration-first Python methods that consume their receiver
use `Owned`; user-defined Sifr instance methods use `SharedBorrow` or
`MutableBorrow`.

Normalize every class, protocol, enum, imported, and synthesized instance
method signature so `self` is never present in `FunctionType.params`.
`FunctionType.receiver` is the only receiver slot. Remove the current
`self_offset`/conditional-skip behavior from method-parameter consumers,
including registry argument lowering and mutable-argument validation.

For source compatibility, user-defined `own self` maps to `SharedBorrow` and
`own mut self` maps to `MutableBorrow`; neither consumes the user-class
instance. `Owned` remains reserved for method surfaces with an existing
semantic consume contract, such as declaration-first Python consuming methods.

Method-call HIR also retains the receiver, call, and argument source ranges
needed for ownership diagnostics. Compiler-synthesized calls may have no source
site, but all source-originated calls must have one. The completed HIR verifier
must reject a regular source method call with missing/unresolved receiver
convention as an internal compiler error; codegen must not default it.

Add the invariant check as a focused lowering post-pass before
`LoweringResult` construction. Missing convention/range metadata for a
source-originated resolved method call emits the centralized internal compiler
diagnostic through `LowerCtx`, preserving a precise source range without
allowing malformed HIR to reach codegen. Focused tests exercise the invariant
with compiler-authored malformed HIR. No new general HIR verifier framework is
assumed.

### 2. Receiver convention inference

Move user-class receiver inference out of
`crates/sifr_codegen/src/class_method_receiver_analysis.rs` and into a
lowering-owned fixed-point analysis that runs after class bodies are lowered
and before call-place validation or flow-graph construction.

Seed `MutableBorrow` for an instance method that:

- explicitly declares `mut self`;
- explicitly declares `own mut self`;
- assigns to or mutates a field rooted at `self`;
- invokes a canonical mutable builtin method on `self` or a `self` field;
- passes `self` or one of its projections to a `mut` parameter;
- calls another `MutableBorrow` method through `self` or a `self` projection.

Propagate those facts to a fixed point across direct delegation, class-field
method calls, generic specializations, and inheritance. Imported method
metadata and protocol signatures participate as already-declared facts.

Protocol methods without executable bodies must explicitly declare `mut self`
to expose `MutableBorrow`; otherwise they are `SharedBorrow`. Existing inferred
`self` mutability remains source-compatible: `self` is accepted as a mutable
root when the analysis infers `MutableBorrow`, even if `mut self` was omitted.

Receiver convention is part of protocol conformance:

- a `SharedBorrow` protocol method may only be implemented by a
  `SharedBorrow` class method;
- a `MutableBorrow` protocol method may be implemented by either
  `SharedBorrow` or `MutableBorrow`, because its `&mut self` bridge can call a
  less-demanding shared implementation;
- user-declared `Owned` protocol receivers remain unsupported.

If a class infers `MutableBorrow` for a method whose implemented protocol
declares `SharedBorrow`, checking emits `SIFR-PROTO-0005` before codegen.
Protocol trait and bridge signatures always use the protocol convention; the
delegated class method uses its own conforming convention.

Rust standard-trait dunders are a separate fixed-receiver contract. Their
`SharedBorrow` or `Owned` convention comes from the compiler's trait-bridge
registry, not body inference, because Rust fixes receiver shapes for
`PartialEq`, `PartialOrd`, `Add`, and `Display::fmt`. This includes
`__str__`/`__repr__`, whose bodies are inlined into a shared `Display`
receiver even though they bypass the arithmetic/comparison operator emitter. A
fixed-trait dunder body that attempts any receiver mutation is rejected during
receiver analysis with `SIFR-PROTO-0006`; this issue does not generate a
non-conforming `&mut self` trait signature or support mutation of a consumed
trait receiver. Operator and Display bridge emitters consume the fixed registry
convention after that check.

After convergence:

- write the convention to the `HirFunction` and exported/class `FunctionType`;
- resolve every `HirExpr::MethodCall` through nominal class identity, generic
  substitution, protocol method metadata, and the inheritance chain;
- attach the resolved convention to the call;
- use the same convention for Rust receiver signature emission and call-place
  validation.

Unknown/missing method receiver metadata is an internal compiler error after
ordinary unknown-method diagnostics have run. It must not become
`SharedBorrow`, `MutableBorrow`, or a codegen heuristic.

Run the fixed point inside `lower_module_bodies` after all class bodies are
lowered and before module-level functions are lowered. This ordering permits
the pass to:

- annotate and validate calls already present in class bodies;
- update `ctx.class_types` and qualified method signatures;
- make final receiver metadata available while module functions lower;
- annotate/validate module-function calls immediately from final metadata.

Receiver/method-argument flow effects are derived from the final annotated HIR
in `flow_graph/effects.rs`; they are not appended speculatively to
`ctx.flow_effects` before the fixed point. Remove or narrow the current inline
`record_flow_effect` calls for these method-call borrows so the snapshot-effect
tail does not disagree with HIR-derived effects.

### 2.1 Non-class receiver resolution

Every source method call still receives a convention, but the lookup rules are
type-specific:

- user class, protocol, enum, inherited, generic, and imported methods resolve
  from `FunctionType.receiver`;
- declaration-first Python/Rust opaque and resource methods resolve from their
  declaration/registry contract;
- builtin list/dict/set/buffer/iterator/handle methods resolve from one
  canonical `(resolved receiver type, method) -> ReceiverConvention` registry;
- a successfully type-checked non-class method with no mutable/consuming entry
  is defined as `SharedBorrow` and keeps today's value path.

The internal-error rule applies to a resolved class/protocol/declaration method
whose required metadata is missing, not to the deliberately defined
`SharedBorrow` default for unclassified non-class methods.

This issue applies checked-place validation to class/protocol mutable receivers
and to builtin/opaque mutable receivers that require mutation of caller-owned
storage. Mutating an owned rvalue temporary, such as `Helper().bump()` or a
fresh list temporary, remains valid and uses a separately proven
`OwnedTemporary` receiver target. It is not diagnosed as an unsupported
storage place and it must not be routed through field-read clone suppression.

### 3. Canonical mutable place

Introduce `BindingId` in `crates/sifr_lowering/src/scope.rs`. `Scope` assigns a
monotonic id from `define_binding`, stores it in `VarInfo`, retains immutable
binding facts after a frame is popped, and exposes the id through `lookup`.
Every lowered `HirExpr::Name` carries the resolved id. The retained facts
include binding kind, mutability, parameter convention, and source name, so the
post-class fixed point does not depend on a live function scope.

Extend `BindingKind` with `Receiver` and `EphemeralLocal(origin)`. Define
regular-method `self` through `Receiver` rather than as today's generic local.
Classify only `for`/comprehension element targets and `match` case captures as
`EphemeralLocal` with their origin instead of the generic stable-local path.
`with` targets, exception-handler targets, ordinary tuple-unpack targets, and
chained-assignment targets remain stable owned locals; existing idioms such as
`with open(...) as f: f.write(...)` must remain accepted. After fixed-point
inference, patch the retained receiver fact to `SharedBorrow` or
`MutableBorrow`; root eligibility tests the binding kind plus final convention,
never the literal name `"self"`.

Add a lowering-owned place descriptor used by both receiver and argument
validation:

```text
Place {
  root: BindingId,
  projections: [Field(field_identity), ...]
}
```

`BindingId`, not source spelling alone, determines root equality.
`field_identity` is the resolved declaring-class/field identity, so inherited
storage re-rooting does not change semantic place identity.

`Type::Class.fields` does not currently store declaring-class identity. Resolve
`field_identity` during place extraction by walking the nominal receiver class
and its `parent_class` chain in `ctx.class_types`; do not infer it from the
generated Rust `.base` spelling.

Accepted mutable places in this issue are:

- a stable owned local binding introduced by `let`/assignment;
- a mutable borrowed parameter;
- an `own mut` parameter;
- `self` in an enclosing method whose inferred convention is
  `MutableBorrow`;
- zero or more non-optional, non-recursive field projections from one of those
  roots.

An immutable borrowed parameter and an `own` parameter without `mut` are not
mutable roots. They are rejected with `SIFR-OWN-0005`.

The following are deliberately unsupported mutable places in this issue:

- index projections;
- slice projections;
- projections through optional or recursive storage, including a narrowed
  `Option[Box[T]]`;
- callable fields;
- ephemeral loop/comprehension element and `match` capture bindings.

When a resolved `MutableBorrow` method uses one of those receiver shapes,
checking emits `SIFR-OWN-0014`; codegen is not invoked. Optional values that
have not been narrowed continue to receive the earlier ordinary
unknown/optional-method type diagnostic. A narrowed optional/recursive storage
place reaches the receiver-place validator and receives `SIFR-OWN-0014`.

If index, slice, or optional-field typing rejects the expression earlier with
`SIFR-STDLIB-0001`, that existing diagnostic is the accepted boundary; this
issue does not bypass ordinary type resolution merely to replace it with
`SIFR-OWN-0014`. `SIFR-OWN-0014` is required only for an otherwise
successfully resolved mutable method call whose target is still not a proven
place, such as a narrowed local/parameter optional receiver.

Owned constructor/call-result/conditional temporaries are represented as
`MutableReceiverTarget::OwnedTemporary`, not `Place`. They use direct rvalue
emission, remain valid, and never participate in binding-place overlap.

An otherwise resolved mutable method call rooted at `EphemeralLocal` reports
`SIFR-OWN-0014`. This issue does not add `for mut`/mutable match-capture
emission or change Python-style loop-element copy semantics; that can be
designed separately without leaking a Rust `E0596`.

### 4. Place overlap and same-call exclusivity

Two places overlap when they have the same `BindingId` root and either field
projection sequence is a prefix of the other. Equal places overlap. Different
roots and sibling field paths do not overlap.

Examples:

- `self.helper` overlaps `self.helper`;
- `self.helper` overlaps `self.helper.items`;
- `self` overlaps every `self.*` place;
- `self.helper` and `self.other` are disjoint;
- `self.helper` and `self.stock` are disjoint.

For a `MutableBorrow` receiver place `P`, inspect the complete read/borrow/move
footprint of every explicit argument, including nested calls and field reads:

- an overlapping shared read/borrow is rejected with `SIFR-OWN-0002`;
- an overlapping mutable borrow is rejected with `SIFR-OWN-0002`;
- an overlapping owned move is rejected with `SIFR-OWN-0002`;
- a disjoint sibling place under the same root is accepted;
- a non-place expression that contains an overlapping place read is rejected;
- an unsupported/dynamic projection under the same root is conservatively
  treated as overlapping.

The same place-overlap function replaces the existing bare-name-only
same-call check for regular functions and is also used for method arguments.
This closes receiver-versus-argument, argument-versus-argument, bare-root, and
field-projection conflicts with one rule.

This issue intentionally rejects `self.helper.read(self.helper)` rather than
depending on argument auto-cloning or Rust two-phase-borrow behavior. It accepts
`self.helper.read(self.other)` when `helper` and `other` are resolved sibling
fields.

### 5. Code generation

Add one `place_emitter` entry point that emits a checked `Place` in
`MutableBorrow` access mode. It recursively emits every root-to-leaf field hop
without calling the ordinary value/clone field-read path.

The place emitter must:

- preserve all field hops for nested receivers;
- compose with inherited parent-storage re-rooting;
- emit the original root for mutable borrowed and `own mut` parameters;
- contain no `.clone()`, `.cloned()`, `take()`, or temporary materialization;
- reject any HIR/place shape that lowering did not prove rather than falling
  back to ordinary expression lowering.

All generic method-call emitters select their receiver path only from
`HirExpr::MethodCall.receiver_convention`:

- `MutableBorrow` uses the checked place emitter;
- `SharedBorrow` uses a structural shared-receiver borrow path, preserving the
  pre-existing non-cloning behavior of calls such as `self.items.len()`
  without acquiring mutable-place authority;
- `Owned` uses the existing consuming receiver path.

For `MutableReceiverTarget::OwnedTemporary`, emit the already-proved owned
rvalue directly and allow Rust to borrow that temporary for the call. This is a
separate explicit branch, not fallback from a failed place emission.

Mutable method arguments use the same checked place emitter based on their
`ParamConvention`. This replaces, rather than coexists with:

- `pending_self_field_clone_suppression`;
- `method_call_needs_field_clone_suppression`;
- `method_mut_arg_needs_field_clone_suppression`;
- every manual suppression-counter arm/disarm site.

Special intrinsic/stdlib method emitters must either consume the same annotated
receiver convention/place or prove that they fully own their distinct
semantics. No generic mutating receiver may bypass the checked place entry
point.

Every Rust instance-receiver signature site must consume
`HirFunction.receiver`, including regular class emission, protocol trait
signatures, protocol implementation bridges, and enum/newtype type emitters.
Rust standard operator and Display bridges consume their fixed trait-registry
convention after `SIFR-PROTO-0006` validation. Delete the independent
`body_contains_field_assign_codegen` and hard-coded immutable-self decisions at
those sites.

Shared receiver calls borrow their original receiver storage and standalone
field-value reads retain the ordinary clone path. For example, a read-only
`self.helper.peek()` calls `peek()` on `self.helper` without cloning the
receiver, while `value = self.helper` remains an ordinary value read; the
enclosing method is emitted as `&self`, not `&mut self`.

### 6. HIR flow and Rust local mutability

HIR mutation collection uses `receiver_convention == MutableBorrow` and the
canonical place root. It no longer treats every method call on a class-typed
field as mutating.

Consequences:

- an owned local root of an accepted mutable receiver is emitted as a mutable
  Rust local;
- an enclosing `self` method becomes `&mut self` only when its canonical
  receiver analysis says so;
- a shared receiver call does not create a mutation effect;
- aliases and generic/inherited methods do not depend on method-name string
  heuristics.

Consolidate the duplicated builtin mutating-method tables currently present in
HIR analysis and Rust IR optimization behind one canonical
type-and-method-to-`ReceiverConvention` query. HIR consumers use attached call
metadata.

The late Rust IR mutability optimizer is untyped and cannot call that query.
Before optimization, codegen records the Rust local names required mutable by
checked `MutableBorrow` place emission. Pass that protected-local set into
`remove_unneeded_mutability_in_items`; it may demote other locals but must not
demote a protected mutable-place root. The optimizer's current method-name
mutation table is then limited to compiler-generated Rust patterns that lack
HIR provenance and is exported from one module rather than duplicated. User
class receiver mutability never depends on that string table. Run the same pass
in production and test-module assembly.

## Implementation shape

### Item 1: Canonical receiver metadata and inference

Status: **Merged** in
[#3065](https://github.com/sifr-lang/sifr/pull/3065). Claude Opus review pass 1
returned `NOT SATISFIED`; all material findings were addressed, and pass 2
returned `SATISFIED`. The create-PR gate passed with 131/131 E2E fixtures.

1. Add `BindingId`/retained binding facts to `Scope`/`VarInfo`, attach ids to
   lowered names, and capture source ranges while each function scope is live.
2. Add `ReceiverConvention`, normalize `self` out of `FunctionType.params`,
   and add the optional receiver field to type-system method signatures.
3. Extend `HirFunction` and `HirExpr::MethodCall` with resolved receiver
   convention and source-site metadata.
4. Split `lower_module_bodies` at the class/function boundary; add the
   lowering-owned fixed-point analysis and inherited/protocol/generic lookup.
5. Make every class/protocol/type/operator Rust receiver signature consume the
   HIR convention.
6. Make HIR/flow mutation collection consume call metadata, centralize builtin
   receiver conventions, and add the lowering invariant check.
7. Add focused scope-id, type-system, lowering, HIR snapshot, inheritance,
   protocol, opaque/builtin, and codegen signature tests.
8. Run the focused validation and `scripts/run_all_tests.sh --profile
   create-pr`; open, independently review, and merge this metadata-only PR.
9. Update this issue with the merged PR link and checklist state before Item 2.

Item 1 is independently compilable because all `FunctionType` constructors and
method-parameter consumers migrate atomically, all source calls leave lowering
with a convention, and existing codegen behavior may continue to use the
ambient field-clone counter until Item 2. No user-visible receiver-place fix or
counter deletion is claimed by Item 1.

### Item 2: Place checking, place emission, and defect closure

1. Add the canonical `Place`/projection extractor, argument footprint
   collector, prefix-overlap check, and receiver/argument validation.
2. Add `SIFR-OWN-0014`, `SIFR-PROTO-0005`, and `SIFR-PROTO-0006`; reuse
   `SIFR-OWN-0002`/`0005` exactly as specified below.
3. Add the codegen place emitter, route mutable receivers and mutable arguments
   through it, and delete all ambient suppression state/helpers/sites.
4. Add native pass/fail fixtures, focused lowering/codegen tests, emitted-Rust
   no-clone assertions, and regression coverage for every matrix row below.
5. Run focused validation and the authoritative merge gate
   `scripts/run_all_tests.sh`; open, independently review, and merge the fix PR.
6. Update this issue, `internal_docs/architecture.md`,
   `internal_docs/diagnostic_codes.md`, and the roadmap/phase status if the
   issue is linked from one.

The issue closes only after Item 2 is merged and the final independent review
finds no alternate clone or unchecked mutation path.

## Primary implementation anchors and decomposition

Expected existing anchors:

- `crates/sifr_type_system/src/types/definitions.rs`
- `crates/sifr_ir/src/hir_nodes.rs`
- `crates/sifr_lowering/src/scope.rs`
- `crates/sifr_lowering/src/flow_graph/effects.rs`
- `crates/sifr_lowering/src/lower/module_body_lowering.rs`
- `crates/sifr_lowering/src/lower/classes/class_type_collection.rs`
- `crates/sifr_lowering/src/lower/classes/class_body_lowering.rs`
- `crates/sifr_lowering/src/lower/statements/control_flow.rs`
- `crates/sifr_lowering/src/lower/expressions/methods_lambdas_and_comprehensions.rs`
- `crates/sifr_lowering/src/lower/mutating_methods.rs`
- `crates/sifr_lowering/src/lower/expressions/regular_calls.rs`
- `crates/sifr_lowering/src/lower/statements/patterns_and_assignments.rs`
- `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs`
- `crates/sifr_codegen/src/class_method_emitter.rs`
- `crates/sifr_codegen/src/class_emitter.rs`
- `crates/sifr_codegen/src/lib_emitter_state.rs`
- `crates/sifr_codegen/src/type_emitters.rs`
- `crates/sifr_codegen/src/operator_protocol_emitters.rs`
- `crates/sifr_codegen/src/expr_render_helpers/field_and_stdlib_rewrites.rs`
- `crates/sifr_codegen/src/expr_render_helpers/operator_rewrites.rs`
- `crates/sifr_codegen/src/expr_render_helpers/tests.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters/plain_call_args.rs`
- `crates/sifr_codegen/src/intrinsic_method_emitters/recursive_exprs.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/print_calls.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/subscript_augassign_delete.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_method_and_question_mark.rs`
- `crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_wrappers_and_compare.rs`
- `crates/sifr_codegen/src/string_char_cache.rs`
- `crates/sifr_codegen/src/hir_analysis/queries/queries_impl.rs`
- `crates/sifr_codegen/src/ir_optimize/mutability_and_clone_rewrites.rs`
- `crates/sifr_codegen/src/entrypoints.rs`

Before adding code, split by responsibility:

- move receiver inference from codegen into a focused lowering module;
- add a focused lowering place-analysis module;
- add a focused codegen place-emitter module;
- extract class method body/scope setup from `class_body_lowering.rs`;
- extract target-binding definition helpers from
  `statements/control_flow.rs` and `statements/statement_dispatch.rs`;
- extract method-call construction/resolution from
  `methods_lambdas_and_comprehensions.rs`;
- extract mutation/root collection from `queries_impl.rs`;
- extract protocol bridge generation from `operator_protocol_emitters.rs`;
- extract recursive registry method-call lowering from `recursive_exprs.rs`;
- extract wrapper/comparison call lowering from
  `stmt_expr_wrappers_and_compare.rs`;
- extract field-storage re-rooting helpers from
  `field_and_stdlib_rewrites.rs` for reuse by value and place emission;
- split `mutability_and_clone_rewrites.rs` before modifying it if it remains an
  implementation target.

At the time of planning, `field_and_stdlib_rewrites.rs` is 869 lines and
`mutability_and_clone_rewrites.rs` is 882 lines,
`class_body_lowering.rs` is 896 lines, `recursive_exprs.rs` is 882 lines,
`methods_lambdas_and_comprehensions.rs` is 867 lines, `queries_impl.rs` is 866
lines, `stmt_expr_wrappers_and_compare.rs` is 842 lines, and
`operator_protocol_emitters.rs` is 881 lines,
`statements/control_flow.rs` is 897 lines, and
`statements/statement_dispatch.rs` is 895 lines. Each named split must land
before or in the first PR that otherwise grows that file; none may cross the
hand-maintained 900-line cap.

## Diagnostics

### Existing codes

- `SIFR-OWN-0002`: use for every same-call overlapping place conflict.
  Populate the existing `binding` argument with the canonical source display
  of the conflicting place and point at the later conflicting receiver or
  argument. Regenerate/update `docs/errors/SIFR-OWN-0002.mdx` so its documented
  scope explicitly includes shared reads and owned moves that overlap a
  mutable receiver/argument field place, not only duplicate bare-name mutable
  borrows.
- `SIFR-OWN-0005`: use when mutation would traverse an immutable borrowed
  parameter or an `own` parameter lacking `mut`. Populate `binding` with the
  root binding and point at the receiver expression.

### New code

Reserve `SIFR-OWN-0014`:

- title: `Unsupported mutable receiver place.`
- severity/category: `Error` / `OWN`
- owner: `sifr_lowering::lower::method_receiver_places`
- template: `mutable method receiver {place} is not a supported storage place`
- message+JSON argument: `place`
- representative fixture:
  `crates/sifr/tests/e2e/fail/unsupported_narrowed_optional_mutating_receiver.sifr`

Reserve `SIFR-PROTO-0005`:

- title: `Protocol receiver convention mismatch.`
- severity/category: `Error` / `PROTO`
- owner: `sifr_lowering::lower::method_receiver_analysis`
- template:
  `class '{class_name}' method '{method}' requires a mutable receiver but protocol '{protocol}' declares a shared receiver`
- message+JSON arguments: `class_name`, `method`, `protocol`
- representative fixture:
  `crates/sifr/tests/e2e/fail/protocol_receiver_mutability_mismatch.sifr`

Reserve `SIFR-PROTO-0006`:

- title: `Fixed Rust trait receiver mutation is unsupported.`
- severity/category: `Error` / `PROTO`
- owner: `sifr_lowering::lower::method_receiver_analysis`
- template:
  `method '{method}' cannot mutate its receiver because Rust trait '{trait_name}' fixes the receiver convention`
- message+JSON arguments: `method`, `trait_name`
- representative fixture:
  `crates/sifr/tests/e2e/fail/operator_receiver_mutation_rejected.sifr`

Add the registry constant and active entry, generate the error page/navigation
with:

```bash
cargo run -p sifr_diagnostics --bin gen-error-docs
```

Update `internal_docs/diagnostic_codes.md`, and validate the generated/docs
contract with:

```bash
python3 scripts/check_docs_error_code_links.py
```

## Regression matrix

### Native pass fixtures

Add focused fixtures under `crates/sifr/tests/e2e/pass/` that assert observable
post-call state and, where applicable, returned values:

- direct `self.helper.bump()`;
- nested `self.mid.inner.bump()`;
- mutable borrowed-parameter field receiver;
- `own mut` parameter field receiver;
- owned-local field receiver;
- inherited field receiver;
- dual disjoint mutation: mutable receiver plus `mut` sibling argument;
- shared sibling read under the same root;
- generic class method receiver;
- explicitly mutable protocol method receiver through a conforming class,
  exercised with the currently supported `own mut entity: Protocol` parameter
  shape rather than a borrowed trait-object coercion; the protocol also
  declares a getter and the consuming helper returns the post-mutation value so
  the caller can assert the mutation without reusing the moved argument;
- owned temporary class and builtin mutable receivers, proving they remain
  accepted without becoming storage places;
- mutable receiver calls on stable `with`, exception-handler, tuple-unpack, and
  chained-assignment locals, including the existing `open()`/`write()` idiom;
- non-mutating operator and `__str__`/`__repr__` dunders through their fixed
  Rust trait receivers;
- mutating calls in expression statement, assignment/RHS, return, print/
  nested-expression, loop, and `?`/Result contexts where syntactically valid;
- a read-only class-field receiver proving the enclosing method stays `&self`
  and ordinary read cloning remains unchanged.

At least one fixture must extend
`class_method_mut_borrowed_field_argument.sifr` or supersede it so the same call
asserts both receiver-side and mutable-argument mutation.

### Check-fail fixtures

Add one stable `# expect-error[col=N]: CODE` fixture per diagnostic/shape:

- immutable borrowed-parameter field receiver: `SIFR-OWN-0005`;
- immutable owned-parameter field receiver: `SIFR-OWN-0005`;
- equal receiver/argument place: `SIFR-OWN-0002`;
- receiver-prefix argument read: `SIFR-OWN-0002`;
- argument-prefix receiver: `SIFR-OWN-0002`;
- overlapping receiver and `mut` argument: `SIFR-OWN-0002`;
- nested argument expression that reads the receiver place:
  `SIFR-OWN-0002`;
- narrowed local/parameter optional receiver: `SIFR-OWN-0014`;
- loop/comprehension element and `match` capture mutable receivers:
  `SIFR-OWN-0014`;
- mutable class implementation of a shared-receiver protocol method:
  `SIFR-PROTO-0005`;
- mutating operator dunder receiver: `SIFR-PROTO-0006`;
- mutating `__str__`/`__repr__` Display receiver: `SIFR-PROTO-0006`.

Add positive protocol conformance controls for shared protocol/shared
implementation, mutable protocol/mutable implementation, and mutable
protocol/shared implementation.

Also retain or add boundary fixtures proving indexed, sliced, and un-narrowed
optional-field class receivers fail earlier with the existing
`SIFR-STDLIB-0001`. These are not `SIFR-OWN-0014` representative shapes until
ordinary type resolution can produce a resolved mutable method call for them.
If a future narrowing path makes optional/recursive field storage reachable,
the focused lowering unit test must prove that the place validator returns
`SIFR-OWN-0014`.

Include a positive sibling-field control beside the overlap tests so the
checker cannot regress to root-only rejection.

### Focused unit and snapshot tests

- `sifr_type_system`: receiver metadata construction and substitution.
- `sifr_ir`/HIR snapshots: every source method call has a resolved convention
  and source ranges.
- `sifr_lowering`: inference fixed point, delegation, inheritance, protocol/
  generic resolution, root eligibility, place extraction, footprint
  collection, prefix overlap, and exact diagnostics.
- `sifr_codegen`: place emission for direct/nested/inherited roots; every
  generic expression context; shared receiver/value-read preservation; mutable
  local inference; absence of ambient suppression state.
- Refresh and review emitted-Rust snapshots affected by narrowing read-only
  class methods from `&mut self` to `&self`; treat unexpected mutation changes
  as regressions rather than mechanical snapshot acceptance.
- emitted-Rust assertions must reject `.clone()`, `.cloned()`, `take()`, or a
  temporary anywhere between the accepted storage root and mutating call.
- a repository search test/review must show no remaining
  `pending_self_field_clone_suppression`,
  `method_call_needs_field_clone_suppression`, or
  `method_mut_arg_needs_field_clone_suppression`.

## Validation

Run focused checks during each item:

```bash
cargo test -p sifr_type_system
cargo test -p sifr_ir
cargo test -p sifr_lowering receiver
cargo test -p sifr_lowering place
cargo test -p sifr_codegen receiver
cargo test -p sifr_codegen place
cargo run -q -p sifr -- check <each-new-fail-fixture>
cargo run -q -p sifr -- emit <representative-pass-fixture>
cargo run -q -p sifr -- run <each-new-pass-fixture>
python3 scripts/check_docs_error_code_links.py
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_file_size_guardrails.py
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

Before each PR:

```bash
scripts/run_all_tests.sh --profile create-pr
```

Before final merge/closure:

```bash
scripts/run_all_tests.sh
```

CI is confirmatory; do not wait on CI instead of running the local gates.

Current Item 2 validation evidence:

- focused lowering, codegen, scope, optimizer, pass-fixture, and fail-fixture
  checks pass;
- full lowering tests pass (`922 passed`, `1 ignored`), full codegen tests pass
  (`941 passed`), the E2E pass suite passes (`680/680`, report signature
  `8871ba51135353a4`), and the E2E fail test
  passes with the complete annotated fail corpus;
- formatting, workspace clippy with warnings denied, HIR maintainability,
  file-size, diagnostic-doc links, and diff checks pass;
- the representative performance suite has an official isolated green run
  (`8/8`) with `CARGO_BUILD_JOBS=1`;
- merge-profile pass 13 on exact head
  `31af48ac8935b869cce4369f5c0c939c5c5b076f` passes every pre-performance
  functional lane, including core/diagnostic guardrails, CPython differential,
  all 25 Python interop variants, Rust interop, frontend guardrails, and all 32
  developer-tooling variants;
- whole-profile performance attempts were scheduler-contended by independently
  running local compiler, E2E, release, and benchmark jobs. The three affected
  exact-head cases were therefore remeasured in short uncontended windows and
  checked with the repository's official `check_budgets.py --allow-subset`
  gate: project check `1339.235ms < 1357.524ms` (`5` samples), arithmetic
  check `1328.513ms < 1334.139ms` (`5` samples), and JSON diagnostics
  `1317.663ms < 1335.954ms` (`5` samples). The official subset checker passes.
  No budget, baseline, sample count, threshold, or waiver was changed.

## Acceptance criteria

- Every accepted `MutableBorrow` receiver and `mut` argument is lowered through
  the same checked place emitter.
- Direct, nested, inherited, local, `self`, mutable-borrowed, and `own mut`
  field receiver mutations are observable after the call.
- Emitted Rust contains no clone or temporary on any accepted mutable receiver
  root-to-leaf path.
- Receiver convention inference is lowering-owned, fixed-point closed, stored
  in signatures/HIR, and is the sole source for Rust `self` signature,
  ownership effects, place validation, and receiver codegen.
- `FunctionType.params` excludes `self` everywhere, and every class, protocol,
  type, and operator-protocol receiver signature consumes the canonical
  convention.
- Resolved `BindingId` and retained binding facts make place equality and root
  eligibility independent of popped lowering scopes.
- Shared-receiver calls do not spuriously make the enclosing method mutable,
  and ordinary field reads retain existing clone semantics.
- Place-prefix conflict analysis accepts sibling fields and rejects every
  overlapping receiver/argument/read/move shape at Sifr check time.
- Immutable parameter roots report `SIFR-OWN-0005`; overlapping places report
  `SIFR-OWN-0002`; unsupported mutable receiver shapes report
  `SIFR-OWN-0014`.
- Resolved narrowed optional/recursive storage targets report
  `SIFR-OWN-0014`; indexed/sliced/un-narrowed optional shapes that fail ordinary
  resolution retain `SIFR-STDLIB-0001`; proven owned temporaries remain valid.
- Ephemeral loop/comprehension elements and `match` captures report
  `SIFR-OWN-0014`, while stable `with`, exception, tuple-unpack, and
  chained-assignment locals retain accepted mutable-receiver behavior.
- Protocol receiver variance follows the declared conformance rule, and a
  mutable implementation of a shared protocol method reports
  `SIFR-PROTO-0005`.
- Fixed-receiver operator and Display dunders reject receiver mutation with
  `SIFR-PROTO-0006` instead of emitting a non-conforming Rust trait method.
- Builtin, opaque/resource, imported, inherited, generic, and protocol method
  receivers all follow the explicit resolution rules without codegen guessing.
- The ambient clone-suppression counter, its helpers, and every arm/disarm site
  are deleted.
- Duplicated mutating-method classification is removed or routed through one
  canonical receiver-convention query, and the untyped Rust IR mutability pass
  preserves roots recorded by checked place emission.
- Focused tests, E2E pass/fail fixtures, diagnostic docs checks, HIR/file-size
  guardrails, formatting, clippy, the create-PR gate, and the authoritative
  merge gate pass.
- Final independent review finds no silent clone, unchecked receiver path,
  root-only over-rejection, or unresolved implementation choice.

## Review ledger

- M10 originating review:
  [`ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-7.md`](../../reviews/active/ad-hoc-declaration-first-python-interop-m10-milestone-fable-high-review-pass-7.md)
  identified the silent receiver clone and adjacent same-call/index parity
  gaps.
- Claude Opus implementation-readiness pass 2:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-2.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-2.md)
  returned `NOT SATISFIED`; this revision addresses its 15 material findings.
- Claude Opus implementation-readiness pass 3:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-3.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-3.md)
  returned `NOT SATISFIED`; the next revision added concrete binding identity
  and scope-fact retention, class/function pass ordering, non-class resolution,
  protected local mutability, protocol/type emitter coverage, reachable
  diagnostic fixtures, normalized receiver parameter alignment, and complete
  near-cap decomposition.
- Claude Opus implementation-readiness pass 4:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-4.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-4.md)
  returned `NOT SATISFIED`; the next revision made receiver convention part of
  protocol conformance with `SIFR-PROTO-0005`, classified ephemeral bindings
  as unsupported mutable roots with `SIFR-OWN-0014`, completed counter-site
  ownership, corrected internal-diagnostic ownership, and budgeted read-only
  receiver snapshot churn.
- Claude Opus implementation-readiness pass 5:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-5.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-5.md)
  returned `NOT SATISFIED`; the next revision preserved stable `with`,
  exception, tuple-unpack, and chained-assignment locals, narrowed ephemeral
  rejection to iteration/comprehension elements and match captures, and added
  the fixed-operator receiver contract with `SIFR-PROTO-0006`.
- Claude Opus implementation-readiness pass 6:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-6.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-6.md)
  returned `NOT SATISFIED`; the next revision extended the fixed Rust trait
  receiver contract and `SIFR-PROTO-0006` to `__str__`/`__repr__` Display
  bridges and added their class-emitter/test coverage.
- Claude Opus implementation-readiness pass 7:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-7.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-claude-opus-review-pass-7.md)
  returned `SATISFIED`; no material semantic/design ambiguity, infeasible
  sequencing, silent fallback, unchecked fixed-receiver or method-call path,
  diagnostic mismatch, or acceptance/test contradiction remains.
- Item 1 implementation review pass 1:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item1-claude-opus-review-pass-1.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item1-claude-opus-review-pass-1.md)
  returned `NOT SATISFIED`; the implementation was corrected for owned-local
  clone preservation, source-range alignment, non-class receiver contracts,
  protocol consistency, final HIR verification, and missing coverage.
- Item 1 implementation review pass 2:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item1-claude-opus-review-pass-2.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item1-claude-opus-review-pass-2.md)
  returned `SATISFIED`; Item 1 merged in
  [#3065](https://github.com/sifr-lang/sifr/pull/3065).
- Item 2 implementation review pass 1:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-1.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-1.md)
  returned `NOT SATISFIED`; remediation addresses shared-receiver clone
  regressions, transitive fixed-trait mutation, conditional and chained owned
  temporaries, full-gate evidence, operator protocol traversal, mutable borrow
  flow effects, optimizer fallback scope, tracking state, and the retained
  plain file-handle fixture.
- Item 2 implementation review pass 2:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-2.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-2.md)
  returned `NOT SATISFIED`; remediation closes shared-receiver/mutable-argument
  overlap, rejects re-materialized module constants as mutable roots, accepts
  fresh slice temporaries while explicitly rejecting walrus bindings, deletes
  the final dead codegen receiver helper, documents guarded indexed-storage
  behavior, restores production optimizer execution and compiler-generated
  fallback coverage, and requires an uncontended green authoritative gate.
- The fail-suite CFG panic-hook observation from review pass 2 is pre-existing:
  the exact detached base `b3495318dc59a79c678fe874619f993fed5deb4b`
  emits the same two `cfg.rs:300` incomplete-branch panics while its 537-fixture
  fail lane exits successfully. Item 2 neither introduced nor widened that
  masked internal error; it remains separate compiler-health debt.
- Item 2 implementation review pass 3:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-3.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-3.md)
  returned `NOT SATISFIED`; remediation makes constructor `self` a checked
  fresh mutable root and materializes a synthetic post-initialization Rust
  instance, ships the LRU migration through upstream corpus
  [#40](https://github.com/sifr-lang/leetcode/pull/40), pins the
  `append(len(...))` snapshot boundary and user guidance, adds string-literal
  membership narrowing to the already-live plain-dict indexed-storage path,
  and restores the full compiler-generated optimizer fallback union with
  unprotected `write`/`append` coverage.
- LRU corpus milestone review pass 1:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-lru-corpus-pr-40-claude-opus-review-pass-1.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-lru-corpus-pr-40-claude-opus-review-pass-1.md)
  returned `SATISFIED`; the exact reviewed head merged in
  [sifr-lang/leetcode#40](https://github.com/sifr-lang/leetcode/pull/40), and
  this Item 2 tree records merged corpus pointer `7772857`.
- Item 2 implementation review pass 4:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-4.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-4.md)
  returned `NOT SATISFIED`; remediation replaces the constructor statement
  partition with a source-order materialization boundary, evaluates field
  initializers at their source positions, preserves dependencies and effects
  across that boundary, structurally rewrites statement-carried `self` storage
  roots as well as expression roots, and rejects receiver use before complete
  own/inherited storage with check-time `SIFR-OWN-0014`.
- Item 2 implementation review pass 5:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-5.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-review-pass-5.md)
  returned `SATISFIED`; both pass-4 blocking findings are closed, all claimed
  validation evidence was independently reproduced, and no material
  correctness, ownership, constructor-initialization, codegen, optimizer,
  diagnostic, submodule, or test gap remains.
- Item 2 exact-head PR review pass 6:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-6.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-6.md)
  returned `NOT SATISFIED`; remediation corrects the validation ledger and
  sample counts, gives constructor `SIFR-OWN-0014` a structured `place=self`
  argument with source-facing field/parent guidance and the first offending
  statement span, and keeps same-named constructor parameters available as
  materialization seeds even when an explicit field assignment appears later.
- Item 2 exact-head PR review pass 7:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-7.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-7.md)
  returned `NOT SATISFIED`; pass-6 findings 1 and 2 were independently closed,
  but the parameter-seed remediation had also removed explicit-initializer
  deduplication. The follow-up keeps parameter seeds and first explicit
  initializers as separate facts, so a repeated field assignment before
  complete storage now reports check-time `SIFR-OWN-0014` instead of leaking
  Rust `E0063`, with focused lowering and annotated fail-fixture coverage.
