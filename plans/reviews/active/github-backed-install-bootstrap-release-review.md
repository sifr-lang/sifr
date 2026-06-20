Based on the diff in `97b01e981`, validated test runs, and the published beta.7 outcome, here's my review.

## Blocking findings

None. The published beta.7 release matches the new contract end-to-end:
- `crates/sifr/src/self_update_metadata.rs:10-11` and the generated dispatchers both resolve channels from `releases/download/channels/channels.json`.
- `generate_dispatchers.sh:25` and `:198` derive the installer URL as `…/<version>/sifr-installer-<version>`, which matches the asset names the user confirmed on the GitHub release.
- The dispatcher's `--version` short-circuit (lines 175-181) skips metadata entirely, so a direct `--version 0.1.0-beta.7` pin needs only the release asset, which exists.
- `validate_self_update_metadata.sh` is invoked by `create_new_version.sh:341-343` after dispatcher regen, so any drift introduced by regen is caught in real-run.

## Non-blocking suggestions

1. **Dry-run drift check is asymmetric with the production contract.** `scripts/distribution/create_new_version.sh:174-176` only checks `CHANNEL_METADATA_URL` on `index` (not `alpha`/`beta`) and only forbids `versions/` (not `metadata/channels.json`). The exact regression that broke beta.6 — a website-hosted metadata URL — would not be caught on the `alpha` or `beta` dispatchers by dry-run. `validate_self_update_metadata.sh:110-118` validates all three; consider calling it (or replicating its asserts) from `validate_site_dispatchers` so dry-run mirrors the full contract.

2. **`metadata/` directory contract gap.** Per `internal_docs/distribution_pipeline.md:25` the site must not publish `public/install/metadata/`. `validate_self_update_metadata.sh:124` only blocks `metadata/channels.json`, not other files (`metadata/foo.json`) or an empty `metadata/` dir; `generate_dispatchers.sh:226` does `rm -rf "${INSTALL_ROOT}/metadata"` so regen self-heals, but the validator should match the doc.

3. **`read_current_channel_versions` silently prefers a local `channels.json`.** `create_new_version.sh:133-142` falls back to GitHub only if `${INSTALL_ROOT}/channels.json` is absent. In production the site repo should never contain `apps/sifr-site/public/install/channels.json`, but nothing in `validate_site_dispatchers` forbids it, and `generate_dispatchers.sh:226` cleans `versions/` and `metadata/` but not a stray top-level `channels.json`. A committed file would plan future releases off stale state with no warning. Either explicitly forbid `${INSTALL_ROOT}/channels.json` in validate_site_dispatchers or remove it during regen.

4. **`read -r` failure mode under `set -e`.** `create_new_version.sh:144` uses `read -r CURRENT_ALPHA CURRENT_BETA < <("${SCRIPT_DIR}/read_channel_versions.py" …)`. If the Python helper exits non-zero (malformed JSON, missing keys), `read` gets EOF, returns non-zero, and `set -e` exits before the `fail` on line 145 can produce a "create-new-version:" diagnostic. The Python stderr still surfaces, but the wrapper's contextualized error is bypassed. Capture into a temp first, or check the python exit status explicitly.

5. **Dry-run now requires network.** Previously `validate_site_dispatchers` was offline (it parsed dispatcher vars). Now `read_current_channel_versions` fetches GitHub when no local `channels.json` exists — `create_new_version.sh:139-141`. CI/maintainer workstations are online so it's not a release risk, but it's a behavior change worth documenting in `internal_docs/distribution_pipeline.md` if any tooling assumes offline dry-run.

6. **`channel_version_from_metadata` is JSON-format-coupled.** `generate_dispatchers.sh:119-123` uses sed regex against the literal `"<channel>": "<version>"` shape. It works for the current `generate_channel_metadata.sh` output (both pretty-printed and minified — verified by walking the greedy match), but any future channels.json schema change (e.g., nested objects, additional channels) breaks the dispatcher silently. Since the dispatcher is the long-lived deployed surface and the metadata generator can be redeployed any time, a brittle bash JSON parser here is a coupling worth flagging — or worth adding a contract test that locks the shape.

7. **Test name no longer matches assertion.** `channel_metadata_missing_installer_rejected.sh` now asserts "website must not publish metadata/channels.json" rather than anything about a missing installer. Consider renaming.

8. **Temp file leak on failure path.** `create_new_version.sh:137-149` doesn't trap-clean the mktemp temp file; if the read or post-read fail fires, the file is leaked. Cosmetic.
