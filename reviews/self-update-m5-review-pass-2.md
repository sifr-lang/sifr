Second narrow review pass for M5 (`ad-hoc-self-update-m5`) covering the three doc clarifications applied after Pass 1.

## Verdict: READY

All three polish notes from Pass 1 have been resolved correctly and the new wording matches the code paths it describes. No new findings.

## Pass 1 polish notes — resolution status

1. **`self version` field enumeration (Pass 1 note 1) — resolved.**
   - `docs/self_update.md:24-27` now reads "reports the current executable, receipt version, install directory, receipt binary path, target, channel, receipt match status, and the current executable version."
   - Cross-checked against `render_version_json` (`crates/sifr/src/self_update_cli.rs:276-288`). The JSON emits `current_executable`, `current_version`, `receipt_version`, `install_dir`, `binary_path`, `channel`, `target`, `matches_receipt` (plus structural `schema_version` and `warnings`). The doc enumerates all eight user-facing fields, including the previously missing `current_version` and `binary_path`. ✓
   - Minor: text mode also emits `receipt_path` (`self_update_cli.rs:265`), which the doc does not enumerate. Non-blocking — Pass 1 only flagged the missing JSON fields, and `receipt_path` is a text-only convenience that does not belong to the canonical JSON snapshot.

2. **Stable vs RC gating distinction (Pass 1 note 2) — resolved.**
   - `docs/self_update.md:38-41` now reads "`stable` channels and stable-looking version pins remain gated until Phase 39 stable-channel promotion. Release-candidate channels and `-rc.N` pins are also rejected before Phase 39."
   - Cross-checked against the diagnostics:
     - `parse_channel` (`self_update_metadata.rs:235-237`): stable → "disabled until Phase 39". Doc wording "gated until Phase 39 stable-channel promotion" matches the forward-looking framing.
     - `parse_channel` (`self_update_metadata.rs:238-240`): rc → "disabled before Phase 39". Doc wording "rejected before Phase 39" matches.
     - `PreviewVersion::parse` (`self_update_metadata.rs:38-47, 66-70`): `-rc.` and stable-looking pins emit the same families. Doc wording covers both pin and channel forms. ✓
   - The two gates are now visibly separate sentences with the temporal asymmetry preserved (stable is gated *until*, rc is rejected *before*).

3. **Demo `<tmp>` placeholder convention (Pass 1 note 3) — resolved.**
   - `demos/self_update_demo/README.md:10-11` now reads "Expected output uses `<tmp>` as a placeholder for the absolute directory created by `mktemp -d`."
   - The note is positioned in the transcript preamble before the first fixture step, so a reader hits it before encountering `<tmp>` in any expected output block (`README.md:101, 128, 158`). ✓

## Spot checks beyond the diff

- No regression in the Pass 1 contract checkpoints. `docs/self_update.md` still scopes `self update` to standalone receipt-managed installs (`docs/self_update.md:3-15`), keeps the dry-run-only JSON contract (`docs/self_update.md:34`), and keeps the force semantics aligned with `resolve_update_plan` (`docs/self_update.md:43-51`).
- `internal_docs/distribution_pipeline.md` was not touched in this pass; its existing Phase 39 framing remains consistent with the refined public-doc wording.
- The demo transcript's mechanics (synthetic receipt, fake curl, padding-and-shebang installer, lock-presence self-check) are unchanged and still match the runner behavior validated in Pass 1.

## Deferred / out of scope

The Pass 1 non-blocking items 4 (`SIFR_INSTALL_MANIFEST_DIR` left undocumented as an internal/test affordance) and 5 (coarse `SELF_UPDATE_UNMANAGED_RECEIPT` diagnostic family deferred per the phase contract) are unchanged in this pass and remain non-blocking per the original review.

M5 remains **READY**.
