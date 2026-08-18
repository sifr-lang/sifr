# stdlib_parity_rng_3 CPython Traceability Matrix

Capability: `stdlib_parity_rng_3`
Scope: final polish waiver reduction for `sifr.statistics`, residual `sifr.textwrap`, and residual `sifr.html`

## CPython Harvest Inputs

- `Lib/test/test_statistics.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_html.py`

## Adopt / Adapt / Waive (Capability 3)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_statistics` grouped-median deterministic surface | ship `median_grouped(data, interval)` with typed deterministic error boundaries for empty data and non-positive interval | `adapted` (shipped) | `stdlib/sifr/statistics.sifr`, `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr` |
| `test_statistics` `NormalDist` class family | keep class-oriented normal distribution surface out of scope for this implementation pass and enforce unsupported boundary | `unsupported` | `crates/sifr/tests/e2e/fail/statistics_normaldist_unsupported.sifr` |
| `test_textwrap` residual formatter options | close residual `TextWrapper` option waivers for `fix_sentence_endings`, `max_lines`, and `placeholder` without broad textwrap redesign | `adapted` (shipped) | `stdlib/sifr/textwrap.sifr`, `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`, `crates/sifr/tests/e2e/pass/textwrapper_options.sifr` |
| `test_html` top-level module boundary | retain top-level `html.escape`/`html.unescape` readiness and keep package-wide `html.parser` family explicitly unsupported | `adapted` (shipped boundary) | `stdlib/sifr/html.sifr`, `crates/sifr/tests/e2e/pass/stdlib_html.sifr`, `crates/sifr/tests/e2e/fail/html_package_parser_unsupported.sifr` |

## CPython `test_statistics.py` / `test_textwrap.py` / `test_html.py` Case Mapping (Capability 3)

| CPython test case | Sifr adaptation direction | Local anchor(s) | Coverage status |
| --- | --- | --- | --- |
| `TestMedianGrouped.test_interval_argument` and invalid-domain variants | `median_grouped(data, interval)` supports interval argument and typed rejection for empty/invalid interval domains | `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered |
| `TestMode` tie behavior (`first encountered mode`) | `mode()` keeps first-encountered tie behavior for multi-modal integer inputs | `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered |
| `WrapTestCase.test_fix_sentence_endings` (selected sentence-spacing cases) | `TextWrapper(fix_sentence_endings=True)` enforces sentence-spacing normalization on short lines and newline-normalized inputs | `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered |
| `HtmlTests.test_escape` | `html.escape` quote-default and `quote=False` surfaces match escaped-output expectations for top-level helper behavior | `crates/sifr/tests/e2e/pass/stdlib_html.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered |
| `HtmlTests.test_unescape` numeric references (`&#39;`, `&#x27;`/`&#X27;`, `&#60;`, `&#x3C;`/`&#X3C;`, `&#62;`, `&#x3E;`/`&#X3E;`) | `html.unescape` decodes named entities plus the shipped numeric-reference subset used by top-level helper tests, including lowercase/uppercase hex prefixes | `stdlib/sifr/html.sifr`, `crates/sifr_codegen/src/intrinsics/html.rs`, `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` | covered |

## Explicit Waivers / Boundaries After Capability 3

- Decimal/Fraction/context-sensitive statistics semantics remain explicitly unsupported.
- `statistics.NormalDist` class family remains explicitly unsupported.
- Package-wide `html.parser` families remain explicitly unsupported.
- No residual `textwrap` formatter-option waiver remains for `fix_sentence_endings`, `max_lines`, or `placeholder`.

## Local Fixture Anchors (Capability 3)

- Positive fixtures:
  - `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`
  - `crates/sifr/tests/e2e/pass/textwrapper_options.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_rng_additional_subset.sifr` (post-readiness statistics/textwrap/html adaptation subset)
- Demo:
  - `demos/text_and_statistics/main.sifr`
- Negative fixture:
  - `crates/sifr/tests/e2e/fail/statistics_normaldist_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/html_package_parser_unsupported.sifr`
