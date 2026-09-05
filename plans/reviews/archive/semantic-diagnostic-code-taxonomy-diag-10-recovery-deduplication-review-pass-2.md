# Review: milestone_diag_10 slice 5 - recovery deduplication, pass 2

Reviewer: agent
Date: 2026-05-03
Branch: `codex/diag-10-recovery-dedupe`

## Verdict

Reviewer-satisfied.

No bugs or regressions found after the reveal-type recovery-cap regression was fixed.

## Findings

- Exact deduplication now uses code, message template, registry dedupe args, and primary span file/byte range, preserving first emission for each unique recovery key.
- Similar-cap grouping remains separate and uses severity, code, message template, registry dedupe args, and primary file.
- Including registry dedupe args in similar-cap grouping prevents distinct `reveal_type(...)` notes with different `revealed_type` args from being collapsed before the top-level cap.
- The top-level cap behavior is covered by both the driver recovery test for 60 distinct reveal types and the CLI `check_entrypoint` reveal-type cap regression test.
- `DiagnosticArg::canonical_json_bytes` is the correct identity source for dedupe args.

## Residual Risk

None identified.
