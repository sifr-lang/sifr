# OSS Gate Policy

This policy defines the curated blocking OSS gate and broader non-blocking ecosystem lane.

## Curated OSS Gate (Blocking)

Manifest:
- `verification/oss/curated_manifest.json`

Contract per entry:
- pinned revision
- owner
- rationale
- commands (`check|build|run|test` as applicable)
- timeout policy
- expected result classification

Pinned revision contract:
- format must be `local-main@<git-sha-prefix>`
- `<git-sha-prefix>` must match the latest commit that touched `project_root`
- mismatches fail fast in the suite as `pinned_revision_mismatch`

Execution:
- suite name `oss-curated`
- runner: `scripts/run_verification_hardening.py`
- blocking: true

## Broader Ecosystem Lane (Non-blocking)

Manifest:
- `verification/oss/ecosystem_broader_manifest.json`

Purpose:
- compatibility signal collection
- backlog generation and prioritization
- non-blocking execution with machine-readable output

Execution:
- suite name `ecosystem-broader`
- runner: `scripts/run_verification_hardening.py`
- blocking: false

## Result Classification

Allowed classifications:
- `pass`
- `known-failure`
- `investigate`

Any mismatch between expected command outcomes and observed outcomes is recorded in structured artifacts.
Only `oss-curated` mismatches block merges in phase 29.
