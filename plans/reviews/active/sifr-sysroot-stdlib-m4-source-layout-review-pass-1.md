## M4 Review

I read the milestone, the architecture target, and walked the changed code/tests/docs. Below are blockers vs. non-blocking suggestions.

### Blockers
**None.** M4 acceptance items map to real implementation and tests:

- Public sources move repo-side `lib/sifr/*.sifr` → `stdlib/sifr/*.sifr`; private placeholders under `stdlib/_sifr/*.sifr`. `lib/sifr/` is gone.
- Layout validation requires `lib/sifr/stdlib/{sifr,_sifr}`, `crates/sifr_runtime/Cargo.toml`, **and** `crates/sifr_stdlib/Cargo.toml`; workspace manifest must declare both members with resolver `"2"` — covered by `layout_validation_checks_all_skeleton_assets`, `workspace_validation_requires_generated_stdlib_member`, and `installed_layout_workspace_supports_offline_cargo_metadata` (`crates/sifr_sysroot/src/tests.rs:182,210,238`).
- `sifr_stdlib_model::{load_stdlib_sources_from_sysroot, validate_stdlib_source_inventory, PRIVATE_STDLIB_MODULES}` enforces public/private inventory against the resolved sysroot, with tests for missing public, missing private, stale private, and source‑tree match (`crates/sifr_stdlib_model/src/sources.rs:701,720,732,746,760`).
- `compile_stdlib_uncached` resolves `ResolvedSysroot`, loads physical files, and passes the physical path to the parser (used by diagnostics/LSP) (`crates/sifr_driver/src/stdlib/bootstrap.rs:24,48`). Failures route through `STDLIB_BOOTSTRAP_FAILURE`, not panics. `is_source_tree_development_mode()` gates dev resolution to `debug_assertions`, preserving the release/dev boundary.
- Analysis hosts route through `sifr_driver::stdlib_external_defs()` so LSP/CLI share the same sysroot (`crates/sifr_analysis/src/host/{implementation,overlay_updates}.rs`); tests renamed in `host/stdlib_tests.rs` and `hover_and_signature_cover_stdlib_calls_inside_try_blocks` still passes.
- CLI `--print sysroot --json` exposes `stdlib_public_sources`, `stdlib_private_sources`, `stdlib_crate`, `stdlib_crate_manifest` (`crates/sifr/src/sysroot_cli.rs:33`; asserted in `crates/sifr/tests/sysroot_cli.rs:80`).
- User `_sifr.*` imports still rejected via `forbidden_intrinsic` / `IMPORT_FORBIDDEN_INTRINSIC` (unchanged).
- No stale `lib/sifr` references remain in code/docs outside test fixtures asserting the installed-layout path.

### Non-blocking suggestions

1. `STDLIB_SOURCES` still embeds every public stdlib file via `include_str!`. Only `.module` is consumed at runtime (`is_bare_stdlib_tail` and friends); `.source` is never read by the bootstrap. The arch explicitly says "Released tools must not use embedded fallback sources." Shrink to a `STDLIB_MODULE_NAMES: &[&str]` (or keep `module` only) in a follow-up to drop the embedded copy.
2. `stdlib_source_root` (`crates/sifr_sysroot/src/layout.rs:102`) picks between `<root>/stdlib` (repo) and `<root>/lib/sifr/stdlib` (installed) by filesystem probe. Works today; a stray `stdlib/` in an installed sysroot would silently win. Consider deciding mode once at resolution and storing it on `ResolvedSysroot`.
3. Coverage gap: `validate_module_files` is symmetric, but only stale‑private and missing‑public/private are tested. Add a stale‑public case and a duplicate‑module case (`validate_unique_modules`) for parity.
4. `PRIVATE_STDLIB_MODULES` has `_sifr.io` and `_sifr.test`, which the architecture's "private wrappers" table does not list (they have no public importers yet). Either drop them until they back something, or add a one‑line note in the arch doc that placeholders exist for unwrapped intrinsics.
5. `validate_module_files` returns "stdlib source filename is not valid UTF-8" without saying which root (public vs private) it found — small polish for diagnostic clarity.
6. As you flagged: `crates/sifr_driver/src/build/materialize.rs` adds `command.env_remove("CARGO_TARGET_DIR")`. Independent fix; should not ride with M4.

### Verdict
**review-satisfied.**
