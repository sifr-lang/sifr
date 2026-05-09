# Review: Ad-Hoc Phase — Integer Model and Fixed-Width Numeric Contract (Pass 3)

Reviewer: Claude Opus 4.7
Date: 2026-05-05
Source: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`
Pass 1: `reviews/integer-model-fixed-width-contract-review-pass-1.md`
Pass 2: `reviews/integer-model-fixed-width-contract-review-pass-2.md`

## Verdict: LOCK-READY

All three pass-2 polish items are resolved in the current issue text. No new lock-blocking issues are present. The design is internally consistent, the contracts are concrete, and the slice ordering respects the representation-first dependency. This is ready to serve as the source of truth for the implementation slices.

The reviewer recommends marking the pass-3 box on the issue's `Review Status` checklist and proceeding to slice 1 (lock + update architecture references).

---

## Pass-2 polish item resolution

### Polish 1. `True == 1` — RESOLVED

Line 183 now states unambiguously:

> `bool` remains a separate type: `int(True)` is allowed as an explicit conversion, but `True == 1` is a compile error; users write `int(True) == 1` when they want that comparison. `True` must not alias `1` as a dict/set key.

This is the cleanest possible answer: it removes the deferred-decision footnote, is consistent with the no-implicit-narrowing rule and the "bool is not int" decision, and makes `match` over an `int` subject with `case True:` arms a compile error by the same fitting rule. Pattern matching does not need a separate clause.

### Polish 2. `int / int` ergonomic expectation — RESOLVED

Line 101 now adds the call-site-friction note:

> In practice, `int / int` is fallible at the call site for non-literal operands. This is intentional: silent precision loss across the exact `int` to approximate `float` boundary would violate the no-silent-loss rule.

This sets reader expectations correctly: `int / int` is essentially always fallible in non-literal code, and that is a deliberate cost of the no-silent-loss core rule rather than a compiler limitation. Library authors will reach for `//`, `Decimal`, or explicit `float(...)` accordingly.

### Polish 3. `int * int` allocation framing — RESOLVED

Line 103 now reconciles the no-panic guarantee with allocator behavior:

> Straight-line `+`, `-`, and `*` remain `int`-returning; process-level resource exhaustion such as allocator failure or stack exhaustion is an operational concern bounded by parser limits and configured arithmetic budgets, not by turning ordinary `int` arithmetic into overflow-prone fixed-width arithmetic.

Combined with the explicit `ArithmeticLimitError` budget for `**` and `<<` in the same paragraph, the reader now has a clear mental model: typed errors for explosive operators, OS-bounded allocation for straight-line arithmetic, and the parser/external-boundary digit limits as the upstream defense. The no-panic guarantee is correctly framed as "no user-triggerable typed runtime panic", not "no OOM ever".

---

## Internal consistency verification

I cross-checked the load-bearing rules against each other for contradictions. None remain.

| Pair of rules | Consistent? | Notes |
| --- | --- | --- |
| Scalar fixed-width promotes to `int` (line 162) vs. array/tensor element-wise stays in dtype (lines 214–219) | Yes | The carve-out is explicit and the dtype kernels are the only path that preserves the fixed-width element type. |
| `int` is `Eq`/`Hash` but not `Copy` (line 347) vs. source-level value semantics ("non-consuming", line 349) | Yes | Codegen owns the borrow/clone choice; the source language never sees move semantics. |
| `int8(-1) != uint8(255)` (line 183) vs. cross-family hash agreement (same line) | Yes | Hash agreement is conditional on equality, and these compare unequal as mathematical values, so no hash constraint applies. |
| `int / int` is `Result[float, ...]` (line 97) vs. no-silent-loss core rule (line 226) | Yes | Polish 2's note now ties these together explicitly. |
| `int ** int` integer-preserving (line 99) vs. `2 ** -1` Python behavior | Yes — intentional break | The doc names the break and the migration path (`float(2) ** -1`). |
| `bytes[i] -> uint8` (line 200) vs. no-implicit-widening rule | Yes | Reads return `uint8`; widening to `int` requires explicit `int(b)`. The asymmetry pass 1 flagged is gone. |
| `len() -> int` (line 322) vs. compiler-internal `usize` conversion (line 324) | Yes | The wasm32 carve-out (line 324) makes the 32-bit case explicit. |
| Fixed-width literal fitting (lines 51–55) vs. const-fold scope (lines 76–79) | Yes | The doc names exactly which constructs participate (literals, unary ±, `+`/`-`/`*`, fitting-shift, parens, const-evaluable module constants) and which do not (function calls, conditionals, lookups, non-constant names). |
| Generic `T + T -> T` invalid for fixed-width (line 166) vs. `sum_int32` example (lines 169–174) | Yes | The example uses an explicit `int` accumulator, which is the workaround the doc recommends. |

The rules read as one coherent system rather than a collection of independent decisions.

---

## Quality observations

A few things worth noting for the implementation team picking this up:

- **The const-evaluable rule (lines 76–79) is precise enough to implement directly.** No interpretation is needed. The "immutable module constants whose initializer is itself const-evaluable" clause does the right thing without dragging in flow-sensitive analysis.

- **The `SifrInt` enum (lines 340–345) is the smallest commitment that pins ownership semantics.** Naming `num_bigint::BigInt` rather than leaving the bignum dependency open avoids a bikeshed during slice 2 and makes the trait surface (`Clone`, `Eq`, `Ord`, `Hash`, `Send`, `Sync`, not `Copy`, no `repr(C)`) verifiable from the dependency's documentation.

- **The arithmetic table (lines 91–97) and the ArithmeticLimitError mechanism are well-scoped.** Budgeting only the explosive operators (`**`, `<<`) keeps `+`/`-`/`*` cheap while still giving operators a typed escape hatch for hostile input.

- **The slice ordering (lines 386–395) puts `SifrInt` before broad operator codegen (slice 2 → slice 5).** This is the correct dependency direction; flipping it would force re-doing every operator codegen decision when the runtime shape changes.

- **Compiler architecture impact item 9 (line 380) names the ownership/codegen change explicitly.** The "value-semantic but not `Copy`" framing is the contract HIR/codegen need; the optimizer-can-use-primitive-locals-when-sound clause leaves room for future small-int specialization without changing the source semantics.

None of these are blockers. They are reasons the document will be easier to implement against than the pass-1 version.

---

## Items intentionally left for follow-up phases

These are correctly out of scope for this issue and are noted only to confirm they were considered:

- **Detailed contracts for `bit_length()`, `bit_count()`, `to_bytes(...)`, `from_bytes(...)`** (line 353) — reserved at the surface level, deferred to the stdlib phase. Right call.
- **Decimal-family equality semantics** (line 183) — defers to "the Phase 28 exact numeric policy". Right call; restating it here would risk drift.
- **Future zero-copy views between `bytes` and `array[uint8]`** (line 202) — explicitly named as a future feature with mutability and view-lifetime requirements. Right call; that is a real type-system question and does not belong in the integer model.
- **`int128`/`uint128`** (line 35) — reserved as future names. Right call; shipping them now would expand the test matrix without a clear use case.
- **Enum representation design** (lines 327–328) — constrains valued enums to `int64`-representable values until a broader enum representation lands. Right call; tying the integer model to an unfinished enum design would block lock.

---

## Things to keep as-is from pass 2

The list of "newly added that I think are right" from pass 2 (slice ordering, architecture impact item 9, reserved 128-bit names, wasm32 carve-out, bytes/array distinction) all stand. No regressions in this revision.

The pass-1 "things to keep as-is" list (no bare `uint`, fixed-width-to-`int` promotion for ordinary arithmetic, named `checked_*`/`wrapping_*`/`saturating_*`/`overflowing_*`, JSON profiles, removing user-facing `bigint`, signed indexing, negative-literal rejection at compile time, bool not subclassing int, dtype-driven loading) all stand too.

---

## Summary

Lock-ready. The eight pass-1 blockers and the three pass-2 polish items are all resolved with implementable contracts. Internal consistency holds across the rules. Slice ordering is correct. The issue is fit to serve as the source of truth for slices 2–8.

Recommendation: **lock**. Mark the pass-3 checkbox on the issue's `Review Status` section, complete slice 1 (lock + update architecture references that still say `int = i64`), and proceed to slice 2 (`SifrInt` representation and source-level value semantics).
