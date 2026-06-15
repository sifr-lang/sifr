# Wave 4 self-update missing-receipt baseline — review

## Verdict

**No blockers.** Coverage claims, counts, source hash, and diagnostic output all check out, including an independent re-run of `sifr self version` under the pinned `SIFR_INSTALL_MANIFEST_DIR`. **No additional review round required** before the broad `scripts/run_all_tests.sh --profile create-pr` and merge-gate runs.

## Findings (ordered by severity)

### [verified] SIFR-BUILD-0901 rendered baseline coverage is correctly closed
- `crates/sifr/src/self_update_cli.rs:139-149` `cmd_version` calls `discover_production_receipt`, which calls `discover_install_receipt` (`crates/sifr/src/self_update_receipt.rs:82-92`). With `SIFR_INSTALL_MANIFEST_DIR` set to a directory whose `install.json` does not exist, the function returns `missing_receipt_diagnostic("standalone install receipt is missing at {} from SIFR_INSTALL_MANIFEST_DIR")`, the diagnostic code is `SELF_UPDATE_UNMANAGED_RECEIPT = SIFR-BUILD-0901`, and `render_user_error` returns `EXIT_USER_DIAGNOSTIC = 1`.
- I rebuilt and ran `SIFR_INSTALL_MANIFEST_DIR=<fixture>/missing-receipt cargo run -q --release -p sifr -- --diagnostic-format compact self version` directly and got byte-identical output (modulo the `<WORKSPACE>` normalization) to `self-version-compact.stderr.txt`, an empty stdout, and exit code `1`. One diagnostic, exactly the intended code.
- `code_baseline_coverage.json` now flips SIFR-BUILD-0901 from deferral → `baseline_fixture_id: "self_update_missing_receipt"` with `renderer_formats: ["compact"]`, matching what the fixture/manifest provide.

### [verified] Lower-level BUILD diagnostics are not over-claimed
- `code_baseline_coverage.json` still carries deferrals for `SIFR-BUILD-0002`, `0003`, `0004`, `0005`, `0006` — none flipped. The Wave 4 reason text is unchanged on those. Their registry sites (`crates/sifr_diagnostics/src/codes/registry/registry_entries/project_and_backend.rs:160-186`) target `sifr_driver::build::workspace` fault paths (Cargo manifest gen, rustc/cargo execution, missing artifact) and are not reachable from `sifr self version`.
- The plan's text accurately limits this slice's claim to `SIFR-BUILD-0901` only.

### [verified] Manifest / metadata / coverage / source-hash internal consistency
- `manifest.json:454-462` registers `self_update_missing_receipt` with `command=self-version`, `expect_exit_code=1`, `diagnostic_formats=["compact"]`. `code_baseline_coverage.json` agrees on the single renderer.
- `baseline_metadata.json` records `source_hash=sha256:812141fdbe57ddcfb0246fb0640923aa4c9a7435077667c3f4b75467a6632ed4`; `shasum -a 256` on `main.sifr` returns the same digest.
- Normalizers (`workspace-path`, `tmp-path`, `crlf`, `artifact-cache-lines`), owner, renderer, suite, bless_reason fields match the Wave 4 entry pattern.
- All three baseline files exist with correct shapes: empty stdout, exit `1`, single-line compact stderr.

### [verified] Coverage counts and remaining deferral families
- 170 active codes total (170 `active_entry!` macros, 0 reserved); 124 covered (`baseline_fixture_id` set, `deferral=null`); 46 deferred; 0 unclassified. Matches the plan's 124/46.
- Deferral by family from the JSON: `BUILD=5, INTERNAL=1, PACKAGE=34, STDLIB=2, WORKSPACE=4` — exactly matches the plan text.
- Baseline suite case count is 122 and variant count is 150 (sum of `len(diagnostic_formats)` per case), matching the `--bless` and replay numbers in the plan.

### [verified] Adapter change is minimal and scoped
- `area_adapter.py:24` adds `"self-version"` to `BASELINE_COMMANDS`; the only `self-version` usage in the manifest is the new case (verified by `grep -c "self-version" manifest.json` → 1), so no other cases pick up the new dispatch.
- `area_adapter.py:455-462` constructs `["self", "version"]` argv (no fixture path passed) and forces `SIFR_INSTALL_MANIFEST_DIR=<fixture>/missing-receipt`. Other branches still pass `env=None`, inheriting the parent env; behavior is unchanged for `check/run/build/test/lint/fmt-check`.
- Adapter is 610 lines, well under the 900-line cap.

### [info] One of multiple SIFR-BUILD-0901 message variants is covered
- The diagnostic is also raised for: receipt-present-but-unreadable, malformed JSON, wrong target/channel, mismatched binary path, schema-version drift, etc. (`self_update_receipt.rs:68/123/129/141/156/172/226/231/239/249/256/288`). The fixture exercises the missing-receipt-from-env path only.
- This is consistent with the project's "rendered baseline coverage" contract (the code is now covered) and matches how other multi-message codes have been treated in prior slices. Flagged here only so reviewers don't mistake this for full surface coverage of 0901 message variants.

### [info] `bless_reference` is a placeholder, inconsistent with the last five slices
- New metadata uses `"bless_reference": "wave-4-build-self-update-compact-baseline-pr"`. The five most recent Wave 4 slices (PRs #2580, #2582, #2583, #2584, #2586) inlined real `https://github.com/sifr-lang/sifr/pull/XXXX` URLs. Older slices (`wave-4-diagnostic-baseline-catalog-pr`, `wave-4-hir-recovery-baseline-pr`) kept placeholders post-merge.
- Recommend updating to the actual PR URL once the PR is opened, to match the recent convention. Optional; not a blocker.

### [info] Fixture-specific env in the adapter doesn't yet need generalization
- `area_adapter.py:459-462` burns the missing-receipt env path into the adapter rather than declaring it on the manifest case. With only one such case today this is pragmatic; if a second case needs env overrides, this should move to a manifest-level `env` field rather than a second `elif`.

### [info] Focused validation evidence is appropriate for the slice scope
- The plan's focused-validation list (direct compact CLI check, `--bless` then replay of `--suite baselines`, `--suite contracts`, `py_compile`, file-size guardrail, `git diff --check`) covers the surface that this slice actually touches (Python adapter + JSON data + new fixture). Broad cargo gates (`cargo test`, `cargo clippy`, `cargo fmt --check`, `check_hir_maintainability_guardrails.py`, full `run_all_tests.sh`) are correctly listed as still-pending in the plan's "Status" line — no overclaim.
- Sufficient to proceed to `--profile create-pr` and merge-gate validation.

## Another review round required?

**No.** After the broad gates pass, this slice can go to PR submission without a re-review. The only optional follow-up is rewriting `bless_reference` to the real PR URL when the PR is opened — that's a one-line edit and does not warrant a fresh review pass.
