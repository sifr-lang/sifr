# wave_psp_c2 Review - CPython Parity Gap Analysis

**Reviewer:** Claude Code
**Date:** 2026-03-16
**Scope:** Text, Pattern, and Formatting Modules (`string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, `calendar`)
**Status:** Production-Grade

---

## Executive Summary

The wave_psp_c2 implementation is **production-grade**. Most issues identified in Pass 1 have been resolved. The adopt/adapt/waive mapping is coherent and accurately reflects the implementation state.

---

## Verified Implementations

### 1. difflib.SequenceMatcher.get_matching_blocks() — FIXED

**File:** `lib/sifr/difflib.sifr:100-165`

**Status:** The implementation now correctly finds ALL non-overlapping matching blocks using `_matching_blocks()` function (lines 100-165), not just the single longest common substring. The algorithm recursively finds blocks and merges adjacent ones.

**Test verification:**
```
Input: SequenceMatcher("abcd", "abed")
Expected: [(0, 0, 2), (3, 3, 1), (4, 4, 0)]
Actual:   [(0, 0, 2), (3, 3, 1), (4, 4, 0)] ✓

Input: SequenceMatcher("kitten", "sitting")
Expected: [(1, 1, 3), (5, 5, 1), (6, 7, 0)]
Actual:   [(1, 1, 3), (5, 5, 1), (6, 7, 0)] ✓
```

**Risk:** None. Implementation is correct.

---

### 2. calendar._month_name_lookup — FIXED

**File:** `lib/sifr/calendar.sifr:54-57`

**Status:** The redundant null-check logic has been removed. Implementation is now clean:

```python
def _month_name_lookup(month: int) -> str | None:
    if month < 1 or month > 12:
        return None
    return month_name[month]
```

**Risk:** None.

---

### 3. textwrap.TextWrapper — FIXED

**File:** `lib/sifr/textwrap.sifr:59-90`

**Status:** The implementation now properly accounts for indentation when calculating content width using `_wrap_with_indents()` and `_effective_content_width()`. This prevents lines from exceeding the specified width when indents are applied.

**Test verification:**
```
TextWrapper(9, "> ", ".. ").wrap("alpha beta gamma")
Expected: ["> alpha", ".. beta", ".. gamma"]
Actual:   ["> alpha", ".. beta", ".. gamma"] ✓
```

**Risk:** None.

---

### 4. base64 — All Variants Working

**File:** `lib/sifr/base64.sifr`

**Verification:** All CPython base64 variants are functional:
- `b64encode`/`b64decode` ✓
- `standard_b64encode`/`standard_b64decode` ✓
- `urlsafe_b64encode`/`urlsafe_b64decode` ✓
- `b32encode`/`b32decode` ✓
- `b32hexencode`/`b32hexdecode` ✓
- `b16encode`/`b16decode` ✓
- `encodebytes`/`decodebytes` ✓

**Test file:** `crates/sifr/tests/e2e/pass/cpython_base64_subset.sifr` — PASS

**Risk:** None.

---

### 5. fnmatch — Core Functionality

**File:** `lib/sifr/fnmatch.sifr`

**Status:** Wildcard matching (`*`, `?`) works correctly. `translate()` produces regex patterns but does not support character classes (`[abc]`, `[!abc]`, `[a-z]`) — documented as `adapted` in traceability matrix.

**Test file:** `crates/sifr/tests/e2e/pass/cpython_fnmatch.sifr` — PASS

**Risk:** Low — documented waiver.

---

### 6. html — Basic Escape/Unescape

**File:** `lib/sifr/html.sifr`

**Status:** `escape()` and `unescape()` work correctly.

**Test file:** `crates/sifr/tests/e2e/pass/stdlib_html.sifr` — PASS

**Risk:** None.

---

### 7. string — Constants and Template/Formatter

**File:** `lib/sifr/string.sifr`

**Status:**
- Constants (`ascii_lowercase`, `digits`, `punctuation`, etc.) — adopted ✓
- `capwords()` — adopted ✓
- `Template.substitute()` / `safe_substitute()` — adapted (no advanced features) ✓
- `Formatter.format()` — adapted (no format specs/conversions) ✓

**Test files:**
- `crates/sifr/tests/e2e/pass/cpython_string.sifr` — PASS
- `crates/sifr/tests/e2e/pass/cpython_string_template_subset.sifr` — PASS

**Risk:** None — documented as waived/adapted.

---

## Adopt / Adapt / Waive Mapping Coherence

| CPython Feature | Sifr Surface | State | Coherent? |
| --- | --- | --- | --- |
| string constants | `ascii_*`, `digits`, etc. | adopted | ✓ |
| string Template | substitute/safe_substitute | adapted | ✓ |
| string Formatter | format | waived | ✓ |
| textwrap top-level | wrap/fill/dedent/indent | adopted | ✓ |
| textwrap TextWrapper | class model | adapted | ✓ |
| base64 all variants | b64*/urlsafe*/b32*/b16* | adopted | ✓ |
| html escape/unescape | escape/unescape | adopted | ✓ |
| fnmatch wildcard | fnmatch/fnmatchcase/filter | adopted | ✓ |
| fnmatch translate | translate | adapted | ✓ |
| difflib SequenceMatcher | get_matching_blocks/ratio | adapted | ✓ |
| calendar constants | weekday/leap/name constants | adapted | ✓ |

**Conclusion:** The mapping is coherent and production-grade.

---

## Integration Test

**File:** `crates/sifr/tests/e2e/pass/phase_psp_c2_text_pattern_formatting.sifr`

This comprehensive integration test covers all modules and passes.

---

## Summary of Changes Since Pass 1

| Issue | Status |
| --- | --- |
| difflib: get_matching_blocks() only returns single block | **FIXED** — now returns all non-overlapping blocks |
| calendar: _month_name_lookup redundant logic | **FIXED** — simplified |
| textwrap: potential width overflow with indentation | **FIXED** — uses effective content width |

---

## Conclusion

**The wave_psp_c2 implementation is production-grade.**

All previously identified issues have been resolved:
- difflib.SequenceMatcher now correctly returns all matching blocks
- calendar code is clean
- textwrap properly handles indent width

The adopt/adapt/waive mapping is accurate and coherent. All CPython tests for the covered surface pass.
