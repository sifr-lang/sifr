# Rust Interop Verification Area

This area tracks Rust interop verification.

The canonical design is `internal_docs/rust_interop_architecture.md`. This
area is the compatibility inventory for Rust interop: it names every fixture
required by the architecture, records tier assignment, reserves diagnostic
families, and publishes the compatibility matrix used by docs and reviewers.

The fixture matrix is contract-first. Every fixture must declare both positive
and negative evidence, and every fixture directory must contain concrete source
files for those evidence IDs. A compatibility row can use `supported`,
`supported-through-bridge`, or `unsupported-by-design` only when both evidence
directions are `passing`. Rows that are still `planned` or otherwise incomplete
must be categorized as `future-owned-by-separate-phase`.

## Fixture Layout

Each directory under `fixtures/` contains:

- `README.md`: explanatory evidence notes that repeat canonical test names for
  readers. README prose is not validator input.
- `fixture.json`: schema-v2 machine-readable fixture metadata mirroring the
  fixture matrix row. Every passing evidence side binds one exact Rust test
  through `validation.profile`, `step`, `suite_id`, `test_file`, and
  `test_name`; non-passing evidence has no validation binding.
- `positive/<positive_evidence.id>.sifr`: the positive evidence source.
- `negative/<negative_evidence.id>.sifr`: the negative evidence source.
- `examples/<crate>.sifr`: one full package example for each crate listed in
  the fixture row's `required_crates`.
- `examples/<scenario>/`: one full scenario package for fixture families whose
  evidence depends on package or workspace layout rather than a registry crate
  list.

The `matrix` suite validates this layout, verifies that fixture metadata
matches `data/rust_interop_fixture_matrix.json`, and rejects passing claims
backed only by README prose. It resolves every structured test binding to the
weakest blocking profile that executes it, checks suite/package/file
ownership, and reserves each Rust test for one evidence side. Positive and
negative evidence files must include a
`verify_<evidence-id>` function that calls every Rust-decorated binding in the
file, including opaque-handle methods. Package examples must be referenced from
`fixture.json.package_examples`, must use the exact `examples/<crate>.sifr`
path, and must include a concrete `@rust(...)` declaration plus a
`verify_<crate>_package` function that exercises that binding. Scenario
examples must be referenced from `fixture.json.scenario_examples`, must use the
exact `examples/<scenario>/` directory, and must include a README, Sifr package
config, Sifr source, Cargo manifest, Rust source, and verifier call sites for
every Rust-decorated binding.

The area-level `network_mode` is `offline` for compile/probe and contract
checks. Runtime-observed fixtures that need services such as Redis or
PostgreSQL must use explicit local service configuration recorded in the
fixture evidence; they must not silently degrade to compile-only coverage.

## Tier And Execution Semantics

Tier records subject breadth; `execution_kind` records evidence strength. The
only valid combinations are:

| Tier | Allowed execution kinds |
| --- | --- |
| 0 | `compiler-diagnostic` |
| 1 | `cargo-probe` |
| 2 | `contract-only`, `cargo-probe`, `runtime-observed` |
| 3 | `cargo-probe` |
| 4 | `contract-only`, `cargo-probe`, `runtime-observed` |

`contract-only` certifies only the compiler or metadata contract named by the
row. `cargo-probe` exercises the real Cargo package graph: positive directions
build generated/package Rust code, while negative directions may observe a
required compiler rejection before Cargo execution. `runtime-observed` executes
the lifecycle or runtime behavior named by the row. Neither a contract-only row
nor a compiler-diagnostic row satisfies a build or runtime claim.

Tier-0 rows may list crates only to identify the API shapes used by diagnostic
examples. Such rows must repeat an identical
`diagnostic_crate_rationale` object in the fixture matrix, compatibility row,
and fixture manifest. Its `linked` and `executed` fields are both `false`;
package-example metadata on those rows is illustrative and is not compiled
evidence.

## Suites

- `matrix`: verifies the fixture matrix, required fixture directories, crate
  coverage, schema-v2 evidence and exact executable-test provenance, fixture
  source files, package examples for every required Rust crate, scenario
  examples for package-layout fixture families, and diagnostic family
  inventory.
- `tiers`: verifies tier definitions and fixture tier assignments.
- `compatibility-matrix`: loads fixture manifests, verifies that public
  compatibility rows match fixture evidence, requires two distinct valid test
  bindings for claimed-support rows, and ensures no fixture family is omitted.
- `stale-drafts`: scans active planning and documentation paths for accepted
  examples of abandoned Rust interop syntax and runs the isolated scanner
  mutation self-test in every authoritative profile. Rejected block examples
  open with exactly `` ```sifr-rejected ``; inline mentions require the exact
  `<!-- rust-interop-rejected -->` marker on the same physical line in
  Markdown, or `{/* rust-interop-rejected */}` in MDX. Nearby prose never
  supplies rejection context, accepted examples remain in `sifr` fences, and
  Sifr Rust decorators in `python` fences are always errors.

Run the complete area directly with:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area rust_interop
```

The authoritative create-PR, merge, nightly, and release profiles select all
four suites and execute them through the `rust_interop_checks` legacy-facade
step. Execute the create-PR profile with:

```bash
scripts/run_all_tests.sh --profile create-pr
```

Inspect the same profile's plan without executing it:

```bash
scripts/run_all_tests.sh --profile create-pr --emit-plan
```

## Compatibility Categories

- `supported`: positive and negative fixture evidence both pass for the stated
  execution kind.
- `supported-through-bridge`: the contract is supported through an explicit
  local or shared bridge; direct binding is not implied.
- `unsupported-by-design`: the rejected surface has passing diagnostic evidence
  and no fallback path.
- `future-owned-by-separate-phase`: at least one evidence direction is not
  passing. The row must point at a concrete active issue or phase.

## Runner Modules

The `runner/` modules provide the stable names for compiler, Cargo, probe, and
native-link orchestration used by fixture families:

- `cargo_probe.py`: Cargo metadata and signature probe orchestration.
- `bridge_check.py`: package-local/shared bridge projection checks.
- `trust_check.py`: pre-execution and post-execution trust evidence checks.
- `native_probe.py`: native-link and build-script evidence checks.
- `report.py`: fixture evidence reporting helpers.
