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

## Explicit Waivers / Boundaries After Wave 3

- Decimal/Fraction/context-sensitive statistics semantics remain explicitly unsupported.
- Package-wide `html.parser` families remain explicitly unsupported.
- No residual `textwrap` formatter-option waiver remains for `fix_sentence_endings`, `max_lines`, or `placeholder`.

## Local Fixture Anchors (Wave 3)

- Positive fixtures:
  - `crates/sifr/tests/e2e/pass/cpython_statistics_subset.sifr`
  - `crates/sifr/tests/e2e/pass/cpython_textwrap_textwrapper_subset.sifr`
  - `crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr`
- Demo:
  - `demos/ad_hoc_rng_wave3_polish_waiver_reduction_demo.sifr`
- Negative fixture:
  - `crates/sifr/tests/e2e/fail/phase_psp_struct_0_html_package_parser_unsupported.sifr`
