# Rust Interop Verification Area

This area tracks Rust interop verification.

The canonical design is `internal_docs/rust_interop_architecture.md`. This
area intentionally starts as an architecture scaffold: it names every fixture
required by the architecture, records the tier assignment, reserves the
diagnostic families, and provides runner skeletons for subsequent implementation.

The matrix is contract-first. A fixture can be marked `planned`, `probe-only`,
or `runtime-observed`, but it must always declare both positive and negative
evidence before a subsequent implementation step can call the capability complete.

The area-level `network_mode` is `offline` for scaffold and compile/probe
checks. Later runtime-observed fixtures that need services such as Redis or
PostgreSQL must use explicit local service configuration recorded in the
fixture evidence; they must not silently degrade to compile-only coverage.

## Suites

- `matrix`: verifies the fixture matrix, required fixture directories, crate
  coverage, evidence placeholders, and diagnostic family inventory.
- `tiers`: verifies tier definitions and fixture tier assignments.
- `stale-drafts`: scans active planning and documentation paths for accepted
  examples of abandoned Rust interop syntax.

## Runner Skeletons

The `runner/` modules are intentionally thin until subsequent implementation
wires them to compiler, Cargo, probe, and native-link execution:

- `cargo_probe.py`: Cargo metadata and signature probe orchestration.
- `bridge_check.py`: package-local/shared bridge projection checks.
- `trust_check.py`: pre-execution and post-execution trust evidence checks.
- `native_probe.py`: native-link and build-script evidence checks.
- `report.py`: fixture evidence reporting helpers.
