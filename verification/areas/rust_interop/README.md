# Rust Interop Verification Area

This area tracks Rust interop verification.

The canonical design is `internal_docs/rust_interop_architecture.md`. This
area is the compatibility inventory for Rust interop: it names every fixture
required by the architecture, records tier assignment, reserves diagnostic
families, and publishes the compatibility matrix used by docs and reviewers.

The fixture matrix is contract-first. Every fixture must declare both positive
and negative evidence. A compatibility row can use `supported`, `supported-through-bridge`, or
`unsupported-by-design` only when both evidence directions are `passing`. Rows that are still `planned` or otherwise incomplete
must be categorized as `future-owned-by-separate-phase`.

The area-level `network_mode` is `offline` for compile/probe and contract
checks. Runtime-observed fixtures that need services such as Redis or
PostgreSQL must use explicit local service configuration recorded in the
fixture evidence; they must not silently degrade to compile-only coverage.

## Suites

- `matrix`: verifies the fixture matrix, required fixture directories, crate
  coverage, evidence objects, fixture READMEs, and diagnostic family inventory.
- `tiers`: verifies tier definitions and fixture tier assignments.
- `compatibility-matrix`: verifies that public compatibility rows match fixture
  evidence and that no fixture family is omitted.
- `stale-drafts`: scans active planning and documentation paths for accepted
  examples of abandoned Rust interop syntax.

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
