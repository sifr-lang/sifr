# M3 Plan: Typed Intrinsic Identity and Source-Declared Retained Callables

## Objective

Replace raw function-name intrinsic interception with typed HIR identity while
preserving the 17 intentionally retained compiler operations and migrating
`bytes_to_hex_strict` to the native stdlib bridge.

## Work waves

- [x] Define `CompilerIntrinsicId`, `HirFunction.compiler_intrinsic`, and
  `HirExpr::IntrinsicCall` with result type, call range, and argument ranges.
- [x] Add sysroot-only `@compiler_intrinsic(<id>)` declaration validation,
  export metadata through `ExternalDefs`, preserve it through imports,
  aliases, and re-exports, and reject first-class value use.
- [x] Convert the seven `sifr.test` callables and `sifr.task.current_context`
  to source declarations with no emitted body; remove the `_sifr.task`
  placeholder dependency.
- [x] Migrate `bytes_to_hex_strict` to a public `sifr_stdlib::bytes` adapter,
  declare it in `_sifr.bytes`, and keep the `hashlib.sifr` caller on that
  bridge.
- [x] Convert open, bytes construction, string encode, and bytes decode
  lowering sites to typed intrinsic calls.
- [x] Make codegen exhaustive over typed IDs and remove raw-name dispatch from
  every ordinary `HirExpr::Call`.
- [x] Update every HIR consumer, walker, analysis, source-map path, snapshot,
  and runtime-needs query for the new variant.
- [x] Add declaration-policy, alias/re-export, first-class-use, callsite-range,
  primitive-construction, and former-name collision regressions.
- [x] Run focused check/emit/build/run coverage, guardrails, and the create-PR
  gate.
- [ ] Run Claude Opus review rounds until `SATISFIED`, merge the M3 PR, and
  update the phase record.

## Typed ID inventory

| Responsibility | IDs |
| --- | --- |
| Test assertions | `TestAssertEqual`, `TestAssertNotEqual`, `TestAssertTrue`, `TestAssertFalse`, `TestAssertAlmostEqual`, `TestAssertGreaterThan`, `TestAssertLessThan` |
| Shadowable open | `OpenBinary`, `OpenText` |
| Bytes construction | `BytesFromHex`, `BytesWithSize`, `BytesFromIntegers` |
| Encoding primitives | `StringEncode`, `StringEncodeWithEncoding`, `BytesDecode`, `BytesDecodeWithEncoding` |
| Task context | `TaskCurrentContext` |

Counter IDs remain temporarily typed in M3 only if required to keep M4 as the
bounded deletion milestone; they must not remain reachable by raw callable
name.

## Validation evidence

- Focused lowering policy/identity tests: 7 passed, including unaliased import
  shadowing by a local function declaration.
- Full M3 crates: codegen 745 passed; driver 327 passed, 18 ignored by the
  unit profile; lowering 612 passed, 1 ignored by the unit profile.
- Synthetic stdlib re-export and public declaration export tests passed.
- `stdlib_hashlib_intrinsics.sifr` passed check, emit, build, and run; emitted
  Rust calls `sifr_stdlib::bytes::bytes_to_hex_strict` directly.
- `compiler_intrinsic_name_collisions.sifr` passed native run with local,
  nested, method, imported, test-alias, and task-alias coverage.
- `demos/typed_compiler_boundary_demo.sifr` passed native run.
- Manifest schema, typed intrinsic allowlist plus self-test, bootstrap ordering,
  HIR maintainability, and 900-line file-size guards passed.
- Authoritative `scripts/run_all_tests.sh --profile create-pr` passed with a
  clean temporary Cargo cache: crate tests 130,922 ms / 600,000 ms, runtime
  platform 59,782 ms / 120,000 ms, E2E 402,726 ms / 600,000 ms, and 129/129
  E2E fixtures passed. The 869.28-second cold-cache wall time was advisory.
- Claude Opus review rounds 1, 2, and 3 returned `SATISFIED`; round 3 verified
  the local-shadowing reconciliation introduced from the round-2 advisory.
