# Phase 31 Remediation Wave 1 Demo

## Goal

Demonstrate that the first compatibility remediation wave improves the canonical 50-case Phase 31 seed corpus without regressing the existing passing slice.

## Fixed Cases

- `0069_sqrtx`
- `0151_reverse_words_in_a_string`
- `2235_add_two_integers`

## Before/After Seed Metrics

- Baseline: `PASS=2`, `CHECK_ERROR=46`, `RUN_ERROR=2`
- After wave 1: `PASS=5`, `CHECK_ERROR=45`, `RUN_ERROR=0`

## Canonical Validation Commands

```bash
target/release/sifr run audit/leetcode/0069_sqrtx.sifr
target/release/sifr run audit/leetcode/0151_reverse_words_in_a_string.sifr
target/release/sifr run audit/leetcode/2235_add_two_integers.sifr
python3 scripts/run_phase31_leetcode.py \
  --manifest verification/leetcode/phase31_seed_corpus.json \
  --output verification/leetcode/phase31_seed_results_wave1.json
```

## Demo Outcome

- All three selected repros run end-to-end through the canonical release binary.
- The seed corpus gains three additional passes.
- Remaining failures stay in `CHECK_ERROR`; the prior run-stage bucket is eliminated from the seed corpus.
