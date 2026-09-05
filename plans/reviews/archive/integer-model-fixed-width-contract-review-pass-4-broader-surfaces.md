# Review: Ad-Hoc Phase — Integer Model and Fixed-Width Numeric Contract (Pass 4, Broader Surfaces)

Reviewer: agent
Date: 2026-05-05
Source: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`
Pass 1: `reviews/integer-model-fixed-width-contract-review-pass-1.md`
Pass 2: `reviews/integer-model-fixed-width-contract-review-pass-2.md`
Pass 3: `reviews/integer-model-fixed-width-contract-review-pass-3.md`

## Verdict: LOCK-READY

The principal-engineering expansion holds together. The new sections — type inference/defaulting, builtins/stdlib surface, web/validation/public API models, identifiers/time/domain values, diagnostics/DX, and the validation matrix — are coherent with the locked core (passes 1–3) and don't introduce contradictions with the scalar/array/serialization rules. None of the items below block lock; they are polish opportunities and follow-up notes that can be applied in the lock commit or filed as small follow-ups.

The recommendation is: **lock pass 4** and proceed to slice 1 (lock + update architecture references). Address P1, P2, P3, P5, and P10 either inline or as small follow-up edits — they are the items most likely to come up during the first implementation slice.

---

## Actionable findings

### P1. The "fixed-width signature returning fixed-width" ergonomic cost is implicit

The promotion-to-`int` rule (line 162) plus no-implicit-narrowing (lines 76–77) means everyday fixed-width API authors must wrap arithmetic in fallible constructors:

```python
def increment(port: int16) -> int16:
    return port + 1            # compile error: int is not assignable to int16
    return int16(port + 1)     # required form, fallible
```

This is a real cost paid by anyone writing fixed-width-shaped library code, and it's the natural consequence of the design — but the "Type Inference and API Defaulting" section (lines 222–246) does not call it out. Readers will assume from the Python-shaped surface that this is ergonomic. The fix is one paragraph in that section pointing fixed-width API authors at the explicit alternatives:

- `int16(a + b)` for fallible narrowing
- `int16.checked_add(a, b)` / `wrapping_add` / `saturating_add` for representation-preserving arithmetic
- `int` accumulator + final narrowing for pipelines

This isn't a redesign; it's setting reader expectations correctly so the implementation team doesn't get pulled into "make `port + 1` infer as `int16`" requests during slice 6.

### P2. `**` participation in compile-time fitting is unspecified

The const-evaluation rule (lines 76–79) lists `+`, `-`, `*`, shifts, parens, and immutable module constants. It does not mention `**`. But the container example `e: list[int32] = [1, 2, 10 ** 100]` (line 240) is described as a compile error in a way that reads as if the compiler proved `10 ** 100` does not fit. Either path leads to a compile error (the runtime-typed `**` returns `int`, and `int → int32` requires explicit narrowing), but the example reads as if `**` participates in fitting.

Pick one and write it:

- Extend const-eval to include `**` with the same `ArithmeticLimitError` budget that runtime `**` has, so `x: int32 = 10 ** 3` is allowed and `x: int32 = 10 ** 100` is a fitting failure with a clear message.
- Or state explicitly that `**` does not participate in const-eval fitting, so `x: int32 = 10 ** 3` is a compile error pointing the user at `int32(10 ** 3)`.

The first option is more ergonomic and consistent with the "compile-time fitting rule applies to const-evaluable integer expressions, not only single tokens" framing. Either way, name the choice.

### P3. JSON profile applies element-by-element for collections

The web/serialization rules (lines 264–286) and the JSON profile table (lines 326–332) describe field-level behavior. But `payload: list[int]` with a value `[1, 2, 10**20]` under `json.web` raises the same JS-safe question one element at a time. The doc doesn't say whether:

- the profile applies per-element (most consistent with the field-level rule),
- the field-level "string-encoded" opt-in extends to "string-encoded list elements" automatically, or
- collection elements use a separate sub-profile.

Add a one-liner to the JSON section: "Profile rules apply element-by-element for `list`, `dict`, and other collection-valued fields. A field opting into string encoding string-encodes its elements." This avoids a real bug-class for the first user who has `transaction_ids: list[int]` in a public API.

### P4. `int64` IDs under the default `json.web` profile are a footgun

The framework default for browser-facing APIs is `json.web` (line 273). Many real systems have `int64` IDs that exceed `2^53` (Twitter Snowflake, Discord, Stripe). The example model at lines 280–283 uses `id: int64` and notes "JSON number only while web-safe or schema-forced", but nothing in the type system warns at compile time when an `int64` field is exported under `json.web` without an explicit string-encoding opt-in. The first failing serialization will happen at runtime, with a real ID, in production.

Two options, both small:

- **Diagnostic option**: add a `SIFR-*` family for "fixed-width integer field exceeds JSON-web safe range without explicit serialization policy". Emit it at model-export time when a `int64`/`uint64`/`uint32` (whose max exceeds `2^53`) field appears in a `json.web`-defaulted response model without an explicit string-encoding annotation. This is a compile-time nudge; it doesn't change semantics.
- **Docs/default option**: state that `int64`/`uint64` ID fields should default to string encoding under `json.web`, and the framework provides a one-line annotation to opt out for fields that are guaranteed safe.

The doc currently leaves this entirely to the user's vigilance. Worth an explicit decision.

### P5. Dtype-preserving `sum`/`min`/`max`/`abs` API names are unspecified

Stdlib row (line 255): "`sum(list[int32])` returns `int` by default; dtype-preserving sum is an explicit checked/wrapping/saturating API." The name of that API is not pinned. Candidates:

- `int32.checked_sum(xs)` — consistent with `int32.checked_add` naming
- `xs.checked_sum()` — method on the iterable
- `sum(xs, dtype=int32)` — dtype-parameterized builtin
- something else

The naming bikeshed will land in stdlib slice 8. Pin it now to remove a coordination cost. The first form (`int32.checked_sum(xs)`) is the most consistent with the existing fixed-width method surface.

Same question applies to `min`/`max`/`abs` element-wise over fixed-width arrays, which are not explicitly mentioned in the array/tensor carve-out (lines 214–219). The carve-out only lists `_add`. Worth a one-liner that the carve-out covers all element-wise arithmetic and reductions, with the same checked/wrapping/saturating/widen naming.

### P6. `int + float` and the no-silent-loss rule

The arithmetic table (lines 91–97) handles `int / int → Result[float, ...]` because `int → float` can lose precision. The "Decimal mixing" paragraph (line 181) defers `float` mixing to "the existing float policy, with no implicit decimal conversion". But the same precision-loss concern that makes `int / int` fallible also applies to `int(2**60) + 1.0`: silently converting `int` to `float` for the operation discards mantissa bits.

If the existing float policy is "implicit widening of `int` to `float` is allowed when the operation is float-typed", that contradicts the no-silent-loss core rule for large `int` values. Three options:

- Restate the rule explicitly here: `int op float → Result[float, FloatPrecisionLossError]` when the `int` exceeds float-representable precision, otherwise `float`.
- Cross-reference the Phase 28 numeric policy with a one-liner ("see Phase 28 §X for `int op float` precision-loss handling").
- Decide that `int op float` requires explicit `float(...)` conversion at the call site (consistent with `/`).

The doc currently leaves this to readers' inference of "the existing float policy", which is the kind of gap that surfaces at implementation slice 5. Pin the answer.

### P7. Newtype mechanism is referenced but not specified

The "Identifiers, Time, and Domain Values" section (lines 288–297) recommends "nominal newtypes over raw `int64`/`uint64` in domain models". This is correct principal-engineering guidance, but Sifr's newtype facility isn't named. Without a named mechanism, the first implementations of "DB ID newtypes" will use raw `int64`, accumulate migration debt, and the section becomes aspirational documentation.

Either:

- Point at the existing Sifr newtype/branded-type mechanism by name.
- Flag this section as dependent on a future newtype RFC, so readers don't expect to act on it during slice 1.

This is not a blocker for the integer model itself, but the section currently reads as actionable advice when it may not be.

### P8. Validation matrix is incomplete

The matrix (lines 480–490) has good coverage but is missing several rows that the doc has substantive rules for:

- **Pattern matching with literal arms** — positive: in-range literal arms; negative: out-of-range literal pattern (lines 398–409 already specify this; the matrix should reference it).
- **Mixed numeric arithmetic** — `int + decimal`, `int8 + decimal`, `int + float` (covered in lines 178–181 but not in the matrix).
- **Format strings and integer methods** — `f"{x:08x}"`, `bit_length`, `bit_count`, `to_bytes`, `from_bytes` (line 446).
- **Range and large-bound iteration** — lazy `range(10**100)` iteration; materialization OOM/typed error (line 415).
- **`usize`/`isize` boundary** — positive: FFI-shaped signature accepts `usize`; negative: `usize` leaking into application code without explicit narrowing.
- **Performance regression** — inline-small-int operations should not allocate in tight loops (the `SifrInt` `Small(i64)` arm exists for this; the matrix should ensure it's tested).
- **Cross-type dict/set lookup** — `dict[int64, T]` lookup with `int(5)` requires explicit `int64(5)`; positive and negative cases.
- **Equality/hash across families** — already alluded to in scalar `int` row, but should be its own row given how load-bearing it is.

The matrix is fine for what it covers; rounding it out closes obvious test-coverage gaps before slice 8.

### P9. Diagnostic for `port + 1`-style narrowing failure deserves explicit mention

The "Diagnostics and Developer Experience" section (lines 299–312) lists "implicit narrowing attempt" as a diagnostic family. The most common form of this in real code is the P1 scenario: a function with a fixed-width return type returning `param + 1`. The diagnostic should specifically recognize that pattern and suggest:

- `int16(port + 1)` for fallible narrowing, or
- `int16.checked_add(port, 1)` for explicit checked addition.

Listed as its own bullet in the diagnostic families, not just a sub-case of "implicit narrowing attempt". This is the diagnostic that will fire most often during the first month of fixed-width usage; it should have a tailored message and a `SIFR-*` code reserved.

### P10. `int` field in `repr(C)` struct — unaddressed

The doc states `SifrInt` is not `#[repr(C)]` (line 440) and that FFI uses fixed-width or a future bignum handle. But Sifr classes/dataclasses with `int` fields produce non-`repr(C)` structs by transitivity. Users wanting C-ABI-stable struct layouts must use only fixed-width fields for the integer slots. This is the correct answer, but it's not stated.

One sentence in the Rust Interop section: "Sifr structs containing `int` fields are not C-ABI compatible; use only fixed-width integer types for FFI struct fields." Saves a "why doesn't my `extern struct { x: int }` work?" question.

### P11. Port number example uses `int16` but ports are unsigned

Line 16 example: `port: int16 = 5432`. The "Identifiers, Time, and Domain Values" guidance (line 294) says "ports, status codes, byte values, and protocol fields: use fixed-width/newtype wrappers". Ports are unsigned 16-bit (0–65535). The natural fixed-width is `uint16`, not `int16`. The example contradicts the section's own guidance.

Minor; replace the example or pick a different domain value (e.g., `int16` could be reasonable for a file descriptor, sequence number, or audio sample).

---

## Internal consistency verification

I cross-checked the new sections against the locked core for contradictions. None of the rules in passes 1–3 are weakened.

| Pair of rules | Consistent? | Notes |
| --- | --- | --- |
| Container inference (lines 236–241) vs. invariance (line 411) | Yes | `c = [int32(1), 2]` infers `list[int]` because the fixed-width value widens; `d: list[int32] = [1, 2, 3]` is allowed only when literals fit, consistent with the contextual fitting rule. |
| `sum(list[int32]) → int` (line 255) vs. fixed-width promotion (line 162) | Yes | The default `sum` follows the scalar promotion rule. Dtype-preserving sums use the array carve-out's checked/wrapping/saturating naming. |
| `min/max(list[int32]) → int32` (line 256) vs. promotion rule | Yes | min/max selects existing values; no arithmetic is performed, so promotion doesn't apply. Subtle but correct. |
| `abs(int8) → int` (line 257) vs. `int8::MIN.abs()` overflow | Yes | Widening to `int` is the only way to represent `abs(int8::MIN)` exactly; this matches the no-silent-failure rule. |
| `json.web` default (line 273) vs. `int64` IDs in models | Consistent but a footgun | See P4. The rules don't contradict; they just put the burden on the user to opt fields into string encoding. |
| Route `int` parsing under digit limit (line 270) vs. parser limits (line 425) | Yes | Web boundary inherits the 4096-digit default; per-route stricter limits are allowed. |
| `int / int → Result[float, ...]` (line 97) vs. `random.randrange` accepting `int` (line 259) | Yes | `randrange` doesn't divide; it samples. |
| Diagnostics families (lines 304–311) vs. behaviors specified earlier | Yes | Each family corresponds to a specified rule. The "bool/integer comparison" bullet matches the line 183 compile-error decision. |
| Validation matrix Domain newtypes row vs. "Identifiers, Time, and Domain Values" | Yes | Both push toward newtype-over-fixed-width; matrix exercises the "treating wrappers as raw ints" failure mode. |
| Stdlib `sum`/`min`/`max`/`abs` vs. array dtype carve-out | Yes — but extension to all reductions is implied, not stated | See P5. The current text only carves out element-wise `_add`; reductions like `sum`/`max` aren't explicitly named. |

The new sections compose cleanly with the locked rules. Where ambiguity exists (P2, P3, P5, P6), it is in the boundary cases between sections, not in the core scalar/dtype contracts.

---

## Quality observations

A few things worth keeping for the implementation team:

- **The "Type Inference and API Defaulting" section is the right shape.** Stating that "fixed-width types appear only from explicit annotations, constructors, imported schemas, FFI signatures, or dtype declarations" (line 229) closes the door on inference picking up width by accident — which is exactly the surprise that breaks Python-shaped intuition. The container inference table (lines 236–241) is concrete enough to ship a HIR rule directly.

- **The "Web, Validation, and Public API Models" section makes an opinionated framework default visible.** `json.web` as the default for browser-facing APIs (line 273) is the correct call — most Sifr web users will be writing browser-targeted endpoints, and the JS-safe-int trap is the single most common JSON precision bug in the wild. The TypeScript client mapping (line 274) keeps the contract honest end-to-end.

- **The "Identifiers, Time, and Domain Values" section is the missing layer most numeric-model RFCs skip.** Saying explicitly that `decimal`/minor-unit newtypes are right for money, that timestamps belong to `datetime`/`duration` types, and that DB IDs should be newtypes over `int64` — these are the real-world recommendations a Python developer needs when they reach for `int` and get surprised. This section is the "what should I actually use" guide that pays off during user docs.

- **The diagnostics families section pre-commits to the right scope.** Naming each family (out-of-range, implicit narrowing attempt, unsafe `/`, missing array overflow policy, JSON serialization failure, bool/integer comparison) without pinning `SIFR-*` codes is the correct level of specificity. Future-you can register the codes in the diagnostic registry without rewriting the design doc.

- **The validation matrix anchors slice 8 work.** Even with the gaps in P8, the matrix gives implementation slices a concrete checklist of positive and negative cases per surface. Filling out the matrix during slices 5–9 is straightforward.

---

## Items intentionally left for follow-up phases

These are correctly out of scope:

- Detailed contracts for `bit_length`, `bit_count`, `to_bytes`, `from_bytes` (line 446) — surface reserved, contracts deferred to stdlib phase. Right call.
- GraphQL/gRPC/etc. surface coverage beyond JSON+OpenAPI+TS — would expand scope unnecessarily; the JSON profile model generalizes.
- Full enum representation design (lines 421–422) — already deferred; constraining valued enums to `int64`-representable values until that phase lands is the correct interim stance.
- `int128`/`uint128` (line 35) — name-reserved; shipping is a future stdlib decision.
- Newtype/branded-type RFC dependencies (P7) — flagged here, but the integer model can lock without it.

---

## Things to keep as-is from prior passes

All decisions called "right" in passes 1–3 stand:

- No bare `uint`. Fixed-width-to-`int` promotion for ordinary arithmetic. Named `checked_*`/`wrapping_*`/`saturating_*`/`overflowing_*`. JSON `exact`/`web`/`string_ints` profiles. Removing user-facing `bigint`. `int` indexes/lengths with compiler-internal `usize`. Negative literals rejected at compile time for unsigned. `bool` not subclassing `int`. Dtype-driven Arrow/Parquet/tensor loading. `SifrInt::{Small, Big}` representation. `num_bigint::BigInt` named. wasm32/32-bit `usize` carve-out. Slice 2 ordering before broad operator codegen. Compiler architecture impact item 9 (`int` is value-semantic but not `Copy`).

The pass-4 expansion does not regress any of these.

---

## Summary

The pass-4 expansion is a coherent, principal-engineering-quality broadening of an already lock-ready design. The new sections compose with the locked core without contradiction. The eleven items above are polish opportunities — most are one-line clarifications that prevent re-debating decisions during the implementation slices.

Recommendation: **lock pass 4**. Mark the pass-4 checkbox on the issue's Review Status, and either fold P1–P5 / P10 into the locking commit or file them as immediate follow-ups before slice 1 begins. The remaining items (P6–P9, P11) are smaller still and can land alongside the relevant implementation slice.

The design is fit to serve as the source of truth for slices 2–10.
