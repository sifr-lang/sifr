# M6 Plan: Reachability Guard and Final Recertification

## Objective

Close the phase with executable exactness checks across the full final
compiler/stdlib boundary. Every public native adapter, retained typed
intrinsic, owned lowering/codegen file, and retained direct dependency must be
reachable from one explicit source-owned architecture path, and source-tree
and installed sysroots must certify equivalent behavior and dependency plans.

## Work waves

- [x] Inventory every production `sifr_stdlib` public adapter and classify it
  as an active `@rust` target, documented cross-module substrate, or deletion/
  privatization candidate.
- [x] Add a permanent native-adapter reachability guard and negative self-tests
  for unowned public adapters and stale documented substrates.
- [x] Extend the retained manifest schema with exact lowering/codegen ownership
  fields and backfill all retained surfaces.
- [x] Make the retained-intrinsic guard compare source declarations, typed HIR
  IDs, dispatch implementations, manifest identities, and owned files exactly.
- [x] Tie each retained direct dependency package to a live typed-intrinsic
  feature requirement and reject orphan manifest/dependency-plan rows.
- [x] Add explicit negative tests for forbidden compiler-intrinsic declarations,
  former-name collisions, removed fallback imports/APIs/schema, first-class
  retained callable use, missing private source declarations, and orphan
  retained dependency packages.
- [x] Update durable architecture, roadmap, dependency snapshots, and
  snapshots, and traceability reports for the final boundary.
- [x] Certify representative retained-intrinsic and migrated-bridge programs
  against both source-tree and installed sysroots, including dependency-plan
  equivalence.
- [x] Run focused guards/tests, the authoritative create-PR gate, and Claude
  Opus review rounds until `SATISFIED`.
- [ ] Open, review, and merge the M6 PR; then run the full merge gate on final
  `main` and complete the phase-wide review/closure record.

## Validation evidence

- Native-adapter reachability: `403` public adapters = `399` active `@rust`
  targets + `4` documented compiler substrates; guard and negative self-test
  pass.
- Retained-intrinsic exactness: `17` typed identities, `8` registry files, `9`
  preamble files, `8` lowering files, `4` codegen files, and `6` retained direct
  dependency packages; guard and negative self-test pass.
- Manifest schema and self-test pass with `10` final retained-by-design
  surfaces.
- Focused lowering, package-source rejection, missing-private-source, and
  `sifr_stdlib --all-features` tests pass.
- File-size, HIR maintainability, bootstrap-ordering, formatting, and diff
  guards pass.
- Installed/source-tree boundary equivalence passes: both binaries report
  `stdlib boundary recertification: pass` and both normalized generated plans
  match the reviewed `sifr_stdlib[bytes]` dependency snapshot. A post-review
  rerun also passes with separate fresh bridge-probe caches for the installed
  and source-tree paths (`645,113 ms`).
- Authoritative create-PR gate passes with crate tests at `153,367 ms /
  600,000 ms`, runtime-platform at `58,133 ms / 120,000 ms`, and `130/130`
  E2E fixtures at `32,483 ms / 600,000 ms`. The initial run exposed a cold LSP
  preview-cache timeout; the exact cache was warmed through the normal compiler
  metadata path, the standalone smoke passed, and the complete gate rerun was
  green. Review rounds, PR merge, final merge gate, and phase-wide closeout
  review remain pending.
- Claude Opus M6 review round 1 reported no blocking findings and ended
  `SATISFIED`. Its soft blind spots were nevertheless hardened: public
  unsafe/extern functions are now inventoried, structured compiler consumers
  require an exact Rust IR path, and installed/source probe caches are isolated.
  Round 2 reported no blockers or optional cleanups and also ended `SATISFIED`.
