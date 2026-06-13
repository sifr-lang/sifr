# Post-Closure CPython Adaptation Review Pass 1: Ad Hoc Stateful RNG, Crypto, and Polish Parity Expansion

**Phase**: `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
**Execution ledger**: `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion-execution.md`
**Review type**: post-closure CPython adaptation pass — missing/unadapted test gap analysis
**Date**: 2026-03-21
**Reviewer**: Claude Code
**Commit under review**: `f2df08d6` (post-closure CPython adaptation coverage added)
**Phase status**: Production-grade closed; post-closure CPython adaptation pass completed

---

## Executive Summary

The post-closure CPython adaptation pass added `cpython_rng_phase_additional_subset.sifr` covering state-boundary validation, hashlib vectors/case/copy, statistics grouped interval, textwrap sentence cases, and html escape coverage. This is a meaningful increase in CPython traceability depth.

However, a systematic gap analysis against all six upstream CPython test families (`test_random`, `test_hashlib`, `test_base64`, `test_statistics`, `test_textwrap`, `test_html`) reveals **missing coverage in three categories**:

1. **Missing feature**: `statistics.NormalDist` was explicitly in-scope per the phase document (line 128) but was never shipped or formally waived. This is a **scope compliance gap**.
2. **Missing CPython test adaptations**: A large number of CPython test functions for all six modules have no Sifr fixture coverage. Most are lower-priority (variance tests, edge-case distributions), but several represent meaningful surface gaps.
3. **Known waivers correctly enforced**: `choices(weights=...)`, SHA3/SHAKE, `html.parser`, `SystemRandom` state, and `Decimal`/`Fraction` statistics are correctly unimplemented with negative fixtures.

**Verdict**: Three actionable findings — one scope compliance gap (HIGH), one statistical parity gap (MEDIUM), one test coverage gap (LOW). No regressions. No correctness issues in shipped code.

---

## 1. Scope Compliance Finding

### Finding 1: `statistics.NormalDist` Was In-Scope But Never Shipped or Formally Waived

**Severity**: HIGH
**Module**: `statistics`
**File**: `lib/sifr/statistics.sifr` (335 lines — no `NormalDist` class)
**Reference**: Phase planning doc, line 128: *"`NormalDist` is in scope only if it can remain float-only and deterministic."*
**Reference**: Architecture lock doc, `phase_psp_rng_architecture_lock.md` — not mentioned
**Reference**: Execution ledger, `wave_psp_rng_3` scope — not listed
**Reference**: `wave_psp_rng_3_cpython_traceability.md` — not listed
**Reference**: `milestone_psp_7_parity_governance_inventory.md` — `statistics` listed as `parity-closed` with `wave_psp_e1 + wave_psp_rng_3` attribution, but `NormalDist` is absent from the module's shipped surface

**Issue**: The phase planning document (line 128) explicitly lists `NormalDist` as in-scope, conditioned on "float-only and deterministic". The `wave_psp_rng_3` scope and the `wave_psp_rng_3_cpython_traceability.md` do not mention it. The implementation (`lib/sifr/statistics.sifr`) has no `NormalDist` class. The governance inventory marks `statistics` as `parity-closed` without flagging `NormalDist` as a gap. There is no negative fixture asserting `NormalDist` is unsupported.

**CPython upstream**: `test_statistics.py` contains a full `NormalDist` test class with ~20 test methods (`test_instantiation_and_attributes`, `test_sample_generation`, `test_pdf`, `test_cdf`, `test_inv_cdf`, `test_quantiles`, `test_overlap`, `test_zscore`, `test_properties`, `test_translation_and_scaling`, `test_unary_operations`, `test_equality`, `test_copy`, `test_pickle`, `test_hashability`, `test_repr`, `test_slots`).

**Implication**: Either (a) `NormalDist` should be added as a shipped feature (requires implementation), or (b) it should be formally waived with a negative fixture and governance documentation. The current state — neither shipped nor waived — is a scope compliance gap.

**Actionable**: Decide whether `NormalDist` ships or is formally waived. If waived, add:
- Negative fixture: `phase_psp_rng_3_normaldist_unsupported.sifr` asserting compile-time or runtime rejection
- Waiver entry in `wave_psp_rng_3_cpython_traceability.md`
- Update to `milestone_psp_7_parity_governance_inventory.md` waiver index
- Update phase planning doc scope section

---

## 2. Statistical Surface Gaps

### Finding 2: `statistics` Advanced Type Support Not Covered

**Severity**: MEDIUM
**Module**: `statistics`
**Files**: `lib/sifr/statistics.sifr`, `crates/sifr/tests/e2e/pass/cpython_statistics.sifr`

**Issue**: CPython's `statistics` module accepts `Decimal`, `Fraction`, and mixed `int`/`float` inputs for most functions. Sifr's `statistics.sifr` uses `list[float]` as the input type. All CPython test functions that pass `Decimal`, `Fraction`, or mixed types are not adapted:

| CPython test function | Sifr coverage | Status |
|---|---|---|
| `TestFmean.test_fraction` | NOT covered | Gap |
| `TestFmean.test_decimal` | NOT covered | Gap |
| `TestFmean.test_float` | Covered (`cpython_statistics.sifr`) | OK |
| `TestHarmonicMean.test_fraction` | NOT covered | Gap |
| `TestHarmonicMean.test_decimal` | NOT covered | Gap |
| `TestGeometricMean.test_fraction` | NOT covered | Gap |
| `TestGeometricMean.test_decimal` | NOT covered | Gap |
| `TestMedian.test_fraction` | NOT covered | Gap |
| `TestMedian.test_decimal` | NOT covered | Gap |
| `TestMedianLow.test_fraction` | NOT covered | Gap |
| `TestMedianHigh.test_fraction` | NOT covered | Gap |
| `TestMedianGrouped.test_fraction` | NOT covered | Gap |
| `TestMode.test_fraction` | NOT covered | Gap |
| `TestMode.test_decimal` | NOT covered | Gap |
| `TestStdevVariance.test_fraction` | NOT covered | Gap |
| `TestStdevVariance.test_decimal` | NOT covered | Gap |
| `TestCorrelation.test_fraction` | NOT covered | Gap |
| `TestCorrelation.test_decimal` | NOT covered | Gap |
| `TestLinearRegression.test_fraction` | NOT covered | Gap |
| `TestLinearRegression.test_decimal` | NOT covered | Gap |
| `TestQuantiles.test_fraction` | NOT covered | Gap |
| `TestQuantiles.test_decimal` | NOT covered | Gap |
| `TestNormalDist` (entire class) | NOT covered | Scope gap (Finding 1) |
| `TestKDE` (entire class) | NOT covered | Intentionally unsupported (kernel density estimation) |
| `TestMultimode.test_counter_data` | NOT covered | Gap (uses `collections.Counter`) |

**Assessment**: This is a known and documented `intentional-diff`. The phase planning doc (line 127-128) states: *"Only close narrow advanced surfaces that do not require decimal, fraction, or context-sensitive semantics."* The milestone inventory waiver index correctly classifies `Decimal/Fraction/context-sensitive statistics semantics` as `unsupported`.

**Actionable**: No implementation action required. This gap is intentional and documented. However, the `Decimal`/`Fraction` gap should be more prominently surfaced in the `wave_psp_rng_3_cpython_traceability.md` — the current doc does not explicitly enumerate which CPython `test_statistics` functions are covered vs. waived. Consider adding a column to the case mapping table: "covered / waived / not applicable."

---

## 3. CPython Test Function Coverage — Per-Module Gap Analysis

### 3.1 `test_random` — CPython has ~75 test functions across 3 test classes

**Coverage in Sifr** (from `cpython_random_subset.sifr`, `phase_psp_rng_1_stateful_random_object_model.sifr`, `cpython_rng_phase_additional_subset.sifr`):

Covered:
- `test_saverestore` (deterministic replay)
- `test_setstate_first_arg` / `test_setstate_middle_arg` (invalid-domain boundaries)
- `test_randbytes` (Mersenne Twister)
- `test_randrange_nonunit_step`, `test_randint` (partial)
- `test_choice`
- `test_sample` (partial)
- `test_shuffle` (partial)
- `test_gauss` (partial — only that it returns a float)
- `test_getrandbits` (partial)
- `test_random` (partial — only that it returns [0,1))

**Missing coverage** (by category):

| Missing test | Category | Severity | Notes |
|---|---|---|---|
| `test_choices` | Uniform distribution | MEDIUM | Full `choices` algorithm test. Sifr ships `choices` but only tests uniform k-select. No statistical distribution test for `choices`. |
| `test_choices_subnormal` | Edge cases | LOW | `choices` with subnormal floats as weights |
| `test_choices_with_all_zero_weights` | Edge cases | LOW | Zero-weight edge case |
| `test_choices_negative_total` | Error handling | LOW | `choices` with negative weights |
| `test_choices_infinite_total` | Error handling | LOW | `choices` with infinite-weight edge case |
| `test_choices_algorithms` | Algorithm correctness | MEDIUM | Tests `choices` algorithm correctness for uniform distribution |
| `test_sample_distribution` | Distribution properties | MEDIUM | Tests uniform distribution of `sample` |
| `test_sample_inputs` | Input types | MEDIUM | Tests `sample` with various iterable types |
| `test_sample_on_dicts` | Input types | MEDIUM | Dict input to `sample` |
| `test_sample_on_sets` | Input types | MEDIUM | Set input to `sample` |
| `test_sample_on_seqsets` | Input types | LOW | Sequence of sets |
| `test_sample_with_counts` | Population sampling | LOW | Population sampling with counts |
| `test_sample_counts_equivalence` | Algorithm correctness | MEDIUM | `sample` with counts equivalence |
| `test_gauss` | Statistical properties | MEDIUM | Tests Gaussian distribution properties, cached value reuse |
| `test_53_bits_per_float` | Precision | LOW | Float precision verification |
| `test_randbelow_logic` | Algorithm | LOW | Internal `randbelow` algorithm |
| `test_randrange_index` | Algorithm | LOW | Internal `randrange` index |
| `test_randrange_uses_getrandbits` | Algorithm | LOW | Algorithm implementation detail |
| `test_randbelow_without_getrandbits` | Algorithm | LOW | Fallback algorithm |
| `test_randrange_step` | Edge cases | LOW | `randrange` with various step values |
| `test_rangelimits` | Edge cases | LOW | Range limit boundary values |
| `test_randrange_bug_1590891` | Regression | LOW | Historical bug regression test |
| `test_bigrand` | Large values | LOW | Large random number generation |
| `test_bigrand_ranges` | Large values | LOW | Large range values |
| `test_autoseed` | Seed behavior | MEDIUM | Autoseeding behavior |
| `test_seedargs` | Seed behavior | MEDIUM | Various seed argument types |
| `test_seed_no_mutate_bug_44018` | Regression | LOW | Historical bug |
| `test_long_seed` | Seed behavior | LOW | Long seed inputs |
| `test_pickling` | Serialization | LOW | Pickle round-trip |
| `test_bug_1727780` | Regression | LOW | Historical bug |
| `test_bug_9025` | Regression | LOW | Historical bug |
| `test_mu_sigma_default_args` | Defaults | LOW | Default `gauss(mu, sigma)` arguments |
| `test_random_subclass_with_kwargs` | Subclassing | LOW | Random subclass with kwargs |
| `test_subclasses_overriding_methods` | Subclassing | LOW | Method override behavior |
| `test_zeroinputs` | SystemRandom edge cases | LOW | Zero-input edge cases |
| `test_avg_std` | SystemRandom statistical | LOW | SystemRandom statistical properties |
| `test_constant` | SystemRandom edge cases | LOW | Constant value edge case |
| `test_binomialvariate` | Unshipped distribution | N/A | Feature not in scope |
| `test_von_mises_range` | Unshipped distribution | N/A | Feature not in scope |
| `test_gammavariate_*` | Unshipped distribution | N/A | Feature not in scope |
| `test_choice_with_numpy` | numpy integration | N/A | External dependency |

**Assessment**: The `choices` coverage gap is the most significant. `test_choices` and `test_choices_algorithms` test the full uniform distribution algorithm for `choices`. Sifr's `choices` is shipped but has no statistical distribution verification. This matters because `choices` is a public API that could silently produce biased output without detection.

The `test_gauss` gap is also meaningful — Sifr only verifies `gauss` returns a float, not that it produces a reasonable Gaussian distribution or correctly caches the second value of the Box-Muller pair.

**Actionable**: Consider adding `choices` distribution tests (at minimum `test_choices` and `test_choices_algorithms`). Consider adding `gauss` cached-value and distribution property tests.

---

### 3.2 `test_hashlib` — CPython has ~60 test functions

**Coverage in Sifr** (from `cpython_hashlib_api_subset.sifr`, `cpython_hashlib_object_model_subset.sifr`, `cpython_rng_phase_additional_subset.sifr`, `phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr`):

Covered:
- `test_algorithms_guaranteed`
- `test_algorithms_available`
- `test_unknown_hash`
- `test_new_upper_to_lower`
- `test_copy`
- `test_hexdigest`
- `test_name_attribute`
- `test_blocksize_and_name` (partial — blake2 covered, sha3 not)
- `test_blocksize_name_blake2`
- `test_case_md5_0/1/2`
- `test_case_sha1_0/1/2/3`
- `test_case_sha224_0/1/2/3`
- `test_case_sha256_0/1/2/3`
- `test_case_sha384_0/1/2/3`
- `test_case_sha512_0/1/2/3`
- `test_blake2b` / `test_blake2s` / `test_case_blake2b_0/1/2` / `test_case_blake2s_0/1/2`
- `test_blake2b_all_parameters` / `test_blake2s_all_parameters` / `test_blake2b_vectors` / `test_blake2s_vectors`
- SHA3/SHAKE rejection: `test_case_sha3_224_0`, `test_case_sha3_256_0`, `test_case_sha3_384_0`, `test_case_sha3_512_0`, `test_shakes_zero_digest_length`, `test_shakes_invalid_digest_length`, `test_shakes_overflow_digest_length` — all correctly fail
- `test_file_digest` (partial)
- `test_extra_sha3` (partially — SHA3 algorithms correctly rejected)
- `test_no_unicode`, `test_no_unicode_blake2`, `test_no_unicode_sha3` — all correctly fail

**Missing coverage**:

| Missing test | Category | Severity | Notes |
|---|---|---|---|
| `test_large_update` | Algorithm | MEDIUM | Tests incremental `update` on large data (multi-block). Important for correctness of incremental API. |
| `test_sha256_update_over_4gb` | Performance/algorithm | LOW | 4GB update — definitely out of scope but worth noting |
| `test_sha3_update_over_4gb` | Performance | N/A | SHA3 not shipped — correctly unsupported |
| `test_blake2_update_over_4gb` | Performance/algorithm | LOW | Large blake2 update |
| `test_get_builtin_constructor` | API surface | MEDIUM | `hashlib.sha256()` shorthand (different from `hashlib.new('sha256')`). Not covered in Sifr. |
| `test_gil` / `test_sha256_gil` / `test_threaded_hashing_fast` / `test_threaded_hashing_slow` | Threading | N/A | Threading — host-limited territory |
| `test_get_fips_mode` | FIPS mode | N/A | Platform-specific |
| `test_disallow_instantiation` / `test_hash_disallow_instantiation` / `test_readonly_types` | Security | LOW | Security hardening tests |
| `test_pbkdf2_hmac_c` | KDF | MEDIUM | `pbkdf2_hmac` — not shipped in Sifr, should have negative fixture |
| `test_normalized_name` | API surface | LOW | `name` attribute normalization |
| `test_scrypt` / `test_scrypt_types` / `test_scrypt_validate` | KDF | MEDIUM | `scrypt` — not shipped, should have negative fixture |
| `test_usedforsecurity_true` / `test_usedforsecurity_false` | Security classification | LOW | Platform-specific flag |
| `test_clinic_signature` / `test_clinic_signature_errors` | API metadata | LOW | Argument signature metadata |
| `test_sha256_gil` | Threading | N/A | Host-limited |
| `test_sha3_256_update_over_4gb` | Performance | N/A | SHA3 not shipped |
| `test_blake2_update_over_4gb` | Performance | LOW | Large data |
| `test_sha2_vectors` / `test_sha3_vectors` | Algorithm vectors | N/A | Covered by individual case tests |

**Assessment**: Most significant gaps are the missing KDF functions. `pbkdf2_hmac` and `scrypt` are standard `hashlib` features in CPython that Sifr does not ship. The current governance inventory does not explicitly list them as waived. The `test_get_builtin_constructor` gap means `hashlib.sha256()` (vs `hashlib.new('sha256')`) is not covered, though the shorthand constructor may not be part of Sifr's API contract.

**Actionable**:
1. Add negative fixtures for `pbkdf2_hmac` and `scrypt` (or confirm they are out of scope and add to waiver index)
2. Add `test_large_update` coverage for incremental hash updates (tests that `update()` correctly processes data in chunks)

---

### 3.3 `test_base64` — CPython has ~55 test functions

**Coverage in Sifr** (from `cpython_base64_subset.sifr`, `cpython_base64_rfc4648_vectors.sifr`, `cpython_base64_strictness_subset.sifr`, `phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr`, `phase_psp_rng_2_base64_invalid_bytes_decode_boundary.sifr`):

Covered:
- `test_b64encode` / `test_b64encode_wrapcol`
- `test_b64decode` / `test_b64decode_padding_error` / `test_b64decode_invalid_chars` / `test_b64decode_altchars`
- `test_standard_b64_encode_decode_round_trip`
- `test_urlsafe_b64_encode_decode_round_trip`
- `test_b32encode` / `test_b32decode`
- `test_b32hexencode` / `test_b32hexdecode`
- `test_b32_encode_decode_round_trip`
- `test_b16encode` / `test_b16decode`
- `test_b16_encode_decode_round_trip`
- `test_encodebytes` / `test_decodebytes`
- `test_ErrorHeritage`
- `test_RFC4648_test_cases`
- `test_bytes_encode_decode_round_trip` (partial)
- `test_legacy_encode_decode_round_trip` (partial)
- `test_encode` / `test_decode` (partial)
- `test_encode_file` (partial — via `file_digest` indirect coverage)
- `test_encode_from_stdin` / `test_prints_usage_with_help_flag` / `test_prints_usage_with_invalid_flag` — N/A (CLI only)
- `test_decode_nonascii_str` — partial

**Missing coverage**:

| Missing test | Category | Severity | Notes |
|---|---|---|---|
| `test_a85encode` / `test_a85encode_wrapcol` | ASCII85 codec | LOW | Not shipped — should be in waiver index |
| `test_b85encode` / `test_z85encode` | Base85/Z85 codecs | LOW | Not shipped — should be in waiver index |
| `test_a85decode` / `test_a85decode_errors` | ASCII85 codec | LOW | Not shipped |
| `test_b85decode` / `test_z85decode` | Base85/Z85 codecs | LOW | Not shipped |
| `test_a85_padding` / `test_b85_padding` / `test_z85_padding` | Padding edge cases | LOW | Not shipped |
| `test_a85_encode_decode_round_trip` | Round-trip | LOW | Not shipped |
| `test_b85_encode_decode_round_trip` | Round-trip | LOW | Not shipped |
| `test_decode_nonascii_str` | Encoding edge case | MEDIUM | Tests decode of non-ASCII string input |
| `test_b32decode_casefold` | Case-insensitive decode | MEDIUM | `b32decode` with casefold=True. Sifr's `b32decode` may not support casefold. |
| `test_b32decode_map01` | Base32 variant | LOW | `map01` parameter for 0↔O, 1↔I mapping |
| `test_b32_encode_decode_round_trip` with casefold/map01 | Round-trip variants | LOW | Complex parameter combinations |
| `test_b32hexdecode_other_types` | Type variants | LOW | Hex-decoding other input types |
| `test_encode_file` | File encoding | MEDIUM | `b64encode` from file path (separate from `file_digest`) |
| `test_encode_from_stdin` | CLI | N/A | CLI mode — not applicable |

**Assessment**: The ASCII85/Base85/Z85 codecs are not shipped and should be added to the waiver index. The `test_b32decode_casefold` gap is more meaningful — it tests case-insensitive Base32 decoding. The current Sifr `b32decode` may or may not support this variant; if it does, the test is missing; if it doesn't, the test should be a negative fixture.

**Actionable**:
1. Add ASCII85/Base85/Z85 to the `base64` waiver index
2. Verify `b32decode` casefold support and add test or negative fixture

---

### 3.4 `test_textwrap` — CPython has ~75 test functions across multiple test classes

**Coverage in Sifr** (from `cpython_textwrap.sifr`, `cpython_textwrap_textwrapper_subset.sifr`, `cpython_textwrap_subset.sifr`, `phase_psp_rng_3_textwrap_formatter_options.sifr`):

Covered:
- `test_simple` (top-level)
- `test_spaces`
- `test_whitespace`
- `test_fix_sentence_endings`
- `test_wrap_short` / `test_wrap_short_1line`
- `test_hyphenated` / `test_hyphenated_numbers`
- `test_drop_whitespace_false` / `test_drop_whitespace_false_whitespace_only` / `test_drop_whitespace_false_whitespace_only_with_indent`
- `test_drop_whitespace_whitespace_only` / `test_drop_whitespace_leading_whitespace` / `test_drop_whitespace_whitespace_line` / `test_drop_whitespace_whitespace_only_with_indent` / `test_drop_whitespace_whitespace_indent`
- `test_break_on_hyphens` / `test_break_long_words_on_hyphen` / `test_break_long_words_not_on_hyphen` / `test_break_on_hyphen_but_not_long_words` / `test_do_not_break_long_words_or_on_hyphens`
- `test_placeholder` / `test_placeholder_backtrack`
- `test_max_lines_long` / `test_max_lines_long`
- `test_fill`
- `test_initial_indent` / `test_subsequent_indent`
- `test_bad_width`
- `test_type_error`
- `test_dedent_*` (6 cases)
- `test_indent_*` (6 cases)
- `test_shorten` / `test_empty_string` / `test_width_too_small_for_placeholder` / `test_first_word_too_long_but_placeholder_fits`
- `test_placeholder_backtrack` (phase 3)
- `test_empty_string_with_initial_indent`
- `test_whitespace`
- `test_break_long`
- `test_nobreak_long`

**Missing coverage**:

| Missing test | Category | Severity | Notes |
|---|---|---|---|
| `test_em_dash` | Hyphenation | LOW | Em-dash handling in hyphenated text |
| `test_unix_options` | Hyphenation | LOW | Unix-style option strings |
| `test_funky_hyphens` | Hyphenation | LOW | Various hyphen characters |
| `test_punct_hyphens` | Hyphenation | LOW | Punctuated hyphens |
| `test_funky_parens` | Hyphenation | LOW | Parenthetical hyphenation |
| `test_no_split_at_umlaut` | Character handling | LOW | German umlaut handling |
| `test_umlaut_followed_by_dash` | Character handling | LOW | Umlaut-dash interaction |
| `test_non_breaking_space` | Whitespace handling | MEDIUM | Non-breaking space character behavior |
| `test_narrow_non_breaking_space` | Whitespace handling | LOW | Narrow NBSP behavior |
| `test_split` | Algorithm | LOW | Splitting algorithm behavior |
| `test_roundtrip_spaces` / `test_roundtrip_tabs` / `test_roundtrip_mixed` | Round-trip | LOW | Indent/dedent round-trip |
| `test_indent_no_lines` | Edge case | LOW | Empty indent |

**Assessment**: Textwrap coverage is strong. The most notable gap is `test_non_breaking_space` / `test_narrow_non_breaking_space` — these test how non-breaking spaces interact with the text wrapping algorithm. This could expose edge cases in whitespace normalization.

**Actionable**: Consider adding `test_non_breaking_space` for NBSP whitespace handling.

---

### 3.5 `test_html` — CPython has 2 test functions

**Coverage in Sifr** (from `stdlib_html.sifr`, `cpython_rng_phase_additional_subset.sifr`):

Covered:
- `test_escape` (partial — standard escape; quote handling)
- `test_unescape` (partial — basic unescape)

**Missing coverage**:

| Missing test | Category | Severity | Notes |
|---|---|---|---|
| `test_unescape` (full) | Character references | MEDIUM | Full unescape with numeric character references (`&#39;`, `&#x27;`, `&#60;`, etc.), named entities, and mixed content. Current Sifr coverage tests basic string round-trip but not numeric reference decoding. |

**Reference**: `cpython_rng_phase_additional_subset.sifr` line 243 tests `unescape("no character references")` and `unescape(escape(sample))`. This covers the round-trip but not explicit numeric character references.

CPython's full `test_unescape` tests:
- `&#x27;` → `'`
- `&#39;` → `'`
- `&#60;` → `'<'`
- `&#x3C;` → `'<'`
- `&amp;` → `'&'`
- `&lt;` → `'<'`
- Mixed entities and numeric references

**Assessment**: The `unescape` coverage is incomplete. The `html_unescape` intrinsic uses a replace-chain approach. Need to verify whether numeric character references (`&#...;`) are handled.

**Actionable**: Add explicit numeric character reference tests for `unescape` — `&#39;`, `&#x27;`, `&#60;`, `&#x3C;`. If these are not handled, either add support or add a negative fixture noting the limitation.

---

## 4. Findings Summary

### HIGH — Must Fix

| # | Finding | Module | File | Description |
|---|---------|--------|------|-------------|
| 1 | `NormalDist` in-scope but never shipped or waived | `statistics` | `lib/sifr/statistics.sifr` | Phase planning doc (line 128) lists `NormalDist` as in-scope with float-only/deterministic condition. Implementation has no `NormalDist` class. Governance docs do not mention it. Neither shipped nor formally waived — scope compliance gap. |

### MEDIUM — Should Fix

| # | Finding | Module | File | Description |
|---|---------|--------|------|-------------|
| 2 | `choices` uniform distribution not tested | `random` | `cpython_random_subset.sifr` | `test_choices` and `test_choices_algorithms` from CPython are not adapted. `choices` is shipped but has no statistical distribution verification. |
| 3 | `test_large_update` for hashlib not covered | `hashlib` | (none) | Incremental hash `update()` correctness on multi-block data is not verified. |
| 4 | `pbkdf2_hmac` and `scrypt` have no negative fixture | `hashlib` | (none) | Standard CPython `hashlib` features not shipped, but no negative fixture or waiver index entry. |
| 5 | `test_unescape` numeric character reference coverage incomplete | `html` | `cpython_rng_phase_additional_subset.sifr` | `unescape("&#39;")` → `"'"` and similar numeric references not explicitly tested. |
| 6 | `b32decode` casefold variant not tested | `base64` | (none) | Case-insensitive `b32decode` with casefold parameter may not be supported or tested. |

### LOW — Nice to Have

| # | Finding | Module | File | Description |
|---|---------|--------|------|-------------|
| 7 | ASCII85/Base85/Z85 codecs not in waiver index | `base64` | (none) | `a85encode`, `b85encode`, `z85encode` and decode variants are not shipped but not listed in waiver index. |
| 8 | `test_non_breaking_space` for textwrap not covered | `textwrap` | (none) | Non-breaking space character behavior in wrapping algorithm not tested. |
| 9 | `gauss` cached-value and distribution properties not tested | `random` | `cpython_random_subset.sifr` | Only tests that `gauss` returns a float; does not verify cached value reuse or distribution shape. |
| 10 | `test_b32decode_map01` not covered | `base64` | (none) | Base32 `map01` parameter (0↔O, 1↔I variant) not tested. |
| 11 | `test_get_builtin_constructor` for hashlib not covered | `hashlib` | `cpython_hashlib_api_subset.sifr` | `hashlib.sha256()` shorthand vs `hashlib.new('sha256')` not distinguished in test. |
| 12 | `test_decode_nonascii_str` for base64 not covered | `base64` | (none) | Decode of non-ASCII string input edge case not tested. |

---

## 5. Known Waivers (Correctly Enforced)

The following are correctly unimplemented with appropriate negative fixtures:

| Waiver | Type | Negative fixture | Status |
|--------|------|-----------------|--------|
| `choices(weights=...)` | `unsupported` | `phase_psp_b2_random_choices_weights_unsupported.sifr` | Correct |
| `SystemRandom.getstate`/`setstate` | `unsupported` | `phase_psp_rng_1_system_random_state_unsupported.sifr` | Correct |
| SHA3/SHAKE hashlib families | `unsupported` | `phase_psp_rng_2_sha3_object_model_unsupported.sifr` | Correct |
| Package-wide `html.parser` ecosystem | `unsupported` | `phase_psp_struct_0_html_package_parser_unsupported.sifr` | Correct |
| Decimal/Fraction statistics | `unsupported` | Implied by float/int-only `median_grouped` | Correct but under-documented |
| `statistics.NormalDist` | **Gap** | **NONE** | **Finding 1** |
| `pbkdf2_hmac` / `scrypt` | **Gap** | **NONE** | **Finding 4** |
| ASCII85/Base85/Z85 | **Gap** | **NONE** | **Finding 7** |
| `b32decode` casefold | Unknown | **NONE** | **Finding 6** |
| Numeric character refs in `unescape` | Unknown | **NONE** | **Finding 5** |

---

## 6. Recommendations

### Must-Fix

1. **[HIGH]** Resolve `NormalDist` scope ambiguity: either implement it or formally waive it with negative fixture and governance documentation.
2. **[MEDIUM]** Add `test_choices` and `test_choices_algorithms` CPython adaptations for uniform distribution verification of `choices`.
3. **[MEDIUM]** Add `test_large_update` for hashlib incremental update correctness.
4. **[MEDIUM]** Add negative fixtures for `pbkdf2_hmac` and `scrypt`, or update the waiver index to explicitly list them.

### Should-Fix

5. **[MEDIUM]** Add explicit numeric character reference tests for `html.unescape` (`&#39;`, `&#x27;`, `&#60;`, `&#x3C;`). Verify behavior or add missing support.
6. **[MEDIUM]** Clarify `b32decode` casefold support and add test or negative fixture.

### Nice-to-Have

7. **[LOW]** Add ASCII85/Base85/Z85 to the `base64` waiver index.
8. **[LOW]** Add `test_non_breaking_space` for textwrap NBSP handling.
9. **[LOW]** Add `gauss` cached-value test (verifies second Box-Muller value is cached and returned on next call).
10. **[LOW]** Add `test_b32decode_map01` for Base32 variant.
11. **[LOW]** Consider adding `test_get_builtin_constructor` distinction for hashlib.
12. **[LOW]** Add `test_decode_nonascii_str` for base64.

---

## 7. Verification Commands

All post-closure fixtures pass at `f2df08d6`:

```bash
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_rng_phase_additional_subset.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_1_stateful_random_object_model.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_rng_3_textwrap_formatter_options.sifr
```

Full gate: `scripts/run_all_tests.sh --profile quick` — PASS at `f2df08d6`.

---

*Review completed: 2026-03-21*
*Reviewer: Claude Code*
*Phase: ad-hoc-stateful-rng-crypto-and-polish-parity-expansion*
*Review type: post-closure-cpython-review-pass-1*
*Commit: f2df08d6*
