# Ad-Hoc Phase: Integer Model and Fixed-Width Numeric Contract

## Objective

Choose Sifr's long-term integer model before production, with Python-simple defaults for web/app/data-science code and explicit Rust-compatible fixed-width access for storage, dtypes, binary protocols, and FFI.

Sifr is not production-released yet. This phase intentionally does not preserve old numeric compatibility. The goal is the clean target architecture.

## Decision

Sifr should make `int` the simple Python-like integer: exact, signed, and arbitrary precision. Normal application code should not have to choose a width, and normal integer arithmetic must not overflow, wrap, or panic.

Fixed-width integers are still first-class, but they are explicit types for storage, binary protocols, dataframes, tensors, and FFI:

```python
count: int = 10 ** 100
port: uint16 = 5432
mask: uint32 = uint32(0xff00)
payload: bytes = bytes.from_ints([0, 255])
features: array[float32] = ...
ids: array[int64] = ...
```

There is no bare `uint`. Unsigned arithmetic is too easy to misuse in Python-shaped code because subtraction and negative sentinel/index patterns are common. Users choose an explicit unsigned width only when the domain really is a non-negative fixed-width representation.

## Source-Level Types

| Sifr type | Meaning | Primary use | Rust representation |
| --- | --- | --- | --- |
| `int` | exact signed integer, arbitrary precision | default app/web/business/algorithm integer | `SifrInt`, an inline-small integer runtime type that spills to an arbitrary-precision backing only when needed |
| `int8`, `int16`, `int32`, `int64` | signed fixed-width integer | binary formats, DB/dataframe/tensor schemas, FFI, memory-sensitive storage | Rust `i8`, `i16`, `i32`, `i64` |
| `uint8`, `uint16`, `uint32`, `uint64` | unsigned fixed-width integer | bytes, protocols, IDs with external unsigned schema, FFI | Rust `u8`, `u16`, `u32`, `u64` |
| `isize`, `usize` | pointer-sized Rust interop integer | FFI boundary only | Rust `isize`, `usize` |

Reserve `int128` and `uint128` as future fixed-width type names. Rust supports `i128`/`u128`, and some storage/interop targets need 128-bit values, but Sifr does not need to ship them in the first fixed-width slice.

`bigint` should not remain a separate user-facing numeric type. It was a bootstrap answer to the old `int = i64` decision. Before Sifr reaches production, fold the behavior into `int` and either remove `bigint` or keep it only as a temporary parser alias during local transition work. Because Sifr has no production compatibility promise yet, the clean target is one exact default integer type.

## Literal and Conversion Rules

Unsuffixed integer literals are `int`.

```python
x = 42          # int
y = 10 ** 100  # int
```

Typed literal assignment to a fixed-width type is allowed only when the compiler proves the literal fits:

```python
x: int32 = 42              # ok
y: uint8 = 255             # ok
z: uint8 = 256             # compile error
n: uint8 = -1              # compile error
```

Runtime narrowing is explicit and fallible:

```python
def read_port(value: int) -> Result[int16, OverflowError]:
    return int16(value)

try:
    port: int16 = int16(user_input)
except OverflowError:
    port = int16(0)
```

Widening from every fixed-width integer to `int` is infallible:

```python
small: int32 = 7
wide: int = int(small)     # ok, exact
```

Conversions between signed and unsigned fixed-width types are explicit and fallible unless the source is a fitting literal. There are no implicit narrowing conversions in assignments, calls, returns, list literals, dict literals, or generic specialization.

The compile-time fitting rule applies to const-evaluable integer expressions, not only single tokens. The first implementation should support literals, unary `+`/`-`, integer `+`/`-`/`*`, shifts with fitting constant shift counts, parentheses, and immutable module constants whose initializer is itself const-evaluable. Runtime-dependent conditionals, function calls, collection lookups, and non-constant names do not participate; those require an explicit fallible constructor.

Integer exponentiation participates in compile-time fitting when both operands are const-evaluable, the exponent is non-negative, and the configured compile-time arithmetic budget is not exceeded. This makes `x: int32 = 10 ** 3` valid and `x: int32 = 10 ** 100` a range diagnostic rather than an inference mystery.

```python
x: uint8 = 100 + 27       # ok
y: uint8 = 1 - 2          # compile error
LIMIT: int = 200
z: uint8 = LIMIT          # ok when LIMIT is a const-evaluable module constant
```

## Arithmetic Rules

`int` arithmetic is exact and returns `int` for integer-preserving operators:

| Expression | Result |
| --- | --- |
| `int + int`, `int - int`, `int * int` | `int` |
| `int // int`, `int % int` | `Result[int, DivisionError]` unless non-zero is proven |
| `int ** int` | `int` when the exponent is proven non-negative and the output is within budget; otherwise `Result[int, ValueError | ArithmeticLimitError]` |
| `int << int`, `int >> int` | `int` when the shift is proven valid and within budget; otherwise `Result[int, ValueError | ArithmeticLimitError]` |
| `int / int` | `Result[float, DivisionError | FloatOverflowError]` unless non-zero and float-representability are both proven |

Integer `**` is intentionally integer-preserving. Unlike Python, `2 ** -1` does not implicitly become `0.5`; users write `float(2) ** -1`, `Decimal("2") ** -1`, or another explicit non-integer numeric operation when they want a non-integer result. That keeps integer code exact and prevents a runtime sign check from changing the result type.

In practice, `int / int` is fallible at the call site for non-literal operands. This is intentional: silent precision loss across the exact `int` to approximate `float` boundary would violate the no-silent-loss rule.

Exact integer operations are not allowed to trigger unbounded allocation from hostile input. External parsers enforce digit limits, and explosive operations such as exponentiation and left shift have a configurable maximum output bit length. Exceeding that budget returns `ArithmeticLimitError`. Straight-line `+`, `-`, and `*` remain `int`-returning; process-level resource exhaustion such as allocator failure or stack exhaustion is an operational concern bounded by parser limits and configured arithmetic budgets, not by turning ordinary `int` arithmetic into overflow-prone fixed-width arithmetic.

Fixed-width arithmetic does not silently wrap. The simple default rule is:

```python
a: int32 = 2_000_000_000
b: int32 = 2_000_000_000
c: int = a + b            # fixed-width operands promote to exact int for ordinary arithmetic
d: int32 = int32(a + b)   # explicit fallible narrowing
```

This keeps everyday arithmetic Python-simple while preserving fixed-width storage. It also avoids making every `i32 + i32` expression return a checked result in business code.

For code that specifically needs Rust fixed-width behavior, expose named operations rather than changing operator semantics:

```python
try:
    x: int32 = int32.checked_add(a, b)    # Result[int32, OverflowError]
except OverflowError:
    x = int32(0)

y: int32 = int32.saturating_add(a, b)
z: int32 = int32.wrapping_add(a, b)
r: tuple[int32, bool] = int32.overflowing_add(a, b)
```

The Rust-style names are intentional. The contract is that checked, wrapping, saturating, and overflowing behavior is always visible at the call site.

Bitwise operators on fixed-width integers may return the same fixed-width type because they operate within the representation rather than mathematical magnitude:

```python
mask: uint32 = left & right
try:
    shifted: uint32 = uint32.checked_shl(mask, amount)
except OverflowError:
    shifted = uint32(0)
```

Shift operators need checked semantics because invalid or oversized shifts are external input dependent. Prefer explicit `checked_shl`, `checked_shr`, `wrapping_shl`, and `wrapping_shr` APIs for fixed-width values.

Fixed-width division, floor division, modulo, exponentiation, and shifts use the same no-silent-failure rule. Ordinary scalar `int32 // int32` promotes to `Result[int, DivisionError]`; representation-preserving fixed-width variants live behind explicit checked/wrapping/saturating APIs or dtype-specific array kernels.

## Comparisons and Mixing

Comparisons between `int` and fixed-width integers are allowed and exact:

```python
limit: int = 10 ** 20
narrow: int64 = 9
ok: bool = narrow < limit
```

Arithmetic involving `int` and a fixed-width integer returns `int` unless the operator is representation-specific:

```python
x: int64 = 5
y: int = x + 1
```

Arithmetic between two fixed-width integer operands also promotes to `int` for ordinary `+`, `-`, `*`, `//`, `%`, and `**`. This is the important simplicity rule: width is a storage/interface property, not a surprise arithmetic overflow policy.

`usize` and `isize` follow the same scalar promotion rule when they escape FFI-only signatures: ordinary arithmetic widens to `int`, and narrowing back to pointer-sized storage is explicit and fallible.

Generic arithmetic must model the operator output type. A generic `T + T -> T` bound is valid only for numeric types whose operator output is assignable to `T`. Fixed-width scalar types do not satisfy that bound for ordinary arithmetic because `int32 + int32 -> int`. Generic numeric helpers should either choose an explicit accumulator type or use fixed-width checked/wrapping/saturating protocol methods.

```python
def sum_int32(values: list[int32]) -> int:
    total: int = 0
    for value in values:
        total = total + value
    return total
```

Decimal mixing keeps the Phase 28 policy:

- `int + decimal` -> `decimal`
- `int + bigdecimal` -> `bigdecimal`
- fixed-width integer + `decimal`/`bigdecimal` first widens exactly to `int`, then follows the decimal policy
- `int` or fixed-width integer mixed with `float` is fallible unless the integer operand is proven exactly representable as `float`; otherwise the operation returns `Result[float, FloatPrecisionLossError]` or requires explicit `float(...)` conversion according to the final float operator lowering. There is no silent exact-integer-to-float precision loss.

Equality and ordering compare mathematical values, not bit patterns. `int8(-1) != uint8(255)`. Exact integer and decimal-family equality follows the Phase 28 exact numeric policy; when two hashable numeric values compare equal across exact numeric families, their hashes must agree. `bool` remains a separate type: `int(True)` is allowed as an explicit conversion, but `True == 1` is a compile error; users write `int(True) == 1` when they want that comparison. `True` must not alias `1` as a dict/set key.

## Indexing, Lengths, and `usize`

User-facing indexes and lengths stay `int`.

```python
size: int = len(items)
value: T | None = items[i]
```

Generated Rust may convert to `usize` internally at indexing boundaries, but that conversion is compiler-owned and checked. Users should not need `usize` for ordinary Sifr code. Exposing `usize` in user APIs should be limited to Rust FFI signatures or explicit low-level modules.

Negative indexing remains natural because indexes are signed exact integers.

## Bytes, Data Science, and AI

`bytes` remains raw-byte-backed internally. A byte element is externally observed as `uint8`, not `int`; users widen with `int(b)` when they want scalar exact-integer arithmetic. This keeps byte iteration allocation-free and aligns `bytes` with the fixed-width dtype model.

`bytes` is not an alias for `array[uint8]`. It is an immutable read-only byte buffer with Python-like bytes methods and binary I/O behavior. Future zero-copy views between `bytes` and `array[uint8]` can be added explicitly, but mutability and view lifetimes must remain visible in the type system.

Data science and AI surfaces should treat fixed-width integers as dtype choices:

```python
events: DataFrame = read_parquet("events.parquet", schema={"user_id": int64})
tokens: array[uint32] = tokenizer.encode(text)
labels: array[int64] = ...
```

Scalar `int` remains the ergonomic default for application logic, loop counters, lengths, and small algorithms. Columnar arrays, tensors, binary buffers, and model-runtime boundaries use explicit dtypes so memory layout and external compatibility are predictable.

Array, tensor, and dataframe arithmetic is a carve-out from scalar fixed-width promotion. Element-wise arithmetic over fixed-width dtypes preserves the dtype and must expose overflow policy:

- `array[int32] + array[int32]` returns `Result[array[int32], OverflowError]` by default.
- `xs.wrapping_add(ys)`, `xs.saturating_add(ys)`, and `xs.overflowing_add(ys)` are explicit representation-preserving kernels.
- `xs.widen_add(ys)` or an equivalent explicit API can produce `array[int]` when exact arbitrary-precision element results are desired.
- Float dtype arrays follow float semantics; exact integer scalar promotion does not silently turn fixed-width tensors into arbitrary-precision tensors.

## Type Inference and API Defaulting

The integer model must keep inference predictable. Width should never appear because the compiler guessed that a small literal "probably wants `i32`".

Rules:

- Unsuffixed integer literals infer as `int`.
- Function parameters and return annotations written as `int` mean exact `int`, never "machine integer".
- Fixed-width types appear only from explicit annotations, constructors, imported schemas, FFI signatures, or dtype declarations.
- A contextual fixed-width target may accept a const-evaluable fitting literal.
- Without a contextual fixed-width target, a mixed scalar expression involving a fixed-width value and an unsuffixed literal widens to `int`.

Container inference follows the same rule:

```python
a = [1, 2, 3]                         # list[int]
b = [int32(1), int32(2)]              # list[int32]
c = [int32(1), 2]                     # list[int]; the fixed-width value widens
d: list[int32] = [1, 2, 3]            # ok when every literal fits
e: list[int32] = [1, 2, 10 ** 100]    # compile error
```

This deliberately favors source-level simplicity over silent compact storage. Users who want compact storage say so in the type, dtype, schema, or constructor.

Generic APIs should make accumulator and output types explicit when fixed-width inputs are accepted. A function that accepts `Iterable[int32]` and computes a mathematical sum should usually return `int`; a function that preserves `int32` should use a checked/wrapping/saturating method name or an output dtype parameter.

Fixed-width-shaped library APIs pay an explicit narrowing cost when they return the same fixed-width type after arithmetic:

```python
def increment_port(port: uint16) -> Result[uint16, OverflowError]:
    return uint16(port + 1)

def increment_port_checked(port: uint16) -> Result[uint16, OverflowError]:
    return uint16.checked_add(port, 1)

def add_samples(left: int16, right: int16) -> int16:
    return int16.saturating_add(left, right)
```

This is intentional. The alternatives are an exact `int` accumulator plus final narrowing, a fixed-width checked method, or a wrapping/saturating method whose behavior is visible in the name.

## Builtins and Stdlib Surface

The standard library should reinforce the same distinction between mathematical scalar values and storage representation.

| Surface | Contract |
| --- | --- |
| `len`, `enumerate`, `range`, indexes | return/use `int` at the source level |
| `sum(list[int])` | returns `int` |
| `sum(list[int32])` | returns `int` by default; dtype-preserving sum is an explicit checked/wrapping/saturating API |
| `min`/`max(list[int32])` | returns `int32` because no arithmetic overflow is involved |
| `abs(int8)` | returns `int`; `int8.MIN.abs()` would not fit `int8` |
| `hash(int-like)` | compares by mathematical value across exact/fixed integer families where equality is allowed |
| `random.randrange`, `secrets.randbelow` | accept `int` bounds but reject negative/unbounded impractical ranges with typed errors or configured limits |
| `math` integer helpers (`gcd`, `lcm`, `isqrt`, factorial-like APIs) | accept/return exact `int` and enforce resource budgets where output can explode |

Sorting and ordering over mixed exact/fixed integer values use mathematical ordering. Mixed bool/integer ordering is invalid for the same reason `True == 1` is invalid.

Dtype-preserving fixed-width reductions live on the fixed-width namespace:

```python
total: Result[int32, OverflowError] = int32.checked_sum(values)
wrapped: int32 = int32.wrapping_sum(values)
bounded: int32 = int32.saturating_sum(values)
```

Array/tensor/dataframe reductions use the same naming pattern as element-wise kernels: `xs.checked_sum()`, `xs.wrapping_sum()`, `xs.saturating_sum()`, and explicit widening APIs such as `xs.widen_sum()`. The fixed-width carve-out covers all dtype-preserving arithmetic and reductions, not only addition.

## Web, Validation, and Public API Models

For Sifr's web-app target, integer semantics must be visible in generated request/response contracts.

Rules:

- Route path/query parameters annotated as `int` parse exact decimal strings under the configured digit limit.
- Route path/query parameters annotated as fixed-width types validate range at the boundary and return typed validation errors on failure.
- Public response models must choose a JSON integer profile (`web`, `exact`, or `string_ints`) explicitly or inherit the framework default.
- Framework default for browser-facing APIs is `json.web`; values outside the JavaScript-safe integer range require field-level string encoding or return a serialization error.
- Generated TypeScript clients map `json.web` safe integer fields to `number`, string-encoded `int` fields to `string` or a branded decimal-integer string, and future exact-client profiles may map to `bigint` only when the target runtime supports it.
- Request validation errors should report the target integer type, accepted range or digit limit, and the offending field/path.

Example model intent:

```python
class UserOut:
    id: int64          # database identifier, string-encoded by default under json.web
    balance_cents: int # exact app value; public JSON policy decides number vs string
```

The framework should not infer persistence/storage width from a source-level `int` field. Models that back SQL, Arrow, or external wire schemas must choose width or serialization policy explicitly.

Under `json.web`, schema-driven public models default to JSON numbers only when the field's static range is inside JavaScript's safe integer range. Wider fields such as `int64`, `uint64`, and exact `int` default to decimal string encoding unless the field is explicitly annotated with a runtime range-checked number policy or an exact-client policy. Untyped/dynamic JSON values still use the profile's runtime check and return `JsonIntegerRangeError` for unsafe numbers unless the caller selects string encoding.

## Identifiers, Time, and Domain Values

Many values are numerically shaped but should not be modeled as plain arithmetic integers.

- Database IDs: prefer nominal newtypes over raw `int64`/`uint64` in domain models, with explicit serialization/storage representation.
- Snowflake-style IDs and other unsigned 64-bit identifiers: use `uint64` at storage/wire boundaries; expose a domain newtype when possible.
- Timestamps: use dedicated `datetime`, `date`, `duration`, or `instant` types in application code. Unix timestamps at boundaries should choose `int64` seconds/millis/nanos or exact `int` only when schema requires unbounded values.
- Ports, status codes, byte values, and protocol fields: use fixed-width/newtype wrappers at boundaries so validation happens once.
- Money: use `decimal`, `bigdecimal`, or domain-specific minor-unit newtypes; do not rely on bare `int` unless the domain explicitly chooses exact minor units.

This avoids treating every numeric-looking value as interchangeable. The integer model provides representation tools; domain APIs should still encode meaning.

Newtype guidance depends on Sifr's existing primitive-newtype surface (`class UserId(int64)` / `class Port(uint16)`-style wrappers) or the equivalent branded-type mechanism when that surface is finalized. If a slice lands before newtypes are complete, raw fixed-width fields are acceptable at storage/interop boundaries but should not be presented as the final domain-model style.

## Diagnostics and Developer Experience

The compiler should make integer mistakes explain the representation boundary, not just say "type mismatch".

Required diagnostic families or messages:

- fixed-width narrowing out of range: include source expression, target type, valid min/max, and whether the source was const-evaluable.
- implicit narrowing attempt: suggest `int32(value)`/`uint8(value)` and remind that the result is fallible unless statically proven.
- unsafe `int / int`: explain that exact-to-float conversion is fallible for non-literal operands and suggest `//`, `Decimal(...)`, or explicit `float(...)` depending on intent.
- fixed-width array/tensor arithmetic overflow policy missing: suggest checked, wrapping, saturating, overflowing, or widen APIs.
- JSON/web-safe serialization failure: include field path, value range issue, and policy alternatives.
- bool/integer comparison: suggest `int(flag)` only when that conversion is intentional.
- fixed-width return narrowing from widened arithmetic: for `def f(x: int16) -> int16: return x + 1`, suggest `int16(x + 1)` for fallible narrowing or `int16.checked_add(x, 1)` / `int16.saturating_add(x, 1)` when representation-preserving arithmetic was intended.

These diagnostics should use stable `SIFR-*` codes when implemented; this issue defines the semantic categories, not the final registry numbers.

## Serialization and External Boundaries

The integer model has to be strict at serialization boundaries because Sifr `int` can represent values that common clients, databases, and binary formats cannot.

Core rule: Sifr never silently loses integer precision when crossing a boundary. A serializer either preserves the exact integer, proves the target can represent it, or returns a typed error.

### JSON

Sifr's JSON parser should parse integer number tokens into exact `int` values. A JSON number token with no `.`, `e`, or `E` is an integer token; fractional or exponent-bearing number tokens follow the JSON numeric profile selected by the caller, initially `float` unless a Phase 28 decimal profile is requested. The JSON reader must apply deterministic resource limits such as maximum integer digits and maximum document bytes; exceeding those limits returns `Result::Err(JsonLimitError)` rather than allocating unbounded memory from untrusted input.

JSON writing needs profiles because the JSON grammar can carry arbitrary-length decimal numbers, but JavaScript clients commonly round outside the safe integer range `[-9007199254740991, 9007199254740991]`.

Recommended profiles:

| Profile | `int` behavior | Use case |
| --- | --- | --- |
| `json.exact` | emit canonical base-10 JSON number for every `int` | Sifr-to-Sifr, Python, Rust, backend systems with exact integer parsers |
| `json.web` | emit JSON number only when the value is JavaScript-safe; otherwise return `JsonIntegerRangeError` unless the field opts into string encoding | public web APIs consumed by browsers/TypeScript |
| `json.string_ints` | emit every `int` as a decimal string | APIs that require stable cross-language precision without client bigint support |

Framework defaults should be conservative. For Sifr's future web framework, response JSON should default to the web-safe profile for untyped public responses, while schema-driven APIs can choose exact or string integer fields explicitly.

Profile rules apply recursively to collection-valued fields. For `list[int]`, `dict[str, int]`, nested objects, and other containers, each integer element follows the same profile as the containing field unless a schema annotation overrides the nested element policy. A field that opts into string integer encoding string-encodes its integer elements recursively.

OpenAPI/JSON Schema generation must reflect the chosen boundary:

- fixed-width integer fields map to bounded integer schema with minimum/maximum.
- `int` fields in `json.web` either declare safe-integer bounds or use `type: string`, `pattern: "^-?[0-9]+$"`, and a Sifr extension marker such as `x-sifr-format: integer-decimal-string`.
- exact arbitrary `int` fields must not be emitted as ordinary unbounded `type: integer` for browser-targeted clients without an explicit precision policy.

### Databases

SQL integer columns are fixed-width or database-specific. Writing Sifr `int` into `SMALLINT`, `INTEGER`, `BIGINT`, or unsigned dialect columns is fallible unless statically proven in range.

Recommended mappings:

| Storage target | Sifr contract |
| --- | --- |
| SQL `SMALLINT`/`INTEGER`/`BIGINT` | explicit fixed-width Sifr field or fallible narrowing from `int` |
| SQL unsigned dialect column | explicit `uint*` Sifr field or fallible narrowing from `int` |
| SQL `NUMERIC`/`DECIMAL` with integer scale | exact `int` mapping if precision constraints are checked |
| text column storing integer | explicit string serialization policy |

ORM/model layers should not infer `int64` from source-level `int`. The model schema must choose the storage representation.

### DataFrames, Arrow, Parquet, and Tensors

Columnar and tensor systems are dtype-oriented. `int` is a scalar application type, not a default column memory layout.

Rules:

- Creating a column/tensor from `list[int]` requires an explicit dtype when fixed-width storage is desired.
- Narrowing into `int8`/`int16`/`int32`/`int64` or unsigned dtypes validates every value and returns a typed range error on the first failing row, with row/column context when available.
- Loading Arrow/Parquet integer columns produces the matching fixed-width Sifr dtype, not arbitrary `int`, unless the user explicitly widens.
- Formats without arbitrary integer support must use fixed-width, decimal, or string encoding by schema.

### Binary Formats and RPC

Protocol Buffers, FlatBuffers, Cap'n Proto, C ABI structs, and most binary wire formats require exact widths. Their generated Sifr APIs should expose `int32`, `uint32`, `int64`, `uint64`, etc. directly.

CBOR and MessagePack can represent wider integer families than JSON but still have format-specific limits and extension mechanisms. The serializer must choose one of:

- exact native integer encoding if the format supports the value;
- explicit bignum extension/tag encoding if standardized for that format;
- decimal string encoding by schema;
- typed range error.

### CSV, Environment Variables, and URLs

Text boundaries parse `int` exactly from decimal strings by default, subject to digit limits. The initial default maximum for untrusted JSON/CSV/env/URL integer tokens should be 4096 decimal digits, configurable upward or downward per decoder. Parsing into a fixed-width target validates range. CSV/dataframe ingestion should prefer schema-driven dtype selection so large identifiers are not accidentally narrowed.

## Edge Cases and Invariants

### Equality and Hashing

If `int` compares equal to a fixed-width integer, hashes must agree so dict/set behavior stays coherent:

```python
assert int(1) == int8(1)
assert hash(int(1)) == hash(int8(1))
```

`bool` remains a separate Sifr type. This section repeats the comparison rule from above intentionally: `True == 1` is a compile error, and `True` must not alias `1` in dict/set keys.

### Pattern Matching and Literals

Literal patterns obey the same fitting and exactness rules as assignments:

```python
def classify(x: uint8) -> str:
    match x:
        case 0:
            return "zero"
        case 255:
            return "max"
        case 256:
            return "unreachable"  # compile error: literal does not fit uint8
```

Matching an `int` subject with integer literal arms is allowed and exact. Matching a fixed-width subject with an out-of-range literal is a compile-time error. Generic containers remain invariant: `list[int]` is not assignable to `list[int32]`, and `list[int32]` is not assignable to `list[int]` without explicit element-wise conversion.

### Ranges and Collection Sizes

`range` endpoints are `int`, so very large ranges are representable as lazy ranges. Materializing a range into a list, bytes object, tensor, or dataframe column is fallible when the length cannot fit addressable memory or the target dtype.

`len(...)` returns `int` because user code should not see `usize`. Generated Rust may use `usize` internally only after compiler-owned checked conversions. On `wasm32` or any 32-bit target, this means the internal conversion boundary is narrower even though source-level `int` remains exact; materialization and indexing guards must use the target's actual `usize` width.

### Enum Values

Rust-backed enum discriminants require a concrete representation. Until Sifr has a broader enum-representation design, valued enums should stay constrained to `int64`-representable values or require an explicit enum representation such as `enum Status: uint16`.

### Parsing and Resource Limits

Exact arbitrary integers can become a denial-of-service vector if untrusted input contains extremely large digit strings. All external parsers that create `int` values must have deterministic limits and typed errors. The default untrusted text limit is 4096 decimal digits unless a module defines a stricter profile. The limit belongs to the parser/decoder boundary, not to the `int` type itself.

### Performance Contract

The source language does not promise that `int` is `Copy`, pointer-sized, or ABI-stable. The implementation must still make ordinary small integers cheap.

Canonical runtime representation:

```rust
pub enum SifrInt {
    Small(i64),
    Big(Box<num_bigint::BigInt>),
}
```

`SifrInt` is immutable, `Clone`, `Eq`, `Ord`, `Hash`, `Send`, and `Sync` when its backing implementation supports those traits. It is not `Copy` and has no `#[repr(C)]` ABI guarantee. Rust FFI APIs must not expose `SifrInt` as a C-compatible integer; FFI either uses fixed-width integers or a future explicit big-integer handle/adapter.

Sifr source treats `int` as scalar value-semantic and non-consuming: using an `int` binding in more than one expression is always legal. Codegen is responsible for borrowing, cloning, or primitive-local optimization so Rust ownership does not leak into ordinary integer use. Performance-sensitive storage should use fixed-width dtypes explicitly.

### Formatting and Integer Methods

Decimal string formatting of `int` is exact. Format specs for binary, octal, hexadecimal, width, and padding follow Python's integer-formatting shape: bounded width pads but does not truncate the natural representation. The standard integer surface should include Python-compatible `bit_length()`, `bit_count()`, `to_bytes(...)`, and `from_bytes(...)`; detailed method contracts can live in the stdlib phase, but the integer model should reserve the surface.

## Rust Interop

Rust FFI should require exact signatures. If a Rust function takes `u32`, Sifr exposes `uint32`, not `int`.

```python
extern rust "crate::net":
    def set_flags(flags: uint32) -> Result[None, IOError]
```

Passing an `int` to that function requires `uint32(value)` or a compiler-proven fitting literal. Returning a Rust `u32` produces `uint32`; users widen with `int(value)` when they want Python-style arithmetic.

Sifr structs/classes containing `int` fields are not C-ABI-compatible because `SifrInt` has no `repr(C)` layout guarantee. FFI structs must use fixed-width integer fields for integer slots or an explicit future big-integer handle type.

Panics from Rust FFI remain an interop boundary concern and should be caught or rejected according to the FFI safety contract. Integer overflow inside Sifr-generated fixed-width helper methods must not panic in user-triggerable paths.

## Compiler Architecture Impact

The existing implementation currently assumes `Type::Int` is Rust `i64` in many places. The clean target requires these changes:

1. Replace `Type::Int` codegen from `i64` to a canonical runtime `SifrInt`.
2. Add `Type::Int8`, `Type::Int16`, `Type::Int32`, `Type::Int64`, `Type::UInt8`, `Type::UInt16`, `Type::UInt32`, and `Type::UInt64`.
3. Change `LiteralInt` from `i64` to an arbitrary-precision literal representation, preferably a normalized decimal string or `num_bigint::BigInt` in type-system internals.
4. Remove the user-facing need for `Type::BigInt`; keep only a temporary compatibility alias if implementation staging needs it.
5. Update numeric operator type checking so ordinary fixed-width arithmetic promotes to `int`.
6. Add explicit fallible narrowing constructors and fixed-width checked/wrapping/saturating/overflowing APIs.
7. Add array/tensor/dataframe dtype arithmetic contracts so scalar promotion does not infect fixed-width columnar kernels.
8. Update range, len, indexing, enum values, byte boundaries, diagnostics, and generated Rust casts that currently assume `i64`.
9. Teach ownership/codegen that source-level `int` is value-semantic but no longer a Rust `Copy` scalar, while allowing optimizer/codegen passes to use Rust primitive locals when statically sound.
10. Update type inference, container specialization, builtin signatures, web/model schema generation, and diagnostics so widths appear only through explicit annotations, constructors, schemas, FFI, or dtype declarations.

## Validation Matrix

Implementation slices should add positive and negative tests for each boundary, not only core arithmetic.

| Area | Positive cases | Negative cases |
| --- | --- | --- |
| Scalar `int` | exact large arithmetic, repeated use after calls, hashing/equality | `int / int` without handling, over-budget `**`/`<<`, bool/int comparison |
| Fixed-width scalars | fitting literals, fallible constructor handling, checked/wrapping/saturating APIs | out-of-range literal, implicit narrowing, negative unsigned literal |
| Type inference | `list[int]`, `list[int32]`, contextual fixed-width literals | mixed list inference surprises, generic `T + T -> T` with fixed-width |
| Bytes | indexing/iteration yields `uint8`, explicit `int(b)` widening | assigning arbitrary `int` to byte without validation |
| Arrays/tensors/dataframes | checked dtype-preserving arithmetic, explicit widen kernels, schema-driven loads | unchecked overflow policy, accidental `array[int]` from fixed-width kernels |
| Serialization | JSON exact/web/string profiles, OpenAPI/TypeScript mapping, DB narrowing | JS-unsafe `json.web` output, SQL range overflow, missing schema policy |
| Web validation | route/query/path parsing with range/digit diagnostics | over-limit integer strings, fixed-width validation failures |
| Domain newtypes | ID/port/status-code wrappers over fixed-width storage | treating domain wrappers as raw interchangeable ints |
| Interop | Rust `u32`/`i64` signatures map to fixed-width Sifr types | passing exact `int` to FFI without explicit narrowing |
| Pattern matching | in-range literal arms for fixed-width subjects | out-of-range literal patterns and bool arms against integer subjects |
| Mixed numeric arithmetic | exact `int` with decimal-family values, fixed-width with decimal-family values, handled `int`/`float` precision cases | silent `int`/`float` precision loss, invalid bool/integer comparisons |
| Formatting and integer methods | exact decimal/binary/hex formatting, `bit_length`, `bit_count`, `to_bytes`, `from_bytes` | truncating format specs, out-of-range byte conversion |
| Range and large-bound iteration | lazy `range(10 ** 100)` behavior, target-width indexing guards | materializing unaddressable ranges without typed error |
| Pointer-sized boundaries | `usize`/`isize` in FFI signatures and internal indexing conversions | leaking `usize`/`isize` into ordinary APIs without explicit conversion |
| Performance | small `int` loops stay on `SifrInt::Small` without per-iteration heap allocation | regressions that allocate for ordinary small counters/arithmetic |
| Cross-type dict/set lookup | equal exact/fixed integer keys hash consistently where equality is allowed | implicit lookup between incompatible fixed-width key domains or bool/int aliasing |

## Implementation Slices

Because Sifr is pre-production, do not carry a backward-compatibility layer into the language design.

Recommended implementation slices:

1. Lock this issue as the numeric design source of truth and update architecture references that still say `int = i64`.
2. Implement `SifrInt` representation and source-level value semantics before changing broad operator codegen.
3. Add the fixed-width type names to the parser/type annotation resolver and diagnostics, initially without broad operator support.
4. Convert `int` literals and `LiteralInt` internals away from `i64`.
5. Change `Type::Int` codegen to `SifrInt` and update ownership/category assumptions.
6. Implement exact `int` arithmetic, fixed-width scalar widening arithmetic, and explicit fixed-width checked/wrapping/saturating/overflowing APIs.
7. Replace `bigint` user-facing docs/tests with `int`; keep only targeted transition fixtures if needed.
8. Update builtin/stdlib contracts (`sum`, `abs`, `range`, `random`, `math`) and diagnostics around fixed-width/exact integer boundaries.
9. Add serialization/web/model validation contracts and tests for JSON, OpenAPI/TypeScript, DB, and text boundaries.
10. Add dtype-focused tests for `bytes`, arrays/dataframes/tensors when those phases land.

## Review Status

- [x] Claude review pass 1 completed: `reviews/integer-model-fixed-width-contract-review-pass-1.md`.
- [x] Claude review pass 2 completed after addressing pass 1 findings: `reviews/integer-model-fixed-width-contract-review-pass-2.md`.
- [x] Claude review pass 3 completed after lock-ready polish: `reviews/integer-model-fixed-width-contract-review-pass-3.md`.
- [x] Final architecture references updated in `internal_docs/architecture.md`.
- [x] Principal-engineer broader-surface review pass 4 completed: `reviews/integer-model-fixed-width-contract-review-pass-4-broader-surfaces.md`.
- [x] Principal-engineer broader-surface review pass 5 completed after pass 4 polish: `reviews/integer-model-fixed-width-contract-review-pass-5-broader-surfaces-final.md`.

## Non-Goals

- No integer literal suffix syntax in the first design. Type annotations and constructors are enough.
- No bare `uint`.
- No implicit narrowing.
- No silent fixed-width wrapping through operators.
- No public guarantee that `int` storage is `Copy`, pointer-sized, or ABI-stable.
