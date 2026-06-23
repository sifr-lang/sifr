# M9 Wave 1 — `_sifr.platform` / `_sifr.html` migration review (pass 2)

Scope: re-review of the uncommitted diff after the pass-1b follow-ups (doc
comment + negative test for sysroot probe feature derivation; restored
stdlib-model platform/html entries with explanatory comments; phase plan
records pass-1b PASS and the retained-bootstrap rationale).

## Follow-up verification

### Follow-up 1 — doc comment and negative test for sysroot probe feature derivation

- `crates/sifr_driver/src/build/rust_interop_probe.rs:185-194` adds a doc
  comment on `crate_feature_exists` explaining that undeclared path segments
  are treated as "no feature" on purpose, so sysroot-interop tests can use
  minimal temp crates and future flat targets can still probe.
- `sysroot_stdlib_probe_features_ignore_undeclared_target_segment`
  (`rust_interop_probe.rs:548-562`) exercises the "second segment is not a
  declared `sifr_stdlib` feature" path against the real `sifr_stdlib`
  manifest and asserts the feature list is empty.
- Caveat (non-blocking): the doc comment lives on `crate_feature_exists` rather
  than on `dependency_features` (where pass 1b suggested it). Both functions
  are small and adjacent, so this is fine, but `dependency_features` is the
  public-ish entry point and would have been the more discoverable home.

### Follow-up 2 — retained stdlib-model platform/html intrinsic entries

- `crates/sifr_stdlib_model/src/lib.rs:149-151,163-165` keeps
  `_sifr.platform` / `_sifr.html` entries in `get_intrinsic_module` with
  explicit "Retained as a stdlib-lowering bootstrap fallback while M9 migrates
  these leaves to compiled private declarations" comments.
- `crates/sifr_stdlib_model/src/platform_misc.rs:27,135` updates the
  `intrinsic_platform()` / `intrinsic_html()` docstrings to "bootstrap
  signatures" wording and drops the now-misleading per-function comments
  describing the lowered shape.
- In the canonical control flow (private declarations now compile with
  exported functions), `compile_stdlib_sources_with_sysroot`
  (`crates/sifr_driver/src/stdlib/bootstrap.rs:167-218`) hits the
  `has_compiled_exports` branch first and the retained intrinsic entries are
  dead. The risk pass 1b flagged (signature drift if the fallback ever fires)
  is unchanged, but now visibly called out at the registration site. The
  combination of the comment at the registration site and the updated
  docstrings on the builders is sufficient.

### Follow-up 3 — phase plan updates

- `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md:597-643` now
  records: wave-1 scope, the rationale that `sifr_codegen` no longer routes
  these leaves through active intrinsic dispatch, the retained
  stdlib-lowering bootstrap signatures and the explicit fact that those are
  not active codegen lowerers, the focused validation suite, and the pass-1b
  PASS with non-blocking items addressed.
- `internal_docs/sifr_sysroot_and_stdlib_architecture.md:393-397` adds the
  current migrated leaves to the architecture doc.

## Diff-wide spot checks

- Active codegen no longer registers platform/html as intrinsics:
  `crates/sifr_codegen/src/intrinsics/registry.rs:14,23,558-562,791-792`
  drops the `mod html;` / `mod platform;` declarations and all lowering arms;
  registry tests
  (`registry_core_tests.rs:817-826`, `registry_extended_tests.rs:357-361`)
  assert these names are now unhandled.
- Public-routing tests assert no intrinsic registration:
  `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:11-27`
  and `:31-55` verify `_sifr.{platform,html}` have empty intrinsic-name sets
  and that the public modules' intrinsic sets do not contain
  `platform_system` / `html_escape`, while still depending on the private
  modules through `transitive_deps`.
- `crates/sifr_codegen/src/module_prescan.rs:30-51` correctly preserves
  legacy behaviour for unmigrated `_sifr.*` placeholders (they have no
  `stdlib_intrinsic_names` entry, so the `else` branch still flags all
  imported names as intrinsic). This is the load-bearing fix that allowed
  the per-module dispatch removal without breaking the rest of `_sifr.*`.
- The source-load reordering in
  `crates/sifr_stdlib_model/src/sources.rs:359-381` (privates first, then
  publics) is what allows `has_compiled_exports` in `bootstrap.rs:167-174`
  to find compiled exports for `_sifr.platform` / `_sifr.html` when
  processing the importing public modules.
- `crates/sifr_stdlib_model/src/features/generated_stdlib_features.rs:31-32`
  and `crates/sifr_stdlib_model/src/features_tests.rs:180-186,213-231`
  confirm both leaves emit only `sifr_stdlib = { default-features = false,
  features = ["<leaf>"] }` with no third-party direct dependencies and no
  umbrella feature pollution.
- `crates/sifr_stdlib/src/platform.rs` and `crates/sifr_stdlib/src/html.rs`
  match the lowered shape of the removed intrinsics (string `replace`
  chains; `uname -r` / `-v` fallback to `std::env::consts::OS`; same OS
  string table). The Rust-level `api_behavior` tests cover both leaves.

## Non-blocking observations carried forward

- `dependency_line`
  (`crates/sifr_driver/src/build/rust_interop_probe.rs:154-164`) still omits
  `default-features = false` for probe manifests, while user-facing generated
  `Cargo.toml`s set it (`generated_stdlib_features.rs` path). Harmless today
  because `sifr_stdlib` has `default = []`, but worth tracking as a divergence
  if defaults are ever added.
- The full `scripts/run_all_tests.sh --profile create-pr` suite is still the
  required pre-PR gate per `AGENTS.md`. The task statement enumerates only
  focused tests; the wider suite must still run before opening the PR. Not a
  code finding, but listed here for completeness.
- The pass-1b suggestion to add a focused unit test for the new
  `has_compiled_exports` branch in `bootstrap.rs` is still applicable, though
  the end-to-end `stateless_private_codegen_tests` cover the behaviour
  adequately for this wave.

## Verdict rationale

- Follow-up 1 is correctly applied: the assumption is now documented and the
  negative path is tested against the real `sifr_stdlib` manifest.
- Follow-up 2 leaves the bootstrap fallback in place but with explicit,
  co-located comments and rewritten docstrings so future maintainers
  understand why these entries exist; the canonical control flow no longer
  uses them. Accepted as a transitional state for this wave.
- Follow-up 3 brings the phase plan in line with the migration's actual
  shape and references the pass-1b PASS.
- No new blockers introduced; pass-1b PASS findings remain non-blocking.
- After running `scripts/run_all_tests.sh --profile create-pr`, the wave is
  ready to open as a PR.

VERDICT: PASS
