# Review: Ad-Hoc Phase — Integer Model and Fixed-Width Numeric Contract (Pass 2)

Reviewer: agent
Date: 2026-05-05
Source: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`
Pass 1: `reviews/integer-model-fixed-width-contract-review-pass-1.md`

## Verdict: LOCK-READY (with three minor polish items)

All eight blocking findings from pass 1 are resolved with clear, implementable contracts. The non-blocking suggestions A–J are largely addressed too. The revised issue is internally consistent, the array carve-out is unambiguous, the runtime representation is concrete, and the result-type tables for division/exponent/shift have stopped punting. I would lock this as the source of truth after the three small polish edits below — none of them affect implementation slice ordering.

---

## Pass-1 blocker resolution

### 1. Array/tensor carve-out — RESOLVED

Lines 207–212 explicitly state `array[int32] + array[int32] -> Result[array[int32], OverflowError]` by default, with `wrapping_add`/`saturating_add`/`overflowing_add` as named representation-preserving kernels and `widen_add` as the explicit promotion path. Float dtype arrays follow float semantics. The carve-out is clean and the default-fallible choice is the right one given the no-panic guarantee.

### 2. `SifrInt` representation and source value semantics — RESOLVED

Lines 333–338 commit to:

```rust
pub enum SifrInt {
    Small(i64),
    Big(Box<num_bigint::BigInt>),
}
```

Trait surface (`Clone`, `Eq`, `Ord`, `Hash`, `Send`, `Sync`, not `Copy`, not `#[repr(C)]`), FFI shape (no `SifrInt` over C ABI; FFI uses fixed-width or a future bignum handle), and source-level value semantics ("non-consuming: using an `int` binding in more than one expression is always legal — codegen is responsible") are all stated. This was the highest-impact pass-1 blocker and the answer is concrete.

The slice ordering also moved to make this slice 2, before broad operator codegen, which directly addresses pass-1 suggestion J.

### 3. Division / exponent / shift / fixed-width division contracts — RESOLVED

Lines 86–96 give a complete table:

- `int + int`, `int - int`, `int * int` → `int`
- `int // int`, `int % int` → `Result[int, DivisionError]` unless non-zero proven
- `int ** int` → `int` when exponent non-negative and within budget; otherwise `Result[int, ValueError | ArithmeticLimitError]`
- `int << int`, `int >> int` → `int` when shift valid and within budget; otherwise `Result[int, ValueError | ArithmeticLimitError]`
- `int / int` → `Result[float, DivisionError | FloatOverflowError]` unless non-zero and float-representability proven

The "integer `**` is intentionally integer-preserving" justification (line 94) is the right call and answers the negative-exponent question explicitly. The DoS surface for `**` and `<<` is gated by an `ArithmeticLimitError` budget. Fixed-width scalar division (`int32 // int32`) is now stated to follow the same promote-to-`Result[int, DivisionError]` rule (line 136). All four sub-blockers under pass-1 #3 are now answered.

### 4. `bytes` element type and `bytes` vs `array[uint8]` — RESOLVED

Lines 193–195 commit to `uint8` as the element type (the dtype-consistent option) and explicitly say `bytes` is not an alias for `array[uint8]` — it's an immutable, read-only byte buffer, and zero-copy views between the two are a future explicit feature. The asymmetry pass-1 flagged (write fallible-narrows from `int`, read silently widens to `int`) is gone. Good.

### 5. Fixed-width generic arithmetic and `usize`/`isize` — RESOLVED

Line 158 specifies `usize`/`isize` follow the scalar promotion rule when they leave FFI signatures, with explicit fallible narrowing. Lines 159–167 specify that `T + T -> T` is invalid for fixed-width scalars and gives the recommended workaround (explicit accumulator type or use named checked/wrapping/saturating APIs), with a worked `sum_int32` example. The implication that generic numeric code over fixed-width must explicitly carry an accumulator type is a real constraint, but it is now stated rather than implicit.

### 6. Equality / hashing across int / fixed / decimal / bool — RESOLVED

Line 176 covers all four cases: mathematical-value equality (`int8(-1) != uint8(255)`), cross-family hash agreement, decimal-family deferral to Phase 28 policy, and bool not aliasing `1` as a dict/set key. One small residual ambiguity is flagged below as polish item 1.

### 7. Constant folding scope for narrowing — RESOLVED

Lines 73–80 specify exactly what counts as const-evaluable: literals, unary `+`/`-`, integer `+`/`-`/`*`, shifts with fitting constant shift counts, parentheses, and immutable module constants whose initializer is itself const-evaluable. Runtime-dependent conditionals, function calls, collection lookups, and non-constant names do not participate. Three worked examples cover the boundaries. This is implementable as-stated.

### 8. Generic specialization and pattern matching edge cases — RESOLVED

Lines 296–311 give the literal-pattern fitting rule with an `unreachable` example for `case 256:` against `uint8`, and explicitly state generic-container invariance (`list[int]` is not assignable to `list[int32]` and vice versa without explicit element-wise conversion). Both pass-1 sub-questions answered.

---

## Pass-1 non-blocking suggestions — status

| Suggestion | Status |
| --- | --- |
| A. Name `SifrInt` runtime crate, ABI, trait surface | Resolved — `num_bigint::BigInt`, traits enumerated, no `repr(C)` (lines 333–340) |
| B. Show `?`-propagation for `int16(value)` Result | Not addressed — minor; show one example |
| C. JSON number parsing rule explicit | Resolved — line 222: token with no `.`, `e`, `E` is `int` |
| D. OpenAPI `format: integer-decimal-string` flagged | Resolved — line 239 names `x-sifr-format` extension |
| E. Reserve `int128`/`uint128` | Resolved — line 35 |
| F. wasm32 / 32-bit `usize` story | Resolved — line 317 |
| G. Resource-limit defaults | Resolved — 4096-digit default, line 281 / 325 |
| H. Formatting/repr behavior | Resolved — line 346 |
| I. `bit_length`/`bit_count` scope | Resolved — line 346 reserves the surface, defers contracts to stdlib phase |
| J. Slice 8 borrow-checker impact | Resolved — split into representation-first (slice 2) and ownership propagation (slice 9) |

---

## Polish items (non-blocking, do before lock or in the same edit)

### Polish 1. `True == 1` is still "false or a type error"

Line 176: "`True == 1` is false or a type error according to the final comparison policy". This is the only place in the doc that defers a decision. It is a small surface — a one-line commitment ("`True == 1` is a compile error; users write `int(True) == 1` to compare across types") removes the "to be decided later" footnote. Otherwise, equality across `bool` and `int` will resurface as a pattern-matching question (`match x: case True:` against an `int` subject) at implementation time.

Recommendation: pick "compile error" (consistent with no-implicit-narrowing and bool-not-int). One sentence.

### Polish 2. Float-representability proof in `int / int` is rarely achievable

Line 92: `int / int -> Result[float, DivisionError | FloatOverflowError]` unless non-zero and float-representability are both proven. The compiler can prove non-zero in a few cases (literal divisor, refinement-flow narrowing), but float-representability of an arbitrary `int` is essentially never statically provable outside literal arithmetic. The practical effect is that virtually every `int / int` becomes fallible, which is a real ergonomic constraint for ordinary code (`average = total / count` requires `?` or `match`).

This is a defensible design choice — it forces users to make the representational question visible — but the issue should call that effect out so readers do not assume `int / int` is usually inferred as plain `float`. A one-line note ("In practice, `int / int` is fallible at the call site for non-literal operands; this is intentional, since silent precision loss across the int→float boundary contradicts the no-silent-loss core rule.") would set expectations.

Alternative if the friction is unacceptable: introduce a `safe_float_div` or require `//` for integer-preserving and `Decimal` / `float(...)` for explicit precision-loss paths, with `/` reserved as a syntax for the explicit non-integer numeric operation rather than a fallible bridge. This is a larger redesign and not necessary if the call-site friction is acceptable.

### Polish 3. `int * int` allocation behavior is documented inconsistently

Line 96 says straight-line `+`, `-`, `*` remain `int`-returning, with "resource exhaustion outside configured safe profiles … treated as process resource exhaustion, not integer overflow." This is the right pragmatic call (otherwise `a * b` with two large `int`s would have to be fallible, and that would poison ordinary arithmetic).

But the no-panic guarantee in `AGENTS.md` ("if it compiles, it works — no user-triggerable runtime panics") and the language target's reliance on Rust's allocation behavior need a one-sentence reconciliation. As written, a hostile actor who controls two `int` operands can force OOM through repeated multiplication. The doc handles this well for `**`/`<<` (budgeted) and for parsers (digit caps), but is silent on plain `*`. The honest framing — already implied — is that arithmetic is bounded by process memory, that's an OS-level concern, and parsers/external boundaries are responsible for bounding the magnitude of `int` values that reach `*`. State this explicitly so the no-panic guarantee is understood as "no user-triggerable *typed* runtime panic" rather than "no OOM ever".

Suggested addition near line 96:

> The no-panic guarantee covers typed runtime errors. Process-level resource exhaustion (allocator failure, stack exhaustion) is an operational concern bounded by parser limits and configured arithmetic budgets, not by the `int` type itself.

---

## Things newly added that I think are right

- Slice 2 reorder ("Implement `SifrInt` representation and source-level value semantics before changing broad operator codegen") — directly addresses pass-1 J and is the right ordering. Without `SifrInt` shape pinned, every operator codegen decision would have to be redone.
- Compiler architecture impact item 9 — making explicit that `int` is "value-semantic but no longer a Rust `Copy` scalar, while allowing optimizer/codegen passes to use Rust primitive locals when statically sound". This is exactly the contract the HIR/codegen need.
- Reserved future names `int128`/`uint128` (line 35) — small but cheap.
- The wasm32 carve-out for `usize` width (line 317) — important and easy to forget.
- The bytes/`array[uint8]` distinction with future zero-copy views as an explicit, lifetime-typed feature — the right design boundary, not a hack.

---

## Summary

The eight pass-1 blockers are all resolved with implementable contracts. Slice ordering now respects the `SifrInt`-first dependency. The three polish items are small, single-line edits and do not change any implementation slice. Lock as the source of truth after polishing items 1–3, or merge as-is and treat them as follow-up cleanups — either is defensible.

Recommendation: **lock**, with polish 1–3 either applied in the locking commit or filed as a tiny follow-up.
