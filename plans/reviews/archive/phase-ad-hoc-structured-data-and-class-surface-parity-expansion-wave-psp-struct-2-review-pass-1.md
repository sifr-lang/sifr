# wave_psp_struct_2 Review Pass 1 (Collections and CLI Class-Surface Expansion)

**Phase**: `ad-hoc-structured-data-and-class-surface-parity-expansion`
**Wave**: `wave_psp_struct_2` - Collections and CLI Class-Surface Expansion
**Reviewer**: agent (Pass 1 - Completion-Gap)
**Date**: 2026-03-18
**Status**: **APPROVED**

---

## Executive Summary

The `wave_psp_struct_2` implementation successfully delivers collections and CLI class-surface expansion for `collections` and `argparse` modules. All required features are implemented according to the phase contract, and both positive and negative path validations pass correctly.

---

## Review Criteria Assessment

### 1. Fixed Contract Clarity ✅

**Status**: PASS

The implementation follows the locked contract from `verification/stdlib/phase_psp_struct_architecture_lock.md`:

| Module | Contract Requirement | Implementation Status |
|--------|---------------------|----------------------|
| `collections` | `Counter(iterable)` and `Counter(mapping)` constructor parity | ✅ Implemented |
| `collections` | `Counter(**kwargs)` out of scope | ✅ Enforced via type system |
| `collections` | Promote `defaultdict` toward explicit class/object semantics | ✅ Implemented as `defaultdict` class |
| `argparse` | Expand `subparsers` | ✅ Implemented via `add_subparsers()` and `add_parser()` |
| `argparse` | Bounded `nargs` matrix (`int`, `?`, `*`, `+`) | ✅ Implemented in `add_argument_typed()` |
| `argparse` | Typed `type=` coercers under deterministic behavior | ✅ Implemented via `type_name` parameter |

---

### 2. Collections Implementation ✅

**Status**: PASS

**Implementation Analysis** (`lib/sifr/collections.sifr`):

| Feature | Method | Status |
|---------|--------|--------|
| Counter | `Counter(source: dict[T, int] \| None, iterable: list[T] \| None)` | ✅ Implemented |
| Counter | `get(key, default)` | ✅ Implemented |
| Counter | `increment(key)` | ✅ Implemented |
| Counter | `total()` | ✅ Implemented |
| Counter | `most_common(n)` | ✅ Implemented |
| Counter | `keys()`, `values()`, `items()` | ✅ Implemented |
| Counter | `copy()`, `clear()`, `update()`, `subtract()` | ✅ Implemented |
| Counter | `elements()` | ✅ Implemented |
| Counter | `__add__`, `__sub__` | ✅ Implemented |
| defaultdict | `class defaultdict[K]` | ✅ Implemented |
| defaultdict | `ensure(key)` | ✅ Implemented |
| defaultdict | `set(key, value)` | ✅ Implemented |
| defaultdict | `has(key)`, `pop(key)`, `clear()` | ✅ Implemented |
| defaultdict | `keys()`, `values()`, `items()`, `len()` | ✅ Implemented |
| deque | Full class implementation | ✅ Previously implemented |

---

### 3. Argparse Implementation ✅

**Status**: PASS

**Implementation Analysis** (`lib/sifr/argparse.sifr`):

| Feature | Method | Status |
|---------|--------|--------|
| ArgumentSpec | Class to hold argument metadata | ✅ Implemented |
| Namespace | Class to hold parsed values | ✅ Implemented |
| Namespace | `set()`, `set_bool()`, `set_list()` | ✅ Implemented |
| Namespace | `get()`, `get_bool()`, `get_list()` | ✅ Implemented |
| Namespace | `merge_from()`, `copy()` | ✅ Implemented |
| ArgumentParser | `add_subparsers(dest)` | ✅ Implemented |
| ArgumentParser | `add_parser(name, parser)` | ✅ Implemented |
| ArgumentParser | `add_argument()` | ✅ Implemented |
| ArgumentParser | `add_argument_typed(name, dest, action, default, nargs, type_name)` | ✅ Implemented |
| ArgumentParser | `parse_args(args)` | ✅ Implemented |
| Type coercion | `int`, `float`, `bool`, `str` | ✅ Implemented |
| nargs support | `1`, `?`, `*`, `+`, integer values | ✅ Implemented |
| Subparser parsing | Command dispatch | ✅ Implemented |

---

### 4. Positive Path Validation ✅

**Status**: PASS

| Test | Command | Expected | Result |
|------|---------|----------|--------|
| Phase test | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_struct_2_collections_argparse_expansion.sifr` | PASS | ✅ PASS |
| Demo | `cargo run -q -p sifr -- run demos/ad_hoc_struct_wave2_collections_argparse_expansion_demo.sifr` | PASS | ✅ PASS |
| Regression: argparse | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_argparse.sifr` | PASS | ✅ PASS |
| Regression: collections | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_collections_consolidated.sifr` | PASS | ✅ PASS |
| Regression: CPython argparse | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_argparse_subset.sifr` | PASS | ✅ PASS |
| Regression: CPython collections | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_collections_subset.sifr` | PASS | ✅ PASS |

---

### 5. Negative Path Validation ✅

**Status**: PASS

| Test | Command | Expected | Result |
|------|---------|----------|--------|
| Counter kwargs | `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_counter_kwargs_constructor_unsupported.sifr` | FAIL | ✅ FAIL (type error: "Counter() got an unexpected keyword argument 'alpha'") |
| Argparse formatter_class | `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_argparse_formatter_class_unsupported.sifr` | FAIL | ✅ FAIL (type error: "ArgumentParser() got an unexpected keyword argument 'formatter_class'") |

---

### 6. Waiver Ledger Compliance ✅

**Status**: PASS

Per `verification/stdlib/milestone_psp_7_parity_governance_inventory.md`:

| Waiver Entry | Status |
|--------------|--------|
| `Counter(**kwargs)` constructor | ✅ Explicitly unsupported - enforced by type system |
| `argparse` formatter-class/help-formatting ecosystems | ✅ Explicitly unsupported - enforced by type system |

---

### 7. Intrinsic Lowering Implementation ✅

**Status**: PASS

**Collections intrinsics** (`crates/sifr_hir/src/stdlib/collections_bytes_time.rs`):

| Intrinsic | Signature | Status |
|-----------|-----------|--------|
| `counter_from_list` | `(items: list[str]) -> str` | ✅ Implemented |
| `counter_get` | `(counter: str, key: str) -> int` | ✅ Implemented |
| `counter_most_common` | `(counter: str, n: int) -> str` | ✅ Implemented |
| `counter_total` | `(counter: str) -> int` | ✅ Implemented |
| `counter_values` | `(counter: str) -> list[int]` | ✅ Implemented |
| `counter_keys` | `(counter: str) -> list[str]` | ✅ Implemented |
| `counter_items` | `(counter: str) -> str` | ✅ Implemented |
| `counter_increment` | `(counter: str, key: str) -> str` | ✅ Implemented |
| `defaultdict_new` | `(default_value: int) -> str` | ✅ Implemented |
| `defaultdict_get` | `(dd: str, key: str) -> int` | ✅ Implemented |
| `defaultdict_set` | `(dd: str, key: str, value: int) -> str` | ✅ Implemented |

**Codegen** (`crates/sifr_codegen/src/intrinsics/collections.rs`):

All collections intrinsics have corresponding lowering implementations that generate correct Rust code.

---

### 8. Architecture Lock Alignment ✅

**Status**: PASS

Per `verification/stdlib/phase_psp_struct_architecture_lock.md`:

| Locked Direction | Implementation |
|-----------------|----------------|
| Expand `Counter(iterable)` and `Counter(mapping)` | ✅ Both constructor forms supported |
| Keep `Counter(**kwargs)` out of scope | ✅ Enforced via type error |
| Promote `defaultdict` toward explicit class/object semantics | ✅ Implemented as full class with methods |
| Expand `subparsers` | ✅ `add_subparsers()`, `add_parser()` implemented |
| Bounded `nargs` matrix (`int`, `?`, `*`, `+`) | ✅ Supported via `_normalize_nargs()` |
| Typed `type=` coercers | ✅ `str`, `int`, `float`, `bool` supported |

---

## Issues Summary

| Issue | Severity | Description |
|-------|----------|-------------|
| None | - | No issues identified |

---

## Required Actions

None - the implementation is complete and meets all contract requirements.

---

## Recommendation

**APPROVED** - The wave can proceed to production-grade review. All required features are implemented, positive and negative path validations pass, and the implementation aligns with the locked architecture contract.

---

## Additional Notes

1. **Counter constructor design**: The `Counter` class supports both `source: dict[T, int]` (mapping) and `iterable: list[T]` constructor forms, matching CPython's API. The `source` parameter takes a dict directly rather than JSON string.

2. **defaultdict type constraint**: The `defaultdict` class is currently typed as `defaultdict[K: Hashable]` with `int` values. This is a simplified version compared to CPython's generic defaultdict that accepts any callable as `default_factory`. The implementation stores `default_factory` as an `int` and uses it as the default value for missing keys.

3. **Argparse pure Sifr implementation**: The entire argparse module is implemented in pure Sifr (`lib/sifr/argparse.sifr`) rather than using intrinsics. This approach provides flexibility and follows the pattern established in earlier waves.

4. **Type coercion**: The argparse implementation handles type coercion for `str`, `int`, `float`, and `bool` types. Bool coercion accepts "1", "true", "yes", "on" as true and "0", "false", "no", "off" as false (case-insensitive).

5. **nargs handling**: The implementation correctly handles:
   - Fixed count: `"1"`, `"2"`, etc.
   - Optional: `"?"` (0 or 1)
   - Variadic: `"*"` (0 or more), `"+"` (1 or more)
