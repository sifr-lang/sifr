# Structured Artifact Schema and Retention

Canonical artifact:
- `target/verification/hardening-results.json`

Producer:
- `uv run --project verification --locked python -m sifr_verify.hardening`

## Schema (high level)

- top-level execution metadata:
  - profile
  - shard index/total
  - rerun policy
  - quarantine metadata source/count
  - generation timestamp
- per-suite records:
  - suite identity and runner kind
  - blocking flag
  - case list with variant outcomes
  - failure totals
  - optional rerun attempts and flake events
- summary:
  - total variants
  - total failures
  - blocking failures
  - non-blocking failures

## Retention Policy

- latest run artifact is overwritten at the canonical path above.
- rerun and flake transitions are embedded directly in the suite records.
- failures may emit supporting local artifacts under `target/verification/actual/`.
- inspectable rules artifacts remain checked in under `verification/` (manifests, indices, policies, baselines).
