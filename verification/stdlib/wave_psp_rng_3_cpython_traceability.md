# wave_psp_rng_3 CPython Traceability Matrix

Wave: `wave_psp_rng_3`  
Scope: final polish waiver reduction for `sifr.statistics`, residual `sifr.textwrap`, and residual `sifr.html`

## CPython Harvest Inputs

- `Lib/test/test_statistics.py`
- `Lib/test/test_textwrap.py`
- `Lib/test/test_html.py`

## Adopt / Adapt / Waive (Wave 3)

| CPython family | Sifr surface direction | State | Local anchor |
| --- | --- | --- | --- |
| `test_statistics` grouped-median deterministic surface | ship `median_grouped(data, interval)` with typed deterministic error boundaries for empty data and non-positive interval | `adapted` (shipped) | `lib/sifr/statistics.sifr`, `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr` |
| `test_textwrap` residual formatter options | close residual `TextWrapper` option waivers for `fix_sentence_endings`, `max_lines`, and `placeholder` without broad textwrap redesign | `adapted` (shipped) | `lib/sifr/textwrap.sifr`, `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`, `crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr` |
| `test_html` top-level module boundary | retain top-level `html.escape`/`html.unescape` closure and keep package-wide `html.parser` family explicitly unsupported | `adapted` (shipped boundary) | `lib/sifr/html.sifr`, `crates/sifr/tests/e2e/pass/stdlib_html.sifr`, `crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr` |

## CPython `test_statistics.py` / `test_textwrap.py` / `test_html.py` Case Mapping (Wave 3)

| CPython test case | Sifr adaptation direction | Local anchor(s) | Coverage status |
| --- | --- | --- | --- |
| `TestMedianGrouped.test_interval_argument` and invalid-domain variants | `median_grouped(data, interval)` supports interval argument and typed rejection for empty/invalid interval domains | `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_phase_additional_subset.sifr` | covered |
| `TestMode` tie behavior (`first encountered mode`) | `mode()` keeps first-encountered tie behavior for multi-modal integer inputs | `crates/sifr/tests/e2e/pass/cpython_rng_phase_additional_subset.sifr` | covered |
| `WrapTestCase.test_fix_sentence_endings` (selected sentence-spacing cases) | `TextWrapper(fix_sentence_endings=True)` enforces sentence-spacing normalization on short lines and newline-normalized inputs | `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_phase_additional_subset.sifr` | covered |
| `HtmlTests.test_escape` | `html.escape` quote-default and `quote=False` surfaces match escaped-output expectations for top-level helper behavior | `crates/sifr/tests/e2e/pass/stdlib_html.sifr`, `crates/sifr/tests/e2e/pass/cpython_rng_phase_additional_subset.sifr` | covered |

## Explicit Waivers / Boundaries After Wave 3

- Decimal/Fraction/context-sensitive statistics semantics remain explicitly unsupported.
- Package-wide `html.parser` families remain explicitly unsupported.
- No residual `textwrap` formatter-option waiver remains for `fix_sentence_endings`, `max_lines`, or `placeholder`.

## Local Fixture Anchors (Wave 3)

- Positive fixtures:
  - `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`
  - `crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_rng_phase_additional_subset.sifr` (post-closure statistics/textwrap/html adaptation subset)
- Demo:
  - `demos/ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr`
- Negative fixture:
  - `crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr`
