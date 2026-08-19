# Ad Hoc Follow-up: Rust-Interop Fixture Matrix Repair

Status: queued after the pre-v1 compatibility-removal phase closes.

## Objective

Restore exact source-path ownership for the shared-bridge negative fixture.

## Source

The pre-v1 Item 16 create-PR and merge gates each stopped at the Rust-interop
matrix on exact candidate `d3ae65fadfa95d4dcc44428ea4ab8b41106466dd`.
Both attempts reported that `shared_bridge_crate` could not find
`negative/package_generated_type_import_rejected.sifr`.

The checked-in source is currently at
`verification/areas/rust_interop/fixtures/shared_bridge_crate/negative/src/package_generated_type_import_rejected.sifr`,
while `fixture.json` records the path without `src/`.

## Item 0: Reconcile Shared-Bridge Evidence Location

Select one canonical fixture layout. Update the manifest, source location,
README evidence, and all matrix consumers together. Do not add a fallback path
or accept both layouts.

Acceptance criteria:

- `fixture.json` names the only checked-in negative source path.
- The focused Rust-interop matrix and its self-test pass.
- The negative Cargo-probe test consumes the same source.
- No compatibility reader accepts the old path.

## Validation

- `python3 verification/areas/rust_interop/runner.py --suite matrix`
- The focused shared-bridge negative Cargo-probe test
- File-size and diff checks

## Next Action

Start Item 0 in a separate phase-closure session. Do not rerun either consumed
pre-v1 Item 16 gate as evidence for this repair.
