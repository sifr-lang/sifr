# Sifr M6 Sysroot Distribution — Code Review

## Blocking findings

### B1. Self-update validator forces `/bin` layout but installer still writes a "flat" receipt — silent break for any non-`/bin` SIFR_INSTALL_DIR

`crates/sifr/src/self_update_receipt.rs` (the new block around the inserted `sysroot_path` validation in `validate_receipt_eligibility`):
```rust
let sysroot_path = canonicalize_for_receipt(Path::new(&receipt.sysroot_path), "sysroot_path")?;
let expected_binary_parent = sysroot_path.join("bin");
if !paths_same_after_canonicalization(&expected_binary_parent, binary_parent) {
    return Err(unmanaged_receipt_diagnostic(format!(
        "standalone install receipt binary_path {} is not paired with sysroot_path {}",
        receipt.binary_path, receipt.sysroot_path
    )));
}
```

But `scripts/distribution/generate_version_installer.sh` happily picks a flat layout when the user-supplied `SIFR_INSTALL_DIR` does not end in `/bin`:
```
default_sysroot_dir=""
if [ "$(basename "${install_dir}")" = "bin" ]; then
  default_sysroot_dir="$(dirname "${install_dir}")"
else
  default_sysroot_dir="${install_dir}"
fi
...
sysroot_dir="${SIFR_SYSROOT_INSTALL_DIR:-${default_sysroot_dir}}"
```
And `manifest_dir` mirrors the same fall-through. So for `SIFR_INSTALL_DIR=$HOME/sifrbin` (no `/bin`):
- `binary_path = $HOME/sifrbin/sifr`, `binary_parent = $HOME/sifrbin`
- `sysroot_path = $HOME/sifrbin`, `expected_binary_parent = $HOME/sifrbin/bin`
- Validation always fails → `sifr self update` reports the official install as "unmanaged".

This is a real regression: the prior docs/examples used `SIFR_INSTALL_DIR="$HOME/bin"`. The docs were updated to `$HOME/.local/sifr/bin` but the installer does not enforce the new contract, so an existing installation following the old example, or anyone calling the installer with a custom path that doesn't end in `/bin`, will install successfully and then be locked out of self-update with a misleading "unmanaged" diagnostic.

Pick one resolution before merge:
- Have the installer hard-fail when `SIFR_INSTALL_DIR` does not end in `/bin` (and document the contract clearly), or
- Relax the validator: when `sysroot_path == binary_parent`, accept the flat layout. The other receipt invariants (sysroot.toml, schema version, target/version pairing) still hold.

Either way add a verification case for the chosen behavior — none of the new cases exercise the non-`/bin` install.

### B2. `sysroot_content_sha256` is a hard-coded zero string everywhere, defeating its integrity purpose

`scripts/distribution/build_preview_artifacts.sh` → `write_sysroot_manifest`:
```bash
"sysroot-content-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
```
The installer reads this value verbatim and copies it into `install.json`. The schema `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json` accepts any 64-hex string, so the constant validates. Every shipped receipt will carry the same all-zero hash, and the test in `artifact_self_update_receipt_rules.sh` only checks length, not content.

This makes a documented integrity field pure ceremony — receipts can't be cross-checked against the sysroot they claim to belong to, and any future code that does verify the hash will need a coordinated migration. Either:
- Compute the real hash now (sorted-file SHA over the staged sysroot tree, excluding the manifest itself), wire it into both `build_preview_artifacts.sh` and the runtime side, and add a test that recomputes and compares; or
- Drop the field from schema v2 until M-* fills it in, rather than freezing a placeholder into a versioned schema.

The current "schema v2 with placeholder hash" state is the worst of both: the version bump makes it harder to fix later without another schema bump.

## Non-blocking suggestions

### N1. `validate_archive_listing` relies on `set -e` propagating a subshell `exit` through a pipe
`scripts/distribution/generate_version_installer.sh`:
```sh
tar -tzf "${archive_path}" | while IFS= read -r member; do
  case "${member}" in
    ""|/*|../*|*/../*|*/..|..)
      fail "artifact ${archive_name} contains unsafe path ${member}"
```
The `while` runs in a subshell, so `fail`'s `exit 2` only kills that subshell; safety depends on `set -e` honoring the pipeline status. Add `set -o pipefail` near the top of the generated installer, or refactor with process substitution / a temp file, so the failure path is explicit rather than incidental.

### N2. Stub `rollback_install_transaction() { :; }` is dead code
The early stub above the `trap cleanup` block is shadowed by the real definition before any code can set `rollback_active=1`. It's safe today but confusing — drop the stub and move the real definition above the trap.

### N3. Receipt parser does not enforce format of new fields
`parse_install_receipt_json` accepts any string for `sysroot_target_triple` and `sysroot_content_sha256`. The schema constrains them at distribution-time, but on the consumer side a hand-edited receipt can have arbitrary values and still pass parsing. A cheap hex-charset check on `sysroot_content_sha256` (and a non-empty check on `sysroot_target_triple`) would make the unmanaged-receipt diagnostics fire earlier and more precisely.

### N4. `channel_dispatcher_points_to_generated_installer.sh` assigns `install_root` twice
First `install_root="${tmp_dir}/install-root"` is immediately overwritten by `install_root="${tmp_dir}/installed"`. Delete the first assignment.

### N5. Test coverage gaps worth filling alongside this PR
- No verification case for `SIFR_INSTALL_DIR` that does not end in `/bin` — whichever resolution B1 picks, add a case that exercises it.
- No case that mutates `sysroot.toml` after install and shows `sifr self update` flags it as unmanaged (today the sysroot-pairing checks rely on the file just being present).
- `artifact_broken_sysroot_archives_rejected.sh` exercises missing assets but not the `verify_release_archive.py` symlink / absolute-path / `..` rejection branches that are now part of the contract.

### N6. Validation gap acknowledged by the implementer
The report flags that full `scripts/run_all_tests.sh --profile create-pr` did not complete cleanly because of an unrelated runner-collision artifact. The focused suites listed are reasonable, but per project guardrails the `create-pr` profile is the authoritative gate. Run it once cleanly (no overlapping runners) before merging — this PR touches packaging, installer, and self-update on the same change and is exactly the surface that the profile is designed to catch.

### N7. `COMPILER_SIFR_VERSION` now sourced from a build script
`crates/sifr_sysroot/build.rs` is fine, but the new `build-script-build` target is also being classified in `verification/areas/coverage_matrix/data/cargo_metadata_classification.json` as `internal_codegen_tool`. Confirm that `sifr_sysroot` build-script execution is gated by the same workspace-level build-script policy as other first-party crates — the classification entry only documents it.

## Verdict

**FAIL** — B1 is a real behavior regression for any custom-dir install that does not end in `/bin`, and B2 freezes a zero-valued integrity hash into a newly bumped schema. Both should be resolved (or B2 explicitly deferred by removing the field from schema v2) and the `create-pr` profile re-run to completion before merging.
