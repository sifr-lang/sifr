I completed the review. All three pass-5 findings are closed and independently re-verified; one documentation finding remains.

## Review — Phase 40 / milestone_40_1, pass 6

Branch `codex/phase-40-milestone-40-1` (8 commits) vs `origin/main`; 44 files, +5198/−160. New since pass 5: `96019549f fix(release): regenerate qualification installer` (7 files, +401/−58).

### Pass-5 findings: re-tested

| # | Pass-5 finding | Status |
|---|---|---|
| 1 | Non-UTF-8 `sysroot.toml` in an archive → exit 1 with a raw traceback; the pass-4 `read_utf8` fix unreachable | **Fixed** |
| 2 | `validate_installer_identity` line-anchored heuristic; 5 shell assignment forms evade | **Fixed at the class level** — textual parsing removed entirely, replaced by governed-producer byte equality |
| 3 | Ordered-claims contract not exercised (single-claim fixture) | **Fixed** |

---

### (1) Non-UTF-8 `sysroot.toml` — governed on every live path

`scripts/distribution/verify_release_archive.py:151-156` now wraps the member decode:

```python
try:
    manifest_source = content.decode("utf-8")
except UnicodeDecodeError as exc:
    raise SystemExit(f"sysroot.toml must be readable UTF-8: {exc}") from exc
```

`grep -rn '\.decode('` over `scripts/distribution/**` and `verification/areas/distribution_release/governance/**` now returns exactly this one site, and it is guarded. Reproduced with a hand-built archive carrying `sysroot.toml = b"\xff\xfebad"` and a recomputed sidecar, against both live callers:

| Live path | Result |
|---|---|
| `verify_release_archive.py` standalone — assemble job (`release-qualification.yml:267`), preview job (`preview-release.yml:179`), `build_release_artifacts.sh:271`, and `generate_version_installer.sh:115` | exit 1, `sysroot.toml must be readable UTF-8: 'utf-8' codec can't decode byte 0xff in position 0: invalid start byte`, **no traceback** |
| `qualify_stable_target.py` — matrix job (`release-qualification.yml:118`) | exit 2, `stable-target-qualification: …: archive verification failed: sysroot.toml must be readable UTF-8: …`, **no traceback** |

Exit 1 on the standalone path is that script's uniform convention (`raise SystemExit(<message>)` at `:136,145,164,175,178,186`); it is a governed single-line diagnostic, and every consumer runs under `set -euo pipefail`. Permanent coverage added at `cases/artifact_stable_candidate_generation.sh:79-127`, which builds a real host archive, substitutes the bad manifest, recomputes the sidecar, and asserts both the diagnostic text and the absence of `Traceback`. That case **PASSES** on a real `aarch64-apple-darwin` build.

### (2) Installer identity — bound to the pinned governed producer

`planner.py:349-401` replaces `validate_installer_identity` with `validate_installer_bytes`: it copies the transported `binary-archive-<target>` and `checksum-<target>` artifacts for all four targets into a temp dir, runs `source_root/scripts/distribution/generate_version_installer.sh --version <plan version>` from the pinned checkout, and requires `regenerated.read_bytes() == transported.read_bytes()`. No shell is parsed.

All five pass-5 evasions re-tested against coherently resealed bundles (installer bytes rewritten; index row `sha256`/`size_bytes`, `plan.installer_sha256`, `desired_release.installer_sha256`, and the qualification-index plan reference all recomputed):

```
baseline-unmodified            exit=0 ACCEPTED
semicolon-prefixed-channel     exit=2 $.installer_sha256: transported installer bytes do not match the governed generator
readonly-channel               exit=2 (same)
eval-channel                   exit=2 (same)
and-chained-channel            exit=2 (same)
if-block-channel               exit=2 (same)
comment-only-append            exit=2 (same)   <- even a no-op comment is rejected
byte-identical-rewrite         exit=0 ACCEPTED  <- no false positive
```

The binding is genuinely closed, not another probe-fitted filter:

- **Output is a pure function of governed inputs.** I audited the generator heredoc (`generate_version_installer.sh:127-…`, unquoted `EOF`) for unescaped expansions: zero references outside `{VERSION, ARTIFACT_BASE_URL, INSTALLER_CHANNEL, case_entries}`. `ARTIFACT_BASE_URL` and `INSTALLER_CHANNEL` both derive from `VERSION`; `case_entries` derives from the four archive digests, which the generator itself re-verifies against the sidecars. No environment variable, timestamp, hostname, or path influences the bytes — so an attacker cannot make the planner regenerate *to* a tampered installer.
- **Production and planner invocations agree.** Both `release-qualification.yml:272` and `preview-release.yml:189` omit `--artifact-base-url`, matching the planner's call, so byte equality is achievable in production. Sidecars are bare-hash (`build_release_artifacts.sh:275`), which `tr -d '[:space:]'` at `:109` consumes correctly.
- **Tampering the generator is caught upstream.** Probed three ways; all rejected before regeneration runs:

```
generator-tampered-uncommitted  exit=2 $.source_commit: release checkout must be clean
generator-tampered-committed    exit=2 $.source_commit: must equal the resolved checkout HEAD
generator-symlinked             exit=2 $.source_commit: release checkout must be clean
```

  `validate_source_identity` (`planner.py:203-231`) runs at `:63`, well before `bind_aggregate_artifacts` at `:122`, and requires HEAD == plan `source_commit` == `--source-ref` with `git status --porcelain --untracked-files=all` empty. Production additionally binds `source_commit` to the qualification index and to `github.sha`, so the generator is transitively pinned. `validate_installer_bytes:355` also independently rejects a missing or symlinked generator.
- **No coverage regression from dropping the `installer` digest-sensitivity variant** (`qualification_selftest.py:848`). The variant was removed because the fixture now generates the installer rather than stamping a marker into it, so it could no longer vary independently. Installer-digest sensitivity is still exercised, transitively and for the right reason — verified directly:

```
nochange         installer_sha256=ab52b84adab50b6b
target-artifact  installer_sha256=e95e4838beddd280   (differs)
sysroot          installer_sha256=44f9fe0db272ee66
```

  The five `binary-installer` / `installer-*` negatives (`qualification_selftest.py:526-531`) remain and now fail closed on byte inequality.

### (3) Ordered Rust claim IDs — fixture has two claims plus a reversal negative

`qualification_fixture.py:725-737` now emits two baseline claims; `stable_claims(variant="rust-claims")` adds a third. Confirmed at runtime:

```
fixture baseline claims=2 ids=['direct_crate_fixture', 'bridge_fixture']
claim-count               exit=0 ACCEPTED
claim-order-reversed      exit=2 $.rust_interop.advertised_claim_ids: must exactly match the ordered stable support claims
claim-order-swap-only     exit=2 (same message; claims *file* reversed with its digest resealed)
```

The reversal is a permanent case (`rust-claim-order`, `qualification_selftest.py:534,568-571`), and it fails on the intended check with the intended message in both directions — plan-list reversal and claims-file reversal — so `planner.py:150` is now genuinely reachable.

### (4) No regression — evidence custody, workflow identity, capability naming, scope

| Dimension | Result |
|---|---|
| Evidence custody | `test_evidence_custody_mutations`, `test_artifact_index_mutations`, `test_release_report_mutations`, `test_surface_contract_mutations`, `test_strict_loader_rejects_duplicate_keys` all pass in the `full` suite (43 variants, 0 failures) |
| Exact workflow identity | `cases/release_qualification_workflow_contract.sh` **PASS** — `contents:read`/`actions:read` only; `WORKFLOW_COMMIT == SOURCE_COMMIT`; exact per-target download names ×1 each; `overwrite: false` ×4 and `retention-days: 30` ×4; no `environment`; no `contents: write`/`gh release`/`vsce publish`/`repository_dispatch`; `cargo build --locked` pinned |
| Capability naming | `grep -rniE 'phase[_ -]?40\|milestone[_ -]?40'` over `scripts/distribution/`, `verification/areas/distribution_release/`, the demo, the workflow, and the pipeline doc → **no matches** |
| Phase 40 scope | `self_update_receipt.rs:140` relaxes receipt *discovery* to accept `stable`, which milestone 40.1 needs for `sifr self version` on an isolated stable install. Stable *self-update* remains gated: `resolve_update_plan` calls `PreviewVersion::parse(current_version)` first, and `PreviewVersion::parse("0.1.0")` errors (pinned by `rejects_stable_and_rc_versions`); `parse_channel("stable")`, the `ga_status != "preview"` check, and the stable-metadata-key refusal (`rejects_stable_metadata`) are three further independent gates. Verified on a real binary with a stable receipt: `sifr self version` → exit 0; `sifr self update --channel stable` → `stable channel self-update is disabled while stable release channels are disabled`; no panic on any path. `stable_gate_inventory.json` records this exactly ("read-only receipt discovery accepts alpha, beta, and stable; stable update resolution remains gated"). `rc` acceptance in the generator is pre-existing, documented, and deferred to 40.2 |
| Peripheral diff | `build_preview_artifacts.sh` → `build_release_artifacts.sh` rename applied consistently across `preview-release.yml`, `create_new_version.sh`, `sysroot_release/runner.py`, and the preview demo README; `selftest.py:670` registers the new `qualification` suite |

### (5) Tracking accuracy

`plans/issues/active/phase-40-stable-channel-ga-execution.md:245-253` records pass 5 and its remediation precisely: the verifier decode now governed without a traceback, byte-for-byte regeneration replacing shell parsing, and the two-claim fixture plus order reversal. The evidence list (`:262-265`) adds "byte-divergent installer regeneration" and "Rust-claim ordering". Three checklist boxes are checked and three legitimately open (no PR yet; docs/VSIX materialize in 40.4).

The pass-4 entry's wording at `:236-238` — remediation "now governs every release-profile/checksum/**sysroot** text decode" — was inaccurate when written, but the pass-5 entry immediately below it states that pass 5 "found the archive verifier's earlier non-UTF-8 sysroot decode," so the round-by-round record self-corrects. Not actionable.

### Additional adversarial probes — no further defects

| Dimension | Probes | Result |
|---|---|---|
| Installer regeneration surface | generator tampered (uncommitted / committed), generator symlinked, comment-only byte drift, byte-identical rewrite, heredoc expansion audit for environment influence, base-URL agreement between planner and both workflows, sidecar format compatibility | all closed; no false positive |
| Non-UTF-8 sysroot | synthetic archive on both live paths | governed, no traceback |
| Claim ordering | plan-list reversal, claims-file reversal, baseline 2-claim accept | 2 rejected with the exact ordered-claims message, baseline accepted |
| Digest sensitivity | `target-artifact` / `sysroot` propagation into `installer_sha256` | sensitive |
| Stable receipt runtime | `self version`, `self update`, `self update --dry-run`, `self update --channel stable` on a real stable receipt | 1 correct success, 3 governed refusals, 0 panics |

### Validation performed (read-only; no repository files modified)

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite qualification` | pass — 8 self-tests, 1 variant, 37.6 s |
| `areas run --area distribution_release --suite full` | pass — 43 variants, 0 failures |
| `cases/release_qualification_workflow_contract.sh` | PASS |
| `cases/artifact_stable_candidate_generation.sh` (real host build) | PASS |
| **`demos/stable_candidate_qualification_demo.sh`** — run in a clean local clone of `96019549f` with recursive submodules, closing the pass-5 gap | **PASS** — real `aarch64-apple-darwin` archive, isolated install, `sifr --version` / `sifr check` / `sifr self version`, governed installer regenerated and byte-matched across the real host archive plus three fixture targets, canonical unapproved plan written outside the checkout |
| `cargo test --release -p sifr -- self_update` | pass — 42 tests |
| `scripts/check_file_size_guardrails.py` | PASS (2857 files, limit 900) |
| `scripts/check_hir_maintainability_guardrails.py` | PASS |

Not run: `scripts/run_all_tests.sh --profile create-pr`/`merge` and the GitHub-hosted matrix jobs (they require GitHub runners and the Actions artifact API).

---

## Remaining finding

**P3 — documentation. `internal_docs/distribution_pipeline.md` does not record the installer-regeneration binding, and its planner description now understates what the planner does.**

Files: `internal_docs/distribution_pipeline.md:179-186` (the `plan-stable-release` paragraph in the new **Stable Candidate Qualification** section).

Evidence: the paragraph enumerates the planner's requirements and concludes "It hashes and cross-checks those exact bytes before writing one canonical `stable-release-plan.json`." As of `96019549f` the planner does materially more: `planner.py:349-401` **executes** `scripts/distribution/generate_version_installer.sh` from the pinned checkout, which re-verifies all four candidate archives against their sidecars (`generate_version_installer.sh:98-124`), and then requires the transported installer to be byte-identical to that output. `grep -rn -i 'regenerat|byte-for-byte|byte equality|identity'` over the file returns nothing for this section. This is the sole binding between the published installer digest and candidate identity — it exists because pass 1 #6 found `installer_version` was a self-attested echo, and it replaced the textual check that passes 4 and 5 both defeated. It also has operational consequences a reader should know about (the planner shells out to `bash` and `python3`, decompresses four archives, and fails closed as `$.installer_sha256: governed installer regeneration failed: …` if either interpreter or the generator is unavailable). Per `AGENTS.md`, durable architecture belongs in `internal_docs/`, and this branch established that section as the place for it.

Fix: extend that paragraph with one or two sentences — the planner regenerates the immutable installer with the governed generator at the pinned `source_commit` from the transported per-target archives and checksums, and requires byte equality with the transported installer, so the installer digest is bound to the governed producer rather than to any textual self-attestation.

Non-blocking observations, offered without requesting changes:

- `cases/release_qualification_workflow_contract.sh:68` pins that the assemble job *invokes* `generate_version_installer.sh`, but not its argument set. The planner's regeneration assumes the workflow passes no `--artifact-base-url`. If that ever drifts, the failure is fail-closed and clearly diagnosed (`transported installer bytes do not match the governed generator`), so this is brittleness rather than a hole; pinning the exact invocation would make the coupling explicit.
- Nothing pins `parse_channel("stable")` erroring. The stable self-update gate does not depend on it — `PreviewVersion::parse("0.1.0")` fails first and *is* pinned — so this is redundancy, not a gap.

Everything the review was asked to verify is confirmed closed, and the milestone is otherwise mergeable; the single remaining item is documentation-only, with no security or correctness impact.

**CHANGES_REQUESTED**
