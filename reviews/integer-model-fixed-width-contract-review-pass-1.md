# Review: Ad-Hoc Phase — Integer Model and Fixed-Width Numeric Contract (Pass 1)

Reviewer: Claude Opus 4.7
Date: 2026-05-05
Source: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`

## Verdict: REVISE

The high-level direction is right and the issue is the strongest numeric design Sifr has had so far. The "exact `int` by default, explicit fixed-width for storage/FFI" split is correct for a Python-replacement that must also do dataframes, AI tensors, and Rust interop. The promotion-to-`int` arithmetic rule is the single most important call and I think it is the right one for scalar code.

However, several load-bearing decisions are either under-specified or contradict goals stated elsewhere in the same document. I would not lock this as the source of truth in its current form. The blockers are concentrated in: array/tensor element-wise arithmetic semantics, `int` performance representation, division/exponent/shift result contracts, and the bytes/`uint8` ergonomic boundary. These are answerable today and will save thrash during the implementation slices.

The rest of this review is grouped as: blocking findings (must answer before locking), then non-blocking suggestions.

---

## Blocking findings

### 1. Array/tensor element-wise arithmetic must carve out of the promotion rule

Section "Arithmetic Rules" says ordinary `i32 + i32 -> int`. Section "Bytes, Data Science, and AI" says fixed-width is the dtype for arrays/tensors. These two rules collide for the most common data-science expression in the language:

```python
xs: array[int32] = ...
ys: array[int32] = ...
zs = xs + ys     # what is the dtype of zs?
```

If the scalar promotion rule applies element-wise, `zs: array[int]`, which means every batched arithmetic op produces a heap-allocated arbitrary-precision element column. That is unusable for AI/DS — it defeats the whole reason fixed-width dtypes exist.

The issue must say explicitly that array/tensor element-wise arithmetic **stays in the source dtype** and that overflow policy on a fixed-width dtype column/tensor is part of the dtype itself (or part of the arithmetic call). Most likely:

- default `array[int32] + array[int32]` is a typed error unless the user opts into a wrapping/saturating/checked policy, because the no-panic guarantee means we cannot silently wrap and we cannot "expand to bigint" element-wise without breaking storage; or
- there is a per-array overflow mode (`array[int32, on_overflow=wrap]`) that NumPy-style wraps deterministically, with `add_checked`/`add_saturating` operator-equivalents available, and the default surface for unchecked `+` returns `Result[array[int32], OverflowError]`.

Pick one and write it. Without this, the design contradicts itself the moment array types land.

### 2. `int` performance representation needs a concrete contract, not "may optimize"

"Performance Contract" says the compiler "may" optimize proven-small `int` locals into Rust primitives, with `SifrInt` as a wrapper over `num_bigint::BigInt` otherwise. For a Python-replacement that also targets web/AI:

- `num_bigint::BigInt` heap-allocates a `Vec<u64>` for every value, including `1`. Loop counters, `len()`, dict keys, and indices that go through `int` will pay an allocation per iteration unless escape analysis is perfect — and Rust does not give Sifr that for free.
- "Compiler may optimize" is not a contract a downstream library author can rely on. Library boundaries kill optimizer visibility.

The issue should commit to a small-int representation in `SifrInt` itself. The standard answer is a tagged niche: an inline `i64` (or `i128`) for values that fit, escaping to a heap-allocated `BigInt` only when they overflow. Either:

- Use `malachite::Integer` (already does this) instead of `num_bigint::BigInt`; or
- Define `SifrInt` as `enum { Small(i64), Big(Box<BigInt>) }` and document the inline range.

This belongs in the design doc because it changes (a) the size and Copy-ness of `SifrInt`, (b) the FFI shape of generated functions taking `int`, and (c) what "the compiler may optimize" actually means. The issue currently leaves all three open and they are not implementation details — they affect every generated function signature in the language.

Related: the doc says "no public guarantee that `int` storage is small or `Copy`". That is fine as a *public* guarantee, but the *implementation* needs to pick one because today's choice (`num_bigint::BigInt`) makes `int` non-`Copy` and ~3 pointers wide, which silently changes ownership/borrow behavior across the entire HIR. The issue acknowledges this in slice 8 but does not commit to an answer.

### 3. Division, exponent, and shift result contracts are vague or wrong

The arithmetic table has gaps:

- `int / int`: "follows the general Sifr division policy; if it produces float, overflow/invalid conversion must be explicit in the result contract." This is a punt. The issue is locking the integer model — pick a result type. Three candidates: (a) `Result[float, ...]` always (Python-3-ish), (b) `decimal`/rational (mathematically exact), (c) compile error, force `//` or explicit `float(...)`. Without an answer, codegen cannot be written.
- `int ** int` for negative exponent returning `Result[int, ValueError]` is questionable. Python returns a `float` for negative integer exponents (`2 ** -1 == 0.5`). Returning an `int`-typed Result with `ValueError` is surprising because the operation is well-defined mathematically — it just is not an integer. Either change the result type to `Result[float, ...]` / `decimal` / a rational, or explain why `**` on integers refuses to return non-integers. Note that Sifr having no implicit decimal conversion (per existing policy) makes the rational-result case a real design question.
- `int ** int` for very large positive exponents (`2 ** (10 ** 18)`) is the obvious DoS. The issue mentions parser limits but not arithmetic-result limits. Specify: does `int.pow(b)` cap by output bit-length, by exponent magnitude, or not at all? "No user-triggerable runtime panic" means OOM-from-allocation is on the table, and the answer is either (a) `pow` returns `Result[int, OverflowError]` with a configurable bit limit, or (b) `pow` is best-effort-but-OOM-aborts (which violates the no-panic guarantee).
- Bit shifts on `int`: undefined in the doc. `1 << (10 ** 18)` is the same DoS. Specify a result contract for shifts on `int`.
- `int // 0` and `int % 0` correctly return `Result[int, DivisionError]`. Good, but the same rule must be stated for fixed-width division (`int32 // int32` is currently not addressed at all in the table).

### 4. `bytes` element type contradicts the no-implicit-narrowing rule

"A byte element is externally observed as `int` for Python compatibility, while the representation uses `uint8`/Rust `u8` internally."

This breaks two stated invariants:

- Performance: `for b in some_bytes:` materializes a `SifrInt` per byte. For a 1 MB buffer that is potentially 1 M heap allocations unless the optimizer eliminates them perfectly. See blocker 2 for why "the optimizer will fix it" is not enough.
- Symmetry: writing into `bytes` requires fitting into `uint8`, so writes are fallible-narrowed from `int`, but reads silently widen. That asymmetry is exactly the kind of thing that makes Python-shaped code subtly slow on Sifr.

Two coherent options:
- `bytes[i] -> uint8` (matches the dtype design from the same doc; users widen with `int(b)` if they want exact), making `bytes` a typed `array[uint8]`-like; or
- `bytes[i] -> int` but commit explicitly to the small-int representation in `SifrInt` and document this is one of the cases that motivated the inline tag.

The issue should not have it both ways. Picking option 1 (return `uint8`) is more honest about the dtype model already established for arrays and avoids the implicit widening that the rest of the doc forbids.

Related: the relationship between `bytes` and `array[uint8]` is not stated. Are they aliases, related views, or unrelated types? This needs one sentence.

### 5. Fixed-width-to-fixed-width promotion rule is silently load-bearing

"Arithmetic between two fixed-width integer operands also promotes to `int` for ordinary `+`, `-`, `*`, `//`, `%`, and `**`."

The rule is good for ergonomics but has two unaddressed consequences:

- **Generic numeric code.** Any `def sum[T: Numeric](xs: list[T]) -> T` is impossible to write because for `T = int32`, `T + T -> int`, not `T`. Either Sifr does not allow generic arithmetic over fixed-width types, or `Numeric` becomes a non-trivial trait that exposes a concrete element type and a separate accumulator type. Pick one; today this design effectively forbids ergonomic generic numerics.
- **`int` + `usize`/`isize` is unspecified.** If a Rust FFI returns `usize` and a user adds `1`, what happens? Pointer-sized types are listed as types but excluded from the "fixed-width" promotion language. The issue should either say `usize`/`isize` follow the same promotion rule (then they barely deserve their own row in the source-level type table) or stay representational (then `usize + 1` is fallible/a type error).

### 6. Equality across signed/unsigned and decimal needs to be explicit

"If `int` compares equal to a fixed-width integer, hashes must agree" — good, but only the easy case.

The hard cases:

- `int8(-1) == uint8(255)`? Mathematically these are different values (-1 ≠ 255). The wrapping representation must not be observable as equality. State this so future implementers do not bit-compare.
- `int(1) == decimal("1.0")`? Cross-numeric-family equality and hashing is referenced in passing under "Decimal mixing" but not for `==`/`hash`. Phase 28 policy may already cover this; either link to it or restate.
- `bool` as a separate type — say once whether `int(True)` is allowed, whether `True == 1` is allowed, and whether `True` in an `int`-keyed dict aliases. Python's behavior is inherited by accident if the doc is silent.

### 7. Literal range checking needs to define how far constant folding goes

`x: uint8 = 255` ok; `x: uint8 = 256` compile error; good. But:

- `x: uint8 = 1 - 2`?
- `x: uint8 = 100 + 200`?
- `x: uint8 = SOME_CONST` where `SOME_CONST: int = 100`?
- `x: uint8 = if cond: 10 else: 20`?

The "fits-the-literal" rule is a fits-the-constant rule once any folding happens. Without a clear bound on how aggressive const folding is for these checks, two implementations will diverge and users will hit "obvious" cases that mysteriously do not narrow. State the bound: literal-only, full const-eval over pure expressions, or only over compile-time-known constants.

### 8. Generic specialization and pattern matching are referenced but not specified

- `match` over an `int` value with literal arms (`case 0:`, `case 1:`): allowed? Across types? `match x: int8 with case 256:` should be a compile error by the same rule as literal narrowing.
- Generic specialization with `int` vs fixed-width is mentioned ("no implicit narrowing in ... generic specialization"). What does that imply for `list[int]` flowing into a function expecting `list[int32]`? Almost certainly a compile error; say so.

These will appear during implementation slice 5 and stall it if not pinned now.

---

## Non-blocking suggestions

### A. Name the `SifrInt` runtime crate and ABI in this issue

Since the Performance Contract section ties into ownership and codegen, name (a) the chosen big-int dependency, (b) whether `SifrInt` is `Copy`/`Clone`/`Send`/`Sync`, and (c) whether `SifrInt` has a `#[repr(C)]` ABI for FFI extern modules. These are decisions that will pop out during slice 4.

### B. Add a short table for narrowing constructor naming

The doc uses `int16(value)` returning `Result[int16, OverflowError]`, `int32.checked_add`, `int32.saturating_add`, `int32.wrapping_add`, `int32.overflowing_add`. Two minor consistency issues:

- The constructor returns a `Result`, but the syntax `int16(value)` looks like an infallible call. In Sifr's syntax this presumably forces `?` propagation or `match`. Show one example with `?` so readers see how it looks at a call site.
- `checked_*`/`saturating_*`/`wrapping_*`/`overflowing_*` is a verbatim Rust naming carry-over. That is fine but call it out so the implementation slice does not bikeshed.

### C. State the JSON number parsing rule explicitly

JSON numbers are not syntactically partitioned into int/float. The doc says the parser "should parse integer number tokens into exact `int` values", but a JSON token `1.0` is a number too. Spell out: a JSON number with no `.`, `e`, or `E` is parsed as `int`; otherwise `float` (or `decimal` per Phase 28 policy). Otherwise implementers will pick differently.

### D. OpenAPI `format: integer-decimal-string` is non-standard; flag it

The proposed `format: integer-decimal-string` is a Sifr extension, not an OpenAPI/JSON Schema standard format. Either pick a published convention (some teams use `type: string` + `pattern: '^-?[0-9]+$'` + custom `x-sifr-int`) or commit to introducing it as a Sifr-defined extension and document the keyword name. This will affect generated client SDKs.

### E. Reserve `int128`/`uint128`?

The fixed-width row stops at 64. Rust has stable `i128`/`u128`. Some real targets (Postgres `numeric`, certain timestamp/currency formats, IPv6 addresses) want 128-bit fixed-width. You do not have to ship them now, but reserving the names so a future user-defined type does not collide is cheap.

### F. Consider a stricter `range`/`len` story on 32-bit targets

`len` returns `int`, with internal `usize` conversion. On a 32-bit target (wasm32, embedded), Sifr `int` values up to `2^32` are addressable; on 64-bit they are addressable up to `2^64`. The issue says materialization is fallible when "the length cannot fit addressable memory", which is correct, but: comparisons against `len` in tight loops (`for i in range(len(xs)):`) will go through `int` arithmetic. If blocker 2 is resolved with an inline-small-int `SifrInt`, this is fine. Worth a sentence calling out the wasm32 case explicitly since web is a stated target.

### G. Resource-limit defaults belong in the issue, not "to be configured"

Section "Parsing and Resource Limits" says limits exist but does not state the defaults. Pick concrete numbers (e.g., 4096-digit max for JSON/CSV/env, 1 MiB max document, configurable up). Without defaults, the same fixture parses differently across projects and the no-panic guarantee is dependent on every parser caller setting them.

### H. State the formatting/repr behavior briefly

`f"{x}"` for an arbitrary-precision `int` is unbounded. Format specs like `{x:08b}` or `{x:032x}` truncate vs. pad questions arise. One sentence saying "format specs are honored as in Python; bounded specs do not truncate, they pad up to the natural width" or similar will save a future debate.

### I. Consider adding `int.bit_length()` / `int.bit_count()` as part of the contract

Both are on Python's `int` and are useful in DS/AI work. Not specifying them is fine, but mention scope: "stdlib will provide bit_length, bit_count, to_bytes, from_bytes; these are not part of this issue's contract."

### J. Implementation slice 8 underplays the borrow-checker impact

"Teach ownership/codegen that source-level `int` is no longer a trivial `Copy` scalar" — this is a much larger change than slice 8 implies. Every `let x: int = ...` followed by `f(x); g(x)` becomes a borrow-checker question. If `SifrInt` is heap-backed, every `Clone` is an allocation. This argues again for resolving blocker 2 before this slice starts. Consider splitting slice 8 into "decide representation" and "propagate ownership effects".

---

## Things to keep as-is

- No bare `uint`. Correct call. The unsigned-subtraction trap is the single biggest source of off-by-one bugs in C/C++ code that Python users would not anticipate.
- Fixed-width-to-`int` promotion for ordinary arithmetic. Right call for application code; just needs the array carve-out (blocker 1) and the generic-numeric note (blocker 5).
- Named `checked_*`/`wrapping_*`/`saturating_*`/`overflowing_*` rather than overloaded operators. Right — overflow policy must be visible at the call site.
- JSON profiles (`exact`, `web`, `string_ints`) with the web profile defaulting for untyped public responses. Captures the real-world JS-safe-integer trap correctly and gives schema-driven APIs an opt-out.
- Removing user-facing `bigint`. Correct given pre-production status; the temporary parser-alias suggestion is the right transition path.
- Indexes/lengths as `int`, with `usize` reserved for FFI. Right ergonomic call; matches the no-bare-`uint` decision.
- Negative literal rejected for unsigned at compile time. Correct and consistent.
- Bool not subclassing int. Correct break from Python. Just specify the conversion semantics (suggestion 6).
- DataFrame/Arrow/Parquet/tensor dtype-driven loading, refusing to default to `int` for columnar data. Correct; this is the design that makes Sifr usable for AI pipelines.

---

## Summary

Direction: correct. Lock-readiness: not yet.

Resolve blockers 1 (array element-wise), 2 (`SifrInt` representation), 3 (division/exponent/shift contracts), and 4 (`bytes` element type). Blockers 5–8 are smaller but each one will cost an implementation slice if left open. Once those eight items have one-line answers in the issue, this is a clean target architecture and the implementation slices should be straightforward.
