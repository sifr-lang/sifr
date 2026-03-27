# wave_psp_rng_3 Review Pass 1

**Phase**: `ad-hoc-stateful-rng-crypto-and-polish-parity-expansion`
**Wave**: `wave_psp_rng_3`
**Date**: 2026-03-21
**Commit**: `b4fb1105`

## Summary

This wave implements final polish waiver reduction for `sifr.statistics`, residual `sifr.textwrap`, and confirms `sifr.html` boundary governance.

---

## 1. Root-Cause Correctness

### 1.1 `sifr.statistics` — `median_grouped(data, interval)`

**Implementation location**: `lib/sifr/statistics.sifr:139-165`

**Correctness analysis**:
- Formula used: `lower + interval * ((n/2 - cf) / f)`
  - `lower` = midpoint - interval/2
  - `cf` = cumulative frequency below midpoint
  - `f` = frequency at midpoint
- Error handling covers:
  - Empty data → `StatisticsError("median_grouped requires at least one data point")`
  - Non-positive interval → `StatisticsError("median_grouped: interval must be > 0")`
  - Zero frequency at midpoint → `StatisticsError("median_grouped: grouped frequency is zero")`
- CPython verification: `statistics.median_grouped([1.0, 2.0, 2.0, 3.0, 4.0], 1.0)` returns `2.25` ✓

**Test case**: `cpython_statistics_subset.sifr` line 41-50 tests the function with interval=1.0 and validates result ≈ 2.25.

### 1.2 `sifr.textwrap` — Formatter Options

**Implementation location**: `lib/sifr/textwrap.sifr`

| Option | Lines | Implementation |
|--------|-------|----------------|
| `fix_sentence_endings` | 206-232 | `_apply_sentence_endings_line()` adds double space after `.`, `!`, `?` |
| `max_lines` | 242-287 | `_apply_max_lines()` truncates output to N lines |
| `placeholder` | 269-272 | Custom placeholder text (default: `" [...]"`) |

**Correctness analysis**:
- `fix_sentence_endings`: Correctly detects sentence-ending punctuation and ensures single trailing space is converted to double space.
- `max_lines`: Correctly handles:
  - `None` (no limit) → returns all lines
  - `≤ 0` → returns empty list
  - `≥ len(lines)` → returns all lines
  - Truncation with placeholder replacement
- `placeholder`: Correctly truncates to fit available width

**Test cases**:
- `phase_psp_rng_3_textwrap_formatter_options.sifr` validates all three options work correctly
- Demo file `ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr` demonstrates integration

---

## 2. Waiver/Governance Accuracy

### 2.1 Waivers Closed

| Waiver | Evidence | Status |
|--------|----------|--------|
| `textwrap.TextWrapper(max_lines=...)` | Previously in `crates/sifr/tests/e2e/fail/phase_psp_rng_0_textwrap_max_lines_unsupported.sifr` | **Closed** — feature shipped |
| `textwrap.TextWrapper(fix_sentence_endings=...)` | New implementation | **Closed** — feature shipped |
| `textwrap.TextWrapper(placeholder=...)` | New implementation | **Closed** — feature shipped |

### 2.2 Governance Alignment

**Per `wave_psp_rng_3_cpython_traceability.md`**:
- `statistics.median_grouped` → `adapted` (shipped)
- `textwrap` residual formatter options → `adapted` (shipped)
- `html` top-level boundary → `adapted` (shipped boundary)

**Per `milestone_psp_7_parity_governance_inventory.md`**:
- `statistics`: `wave_psp_e1 + wave_psp_rng_3` → `parity-closed` ✓
- `textwrap`: `wave_psp_c2 + wave_psp_struct_4 + wave_psp_rng_3` → `parity-closed` ✓
- `html`: `wave_psp_c2 + wave_psp_struct_4 + wave_psp_rng_3` → `parity-closed` ✓

### 2.3 Residual Explicit Waivers

Per traceability document:
- Decimal/Fraction/context-sensitive statistics semantics remain explicitly unsupported
- Package-wide `html.parser` families remain explicitly unsupported
- No residual `textwrap` formatter-option waiver remains

---

## 3. CPython Traceability

### 3.1 Harvest Sources

| CPython family | Evidence |
|----------------|----------|
| `test_statistics` | `Lib/test/test_statistics.py` — `median_grouped` deterministic surface |
| `test_textwrap` | `Lib/test/test_textwrap.py` — residual formatter options |
| `test_html` | `Lib/test/test_html.py` — top-level module boundary |

### 3.2 Local Fixture Anchors

**Positive fixtures**:
- `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr` — includes `median_grouped` test case
- `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr` — existing textwrap tests
- `crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr` — new formatter options tests

**Demo**:
- `demos/ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr` — end-to-end demonstration

**Negative fixture removed**:
- `crates/sifr/tests/e2e/fail/phase_psp_rng_0_textwrap_max_lines_unsupported.sifr` — **deleted** (waiver closed)

---

## 4. Test Coverage

### 4.1 Execution Verification

All test fixtures pass:

```bash
cargo run -q -p sifr -- run demos/ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr  # ✓
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr  # ✓
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr  # ✓
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr  # ✓
```

### 4.2 Coverage Analysis

| Component | Test cases | Coverage |
|-----------|------------|----------|
| `median_grouped` | 1 (positive) | Basic correctness validated |
| `fix_sentence_endings` | 1 (positive) | Double-space insertion validated |
| `max_lines` | 2 (default + custom placeholder) | Truncation + placeholder replacement |
| `placeholder` | 1 (custom) | Width-based truncation |

---

## 5. Findings

### 5.1 Strengths

1. **Root-cause correctness**: `median_grouped` implementation correctly implements the grouped median formula with proper error handling for all edge cases.

2. **Complete waiver closure**: All three textwrap formatter options (`fix_sentence_endings`, `max_lines`, `placeholder`) are now shipped, removing the last residual textwrap waivers.

3. **Governance alignment**: The milestone inventory correctly documents the terminal state for all affected modules (`statistics`, `textwrap`, `html`) with proper wave attribution.

4. **Test coverage**: New test fixtures validate the shipped functionality with clear assertions.

### 5.2 Observations

1. **Negative fixture removal**: The deleted fail fixture (`phase_psp_rng_0_textwrap_max_lines_unsupported.sifr`) correctly reflects the closed waiver — the feature is now supported.

2. **Deterministic semantics**: `median_grouped` follows CPython's deterministic behavior for grouped data, not a statistical approximation.

---

## 6. Recommendation

**Status**: ✅ **Approved**

The wave_psp_rng_3 implementation is correct, complete, and properly governance-aligned. All changes are traceable to CPython sources, test coverage is adequate, and waiver/governance documentation is accurate.

---

## 7. Sign-off

| Role | Name | Date |
|------|------|------|
| Reviewer | Claude | 2026-03-21 |
| Phase owner | Yaser Al-Najjar | — |
