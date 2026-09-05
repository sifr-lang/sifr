# Review: milestone_diag_10 slice 5 - recovery deduplication, pass 1

Reviewer: agent
Date: 2026-05-03
Branch: `codex/diag-10-recovery-dedupe`

## Verdict

Reviewer-satisfied.

No bugs or regressions found.

## Findings

- `RecoveryDedupeKey` correctly uses diagnostic code, message template, registry-declared dedupe args, and primary source range via file plus byte start/end.
- Dedupe arg serialization uses `DiagnosticArg::canonical_json_bytes`, avoiding ad hoc rendered-message parsing.
- Registered diagnostics use only `entry.dedupe_args`; non-dedupe args no longer affect recovery deduplication.
- Deduplication runs before the existing similar-diagnostic grouping and top-level cap, keeping exact duplicate suppression separate from cap grouping.
- Tests cover exact duplicate suppression, distinct byte ranges remaining distinct, registry dedupe args excluding non-dedupe args, existing similar cap behavior, existing top-level cap behavior, and CLI canonical stream parity.

## Residual Risk

None identified.
