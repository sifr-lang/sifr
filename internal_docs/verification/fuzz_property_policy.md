# Fuzz and Property Policy

This policy defines phase-29 fuzz/property operating rules.

## Local Deterministic Smoke Gates (Blocking)

Canonical manifests:
- `verification/areas/fuzz_property/property_manifest.json`
- `verification/areas/fuzz_property/fuzz_smoke_manifest.json`

Canonical runner:
- `uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite fuzz-smoke`

Contracts:
- deterministic seeds and deterministic mutation stream
- reproducible local results (same inputs -> same outcome)
- no internal compiler panic signals in stderr/stdout
- machine-readable result artifacts emitted through `target/verification/hardening-results.json`
- mutation operators include import lines, string/numeric literals, and function signature shapes in addition to line-level edits

## Seed Corpus Rules

- Seed files are version-controlled under `verification/areas/fuzz_property/seeds/`.
- Seed updates require reviewable diffs and manifest updates.
- Duplicate equivalent seeds should be removed; dedup decisions are captured in PR notes.
- Seed corpus must cover control flow, import paths, callable signatures, and string/numeric literal shapes.

## Triage and Minimization Workflow

For every fuzz-found issue:
1. Reproduce with the exact seed/mutated source and random seed metadata.
2. Minimize to smallest stable reproducer.
3. Classify root cause and open/link issue.
4. Add sentinel to `crashes` if unresolved.
5. Promote to `fixedbugs` after fix lands.

## Sustained Fuzzing Lane (Non-blocking)

Long-running fuzzing is separate from local blocking smoke gates:
- definition: `verification/areas/fuzz_property/sustained_lane.md`
- status is signal-only and backlog-generating, not merge-blocking.
