# Rust Interop Certification 4 Review — Round 2

Reviewer: Claude Opus 5 (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer verified that round 1's file-size, proxy isolation, cancellation
accounting, runtime identity, native-link trust, provenance, target-copy,
matrix/count, transfer-inventory, and gate findings were fixed. It independently
ran the hostile-proxy runtime scenario, all three generated-build tests, 149
focused driver tests, Clippy, formatting, and matrix checks with the unrelated
opaque-row hunk excluded in memory.

Blocking findings:

1. The local-function exemption for a bare `block_on()` call depended on source
   traversal order, so a local helper declared after its caller was rejected.
2. The audit scanned only `package_root/src`, even though manifest-declared
   bridge roots may be outside `src`.

Additional findings covered module/crate aliases for Tokio runtime constructors,
a pre-probe assertion accidentally placed in a sync bridge test, missing
mutation self-tests for the async scenario policy, repeated source parsing per
declaration, use of an inline bridge-root predicate, and low line-count headroom
in an existing test module.

Correction wave:

- Added a local-function pre-pass so audit results are declaration-order
  independent.
- Audited the union of package `src/` and every declared bridge root, with
  deduplicated diagnostics.
- Added Tokio crate/module alias tracking and tests for
  `runtime::Builder`/renamed-`tokio` constructors.
- Cached violations per package during one resolver pass and reused
  `uses_bridge_root`.
- Moved the pre-probe assertion to the async negative generated-build test.
- Added six async scenario self-test cases covering the baseline, exact reqwest
  and Tokio policy, build-script/native-link trust, and proxy bypass.
- Updated public/architecture/transfer-inventory scope and retained all touched
  hand-maintained files below 900 lines.

Round 3 is required.
