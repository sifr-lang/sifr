# M4 Plan: Collections Residue Removal

## Objective

Delete the dead JSON-backed Counter intrinsic path and serialized defaultdict
adapter while preserving generic source `Counter[T]`, typed language-owned
`defaultdict`, and checked public bytes wrapper behavior.

## Work waves

- [x] Remove the eight temporary Counter IDs, codegen dispatch arms, lowerers,
  feature requirements, registry-only tests, and retained fallback signatures.
- [x] Delete the serialized `_defaultdict_new/get/set_impl` declarations,
  public wrappers, native Rust implementations, and their driver/unit tests.
- [x] Remove Counter-only direct `serde`/`serde_json` feature-planning
  dependencies and update dependency-plan tests.
- [x] Rewrite retained-boundary manifest rows for typed defaultdict language
  semantics and primitive bytes constructors using the current schema.
- [x] Add static residue guards and focused behavior evidence proving Counter
  routes through `stdlib/sifr/collections.sifr` and typed defaultdict remains
  compiler/type-system lowering rather than serialized bridge glue.
- [x] Prove public `sifr.bytes` wrappers execute checked source bodies and only
  those bodies reach primitive typed intrinsic HIR.
- [x] Run generic Counter, typed defaultdict, bytes-wrapper, codegen, bootstrap,
  manifest, allowlist, file-size, and authoritative create-PR validation.
- [x] Run Claude Opus review rounds until `SATISFIED`, merge the M4 PR, and
  update the phase record.

## Deletion inventory

- Temporary typed identities and dispatch:
  `CompilerIntrinsicId::{CounterFromList, CounterGet, CounterMostCommon,
  CounterTotal, CounterValues, CounterKeys, CounterItems, CounterIncrement}`.
- Registry code:
  Counter functions in
  `collections/set_and_list_intrinsics.rs` plus
  `collections/counter_defaultdict_intrinsics.rs`.
- Fallback signatures:
  eight `counter_*` entries and three serialized `_defaultdict_*_impl` entries
  in `sifr_retained_intrinsics`.
- Serialized bridge:
  `_sifr.collections` declarations, public `sifr.collections` wrappers, and
  `sifr_stdlib::collections::defaultdict_*` implementations/tests.
- Dependency residue:
  `SerdeJson` retained direct specs and the manifest allowlist entries for
  `serde` and `serde_json` when no remaining compiler-owned surface needs them.

## Validation evidence

- Affected compiler, driver, stdlib, manifest, and retained-signature crates
  compile cleanly.
- Codegen unit suite: 743 passed.
- Typed defaultdict lowering filters: 8 passed; Counter/defaultdict ownership
  filters: 2 passed.
- Driver bootstrap/codegen proofs passed for `_sifr.collections` set bridges
  and public `sifr.bytes` checked wrapper bodies.
- Stdlib manifest: 28 passed; retained signatures: 4 passed.
- Native runs passed for `collections_boundary_ownership`,
  `generic_counter_int`, `generic_counter_bigint`,
  `generic_counter_custom_class`, and `defaultdict_len_and_deque`.
- Native allowlist plus self-test, manifest schema plus self-test, and bootstrap
  ordering plus self-test passed with 17 exact intrinsic IDs and six retained
  direct dependency packages.
- Workspace Clippy with warnings denied and changed-file rustfmt checks passed.
- Authoritative `scripts/run_all_tests.sh --profile create-pr` passed with the
  current temp-target compiler/LSP binary: crate tests 126,439 ms / 600,000 ms,
  runtime platform 58,139 ms / 120,000 ms, E2E 31,402 ms / 600,000 ms, and
  130/130 E2E fixtures passed. The 445.61-second warm-target advisory was
  non-blocking.
- Claude Opus review rounds 1 and 2 returned `SATISFIED`; the round-1 soft
  structured-intrinsic coverage gap was restored with retained bytes/test IDs
  and verified in round 2.
