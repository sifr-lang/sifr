# Verdict: CHANGES REQUIRED

Two compiler-soundness defects remain. Both can admit Sifr programs that fail during generated Rust compilation, violating the project’s compile-time guarantee.

## Findings

### High — Keyed `sorted()` admits element types that emitted Rust unconditionally clones

Lowering validates only the key result’s ordering capability after rejecting affine elements. It does not require the element type itself to support `Clone`: `crates/sifr_lowering/src/lower/expression_sum_sorted.rs:162` and `:211`.

Both keyed code-generation branches clone the comparator arguments before calling the key:

- Float-key branch: `crates/sifr_codegen/src/intrinsic_method_emitters/builtin_core_methods.rs:421`, clones at line 450.
- Ordinary-key branch: the same file at line 487, clones at line 513.

Consequently, this shape is admitted because the key returns `int`, but its generated Rust requires `Local: Clone`:

```python
class Local(NonSend):
    pass

def key(value: Local) -> int:
    return 0

def order(values: list[Local]) -> list[Local]:
    return sorted(values, key=key)
```

The current lowering test covers non-`Ord` elements and key results but not a valid key over a non-Clone element. Fix this by either emitting a key call that does not require cloning or admitting keyed sorting only when all emitted ownership requirements are satisfied. Add native accepted-valid and compile-fail coverage for this boundary.

### High — Generic method bounds remain non-consumer-specific and specialization rejection misses unrelated non-Clone parameters

Clone-bound inference renders an entire method, searches for any `.clone()`, and then adds `Clone` to every class type parameter in `crates/sifr_codegen/src/class_emitter.rs:138` and `:164`.

Lowering tries to compensate by rejecting a specialization only when a non-Clone field or borrowed parameter appears in the method return type in `crates/sifr_lowering/src/lower/expressions/method_type_objects.rs:251`. That does not identify which declared type variable the emitted clone actually consumes.

For example:

```python
class Pair[A, B]:
    first: A
    second: B

    def first_value(self) -> A:
        return self.first

class Local(NonSend):
    pass

def read(value: Pair[int, Local]) -> int:
    return value.first_value()
```

`first_value()` clones `A`, but codegen emits an impl requiring both `A: Clone` and `B: Clone`. Lowering sees that the concrete return type is `int`, does not associate `Local` with that return, and admits the call. Rust then reports that the method is unavailable because `Local` is not `Clone`.

There is also a remaining blanket-bound path for binary operator impls: every generic parameter receives `Clone + Display + PartialOrd`, plus possible hash bounds, through `crates/sifr_codegen/src/operator_protocol_emitters.rs:34` and `crates/sifr_codegen/src/generic_bounds_helpers.rs:129`.

Bounds need to be derived per type parameter from actual operations, with lowering enforcing the same obligations. The existing one-parameter negative fixture does not cover this multi-parameter failure.

### Medium — Tracking evidence overstates completion of the two boundaries above

The phase ledger says individual-method bounds follow emitted requirements and that all ordering gaps were remediated. The activation matrix similarly records recursive Clone/ordering/generic-bound negative evidence as passing. The buffer surface’s active status itself is supported, but these detailed closure claims are not yet accurate. Revise or complete them alongside the compiler and regression fixes.

## Accounting for the nine requested concerns

1. **Ordering admission — fails.** `sorted()` correctly uses `f64::total_cmp` for float elements and float keys, while `list.sort()` requires ordinary total `Ord` and rejects floats because its Rust emitter uses `.sort()`. Keyed `sorted()` nevertheless misses its element-Clone requirement.
2. **Generic/inherited Display and Debug — passes.** Capability calculation and emitted conditional impls agree for the reviewed generic and inherited cases.
3. **Consumer-specific generic bounds — fails.** Hash/Eq storage bounds and conditional formatting bounds are substantially improved, but method Clone inference still binds every type parameter, and operator impls retain blanket Clone/Display/PartialOrd bounds.
4. **Affine reusable closures — passes.** Lambda free-variable analysis and nested-function capture collection reject affine `PythonBuffer` captures before HIR. Both unit and compile-fail fixtures cover the behavior.
5. **Affine walrus aliasing — passes.** Named expressions reject values containing an affine resource before defining the alias. The diagnostic and use-after-move behavior are coherent.
6. **Documentation and activation — fails narrowly.** The active buffer surface and pending Wave 3 status match implementation. The detailed negative-evidence and bounds claims overstate the remaining generic/sorting coverage.
7. **Cold-build regressions — partially passes, overall fails.** Inherent-impl deduplication now includes impl item names; ChannelReceiver and Counter-style positive fixtures are present and recorded as passing after the clean build. Non-Clone getter rejection is not general across multi-parameter specializations.
8. **Behavioral regression coverage — fails.** The PR contains genuine native pass and compile-fail tests, not merely textual snapshots. However, neither blocking counterexample above is tested.
9. **File size and decomposition — passes.** The maintainability guardrail passes. No touched guarded source file exceeds 900 lines; three are exactly 900. New responsibilities are split into focused modules.

## Validation and inspection performed

- Read all requested workflow, phase, prior-review, architecture, activation, and tracking documents.
- Reviewed the complete `main...HEAD` change set and relevant compiler, runtime, stdlib, fixture, and documentation paths.
- Confirmed the only working-tree dirt is the excluded `third_party/ruff` submodule.
- `git diff --check main...HEAD`: passed.
- HIR maintainability guardrail: passed.
- Focused compiled tests passed for sorted total-order rejection, generic/inherited formatting, affine lambda/nested capture rejection, affine walrus rejection, stdlib impl deduplication, and ten PythonBuffer code-generation tests.
- Inspected the recorded post-`cargo clean` create-PR evidence and traced each reported cold-build regression to its implementation and fixtures.
- Inspected the buffer FFI/runtime lifecycle and found no new data-dependent `unwrap`, `expect`, or user-triggerable panic path.

## Residual risks

Beyond the blocking findings, the principal remaining risk is the inherently unsafe raw Python buffer FFI boundary and its live NumPy-compatible matrix, which is appropriately deferred to Wave 3. The full clean create-PR facade was not rerun in this read-only review; the reported clean result was treated as validation evidence, not as proof of semantic correctness.

## Remediation

- Keyed `sorted()` now invokes shared-borrow key callables directly from the
  comparator's `&T` bindings. Owned key parameters are admitted only for
  Clone-capable elements and receive an explicit clone; mutable-borrow key
  parameters are rejected because Rust sorting exposes only shared references.
  Preserved non-iterator sources must also be Clone-capable because the result
  materialization copies their elements, while consumed temporaries and
  iterators retain their owned elements.
- Generic method and binary-operator impl bounds are now constructed per type
  parameter from the function body and emitted consumer. A clone in an
  `A`-producing getter no longer adds `Clone` to unrelated `B`; blanket
  `Clone + Display + PartialOrd` operator bounds were removed.
- Permanent native coverage includes
  `sorted_non_clone_temporary_borrowed_key.sifr` and
  `generic_class_unrelated_non_clone_param.sifr`. Compile-fail coverage includes
  `sorted_non_clone_preserved_source_rejected.sifr` and
  `sorted_non_clone_owned_key_rejected.sifr`.
- Focused validation after remediation passes: code generation `822/822`, HIR
  lowering `745` passed with one ignored, and the complete compile-fail matrix
  `518/518`. Both new native-positive fixtures build and run. The requested
  clean build additionally exposed and repaired transitive collection method
  bounds, stale inferred-return call signatures, and parity fixtures that
  predated the active identity and empty-collection contracts. The complete
  merge-profile E2E suite passes `657/657` with signature
  `18e6999f2fd35220`; the authoritative create-PR facade passes every blocking
  lane, including Python interop `11/11`, runtime platform `28/28`, and E2E
  `131/131` with signature `7c39b8c1dd4fec7c`.
