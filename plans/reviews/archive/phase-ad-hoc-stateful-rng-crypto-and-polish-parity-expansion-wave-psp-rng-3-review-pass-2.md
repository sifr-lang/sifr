# wave_psp_rng_3 Review Pass 2

**Phase**: `ad-hoc-stateful-rng-crypto-and-polish-parity-expansion`
**Wave**: `wave_psp_rng_3`
**Date**: 2026-03-21
**Commit**: `faf80518` (review pass 1 recorded), `b4fb1105` (implementation merged)
**Review Pass**: 2 (production-grade)

## Executive Summary

The `wave_psp_rng_3` implementation is **PRODUCTION-GRADE**. This wave delivers the final polish waiver reduction for `sifr.statistics`, `sifr.textwrap`, and confirms the `sifr.html` boundary governance. All three previously-waived `TextWrapper` formatter options are now shipped, `median_grouped` is correctly implemented, and the `html` package-wide parser family boundary is re-confirmed. No code changes are required.

**Status**: APPROVED for production deployment.

---

## 1. Production-Grade Verification

### 1.1 Local Validation

Full quick profile validation completed:

```
HIR maintainability guardrails: PASS
sifr_driver maintainability guardrails: PASS
cargo test -p sifr -- --skip test_e2e_pass: 37 passed, 0 failed
e2e fail/runtime/corpus lane: 25 passed, 0 failed
validation contract matrix (frontend_mode_parity, phase23_graph_isolation): 7 rows, PASS
e2e pass suite (profile=quick): 24 fixtures, PASS
```

Report signature: `e1bf653aaa770517` (e2e pass suite).

### 1.2 Wave-Specific Test Execution

| Command | Result |
|---|---|
| `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr` | PASS |
| `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr` | PASS |
| `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr` | PASS |
| `cargo run -q -p sifr -- run demos/ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr` | PASS |
| `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` | expected compile failure (PASS) |

All wave-specific test fixtures pass. The negative fixture for `html.parser` correctly rejects the unsupported boundary.

---

## 2. Root-Cause Correctness Review

### 2.1 `sifr.statistics` — `median_grouped(data, interval)`

**Implementation**: `lib/sifr/statistics.sifr:139-165`

**Formula verification**: `lower + interval * ((n/2 - cf) / f)`
- `lower = midpoint - interval / 2.0` ✓
- `cf` = cumulative frequency below midpoint ✓
- `f` = frequency at midpoint ✓

**Edge case handling**:
- Empty data → `StatisticsError("median_grouped requires at least one data point")` ✓
- Non-positive interval → `StatisticsError("median_grouped: interval must be > 0")` ✓
- Zero frequency at midpoint → `StatisticsError("median_grouped: grouped frequency is zero")` ✓
- Index error → `StatisticsError("median_grouped: index error")` ✓

**CPython verification**: `statistics.median_grouped([1.0, 2.0, 2.0, 3.0, 4.0], 1.0)` → `2.25` ✓

**Root cause**: The implementation correctly counts the frequency of the midpoint value (not just checks its existence) to compute the grouped median. The formula is deterministic and matches CPython's behavior.

### 2.2 `sifr.textwrap` — Formatter Options

#### `fix_sentence_endings` (`lib/sifr/textwrap.sifr:206-232`)

**Algorithm**: `_apply_sentence_endings_line()` scans each character; when a sentence-ending punctuation (`.`, `!`, `?`) is followed by a single space, it inserts a second space.

**Correctness**: This matches CPython's behavior for ensuring double-space after sentence endings. The implementation correctly:
- Detects `.`, `!`, `?` as sentence-ending characters ✓
- Checks that the next character is a single space ✓
- Inserts a second space only when not already double-spaced ✓
- Applies to each line independently via `_apply_sentence_endings_lines()` ✓

#### `max_lines` (`lib/sifr/textwrap.sifr:242-287`)

**Algorithm**: `_apply_max_lines()` takes the wrapped lines, applies the `max_lines` limit, and replaces the truncated last line's content with the placeholder.

**Edge cases handled**:
- `max_lines = None` → returns all lines unchanged ✓
- `max_lines <= 0` → returns empty list ✓
- `len(lines) <= max_lines` → returns all lines unchanged ✓
- Truncation replaces the last line's tail with placeholder ✓
- Placeholder is width-clipped if it exceeds available width ✓

**Correctness**: The implementation correctly handles all boundary conditions and properly replaces the truncated content with the placeholder while preserving the width constraint.

#### `placeholder` (`lib/sifr/textwrap.sifr:269-272`)

**Default value**: `" [...]"` (with leading space) ✓
**Width enforcement**: If placeholder exceeds available width, it is clipped to fit ✓

**Correctness**: The placeholder is correctly applied as the last token of the truncated line, and its length is properly accounted for in the width calculation.

### 2.3 `sifr.html` — Top-Level Boundary Governance

**Implementation**: `lib/sifr/html.sifr` provides `escape()` and `unescape()` as top-level closures. The package-wide `html.parser` family is explicitly not in scope.

**Boundary enforcement verified**:
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` → correctly rejects `sifr.html.parser` ✓

**Governance alignment**: The wave-3 traceability doc correctly classifies `html` as `adapted` (shipped boundary) with the explicit caveat that the `html.parser` family remains `unsupported`.

---

## 3. Waiver Closure Review

### 3.1 Waivers Closed by Wave 3

| Waiver | Status | Evidence |
|---|---|---|
| `textwrap.TextWrapper(max_lines=...)` | **Closed** | Was in `phase_psp_rng_0_textwrap_max_lines_unsupported.sifr` → fixture deleted |
| `textwrap.TextWrapper(fix_sentence_endings=...)` | **Closed** | New implementation shipped |
| `textwrap.TextWrapper(placeholder=...)` | **Closed** | New implementation shipped |

### 3.2 Residual Explicit Waivers After Wave 3

Per `wave_psp_rng_3_cpython_traceability.md`:

| Waiver | Classification | Rationale |
|---|---|---|
| Decimal/Fraction/context-sensitive statistics semantics | `unsupported` | Float/int deterministic surfaces only in this phase |
| Package-wide `html.parser` family | `unsupported` | Top-level `html.escape`/`html.unescape` closure only; parser ecosystem out of scope by design |
| SHA3/SHAKE constructor families | `unsupported` | No runtime dependency registered for SHA3/SHAKE |
| `SystemRandom` state export/import | `unsupported` | Host-random is non-deterministic by design |
| `choices(weights=...)` | `unsupported` | Weighted distribution requires additional implementation |

All residual waivers are correctly documented and have negative test fixtures.

---

## 4. Governance / Documentation Consistency

### 4.1 Traceability Chain

| Document | Status | Assessment |
|---|---|---|
| `wave_psp_rng_0_cpython_traceability.md` | Updated | Documents wave-3 ownership for `statistics`, `textwrap`, `html`; historical note added for retired waiver fixture |
| `wave_psp_rng_1_cpython_traceability.md` | Current | CPython `test_random` case mapping; wave-1 coverage documented |
| `wave_psp_rng_2_cpython_traceability.md` | Current | CPython `test_hashlib`/`test_base64` mapping; SHA3/SHAKE waiver documented |
| `wave_psp_rng_3_cpython_traceability.md` | New | CPython `test_statistics`/`test_textwrap`/`test_html` harvest; waiver closure documented |
| `phase_psp_rng_architecture_lock.md` | Updated | Historical note added for retired waiver fixture |
| `milestone_psp_7_parity_governance_inventory.md` | Updated | Terminal state correctly updated for `statistics`, `textwrap`, `html` to `parity-closed` |

### 4.2 Execution Ledger

`issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-execution.md` correctly records:
- Wave-3 implementation merged via PR `#1382`
- Review pass 1 validated with no code-change findings
- Wave-3 status: `implementation merged; external review pass 1 validated`

### 4.3 Documentation Cross-References

All documentation cross-references are internally consistent:

| Cross-reference | Status |
|---|---|
| Traceability docs → local anchors | All files referenced exist |
| Inventory → traceability docs | Correct wave attribution for `statistics` (`wave_psp_e1 + wave_psp_rng_3`), `textwrap` (`wave_psp_c2 + wave_psp_struct_4 + wave_psp_rng_3`), `html` (`wave_psp_c2 + wave_psp_struct_4 + wave_psp_rng_3`) |
| Execution ledger → PR numbers | PR `#1382` (implementation) and `#1383` (review pass 1) exist |
| Phase doc → execution ledger | Correct links maintained |

### 4.4 Phase-Level Consistency

| Check | Status |
|---|---|
| All waves before wave_psp_rng_3 marked as completed | ✓ (`wave_psp_rng_0`, `wave_psp_rng_1`, `wave_psp_rng_2` all completed |
| Wave-3 scope matches phase plan | ✓ `statistics` waiver reduction, `textwrap` residual closure, `html` boundary re-confirmation |
| Governance inventory reflects wave-3 changes | ✓ `statistics`, `textwrap`, `html` all marked `parity-closed` |

---

## 5. Test Coverage Analysis

### 5.1 Positive Coverage

| Fixture | Coverage | Result |
|---|---|---|
| `cpython_statistics_subset.sifr` | `median_grouped` correctness + error boundaries | PASS |
| `cpython_textwrap_textwrapper_subset.sifr` | Existing textwrap coverage (pre-wave-3) | PASS |
| `phase_psp_rng_3_textwrap_formatter_options.sifr` | `fix_sentence_endings`, `max_lines`, `placeholder` | PASS |
| `ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr` | End-to-end integration (statistics + textwrap + html) | PASS |

### 5.2 Negative Coverage

| Fixture | Coverage | Result |
|---|---|---|
| `phase_psp_struct_0_html_package_parser_unsupported.sifr` | `html.parser` boundary rejection | PASS (expected compile failure) |

### 5.3 Coverage Adequacy

- `median_grouped`: 1 positive test case (correctness) + 1 error case (interval=0) ✓
- `fix_sentence_endings`: 1 test case covering double-space insertion ✓
- `max_lines`: 2 test cases (default placeholder + custom placeholder) ✓
- `placeholder`: 1 test case (custom placeholder with width clipping) ✓
- `html`: 1 negative fixture (package parser boundary) ✓

The test coverage is adequate for production-grade validation. Edge cases are covered through error-path tests in `cpython_statistics_subset.sifr`.

---

## 6. Findings

### 6.1 Strengths

1. **Complete waiver closure**: All three previously-waived `TextWrapper` formatter options (`fix_sentence_endings`, `max_lines`, `placeholder`) are now shipped with correct implementations.

2. **Correct median_grouped formula**: The grouped median implementation correctly computes the frequency of the midpoint value and applies the standard interpolation formula.

3. **Proper governance closure**: The milestone inventory correctly documents the terminal state for `statistics`, `textwrap`, and `html` with accurate wave attribution (`wave_psp_e1 + wave_psp_rng_3` for statistics, etc.).

4. **Consistent documentation**: All traceability docs, execution ledger, and governance inventory are cross-referenced and consistent.

5. **Clean residual waiver state**: No residual textwrap formatter-option waivers remain. The only waivers after wave 3 are explicitly classified (`html.parser`, SHA3/SHAKE, SystemRandom state, weighted choices).

### 6.2 Observations

1. **No Rust codegen changes in wave 3**: Wave 3 only adds pure-Sifr implementations (`median_grouped` in `statistics.sifr`, formatter options in `textwrap.sifr`). No intrinsic or codegen changes were required.

2. **Pre-existing clippy issue unrelated to wave 3**: `RuntimeNeeds` struct in `sifr_codegen/src/lib.rs` has 4 bools (clippy: `struct_excessive_bools`). This was introduced in `wave_psp_rng_1` (commit `b7462b0d`), not in wave 3. Not a blocker.

3. **Minor fmt drift in pre-existing Rust files**: Some Rust files in `sifr_codegen` have fmt drift (e.g., `hashlib.rs`, `mod.rs`, `preamble.rs`). These are from pre-existing wave-1/wave-2 code, not from wave 3 changes. Not a blocker.

4. **Demo file demonstrates full integration**: `ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr` correctly demonstrates all three wave-3 modules working together (statistics + textwrap + html).

---

## 7. Review Pass 1 Follow-Up

Review pass 1 (commit `faf80518`) identified no code changes required. This review pass 2 validates:

1. **Production-grade criteria met**: All test suites pass, governance docs are consistent, waiver inventory is accurate.

2. **No regressions introduced**: Wave-3 changes are additive only (new functions + new test fixtures). No breaking changes to existing APIs.

3. **Traceability preserved**: All wave-3 changes are traceable to CPython harvest inputs (`test_statistics.py`, `test_textwrap.py`, `test_html.py`).

---

## 8. Recommendation

**Status**: ✅ **APPROVED FOR PRODUCTION DEPLOYMENT**

The `wave_psp_rng_3` implementation satisfies all production-grade criteria:

1. **Root-cause correctness**: All implementations match CPython behavior with proper typed error boundaries
2. **Complete waiver closure**: All three textwrap formatter options shipped; no residual textwrap formatter waivers remain
3. **Governance accuracy**: Traceability, inventory, and execution docs are consistent and accurate
4. **Test coverage**: Positive and negative coverage adequate; all fixtures pass
5. **Local validation**: Full quick profile validation passes

### Action Items

None required. The wave is production-ready.

---

## 9. Sign-off

| Role | Name | Date |
|------|------|------|
| Reviewer | agent | 2026-03-21 |
| Phase owner | Yaser Al-Najjar | — |
