# Verification Suite Taxonomy

This document defines the canonical suite taxonomy for compiler verification hardening.

## Canonical Suite Kinds

| suite | purpose | blocking | owner | required artifacts |
| --- | --- | --- | --- | --- |
| `diagnostics` | Lock compiler diagnostics contract (code/message/severity/url/spans/renderer views/exit code). | yes | compiler/diagnostics | fixtures, baselines, machine-readable run summary |
| `project_workspace` | Lock multi-file/module project behavior and deterministic project-mode outcomes. | yes | compiler/frontend | fixtures, baselines, machine-readable run summary |
| `fixedbugs` | Permanent issue-linked regressions for resolved compiler bugs. | yes | compiler/hardening | regression index, issue/root-cause metadata, run summary |
| `crashes` | Visible sentinel corpus for unresolved crashes/invariant failures. | yes | compiler/hardening | sentinel index, issue linkage, promotion log |
| `property` | Invariant tests for deterministic/high-value compiler behaviors. | yes | compiler/hardening | property manifest, deterministic run summary |
| `fuzz-smoke` | Deterministic local fuzz smoke over curated corpora. | yes | compiler/hardening | seed corpus manifest, deterministic run summary |
| `oss-curated` | Small pinned curated real-world gate that blocks merges. | yes | compiler/verification | corpus manifest, per-project outcomes, run summary |
| `ecosystem-broader` | Larger non-blocking compatibility lane for signal and backlog generation. | no | compiler/verification | lane manifest, signal report |

## Fixture and Baseline Conventions

### Diagnostics
- Fixture root: `verification/areas/diagnostics/fixtures/diagnostics/<case_id>/main.sifr`.
- Required baseline files:
  - `baselines/check-human.stdout.txt`
  - `baselines/check-human.stderr.txt`
  - `baselines/check-human.exit-code.txt`
  - `baselines/check-json.stdout.txt`
  - `baselines/check-json.stderr.txt`
  - `baselines/check-json.exit-code.txt`
  - `baselines/check-compact.stdout.txt`
  - `baselines/check-compact.stderr.txt`
  - `baselines/check-compact.exit-code.txt`
- Canonical expected representation:
  - Human and compact renderer output baselines are text files.
  - JSON diagnostics are canonicalized and pretty-printed before baseline compare.
  - Exit code is baseline-checked separately.

### Project Workspace
- Fixture root: `verification/areas/project_workspace/fixtures/project/<case_id>/`.
- Entrypoint: `main.sifr`.
- Required baseline files per configured command variant:
  - `<variant>.stdout.txt`
  - `<variant>.stderr.txt`
  - `<variant>.exit-code.txt`
- Project fixtures are multi-file and use import/module behavior representative of real project use.

### Fixedbugs
- Fixture root: fixed bug locks usually point at the canonical e2e/demo fixture that reproduces the resolved bug.
- Index source of truth: `verification/areas/regression/data/fixedbugs.json`.
- Required metadata:
  - issue or finding id
  - root-cause category
  - owning suite location
  - concise context note (when needed)

### Crashes
- Sentinel root: `verification/areas/regression/fixtures/crashes/<case_id>.sifr`.
- Index source of truth: `verification/areas/regression/data/crashes.json`.
- Required metadata:
  - issue id
  - crash or sentinel classification
  - current status (`unresolved` or `promoted`)
  - minimized `reproducer_fixture` path
  - promotion target once fixed

### Property and Fuzz-Smoke
- Policy and manifests live under `verification/areas/fuzz_property/`.
- Seeds and deterministic generation rules are version-controlled.
- Each run emits machine-readable case outcomes and seed provenance.
- Operational policy: `internal_docs/verification/fuzz_property_policy.md`.

### OSS-Curated and Ecosystem-Broader
- Manifests live under `verification/oss/`.
- `oss-curated` entries must include pinned revision, owner, rationale, commands, timeout, and expected class.
- `ecosystem-broader` is explicitly non-blocking and emits signal-only reports.
- Operational policy: `internal_docs/verification/oss_gate_policy.md`.

## Manifest Source of Truth

- Canonical legacy manifest for unmigrated hardening suites:
  `verification/suites/manifest.json`.
- Migrated diagnostics and project workspace suite ownership live in
  `verification/areas/diagnostics/manifest.json` and
  `verification/areas/project_workspace/manifest.json`.
- Suite runners must not hardcode fixture lists outside their owning manifest.
- Manifest updates are review artifacts and follow normal PR review.
- Corpus lifecycle and promotion rules: `internal_docs/verification/regression_corpus_policy.md`.
- Deterministic sharding + flake policy: `internal_docs/verification/deterministic_sharding_and_flake_policy.md`.
- Structured artifact schema + retention: `internal_docs/verification/artifact_schema_and_retention.md`.
