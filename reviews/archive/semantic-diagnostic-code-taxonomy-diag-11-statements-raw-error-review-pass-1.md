# Review: semantic-diagnostic-code-taxonomy-diag-11-statements-raw-error-review-pass-1

Claude could not write this artifact directly and returned the review through stdout.

## Findings

### Critical: `yield_without_value.sifr` expects wrong diagnostic code

Reviewer reported that `crates/sifr/tests/e2e/fail/yield_without_value.sifr` expected `SIFR-FLOW-0007` instead of `SIFR-FLOW-0006`.

Codex verification after the review showed the current workspace already contains:

```sifr
# expect-error[col=5]: SIFR-FLOW-0006
```

No code change was required for this finding.

### Low: Registry representative fixture path for `SIFR-FLOW-0007` inconsistent

Reviewer reported that `SIFR-FLOW-0007` pointed at `for_loop_invalid_iterable.sifr`.

Codex verification after the review showed the current workspace already points `SIFR-FLOW-0007` at:

```text
crates/sifr/tests/e2e/fail/invalid_assignment_target_attribute_base.sifr
```

No code change was required for this finding.

## Satisfied Items

- Raw `ctx.error(String)` transport was eliminated from `crates/sifr_hir/src/lower/statements.rs`.
- Structured helpers now cover statement forms, assignment targets, iteration diagnostics, match-pattern forms, Result try/except diagnostics, and uninitialized variables.
- Primary ranges are explicit.
- Docs, focused tests, and transport cleanup guardrails were present for the reviewed slice.

## Follow-up

Codex self-review found and fixed an additional non-name `except` type fallback after this review: explicit non-name except type expressions now emit `SIFR-RESULT-0006` instead of being treated as catch-all arms. A fresh review pass is required for the updated implementation.
