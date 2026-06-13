# wave_psp_c2 Review - Pass 2 (Gap Analysis & CPython Parity)

**Reviewer:** Claude (External Review)
**Scope:** Text, Pattern, and Formatting Modules (`string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, `calendar`)
**Date:** 2026-03-17

---

## Executive Summary

The wave_psp_c2 implementation has **one critical test bug** in difflib that creates a false sense of CPython parity. All other items from the previous review have been resolved or are properly documented as waivers. The implementation is functional but the test suite contains incorrect CPython reference expectations.

---

## Critical Finding

### 1. difflib.SequenceMatcher Test Uses Wrong CPython Reference Behavior

**Files affected:**
- `crates/sifr/tests/e2e/pass/cpython_difflib_subset.sifr:9-21`
- `crates/sifr/tests/e2e/pass/phase_psp_c2_text_pattern_formatting.sifr:95-99`

**Issue:** The tests expect behavior that only occurs with `SequenceMatcher(None, a, b)` (autojunk=False), but the test uses direct string arguments like `SequenceMatcher("abcd", "abed")` which defaults to autojunk=True in CPython.

**Evidence:**

| Input | Test Expects | CPython `SequenceMatcher(a, b)` | CPython `SequenceMatcher(None, a, b)` |
|-------|---------------|----------------------------------|----------------------------------------|
| "abcd" vs "abed" | `[(0, 0, 2), (3, 3, 1), (4, 4, 0)]`, ratio=0.75 | `[Match(a=4, b=0, size=0)]`, ratio=0.0 | `[(0, 0, 2), (3, 3, 1), (4, 4, 0)]`, ratio=0.75 |
| "kitten" vs "sitting" | `[(1, 1, 3), (5, 5, 1), (6, 7, 0)]`, ratio≈0.62 | `[Match(a=7, b=0, size=0)]`, ratio=0.0 | `[(1, 1, 3), (5, 5, 1), (6, 7, 0)]`, ratio≈0.62 |

**Impact:**
- Tests pass but measure wrong reference behavior
- Sifr implementation actually implements non-junk matching (correct algorithm but different default)
- Users comparing Sifr to CPython documentation may get unexpected results

**Traceability claim vs shipped behavior:**
- Claim in `wave_psp_c2_cpython_traceability.md`: "adapted - close-match + matcher object model"
- Actual: Implementation matches `autojunk=False` behavior, not CPython's default

**Recommendation:** Either:
1. Document that Sifr uses non-junk matching by default (more intuitive behavior)
2. Or add explicit `autojunk` parameter support and default to True for CPython parity

---

## Resolved Issues from Pass 1

### 2. calendar._month_name_lookup - FIXED

**Previous issue:** Redundant/unclear logic with `if label is not None` check

**Current state:** Simplified to direct return (lines 54-57 in `lib/sifr/calendar.sifr`):
```python
def _month_name_lookup(month: int) -> str | None:
    if month < 1 or month > 12:
        return None
    return month_name[month]
```

**Status:** Resolved.

---

### 3. textwrap.TextWrapper Width/Indent - FIXED

**Previous issue:** Potential width overflow with indentation

**Current state:** Implementation now properly accounts for indents using `_effective_content_width()` (lines 53-57, 59-90 in `lib/sifr/textwrap.sifr`):
```python
def _effective_content_width(total_width: int, indent: str) -> int:
    available: int = total_width - len(indent)
    if available <= 0:
        return 1
    return available
```

**Status:** Resolved.

---

### 4. string.Template $! Validation - VERIFIED WORKING

**Previous issue:** Needed verification that `$!` triggers proper error

**Current state:** Verified working - test at `cpython_string_template_subset.sifr:28-33` passes:
```python
invalid_placeholder_ok: bool = False
try:
    _bad: str = Template("bad $! token").substitute({})
except ValueError as e:
    invalid_placeholder_ok = e.message.startswith("invalid template placeholder")
```

**Status:** Resolved - implementation correctly validates invalid placeholder characters.

---

## Documented Waivers (Acceptable)

### 5. fnmatch Character Classes - DOCUMENTED

**Status:** Documented as `adapted` in traceability matrix

- `[abc]`, `[!abc]`, `[a-z]` not implemented
- Implementation treats character classes as literal characters
- Test at `cpython_fnmatch_translate_subset.sifr` only tests `*` and `?` wildcards

---

### 6. string.Formatter Advanced Features - DOCUMENTED

**Status:** Documented as waived in traceability matrix

- Format specs: `{name:>10}`, `{value:.2f}` - NOT implemented
- Conversions: `{name!r}`, `{name!s}` - NOT implemented
- Implementation provides basic key-based substitution only

---

### 7. difflib Advanced Classes - DOCUMENTED

**Status:** Documented as waived

- `Differ`, `HtmlDiff`, full opcode/group APIs - NOT implemented
- Only `SequenceMatcher` and `get_close_matches` implemented

---

### 8. calendar Full Rendering - DOCUMENTED

**Status:** Documented as waived

- Full rendering family and locale formatting - NOT implemented
- Only constants/helper and core class entry surfaces implemented

---

## Test Coverage Assessment

### Adequate Coverage
- Error paths tested (missing values, invalid months, etc.)
- Multiple module integration test (`phase_psp_c2_text_pattern_formatting.sifr`) covers cross-module interactions
- Base64 RFC4648 vectors properly tested
- HTML escape/unescape round-trip tested

### Gaps
- **difflib:** Test uses wrong CPython reference (autojunk=False vs default autojunk=True)
- fnmatch: No test for character class handling (documented waiver)
- textwrap: Limited edge case testing for width/indent combinations

---

## Actionable Findings

| Priority | Finding | Action Required |
|----------|---------|-----------------|
| **HIGH** | difflib test uses wrong CPython reference | Fix test expectations or document non-junk behavior |
| LOW | fnmatch character class gaps | Already documented as adapted - no action needed |
| LOW | Formatter format spec gaps | Already documented as waived - no action needed |

---

## Conclusion

The wave_psp_c2 implementation is functional but has one critical test bug in difflib that creates false parity with CPython. The Sifr implementation actually implements the non-junk matching algorithm (which is arguably more intuitive), but the tests incorrectly expect this to match CPython's default junk-aware behavior.

All other findings from Pass 1 have been resolved or are properly documented as waivers. The implementation quality is good; the test suite needs correction for difflib.
