# Fuzz and Property Policy

This policy defines the compiler fuzzing and semantic-property rules.

## Blocking local evidence

The `property` suite runs deterministic source checks and Rust semantic-property tests. The Rust
properties cover union normalization, type narrowing, incremental/full query equivalence, and
deterministic code generation.

The `mutation-smoke` suite applies a fixed mutation stream to version-controlled seeds. It is not
coverage-guided fuzzing. The merge profile runs this suite as a blocking check.

Canonical inputs:

- `verification/areas/fuzz_property/property_manifest.json`
- `verification/areas/fuzz_property/mutation_smoke_manifest.json`
- `verification/areas/fuzz_property/seeds/`

Run the blocking evidence with:

```bash
uv run --project verification --locked python -m sifr_verify areas run \
  --area fuzz_property --suite property --suite mutation-smoke
```

The mutation manifest owns the deterministic seed, command, expected exit, timeout, uniqueness,
reproduction, and minimization rules for each target. The runner rejects missing targets, missing
seeds, duplicate identifiers, and invalid commands.

## Coverage-guided targets

The excluded Cargo fuzz project is `verification/fuzz/Cargo.toml`. It contains six libFuzzer
targets:

- `parser` parses bounded source bytes.
- `lowering` lowers bounded source bytes.
- `ownership` generates bounded ownership-heavy programs and lowers them.
- `codegen_validation` generates valid programs, compiles them, and validates generated Rust.
- `diagnostics` builds bounded structured diagnostics and uses all presentation renderers.
- `project_graph` builds bounded package metadata and derives a package graph.

The target contract and time budgets are in
`verification/areas/fuzz_property/sustained_fuzz_manifest.json`. Nightly and release profiles run
the non-blocking `sustained-fuzz` suite. The CI lane installs `cargo-fuzz` 0.13.2 and uses the
nightly Rust toolchain. Corpus and crash artifacts are written under `target/verification/fuzz/`.

To run one target directly:

```bash
cargo +nightly fuzz run --fuzz-dir verification/fuzz parser
```

Broad fuzz findings are signal only. They become blocking only after a contributor minimizes the
input and promotes it to the crash or fixed-bug regression area.

## Corpus and promotion rules

- Keep deterministic Sifr seeds under `verification/areas/fuzz_property/seeds/`.
- Record target ownership in the applicable manifest.
- Add a minimized finding before you remove duplicate corpus entries.
- Record the target, input hash, minimized path, and exact reproduction command.
- Do not use the inherited Ruff fuzzers as evidence for Sifr-owned compiler stages.

For each finding:

1. Reproduce it with the recorded target and input.
2. Minimize the input.
3. Classify the root cause and create or link an issue.
4. Add an unresolved crash to `verification/areas/regression/fixtures/crashes/`.
5. Add a resolved defect to `verification/areas/regression/data/fixedbugs.json`.

Checked-in minimizers:

- `python3 verification/areas/fuzz_property/checks/minimize_seed.py --target <target-id> <source>`
- `python3 verification/areas/fuzz_property/checks/minimize_diagnostic_json.py --target diagnostic_renderer_entrypoint <json>`
- `python3 verification/areas/fuzz_property/checks/minimize_project_tree.py --target package_project_manifest_entrypoint <project-dir>`
