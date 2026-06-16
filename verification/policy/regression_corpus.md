# Regression Corpus Policy (`fixedbugs` + `crashes`)

This policy defines the canonical regression corpus contract for verification hardening.

## `fixedbugs` Contract

Source of truth:
- `verification/areas/regression/data/fixedbugs.json`

Each fixedbug entry must include:
- `id` (stable corpus id)
- `issue` (issue/finding/PR identifier)
- `root_cause_category`
- `suite_location` (fixture path)
- `command` (`check|run|build|test`)
- `expect_exit_code`
- `note` (short context when name alone is insufficient)

Execution contract:
- Every fixedbug entry is executed by `uv run --project verification --locked python -m sifr_verify.hardening`.
- Exit-code contract is enforced for each entry.
- Missing metadata fields fail the hardening gate.

## `crashes` Sentinel Contract

Source of truth:
- `verification/areas/regression/data/crashes.json`

Each crash sentinel entry must include:
- `id`
- `issue`
- `owner`
- `status` (`unresolved|promoted`)
- `root_cause_category`
- `source_reference` (must exist)
- `reproducer_fixture` (must exist and remain minimized)
- `promotion_target_suite` (currently `fixedbugs`)
- `note`

Execution contract:
- Crash sentinels are machine-validated by `uv run --project verification --locked python -m sifr_verify.hardening`.
- Invalid metadata or missing `source_reference`/`reproducer_fixture` paths fail the hardening gate.
- Unresolved sentinels remain visible and blocking until resolved or explicitly promoted.

## Promotion Rule (`crashes` -> `fixedbugs`)

When a sentinel is fixed:
1. Add/confirm minimized reproducible regression in `fixedbugs`.
2. Link the fixedbug entry to the issue/finding.
3. Mark the crash entry `status=promoted` and point to the fixedbug id.
4. Keep issue linkage and root-cause continuity intact.

Promotion is complete only when the new fixedbug regression passes under the canonical verification gate.
