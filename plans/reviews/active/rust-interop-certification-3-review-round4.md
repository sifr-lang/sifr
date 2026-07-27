# Rust Interop `certification_3` Review — Round 4

Reviewer: Claude Opus 5 (`medium`)

Result: **UNSATISFIED**

## Confirmed round-3 resolutions

Opus confirmed decorator-order-independent effective abort aggregation, ambient
release-profile abort rejection, and source-derived `OwnMutable` preservation
and rejection. It also confirmed that every other convention consumer treats
`OwnMutable` as owned and that inventory arithmetic remains correct against the
intended opaque-future baseline.

## Remaining findings

1. **Low:** selected Cargo panic strategy was filesystem-read and TOML-parsed
   for every interop declaration, even when no call-scoped callback existed.
2. **Low:** `OwnMutable` emitted a message naming mutable-borrow rather than the
   source `mut callback` spelling.

## Required disposition before round 5

- Gate abort-profile discovery to packages that actually own call-scoped
  callback targets and evaluate it once per package.
- Give `OwnMutable` a source-specific remove-`mut` diagnostic and pin that text
  in the real source-level test.
