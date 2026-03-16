# wave_psp_c2 CPython Traceability Matrix

Wave: `wave_psp_c2`  
Scope: `string`, `textwrap`, `base64`, `html`, `fnmatch`, `difflib`, `calendar`

## CPython Harvest Inputs

- `Lib/test/test_string/test_string.py`
- `Lib/test/test_string/test_templatelib.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_base64.py`
- `Lib/test/test_html.py`
- `Lib/test/test_fnmatch.py`
- `Lib/test/test_difflib.py`
- `Lib/test/test_calendar.py`

## Adopt / Adapt / Waive

| CPython family | Sifr surface | State | Local regression |
| --- | --- | --- | --- |
| `test_string` constants / `capwords` | `ascii_*`, `digits`, `hexdigits`, `octdigits`, `punctuation`, `whitespace`, `printable`, `capwords` | `adopted` | `crates/sifr/tests/e2e/pass/cpython_string.sifr` |
| `test_string` template behavior | `string.Template.substitute`, `safe_substitute` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_string_template_subset.sifr` |
| `test_string` formatter behavior | `string.Formatter.format` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_string_template_subset.sifr` |
| `test_textwrap` top-level wrappers | `wrap`, `fill`, `dedent`, `indent`, `shorten` | `adopted` | `crates/sifr/tests/e2e/pass/cpython_textwrap.sifr` |
| `test_textwrap` class model | `textwrap.TextWrapper.wrap`, `fill` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr` |
| `test_base64` core codec vectors | `b64*`, `urlsafe_b64*`, `b32*`, `b32hex*`, `b16*` | `adopted` | `crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr` |
| `test_html` escape/unescape | `html.escape`, `html.unescape` | `adopted` | `crates/sifr/tests/e2e/pass/stdlib_html.sifr` |
| `test_fnmatch` wildcard matching | `fnmatch`, `fnmatchcase`, `filter` | `adopted` | `crates/sifr/tests/e2e/pass/cpython_fnmatch.sifr` |
| `test_fnmatch` translate/filterfalse helpers | `translate`, `filterfalse` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_fnmatch_translate_subset.sifr` |
| `test_difflib` close-match + matcher object model | `get_close_matches`, `SequenceMatcher` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_difflib_subset.sifr`, `crates/sifr/tests/e2e/fail/phase_psp_c2_difflib_sequence_matcher_isjunk_unsupported.sifr` |
| `test_calendar` constants/helpers | weekday/leap helpers + name/abbr constants + class family | `adapted` | `crates/sifr/tests/e2e/pass/cpython_calendar_subset.sifr` |

## Explicit Waivers

- `string.Formatter` advanced CPython capabilities (`auto-numbering`, conversion specifiers, attribute/index lookup, format-spec mini-language) are waived for this wave and kept as `adapted` map-only formatting.
- `textwrap.TextWrapper` advanced options (`break_on_hyphens`, sentence-end fixing, tabsize/drop_whitespace variants) remain waived and tracked as `adapted`.
- `fnmatch` character-class and platform path-normalization semantics (`[]`, ranges, normcase behavior) remain waived and tracked as `adapted`.
- `difflib` advanced class families (`Differ`, `HtmlDiff`, full opcode/group APIs) remain waived and tracked as `adapted`.
- `difflib.SequenceMatcher` keeps a simplified constructor surface (`SequenceMatcher(a, b)` only) and does not expose CPython's `isjunk` / `autojunk` parameter matrix; this wave intentionally uses deterministic non-junk matching semantics and guards the unsupported call shape via `phase_psp_c2_difflib_sequence_matcher_isjunk_unsupported.sifr`.
- `calendar` full rendering family and locale/platform formatting behavior remain waived; this wave closes constants/helper and core class entry surfaces only.
