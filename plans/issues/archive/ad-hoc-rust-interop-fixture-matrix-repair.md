# Ad Hoc Follow-up: Rust-Interop Fixture Matrix Repair

Status: complete on 2026-08-22.

## Objective

Restore exact source-path ownership for the shared-bridge negative fixture.

## Source

The pre-v1 Item 16 create-PR and merge gates stopped at the Rust-interop
matrix. The matrix could not find
`negative/package_generated_type_import_rejected.sifr`.

The pre-v1 regression phase gates later stopped at the same row. Those items
did not change or absorb this separately owned fixture defect.

## Root Cause and Correction

The fixture matrix requires each evidence source at
`<side>/<evidence-id>.sifr`. The shared-bridge manifest used that canonical
path, but the checked-in source was under `negative/src/`.

The Cargo probe independently included the nested source. It could pass while
the authoritative matrix failed.

[#3480](https://github.com/sifr-lang/sifr/pull/3480) moved the source to the
manifest's canonical flat path. It also changed the Cargo probe and README to
name that path. No reader accepts the old location, and no fallback exists.

## Item 0 Record

State: complete

PR: [#3480](https://github.com/sifr-lang/sifr/pull/3480)

Base SHA: `5865bdec8776c909775726dbc69aff6b65beded7`

Candidate SHA: `d9339de46a9aca8c0e6ddd95af807648bd5de398`

Merge SHA: `2021f60ca8970bca76e4f5060cec28994f9addc8`

Changed paths: the negative evidence source location, the Cargo-probe source
include, and the fixture README. The manifest already named the selected
canonical path and did not require a content change.

Validation:

- The configured Rust-interop matrix area passed both variants with zero
  failures.
- The focused matrix self-test passed all 237 cases.
- The focused negative Cargo probe passed.
- HIR maintainability, diff, and file-size checks passed.
- The touched Rust test file contains 799 lines.

Review evidence: the exact-SHA Opus review returned `SATISFIED` with no
blocking finding. The evidence is in the
[#3480 review comment](https://github.com/sifr-lang/sifr/pull/3480#issuecomment-5382310262).

No Sifr create-PR or merge gate applied. The item changed fixture evidence,
documentation, and a test source include, but no compiler implementation.

Deferred follow-up: none.

Next action: the parent pre-v1 regression phase consumes this merge in Item
11.
