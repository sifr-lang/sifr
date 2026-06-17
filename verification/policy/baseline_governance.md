# Baseline Governance

This document defines canonical baseline governance for compiler-facing verification outputs.

## Canonical Bless and Verify Workflow

- Verify baselines:
  - `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines`
  - `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite rules`
  - `cargo test -p sifr_codegen`
- Bless baselines:
  - `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines --bless`
  - `cargo insta review -p sifr_codegen` for intentional codegen snapshot changes

Only explicit `--bless` updates checked-in baseline files.

## Baseline-Backed Artifacts in Scope

- Diagnostics renderer output (`human`, `json`, `compact`) owned by
  `verification/areas/diagnostics/manifest.json`
- Exit-code behavior
- Selected multi-file project behavior
- Machine-readable suite result summaries
- `sifr_codegen` insta snapshots and generated-Rust assertion baselines owned by
  `crates/sifr_codegen/src/**`

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

## Review Rules

- Baseline diffs are first-class review artifacts.
- Incidental baseline updates are not allowed.
- Any baseline change must be justified by an intentional rules change.
- Diagnostics baseline changes must update `baseline_metadata.json`, preserve
  source hashes, and pass `diagnostics:rules`; stale, unused, incomplete, or
  metadata-less baseline files are blocking failures.
- `sifr_codegen` snapshot blesses must be reviewed with the corresponding generated-Rust
  rules change, and `cargo test -p sifr_codegen` must pass before the bless is accepted.

## Suggestion/Autofix Boundary (Verification Hardening)

- Suggestions are part of the stable diagnostics schema.
- Suggestion rendering is baseline-validated through diagnostics JSON/human/compact baselines.
- Automated suggestion application is not currently part of the stable compiler rules in the current stable compiler rules.
- Autofix execution validation is therefore out-of-scope for verification hardening and explicitly deferred.
