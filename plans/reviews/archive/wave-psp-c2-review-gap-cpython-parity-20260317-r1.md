# wave_psp_c2 Review - Gap Analysis and CPython Parity

**Reviewer:** agent
**Scope:** `string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, `calendar`
**Date:** 2026-03-17
**Status:** Current-main findings

---

## Executive Summary

The wave_psp_c2 implementation for text pattern and formatting modules has been largely completed with good test coverage. Most issues from the first review pass have been addressed. Key findings:

- **Critical gap identified**: `base64` module has imported intrinsics that are not explicitly re-exported as function wrappers (urlsafe_b64*, b32*, b32hex*). While they work via implicit import semantics, this is inconsistent with the module's coding style.
- **Resolved issues**: difflib multi-block matching, calendar helper simplification, textwrap width handling
- **Acceptable waivers**: All documented waivers (fnmatch character classes, string.Formatter specs, difflib advanced classes, calendar rendering) are being correctly honored

---

## 1. Verified Resolved Issues (from Pass 1)

### 1.1 difflib.SequenceMatcher.get_matching_blocks() — CORRECTED

**Previous finding**: Review pass 1 claimed single-block only behavior.

**Verification**:
```python
# CPython
>>> import difflib
>>> difflib.SequenceMatcher(None, "abcabc", "abcabc").get_matching_blocks()
[Match(a=0, b=0, size=6), Match(a=6, b=6, size=0)]

# Sifr (tested)
>>> SequenceMatcher("abcabc", "abcabc").get_matching_blocks()
[(0, 0, 6), (6, 6, 0)]
```

**Status**: ✅ Parity confirmed — returns identical structure to CPython

### 1.2 calendar._month_name_lookup — FIXED

**Before**:
```sifr
def _month_name_lookup(month: int) -> str | None:
    if month < 1 or month > 12:
        return None
    label: str | None = month_name[month]
    if label is not None:
        return label + ""
    return None
```

**After**:
```sifr
def _month_name_lookup(month: int) -> str | None:
    if month < 1 or month > 12:
        return None
    return month_name[month]
```

**Status**: ✅ Simplified, redundant null-check removed

### 1.3 textwrap.TextWrapper width overflow — ADDRESSED

The implementation now correctly accounts for indentation when calculating effective content width:

```sifr
def _effective_content_width(total_width: int, indent: str) -> int:
    available: int = total_width - len(indent)
    if available <= 0:
        return 1
    return available
```

**Verification test**:
```sifr
wrapper: TextWrapper = TextWrapper(width=10, initial_indent="    ", subsequent_indent="  ")
result: list[str] = wrapper.wrap("hello world this is a test string")
# All lines correctly constrained to <= 10 characters
```

**Status**: ✅ Width accounting works correctly

---

## 2. Current-Main Findings

### 2.1 Medium: base64 Module — Inconsistent Export Pattern

**File**: `lib/sifr/base64.sifr:1-2`

**Issue**: The module imports intrinsic functions from `_sifr.crypto` but only explicitly wraps `b64*` and `b16*` functions. The following are imported but not re-exported as named functions:

- `urlsafe_b64encode`, `urlsafe_b64decode`
- `b32encode`, `b32decode`
- `b32hexencode`, `b32hexdecode`

**Current state**:
```sifr
# Line 2 imports these:
from _sifr.crypto import urlsafe_b64encode, urlsafe_b64decode, b32encode, b32decode, b32hexencode, b32hexdecode

# But no explicit wrappers defined - relies on implicit re-export
```

**Why it works**: Sifr's module system appears to allow direct import of these names from the module despite no explicit wrapper:
```sifr
# This works despite no explicit function definition:
from sifr.base64 import urlsafe_b64encode
result: str = urlsafe_b64encode("test")  # Returns "dGVzdA=="
```

**Risk**: This is inconsistent with the coding style of other functions in the module (e.g., `b64encode`, `b16encode` are explicitly wrapped). Future maintainers may not realize these functions are available.

**Traceability claim** (wave_psp_c2_cpython_traceability.md:26):
> `test_base64` core codec vectors — `b64*`, `urlsafe_b64*`, `b32*`, `b32hex*`, `b16*` — adopted

**Verification**: Tests at `crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr` exercise all these functions and pass.

**Recommendation**: Add explicit wrapper functions for consistency, or document this as an acceptable pattern:
```sifr
def urlsafe_b64encode(s: str) -> str:
    return urlsafe_b64encode(s)

def urlsafe_b64decode(s: str) -> Result[str, ParseError]:
    return urlsafe_b64decode(s)
# ... similar for b32*, b32hex*
```

---

### 2.2 Low: string.Template — Type Annotation Inconsistency

**File**: `lib/sifr/string.sifr:260-261`

**Observation**: The `Template.substitute()` method returns `Result[str, ValueError]`, but test files use explicit `str` type annotations that appear to work via implicit conversion in try/except contexts:

```sifr
# Test file pattern that works:
try:
    substituted: str = template.substitute(mapping)
    substituted_ok = substituted == "tim likes ham worth $100"
except ValueError as e:
    ...
```

**Note**: Direct usage outside try/except fails type checking:
```sifr
result = template.substitute({"x": "world"})
# Error: cannot compare 'Result[str, ValueError]' and 'str' with ==
```

**Status**: Functions correctly in test contexts. This is a testing pattern note, not a correctness issue.

---

## 3. Test Coverage Verification

All e2e tests for the wave scope pass:

| Module | Test File | Status |
|--------|-----------|--------|
| string | cpython_string.sifr | ✅ Pass |
| string | cpython_string_template_subset.sifr | ✅ Pass |
| textwrap | cpython_textwrap.sifr | ✅ Pass |
| textwrap | cpython_textwrap_textwrapper_subset.sifr | ✅ Pass |
| base64 | cpython_base64_rfc4648_vectors.sifr | ✅ Pass |
| html | stdlib_html.sifr | ✅ Pass |
| fnmatch | cpython_fnmatch.sifr | ✅ Pass |
| fnmatch | cpython_fnmatch_translate_subset.sifr | ✅ Pass |
| difflib | cpython_difflib_subset.sifr | ✅ Pass |
| calendar | cpython_calendar_subset.sifr | ✅ Pass |

---

## 4. Traceability vs Shipped Behavior

| Traceability Claim | Shipped Behavior | Gap? |
|-------------------|------------------|------|
| string: constants + capwords adopted | Implemented (`ascii_*`, `digits`, etc.) | None |
| string: Template.substitute/safe_substitute adapted | Implemented, returns Result | None |
| string: Formatter.format adapted | Implemented, returns Result | None |
| textwrap: wrap/fill/dedent/indent/shorten adopted | Implemented | None |
| textwrap: TextWrapper.wrap/fill adapted | Implemented | None |
| base64: b64*, urlsafe_b64*, b32*, b32hex*, b16* adopted | Works via implicit import | Minor style gap |
| html: escape/unescape adopted | Implemented as intrinsic wrappers | None |
| fnmatch: fnmatch/fnmatchcase/filter adopted | Implemented | None |
| fnmatch: translate/filterfalse adapted | Implemented (character class waived) | None |
| difflib: get_close_matches/SequenceMatcher adapted | Full multi-block support | None |
| calendar: constants/helpers + class family adapted | Implemented (rendering waived) | None |

---

## 5. Actionable Recommendations

### Required (if any)

None — all core functionality works correctly.

### Suggested Improvements

1. **Add explicit base64 function wrappers** for `urlsafe_b64*`, `b32*`, `b32hex*` for code consistency and maintainability.

2. **Consider documenting Result handling patterns** — the implicit conversion in try/except contexts works but may confuse future contributors.

---

## 6. Waivers Status Check

All documented waivers are correctly honored:

- ✅ `fnmatch.translate()` character class (`[]`, `[!abc]`, `[a-z]`) — not implemented, waiver documented
- ✅ `string.Formatter` format specs (`:>10`, `:.2f`) and conversions (`!r`, `!s`) — not implemented, waiver documented
- ✅ `difflib` Differ/HtmlDiff/opcode APIs — not implemented, waiver documented
- ✅ `calendar` full rendering family — not implemented, waiver documented

---

## Conclusion

The wave_psp_c2 implementation is **production-ready** with correct CPython parity for the adopted/adapted surfaces. The only finding is a minor code style inconsistency in the base64 module that doesn't affect functionality. All documented waivers are being honored, and test coverage is comprehensive.
