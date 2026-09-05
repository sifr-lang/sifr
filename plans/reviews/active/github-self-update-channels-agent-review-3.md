## Review pass 3: GitHub-hosted self-update channels

### Pass-2 items — status

- **B4 (dual-path race):** Resolved. `create_new_version.sh:137` hard-rejects any `--mutation-mode` other than `local`, default is `local` (`:43`), help text says "GitHub publication is handled by preview-release.yml" (`:30`), and `internal_docs/distribution_pipeline.md:294-297` names `preview-release.yml` as the only authoritative GitHub-publish path with `trigger_preview_release.sh` as the operator entry. Dual-path race surface eliminated.
- **H4 (silent bootstrap fallback):** Resolved. `.github/workflows/preview-release.yml:191` now emits `::warning::channels release asset fetch failed; bootstrapping metadata from public GitHub prereleases` before the bootstrap branch runs.
- **H5 (OLD-binary migration):** Resolved. `docs/self_update.md:59-70` adds a "Migrating Earlier Previews" section telling users to bridge with `--version <new-preview>` or re-run the installer.
- **H6 (one-sided post-publish verification):** Resolved by design. Workflow lines 285-325 parse the locally-generated `channels.json`, then for **both** alpha and beta run `gh release view ${channel_version} --json assets` and assert `sifr-installer-${channel_version}` exists with size ≥ 1024 bytes before the channels release is touched. This also catches stale prereleases whose installer asset is gone.
- **M2 (bootstrap downgrade protection):** Resolved. `bootstrap_channel_metadata.py:36-39` rejects `version_key(args.version) < version_key(existing)`, and the new fixture `channel_metadata_bootstrap_from_github_releases.sh:71-77` asserts the error string. `0.1.0-beta.7+ignored` in the same fixture exercises the `fullmatch` rejection of build metadata.

### Remaining blocker

**B5. `.github/workflows/preview-release.yml` is not valid YAML.** Three inline Python heredocs (lines 204-217, 289-301, 308-324) place the heredoc body and closing `PY` at column 0. The enclosing `run: |` block scalar's content indent is 10 spaces (first content line `set -euo pipefail`). YAML 1.2's literal block scalar terminates when a non-empty line is less indented than the block indent indicator. Verified:

- PyYAML: `could not find expected ':' while scanning a simple key at line 205 column 1`
- Ruby Psych (the parser GitHub Actions uses): same error.

This means GitHub Actions will reject the workflow on dispatch — none of the pass-2-resolved checks (B3, H1, H6) ever runs because the file does not parse. Pass-3 local validation (`bash -n`, `cargo test`, verification suite) does not touch `.github/workflows/preview-release.yml`, so the regression went undetected. A `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/preview-release.yml'))"` step in `scripts/run_all_tests.sh` (or the local validation profile) would have caught it. Affected steps:

- `Generate channel metadata asset` (reads current alpha/beta from existing channels.json)
- `Verify published release assets` outer heredoc (extracts both channel versions)
- `Verify published release assets` inner heredoc (asserts installer asset size ≥ 1024)

Suggested fix shape: move each heredoc to a small standalone script under `scripts/distribution/` or `.github/workflows/scripts/` and call `python3 path/to/script.py "${current_metadata}"`. The heredoc-inline pattern can't be salvaged because `<<'PY'` requires the closing delimiter at column 0, which is incompatible with YAML block-scalar indentation. (`<<-'PY'` strips only leading tabs, and YAML literal block scalars disallow tab indentation.)

### Other observations (not blockers)

- **N1.** `scripts/distribution/generate_dispatchers.sh:251` still echoes `"generated dispatchers and channel metadata under ${INSTALL_ROOT}"` even though the channel-metadata generation was extracted to `generate_channel_metadata.sh`. Cosmetic; drop "and channel metadata".
- **N2.** Bootstrap fallback still has a residual silent-regression risk for the *other* channel: if the curl fetch of `channels.json` fails transiently AND the other channel's most-recent prerelease was deleted from `gh release list`, the bootstrap output silently downgrades the un-published channel. The H4 warning makes this visible in the run log but does not prevent it. Acceptable for now since the warning surfaces it; a future hardening would compare gh-release-derived "other channel" against the prior `channels.json` (e.g., via a separate sticky cache) before writing.
- **N3.** `crates/sifr/src/self_update_metadata.rs:88-93` carries the cosmetic carry-over from pass 2 M4 (`format!(..., self.text, self.text)`). Two-arg interpolation is functionally fine; readability nit only.

### Bottom line

The four pass-2 items called out for this round (B4, H4, H5, H6) and the M2 downgrade-guard are all correctly implemented. **B5 is merge-blocking**: the GitHub Actions workflow YAML does not parse, so the entire publication path — including all the verification gates resolved in pass 2 — is dead on arrival. Extract the three inline Python heredocs to standalone scripts (or otherwise restructure them) before merge, and add a YAML-parse step to local validation so this can't regress.

(Pass-3 review file `plans/reviews/active/github-self-update-channels-agent-review-3.md` left empty per the no-modify instruction.)
