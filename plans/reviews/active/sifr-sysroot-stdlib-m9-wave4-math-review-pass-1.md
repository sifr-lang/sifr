I have enough context. Let me compose the review.

# M9 Wave 4 (Math Migration) — Review Findings

## Severity legend
- **High** = behavior bug, leakage of private API, or panic vector
- **Medium** = consistency/correctness concern that warrants discussion but doesn't break the wave's contract
- **Low / Note** = pre-existing observation or cosmetic clean-up

---

## Findings

### Medium — Frontend duplicate of `should_export_callable` lacks the math-impl filter
- `crates/sifr_driver/src/export_policy.rs:2-6` excludes `dist_impl|fsum_impl|sumprod_impl` from `sifr.math`'s exports. This is correctly hooked through `crates/sifr_driver/src/stdlib/bootstrap.rs:256-258` (the `retain` on `fn_exports`), and `stateless_private_codegen_tests.rs:171-176` proves the public defs do not contain the `_impl` names.
- However, `crates/sifr_frontend/src/query_diagnostics.rs:264-273` is a near-duplicate of `should_export_callable` and is the version called by `collect_module_exports` (`query_diagnostics.rs:275`), which is invoked from both the driver project flow (`crates/sifr_driver/src/project/frontend.rs:65, 124, 231`) and the analysis cache (`crates/sifr_frontend/src/graph_cache_and_queries.rs:556, 573, 742`). Today the leak is not user-reachable (user modules cannot live in the `sifr.*` namespace and cannot import `_sifr.*`), so this is not a runtime defect for this wave; but it is now a load-bearing invariant maintained in two places that disagree. Either delete the frontend duplicate and re-use the driver-side policy, or mirror the math filter into the frontend copy, before the next wave adds more entries.

### Low — Empty `registry/math/` directory left behind on disk
- `git status` shows the four `crates/sifr_codegen/src/intrinsics/registry/math/*.rs` files as deleted plus `registry/math.rs` (good), but the empty `registry/math/` directory remains in the working tree. It is harmless (no `mod math;` is declared in `registry.rs`) and will not appear on a fresh checkout. Cosmetic only.

### Low — `LEAF_FEATURES` contract is incomplete (pre-existing)
- `crates/sifr_stdlib/src/feature_contract.rs:1-25` lists `math` (good for this wave) but still does not include `random`, `time`, `encoding`, or `numeric`, all of which are defined in `crates/sifr_stdlib/Cargo.toml:101-110`. The `marker_modules_report_leaf_names` test in `tests/api_behavior.rs:177-221` only covers the leaves named in `LEAF_FEATURES`, so the contract drift is not caught. Pre-existing; not introduced by this wave.

### Note — Semantic divergences carried over from the deleted intrinsics
These are *parity* with the removed intrinsic implementations, not regressions, but they're worth recording so M12 doesn't claim "if it compiles, it works" parity with CPython that the math leaf can't actually deliver:

- `crates/sifr_stdlib/src/math.rs:116-118` `fma(x, y, z)` is implemented as `(x * y) + z` (two ops, double rounding). Python `math.fma` is a true single-rounded FMA. Same as old `lower_fma`.
- `crates/sifr_stdlib/src/math.rs:121-123` `isqrt(n)` saturates: negative `n` → `sqrt` → NaN → `as i64` → `0`. Python raises `ValueError`. Sifr cannot raise; result is consistent with the old intrinsic.
- `crates/sifr_stdlib/src/math.rs:339-341` `ldexp(m, e)` casts via `e.to_i64_saturating() as i32`. `as i32` *truncates* (not saturates) on `i64` overflow, so very large `e` wraps before `powf`. Python raises `OverflowError`; old intrinsic had the same wrap. No panic.
- `crates/sifr_stdlib/src/math.rs:285-294` `gamma(f64::INFINITY)` evaluates `inf.powf(...) * (-inf).exp() * sum` = `inf * 0 * finite` = NaN, where CPython returns `+inf`. Same Lanczos algorithm as the old intrinsic.

None of these introduce a user-triggerable panic. All four are reachable only via well-typed float arguments and rely on IEEE-defined operations or saturating integer casts.

---

## Confirmation of the requested invariants

- **No `dist_impl/fsum_impl/sumprod_impl` leakage on the public surface.** `stdlib/sifr/math.sifr:3` does re-import them and `re_exports::re_export_stdlib_imports` does copy them into `sifr.math`'s `fn_exports`, but `bootstrap.rs:256-264` retains via `should_export_callable`, which explicitly drops those three names for `sifr.math` (`export_policy.rs:2-6`). The test at `stateless_private_codegen_tests.rs:171-176` asserts this directly. Public `dist`/`fsum`/`sumprod` keep `ParamConvention::borrow()` (`stateless_private_codegen_tests.rs:160-170`), so list ownership does not leak into the public API even though the private bridges in `stdlib/_sifr/math.sifr:195-205` take `own list[float]` to forward owned `Vec<f64>` to the Rust helpers.

- **No private-vs-public Rust name collision in generated code.** `_sifr.math` emits Rust wrappers (`dist_impl`, `fsum_impl`, `sumprod_impl`, plus `sqrt`, `floor`, …) that forward to `sifr_stdlib::math::*`; `sifr.math` emits the public Sifr functions (`dist`, `fsum`, `sumprod`, `factorial`, `_copy_float_list`, etc.). The wrapper/bridge names never collide. `stateless_private_codegen_tests.rs:127-130` verifies the private code contains `sifr_stdlib::math::sqrt(x)`/`pow_val`/`floor`/`frexp`.

- **Bootstrap/re-export policy change is appropriately narrow.** `export_policy.rs:2-6` adds one literal `(module, name)` early-return; the existing `_`-prefix rule and the heapq carve-out (`export_policy.rs:7-15`) are untouched. The filter cannot accidentally hide names from `sifr.heapq` or any module other than `sifr.math`.

- **No new user-triggerable panic paths in math.** `crates/sifr_stdlib/src/math.rs` uses only IEEE-total f64 operations, `SifrIntBridge::to_i64_saturating()`, and `as i64` (NaN→0, ±inf→i64 saturation). `nextafter` (line 360-378) excludes NaN up front, handles `x == y` and `x == 0.0`, and `bits ± 1` cannot underflow `u64` from non-NaN inputs (the smallest reachable bit pattern is `1`, used only for the `x == 0.0` branch). `ulp` (line 381-393) excludes NaN/inf/zero/`±f64::MAX` before calling `nextafter`. `gamma`/`lgamma` short-circuit the non-positive integer pole. `lanczos_sum` (line 269-275) iterates over fixed coefficients with non-zero positive denominators for x in the post-shift domain.

- **Feature/dependency planning is minimal and complete.** `features/generated_stdlib_features.rs:34` maps both `sifr.math` and `_sifr.math` to `["math"]`. `features_tests.rs:198-202` asserts the plan excludes `json|regex|http|python`, and `features_tests.rs:226` asserts `sifr.math` emits exactly one `sifr_stdlib` dependency with `features = ["math"]` and `default-features = false`. The e2e fixture harness mirrors this at `fixture_dependency_paths.rs:65-67` and `fixture_cargo_toml.rs:343-346`; `harness_behavior_tests.rs:516-539` covers the combined-features ordering.

- **Compiler intrinsic dispatch deletion is complete.** `registry.rs` has zero references to `math::` and zero math leaf names; `grep -rn "math::lower_\|intrinsic_math" crates/sifr_codegen/` returns nothing. `registry_core_tests.rs:33-51` and `registry_extended_tests.rs:639-661` collectively assert that representative core and extended math names return `None` from `lower_intrinsic`. The retained `intrinsic_math()` entry in `sifr_stdlib_model/src/lib.rs:147-149` is correctly documented as bootstrap-fallback-only.

- **Architecture doc** (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:393-407`) accurately records the wave including the borrowed-public/owned-private boundary rationale.

## VERDICT: PASS
