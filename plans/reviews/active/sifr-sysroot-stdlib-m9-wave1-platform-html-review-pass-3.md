Review complete. Here's the verdict.

---

## VERDICT: PASS

The M9 wave 1 migration of `_sifr.platform` and `_sifr.html` from active compiler intrinsics to private `@rust(sifr_stdlib.*)` declarations is correctly scoped, behavior-preserving, and consistent with the architecture/phase docs. The accompanying Python-runtime native-link / CARGO_TARGET_DIR fix is also correctly implemented.

### Key correctness checks

- **Routing is end-to-end**: `stdlib/_sifr/{platform,html}.sifr:1-25` declare `@rust(sifr_stdlib.*)` leaves → `crates/sifr_stdlib/src/{platform,html}.rs:1-54` own the Rust impls behind narrow leaf features → `crates/sifr_stdlib/Cargo.toml:96-97` adds `html`/`platform` as empty-deps features → `crates/sifr_stdlib_model/src/features/generated_stdlib_features.rs:31-32` maps both public/private module names to those features.
- **Active intrinsic dispatch removed**: `crates/sifr_codegen/src/intrinsics/registry.rs:14,23,558-562,791-792` deletes the `mod html;`/`mod platform;` declarations and lowering arms; `registry_core_tests.rs:817-826` and `registry_extended_tests.rs:357-361` assert `lower_intrinsic(...)` now returns `None` for the migrated names.
- **Compiled-exports branch wins over legacy fallback**: bootstrap rewrite at `crates/sifr_driver/src/stdlib/bootstrap.rs:167-218` prefers compiled exports; ordering load-bearing change in `crates/sifr_stdlib_model/src/sources.rs:359-381` puts privates first so the compiled lookup is populated when public wrappers iterate. `stateless_private_codegen_tests.rs:11-55` covers the end-to-end shape (private Rust references `sifr_stdlib::{platform,html}::...`, intrinsic-name set is empty for `_sifr.{platform,html}`, transitive deps wire correctly).
- **Transitive `_sifr.*` deps reach codegen**: `crates/sifr_codegen/src/lib_modules_and_codegen.rs:313-320` extends the dep-walk to include `_sifr.*` so private preamble code is actually emitted.
- **Prescan preserves legacy behavior for unmigrated `_sifr.*`**: `crates/sifr_codegen/src/module_prescan.rs:30-51` keeps the `else` branch treating un-keyed imports as intrinsic.
- **No user-triggerable panics introduced**: `crates/sifr_stdlib/src/platform.rs` uses `.ok()` / `.unwrap_or_else` on errors only; `html.rs` only `.replace()` chains; trust label `trusted_no_panic` matches the actual code shape (pre-existing behavior of `uname` swallowing errors is preserved).
- **No user-package trust expansion**: `crates/sifr_driver/src/build/python_runtime.rs:99-118` only adds the *selected packaged* Python runtime's `libpython` link to the trusted set; `materialize.rs:289-303` unions it with explicit interop trust requirements and nothing more. Architecture doc `internal_docs/sifr_sysroot_and_stdlib_architecture.md:400-404` correctly scopes this trust.
- **CARGO_TARGET_DIR normalization is correct**: `verification/areas/python_interop/runner/env.py:28-35` resolves relative `CARGO_TARGET_DIR` against `repo_root` before the subprocess switches cwd to the package; applied in both `example_packages.py:245` and `live_examples.py:225`.
- **Probe feature derivation hardened**: `rust_interop_probe.rs:166-197` doc-comments the "undeclared segment → no feature" intent and is covered by both positive (`sysroot_stdlib_probe_features_follow_target_module_segment`) and negative (`sysroot_stdlib_probe_features_ignore_undeclared_target_segment`) tests against the real `sifr_stdlib` manifest.
- **No generated direct third-party deps**: `features_tests.rs:213-231` (`stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies`) asserts both leaves emit only `sifr_stdlib = { ..., default-features = false, features = ["<leaf>"] }`.

### Non-blocking follow-ups

1. **Legacy `intrinsic_platform()`/`intrinsic_html()` bootstrap fallback retained**: `crates/sifr_stdlib_model/src/lib.rs:149-151,163-165` keeps the entries with explicit "transitional bootstrap fallback" comments. Since the codegen lowering arms are gone, if the fallback ever fires it would fail to build (not silently miscompile) — but the dual registration is still a maintenance hazard. Either delete in a follow-up wave once all M9 leaves migrate, or add a regression test forcing the compiled-exports branch.
2. **`dependency_line` probe manifests omit `default-features = false`** (`rust_interop_probe.rs:154-164`), while user-facing generated `Cargo.toml` sets it. Harmless today because `sifr_stdlib`'s `default = []`, but the divergence will bite if any default is ever added.
3. **Doc-comment placement**: the `crate_feature_exists` doc comment (`rust_interop_probe.rs:183-185`) lives on the helper rather than on the public-ish `dependency_features` entry point. Fine, but `dependency_features` would have been more discoverable.
4. **Final pre-PR gate**: per `AGENTS.md`, `scripts/run_all_tests.sh --profile create-pr` is still the authoritative gate before opening the PR — the task statement covers focused tests only.
