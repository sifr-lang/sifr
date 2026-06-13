I've reviewed all the new and changed files against the M4 contract. Here is my assessment.

# READY (with non-blocking polish notes)

The M4 implementation satisfies every "definition of done" item in the contract:

- ✅ Distribution validation wired in through the existing `verification/distribution/*.sh` glob in `scripts/run_distribution_validation.sh`.
- ✅ Validator (`scripts/distribution/validate_self_update_metadata.sh`) checks schema, channel allowlist, ordering, semver prerelease label per channel, dispatcher↔dispatcher cross-consistency, metadata↔dispatcher agreement, and immutable installer `APP_VERSION`.
- ✅ One-plan flow: `create_new_version.sh` real-run calls `generate_version_installer.sh` then `generate_dispatchers.sh` (which writes both dispatchers and `metadata/channels.json`).
- ✅ Dry-run prints `channel_metadata=…` and `channel_metadata_update=alpha:…,beta:…` mutation lines; asserted by `create_new_version_alpha_dry_run.sh`.
- ✅ Real-run fixture validates dispatcher, metadata, and immutable installer agreement end-to-end and then runs the new validator against the mutated install root.
- ✅ Stable metadata rejected at validation time (`channel_metadata_stable_rejected.sh`) and at dispatcher generation (`generate_dispatchers.sh`/`create_new_version.sh` reject stable-looking versions).
- ✅ APP_VERSION extracted from immutable installer and cross-checked vs metadata (and transitively vs dispatcher, since metadata↔dispatcher is asserted in the same run).
- ✅ `internal_docs/distribution_pipeline.md` updated with the new drift validation surface.

## Non-blocking polish notes

1. **Validator's Python error diagnostics get clipped on stderr.** `scripts/distribution/validate_self_update_metadata.sh:59-100` captures only stdout from the heredoc, so when `SystemExit("…")` fires, the message lands on stderr while `metadata_values` is empty — then `|| fail "${metadata_values}"` prints an empty `self-update metadata validation:` line. The tests still pass because `require_failure_contains` reads `2>&1`, but the bash wrapper line is dead noise. Consider running python without command substitution (e.g., `python3 - "${metadata_path}" <<'PY' … PY || fail "channel metadata validation failed"`) so the Python message stands alone.

2. **Redundant prerelease label check.** `validate_installer` re-runs `preview_channel_for_version` against the metadata version (`validate_self_update_metadata.sh:142`) even though the Python regex already enforces the prerelease label for that channel. Harmless but dead code; consider removing or relocating it to validate the *installer*'s embedded version against the channel label rather than the metadata's.

3. **`schema_version != 1` accepts `True`.** Python's `True == 1` means `{"schema_version": true}` would slip past `validate_self_update_metadata.sh:76`. Edge case unlikely in practice (your generator only writes the literal `1`), but `if metadata.get("schema_version") is not 1 or isinstance(metadata["schema_version"], bool)` (or a `type(…) is int` check) would close it.

4. **Seeded drift coverage is intentionally narrow.** The 4 fixtures cover (dispatcher drift, installer drift, stable, agreement). The validator catches more cases (missing file, unknown non-stable channel, schema mismatch, ordering, label-vs-channel mismatch), but those branches are exercised only by reading the code. Contract just says "every seeded drift class" — what you seed is caught — so this is in spec, but you may want a fifth seeded test covering "metadata file missing" or "unknown channel `rc` in metadata" for completeness in M5 if/when other drift classes appear.

5. **`make_self_update_install_root_fixture` deletes its artifact dir.** `verification/distribution/common.sh:157` removes `${artifact_dir}` after generating installers, but the immutable installer at `${install_root}/versions/<v>` has `ARTIFACT_BASE_URL=file://${artifact_dir}/<channel>` baked in. Drift validation doesn't run the installer so this is intentional and fine, but a one-line comment in `common.sh` would save the next maintainer from puzzling over the dangling URL.

None of the above blocks merge. Recommend READY.
