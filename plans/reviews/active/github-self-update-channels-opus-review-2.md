## Review pass 2: GitHub-hosted self-update channels

### Previous blockers — status

- **B1 (concurrency guard):** Resolved. `.github/workflows/preview-release.yml:27-29` adds `concurrency: { group: preview-release-channels, cancel-in-progress: false }`. Both alpha and beta dispatches serialize against the shared `channels` release.
- **B2 (bootstrap brittleness):** Resolved within the documented contract. `scripts/distribution/bootstrap_channel_metadata.py:34-41` overrides the *current* channel with `VERSION` and only aborts when the *other* channel is unknown. The first-time precondition is now documented at `internal_docs/distribution_pipeline.md:80-97`.
- **B3 (no agreement check on workflow):** Resolved. The workflow now runs (a) `Verify immutable installer asset` (size + shebang) before publish, then (b) `Verify published release assets` (parses `gh release view --json assets`, asserts the installer is present and ≥1024 bytes) before the `channels` release is touched. Ordering is correct: VERSION release → asset-set check → installer check → prerelease publish → published-asset check → channels.json publish.

### Previous high-risk — status

- **H1 (installer verification):** Resolved. `.github/workflows/preview-release.yml:231-245` mirrors the runner's two checks.
- **H2 (GitHub vs. site drift):** **Not fully resolved — see new B4 below.**
- **H3 (stale `metadata/channels.json` on site):** Partially resolved — documented at `internal_docs/distribution_pipeline.md:30`, and fixtures cleanly remove it (`common.sh:169-170`), but there is no automated site-side enforcement.

### Remaining blockers

**B4. Dual-path race: `create_new_version.sh --mutation-mode github` vs. the GitHub Actions workflow.** The concurrency group only serializes workflow runs against other workflow runs. `scripts/distribution/create_new_version.sh:145-169` reads `CURRENT_ALPHA`/`CURRENT_BETA` from the *site dispatcher files* — it never curls the GitHub `channels.json`. So the two paths have different sources of truth:

- Workflow: GitHub `channels.json` is source of truth (curl → modify slot → upload).
- Script with `--mutation-mode github`: site dispatchers are source of truth (read site `index`/`alpha`/`beta` → modify slot → upload).

If the workflow publishes `{alpha: a5, beta: b4}` and the site dispatchers still read `{alpha: a4, beta: b4}`, an operator running `create_new_version.sh --channel beta --version b5 --mutation-mode github` will overwrite GitHub channels.json with `{alpha: a4, beta: b5}`, silently dropping the workflow's alpha bump. The default `MUTATION_MODE="github"` (`create_new_version.sh:43`) makes this the path of least resistance. Either:
- have the script curl GitHub `channels.json` for its baseline (same shape as the workflow), or
- change the default to `local` and document the workflow as the only authoritative GitHub-publish path, or
- explicitly forbid `--mutation-mode github` in `internal_docs/distribution_pipeline.md` and remove the `publish_github_release` branch.

The docs at lines 263–296 describe both paths as if interchangeable; pick one or reconcile them.

### High-risk findings

**H4. Bootstrap falls back silently on any curl error.** `.github/workflows/preview-release.yml:188-201` treats every `curl -fsSL` failure (404, 5xx, DNS, TLS) as "channels.json does not exist" and silently switches to deriving state from `gh release list`. In the steady state, bootstrap should never fire — but a transient 5xx or CDN blip would silently re-derive channels.json from `gh release list` and overwrite the live state. Add an `echo "::warning::falling back to bootstrap because channels.json fetch failed"` (or differentiate 404 from other failures via `curl -w "%{http_code}"`) so a misfire surfaces in the run log. Currently you cannot tell from a green run whether the read-modify-write was performed or the bootstrap path fired.

**H5. Migration of pre-existing OLD-URL binaries is not addressed.** OLD binaries shipped before this PR resolve metadata at `https://sifr.sh/install/metadata/channels.json` and installers at `https://sifr.sh/install/versions/<version>`. After the documented site cleanup at `internal_docs/distribution_pipeline.md:30`, channel-based self-update breaks for every OLD binary in the wild. Version-pinned self-update (`--version 0.1.0-beta.X`) still works because `versions/<version>` remains. Decide and document one of: (a) keep `metadata/channels.json` on the site for a deprecation window, (b) put a static-site redirect from the old URL to the GitHub URL, or (c) communicate to users that `--version` must be used to bridge. As written, the site cleanup is silent client breakage.

**H6. Post-publish verification does not validate the *other* channel's installer is still present.** `.github/workflows/preview-release.yml:284-308` only checks `sifr-installer-${VERSION}`. If a prior preview release lost its installer asset (manual deletion, mis-fired cleanup), publishing the new `channels.json` cements a broken pointer for the un-bumped channel and the workflow won't notice. Defense-in-depth: parse the local `channel-metadata/channels.json`, then for both `alpha` and `beta` `gh release view <version> --json assets` and assert each `sifr-installer-<version>` exists and is ≥1024 bytes before the `Publish channel metadata` step.

### Medium

- **M1. `create_new_version.sh` does not curl GitHub channels.json.** Even if B4 is resolved by docs, the validator (`validate_self_update_metadata.sh`) at `:382-384` runs against the locally-generated `channels.json`, not the *currently-published* GitHub one. So drift between the local plan and what GitHub will end up with is not visible until after upload.
- **M2. Bootstrap helper accepts a `--channel` mismatched with the release `latest` map.** `bootstrap_channel_metadata.py:35` unconditionally overrides `latest[args.channel] = args.version` after computing the latest from history. That's correct for the "current publication wins" semantics, but there is no check that the override does not *regress* the channel — e.g., publishing `0.1.0-beta.5` after `0.1.0-beta.7` already exists in `gh release list` would silently downgrade the metadata. Add: `if existing := latest.get(channel)` and `version_key(args.version) < version_key(existing)` → SystemExit, unless an explicit `--allow-downgrade` is passed.
- **M3. Carry-over nit: `create_new_version.sh:307-323` still does `gh release create` then `gh release edit` even when the release was just created.** Harmless; redundant call surface.
- **M4. Carry-over nit: `self_update_metadata.rs:88-93` interpolates `self.text` twice in the URL.** `let v = &self.text; format!("{BASE}/{v}/sifr-installer-{v}")` reads cleaner.

### Test gaps

- **T1.** The new `channel_metadata_bootstrap_from_github_releases.sh` covers happy-path and "missing alpha" rejection. Add cases for: (a) zero prereleases at all → reject, (b) only the current channel exists → reject, (c) a `0.1.0-rc.1` tag in the list is correctly ignored, (d) `0.1.0` stable tag is ignored, (e) prereleases with build metadata (`+sha`) are ignored. The current fixture mixes some of these but the assertions only check one path.
- **T2.** No case exercises the **workflow's** curl-fails → bootstrap path. The workflow's inline shell can only be tested by running the YAML, but the bootstrap-failure-handling branch could be extracted further and unit-tested. Currently a bootstrap-then-publish that produces a regressing channels.json would pass review and CI.
- **T3.** No regression test asserts that running `create_new_version.sh --mutation-mode github` after a stale channels.json triggers a guard. Without B4 being resolved, there is no test that detects the dual-path race.
- **T4.** No test confirms the new GitHub-derived `installer_url()` actually works against a recorded fixture (e.g., a saved 302 redirect response). The dry-run JSON snapshot encodes the URL string but does not exercise the runner's `--proto-redir =https` against GitHub's `objects.githubusercontent.com` redirect. A mocked-curl test would close this loop.

### Documentation drift

- **D4.** `internal_docs/distribution_pipeline.md:295` mentions the workflow concurrency group but does not name the workflow file or say which operator action is forbidden during a workflow run. Add: "While `preview-release.yml` is the authoritative GitHub-publish path, do not run `create_new_version.sh --mutation-mode github` from a workstation."
- **D5.** `docs/self_update.md` does not mention what happens to OLD binaries (pre-PR) after `metadata/channels.json` is removed from the site. End users with caches will see a different failure than the diagnostic anticipates. Add a "Migrating from earlier previews" note.
- **D6.** The bootstrap helper's first-time-setup section (`distribution_pipeline.md:80-97`) does not document the regression-protection check (see M2). If M2 is fixed, document it. If not, document explicitly that operators must hand-check the proposed `channels.json` before uploading because the helper does not protect against accidental downgrade.

### Not a blocker / confirmed good

- The metadata trust boundary is intact: `installer_url()` is derived from a compile-time constant base (`self_update_metadata.rs:8-9`) plus the resolved version. No environment override path was introduced. `ChannelMetadata::parse` still rejects `stable`/`rc` and any unknown channel (`self_update_metadata.rs:189-204`).
- `--proto-redir =https` in the runner (`self_update_runner.rs:79-82`) correctly handles `releases/download` → `objects.githubusercontent.com` HTTPS-to-HTTPS redirects; user docs at `docs/self_update.md:20-21` call this out.
- The publication ordering inside the workflow is correct: VERSION release published and asset-verified *before* `channels.json` is uploaded; the channels-release upload is the very last step.
- The new `channel_metadata_missing_installer_rejected.sh` closes the gap the first review's T3 raised — `validate_self_update_metadata.sh` now catches the "channels.json points at a version without a corresponding installer" case.
- The byte-equality `cmp` between `${WORK_DIR}/sifr-installer-${VERSION}` and `${INSTALL_ROOT}/versions/${VERSION}` in `create_new_version_real_run_plan_reuse.sh:51` prevents a future regeneration-vs-copy divergence.
- The `generate_channel_metadata.sh:51` regex no longer accepts `rc`, aligning input gating with `parse_channel`/`PreviewVersion::parse`.

### Bottom line

The previous three blockers (B1, B2, B3) and H1 are resolved. **B4 (dual-path race between the script's `--mutation-mode github` and the workflow) should block merge** — the concurrency group only protects half the surface area. **H4 (silent bootstrap fallback), H5 (OLD-binary migration), and H6 (one-sided post-publish verification) are operationally hazardous** and worth fixing in this PR. M1–M4, the T-series test gaps, and D4–D6 doc drift can land as small follow-up patches but B4 + H4–H6 are the integrity-of-channels.json issues that warrant action here.

Note: the empty `plans/reviews/active/github-self-update-channels-opus-review-2.md` file in the working tree was not touched per your "do not modify files" instruction.
