# Fuzz and Property Policy

This policy defines verification-hardening fuzz/property operating rules.

## Local Deterministic Smoke Gates (Blocking)

Canonical manifests:
- `verification/areas/fuzz_property/manifest.json`
- `verification/areas/fuzz_property/property_manifest.json`
- `verification/areas/fuzz_property/fuzz_smoke_manifest.json`

Canonical runner:
- `uv run --project verification --locked python -m sifr_verify areas run --area fuzz_property --suite property --suite fuzz-smoke`
- Merge profile runs the deterministic `fuzz-smoke` suite; `property` remains part of the broader local/nightly/release hardening command above.

Contracts:
- deterministic seeds and deterministic mutation stream
- reproducible local results (same inputs -> same outcome)
- no internal compiler panic signals in stderr/stdout
- machine-readable result artifacts emitted through `target/verification/areas/fuzz-property-results.json`
- mutation operators include import lines, string/numeric literals, and function signature shapes in addition to line-level edits
- `fuzz_smoke_manifest.json` carries the versioned target contract. The runner rejects missing or duplicate target ids, missing seed files, malformed reproduction/minimization commands, and unknown target references from `property_manifest.json`.
- The deterministic fuzz-smoke runner dispatches by target id. Source mutation targets run their own seed corpus, command, diagnostic format, exit-code policy, and uniqueness budget. Codegen smoke runs valid source seeds through the generated-binary path. Structured diagnostic and project/package targets execute their declared reproduction commands and enforce exit-code and panic-signal checks.
- Required first-party hardening target ids are:
  - `parse_check_entrypoint`
  - `hir_type_ownership_entrypoint`
  - `codegen_entrypoint`
  - `diagnostic_renderer_entrypoint`
  - `package_project_manifest_entrypoint`
- The parser-fork fuzzers under `third_party/ruff/fuzz` are separate inherited-parser signal. Sifr-original compiler fuzzing must use the target ids above and cannot claim coverage by pointing at the Ruff fork alone.

## Seed Corpus Rules

- Seed files are version-controlled under `verification/areas/fuzz_property/seeds/`.
- Target-specific seed ownership is recorded in `fuzz_smoke_manifest.json` under each target's `seed_files`.
- Seed updates require reviewable diffs and manifest updates.
- Duplicate equivalent seeds should be removed; dedup decisions are captured in PR notes.
- Seed corpus must cover control flow, import paths, callable signatures, and string/numeric literal shapes.

## Target Classes

- `valid-only` targets run programs expected to compile or execute successfully. They hunt wrong-code, codegen panics, and invariant drift.
- `invalid-only` targets run programs expected to be rejected. They hunt internal compiler errors, diagnostic-renderer crashes, and nondeterministic diagnostics.
- `mixed-valid-invalid` targets may produce either accepted or rejected Sifr source. They must bucket expected exits explicitly and must not treat user diagnostics as failures.
- `structured-diagnostics` targets consume `RenderedDiagnostic` JSON-shaped values or in-memory `RenderedDiagnostic` values produced by compiler APIs. They do not consume source text as their primary grammar, so they do not duplicate parser fuzzing.

### Diagnostic Renderer Grammar

The diagnostic renderer fuzz target operates on the rendered diagnostic envelope shape:

- `code`: active diagnostic code string using the registry's `SIFR-<family>-<number>` shape
- `severity`: one of `error`, `warning`, or `note`
- `message`: non-empty UTF-8 string
- `spans`: zero or more rendered spans with optional file, 1-based line/column, primary flag, and snippet lines
- `args`: JSON object containing strings, integers, booleans, arrays, objects, or null
- `children` and `suggestions`: optional arrays using the same bounded JSON value rules

The deterministic smoke path for this target is `cargo run --locked -q -p sifr_driver --bin diagnostic_contract_harness -- --target diagnostic_renderer_entrypoint --seed <fixture>`, which renders the named structured diagnostic through JSON, human, and compact renderers. Sustained fuzzing must mutate the structured envelope directly before rendering.

## Triage and Minimization Workflow

For every fuzz-found issue:
1. Reproduce with the exact seed/mutated source and random seed metadata.
2. Minimize to smallest stable reproducer.
3. Classify root cause and open/link issue.
4. Add sentinel coverage under `verification/areas/regression/fixtures/crashes/` if unresolved.
5. Promote through `verification/areas/regression/data/fixedbugs.json` after fix lands.

Checked-in minimization commands:
- `python3 verification/areas/fuzz_property/checks/minimize_seed.py --target <target-id> <failing-source>`
- `python3 verification/areas/fuzz_property/checks/minimize_diagnostic_json.py --target diagnostic_renderer_entrypoint <failing-rendered-diagnostic-json>`
- `python3 verification/areas/fuzz_property/checks/minimize_project_tree.py --target package_project_manifest_entrypoint <failing-project-dir>`

Fuzz reports must include the target id, seed or source hash, minimized candidate path, and exact reproduction command from the target contract.

## Sustained Fuzzing Signal (Non-blocking)

Long-running fuzzing is separate from local blocking smoke gates:
- definition: `verification/areas/fuzz_property/sustained_lane.md`
- status is signal-only and backlog-generating, not merge-blocking.
- nightly/release sustained lanes use the same target ids and promotion workflow as local smoke; broad coverage never bypasses minimization before becoming a merge-blocking regression.
