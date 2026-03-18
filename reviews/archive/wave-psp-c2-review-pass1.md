# wave_psp_c2 Review - Pass 1

**Reviewer:** External Reviewer
**Scope:** Text, Pattern, and Formatting Modules (`string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, `calendar`)
**Status:** Findings below

---

## Severity: Medium

### 1. difflib.SequenceMatcher.get_matching_blocks() returns only single block

**File:** `lib/sifr/difflib.sifr:113-119`

```python
def get_matching_blocks(self) -> list[tuple[int, int, int]]:
    blocks: list[tuple[int, int, int]] = []
    ai, bj, size = _longest_common_substring(self._a, self._b)
    if size > 0:
        blocks.append((ai, bj, size))
    blocks.append((len(self._a), len(self._b), 0))
    return blocks
```

**Issue:** The implementation finds only the **single longest common substring**, but CPython's `SequenceMatcher.get_matching_blocks()` returns **all non-overlapping matching blocks** in sequence order.

**CPython behavior:** For `"abcabc"` vs `"abcabc"`, CPython returns:
```
[(0, 0, 3), (3, 3, 3), (6, 6, 0)]
```

**Sifr behavior (current):** Returns only:
```
[(0, 0, 6), (6, 6, 0)]
```

**Why tests pass:** The test at `crates/sifr/tests/e2e/pass/cpython_difflib_subset.sifr:9-12` only exercises a simple case (`"abcd"` vs `"abed"`) with a single matching block, which happens to work.

**Risk:** Users relying on multi-block matching behavior will get incorrect results.

---

### 2. calendar._month_name_lookup has redundant/unclear logic

**File:** `lib/sifr/calendar.sifr:54-60`

```python
def _month_name_lookup(month: int) -> str | None:
    if month < 1 or month > 12:
        return None
    label: str | None = month_name[month]
    if label is not None:
        return label + ""
    return None
```

**Issue:** The check `if label is not None` is redundant since `month_name` is a static list of strings (indices 1-12 contain non-empty strings). The logic works but is confusing - it implies the list might contain `None` values, which it never does.

**Recommendation:** Simplify to:
```python
def _month_name_lookup(month: int) -> str | None:
    if month < 1 or month > 12:
        return None
    return month_name[month]
```

---

## Severity: Low (Documented Waivers)

### 3. fnmatch.translate() lacks character class support

**File:** `lib/sifr/fnmatch.sifr:62-95`

CPython's `fnmatch.translate()` supports character classes (`[abc]`, `[!abc]`, `[a-z]`) which are not implemented.

**Status:** Documented as `adapted` in `verification/stdlib/wave_psp_c2_cpython_traceability.md` - acceptable.

---

### 4. string.Formatter lacks format spec and conversion support

**File:** `lib/sifr/string.sifr:194-252`

CPython's `Formatter.format()` supports:
- Format specs: `{name:>10}`, `{value:.2f}`
- Conversions: `{name!r}`, `{name!s}`

These are not implemented.

**Status:** Documented as waived in traceability matrix - acceptable.

---

### 5. string.Template missing invalid placeholder validation for `$!`

**File:** `lib/sifr/string.sifr:41-192`

The test at `crates/sifr/tests/e2e/pass/cpython_string_template_subset.sifr:34-39` expects `$!` to raise an error:

```python
invalid_placeholder_ok: bool = False
try:
    _bad: str = Template("bad $! token").substitute({})
except ValueError as e:
    invalid_placeholder_ok = e.message.startswith("invalid template placeholder")
actual.append(invalid_placeholder_ok)
```

**Verification needed:** The code at lines 156-161 checks `_is_identifier_start(next_value)` but `$!` has `!` as next character. This should trigger the error path, but the test needs verification to confirm it works correctly.

---

### 6. textwrap.TextWrapper potential width overflow with indentation

**File:** `lib/sifr/textwrap.sifr:39-91`

When `initial_indent` or `subsequent_indent` are applied, the total line length can exceed `width`:

```python
def wrap(self, text: str) -> list[str]:
    if self.width <= 0:
        return []
    wrapped: list[str] = _wrap_impl(text, self.width)  # wraps to self.width
    # ... then indents are added, potentially exceeding width
    return _apply_line_indents(wrapped, self.initial_indent, self.subsequent_indent)
```

**Status:** Not documented as a known issue. CPython's TextWrapper may behave similarly - needs verification.

---

## Test Coverage Observations

### Positive

- Good coverage of error paths (missing values, invalid months, etc.)
- Tests verify both success and failure cases
- Multiple module integration test (`phase_psp_c2_text_pattern_formatting.sifr`) covers cross-module interactions

### Gaps

1. **difflib:** Missing test for multi-block matching scenarios
2. **textwrap:** No edge case tests for width/indent combinations
3. **Template:** Limited tests for edge cases like consecutive placeholders

---

## Residual Test Risk

- The `get_matching_blocks()` parity issue may cause subtle bugs in production code that relies on multi-block matching
- The TextWrapper width+indent interaction should be tested against CPython reference behavior
- No runtime performance testing included (noted as out of scope for this review)

---

**Conclusion:** The implementation is functional for the tested surface area but has a correctness defect in `difflib.SequenceMatcher.get_matching_blocks()` that could cause behavioral regressions in production. The other findings are either documented waivers or low-risk issues.
