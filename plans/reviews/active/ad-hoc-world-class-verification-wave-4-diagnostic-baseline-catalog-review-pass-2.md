# Wave 4 Diagnostic Baseline Catalog Review Pass 2

Reviewer: agent (`--effort xhigh`)
Date: 2026-06-14
Scope: hardening edits after review pass 1.

## Blockers

None. The hardening edits are approved:

- Baseline expected-file construction now reads each manifest case command instead of assuming `check`.
- Recovery surface validation verifies expected diagnostic-code occurrences against diagnostics compact baselines or legacy e2e fixture expectation evidence.
- `suite_taxonomy.md` documents the synthetic baseline ownership exception used for metadata-owned presentation contract baselines.
- The pass-1 review artifact is present.

## Non-Blocking Notes

1. Some command-label parsing still assumes the current `check` command shape in narrow metadata paths. This is acceptable for the current manifest and should be tightened when a non-`check` diagnostics command is added.
2. `hir_mixed_recovery` still uses repeated `SIFR-TYPE-0002` diagnostics even though the taxonomy says mixed independent semantic errors. Carry this to the next Wave 4 slice.
3. The repeated type recovery validator does not yet assert a distinct summary diagnostic.
4. Legacy fixture recovery evidence counts code substrings in source text; current fixtures only mention codes in `expect-error` lines.

## Approval

Approved. Proceed to PR.
