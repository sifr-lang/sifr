# OSS Gate Policy

This policy defines the curated blocking OSS gate and broader non-blocking ecosystem suite.

## Curated OSS Gate (Blocking)

Manifest:
- `verification/areas/ecosystem_compatibility/data/curated_manifest.json`

Rules per entry:
- pinned revision
- source checksum (`source_checksum_sha256`)
- license (`license`, SPDX identifier)
- owner
- rationale
- commands (`check|build|run|test` as applicable)
- timeout policy
- expected result classification

Pinned revision rules:
- format must be `local-main@<git-sha-prefix>`
- `<git-sha-prefix>` must match a commit in the followed history of tracked files under `project_root`
- path-only moves do not require repinning when the underlying fixture content is unchanged
- mismatches fail fast in the suite as `pinned_revision_mismatch`

Source checksum rules:
- checksum algorithm is SHA-256
- the runner hashes every git-tracked file under `project_root` in sorted project-relative order
- each file contributes its path, a NUL separator, bytes, and a trailing NUL separator
- checksum mismatches fail as `source_checksum_mismatch` before commands execute
- any source file change requires a manifest checksum update in the same PR

License rules:
- each entry must carry an SPDX license identifier
- local first-party fixtures use `MIT`
- imported third-party corpora must preserve upstream license metadata and use the upstream SPDX identifier
- entries without license metadata are rejected before command execution

Execution:
- suite name `oss-curated`
- runner: `uv run --project verification --locked python -m sifr_verify areas run --area ecosystem_compatibility --suite oss-curated`
- blocking: true

## Broader Ecosystem Lane (Non-blocking)

Manifest:
- `verification/areas/ecosystem_compatibility/data/ecosystem_broader_manifest.json`

Purpose:
- compatibility signal collection
- signal queue generation and prioritization
- non-blocking execution with machine-readable output

Execution:
- suite name `ecosystem-broader`
- runner: `uv run --project verification --locked python -m sifr_verify areas run --area ecosystem_compatibility --suite ecosystem-broader`
- blocking: false

Limitations:
- see `verification/policy/ecosystem_limitations.md`

## Result Classification

Allowed classifications:
- `pass`
- `known-failure`
- `investigate`

Any mismatch between expected command outcomes and observed outcomes is recorded in structured artifacts.
Only `oss-curated` mismatches block merge-gate profiles.
