# Ad Hoc Issue: Python Interop Read-Only Inspection Timeout

## Status

Active non-blocking follow-up discovered while running the authoritative
create-PR profile for stable publication governance on 2026-07-26. This is a
Python interop capability failure outside the release-governance change set.
It does not block Phase 40, and the Phase 40 work does not modify, suppress, or
reclassify the failing suite.

## Preserved Evidence

The create-PR profile passed coverage-matrix checks, core guardrails,
diagnostic rules, and CPython differential checks. Eighteen of nineteen
selected Python interop variants also passed. The only failure was
`readonly-check-doctor`:

- `target/validation_lane_reports/create-pr.latest.json` records the
  `python_interop` step as failed after 961,915 ms.
- `target/validation_lane_reports/create-pr.latest.log` records
  `target/debug/sifr python check --json` timing out after 120 seconds in the
  first read-only library inspection.
- An immediate isolated reproduction with
  `python3 verification/areas/python_interop/runner.py --suite
  readonly-check-doctor` failed at the same command and 120-second boundary.

No timeout or validation waiver was changed. The remaining Python interop
variants—including binding authoring, callbacks, dataframes, Arrow, DLPack,
buffers, async declarations, and CPython 3.11 runtime checks—passed in the
profile run.

## Scope

- Diagnose why the first read-only `sifr python check --json` does not
  terminate within the existing bound.
- Fix the root cause in the Python interop/compiler path without increasing the
  timeout or adding a fallback.
- Preserve the suite's file-system immutability, deterministic doctor output,
  trust-resolution, and invalid-target assertions.
- Keep remediation separate from stable-channel release-governance PRs.
- Name any associated demo after its capability; do not include a phase number
  or phase name.

## Acceptance Criteria

- [ ] The isolated `readonly-check-doctor` suite passes repeatedly within its
  existing command timeout.
- [ ] The complete Python interop area passes with every registered blocking
  variant.
- [ ] The authoritative create-PR profile passes without a timeout waiver or
  budget increase.
- [ ] Focused regression coverage proves the diagnosed root cause.
- [ ] Review rounds are satisfied and the remediation PR is merged.
