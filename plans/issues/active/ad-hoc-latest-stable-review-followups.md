# Latest-stable review follow-ups

Status: open follow-ups, non-blocking for the frozen implementation phase.

Origin: initial read-only Opus review of root candidate
`7d3c4198585352504a2d0c82267d6a0824df50cf`, PR #3827, 2026-09-14.
The blocking policy-test registration omission is handled in the
[phase record](ad-hoc-latest-stable-release-convergence.md).
These four observations were classified as suggestions, infrastructure, or
pre-existing coverage; they do not reopen the frozen version selections.

## Probe identity header coupling

Owner: Rust bridge probe validation.

The package-name replacement currently matches the production manifest header.
The identity unit fixture duplicates that header, so a future production-header
edit could leave replacement unapplied without invalidating that fixture.
A bounded follow-up should test the actual production manifest builder and make
an internal header mismatch explicit. The current real-Cargo contract rejection
and artifact separation regressions remain passing.

## Maintained-demo preparation workspace retention

Owner: verification setup and build-storage lifecycle.

`prepare_maintained_demo_cache` allocates a persistent temporary workspace on
each run. Decide which compact reports and current build inputs must remain,
then clean or reuse the other session-owned preparation trees. Preserve actual
validation receipts and respect worktree ownership; do not remove another
session's target or rewrite historical evidence.

## PostgreSQL dialect terminology

Owner: PostgreSQL build documentation.

The build selects `c17` for MSVC and `gnu17` elsewhere. Clarify the broad
C17 wording as C17 with the GNU dialect on non-MSVC targets. No language-mode
change or parser upgrade is requested by this documentation follow-up.

## Continuous base-crate source inventory coverage

Owner: SQLite native-source provenance.

The maintained SQLite checker validates the four patched `sqlite3/*` inputs.
The receipt retains the authenticated base-crate archive hash, and the original
update evidence records the comparison of the unchanged base files. That is
distinct from continuous whole-tree enforcement by the maintained checker.
Consider extending that enforcement to unchanged base inputs, including the
SQLCipher source and bindings. The selected maintained feature graph does not
enable `bundled-sqlcipher`; enabling it requires separate qualification.
