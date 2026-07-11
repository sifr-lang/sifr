# M5 Plan: Fallback Signature Architecture Deletion

## Objective

Delete the fallback signature crate and every resolution path so private
stdlib callables have exactly one authority: compiled `_sifr.*` source
declarations. Missing private declarations must fail with a structured,
deterministic bootstrap diagnostic.

## Work waves

- [x] Remove fallback resolution from driver bootstrap, private stdlib import
  lowering, and the independent module import branch.
- [x] Delete `_sifr.io` and `_sifr.test` placeholders from the sysroot source
  inventory and remove their obsolete fallback-only tests/fixtures.
- [x] Delete `crates/sifr_retained_intrinsics`, workspace/dependent Cargo
  entries, lockfile package state, and dependency-direction policy entries.
- [x] Remove the fallback-glue manifest surface and
  `fallback_signature_modules` schema/allowlist fields.
- [x] Update verification profiles, crate classification, generated-code
  quality ownership, and durable architecture docs for the deleted crate.
- [x] Add bootstrap coverage proving every private import resolves from
  compiled declarations and missing members/modules report structured
  bootstrap diagnostics without recovery.
- [x] Add a permanent repository guard rejecting the deleted crate, Cargo
  dependency, fallback APIs, fallback manifest field, and removed placeholders.
- [x] Run affected checks/tests, native fixtures, all guards, workspace Clippy,
  file-size checks, and the authoritative create-PR gate.
- [x] Run Claude Opus review rounds until `SATISFIED`.
- [ ] Merge the M5 PR and update the phase record with its link.

## Deletion inventory

- Driver: missing-module fallback branch and
  `re_export_intrinsic_fallbacks`.
- Lowering: `resolve_retained_fallback` and the independent
  `get_intrinsic_module` import branch.
- Sysroot: empty `stdlib/_sifr/io.sifr` and `stdlib/_sifr/test.sifr`.
- Crate/dependencies: `crates/sifr_retained_intrinsics`, workspace membership,
  lowering/driver dependencies, lockfile package entry, verification profiles,
  crate classification, and dependency-direction rules.
- Manifest: `retained-fallback-signature-glue` and
  `fallback_signature_modules` schema/allowlist ownership.

## Validation evidence

- The authoritative create-PR gate passed with every blocking lane green:
  crate tests `163565 ms / 600000 ms`, runtime/platform
  `56958 ms / 120000 ms`, and `130/130` selected E2E fixtures at
  `383025 ms / 600000 ms`.
- The stdlib native-intrinsic allowlist guard and negative self-test pass with
  `exact_intrinsics=17`, no fallback-module count, and permanent rejection of
  the deleted crate, APIs, placeholders, Cargo/config state, schema field, and
  durable-doc residue.
- The retained-manifest schema guard and self-test pass with `surfaces=9`; the
  dependency-direction guard and self-test pass; bootstrap ordering passes at
  `private=28, public=61`.
- Focused driver bootstrap tests prove source-backed private imports compile
  and missing private modules/members become structured
  `SIFR-STDLIB-0003` diagnostics. The lowering regression proves an absent
  compiled private member is `SIFR-NAME-0004`, with no recovery path.
- Claude Opus review round 1 returned `SATISFIED`. Because its artifact
  omitted findings it referenced, round 2 repeated the full review with a
  complete acceptance-by-acceptance record, reported no findings, and ended
  `SATISFIED`.
