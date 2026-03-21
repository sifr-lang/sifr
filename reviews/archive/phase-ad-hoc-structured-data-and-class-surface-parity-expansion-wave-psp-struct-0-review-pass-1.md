# wave_psp_struct_0 Review Pass 1 (Completion Gap)

**Phase**: `ad-hoc-structured-data-and-class-surface-parity-expansion`
**Wave**: `wave_psp_struct_0` - Architecture Lock
**Reviewer**: Claude (Pass 1 - Completion Gap)
**Date**: 2026-03-18
**Status**: **ISSUES FOUND - Requires Fix**

---

## Executive Summary

The `wave_psp_struct_0` implementation provides a comprehensive architecture lock for the structured-data/class-surface continuation phase. Most components are correctly implemented, but **one critical bug** was identified: two negative-path test files have swapped content that does not match their filenames.

---

## Review Criteria Assessment

### 1. Fixed Contract Clarity ✅

**Status**: PASS

The `verification/stdlib/phase_psp_struct_architecture_lock.md` provides clear documentation of:

| Surface | Locked Direction |
|---------|-----------------|
| `json` | Keep typed top-level entry points; add typed wrappers; no dynamic hooks |
| `configparser` | Expand interpolation/proxy/write-back with explicit bounds |
| `csv` | Iterator-returning APIs; bounded process-local dialect registry |
| `collections` | Expand `Counter(iterable)` and `Counter(mapping)`; keep `**kwargs` out |
| `argparse` | Expand `subparsers`, bounded `nargs`, typed `type=` coercers |
| `uuid` | Add `uuid3`, `uuid5`, namespace constants; strict constructor parity |
| `datetime` | Fixed-offset timezone only; no zone database/DST/fold |
| `textwrap` | Expand through explicit adjacent option fields |
| `html` | Keep scope bounded to `escape`/`unescape` family |

The contract is explicit, well-structured, and provides clear direction for subsequent waves.

---

### 2. Permanent-Diff Enforcement Fixtures ⚠️

**Status**: FAIL - Content Mismatch Bug Found

#### Positive Path Fixtures ✅

| Fixture | Existence | Execution |
|---------|-----------|-----------|
| `phase_psp_struct_0_architecture_lock.sifr` | ✅ Exists | ✅ PASS |
| `ad_hoc_struct_wave0_json_wrapper_model_demo.sifr` | ✅ Exists | ✅ PASS |
| `ad_hoc_struct_wave0_fixed_offset_datetime_model_demo.sifr` | ✅ Exists | ✅ PASS |

#### Negative Path Fixtures ⚠️

| Fixture | File Exists | Content Match | Execution |
|---------|-------------|---------------|------------|
| `phase_psp_struct_0_json_dynamic_hooks_unsupported.sifr` | ✅ | ❌ **MISMATCH** | ✅ Fail (correct behavior, wrong test) |
| `phase_psp_struct_0_datetime_tzinfo_zoneinfo_unsupported.sifr` | ✅ | ✅ | ✅ Fail (correct) |
| `phase_psp_struct_0_counter_kwargs_constructor_unsupported.sifr` | ✅ | ❌ **MISMATCH** | ✅ Fail (correct behavior, wrong test) |
| `phase_psp_struct_0_csv_dynamic_registry_unsupported.sifr` | ✅ | ✅ | ✅ Fail (correct) |
| `phase_psp_struct_0_argparse_formatter_class_unsupported.sifr` | ✅ | ✅ | ✅ Fail (correct) |
| `phase_psp_struct_0_html_package_parser_unsupported.sifr` | ✅ | ✅ | ✅ Fail (correct) |

#### Critical Bug Detail

**File content swap detected:**

1. **`phase_psp_struct_0_json_dynamic_hooks_unsupported.sifr`** contains:
   ```sifr
   from sifr.collections import Counter

   def main() -> None:
       _counts: Counter[str] = Counter(alpha=1, beta=2)  # Counter kwargs test
   ```
   Should contain: JSON dynamic hooks test (`object_hook`, `object_pairs_hook`, etc.)

2. **`phase_psp_struct_0_counter_kwargs_constructor_unsupported.sifr`** contains:
   ```sifr
   from sifr.json import loads

   def _object_hook(value: str) -> str:
       return value

   def main() -> None:
       _decoded = loads("{\"a\":1}", _object_hook)  # JSON hooks test
   ```
   Should contain: Counter kwargs test

**Impact**: Both tests still fail (correctly rejecting unsupported features), but the test content does not match what the filename claims to test. This creates confusion and reduces the traceability of the test suite.

---

### 3. CPython Family Mapping ✅

**Status**: PASS

The architecture lock includes a comprehensive CPython Family Mapping table:

| CPython Family | Module | Direction | Wave |
|----------------|--------|-----------|------|
| `Lib/test/test_json/` | `json` | adapted | `wave_psp_struct_1` |
| `Lib/test/test_configparser.py` | `configparser` | adapted | `wave_psp_struct_1` |
| `Lib/test/test_csv.py` | `csv` | adapted | `wave_psp_struct_1` |
| `Lib/test/test_collections.py` | `collections` | adapted | `wave_psp_struct_2` |
| `Lib/test/test_argparse.py` | `argparse` | adapted | `wave_psp_struct_2` |
| `Lib/test/test_uuid.py` | `uuid` | adapted | `wave_psp_struct_3` |
| `Lib/test/test_datetime.py` | `datetime` | adapted | `wave_psp_struct_3` |
| `Lib/test/test_textwrap.py` | `textwrap` | adapted | `wave_psp_struct_4` |
| `Lib/test/test_html.py` | `html` | adopted/adapted | `wave_psp_struct_4` |

The mapping is also referenced in the milestone governance inventory (`milestone_psp_7_parity_governance_inventory.md`).

---

### 4. Demos ✅

**Status**: PASS

| Demo | Location | Validation |
|------|----------|-------------|
| JSON wrapper model | `demos/ad_hoc_struct_wave0_json_wrapper_model_demo.sifr` | ✅ PASS |
| Fixed-offset datetime | `demos/ad_hoc_struct_wave0_fixed_offset_datetime_model_demo.sifr` | ✅ PASS |

Both demos execute successfully and demonstrate the locked contracts in action.

---

### 5. Execution Checklist Alignment ✅

**Status**: PASS (with fixture bug noted)

From `issues/ad-hoc-structured-data-and-class-surface-parity-expansion-execution.md`:

| Validation | Command | Expected | Actual |
|------------|---------|----------|--------|
| Positive: architecture lock | `cargo run -q -p sifr -- run ...phase_psp_struct_0_architecture_lock.sifr` | PASS | ✅ PASS |
| Positive: JSON demo | `cargo run -q -p sifr -- run ...ad_hoc_struct_wave0_json_wrapper_model_demo.sifr` | PASS | ✅ PASS |
| Positive: datetime demo | `cargo run -q -p sifr -- run ...ad_hoc_struct_wave0_fixed_offset_datetime_model_demo.sifr` | PASS | ✅ PASS |
| Negative: JSON dynamic hooks | `cargo run -q -p sifr -- check ...phase_psp_struct_0_json_dynamic_hooks_unsupported.sifr` | FAIL | ✅ FAIL |
| Negative: datetime tzinfo | `cargo run -q -p sifr -- check ...phase_psp_struct_0_datetime_tzinfo_zoneinfo_unsupported.sifr` | FAIL | ✅ FAIL |
| Negative: Counter kwargs | `cargo run -q -p sifr -- check ...phase_psp_struct_0_counter_kwargs_constructor_unsupported.sifr` | FAIL | ✅ FAIL |
| Negative: CSV registry | `cargo run -q -p sifr -- check ...phase_psp_struct_0_csv_dynamic_registry_unsupported.sifr` | FAIL | ✅ FAIL |
| Negative: argparse formatter | `cargo run -q -p sifr -- check ...phase_psp_struct_0_argparse_formatter_class_unsupported.sifr` | FAIL | ✅ FAIL |
| Negative: html parser | `cargo run -q -p sifr -- check ...phase_psp_struct_0_html_package_parser_unsupported.sifr` | FAIL | ✅ FAIL |

---

## Issues Summary

| Issue | Severity | Description |
|-------|----------|-------------|
| Fixture content swap | **CRITICAL** | `phase_psp_struct_0_json_dynamic_hooks_unsupported.sifr` contains Counter kwargs test; `phase_psp_struct_0_counter_kwargs_constructor_unsupported.sifr` contains JSON hooks test |

---

## Required Actions

1. **Fix fixture content swap**:
   - Replace content of `phase_psp_struct_0_json_dynamic_hooks_unsupported.sifr` with proper JSON dynamic hooks test (using `object_hook`, `object_pairs_hook`, `parse_float`, `parse_int`, `parse_constant`, or `default` parameters to `loads`/`dumps`)
   - Replace content of `phase_psp_struct_0_counter_kwargs_constructor_unsupported.sifr` with proper Counter kwargs test (using `Counter(alpha=1, beta=2)` style constructor)

2. **Re-validate** after fix:
   - Both negative tests should still fail (correctly rejecting unsupported features)
   - Content should match filename claims

---

## Recommendation

**BLOCKED** - The wave cannot proceed to production-grade review until the fixture content swap is corrected. Once fixed, re-run validation and update this review artifact.
