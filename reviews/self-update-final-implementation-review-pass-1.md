Done reviewing. Producing the verdict now.

---

# Self-Update Ad Hoc Phase — Final Implementation Review

## Methodology

- Read phase contract `issues/ad-hoc-sifr-self-update.md` and execution `issues/ad-hoc-sifr-self-update-execution.md`.
- Read all four CLI modules: `crates/sifr/src/self_update_cli.rs` (510), `self_update_receipt.rs` (652), `self_update_metadata.rs` (532), `self_update_runner.rs` (618).
- Read `cli_model_and_entrypoint.rs` wiring (854 lines — under 900-line guardrail).
- Read diagnostic registration in `crates/sifr_diagnostics/src/codes/registry.rs`.
- Read installer/dispatcher generators (`generate_version_installer.sh`, `generate_dispatchers.sh`), drift validator (`validate_self_update_metadata.sh`), and release flow (`create_new_version.sh`).
- Read verification fixtures (`channel_metadata_*.sh`, `artifact_self_update_receipt_contract.sh`), receipt schema, and `common.sh` helpers.
- Read public docs (`docs/self_update.md`, `docs/cli_command_semantics.md`, `README.md`), internal docs (`internal_docs/distribution_pipeline.md` self-update sections), and `demos/self_update_demo/README.md`.

The user-provided paths under `crates/sifr/src/commands/self_update/**` do not exist; actual modules live at `crates/sifr/src/self_update_*.rs`. Review covers the actual layout.

## Blocking findings

### B1. Phase tracking is not closed despite all PRs being merged

The closeout branch is `codex/ad-hoc-self-update-closeout`, M5 PR #2278 has merged to `main` as `7c87ec190` ("Document self-update release readiness"), and the user lists all five milestone PRs as merged. But neither tracking file reflects that:

- `issues/ad-hoc-sifr-self-update.md:13` — M5 still rendered as `[ ]` with "ready in PR #2278" wording.
- `issues/ad-hoc-sifr-self-update.md:5` — `Status: in progress`.
- `issues/ad-hoc-sifr-self-update-execution.md:5` — `Status: in progress`.
- `issues/ad-hoc-sifr-self-update-execution.md:13` — M5 listed as "in progress".
- `issues/ad-hoc-sifr-self-update-execution.md:42-47` — "Merged PRs" lists M1–M4 only; PR #2278 not recorded.

The contract DoD for M5 explicitly says "The phase execution issue records merged PR links and review artifacts." The user named "phase tracking completeness" in the review requirements. With M5 merged on `main`, both files need:

- M5 checkbox flipped to `[x]`.
- Both `Status:` lines flipped to `complete`.
- A `M5: [PR #2278]` row added to the Merged PRs section.

This is the literal phase-closure gate and is not yet satisfied.

## Non-blocking observations

These would harden the implementation but do not block phase closure.

### N1. Single diagnostic code reused for every self-update failure

`crates/sifr_diagnostics/src/codes/registry.rs:244` registers exactly one code, `SELF_UPDATE_UNMANAGED_RECEIPT` (SIFR-BUILD-0901). The contract reserved the `SIFR-BUILD-09xx` range and explicitly deferred a dedicated CLI family, so allocating a single code is contract-compliant — but the *name* is now stretched across cases that have nothing to do with an unmanaged receipt:

- argument errors (`self_update_cli.rs:154,160,168`) — e.g. "--channel cannot be combined with --version"
- metadata fetch/parse failures (`self_update_metadata.rs:159-227`)
- installer download/validation failures (`self_update_runner.rs:166-202`) — "too small", "does not start with a shebang"
- installer execution failures (`self_update_runner.rs:57-65`) — "self-update installer exited with status N"

These all surface as `SIFR-BUILD-0901 SELF_UPDATE_UNMANAGED_RECEIPT`, which will confuse users following `--explain` links. A pre-stable follow-up should split this into a small family (or at minimum rename to something neutral like `SELF_UPDATE_FAILURE`) before Phase 39.

### N2. Metadata-fetch curl lacks the `--proto`/`--proto-redir` hardening used for installer download

- Installer download (`self_update_runner.rs:78-82`): `curl -fsSL --proto =https --proto-redir =https …`
- Metadata fetch (`self_update_metadata.rs:320-321`): `curl -fsSL …` only

The metadata URL is a compile-time `https://` constant, so the initial request is HTTPS, but a redirect to `http://` is permitted by default `-L`. Practical impact is low — metadata only carries version strings, and the immutable installer download still enforces both protocol guards before any code runs — but aligning the two call sites is cheap and removes one source of asymmetry.

### N3. Install lock has no stale-lock recovery

`self_update_runner.rs:242-256` (and the matching installer-template logic at `generate_version_installer.sh:341-353`) loops on `mkdir` forever with no PID file, no timeout, and no liveness probe. If a previous self-update or installer process is SIGKILLed mid-run, both code paths spin indefinitely.

Concurrent serialization is contract-compliant — and the verification fixtures confirm both the runner-internal serialization and the `SIFR_INSTALL_LOCK_HELD=1` handoff invariant — but stale-lock recovery is a real-world quality-of-life gap to land before stable promotion.

### N4. Minor style inconsistency in diagnostic rendering

`self_update_cli.rs:305-312` does `let diagnostic = *diagnostic;` to deref the box, while `render_user_error` and `render_user_error_with_exit` use `diagnostic.as_ref()` / `std::slice::from_ref`. Pick one style.

### N5. Receipt schema vs. parser scope on `rc`

`verification/distribution/self_update_install_receipt.schema.json:24-27` permits `channel ∈ {alpha, beta, rc}`, while the Rust parser rejects `rc` receipts at `self_update_receipt.rs:127-132`. This works (Rust is the authoritative gate, schema is forward-looking) but is worth a short comment in the schema explaining the schema is intentionally a superset for Phase 39 readiness.

## Contract checks — confirmed compliant

- **Receipt eligibility**: canonicalized `current_exe` + receipt `binary_path` device/inode match on Unix (`self_update_receipt.rs:189-207`); fallback to canonical-path equality on non-Unix; same-file metadata test for both symlink and hardlink under tests. Receipt parent-dir == install_dir invariant enforced. Target allowlist + channel allowlist enforced before network access.
- **Receipt discovery order**: `SIFR_INSTALL_MANIFEST_DIR` → `<current_exe_parent>/install.json` → `~/.sifr/install.json` only when same-file proves default home layout. Fails closed (`self_update_receipt.rs:82-114`).
- **Schema-versioned receipt**: `schema_version: 1` enforced; unknown fields rejected by both BTreeSet diff (Rust) and `additionalProperties:false` (schema). `parse_install_receipt_json` rejects empty/invalid/unknown-field/wrong-type input.
- **Stable + RC gating**: rejected in `parse_channel`, `PreviewVersion::parse`, `ChannelMetadata::parse`, and `validate_receipt_eligibility`. `is_stable_version` correctly rejects `X.Y.Z` without prerelease.
- **Dry-run JSON contract**: field order matches contract; `requested_channel` null vs string snapshotted; `would_run_installer` is false only for no_op; `action` enum covers `no_op/update/reinstall/downgrade/channel_switch`.
- **`self version` JSON contract**: field order matches; `schema_version: 1` independent of receipt schema_version; `matches_receipt` boolean preserved; `--short --format json` rejected before network access (`version_args_diagnostic`).
- **`--force` semantics**: same-version reinstall, downgrade, and channel switch all reject without `--force` and resolve to the correct `UpdateAction` with it; dry-run obeys the same rules before output.
- **Installer URL trust boundary**: `INSTALL_BASE_URL` is a compile-time constant; `PreviewVersion::installer_url` derives URL from constant + resolved version text only. Channel metadata has no URL fields, and `ChannelMetadata::parse` enforces an exact-key set of `{schema_version, channels}` with alpha+beta required and rejects rc/stable/unknown channel names.
- **Installer download safety**: `--proto =https --proto-redir =https`, atomic partial→final rename, `MIN_INSTALLER_BYTES = 1024` floor, shebang-required header check (`self_update_runner.rs:166-202`).
- **Install lock**: `<install_dir>/.sifr-update.lock` is acquired before installer exec, dropped after, and uses the same path as the manual installer template. `SIFR_INSTALL_LOCK_HELD=1` handoff is asserted by the generated installer (`generate_version_installer.sh:344-348`); concurrent runs serialize (runner test + receipt-contract fixture).
- **Receipt-derived environment**: `SIFR_INSTALL_DIR`, `SIFR_INSTALL_LOCK_HELD=1`, `SIFR_NO_MODIFY_PATH` (only when receipt says so), `SIFR_INSTALL_MANIFEST_DIR` (only when receipt path diverges from default), `--force` (only when requested). Manifest override defaults to `~/.sifr/install.json` for the default `~/.sifr/bin` install. Confirmed by `runs_passes_receipt_environment_force_and_manifest_override_to_installer` and `omits_manifest_override_for_default_home_manifest`.
- **Distribution drift guards**: `generate_dispatchers.sh:80-90` writes `metadata/channels.json` next to dispatchers from the same alpha+beta inputs; `validate_self_update_metadata.sh` extracts metadata, dispatcher `ALPHA_VERSION`/`BETA_VERSION`, and immutable installer `APP_VERSION`, and fails on any drift. Stable + rc channels rejected in metadata. `create_new_version.sh` real-run sequences `generate_version_installer.sh` → `generate_dispatchers.sh` from a single SHA-256-pinned plan.
- **File-size guardrail**: `cli_model_and_entrypoint.rs` is 854 lines; the self-update code lives in four sibling modules (510/532/618/652). The entrypoint receives only `SelfCommand(SelfArgs)` and `cmd_self` dispatch.
- **Generated installer receipt**: writes through `mktemp + mv` atomic rename, derives `APP_CHANNEL` from semver prerelease label, and records canonicalized `binary_path`; verified end-to-end by `artifact_self_update_receipt_contract.sh`.
- **Docs**: `docs/self_update.md`, `docs/cli_command_semantics.md:55-61`, `README.md:116-126`, `internal_docs/distribution_pipeline.md:156-196`, and `demos/self_update_demo/README.md` all match the implemented contract.

## Verdict

Code, tests, distribution guardrails, and docs land the contract cleanly with no implementation defects. The single blocking item is administrative: phase tracking is not yet updated to reflect M5 being merged, which is the literal DoD for phase closure and the explicit "phase tracking completeness" item in the review request.

VERDICT: CHANGES_REQUESTED
