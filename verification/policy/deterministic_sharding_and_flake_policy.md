# Deterministic Sharding and Flake Policy

This policy defines deterministic scale checks and flake handling for verification hardening.

## Deterministic Sharding

Canonical runner arguments:
- `--shard-total <n>`
- `--shard-index <0..n-1>`

Selection rule:
- suite assignment is deterministic by stable hash of `suite_name`
- the same suite set and shard parameters always select the same subset

Sharding applies at suite granularity across baseline, corpus, property, mutation-smoke, sustained-fuzz, OSS, and determinism suites.

## Determinism Checks

Determinism-scale manifest:
- `verification/runner/sifr_verify/hardening/data/determinism_manifest.json`

Required checks:
- repeat-run report signature stability
- sequential vs parallel report-signature equivalence

## Rerun and Flake Tracking

Canonical runner argument:
- `--rerun-failures <n>` (default: `1`)

Policy:
- if a suite fails, rerun(s) are executed and recorded
- fail -> pass transitions on rerun are recorded as flake events
- flaky transitions are never silently treated as clean

## Quarantine Policy

Source of truth:
- `verification/policy/flake_quarantine.json`

Each quarantine entry (when present) must include:
- `suite`
- `case_id`
- `reason`
- `owner`
- `added_on`
- `reenable_criteria`

Quarantine is temporary and must be re-evaluated with explicit re-enable criteria.
`verification/policy/flake_quarantine.json` includes a concrete template entry to keep the format operational.
