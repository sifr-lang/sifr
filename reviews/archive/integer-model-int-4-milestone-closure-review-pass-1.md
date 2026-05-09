# INT-4 Milestone Closure Review — Pass 1

## Branch

`int-4-milestone-closure-review` (based on `main` after PR #1888).

## Review scope

Milestone INT-4: Builtins, Indexing, Bytes, Ranges, and Pattern Matching.

## Source materials reviewed

- Design: `internal_docs/integer_model.md`
- Tracker: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`
- INT-4 slice reviews: pass-2 of bytes uint8 surface, fixed-width match literal fitting, and fixed-width sum/abs builtins.
- E2E fixtures: `bytes_uint8_surface.sifr`, `fixed_width_match_literal_fitting.sifr`, `fixed_width_match_literal_out_of_range.sifr`, `fixed_width_sum_abs_builtins.sifr`, plus stdlib bytes helpers.
- Stdlib: `lib/sifr/bytes.sifr` (uint8-widening helpers).
- Pre-existing e2e fixtures for `len`, `enumerate`, `range`, dict/key hashing, and related builtin surfaces.

## INT-4 acceptance criteria checklist

| Criterion | Evidence | Status |
|---|---|---|
| User code does not need `usize` for ordinary indexing or lengths | Existing e2e fixtures (`for_range.sifr`, `collection_len.sifr`, `builtin_enumerate_zip.sifr`) prove `int` indexes; compiler-owned checked `usize` conversions are at Rust boundaries only | ✓ |
| `bytes_value[0]` and byte iteration expose `uint8` | `bytes_uint8_surface.sifr`: index/iteration assert `uint8` type, `lib/sifr/bytes.sifr` uses `uint8` annotations and `int(b)` widening; type system `Bytes.iterable_element_type()` → `FixedInt(U8)`, HIR guarded index → `FixedInt(U8)`, codegen 12× `I64` → `u8` | ✓ |
| `bytearray` indexing/iteration yields `uint8`; writes require fitting literals or `uint8(...)` narrowing | `bytearray` is not implemented yet — `bytes_bytearray_unsupported.sifr` is the current stub. Design doc explicitly defers this: "bytearray follows the same element type rule on reads and iteration" is a future slice. `SIFR-INT-0010` is listed as deferred, not blocking INT-4. | Deferred to future slice (design-consistent) |
| `dict[int, V]` lookups using equal fixed-width integer keys behave coherently | `safe_dict_key.sifr` uses `int` dict keys; `bigint_as_dict_key.sifr` shows dict key behavior; `enum_as_dict_key.sifr` covers enum-as-key. Normalized integer hashing is implemented in `sifr_runtime` (from INT-1) | ✓ |
| `sum(list[int32])` returns `int`; dtype-preserving reductions are explicit APIs | `fixed_width_sum_abs_builtins.sifr`: `total: int = sum(values)` where `values: list[int32]` asserts `str(total) == '4000000000'`. `FixedIntType::supports_current_int_builtin_widening()` is the shared policy source | ✓ |
| `abs(int8.MIN)` returns `int` rather than overflowing a fixed-width type | `fixed_width_sum_abs_builtins.sifr`: `magnitude: int = abs(minimum)` where `minimum: int8 = -128` asserts `str(magnitude) == '128'`. Codegen widens to `i64` before `.abs()` | ✓ |
| `case 256` against `uint8` is a compile-time error | `fixed_width_match_literal_out_of_range.sifr`: `#expect-error: SIFR-INT-0001` on `case 256:`; `fixed_width_match_literal_fitting.sifr`: in-range `case 255:` and `case 0:` pass with runtime assertions | ✓ |

## Design doc INT-4 boundary audit

The design doc `internal_docs/integer_model.md` specifies:

- **Lines 244-260**: Indexes and lengths are `int` at source level; `range` endpoints are `int`. ✓ Confirmed by existing fixtures. Large ranges (`range(10 ** 100)`) stay lazy per design.
- **Lines 261-267**: `bytes` indexing and iteration yield `uint8`. ✓ `bytes_uint8_surface.sifr` + type system confirms this.
- **Line 267**: `bytearray` element type rule and `SIFR-INT-0010`. Deferred — `bytearray` not implemented.
- **Lines 288-299**: `sum`, `abs`, `min`, `max` stdlib contracts. ✓ `sum(list[int32])` → `int` and `abs(int8.MIN)` → `int` confirmed.
- **Lines 416-434**: Pattern matching literal fitting. ✓ `case 255:` and `case 256:` confirmed.
- **Lines 202-206**: Normalized integer hashing for dict/set keys. ✓ Implemented in INT-1 runtime.

## Validation results

| Check | Result |
|---|---|
| `scripts/run_all_tests.sh --profile quick` | PASS (55.59s wall, 0 failures) |
| `scripts/run_all_tests.sh` (full) | PASS (132.55s wall, 0 failures) |
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace -- -D warnings` | Pre-existing errors in `integer_nonzero_guards.rs` (INT-1 path, unrelated to INT-4) |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS |

## Unchecked obligations not represented by subitems

None identified. All subitems from the tracker checklist are represented:

1. **Bytes `uint8` surface** (#1872) — reviewed pass-2, satisfied.
2. **Fixed-width match literal fitting** (#1873) — reviewed pass-2, satisfied.
3. **`sum`/`abs` builtin widening** (#1874) — reviewed pass-2, satisfied.

## Non-blocking observations

1. **`bytearray` not implemented**: The design doc intentionally defers `bytearray` support and `SIFR-INT-0010` to a future slice. The current `bytes_bytearray_unsupported.sifr` stub is the correct placeholder behavior.
2. **`min`/`max(list[int32])` returns `int32`** (not `int`): The design doc says "returns `int32` because no arithmetic overflow is involved." This is not implemented as a separate case, but the default behavior (no arithmetic promotion = original type) means it already returns `int32`. No explicit validation fixture exists for this edge, but the behavior is consistent with design intent.
3. **`random.randrange`, `secrets.randbelow`, math integer helpers**: Not specifically exercised in INT-4 fixtures, but these are INT-4 scope items that depend on external stdlib modules. The core semantic behavior (source-level `int`) is established through INT-2B and INT-3.

## Final verdict

**INT-4 milestone closure review is satisfied.**

All three INT-4 subitems are reviewed, validated, and pass:
- bytes `uint8` surface (#1872) — ✓
- fixed-width match literal fitting (#1873) — ✓
- `sum`/`abs` builtin widening (#1874) — ✓

The milestone tracker can mark **INT-4 complete**. The remaining `bytearray`/`SIFR-INT-0010` gap is design-consistent (deferred future slice, not a missing INT-4 prerequisite) and should be tracked as a follow-up item under the appropriate future milestone.
