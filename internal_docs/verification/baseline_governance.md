# Baseline Governance

This document defines canonical baseline governance for compiler-facing verification outputs.

## Canonical Bless and Verify Workflow

- Verify baselines:
  - `python3 scripts/run_verification_hardening.py --profile merge`
- Bless baselines:
  - `python3 scripts/run_verification_hardening.py --profile merge --bless`

Only explicit `--bless` updates checked-in baseline files.

## Baseline-Backed Artifacts in Scope

- Diagnostics renderer output (`human`, `json`, `compact`)
- Exit-code behavior
- Selected multi-file project behavior
- Machine-readable suite result summaries

## Normalization Rules

Baseline comparison and bless write-path use canonical normalization:

1. Path normalization
- Repository absolute root path is rewritten to `<WORKSPACE>`.
- Temporary directory prefixes (`/tmp/...`, `/private/tmp/...`, `/var/folders/...`) are rewritten to stable placeholders.

2. Text normalization
- Line endings normalized to `\n`.
- Trailing whitespace trimmed per line.

3. JSON normalization
- Output parsed and re-serialized with sorted keys and stable indentation.
- String values inside JSON undergo the same path normalization rules.

4. Nondeterministic ordering
- Canonical sort is applied where ordering is intentionally unordered.
- Unavoidable nondeterministic fields must be removed or normalized before baseline write.

## Review Contract

- Baseline diffs are first-class review artifacts.
- Incidental baseline updates are not allowed.
- Any baseline change must be justified by an intentional contract change.

## Suggestion/Autofix Boundary (Phase 29)

- Suggestions are part of the stable diagnostics schema.
- Suggestion rendering is baseline-validated through diagnostics JSON/human/compact baselines.
- Automated suggestion application is not currently part of the stable compiler contract in this phase.
- Autofix execution validation is therefore out-of-scope for phase 29 and explicitly deferred.
