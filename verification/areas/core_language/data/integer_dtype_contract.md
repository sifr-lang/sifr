# Integer Dtype Contract

This artifact locks INT-6A so later array, tensor, dataframe, Arrow, and
Parquet implementations cannot choose unsafe integer defaults.

## Canonical Integer Dtypes

The integer dtype names are:

- signed: `int8`, `int16`, `int32`, `int64`, `isize`
- unsigned: `uint8`, `uint16`, `uint32`, `uint64`, `usize`

There is no compact dtype named plain `int`. Source-level `int` is exact scalar
application data, not a storage layout.

## Construction And Conversion

Constructing compact column, tensor, or array storage from `list[int]` requires
an explicit dtype:

```python
values: list[int] = [1, 2, 3]
xs: array[int32] = array.from_list(values, dtype=int32)
```

The constructor validates every element against the target dtype range and
returns a typed range error with index, row, or column context when available.
No constructor may infer `int64`, `uint64`, platform pointer width, or a
smallest-fitting dtype from `list[int]`.

Fixed-width scalar values can seed the matching dtype directly. Cross-dtype
construction, exact `int` construction, and external schema construction are
checked conversions unless the source schema already proves the exact target
range.

## Elementwise Arithmetic

Array, tensor, and dataframe arithmetic is dtype-preserving by default and must
be fallible. It is intentionally different from scalar fixed-width arithmetic,
where ordinary scalar operators promote to exact `int`.

Required default contract:

```python
array[int32] + array[int32] -> Result[array[int32], OverflowError]
array[uint16] * array[uint16] -> Result[array[uint16], OverflowError]
```

An implementation must not silently wrap, saturate, or widen. In particular,
`array[int32] + array[int32]` cannot silently wrap and cannot accidentally widen
to `array[int]`.

Explicit policy APIs are required for non-default behavior:

- `checked_add`, `checked_sub`, `checked_mul`: dtype-preserving `Result[...]`
- `wrapping_add`, `wrapping_sub`, `wrapping_mul`: explicit modular arithmetic
- `saturating_add`, `saturating_sub`, `saturating_mul`: explicit saturation
- `overflowing_add`, `overflowing_sub`, `overflowing_mul`: explicit flag return
- `widen_add`, `widen_sub`, `widen_mul`: explicit exact `array[int]` results

Reductions use the same naming pattern: `checked_sum`, `wrapping_sum`,
`saturating_sum`, `overflowing_sum`, and `widen_sum`.

## Diagnostics

When array, tensor, or dataframe surfaces exist, a fixed-width dtype arithmetic
operation without an explicit overflow policy emits `SIFR-INT-0008`.

The diagnostic payload must include:

- operation kind
- left dtype
- right dtype when applicable
- result dtype if known
- surface kind: array, tensor, dataframe, Arrow, or Parquet
- suggestions for checked, wrapping, saturating, overflowing, and widen APIs

## Arrow And Parquet

Arrow and Parquet integer columns map to matching fixed-width Sifr dtypes:

| External type | Sifr dtype |
| --- | --- |
| `Int8` | `int8` |
| `Int16` | `int16` |
| `Int32` | `int32` |
| `Int64` | `int64` |
| `UInt8` | `uint8` |
| `UInt16` | `uint16` |
| `UInt32` | `uint32` |
| `UInt64` | `uint64` |

Loaders must not silently widen external integer columns to source-level
`int`. Widening to exact `int` is an explicit option and must document the
memory and performance cost.

## Validation Sentinels

The validation-contract suite checks this file for these stable sentinels:

- `array[int32] + array[int32] -> Result[array[int32], OverflowError]`
- `array[int32] + array[int32]` cannot silently wrap
- `array[int32] + array[int32]` cannot accidentally widen to `array[int]`
- constructing compact column, tensor, or array storage from `list[int]`
  requires an explicit dtype
- `SIFR-INT-0008`
- Arrow and Parquet integer columns map to matching fixed-width Sifr dtypes

Future runtime work can replace these sentinels with executable fixtures only
after the owning data-science surfaces exist, and only if the replacement still
fails closed for a silent wrapping or implicit widening implementation.
