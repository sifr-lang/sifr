# M9 Wave 3 (UUID) Review — Sifr Sysroot/Stdlib Toolchain

## Findings

### Correctness / parity

- **uuid4 byte-format parity** — *(info)* `crates/sifr_stdlib/src/uuid.rs:9` uses `uuid::Uuid::new_v4().hyphenated().to_string()` which emits the canonical `8-4-4-4-12` lowercase hex with version `4` at byte index 14 and variant nibble in `{8,9,a,b}` at index 19. This matches the prior active intrinsic in the deleted `crates/sifr_codegen/src/intrinsics/registry/uuid.rs` (manual `0x4000` / `0x8000` bit construction). Behavioral parity preserved; `api_behavior.rs:106`–`120` exercises both invariants.
- **uuid3/uuid5 nil-fallback parity** — *(info)* `name_based_uuid` at `crates/sifr_stdlib/src/uuid.rs:21` uses `parse_str(namespace).unwrap_or_else(|_| Uuid::nil())`, mirroring the deleted intrinsic's `unwrap_or(Uuid::nil())`. The `panic=trusted_no_panic` decoration in `stdlib/_sifr/uuid.sifr:1,5,9` is therefore safe (no user-triggered panic path). `api_behavior.rs:118`–`121` regression-pins the nil-fallback output `0421fac3-a9c6-3ea3-aee8-8f20aff3f278`.

### Dependency feature selection

- **uuid crate now needs v4** — *(info)* `Cargo.toml:142` adds `"v4"` to the `uuid` feature list, required by `Uuid::new_v4`. v3/v5 retained.
- **Module-level feature minimization** — *(info)* `crates/sifr_stdlib_model/src/features.rs:608` returns `&[]` for `sifr.uuid`/`_sifr.uuid`, properly delegating to the sifr_stdlib `uuid` feature. `features_tests.rs:186,225` add both the planned-features and direct-deps assertions.
- **Fixture dependency rewiring** — *(info)* `fixture_cargo_toml.rs` removes the direct `rand`+`uuid` insertion (was at lines 46–51) and adds the module to `needs_sifr_stdlib_module_dependency` at line 342. `fixture_dependency_paths.rs:62` propagates the `uuid` sifr_stdlib feature. `harness_behavior_tests.rs:519`–`531` asserts no direct `rand = `/`uuid = { version` lines and verifies combined ordering `["html", "calendar", "platform", "uuid"]`.

### Deleted intrinsic coverage

- **Registry deletion is non-regression-tested** — *(info)* `registry.rs:40,556`–`559` drops the `uuid` mod and three dispatch arms; `registry_extended_tests.rs:201`–`208` (`lowers_uuid_intrinsic_via_registry`) asserts all three names return `None` from `lower_intrinsic`, with a message anchoring the contract ("must lower through private stdlib Rust interop, not active intrinsics").
- **Private codegen path tested** — *(info)* `stateless_private_codegen_tests.rs:89`–`116` (`uuid_private_declarations_codegen_through_sifr_stdlib`) asserts the generated `_sifr.uuid` Rust code references `sifr_stdlib::uuid::uuid4()`, `uuid3_text(namespace, name)`, `uuid5_text(namespace, name)`, that the intrinsic_names set is empty for `_sifr.uuid`, and that `sifr.uuid` transitively depends on `_sifr.uuid` and no longer carries `uuid4` as an intrinsic name. Strong coverage.

### Bootstrap fallback retention

- **`intrinsic_uuid()` still returned by `get_intrinsic_module`** — *(low / informational)* `crates/sifr_stdlib_model/src/lib.rs:148`–`150` moves `_sifr.uuid` under the existing "Retained as a stdlib-lowering bootstrap fallback while these leaves migrate to compiled private declarations" comment. This is dead for the normal path because `crates/sifr_driver/src/stdlib/bootstrap.rs:167`–`190` short-circuits via `has_compiled_exports` once `_sifr/uuid.sifr` ships compiled `@rust(...)` decls. The retention is consistent with the wave-2 calendar approach (same comment, same shape). No correctness issue — flagged only because the now-dead `intrinsic_uuid()` in `crypto_regex_uuid.rs:500`–`533` is a known future cleanup, not a regression.

### File-size and harness trim

- **harness_behavior_tests.rs at 890 lines** — *(info)* Below the 900-line guardrail. The test consolidation at `harness_behavior_tests.rs:514`–`532` collapses three separate per-module probes into one combined toml assertion plus a focused uuid-only block. Coverage of `platform`/`html`/`calendar` is preserved by the combined-modules assertion (each appears in the expected `features = […]` string), so no regression in earlier-wave verification.

### Docs / phase tracker accuracy

- **Architecture doc** — *(info)* `internal_docs/sifr_sysroot_and_stdlib_architecture.md:394`–`400` accurately adds `_sifr.uuid` and `sifr_stdlib::uuid::*` to the migrated-leaves list.
- **Phase tracker** — *(info)* `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:21,36,619`–`625,713`–`741` records wave 3 with implementation evidence, exact `CARGO_TARGET_DIR=target/m9-uuid` commands matching the validation list, and moves UUID out of the pending tasks bullet (line 619) into the wave-3-complete bullet (line 620). PR-link entry deferred with `PR link pending`, consistent with prior waves' tracker discipline.

## Risk summary

- No user-triggerable panics introduced. `getrandom`-backed `new_v4` panic is a system-condition failure with the same surface as the prior `rand::random` path.
- No direct third-party deps leak into generated `sifr_output` Cargo.toml for UUID fixtures (asserted positively *and* negatively).
- Deleted intrinsic surface is locked down by an explicit `is_none()` assertion across all three names.
- Behavioral parity (formatting, version/variant bits, nil-fallback determinism) is pinned by both the stdlib API test and the e2e fixtures `stdlib_uuid_consolidated.sifr` / `cpython_uuid_subset.sifr`.

## VERDICT: PASS
