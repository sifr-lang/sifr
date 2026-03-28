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
| `test_textwrap` class model | `textwrap.TextWrapper.wrap`, `fill`, and adjacent option fields (`expand_tabs`, `tabsize`, `replace_whitespace`, `drop_whitespace`, `break_on_hyphens`) | `adapted` | `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`<br>`crates/sifr/tests/e2e/pass/text_wrapping_and_html.sifr` |
| `test_base64` core codec vectors | `b64*`, `urlsafe_b64*`, `b32*`, `b32hex*`, `b16*` | `adopted` | `crates/sifr/tests/e2e/pass/cpython_base64_rfc4648_vectors.sifr` |
| `test_html` escape/unescape | `html.escape`, `html.unescape` (including `escape(..., quote=False)` parity) | `adopted` | `crates/sifr/tests/e2e/pass/stdlib_html.sifr`<br>`crates/sifr/tests/e2e/pass/text_wrapping_and_html.sifr` |
| `test_fnmatch` wildcard matching | `fnmatch`, `fnmatchcase`, `filter` | `adopted` | `crates/sifr/tests/e2e/pass/cpython_fnmatch.sifr` |
| `test_fnmatch` translate/filterfalse helpers | `translate`, `filterfalse` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_fnmatch_translate_subset.sifr` |
| `test_difflib` close-match + matcher object model | `get_close_matches`, `SequenceMatcher` | `adapted` | `crates/sifr/tests/e2e/pass/cpython_difflib_subset.sifr`, `crates/sifr/tests/e2e/fail/difflib_sequence_matcher_isjunk_unsupported.sifr` |
| `test_calendar` constants/helpers | weekday/leap helpers + name/abbr constants + class family | `adapted` | `crates/sifr/tests/e2e/pass/cpython_calendar_subset.sifr` |

## Explicit Waivers

- `string.Formatter` advanced CPython capabilities (`auto-numbering`, conversion specifiers, attribute/index lookup, format-spec mini-language) are waived for this wave and kept as `adapted` map-only formatting.
- Historical note: residual `textwrap.TextWrapper` formatter ecosystem options (`fix_sentence_endings`, `max_lines`, `placeholder`) were carried from this wave and are closed by `wave_psp_rng_3`.
- `fnmatch` character-class and platform path-normalization semantics (`[]`, ranges, normcase behavior) remain waived and tracked as `adapted`.
- `difflib` advanced class families (`Differ`, `HtmlDiff`, full opcode/group APIs) remain waived and tracked as `adapted`.
- `difflib.SequenceMatcher` keeps a simplified constructor surface (`SequenceMatcher(a, b)` only) and does not expose CPython's `isjunk` / `autojunk` parameter matrix; this wave intentionally uses deterministic non-junk matching semantics and guards the unsupported call shape via `difflib_sequence_matcher_isjunk_unsupported.sifr`.
- `calendar` full rendering family and locale/platform formatting behavior remain waived; this wave closes constants/helper and core class entry surfaces only.

## Structured/Class-Surface Continuation Closure (2026-03-18)

- Continuation phase: `issues/ad-hoc-structured-data-and-class-surface-parity-expansion.md`
- Wave ownership: `wave_psp_struct_4` closed adjacent `textwrap` and top-level `html` polish surfaces (completed).
- Closed in continuation:
  - `TextWrapper` adjacent option fields (`expand_tabs`, `tabsize`, `replace_whitespace`, `drop_whitespace`, `break_on_hyphens`) under bounded deterministic behavior.
  - `html.escape(s, quote: bool = True)` top-level polish with explicit `quote=False` behavior.
  - continuation fixture: `crates/sifr/tests/e2e/pass/text_wrapping_and_html.sifr`
- Locked permanent diffs carried into continuation:
  - package-wide `html` expansion (`html.parser` families) remains `unsupported`,
  - broader formatter ecosystem redesign remains out of scope.
- Successor follow-up owner:
  - `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md` (`wave_psp_rng_3`) for residual-waiver triage/closure without broad parser redesign (completed for `textwrap` formatter options; `html.parser` family remains unsupported).
- Enforcement fixture: `crates/sifr/tests/e2e/fail/html_package_parser_unsupported.sifr`
