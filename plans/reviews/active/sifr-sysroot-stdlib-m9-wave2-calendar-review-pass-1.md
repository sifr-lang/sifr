I have all the context I need. Let me write up the review.

## Code Review — M9 Wave 2 (`_sifr.calendar`)

### Findings

**1. Severity: Info — Gregorian weekday algorithm correctness verified**
`crates/sifr_stdlib/src/calendar.rs:40-43,45-61` — Sakamoto's algorithm with `+6` offset (or equivalently `(... + 6).rem_euclid(7)`) maps Python's `calendar.weekday()` convention (Mon=0..Sun=6). Spot-checked: 2024-01-01→0 (Mon), 2024-02-29→3 (Thu), 2000-01-01→5 (Sat). Test `calendar_leaf_matches_gregorian_helpers` (`crates/sifr_stdlib/tests/api_behavior.rs:71-94`) asserts the expected values. `rem_euclid` (not `%`) means negative intermediates also reduce correctly.

**2. Severity: Info — Type-bridge wiring at the int boundary is sound**
`crates/sifr_codegen/src/rust_interop_direct.rs:33-43,45-51,53-84` — Int params are wrapped via `SifrIntBridge::from(...)`; int return becomes `.to_i64_saturating()`; `list[int]` return becomes `.into_iter().map(...).collect()`. Snapshot-style assertions in `direct_rust_function_body_converts_int_arguments_and_return` and `..._integer_list_return` (`rust_interop_direct.rs:170-238`) and `stateless_private_codegen_tests::calendar_private_declarations_codegen_through_sifr_stdlib` (`crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:57-87`) confirm rendered Rust matches.

**3. Severity: Info — No user-triggerable panics**
`crates/sifr_stdlib/src/calendar.rs:1-71` uses no `.unwrap()`, no `.expect()`, no `panic!`, no `assert!`. `to_i64_saturating` (`crates/sifr_runtime/src/interop.rs:56-64`) saturates instead of unwrapping. Out-of-range month in `days_in_month` returns 30, consistent with the no-panic policy; the public wrapper `TextCalendar.formatmonthname` (`stdlib/sifr/calendar.sifr:105-128`) raises `ValueError` for month∉1..12, and `cpython_calendar_subset.sifr:60-65` exercises that error path.

**4. Severity: Info — Old intrinsic surface removed and asserted gone**
`crates/sifr_codegen/src/intrinsics/registry.rs:1-41,58-872` no longer declares `mod calendar` and no longer dispatches `calendar_isleap/_weekday/_monthrange`. Deletion of `crates/sifr_codegen/src/intrinsics/registry/calendar.rs` is asserted by `lowers_calendar_intrinsics_via_registry` (`registry_extended_tests.rs:364-372`). The compiled path is asserted to win: `stateless_private_codegen_tests.rs:72-86` checks `intrinsic_names["_sifr.calendar"]` is empty and that `sifr.calendar`'s intrinsic names exclude `calendar_isleap`.

**5. Severity: Info — Feature/dependency wiring is narrow and consistent**
- `crates/sifr_stdlib/Cargo.toml:97` — `calendar = ["dep:sifr_runtime"]` (required because `SifrIntBridge` lives in `sifr_runtime`).
- `crates/sifr_stdlib/src/lib.rs:11-12` and `feature_contract.rs:8` register the leaf.
- `crates/sifr_stdlib_model/src/features/generated_stdlib_features.rs:32` maps `sifr.calendar`/`_sifr.calendar` → `["calendar"]`; minimality verified by `planned_sysroot_stdlib_features_are_minimal_for_representative_modules` (`features_tests.rs:181-185`) and stateless-only-`sifr_stdlib` emission verified by `stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies` (`features_tests.rs:218-240`).
- E2e fixture path: `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs:346-347` triggers the stdlib dep for `sifr.calendar`/`_sifr.calendar`; `fixture_dependency_paths.rs:56-58` enables the `calendar` feature; `test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features` (`harness_behavior_tests.rs:515-540`) asserts both single and combined `["html", "calendar", "platform"]` ordering.

**6. Severity: Info — Probe bridge-type stub correctly splits dotted modules**
`crates/sifr_driver/src/build/rust_interop_probe.rs:425-440` now nests `pub mod _sifr { pub mod calendar { ... } }` instead of emitting an invalid `pub mod _sifr.calendar`. Unit-tested by `generated_bridge_type_stubs_split_dotted_module_names` (`rust_interop_probe.rs:587-612`). Necessary because the `_sifr.calendar` private module's bridge stubs are now generated.

**7. Severity: Info — Intentional dual-track: intrinsic type signatures retained as bootstrap fallback**
`crates/sifr_stdlib_model/src/lib.rs:163-168` retains `_sifr.calendar => intrinsic_calendar()` with the wave-1-pattern comment. Consumer `crates/sifr_driver/src/stdlib/bootstrap.rs:165-218` first checks compiled exports and only falls back to `get_intrinsic_module` if none exist. With `stdlib/_sifr/calendar.sifr` present in `PRIVATE_STDLIB_MODULES` (`sources.rs:54-86`), the compiled @rust path wins — confirmed by the intrinsic-names empty-set assertion above. This is consistent with wave 1's treatment of `_sifr.platform` and `_sifr.html`.

**8. Severity: Info — Phase tracker and architecture doc updates are accurate**
`plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md` records wave 2 status with PR-pending note, lists evidence and the validation commands actually run, and updates the M9 task table. `internal_docs/sifr_sysroot_and_stdlib_architecture.md:391-405` adds calendar to the migrated-leaves list and calls out the SifrIntBridge boundary for `list[int]` returns.

### Observations (no action required)

- For pathological year inputs (e.g. `i64::MAX`), `calendar_weekday_i64` could overflow in debug builds. Realistic year ranges (1..9999) are safe and the bridge already saturates inputs to i64. Not introduced by this wave.
- `crates/sifr_codegen/src/intrinsics/registry.rs` is 880 lines — under the 900-line guardrail; this wave reduces it by 3 entries.
- `plans/reviews/active/sifr-sysroot-stdlib-m9-wave2-calendar-review-pass-1.md` is an empty placeholder file (0 bytes). Worth populating with the wave 2 review record before PR, or removing if unused.

### VERDICT: PASS

The wave 2 migration is complete and consistent with wave 1. Boundary types and feature flags are correct, the old intrinsic surface is removed and asserted, no panics are introduced, and the phase tracker / architecture doc accurately record the change. Local validation commands listed in the tracker exercise the relevant unit, registry, codegen, driver, stdlib, model, and e2e paths.
