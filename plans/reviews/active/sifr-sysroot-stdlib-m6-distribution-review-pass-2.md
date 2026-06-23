# Sifr M6 Sysroot Distribution — Code Review (Round 2)

## B1 (validator vs. flat custom layout) — resolved

`crates/sifr/src/self_update_receipt.rs` now accepts either pairing under canonicalization:
```rust
if !paths_same_after_canonicalization(&expected_binary_parent, binary_parent)
    && !paths_same_after_canonicalization(&sysroot_path, binary_parent)
```
The installer mirrors the contract: when `SIFR_INSTALL_DIR` does not end in `/bin`, both `default_sysroot_dir` and `manifest_dir` fall back to the install dir, and the receipt's `sysroot_path` is canonicalized to that same dir. `discover_receipt_path` and `derive_install_manifest_path` both bias to `bin_dir.parent()/install.json` when the parent is named `bin`, then fall back to the flat layout. The new `accepts_flat_custom_install_layout` unit test and the second-half block of `artifact_self_update_receipt_rules.sh` (`flat_install_dir`) explicitly exercise the no-`/bin` install path and assert `sysroot_path == install_dir` plus the presence of `sysroot.toml`. Docs (`docs/installation.mdx`, `internal_docs/distribution_pipeline.md`) document the two-layout contract. **Resolved.**

## B2 (zero-valued sysroot_content_sha256) — resolved

The hash is now computed for real, validated end-to-end, and zero is explicitly rejected at every layer:

- `build_preview_artifacts.sh` adds `sysroot_content_sha256()` over a sorted file list (`Cargo.toml Cargo.lock .cargo/config.toml crates lib vendor`) and writes it into `sysroot.toml` before tarring (the manifest itself is excluded from the input set).
- `scripts/distribution/verify_release_archive.py` recomputes the digest from archive contents and aborts on mismatch or zero placeholder. It is invoked from `build_preview_artifacts.sh`, `generate_version_installer.sh`, and the preview-release workflow.
- `is_sha256_hex` (Rust) requires 64 lowercase hex chars and rejects all-zero; `sha256_field` enforces it at parse time, and `validate_receipt_eligibility` re-checks at use time.
- `self_update_install_receipt.schema.json` constrains the field with `"pattern": "^[0-9a-f]{64}$"` plus `"not": { "const": "0…0" }`.
- Tests cover all branches: `rejects_malformed_sysroot_content_sha256` (parse), `artifact_self_update_receipt_rules.sh` (asserts the receipt's hash equals `tomllib`-parsed `sysroot-content-sha256` from the installed manifest, and rejects zero), `artifact_broken_sysroot_archives_rejected.sh` includes both `zero_digest` and `mismatched_digest` cases.

The Bash producer and Python verifier emit the same `<path>\n<file-hash>\n` stream over the same sorted file set, so cross-tool agreement is structural rather than coincidental. **Resolved.**

## Previous low-cost cleanups

- **N1 (pipefail in archive listing):** Replaced the `tar -tzf | while` subshell with `tar -tzf > listing; while < listing` in `generate_version_installer.sh::validate_archive_listing` — the `fail`'s `exit` now happens in the main shell.
- **N2 (rollback stub):** Single `rollback_install_transaction()` defined once above `trap cleanup`; no stub-then-shadow.
- **N4 (duplicate assignment):** `channel_dispatcher_points_to_generated_installer.sh` now has one `install_root="${tmp_dir}/installed"` with `install_dir="${install_root}/bin"` derived from it.

All resolved.

## New non-blocking observations

1. **Schema version bump is a hard cut.** `schema_version: 1` receipts (the prior beta releases) now hit `SELF_UPDATE_UNMANAGED_RECEIPT` and users must re-run the installer. The diagnostic does point at the install command, so it's a guided migration, but worth a one-line note in release notes since any user currently on beta.1–beta.11 will be told their install is unmanaged.
2. **`sysroot.toml` content integrity isn't checked at update time.** The validator only verifies the file *exists* and that the receipt's hash field is well-formed; it never recomputes the on-disk sysroot tree hash against `sysroot_content_sha256`. A user who hand-edits `sysroot.toml` (or whose disk corrupts the tree) still passes validation. Earlier review's N5 sub-bullet about mutating `sysroot.toml` after install is therefore still uncovered. Cheap follow-up: have `validate_receipt_eligibility` recompute and compare, or at minimum re-parse the on-disk `sysroot.toml` and check its `target-triple` / `sifr-version` against the receipt.
3. **`is_sha256_hex`'s explicit zero rejection is a leaky sentinel.** Any actual sha256 is astronomically unlikely to be all-zero, but technically a hash of 64 `0`s is valid hex. Keep it — it's a guard against the *placeholder*, not the value — but the comment-free `&& value != "0…0"` reads as ad-hoc. A one-line `// reject the placeholder used during M6 transition` would explain it to future readers (and `internal_docs/distribution_pipeline.md` could mention the deny-list).
4. **Redundant `backup_path` calls.** `backup_managed_toolchain` already moves every managed path to `backup_root`; the subsequent `replace_sysroot_path` and `install_binary_from_stage` each call `backup_path` again, which is a no-op because the destination is empty. Harmless but worth a comment to make the intent (idempotent rescue if either function ever runs alone) explicit.
5. **`create-pr` profile gate (prior N6) still unconfirmed.** The current validation report lists `self_update`, `distribution_release` representative, `typescript-go-transfer`, and `sifr_sysroot` suites — but not a clean `scripts/run_all_tests.sh --profile create-pr` run. Per project guardrails this is the authoritative gate; please confirm one clean pass before merging.
6. **Untracked files in PR.** `crates/sifr_sysroot/build.rs`, `scripts/distribution/verify_release_archive.py`, and `verification/areas/distribution_release/cases/artifact_broken_sysroot_archives_rejected.sh` are still untracked in the working tree. They need to be `git add`ed before the PR exists in a reproducible state.

## Verdict

**PASS** — Both blocking findings are resolved with end-to-end coverage. Remaining observations are non-blocking; the only one I'd act on before merge is running the `create-pr` profile once cleanly and ensuring the three untracked new files are staged.
