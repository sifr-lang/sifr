# Wave 4 Diagnostic Baseline Catalog Review Pass 1

Reviewer: Claude Opus 4.7 (`--effort xhigh`)
Date: 2026-06-14
Scope: uncommitted Wave 4 diagnostics-baseline catalog slice.

## Blockers

None. The slice is internally consistent and the wiring matches the stated intent:

- `code_baseline_coverage.py` enforces the Wave 4 plan requirements: registry parity, catalog metadata, ownership, stale/missing files, source hash, and recovery coverage.
- Active `SIFR-PARSE-0002` through `SIFR-PARSE-0009` codes have non-deferred baseline entries with `human`, `json`, and `compact` renderers.
- `presentation_rules_cases` is correctly metadata-owned through synthetic baseline metadata and stays out of the executable manifest.
- Every remaining active code carries an owner, reason, issue, and `expires_in_wave` deferral.
- The recovery surface catalog lists parser, HIR mixed recovery, and repeated type recovery fixtures.
- The phase plan correctly frames this as the first diagnostics-baseline slice, not final Wave 4 closeout.

## Non-Blocking Notes

1. `hir_mixed_recovery` currently reuses three `SIFR-TYPE-0002` instances; a later Wave 4 slice can use a fixture with mixed semantic diagnostic kinds or relax the taxonomy wording.
2. The recovery validator can be hardened by checking that expected diagnostic codes are represented in fixture evidence.
3. `parser_invalid_layout` could also be cross-referenced from parser recovery in a later slice.
4. The taxonomy should mention the synthetic baseline exception used for presentation contract cases.
5. Baseline path construction should read the manifest command rather than assuming `check`.

## Approval

Approved. Proceed to PR after local validation.
