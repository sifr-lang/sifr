# M9 Wave 4 (Math Migration) — Opus Review Pass 3 Findings

## Severity legend
- **High** = behavior bug, leakage of private API, or panic vector
- **Medium** = consistency/correctness concern that warrants discussion but does not break the wave's contract
- **Low / Note** = pre-existing observation or cosmetic clean-up

---

## Findings

### Medium (carried from pass 1) — Frontend duplicate `should_export_callable` — RESOLVED
- `crates/sifr_frontend/src/query_diagnostics.rs:264-278` and `crates/sifr_driver/src/export_policy.rs:1-15` are now byte-equivalent: both early-return `false` for `sifr.math` × `{dist_impl,fsum_impl,sumprod_impl}` before the `_`-prefix rule and the `sifr.heapq` carve-out. The frontend version is the one wired into `collect_module_exports` (`query_diagnostics.rs:296, 319`), so this also closes the latent leakage path through the analysis cache.
- The `pub(super) → pub(crate)` visibility bump is safe: `lib.rs:25` does `pub use query_diagnostics::*;`, which only re-exports `pub` items; `pub(crate)` stays crate-internal.
- New focused test `frontend_export_policy_hides_math_bridge_helpers` (`query_diagnostics_behavior_tests.rs:7-32`) locks the invariant: positive cases for the three `_impl` names, retention of `dist`, the `_heapify_max` carve-out, and the default `_`-prefix rule (`_copy_float_list` dropped). Mirrors the driver-side assertion at `stateless_private_codegen_tests.rs:171-176`. Fix is adequate.

### Low — Empty `registry/math/` directory left on disk
- `crates/sifr_codegen/src/intrinsics/registry/math/` is empty but still present (`ls` confirms). Git does not track empty directories so it will not appear in the PR diff or on a fresh clone; `registry.rs` no longer declares `mod math;`, so it is unreachable. Cosmetic only.

### Low — `LEAF_FEATURES` contract still incomplete (pre-existing)
- `crates/sifr_stdlib/src/feature_contract.rs` adds `math` (correct for this wave) but `random`/`time`/`encoding`/`numeric` features from `Cargo.toml` remain unenumerated and so are not covered by the `marker_modules_report_leaf_names` test. Pre-existing; not introduced by this wave.

### Note — Semantic parity carried over from deleted intrinsics
These match the removed intrinsic implementations bit-for-bit (verified against `HEAD:crates/sifr_codegen/src/intrinsics/registry/math/*`), so they are parity rather than regressions, but worth recording for M12:
- `crates/sifr_stdlib/src/math.rs:116-118` `fma` is `(x*y)+z` (double rounding), not a true FMA.
- `crates/sifr_stdlib/src/math.rs:121-123` `isqrt(n)` saturates: negative → `NaN as i64` → `0`. CPython raises; old intrinsic also saturated.
- `crates/sifr_stdlib/src/math.rs:339-341` `ldexp` uses `e.to_i64_saturating() as i32` — `as i32` truncates rather than saturates on i64 overflow. Same as old intrinsic; no panic.
- `crates/sifr_stdlib/src/math.rs:285-294` `gamma(inf)` evaluates `inf.powf * (-inf).exp * sum` → NaN, where CPython returns `+inf`. Same Lanczos algorithm as before.

### Note — Allocation cost of borrowed→owned bridge
- `dist`/`fsum`/`sumprod` now allocate a fresh `Vec<f64>` per call via `_copy_float_list`. This is the intentional cost of preserving borrowed public list semantics while keeping the `_impl` bridges on owned `Vec<f64>`. Documented in `internal_docs/sifr_sysroot_and_stdlib_architecture.md:404-407`. Not a blocker; flagged for visibility.

---

## Confirmation of the requested invariants

- **No `dist_impl/fsum_impl/sumprod_impl` leakage on the public surface.** Driver-side hook: `export_policy.rs:2-6` + `bootstrap.rs:256-258` retain; frontend-side hook: `query_diagnostics.rs:265-269` + `collect_module_exports`. Both tested (`stateless_private_codegen_tests.rs:171-176`, `query_diagnostics_behavior_tests.rs:7-32`). `_copy_float_list` is filtered by the default `_`-prefix rule.
- **Borrowed-public / owned-private boundary preserved.** `stdlib/sifr/math.sifr:1-122` keeps `def dist(p: list[float], q: list[float])` with default borrow convention; `stateless_private_codegen_tests.rs:160-170` asserts `ParamConvention::borrow()` for `dist`/`fsum`/`sumprod`. Private bridges in `stdlib/_sifr/math.sifr:195-205` use `own list[float]` which lines up with `Vec<f64>` in `sifr_stdlib::math::*`.
- **Compiler intrinsic registry no longer owns math.** `registry.rs:1-60` removes `mod math;` and every math arm; `registry_core_tests.rs:33-51` and `registry_extended_tests.rs:639-661` assert representative core/extended math names return `None`. `intrinsic_math()` remains only as a bootstrap fallback in `sifr_stdlib_model/src/lib.rs:147-149` (correctly documented).
- **Feature planning is minimal.** `features/generated_stdlib_features.rs:34` maps both `sifr.math` and `_sifr.math` to `["math"]`. `features_tests.rs:198-202, 226` and the e2e fixture harness (`fixture_dependency_paths.rs:65-67`, `fixture_cargo_toml.rs:343-346`, `harness_behavior_tests.rs:516-539`) all confirm minimal feature/dep emission.
- **No user-triggerable panics in `math.rs`.** All scalar helpers use IEEE-total f64 operations or `SifrIntBridge::to_i64_saturating()` + saturating `as i64` casts. `nextafter` excludes NaN/equal/zero upfront so `bits ± 1` cannot wrap into a NaN encoding from finite inputs (worst cases land on `±INFINITY` / `±0.0`). `ulp` excludes NaN/inf/zero/`±f64::MAX` before calling `nextafter`. `gamma`/`lgamma` short-circuit the non-positive integer pole. No `.unwrap()`/`.expect()` introduced.
- **Constant precision preserved.** `_sifr/math.sifr` declares `pi = 3.141592653589793`, `e = 2.718281828459045`, `tau = 6.283185307179586` — each exact f64 of `std::f64::consts::{PI,E,TAU}` to the last bit. `inf = 1.0 / 0.0` / `nan = 0.0 / 0.0` are accepted in Rust const context (verified by the executed e2e math suite).
- **Architecture and tracker docs updated.** `internal_docs/sifr_sysroot_and_stdlib_architecture.md:393-410` and `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:21, 37, 619-789` accurately describe the wave, including the borrowed-public/owned-private boundary rationale and validation evidence.

## VERDICT: PASS

Ready to proceed to local `scripts/run_all_tests.sh --profile create-pr` and PR.
