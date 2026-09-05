# Review: Ad-Hoc Phase — Integer Model and Fixed-Width Numeric Contract (Pass 5, Broader-Surfaces Final)

Reviewer: agent
Date: 2026-05-05
Source: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`
Pass 1: `reviews/integer-model-fixed-width-contract-review-pass-1.md`
Pass 2: `reviews/integer-model-fixed-width-contract-review-pass-2.md`
Pass 3: `reviews/integer-model-fixed-width-contract-review-pass-3.md`
Pass 4: `reviews/integer-model-fixed-width-contract-review-pass-4-broader-surfaces.md`

## Verdict: LOCK-READY

All eleven pass-4 polish items (P1–P11) are addressed in the current issue text. The expanded design remains internally consistent with the locked core (passes 1–3), no new contradictions or gaps were introduced by the polish edits, and there are no source-of-truth blockers. The issue is fit to serve as the design source of truth for slices 1–10.

Recommendation: **lock pass 5**, mark the pass-5 box on the issue's `Review Status`, and proceed to slice 1 (lock + update architecture references).

---

## Pass-4 polish item resolution

### P1. Fixed-width-signature ergonomic cost — RESOLVED

Lines 249–262 add a dedicated paragraph and three examples to "Type Inference and API Defaulting":

```python
def increment_port(port: uint16) -> Result[uint16, OverflowError]:
    return uint16(port + 1)

def increment_port_checked(port: uint16) -> Result[uint16, OverflowError]:
    return uint16.checked_add(port, 1)

def add_samples(left: int16, right: int16) -> int16:
    return int16.saturating_add(left, right)
```

The follow-up sentence pins the three intentional alternatives (fallible narrow, checked method, exact-`int` accumulator). Reader expectations are correctly set: `port + 1` will not infer back to `int16`, and that is by design.

### P2. `**` participation in compile-time fitting — RESOLVED

Lines 79–80 take the more ergonomic option and state it directly:

> Integer exponentiation participates in compile-time fitting when both operands are const-evaluable, the exponent is non-negative, and the configured compile-time arithmetic budget is not exceeded. This makes `x: int32 = 10 ** 3` valid and `x: int32 = 10 ** 100` a range diagnostic rather than an inference mystery.

This is consistent with the arithmetic table (line 97) and the container-inference example at line 242 (`[1, 2, 10 ** 100]` as a compile error). Both `10 ** 3` and `10 ** 100` lower through const-eval; the first fits `int32`, the second triggers either the budget or the fitting check, both surfacing as a clear diagnostic. No ambiguity remains.

### P3. JSON profile applies element-by-element for collections — RESOLVED

Lines 367–368 add the recursive-application clause:

> Profile rules apply recursively to collection-valued fields. For `list[int]`, `dict[str, int]`, nested objects, and other containers, each integer element follows the same profile as the containing field unless a schema annotation overrides the nested element policy. A field that opts into string integer encoding string-encodes its integer elements recursively.

This closes the "what about `list[int]` under `json.web`?" gap and makes nested-structure behavior explicit without requiring readers to infer it.

### P4. `int64` IDs under default `json.web` are a footgun — RESOLVED

Lines 313–314 take the docs/default option (the lower-friction choice):

> Under `json.web`, schema-driven public models default to JSON numbers only when the field's static range is inside JavaScript's safe integer range. Wider fields such as `int64`, `uint64`, and exact `int` default to decimal string encoding unless the field is explicitly annotated with a runtime range-checked number policy or an exact-client policy. Untyped/dynamic JSON values still use the profile's runtime check and return `JsonIntegerRangeError` for unsafe numbers unless the caller selects string encoding.

This is the right framing: the default behavior matches what most production users actually want for Snowflake/Discord/Stripe-style IDs, the opt-out is one annotation away for fields that are guaranteed safe, and the dynamic-value path retains its runtime check. The example model at lines 308–310 already shows the pattern (`id: int64` with the noted "string-encoded by default under json.web" comment), so the surface and the rule are coherent.

### P5. Dtype-preserving sum/min/max/abs API names — RESOLVED

Lines 282–289 pin the naming and explicitly extend the carve-out to all reductions:

```python
total: Result[int32, OverflowError] = int32.checked_sum(values)
wrapped: int32 = int32.wrapping_sum(values)
bounded: int32 = int32.saturating_sum(values)
```

> Array/tensor/dataframe reductions use the same naming pattern as element-wise kernels: `xs.checked_sum()`, `xs.wrapping_sum()`, `xs.saturating_sum()`, and explicit widening APIs such as `xs.widen_sum()`. The fixed-width carve-out covers all dtype-preserving arithmetic and reductions, not only addition.

The scalar form (`int32.checked_sum(values)`) lives on the fixed-width type's namespace, the array form (`xs.checked_sum()`) lives on the array, and the closing sentence makes clear the carve-out is general — not just `_add`. This removes the slice-8 bikeshed.

### P6. `int + float` and the no-silent-loss rule — RESOLVED

Line 183 makes the rule explicit rather than deferring to "the existing float policy":

> `int` or fixed-width integer mixed with `float` is fallible unless the integer operand is proven exactly representable as `float`; otherwise the operation returns `Result[float, FloatPrecisionLossError]` or requires explicit `float(...)` conversion according to the final float operator lowering. There is no silent exact-integer-to-float precision loss.

This pins the no-silent-loss rule across the `int↔float` boundary in the same way it is pinned for `int / int`. The "according to the final float operator lowering" hedge correctly defers the implementation choice (Result vs. required explicit conversion) to the float-operator design without leaving the precision-loss question itself open.

### P7. Newtype mechanism — RESOLVED

Line 328 names the mechanism and flags the dependency:

> Newtype guidance depends on Sifr's existing primitive-newtype surface (`class UserId(int64)` / `class Port(uint16)`-style wrappers) or the equivalent branded-type mechanism when that surface is finalized. If a slice lands before newtypes are complete, raw fixed-width fields are acceptable at storage/interop boundaries but should not be presented as the final domain-model style.

This is the correct framing. Readers are no longer expected to act on aspirational newtype guidance during slice 1; the existing primitive-newtype shape is named, and the interim posture for raw fixed-width fields is explicit.

### P8. Validation matrix — RESOLVED

The matrix (lines 516–533) now adds the seven missing rows pass 4 called out:

- Pattern matching with literal arms (line 527)
- Mixed numeric arithmetic, including `int`/`float` precision cases (line 528)
- Formatting and integer methods (`bit_length`, `bit_count`, `to_bytes`, `from_bytes`) (line 529)
- Range and large-bound iteration (line 530)
- `usize`/`isize` boundary (line 531)
- Performance regression (`SifrInt::Small` not allocating in tight loops) (line 532)
- Cross-type dict/set lookup (line 533)

The matrix now covers every load-bearing surface the rules describe, with both positive and negative cases per row. Slice 8 has a concrete checklist.

### P9. Diagnostic for `port + 1`-style narrowing failure — RESOLVED

Line 342 adds an explicit diagnostic family:

> fixed-width return narrowing from widened arithmetic: for `def f(x: int16) -> int16: return x + 1`, suggest `int16(x + 1)` for fallible narrowing or `int16.checked_add(x, 1)` / `int16.saturating_add(x, 1)` when representation-preserving arithmetic was intended.

This is the most-fired diagnostic for fixed-width-API authors and now has its own bullet, its own wording, and a reserved spot in the diagnostic registry under a future `SIFR-*` code. Pairs cleanly with P1's ergonomic-cost paragraph.

### P10. `int` field in `repr(C)` struct — RESOLVED

Line 493 adds the cross-section consequence to the Rust Interop section:

> Sifr structs/classes containing `int` fields are not C-ABI-compatible because `SifrInt` has no `repr(C)` layout guarantee. FFI structs must use fixed-width integer fields for integer slots or an explicit future big-integer handle type.

This closes the natural follow-up question to the "no `repr(C)` for `SifrInt`" rule and gives FFI authors the right pattern explicitly.

### P11. Port number example uses `int16` — RESOLVED

Line 17 now reads `port: uint16 = 5432`, matching the Identifiers/Time guidance at line 324 ("ports … use fixed-width/newtype wrappers"). Internal example consistency is restored.

---

## Internal consistency verification

I cross-checked the pass-5 polish edits against the rest of the document for newly introduced contradictions. None are present.

| Pair of rules | Consistent? | Notes |
| --- | --- | --- |
| Const-eval `**` (lines 79–80) vs. runtime `int ** int` (line 97) | Yes | Both gate on non-negative exponent and the same arithmetic budget. The const-eval and runtime rules use the same predicate; the only difference is when the predicate is checked. |
| Container example `[1, 2, 10 ** 100]` (line 242) vs. const-eval `**` | Yes | Either the budget trips first or the fitting check trips; both lead to the same compile-error outcome the example claims. |
| `int + float` rule (line 183) vs. `int / int` rule (line 97) | Yes | Both express the no-silent-loss rule across the exact↔approximate boundary; `int + float` is fallible by the same argument as `int / int`. |
| `json.web` default (line 273) vs. `int64`/`uint64` string-encoding default (lines 313–314) | Yes | The framework default for browser-facing APIs is `json.web`; under that profile, wider fixed-width fields default to string encoding. The example model at lines 308–310 demonstrates the pattern. |
| Profile recursive application (lines 367–368) vs. TypeScript client mapping (line 301) | Yes | TS clients see `string` (or branded decimal-integer string) for string-encoded fields; recursive application means list elements get the same TS mapping. |
| Scalar dtype-preserving reduction `int32.checked_sum(values)` (line 285) vs. array form `xs.checked_sum()` (line 288) | Yes | Scalar list path needs an explicit dtype hint (the type's static method); array path knows its dtype (instance method). The naming pattern (`checked_*`/`wrapping_*`/`saturating_*`/`widen_*`) is identical. |
| Carve-out coverage statement "all dtype-preserving arithmetic and reductions, not only addition" (line 289) vs. earlier element-wise carve-out (lines 218–222) | Yes | Pass 4's concern was that the carve-out only listed `_add`; line 289 makes the general statement explicit and the element-wise list at lines 218–222 is now correctly read as illustrative, not exhaustive. |
| Newtype dependency framing (line 328) vs. validation-matrix "Domain newtypes" row (line 525) | Yes | The matrix exercises the newtype mechanism's behaviors but the design doc now flags that the mechanism itself is dependent; this is a clean separation between "what tests should exist" and "when the mechanism is final". |
| Pass-4 P11 fix (`port: uint16`) vs. domain-value guidance (line 324) | Yes | Both name `uint16` for ports; the source-level example and the domain-value section now agree. |
| `repr(C)` consequence for structs (line 493) vs. `SifrInt` no-`repr(C)` rule (line 474) | Yes | Line 493 is the natural transitive consequence of line 474; no new constraint, just an explicit statement of one. |

The polish edits compose with the locked rules and with each other.

---

## Quality observations

A few things worth keeping for the implementation team picking this up:

- **The const-eval rule is now uniform across explosive operators.** Lines 76–80 cover `+`, `-`, `*`, shifts, and `**` with the same fitting-and-budget gate. This matches what the runtime rules already do, so the const-eval engine and the runtime engine share a predicate. Slice 6 will benefit from this symmetry.

- **The `json.web` default for `int64`/`uint64` is the right opinionation.** Defaulting wider IDs to string encoding under the web profile is the production-correct behavior for the Discord/Snowflake/Stripe-shaped use case; an opt-in number policy for fields that are statically guaranteed safe keeps the ergonomics for narrower IDs. This decision will save users from production-only runtime serialization failures, which is the failure mode that hurts most.

- **Splitting the diagnostic for `f(x: int16) -> int16: return x + 1` into its own bullet (P9) is the right call.** That specific shape will be fired against during the first month of fixed-width usage by anyone porting Python integer code. Pre-committing to a tailored message and a reserved `SIFR-*` slot avoids a "diagnostic improvement" follow-up later.

- **The dtype-preserving reduction naming (P5) generalizes the carve-out cleanly.** The `int32.checked_sum`/`xs.checked_sum` split mirrors how scalar list reduction needs a dtype hint while array reduction already knows its dtype. The "covers all dtype-preserving arithmetic and reductions, not only addition" sentence is the load-bearing phrase that prevents future confusion about whether `min`/`max`/`abs`/etc. need their own carve-out section.

- **The newtype dependency is correctly flagged without blocking.** Naming the existing primitive-newtype surface (`class UserId(int64)`-shape) gives the section a real grounding, while the "if a slice lands before newtypes are complete" clause prevents downstream rework if the newtype RFC reshapes. This is the pattern other phase docs should follow when they reference dependent surfaces.

None of these are blockers; they are reasons the document remains easier to implement against than the pass-3 version.

---

## Items intentionally left for follow-up phases

These remain correctly out of scope, unchanged from pass 4's accounting:

- Detailed contracts for `bit_length`, `bit_count`, `to_bytes`, `from_bytes` (line 480) — surface reserved, contracts deferred to stdlib phase.
- GraphQL/gRPC and other RPC surfaces beyond JSON+OpenAPI+TS — JSON profile model generalizes; explicit coverage can land per surface.
- Full enum representation design (line 456) — already deferred; valued enums constrained to `int64`-representable until that phase.
- `int128`/`uint128` (line 35) — name-reserved; shipping is a future stdlib decision.
- Newtype/branded-type RFC (line 328) — flagged as a dependency; integer model can lock without it.
- Final float operator lowering (line 183) — `int + float` rule states the no-silent-loss invariant and defers the Result-vs-required-conversion shape to float-operator design.

---

## Things to keep as-is from prior passes

Every decision called "right" in passes 1–4 stands without regression in pass 5:

- No bare `uint`. Fixed-width-to-`int` promotion for ordinary scalar arithmetic. Named `checked_*`/`wrapping_*`/`saturating_*`/`overflowing_*`. JSON `exact`/`web`/`string_ints` profiles with the recursive application clause. Removing user-facing `bigint`. `int` indexes/lengths with compiler-internal `usize`. Negative literals rejected for unsigned at compile time. `bool` not subclassing `int`; `True == 1` is a compile error. Dtype-driven Arrow/Parquet/tensor loading. `SifrInt::{Small, Big}` representation with `num_bigint::BigInt`. wasm32/32-bit `usize` carve-out. Slice 2 ordering before broad operator codegen. Compiler architecture impact item 9 (`int` is value-semantic but not `Copy`). Element-wise array dtype carve-out. Pattern-matching literal fitting. `int / int → Result[float, ...]`. `int ** int` integer-preserving (Python break is named).

The pass-5 polish does not regress any of these.

---

## Summary

Lock-ready. All eleven pass-4 polish items are addressed inline with implementable, internally consistent contracts. No source-of-truth blockers remain. The expanded design — scalar core, array/tensor carve-out, web/serialization boundaries, type inference and stdlib defaulting, identifiers/time/domain values, diagnostics, and the validation matrix — composes as one coherent system.

Recommendation: **lock**. Mark the pass-5 checkbox on the issue's `Review Status` section and proceed to slice 1 (lock + update architecture references that still say `int = i64`). The document is fit to serve as the source of truth for slices 1–10.
