# Ad Hoc Issue: Class-Field Mutating Receiver Place Semantics

## Status

Implementation closure is in progress under
[#3088](https://github.com/sifr-lang/sifr/pull/3088). Item 1, canonical receiver
metadata and inference, merged in
[#3065](https://github.com/sifr-lang/sifr/pull/3065), with tracking follow-up
[#3066](https://github.com/sifr-lang/sifr/pull/3066). Item 2, checked place
semantics and defect closure, merged in
[#3082](https://github.com/sifr-lang/sifr/pull/3082) after terminal exact-head
review pass 12 returned `SATISFIED` with zero actionable findings. Upstream
compatibility snapshots merged in
[sifr-lang/leetcode#40](https://github.com/sifr-lang/leetcode/pull/40) and
[sifr-lang/leetcode#41](https://github.com/sifr-lang/leetcode/pull/41).

The first whole-phase review found missing structured `binding` arguments on
`SIFR-OWN-0002` diagnostics and duplicated inherited-field storage rerooting.
Both were closed in [#3087](https://github.com/sifr-lang/sifr/pull/3087), whose
exact-head Claude Opus review returned `SATISFIED` with zero actionable
findings. A later whole-phase review found that unsupported callable and
recursive field values could still bypass the argument-footprint overlap
check. The correction merged in
[#3090](https://github.com/sifr-lang/sifr/pull/3090) as
`44ab8ad38544fa5225d8d4f09ad3b5026d485c25`; it gives all five
`SIFR-OWN-0002` paths structured arguments, adds every native phase pass
fixture to the create-PR manifest, and preserves precise field identity for
statically resolvable unsupported field values. Its exact-head review pass 5
returned `SATISFIED` with no blocking or non-blocking findings.

Terminal whole-phase review pass 3 then found one narrower over-rejection:
invoking a callable class field inside a same-call argument recorded only the
field's parent object place. The focused correction is under draft PR
[#3092](https://github.com/sifr-lang/sifr/pull/3092); it must pass the
authoritative create-PR gate, repeated exact-head Opus review, and merge before
the integrated closure can receive a new terminal whole-phase review. Closure
PR [#3088](https://github.com/sifr-lang/sifr/pull/3088) remains draft until
that review returns `SATISFIED` and the final merge gate evidence is complete.

The defect predates M10 and was not introduced by the buffer implementation,
but it violates Sifr's core guarantee: a program can compile and silently lose
a source-visible mutation. The closure PR is not ready to merge until the
review ledger below ends in an independent whole-phase `SATISFIED` verdict.

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

Before this phase, the compiler could emit `self.helper.clone().bump()`. The
call mutated a temporary clone, not `self.helper`, and the program observed no
mutation.

The first independent review also reproduced these adjacent failures:

- `self.mid.inner.bump()` clones an intermediate field hop;
- an inherited field receiver can clone after parent-storage re-rooting;
- `box.helper.bump()` silently loses mutation even when `box` is a mutable
  borrowed parameter;
- the immutable-parameter form also compiles and silently loses mutation;
- `self.helper.absorb(self.stock)` preserves the mutable argument mutation but
  loses the receiver mutation.

The pre-phase code contained a partial, ambient workaround:

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

“Unsupported mutable place” applies to selecting the mutating receiver or a
mutable argument; it does not discard statically resolved field identity while
collecting another argument's read/borrow/move footprint. A callable or
optional/recursive field value whose base is a known place retains its
declaring-class field projection for overlap comparison. Only genuinely
unresolvable bases and dynamic index/slice projections collapse to a
conservative root footprint.

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
- a statically resolvable unsupported field value retains precise field
  identity, so a disjoint sibling remains accepted;
- a dynamic index/slice projection or genuinely unresolvable base under the
  same root is conservatively treated as overlapping.

The sole compiler-owned evaluation-order exception is a typed `defaultdict`
list `extend` or set update-family call. Those lowerings insert the destination
entry, materialize all arguments, and only then take the bucket borrow, so a
same-map argument such as `groups[1].extend(groups[2])` is accepted. Lowering
still proves the backing-map place and codegen still requires the dedicated
indexed-storage target plus an in-place method on the resolved bucket type.
Every other specialized indexed mutation retains the conservative overlap
rule.

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

Status: **Merged** in
[#3082](https://github.com/sifr-lang/sifr/pull/3082), with whole-phase
correctness remediations merged in
[#3087](https://github.com/sifr-lang/sifr/pull/3087) and
[#3090](https://github.com/sifr-lang/sifr/pull/3090), with callable-field
invocation precision under draft PR
[#3092](https://github.com/sifr-lang/sifr/pull/3092). Closure remains pending
that remediation and the terminal whole-phase `SATISFIED` review.

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

- `SIFR-OWN-0002`: use for every conflicting borrow, including same-call
  overlapping place conflicts and pending async-generator advances.
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

Closure validation evidence:

- Item 2 exact reviewed head `774a00389c0340a981388a987d7049ffbd88edf4`
  passed full lowering (`936 passed`, `1 ignored`), codegen (`953 passed`),
  focused pass/fail fixtures, formatting, Clippy with warnings denied, and all
  repository guardrails.
- The Item 2 create-PR gate exited 0 with `131/131` E2E fixtures and report
  signature `7c39b8c1dd4fec7c`. Its default-gate functional lanes passed; only
  three representative timing thresholds missed during concurrent host load.
- Item 2 merged as `fbbb69328ae6fe1e733ce25cb6e710aab75990dc`
  after Claude Opus exact-head review pass 12 returned `SATISFIED` with zero
  actionable findings.
- The complete corpus candidate passed `407/411`; the four remaining failures
  reproduced identically on the untouched Item 2 base compiler. The LRU and
  Rotate Array compatibility snapshots pass check, native build, and run.
- Remediation exact head `a26daa7a10a5efce4cc5e5881d395e224b492769`
  passed lowering (`937 passed`, `1 ignored`), codegen (`954 passed`),
  formatting, Clippy, HIR/docs/file-size guardrails, and focused diagnostic and
  inherited-field end-to-end probes.
- The remediation create-PR gate exited 0: Python interop `19/19`, Rust
  interop `10/10`, generated-code quality `5/5`, runtime-platform `28`
  variants with one declared capability skip, crate tests green, and E2E
  `131/131` with report signature `7c39b8c1dd4fec7c`. Every blocking step and
  timing budget passed.
- Remediation PR #3087 merged as
  `a7a5df414b985cc95a9ad23c5b006caa84101f0d` after exact-head Claude Opus
  review returned `SATISFIED` with zero actionable findings.
- Overlap-remediation exact implementation head
  `92b38be705138643b23c37a425892df767beee5d` passed the create-PR gate,
  including Python interop `19/19`, E2E `137/137` with signature
  `eeeeb711211648b0`, full lowering (`941 passed`, `1 ignored`), full codegen
  (`954 passed`), full annotated fail corpus (`564 passed`), formatting,
  Clippy, and every documentation, HIR, and file-size guardrail.
- The post-review manifest lane passed `138/138`, signature
  `4ede7c71d86f381c`, after adding the seventh native phase pass fixture.
- Remediation PR #3090 merged as
  `44ab8ad38544fa5225d8d4f09ad3b5026d485c25` after five Opus review rounds;
  exact-head pass 5 returned `SATISFIED` with no blocking or non-blocking
  findings.
- Two authoritative default merge-profile attempts on integrated closure head
  `260a0d22b2330c2b947fc7a095e150078cee7b27` passed every functional lane:
  coverage and core guardrails, diagnostics, CPython differential, Python
  interop `25/25`, Rust interop `10/10`, frontend/syntax guardrails, and
  developer tooling `32/32`. Both reached the final representative performance
  step; the first ran beside an unrelated four-worker native corpus audit and
  missed three medians, while the second missed arithmetic by `11.368ms` and
  JSON diagnostics by `292.035ms`.
- The repository's unchanged official subset gate then passed arithmetic at
  `1275.878ms < 1334.139ms` and JSON diagnostics at
  `1282.951ms < 1335.954ms`, each with the required five samples and
  `check_budgets.py --allow-subset`. No baseline, threshold, sample count,
  budget, or waiver changed.
- A subsequent complete representative retry passed seven of eight enforced
  variants and missed only JSON diagnostics by `3.961ms`; the accepted
  five-sample JSON subset above is the final uncontended measurement for that
  case. The whole-phase reviewer must assess this exact evidence before PR
  #3088 becomes ready.

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
  overlapping receiver/argument/read/move shape at Sifr check time except the
  audited typed-`defaultdict` iterable mutators whose arguments are explicitly
  materialized before the destination bucket borrow.
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
- Item 2 exact-head PR review pass 8:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-8.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-8.md)
  returned `SATISFIED` with zero actionable findings. The reviewer
  independently reproduced the parameter-seed/explicit-initializer matrix,
  repeated-field rejection before complete storage, acceptance after complete
  storage, source-facing constructor diagnostics, full lowering/codegen and
  fail-corpus results, and the wider checked-place/optimizer/protocol
  invariants.
- Item 2 exact-published-head PR review pass 9:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-9.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-9.md)
  returned `NOT SATISFIED`. Its corpus sweep found that
  `0189_rotate_array.sifr` still read `len(nums)` in the same call that mutably
  borrowed `nums`. The follow-up full runner also exposed a distinct
  `0297_serialize_and_deserialize_binary_tree.sifr` internal diagnostic:
  completed-HIR verification re-resolved two lexically distinct nested
  `dfs` helpers through one module-wide name table. The remediation snapshots
  the stable rotate length through upstream corpus PR #41 and makes plain-call
  verification consume the lexical proof metadata already attached during
  lowering instead of a same-spelling global signature.
- Rotate Array corpus milestone review pass 1:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-rotate-corpus-pr-41-claude-opus-review-pass-1.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-rotate-corpus-pr-41-claude-opus-review-pass-1.md)
  returned `NOT SATISFIED`: the code change was correct and independently
  matched Python across edge cases, but the PR body attributed the local
  verifier-fixed `407/411` sweep to clean published parent head `581b363aa`.
  The evidence was corrected to separate the clean-head `406/411` result from
  the pending Item 2 candidate's `407/411` result.
- Rotate Array corpus milestone review pass 2:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-rotate-corpus-pr-41-claude-opus-review-pass-2.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-rotate-corpus-pr-41-claude-opus-review-pass-2.md)
  returned `SATISFIED` with zero actionable findings. Exact reviewed corpus
  head `4fdb439` merged in
  [sifr-lang/leetcode#41](https://github.com/sifr-lang/leetcode/pull/41) as
  merge commit `e75af095`.
- Item 2 exact-head PR review pass 10:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-10.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-10.md)
  returned `SATISFIED` with zero actionable findings. It independently
  confirmed the merged corpus pin, lexical plain-call verifier correction,
  checked-place fail-closed behavior, constructor materialization,
  protocol/optimizer contracts, full library counts, and candidate-versus-base
  corpus attribution.
- Item 2 final merge-evidence review pass 11:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-11.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-11.md)
  returned `NOT SATISFIED`. It accepted the representative benchmark misses
  as host variance, but correctly found that upstream PR #3081 made the branch
  unmergeable in the defaultdict mutable-bucket path and that the lowering
  count was stale. The reconciliation retains upstream's resolved in-place
  method gate and fallible propagation together with Item 2's
  `MethodCallPlaces` and `emit_checked_place` proof. Merged-tree testing then
  exposed and closed two further integration requirements: all typed
  `defaultdict` in-place bucket methods now carry the checked backing-storage
  target, and only the explicitly materialized `extend`/set-update family may
  evaluate same-map arguments before taking the bucket borrow.
- Item 2 terminal exact-head review pass 12:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-12.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-item2-claude-opus-pr-review-pass-12.md)
  returned `SATISFIED` with zero actionable findings. It independently
  reproduced the lowering/codegen counts, exact create-PR gate, complete
  checked-place and diagnostic matrix, corpus ancestry, upstream
  reconciliation, and default-gate functional result. Item 2 merged in
  [#3082](https://github.com/sifr-lang/sifr/pull/3082) as
  `fbbb69328ae6fe1e733ce25cb6e710aab75990dc`.
- Final whole-phase review pass 1:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-final-whole-phase-claude-opus-review-pass-1.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-final-whole-phase-claude-opus-review-pass-1.md)
  returned `NOT SATISFIED`. The implementation semantics were correct, but
  `SIFR-OWN-0002` omitted its required structured `binding` argument,
  inherited-field rerooting was duplicated, and the phase tracker was stale.
- Whole-phase remediation review pass 1:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-remediation-claude-opus-pr-review-pass-1.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-remediation-claude-opus-pr-review-pass-1.md)
  returned `SATISFIED` with zero actionable findings. It independently
  verified all four same-call diagnostic paths, canonical nested-place
  metadata, shared inherited-field storage rerooting, value-read clone
  preservation, uncloned mutating receivers, focused tests, Clippy, and
  file-size compliance. The remediation merged in
  [#3087](https://github.com/sifr-lang/sifr/pull/3087) as
  `a7a5df414b985cc95a9ad23c5b006caa84101f0d`.
- Final whole-phase review pass 2:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-final-whole-phase-claude-opus-review-pass-2.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-final-whole-phase-claude-opus-review-pass-2.md)
  returned `NOT SATISFIED` after finding that
  unsupported callable/recursive field values could bypass footprint
  collection and leak raw Rust borrow/move errors, that the fifth
  `SIFR-OWN-0002` path lacked its structured `binding`, and that native
  phase-fixture/evidence tracking remained incomplete. Remediation merged in
  [#3090](https://github.com/sifr-lang/sifr/pull/3090) as
  `44ab8ad38544fa5225d8d4f09ad3b5026d485c25`.
- Overlap-remediation PR review pass 1:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-1.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-1.md)
  returned `NOT SATISFIED`: the first conservative fallback closed the missed
  diagnostics but collapsed all fields under a root and rejected legal
  callable/recursive sibling fields. The follow-up retains precise
  `FieldIdentity` projections when the base place resolves statically.
- Overlap-remediation PR review pass 2:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-2.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-2.md)
  returned `NOT SATISFIED` on documentation of record only. It independently
  accepted the corrected implementation, ran 21 targeted lowering tests,
  reproduced both fail fixtures as structured `SIFR-OWN-0002`, and inspected
  the exact-head create-PR pass at
  `92b38be705138643b23c37a425892df767beee5d`; this revision aligns the overlap
  rule and status/review ledger before pass 3.
- Overlap-remediation PR review pass 3:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-3.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-3.md)
  returned `SATISFIED` with no blocking findings. The reviewer independently
  reproduced the callable/recursive overlap failures as structured
  `SIFR-OWN-0002`, accepted disjoint sibling fields, verified the async
  generator diagnostic arguments, ran the full lowering and codegen suites,
  checked all documentation and maintainability guardrails, and inspected the
  exact-head create-PR evidence. The non-blocking fixture-manifest cleanup then
  passed the create-PR E2E lane at `138/138` with report signature
  `4ede7c71d86f381c`.
- That review also recorded separate, pre-existing value-codegen debt:
  independently moving callable or recursive fields, and passing a class field
  as a mutable free-function argument, can still reach Rust move/borrow errors.
  These shapes reproduce on the untouched base and are not receiver/argument
  overlap-analysis regressions; they remain follow-up compiler debt rather than
  hidden closure exceptions for this phase.
- Overlap-remediation PR review pass 4:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-4.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-4.md)
  returned `SATISFIED` with no blocking findings after independently rerunning
  the full lowering, codegen, diagnostics, and fail suites; the targeted
  unsupported-field matrix; formatting, clippy, docs, HIR, and file-size
  guardrails; both exact overlap fixtures; manifest integrity; and both
  authoritative logs. Its only record-precision observation was that an older
  `680/680` Item 2 pass-corpus figure had been left beside freshly updated
  library counts. That stale figure is removed here; the current bounded E2E
  evidence remains the independently verified `138/138` create-PR lane, and
  the full pass corpus remains assigned to the integrated closure merge gate.
- Overlap-remediation exact-head record review pass 5:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-5.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-overlap-remediation-claude-opus-pr-review-pass-5.md)
  returned `SATISFIED` with no blocking or non-blocking findings. It confirmed
  that the stale `680/680` figure is absent, the pass-4 artifact and ledger
  entry match the review that occurred, and the exact reviewed documentation
  head `94acb685ccc53a40755683a74cda0c6baec91e8f` is internally consistent.
- Final whole-phase review pass 3:
  [`ad-hoc-class-field-mutating-receiver-place-semantics-final-whole-phase-claude-opus-review-pass-3.md`](../../reviews/active/ad-hoc-class-field-mutating-receiver-place-semantics-final-whole-phase-claude-opus-review-pass-3.md)
  returned `NOT SATISFIED`. It found that callable-field invocation inside a
  same-call argument retained only the parent object footprint, causing a
  root-only rejection of legal disjoint sibling fields. The focused correction
  is under draft PR
  [#3092](https://github.com/sifr-lang/sifr/pull/3092). Its non-blocking record
  finding is closed by restoring the pass-2 artifact above.
