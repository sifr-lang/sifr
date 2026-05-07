

---

# Code Review: INT-4 Fixed-Width Sum/Abs Builtins — Pass 2

## Pass 1 Blocker Status

| Pass 1 Finding | Status |
|---|---|
| 1. `expressions.rs` exceeded HIR maintainability guardrail (3821 > 3800 lines) | **FIXED** — `abs()` lowering extracted to `expression_abs.rs`, module declared in `mod.rs` line 27. Guardrail now passes. |
| 2. Four identical widening predicates | **FIXED** — Single policy method `FixedIntType::supports_current_int_builtin_widening()` added to `sifr_type_system/src/types.rs:135-140`. HIR, codegen, and nested function inference all call this shared method. |
| 3. e2e fixture not discoverable | **FIxt** — Fixture exists at `crates/sifr/tests/e2e/pass/fixed_width_sum_abs_builtins.sifr`. E2E discovery confirmed: passes when manifest is correctly structured (`{"fixture_names": ["fixed_width_sum_abs_builtins"]}`). |

---

## Validation Results

All local validation steps passed:

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `scripts/check_hir_maintainability_guardrails.py` | PASS |
| `cargo test -p sifr_type_system test_fixed_width_current_int_builtin_widening_policy` | PASS |
| `cargo test -p sifr_hir test_abs_fixed_width_builtin_widens_to_int` | PASS |
| `cargo test -p sifr_hir test_sum_fixed_width_iterable_widens_to_int` | PASS |
| `cargo test -p sifr_codegen sum` | PASS (4 tests) |
| `git diff --check` | PASS |
| E2E fixture `fixed_width_sum_abs_builtins` | PASS |

---

## Contract Alignment

The implementation correctly implements the canonical design from `internal_docs/integer_model.md`:

| Contract Rule | Implementation | Status |
|---|---|---|
| `sum(list[int8/int16/int32/uint8/uint16/uint32])` returns `int` | HIR widening + codegen `.map(\|x\| x as i64).sum::<i64>()` | ✓ |
| `sum(list[int64/uint64/isize/usize])` returns fixed-width | No widening applied | ✓ |
| `abs(int8.MIN)` returns `int` (not `int8`) | Codegen widens to `i64` before `.abs()`, e2e fixture asserts `abs(-128) == 128` | ✓ |
| Conservative set correctly scoped | `I8/I16/I32/U8/U16/U32` included; `I64/U64/ISize/USize` excluded | ✓ |

---

## Codegen Safety Assessment

No user-triggerable panics identified:

| Codegen Path | Analysis |
|---|---|
| `sum` for widening types (line 2136–2176) | Uses `sum::<i64>()` with map cast. Iterator consumed and accumulated. No panics. |
| `sum` for non-widening types (line 2165) | Falls through to element type's native `sum`. No additional wrapping. |
| `abs` for widening types (line 2640–2657) | Cast to `i64` before `.abs()`. Prevents `int8::MIN.abs()` overflow. |
| `abs` for non-widening types (line 2651) | Pass-through to native `.abs()`. No additional wrapping. |

---

## Architecture Quality

**Shared policy**: `FixedIntType::supports_current_int_builtin_widening()` in `sifr_type_system` is the canonical source of truth. One call site in each of:
- `expression_abs.rs:48` — HIR `abs()` lowering
- `expression_sum_sorted.rs:58` — HIR `sum()` lowering
- `nested_function_inference.rs:1142,1172` — Nested function type inference
- `intrinsic_method_emitters.rs:2142,2162,2644` — Codegen (all three occurrences are necessary: two in `sum` for element type check, one in `abs`)

**File decomposition**: `expression_abs.rs` is 72 lines, well-scoped. No monolithic files.

---

## Minor Note (Not a Blocker)

The `--fixture-manifest` usage in the review request specified just the file stem (`fixed_width_sum_abs_builtins`) but the actual manifest format requires a JSON object: `{"fixture_names": ["fixed_width_sum_abs_builtins"]}`. The script's help text could be clearer. Not blocking since the fixture is discoverable via normal e2e discovery when no manifest is specified.

---

## Verdict

**All pass 1 blockers resolved. Review satisfied — no further passes required.**

The implementation is correct, contract-aligned, guardrail-compliant, and codegen-safe. The shared policy method eliminates the duplication that was the primary code quality concern.
