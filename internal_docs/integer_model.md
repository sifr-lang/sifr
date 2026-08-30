# Sifr Integer Model

## Status

This document is the canonical design for Sifr's integer model before production. The implementation is tracked by `integer-model-and-fixed-width-numeric-rules record`.

Sifr intentionally does not preserve the older bootstrap model where source-level `int` lowered to Rust `i64` and arbitrary precision lived behind a separate user-facing `bigint`. The target language model is clean because Sifr is not production-released yet.

## Goals

- Keep ordinary application, web, data science, and AI code Python-simple: `int` is the default integer and arithmetic does not overflow, wrap, or panic.
- Give users explicit Rust-compatible fixed-width access when memory layout or wire/storage representation matters.
- Preserve Sifr's safety guarantee: no silent precision loss, no user-triggerable integer panics, and typed errors at fallible boundaries.
- Make serialization and interop rules visible enough that generated web/API/database rules do not accidentally lie about integer precision.

## Non-Goals

- No separate long-term user-facing `bigint` type.
- No bare `uint`.
- No implicit narrowing.
- No silent fixed-width wrapping through ordinary operators.
- No guarantee that source-level `int` is `Copy`, pointer-sized, ABI-stable, or C-compatible.
- No integer literal suffix syntax in the initial implementation. Type annotations, constructors, dtypes, schemas, and interop signatures are enough.

## Core Decision

Sifr's source-level `int` is an exact signed arbitrary-precision value-semantic scalar.

```python
x: int = 42
count: int = 10 ** 100
```

The underlying target representation for `x: int = 42` is `SifrInt`, initially in its inline-small form:

```rust
pub enum SifrInt {
    Small(i64),
    Big(Box<num_bigint::BigInt>),
}
```

That means `x: int = 42` is conceptually exact arbitrary precision at the source level, and implementation-wise should compile to `SifrInt::Small(42)` or an equivalent optimized local representation that preserves the same observable semantics. Optimizations may keep proven-small locals in Rust primitives internally, but generated public semantics remain `SifrInt`.

The `SifrInt` enum is a runtime crate API for generated Rust, not a C ABI. If INT-1 exposes the concrete `Small`/`Big` variants to generated projects, representation changes such as switching `Small(i64)` to `Small(i128)` must be treated as generated-runtime compatibility changes. If the implementation wants freedom to change the layout later, the variants should be hidden behind constructors and accessors from the initial runtime implementation.

Fixed-width integers are first-class explicit types for representation-sensitive work:

```python
port: uint16 = 5432
mask: uint32 = uint32(0xff00)
ids: array[int64] = ...
tokens: array[uint32] = tokenizer.encode(text)
```

Width is a storage, binary protocol, dtype, schema, or interop choice. It is not the compiler's default guess for small literals.

## Source Types

| Sifr type | Meaning | Primary use | Rust representation |
| --- | --- | --- | --- |
| `int` | exact signed integer, arbitrary precision | default app/web/business/algorithm integer | `SifrInt`, inline-small with arbitrary-precision spill |
| `int8`, `int16`, `int32`, `int64` | signed fixed-width integer | binary formats, DB/dataframe/tensor schemas, interop, memory-sensitive storage | `i8`, `i16`, `i32`, `i64` |
| `uint8`, `uint16`, `uint32`, `uint64` | unsigned fixed-width integer | bytes, protocols, external unsigned schemas, interop | `u8`, `u16`, `u32`, `u64` |
| `isize`, `usize` | pointer-sized Rust interop integer | low-level interop boundary only | `isize`, `usize` |

Reserve `int128` and `uint128` as future fixed-width type names. Rust supports `i128` and `u128`, and some storage targets need 128-bit values, but Sifr does not need to ship them in the initial fixed-width implementation. Using either reserved name before support lands must produce `SIFR-INT-0003`, not a generic unresolved-name diagnostic.

The reserved-width diagnostic is reached after ordinary annotation name resolution. Existing Sifr type names are shadowable, so a user-defined type variable, type alias, or class named `int128` or `uint128` resolves to that user definition instead of emitting `SIFR-INT-0003`. INT-2B should keep this general shadowing behavior rather than create a special anti-shadowing rule only for future integer widths. A later language-wide reserved-identifier policy may tighten this consistently across all builtin and reserved names.

There is no separate user-facing arbitrary-precision integer type. The former transition spelling is an unknown type or callable and follows the ordinary name-resolution diagnostics.

## Literal and Conversion Rules

Unsuffixed integer literals are `int`.

```python
x = 42          # int
y = 10 ** 100  # int
```

Typed literal assignment to a fixed-width target is allowed only when the compiler proves the value fits:

```python
x: int32 = 42              # ok
y: uint8 = 255             # ok
z: uint8 = 256             # compile error
n: uint8 = -1              # compile error
```

The compile-time fitting rule applies to const-evaluable integer expressions, not only single tokens. The first implementation should support:

- integer literals
- unary `+` and `-`
- integer `+`, `-`, `*`
- shifts with fitting constant shift counts
- non-negative integer exponentiation within the compile-time budget
- parentheses
- immutable module constants whose initializer is const-evaluable

Runtime-dependent conditionals, function calls, collection lookups, and non-constant names do not participate. They require explicit fallible constructors.

The first compile-time evaluator budget is 4096 decimal digits for any evaluated integer result, plus an implementation-defined operation-count guard to prevent pathological constant expressions from hanging the type checker. Exceeding the budget is a compile-time `SIFR-INT-0004` diagnostic, not a fallback to runtime narrowing. Imported immutable module constants may carry const-evaluable status across module boundaries only when the frontend query layer can prove the imported initializer and its dependency graph are acyclic and within budget.

Const-evaluable import status is local to the importing module. A module may use an imported immutable constant in its own fixed-width fitting checks, but it does not transitively re-export that imported constant's const value with `from other import LIMIT`. Downstream modules must import from the module that defines the constant, or the intermediate module must define its own public immutable constant with a const-evaluable initializer.

```python
x: uint8 = 100 + 27       # ok
y: uint8 = 1 - 2          # compile error
LIMIT: int = 200
z: uint8 = LIMIT          # ok when LIMIT is const-evaluable
```

Runtime narrowing is explicit and fallible:

```python
def read_port(value: int) -> Result[uint16, OverflowError]:
    return uint16(value)
```

Widening from every fixed-width integer to `int` is infallible:

```python
small: int32 = 7
wide: int = int(small)
```

Conversions between signed and unsigned fixed-width types are explicit and fallible unless the source is a fitting literal. There are no implicit narrowing conversions in assignments, calls, returns, list literals, dict literals, or generic specialization.

## Arithmetic

Exact `int` arithmetic returns `int` for integer-preserving operators.

| Expression | Result |
| --- | --- |
| `int + int`, `int - int`, `int * int` | `int` |
| `int // int`, `int % int` | `Result[int, DivisionError]` unless non-zero is proven |
| `int ** int` | `int` when the exponent is proven non-negative and output stays within budget; otherwise `Result[int, ValueError | ArithmeticLimitError]` |
| `int << int`, `int >> int` | `int` when the shift is proven valid and within budget; otherwise `Result[int, ValueError | ArithmeticLimitError]` |
| `int / int` | `Result[float, DivisionError | FloatOverflowError | FloatPrecisionLossError]` unless non-zero and float-representability are both proven |

Integer `**` is intentionally integer-preserving. Unlike Python, `2 ** -1` does not implicitly become `0.5`; users write `float(2) ** -1`, `Decimal("2") ** -1`, or another explicit non-integer numeric operation when they want a non-integer result.

Exact integer operations are not allowed to trigger unbounded allocation from hostile input. External parsers enforce digit limits, and explosive operations such as exponentiation and left shift have a configurable maximum output bit length. Exceeding that budget returns `ArithmeticLimitError`. Straight-line `+`, `-`, and `*` remain `int`-returning; process-level resource exhaustion such as allocator failure remains an operational concern bounded by parser limits and configured arithmetic budgets.

Ordinary fixed-width scalar arithmetic promotes to exact `int`:

```python
a: int32 = 2_000_000_000
b: int32 = 2_000_000_000
c: int = a + b
d: int32 = int32(a + b)
```

This keeps everyday arithmetic Python-simple while preserving fixed-width storage. Fixed-width arithmetic that preserves representation must be named:

```python
checked: Result[int32, OverflowError] = int32.checked_add(a, b)
sat: int32 = int32.saturating_add(a, b)
wrapped: int32 = int32.wrapping_add(a, b)
overflowed: tuple[int32, bool] = int32.overflowing_add(a, b)
```

The Rust-style names are intentional. Checked, wrapping, saturating, and overflowing behavior must be visible at the call site.

Bitwise operators on fixed-width integers may return the same fixed-width type because they operate within representation rather than mathematical magnitude:

```python
mask: uint32 = left & right
```

Shift operators need checked semantics because invalid or oversized shifts are external-input dependent. Prefer explicit `checked_shl`, `checked_shr`, `wrapping_shl`, and `wrapping_shr` APIs for fixed-width values.

Fixed-width division, floor division, modulo, exponentiation, and shifts use the same no-silent-failure rule. Ordinary scalar `int32 // int32` promotes to `Result[int, DivisionError]`; representation-preserving variants live behind explicit checked/wrapping/saturating APIs or dtype-specific array kernels.

## Comparisons, Mixing, and Hashing

Comparisons between `int` and fixed-width integers are allowed and exact:

```python
limit: int = 10 ** 20
narrow: int64 = 9
ok: bool = narrow < limit
```

Arithmetic involving `int` and a fixed-width integer returns `int` unless the operator is representation-specific. Arithmetic between two fixed-width integer operands also promotes to `int` for ordinary `+`, `-`, `*`, `//`, `%`, and `**`.

`usize` and `isize` follow the same scalar promotion rule when they escape low-level interop signatures: ordinary arithmetic widens to `int`, and narrowing back to pointer-sized storage is explicit and fallible.

Decimal mixing keeps the decimal semantics architecture policy:

- `int + decimal` returns `decimal`.
- `int + bigdecimal` returns `bigdecimal`.
- fixed-width integer plus `decimal` or `bigdecimal` first widens exactly to `int`, then follows the decimal policy.
- `int` or fixed-width integer mixed with `float` is fallible unless the integer operand is proven exactly representable as `float`.

Equality and ordering compare mathematical values, not bit patterns. `int8(-1) != uint8(255)`.

Integer and float comparisons are exact rather than cast-based. `int(2 ** 53 + 1) == float(2 ** 53 + 1)` compares the exact integer to the exact rational value represented by the float and returns `False`; it must not cast the integer to `float` first. Ordering follows the same rule by comparing the integer against the exact decomposed float mantissa/exponent. NaN remains unordered according to the float comparison rules.

If two hashable exact/fixed integer values compare equal, their hashes must agree:

```python
assert int(1) == int8(1)
assert hash(int(1)) == hash(int8(1))
```

`bool` remains separate. `int(True)` is allowed as an explicit conversion, but `True == 1` is a compile error. `True` must not alias `1` as a dict/set key.

Generic arithmetic must model operator output type. A generic `T + T -> T` bound is valid only for numeric types whose operator output is assignable to `T`. Fixed-width scalar types do not satisfy that bound for ordinary arithmetic because `int32 + int32 -> int`.

The existing `Addable` protocol must be refined to carry an associated output type or be limited to `Self`-preserving addition. A generic function that wants mathematical integer addition across `int` and fixed-width families should use a future integer protocol with an explicit accumulator/output type, not assume Rust's `Add<Output = Self>` shape. This refinement belongs to the scalar arithmetic work because it changes operator typing and generic monomorphization.

```python
def sum_int32(values: list[int32]) -> int:
    total: int = 0
    for value in values:
        total = total + value
    return total
```

## Type Inference

The compiler must never infer a width because a literal is small.

- Unsuffixed integer literals infer as `int`.
- Function parameters and return annotations written as `int` mean exact `int`, never machine integer.
- Fixed-width types appear only from explicit annotations, constructors, imported schemas, interop signatures, or dtype declarations.
- A contextual fixed-width target may accept a const-evaluable fitting literal.
- Without a contextual fixed-width target, a mixed scalar expression involving a fixed-width value and an unsuffixed literal widens to `int`.

Container inference follows the same rule:

```python
a = [1, 2, 3]                         # list[int]
b = [int32(1), int32(2)]              # list[int32]
c = [int32(1), 2]                     # list[int]
d: list[int32] = [1, 2, 3]            # ok when every literal fits
e: list[int32] = [1, 2, 10 ** 100]    # compile error
```

Users who want compact storage say so in the type, dtype, schema, or constructor.

## Indexing, Lengths, Ranges, and `usize`

User-facing indexes and lengths stay `int`.

```python
size: int = len(items)
value: T | None = items[i]
```

Generated Rust may convert to `usize` internally at indexing boundaries, but that conversion is compiler-owned and checked. Users should not need `usize` for ordinary Sifr code. Exposing `usize` in user APIs is limited to explicit low-level Rust interop declarations or low-level modules gated by a package/module-level interop opt-in; ordinary modules cannot name `usize` by accident.

Negative indexing remains natural because indexes are signed exact integers.

`range` endpoints are `int`, so very large ranges are representable as lazy ranges. Materializing a range into a list, bytes object, tensor, or dataframe column is fallible when the length cannot fit addressable memory or the target dtype.

On `wasm32` or any 32-bit target, compiler-owned `usize` conversions use the target's actual pointer width even though source-level `int` remains exact.

## Bytes, Arrays, DataFrames, and Tensors

`bytes` remains raw-byte-backed internally. A byte element is externally observed as `uint8`, not `int`; users widen with `int(b)` when they want scalar exact-integer arithmetic.

`bytes` is not an alias for `array[uint8]`. It is an immutable read-only byte buffer with Python-like bytes methods and binary I/O behavior. Future zero-copy views between `bytes` and `array[uint8]` can be added explicitly, with mutability and view lifetimes visible in the type system.

`bytearray` follows the same element type rule on reads and iteration: elements are `uint8`. Writes require a fitting literal or a `uint8` value. Assigning an arbitrary `int` to a bytearray element requires explicit fallible narrowing through `uint8(value)` so mutation cannot silently truncate.

`array` is a future dtype-bearing surface in this design context; references to `array[int32]` describe the required rules for the data-science work even when the runtime container is not implemented yet.
The reviewable and test-owned rules artifact for this deferred dtype surface
is `verification/areas/core_language/data/integer_dtype_rules.md`; the quick
validation profile runs its sentinel check so future runtime work cannot remove the
no-silent-wrap and no-implicit-widen requirements by accident.

Data science and AI surfaces treat fixed-width integers as dtype choices:

```python
events: DataFrame = read_parquet("events.parquet", schema={"user_id": int64})
labels: array[int64] = ...
```

Array, tensor, and dataframe arithmetic is a carve-out from scalar fixed-width promotion. Element-wise arithmetic over fixed-width dtypes preserves dtype and must expose overflow policy:

- `array[int32] + array[int32]` returns `Result[array[int32], OverflowError]` by default.
- `xs.wrapping_add(ys)`, `xs.saturating_add(ys)`, and `xs.overflowing_add(ys)` are explicit representation-preserving kernels.
- `xs.widen_add(ys)` or an equivalent explicit API can produce `array[int]` when exact arbitrary-precision element results are desired.
- Float dtype arrays follow float semantics; exact integer scalar promotion does not silently turn fixed-width tensors into arbitrary-precision tensors.

Array/tensor/dataframe reductions use the same naming pattern: `xs.checked_sum()`, `xs.wrapping_sum()`, `xs.saturating_sum()`, and explicit widening APIs such as `xs.widen_sum()`.

## Builtins and Stdlib

The standard library should reinforce the distinction between mathematical scalar values and storage representation.

| Surface | Rules |
| --- | --- |
| `len`, `enumerate`, `range`, indexes | return/use `int` at the source level |
| `sum(list[int])` | returns `int` |
| `sum(list[int32])` | returns `int` by default; dtype-preserving sum is explicit |
| `min`/`max(list[int32])` | returns `int32` because no arithmetic overflow is involved |
| `abs(int8)` | returns `int`; `int8.MIN.abs()` would not fit `int8` |
| `hash(int-like)` | compares by mathematical value across exact/fixed integer families where equality is allowed |
| `random.randrange`, `secrets.randbelow` | accept `int` bounds but reject negative/unbounded impractical ranges with typed errors or configured limits |
| `math` integer helpers | accept/return exact `int` and enforce budgets where output can explode |

Fixed-width-shaped library APIs pay an explicit narrowing cost when they return the same fixed-width type after arithmetic:

```python
def increment_port(port: uint16) -> Result[uint16, OverflowError]:
    return uint16(port + 1)

def increment_port_checked(port: uint16) -> Result[uint16, OverflowError]:
    return uint16.checked_add(port, 1)

def add_samples(left: int16, right: int16) -> int16:
    return int16.saturating_add(left, right)
```

## Web, Validation, and Public API Models

For Sifr's web-app target, integer semantics must be visible in generated request and response rules.

- Route path/query parameters annotated as `int` parse exact decimal strings under the configured digit limit.
- Route path/query parameters annotated as fixed-width types validate range at the boundary and return typed validation errors on failure.
- Public response models choose a JSON integer profile (`web`, `exact`, or `string_ints`) explicitly or inherit the framework default.
- Framework default for browser-facing APIs is `json.web`.
- Generated TypeScript clients map `json.web` safe integer fields to `number`, string-encoded `int` fields to `string` or a branded decimal-integer string, and future exact-client profiles may map to `bigint` only when the target runtime supports it.
- Request validation errors report the target integer type, accepted range or digit limit, and the offending field/path.

Example:

```python
class UserOut:
    id: int64           # database identifier; needs safe bounds or string_ints
    balance_cents: int  # exact app value; public JSON policy decides number vs string
```

The framework must not infer persistence or storage width from a source-level `int` field. Models that back SQL, Arrow, or external wire schemas must choose width or serialization policy explicitly.

Under `json.web`, schema-driven public models use JSON numbers only when the
field's complete static range is inside the JavaScript-safe range. Wider fields
such as `int64`, `uint64`, and exact `int` fail schema generation unless bounds
prove that range. Select `json.string_ints` explicitly for decimal-string
encoding, or select `json.exact` with an exact-client policy.

## Serialization and External Boundaries

Core rule: Sifr never silently loses integer precision when crossing a boundary. A serializer either preserves the exact integer, proves the target can represent it, or returns a typed error.

The reviewable rules artifact for schema, client, generated serde, and
storage boundary mappings is
`verification/areas/core_language/data/integer_model/serialization_boundary_rules.md`.
Future work on web, ORM, and schema surfaces must update that artifact when
implementing the corresponding runtime surfaces.

### JSON

Sifr's JSON parser parses integer number tokens into exact `int` values. A JSON number token with no `.`, `e`, or `E` is an integer token. Fractional or exponent-bearing number tokens follow the selected numeric profile, initially `float` unless a decimal semantics architecture decimal profile is requested.

JSON readers apply deterministic resource limits such as maximum integer digits and maximum document bytes. Exceeding those limits returns `JsonLimitError` rather than allocating unbounded memory from untrusted input.
The current `sifr.json.loads` path validates JSON integer token digit budgets
before handing input to `serde_json`; compatibility keeps `loads` on
`JSONDecodeError`, while `sifr.json.validate_integer_digit_limits` exposes the
typed `JsonLimitError` boundary directly.

Recommended writing profiles:

| Profile | `int` behavior | Use case |
| --- | --- | --- |
| `json.exact` | emit canonical base-10 JSON number for every `int` | Sifr-to-Sifr, Python, Rust, backend systems with exact integer parsers |
| `json.web` | emit JSON number only when JavaScript-safe; otherwise return `JsonIntegerRangeError` unless the field opts into string encoding | public web APIs consumed by browsers/TypeScript |
| `json.string_ints` | emit every `int` as a decimal string | APIs that require stable cross-language precision without client bigint support |

Profile rules apply recursively to collection-valued fields. For `list[int]`, `dict[str, int]`, nested objects, and other containers, each integer element follows the same profile as the containing field unless a schema annotation overrides the nested element policy.

Generated `serde::Serialize` or `serde::Deserialize` support for Sifr structs/classes must use an explicit integer profile rather than Rust's default primitive serialization for `SifrInt`. Framework-level default derives use `json.web` for public browser-facing responses; internal derives must declare `json.exact` or another profile at the boundary.

OpenAPI/JSON Schema generation must reflect the chosen boundary:

- fixed-width integer fields map to bounded integer schema with minimum and maximum.
- `int` fields in `json.web` either declare safe-integer bounds or use `type: string`, `pattern: "^-?[0-9]+$"`, and a Sifr extension marker such as `x-sifr-format: integer-decimal-string`.
- exact arbitrary `int` fields must not be emitted as ordinary unbounded `type: integer` for browser-targeted clients without an explicit precision policy.

The implemented exact-profile schema marker is
`x-sifr-integer-profile: exact`. The schema also includes
`x-sifr-generated-client-warning` so a generated-client backend can require an
exact JSON integer parser. The backend owns presenting that client warning;
the compiler continues to own `SIFR-INT-0009` for an unsafe or ambiguous
boundary. `pydantic_sifr_core::generate_json_schema` consumes the sealed Core
Schema and the `SerializationPlan` profile. It does not infer another profile
or fall back from `json.web` to `json.string_ints`.

### Databases

SQL integer columns are fixed-width or database-specific. Writing Sifr `int` into `SMALLINT`, `INTEGER`, `BIGINT`, or unsigned dialect columns is fallible unless statically proven in range.

| Storage target | Sifr rules |
| --- | --- |
| SQL `SMALLINT`/`INTEGER`/`BIGINT` | explicit fixed-width Sifr field or fallible narrowing from `int` |
| SQL unsigned dialect column | explicit `uint*` Sifr field or fallible narrowing from `int` |
| SQL `NUMERIC`/`DECIMAL` with integer scale | exact `int` mapping if precision constraints are checked |
| text column storing integer | explicit string serialization policy |

ORM/model layers must not infer `int64` from source-level `int`.

### DataFrames, Arrow, Parquet, and Tensors

Columnar and tensor systems are dtype-oriented. `int` is a scalar application type, not a default column memory layout.

- Creating a column/tensor from `list[int]` requires an explicit dtype when fixed-width storage is desired.
- Narrowing into integer dtypes validates every value and returns a typed range error with row/column context when available.
- Loading Arrow/Parquet integer columns produces the matching fixed-width Sifr dtype, not arbitrary `int`, unless the user explicitly widens.
- Formats without arbitrary integer support must use fixed-width, decimal, or string encoding by schema.

### Binary Formats and RPC

Protocol Buffers, FlatBuffers, Cap'n Proto, C ABI structs, and most binary wire formats require exact widths. Generated Sifr APIs expose `int32`, `uint32`, `int64`, `uint64`, and related fixed-width types directly.

CBOR and MessagePack can represent wider integer families than JSON but still have format-specific limits and extension mechanisms. Serializers choose one of:

- exact native integer encoding if the format supports the value;
- explicit bignum extension/tag encoding if standardized for that format;
- decimal string encoding by schema;
- typed range error.

### CSV, Environment Variables, and URLs

Text boundaries parse `int` exactly from decimal strings by default, subject to digit limits. The initial default maximum for untrusted JSON/CSV/env/URL integer tokens is 4096 decimal digits, configurable per decoder. Parsing into a fixed-width target validates range. CSV/dataframe ingestion should prefer schema-driven dtype selection so large identifiers are not accidentally narrowed.

## Domain Values

Many values are numerically shaped but should not be modeled as plain arithmetic integers.

- Database IDs: prefer nominal newtypes over raw `int64` or `uint64` in domain models, with explicit serialization/storage representation.
- Snowflake-style IDs and other unsigned 64-bit identifiers: use `uint64` at storage/wire boundaries; expose a domain newtype when possible.
- Timestamps: use dedicated `datetime`, `date`, `duration`, or `instant` types in application code.
- Ports, status codes, byte values, and protocol fields: use fixed-width/newtype wrappers at boundaries.
- Money: use `decimal`, `bigdecimal`, or domain-specific minor-unit newtypes; do not rely on bare `int` unless the domain explicitly chooses exact minor units.

Newtype guidance depends on Sifr's primitive-newtype or branded-type surface. If a slice lands before newtypes are complete, raw fixed-width fields are acceptable at storage/interop boundaries but should not be presented as the final domain-model style.

## Pattern Matching, Enums, and Containers

Literal patterns obey the same fitting and exactness rules as assignments.

```python
def classify(x: uint8) -> str:
    match x:
        case 0:
            return "zero"
        case 255:
            return "max"
        case 256:
            return "unreachable"  # compile error
```

Matching an `int` subject with integer literal arms is allowed and exact. Matching a fixed-width subject with an out-of-range literal is a compile-time error.

Generic containers remain invariant: `list[int]` is not assignable to `list[int32]`, and `list[int32]` is not assignable to `list[int]` without explicit element-wise conversion.

Rust-backed enum discriminants require a concrete representation. Until Sifr has a broader enum-representation design, valued enums should stay constrained to `int64`-representable values or require an explicit enum representation such as `enum Status: uint16`.

## Diagnostics

Integer diagnostics should explain the representation boundary.

Required diagnostic families:

- fixed-width narrowing out of range: include source expression, target type, valid min/max, and whether the source was const-evaluable.
- implicit narrowing attempt: suggest `int32(value)` or `uint8(value)` and remind that the result is fallible unless statically proven.
- unsafe `int / int`: explain exact-to-float fallibility and suggest `//`, `Decimal(...)`, or explicit `float(...)` depending on intent.
- fixed-width array/tensor arithmetic overflow policy missing: suggest checked, wrapping, saturating, overflowing, or widen APIs.
- JSON/web-safe serialization failure: include field path, value range issue, and policy alternatives.
- bool/integer comparison: suggest `int(flag)` only when that conversion is intentional.
- fixed-width return narrowing from widened arithmetic: for `def f(x: int16) -> int16: return x + 1`, suggest `int16(x + 1)` for fallible narrowing or explicit fixed-width checked/saturating APIs.

Reserve the `SIFR-INT-*` family for integer-model diagnostics:

| Code | Family |
| --- | --- |
| `SIFR-INT-0001` | fixed-width literal or const expression out of range |
| `SIFR-INT-0002` | implicit narrowing from exact/fixed source to narrower fixed-width target |
| `SIFR-INT-0003` | reserved integer width name such as `int128` or `uint128` before support lands |
| `SIFR-INT-0004` | compile-time integer evaluation budget exceeded |
| `SIFR-INT-0005` | exact integer division, modulo, exponentiation, or shift requires handling a typed failure |
| `SIFR-INT-0006` | exact integer to `float` conversion would overflow or lose precision |
| `SIFR-INT-0007` | bool/integer comparison without explicit conversion |
| `SIFR-INT-0008` | fixed-width array/tensor/dataframe arithmetic missing overflow policy |
| `SIFR-INT-0009` | JSON/web-safe integer serialization policy failure |
| `SIFR-INT-0010` | bytearray/bytes construction or mutation requires fitting `uint8` |

`SIFR-INT-0009` is active. The compiler-owned package-neutral
`JsonIntegerBoundaryDescriptor` verifier and its source declaration surface are documented in
[`const_specialization.md`](const_specialization.md); missing or unsafe compile-time policy fails
before schema or serializer generation.

## Runtime and Codegen Rules

The target runtime placement is a new workspace crate, `crates/sifr_runtime`, linked by generated projects through the codegen-emitted Cargo manifest. `SifrInt`, integer parsing/formatting helpers, arithmetic budget helpers, normalized integer hashing, and JSON integer profile helpers live there rather than being re-emitted into every generated Rust file.

`SifrInt` is immutable, `Clone`, `Eq`, `Ord`, `Hash`, `Send`, and `Sync` when its backing implementation supports those traits. It is not `Copy` and has no `#[repr(C)]` ABI guarantee. Sifr `int` nevertheless retains language-level value semantics; code generation inserts or elides explicit Rust clones according to ownership and liveness.

Rust interop APIs must not expose `SifrInt` as a C-compatible integer. Interop either uses fixed-width integers or `sifr_runtime::interop::SifrIntBridge`, the explicit exact-integer bridge representation defined by the Rust interop architecture.

Sifr source treats `int` as scalar value-semantic and non-consuming: using an `int` binding in more than one expression is always legal. Codegen is responsible for borrowing, cloning, or primitive-local optimization so Rust ownership does not leak into ordinary integer use.

Decimal string formatting of `int` is exact. Format specs for binary, octal, hexadecimal, width, and padding follow Python's integer-formatting shape: bounded width pads but does not truncate. The standard integer surface should include Python-compatible `bit_length()`, `bit_count()`, `to_bytes(...)`, and `from_bytes(...)`.

## Rust Interop

Rust interop requires exact signatures. If a Rust function takes `u32`, Sifr exposes `uint32`, not `int`.

```sifr
@rust(bridge.net.set_flags)
def set_flags(flags: uint32) -> Result[None, IOError | RustPanicError]:
    pass
```

Passing an `int` to that function requires `uint32(value)` or a compiler-proven fitting literal. Returning Rust `u32` produces `uint32`; users widen with `int(value)` when they want Python-style arithmetic.

Sifr structs/classes containing `int` fields are not C-ABI-compatible because `SifrInt` has no `repr(C)` layout guarantee. Low-level interop structs must use fixed-width integer fields for integer slots or generated bridge fields backed by `SifrIntBridge`.

Panics from Rust interop remain a boundary concern and should be caught or rejected according to Rust interop safety rules. Integer overflow inside Sifr-generated fixed-width helper methods must not panic in user-triggerable paths.

## Compiler Architecture Impact

The existing implementation assumes source-level `int` has a Rust signed-64-bit representation in many places. The target requires:

1. Replace source-level `int` codegen from the legacy signed-64-bit representation to canonical `SifrInt`.
2. Add `Type::Int8`, `Type::Int16`, `Type::Int32`, `Type::Int64`, `Type::UInt8`, `Type::UInt16`, `Type::UInt32`, and `Type::UInt64`.
3. Change `LiteralInt` from `i64` to an arbitrary-precision literal representation, preferably a normalized decimal string or `num_bigint::BigInt` in type-system internals.
4. Keep arbitrary precision behind `Type::Int`; do not add a second public integer type variant.
5. Update numeric operator type checking so ordinary fixed-width arithmetic promotes to `int`.
6. Add explicit fallible narrowing constructors and fixed-width checked/wrapping/saturating/overflowing APIs.
7. Add array/tensor/dataframe dtype arithmetic rules so scalar promotion does not infect fixed-width columnar kernels.
8. Update range, len, indexing, enum values, byte boundaries, diagnostics, and generated Rust casts that currently assume `i64`.
9. Teach ownership/codegen that source-level `int` is value-semantic but no longer a Rust `Copy` scalar.
10. Update type inference, container specialization, builtin signatures, web/model schema generation, and diagnostics so widths appear only through explicit annotations, constructors, schemas, interop signatures, or dtype declarations.
11. Add or update HIR maintainability guardrails for the fixed-width type family so adding variants does not produce new monolithic lowering paths.

## Validation Matrix

Implementation increments should add positive and negative tests for each boundary.

| Area | Positive cases | Negative cases |
| --- | --- | --- |
| Scalar `int` | exact large arithmetic, repeated use after calls, hashing/equality | unhandled `int / int`, over-budget `**`/`<<`, bool/int comparison |
| Fixed-width scalars | fitting literals, fallible constructors, checked/wrapping/saturating APIs | out-of-range literal, implicit narrowing, negative unsigned literal |
| Type inference | `list[int]`, `list[int32]`, contextual fixed-width literals | mixed list surprises, generic `T + T -> T` with fixed-width |
| Bytes | indexing/iteration yields `uint8`, explicit `int(b)` widening | assigning arbitrary `int` to byte without validation |
| Arrays/tensors/dataframes | checked dtype-preserving arithmetic, explicit widen kernels, schema-driven loads | unchecked overflow policy, accidental `array[int]` from fixed-width kernels |
| Serialization | JSON exact/web/string profiles, OpenAPI/TypeScript mapping, DB narrowing | JS-unsafe `json.web` output, SQL range overflow, missing schema policy |
| Web validation | route/query/path parsing with range/digit diagnostics | over-limit integer strings, fixed-width validation failures |
| Domain newtypes | ID/port/status-code wrappers over fixed-width storage | treating domain wrappers as raw interchangeable ints |
| Interop | Rust `u32`/`i64` signatures map to fixed-width Sifr types | passing exact `int` to Rust interop without explicit narrowing |
| Pattern matching | in-range literal arms for fixed-width subjects | out-of-range literal patterns and bool arms against integer subjects |
| Mixed numeric arithmetic | exact `int` with decimal-family values, handled `int`/`float` precision cases | silent `int`/`float` precision loss, invalid bool/integer comparisons |
| Integer/float comparisons | exact comparison against finite float mantissa/exponent values | cast-based comparison that rounds large integers |
| Formatting and methods | exact decimal/binary/hex formatting, `bit_length`, `bit_count`, `to_bytes`, `from_bytes` | truncating format specs, out-of-range byte conversion |
| Range and materialization | lazy `range(10 ** 100)`, target-width indexing guards | materializing unaddressable ranges without typed error |
| Pointer-sized boundaries | `usize`/`isize` in low-level interop signatures and internal indexing conversions | leaking `usize`/`isize` into ordinary APIs |
| Performance | small `int` loops stay on `SifrInt::Small` without per-iteration heap allocation | allocations for ordinary small counters/arithmetic |
| Cross-type dict/set lookup | equal exact/fixed integer keys hash consistently | bool/int aliasing or incompatible fixed-width key-domain surprises |
