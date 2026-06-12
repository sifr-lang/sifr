First review pass for M5 (`ad-hoc-self-update-m5`) covering docs and release readiness.

## Verdict: READY

Public docs, troubleshooting guidance, internal architecture description, and the demo transcript all match the implementation that landed in M1–M4. No blocking issues. A handful of non-blocking polish notes are listed below.

## Contract checkpoints verified

1. **Public docs scope `self update` to standalone receipt-managed installs only.**
   - `docs/self_update.md:1-15` opens with "`sifr self update` updates official standalone preview installs created by the Sifr installer" and explicitly disclaims direct archive download/extraction/profile editing.
   - `docs/self_update.md:53-56` directs Cargo/Homebrew/system-package/source builds to update through their original tool and recommends `sifr --version` for unmanaged installs (matches the phase contract's M5 promise about package-manager install guidance).
   - `README.md:116-126` adds a "Update an official standalone preview install" block that says the same thing and links into `docs/self_update.md`.
   - `docs/cli_command_semantics.md:56-61` cross-references `self_update.md` and notes `self update` requires a schema-versioned receipt and stable is gated until Phase 39.

2. **`self version` is documented as also receipt-managed.**
   - `docs/self_update.md:19-27` covers `self version` and its `--short` interaction.
   - The same paragraph plus the troubleshooting section directs unmanaged installs to `sifr --version` (`sifr --version` is implemented via clap at `crates/sifr/src/cli_model_and_entrypoint.rs:29`, so the suggestion resolves).
   - Phase contract's M2 decision that `self version` requires the managed receipt is honored by `cmd_version` at `crates/sifr/src/self_update_cli.rs:139-150`, and the docs do not promise otherwise.

3. **Dry-run JSON-only contract is captured.**
   - `docs/self_update.md:32-33` says "`--format json` is available only with `--dry-run`" matching the rejection in `update_args_diagnostic` at `crates/sifr/src/self_update_cli.rs:158-162`.
   - The docs also explicitly state `--dry-run` does not acquire the install lock.

4. **Stable/RC gating before Phase 39.**
   - `docs/self_update.md:36-39` states `stable`, RC channels, `-rc.N` pins, and stable-looking version pins are gated until Phase 39, matching `parse_channel` (`self_update_metadata.rs:231-245`), `PreviewVersion::parse` (`self_update_metadata.rs:37-84`), and `ChannelMetadata::parse` (`self_update_metadata.rs:184-217`).
   - `internal_docs/distribution_pipeline.md:67` records that stable metadata is absent until Phase 39, consistent with the M4 `channel_metadata_stable_rejected.sh` drift guard.

5. **Force semantics.**
   - `docs/self_update.md:41-49` describes `--force` for same-version reinstall, downgrade, and channel switch, and explicitly notes ordinary newer-version updates within the receipt channel do not require it. Matches `resolve_update_plan` (`self_update_metadata.rs:281-306`) — downgrade-without-force returns the diagnostic the docs implicitly point to, channel-switch-without-force returns the analogous error, and same-version is a no-op unless `force == true`.
   - `internal_docs/distribution_pipeline.md:189-192` mirrors the same rules.

6. **Troubleshooting guidance is concrete and matches diagnostics.**
   - Missing/malformed/pre-schema receipt: `docs/self_update.md:53-64` tells the user to re-run `curl -fsSL https://sifr.sh/install | sh`, which is the exact remediation the unmanaged-receipt diagnostics suggest (`self_update_receipt.rs:226, 239, 249, 288`).
   - Package-manager installs: `docs/self_update.md:53-56` enumerates Cargo, Homebrew, system package manager, and source builds.
   - Custom install dir: `docs/self_update.md:66-70` recommends rerunning the installer with the same `SIFR_INSTALL_DIR` env, which produces a fresh adjacent `install.json` so discovery rule 2 (`<exe-parent>/install.json`, `self_update_receipt.rs:94-99`) picks it up.
   - Receipt mismatch: `docs/self_update.md:72-75` points to `command -v sifr` and direct-binary invocation; mirrors the eligibility check at `self_update_receipt.rs:133-145`.
   - Network/installer failures: `docs/self_update.md:77-80` notes the existing binary is not replaced until the delegated installer validates and installs. Consistent with the runner's atomic temp-download/validate/lock/exec sequence (`self_update_runner.rs:42-66`).

7. **Demo transcript exercises install fixture, dry-run, update, no-op, and forced downgrade against local fixtures.**
   - `demos/self_update_demo/README.md:11-40` constructs a copied `target/debug/sifr` binary, a synthetic schema-versioned receipt at a non-default manifest dir, and an empty fake-bin dir. The receipt fields (schema_version=1, channel=beta, target=aarch64-apple-darwin, paths in tmp, modify_path=false) satisfy the receipt parser and the eligibility checker (same-file dev/ino check passes because the receipt's `binary_path` *is* the copied executable).
   - `demos/self_update_demo/README.md:42-59` dry-runs `--version 0.1.0-beta.2`; the expected text matches `render_dry_run_text` (`self_update_cli.rs:212-229`).
   - `demos/self_update_demo/README.md:62-99` performs a real update via the fake `curl`. The fake-curl script ignores all flags except `-o`, copies the prepared installer into the destination, and the runner's atomic rename plus shebang+size validation (`self_update_runner.rs:166-202`) accept it because the prepared installer starts with `#!/bin/sh` and the 160-line `# padding` block plus the body comfortably exceeds the 1024-byte minimum.
   - `demos/self_update_demo/README.md:102-126` mutates the receipt to 0.1.0-beta.2 with a small python3 snippet and re-runs the same `--version 0.1.0-beta.2` command to trigger the NoOp branch (`self_update_cli.rs:119-127` prints "Sifr 0.1.0-beta.2 is already installed at …").
   - `demos/self_update_demo/README.md:129-156` shows the downgrade rejection message verbatim (matches `self_update_metadata.rs:300-303`) and then re-runs with `--force` to reach the downgrade path. The fake installer captures `args=--force` because the runner only adds `--force` when `plan.force == true` (`self_update_runner.rs:118-120`).
   - The fake installer's `test -d "$SIFR_INSTALL_DIR/.sifr-update.lock"` is a useful self-check that the runner did acquire the lock before invoking the installer.

8. **Internal docs describe metadata, receipt, lock, and immutable-installer delegation.**
   - `internal_docs/distribution_pipeline.md:55-67` documents `metadata/channels.json` as resolution-only with no URL fields and constant-derived installer URLs, matching `CHANNEL_METADATA_URL`/`INSTALL_BASE_URL` and `installer_url()` in `self_update_metadata.rs`.
   - `internal_docs/distribution_pipeline.md:156-176` documents the schema-versioned receipt, its authoritative schema file, the unmanaged-install fall-through, and the channel/binary_path/modify_path derivation rules. The field list is exactly the `RECEIPT_FIELDS` enumeration at `self_update_receipt.rs:209-219`.
   - `internal_docs/distribution_pipeline.md:178-196` covers the TLS/delegation contract, the dry-run-no-lock rule, the install lock at `<install_dir>/.sifr-update.lock`, and the `SIFR_INSTALL_LOCK_HELD=1` handoff — all matching `self_update_runner.rs:111-135` and the M3 generated-installer reciprocity.
   - `internal_docs/distribution_pipeline.md:144-154` documents the immutable installer's atomic receipt write, install lock, and the standalone shell-profile-edit suppression via `SIFR_NO_MODIFY_PATH=1` — consistent with M1.
   - `internal_docs/distribution_pipeline.md:219-227` lists the M4 drift checks (`channel_metadata_installer_agreement`, dispatcher/installer drift, stable rejection) and `scripts/distribution/validate_self_update_metadata.sh`.

## Non-blocking observations

These do not block M5; flag as cleanup candidates if the docs get another polish pass.

1. **`self version` field enumeration is slightly under-specified in the public doc.** `docs/self_update.md:24-26` describes the output as "current executable, receipt version, install directory, target, channel, and receipt match status", which omits `current_version` and `binary_path` from the JSON shape rendered by `render_version_json` (`self_update_cli.rs:276-288`). The deterministic JSON snapshot in the phase contract is the canonical source; the public doc could enumerate the full set or just point at `self_update.md` plus the receipt schema.

2. **Stable vs RC are conflated in the public doc.** `docs/self_update.md:36-39` says "`stable`, release-candidate channels, `-rc.N` pins, and stable-looking version pins are gated until Phase 39 stable-channel promotion." The contract distinguishes *stable* (gated until Phase 39 unblocks it) from *rc* (rejected *before* Phase 39 with a separate diagnostic family). Both are currently unavailable, so the user-facing impact is identical; consider rewording if Phase 39 lands and the two paths diverge.

3. **Demo placeholder substitution.** Expected outputs include `<tmp>` placeholders (e.g., `updated=<tmp>/install/bin args=`). A reader running the transcript end-to-end will see the absolute `/private/var/folders/…` path instead. Worth a one-line note in the transcript explaining the placeholder convention, but not a blocker — the surrounding prose already establishes that `tmp_dir` is `mktemp -d`.

4. **`SIFR_INSTALL_MANIFEST_DIR` is undocumented in public docs.** The variable is used in the demo and in tests, but `docs/self_update.md` doesn't surface it. That is probably intentional — it is an internal/test affordance, and the user-facing "rerun the installer with the same environment" guidance is enough to produce a discoverable receipt for default and `SIFR_INSTALL_DIR`-customized layouts. Noted only to confirm the omission is deliberate.

5. **Diagnostic codes still coarse.** The runner and metadata modules continue to reuse `DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT` for download/execution/lock/metadata failures. The phase contract explicitly defers a dedicated diagnostic family to a later planning change, so this is in-scope-deferred, not a defect.

## Status of remaining M5 work

The user noted `cargo fmt --check`, `python3 scripts/check_file_size_guardrails.py`, and the targeted docs grep all pass. Per the M5 scope the full local validation gate (`scripts/run_all_tests.sh`, `scripts/run_distribution_validation.sh`, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr -- self_update`) is still queued. None of the M5 changes touch compiled crates other than re-asserting an existing contract, so this is expected; the validation ledger in `issues/ad-hoc-sifr-self-update-execution.md` should still record the full run before the M5 PR opens.

M5 is **READY**.
