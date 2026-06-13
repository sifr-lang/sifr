# OSS Gate Policy

This policy defines the curated blocking OSS gate and broader non-blocking ecosystem lane.

## Curated OSS Gate (Blocking)

Manifest:
- `verification/areas/ecosystem_compatibility/data/curated_manifest.json`

Contract per entry:
- pinned revision
- owner
- rationale
- commands (`check|build|run|test` as applicable)
- timeout policy
- expected result classification

Pinned revision contract:
- format must be `local-main@<git-sha-prefix>`
- `<git-sha-prefix>` must match a commit in the followed history of tracked files under `project_root`
- path-only moves do not require repinning when the underlying fixture content is unchanged
- mismatches fail fast in the suite as `pinned_revision_mismatch`

Execution:
- suite name `oss-curated`
- runner: `uv run --project verification --locked python -m sifr_verify areas run --area ecosystem_compatibility --suite oss-curated`
- blocking: true

## Broader Ecosystem Lane (Non-blocking)

Manifest:
- `verification/areas/ecosystem_compatibility/data/ecosystem_broader_manifest.json`

Purpose:
- compatibility signal collection
- backlog generation and prioritization
- non-blocking execution with machine-readable output

Execution:
- suite name `ecosystem-broader`
- runner: `uv run --project verification --locked python -m sifr_verify areas run --area ecosystem_compatibility --suite ecosystem-broader`
- blocking: false

## Result Classification

Allowed classifications:
- `pass`
- `known-failure`
- `investigate`

Any mismatch between expected command outcomes and observed outcomes is recorded in structured artifacts.
Only `oss-curated` mismatches block merge-gate profiles.
