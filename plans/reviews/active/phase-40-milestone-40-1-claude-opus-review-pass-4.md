## Review — Phase 40 / milestone_40_1, pass 4

Branch `codex/phase-40-milestone-40-1` (6 commits) vs `origin/main`; 41 files, +4525/−137.

### Pass-3 findings: re-tested

| # | Pass-3 finding | Status |
|---|---|---|
| 1 | Non-UTF-8 evidence → raw traceback | **Partially fixed** — `planner.py:434-440` adds a governed `read_evidence_text`, and `binary-installer`/`binary-checksums` are now permanent cases (`qualification_selftest.py:597-628`). Probes: non-UTF-8 installer → `exit 2 … evidence is not readable UTF-8`; non-UTF-8 checksums → same. Three sibling sites remain — see P1 #1 |
| 2 | Workflow-contract check validated the wrong file | **Fixed** — `release-qualification.yml:34,45-48` fails the run unless `github.sha == source_commit`, so the executing definition *is* the checked-out one; `collect_qualification_artifacts.py:96` independently requires `run_metadata.head_sha == source_commit`; the contract case pins the `WORKFLOW_COMMIT` fragment (`release_qualification_workflow_contract.sh`, python block). Collector probes: `head_sha` forged, `head_sha` missing, `event=push`, workflow-name drift, fork repo, run-attempt drift → all 6 rejected |
| 3 | Editor/documentation reports had no exact shape or epoch contract | **Fixed** — `planner.py:380-431` adds `validate_editor_report`/`validate_documentation_report` with exact key sets, `schema_version == 2`, exact `kind`, and identity/status. 17 coherently-resealed probes (schema_version 1, schema_version removed, kind forged, extra field, status fail, source/submodule commit forged, package version/path/compat/vsix drift, doc equivalents) → **all 17 rejected**, none accepted |
| 4 | Mismatched resolved ref vs HEAD untested | **Fixed** — `mismatched-ref` case (`qualification_selftest.py:542-557`). Independently reproduced: advancing fixture HEAD after bundle construction → `exit 2 $.source_commit: must equal the resolved checkout HEAD`, no traceback |
| 5 | `source`/`submodule`/`lock` variants confounded | **Fixed** — `qualification_fixture.py:741-757` pins `GIT_AUTHOR_DATE`/`GIT_COMMITTER_DATE` on every commit, and `qualification_selftest.py:737-766` asserts a freshly created baseline fixture reproduces the *same commit id* and the *same plan digest* before the six evidence-only variants run. Verified in-suite |

Also verified: the no-op `nochange` control is real (`:785-786`), the in-repo `--out` guard runs against a valid bundle (`exit 2`, nothing written), and repeat writes to the same output path are refused.

---

### P1 — Blocking

**1. The non-UTF-8 remediation stopped at the planner; three sibling read sites still raise raw `UnicodeDecodeError`.**

`read_evidence_text` was added for the installer and checksums only. These remain unguarded and are all on live qualification paths:

- `verification/areas/distribution_release/governance/release_report.py:349-353` — `canonical_profile_digest` catches `(OSError, json.JSONDecodeError)`; `UnicodeDecodeError` is neither. It is called at `planner.py:67` on `<source_root>/verification/profiles/release.json` **before** any digest comparison, so a release checkout whose release profile is not valid UTF-8 crashes the planner. Reproduced with a coherent plan spec bound to that commit (clean checkout, HEAD == plan `source_commit`):

  ```
  non-utf8 release profile in checkout -> exit 1 | traceback: True
  UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff in position 0
    at release_report.py:350 in canonical_profile_digest
  ```

- `verification/areas/distribution_release/governance/release_report.py:163-169` — `validate_profile_matches_source` has the identical `except (OSError, json.JSONDecodeError)`.
- `scripts/distribution/qualify_stable_target.py:92` (checksum sidecar) and `:114` (`sysroot.toml` extracted from the candidate archive). `main` catches `(GovernanceError, OSError, subprocess.SubprocessError)` at `:58`, so neither is governed. Reproduced:

  ```
  non-utf8 checksum sidecar -> exit 1 traceback True
  ```

`release_governance.py:164` catches only `GovernanceError`, so all four surface as exit 1 with a Python stack trace instead of exit 2 with a governed diagnostic — the exact class pass-1 #3 and pass-3 #1 were opened to remove. Route all four through `read_evidence_text` (or add `UnicodeDecodeError`/`UnicodeError` to the existing `except` tuples), and add a `binary-release-profile` negative to `test_planner_rejects_drift_cases` so the site is permanently covered the way installer/checksums now are.

**2. `validate_installer_identity` is a line-prefix heuristic, not an identity contract, and is trivially defeated.**

`verification/areas/distribution_release/governance/planner.py:337-353` matches only lines that literally start with `APP_VERSION="` / `APP_CHANNEL="` and end with `"`. Any other shell assignment form to the same names is invisible, while the *last* assignment is what the executed installer actually uses. Verified against coherently resealed bundles (index rows and `plan.installer_sha256`/`desired_release.installer_sha256` all recomputed):

```
installer: trailing `APP_CHANNEL=beta` (unquoted)    ACCEPTED exit=0
installer: trailing `export APP_VERSION="9.9.9"`     ACCEPTED exit=0
installer: trailing tab-indented APP_VERSION=9.9.9   ACCEPTED exit=0
installer: indented `  APP_CHANNEL="beta"`           ACCEPTED exit=0
```

In each case the planner emits a canonical plan whose `installer_sha256` is presented as evidence that "the installer embeds the candidate stable version and channel" (`:349-353`), while the installer that will actually run resolves to `beta` or `9.9.9`. This is the sole binding between the published installer digest and the candidate identity — it exists precisely because pass-1 #6 found `installer_version` was an echo rather than evidence, and it does not survive the adversary it was written for. (The governed generator itself emits exactly one unindented assignment each — `generate_version_installer.sh:134,139` — so this is not reachable through the honest producer; it is reachable through any substituted installer, which is what the custody chain must catch.)

Fix: require exactly one line matching `^APP_VERSION=` and one matching `^APP_CHANNEL=` **in any form** (including `export`/leading whitespace), reject when more than one assignment to either name appears anywhere, and require those single assignments to be `APP_VERSION="<version>"` / `APP_CHANNEL="stable"`. Add the four probes above as negatives.

---

### P2 — Should fix before merge

**3. The collector does not enforce the custody boundary the planner does — a symlinked upload container is accepted by the producer.**

`scripts/distribution/collect_qualification_artifacts.py:199-218` builds `directory = artifact_root / workflow_name` (`:169`) and checks `directory.is_dir()`, then rejects symlinks and nested directories among the *entries*. It never checks whether `directory` itself is a symlink, and never resolves the per-file paths against `artifact_root`. Verified:

```
container replaced by symlink (-> outside artifact root)   ACCEPTED
```

The collector then hashes those out-of-tree files and writes them into `qualification-artifact-index.json` as the canonical custody record for the run. This is the mirror image of pass-2 finding 2, which was fixed on the consumer side at `planner.py:232-241` (`resolved_path.is_relative_to(resolved_root)` plus a non-symlink container) but not on the producer side. It is not exploitable end-to-end today — the planner re-hashes its own local tree and would fail — but the index is the artifact a later reviewer reads at publication, and it would attest digests for bytes the workflow never transported. Apply the same `resolve()` + `is_relative_to` + non-symlink-container check in `collect_container_rows`, and add the case to `test_artifact_collector_rejects_drift`.

---

### P3 — Tracking

**4. The pass-4 review record and the milestone checklist are not updated.**
`plans/reviews/active/phase-40-milestone-40-1-claude-opus-review-pass-4.md` is a zero-byte file, and `plans/issues/active/phase-40-stable-channel-ga-execution.md:82-88` still leaves "Qualify all compiler, sysroot, installer, documentation, Rust-claim, site, and VSIX artifacts" and "Record review rounds, PR, validation, and merge" unchecked with no PR link. The evidence list at `:234-246` also predates this round's negatives — `symlink-container`, `binary-installer`, `binary-checksums`, `bad-editor-shape`, `bad-doc-shape`, and `mismatched-ref` are now permanent cases and should be named. AGENTS.md "Required workflow" and the phase Validation Contract both require this per milestone.

---

### Broad adversarial sweep — no further defects found

Beyond the findings above, every probe below was correctly rejected with exit 2 and no traceback.

| Dimension | Probes | Result |
|---|---|---|
| Editor / documentation evidence shape and identity | 17 coherently-resealed mutations | 17 rejected |
| Aggregate checksums binding | dropped row, extra row, duplicate name, malformed row, wrong digest, non-UTF-8 | 6 rejected |
| Installer identity (canonical forms) | channel=beta, version drift, missing `APP_CHANNEL`, duplicate `APP_VERSION`, non-UTF-8, `${EVIL:-beta}` | 6 rejected |
| Filesystem custody (planner) | symlinked file→outside, symlinked container→outside, file replaced by directory | 3 rejected; legitimately symlinked `--artifact-root` correctly **accepted** (no false positive) |
| Artifact-index custody | id↔target swap, report→assemble container, sysroot row dropped, duplicate installer row, submodules dropped/forged, forked repository, retention 60, `overwrite: true`, `schema_version: 1`, beta candidate version, `../` traversal, run-attempt drift | 12 rejected |
| Expiry | expired index, one artifact expiring before the workflow boundary | 2 rejected |
| Plan semantics | `plan_id` prefix drift, version drift, archive/sysroot digest drift, builder drift, `receipt_channel: beta`, `installer_version` drift, `cargo_lock` drift, rustc drift, release-notes digest, site facts-schema digest, submodules forged, profile-manifest digest, report-id drift, index-id drift | 14 rejected |
| GA vs normal transition | `normal` on a preview index, predecessor version drift, rollback-target mismatch, predecessor dropped, predecessor added to `ga-activation` | 5 rejected |
| Rust stable-candidate result (coherently resealed against the release report) | suite dropped, `bless: true`, non-blocking, adversarial self-test case dropped, variant fail, empty variants, extra suite, non-zero summary, manifest forged, exit-code mismatch | 10 rejected; unmutated reseal correctly accepted |
| Collector API metadata & transport | 24 cases (run identity ×6, retention/timestamps ×4, container set, extra file, inner symlink, nested dir, empty file, 0/2 VSIX, submodules ×2, non-UTF-8 metadata, duplicate API name, missing container) | 23 rejected, 1 accepted → finding 3 |
| Source identity | mismatched ref vs HEAD, dirty checkout, floating ref | 3 rejected |
| Output custody | valid bundle + in-repo `--out` (exit 2, not written), repeat write to an existing path | 2 rejected |
| Schema/validator parity | collector output and `schema_contracts.qualification_index()` both conform to the checked-in schema; 9 differential mutations | no unsafe divergence (schema is a strict superset; validator is stricter in every case) |
| Workflow/ref identity | `github.sha == source_commit` gate + collector `head_sha` gate + contract-case pin | bound in all three places |
| Artifact contract exactness | `artifact_contract` pins kind/target/upload-suffix/file-name per governed id; 20-row exact id set; `minItems/maxItems: 20` in schema | consistent |

### Validation performed (read-only; no repository files modified)

| Check | Result |
|---|---|
| `areas run --area distribution_release --suite qualification --suite full --suite evidence-custody` | pass — 45 variants, 0 failures; 8 qualification self-tests (25.3 s) |
| `cases/release_qualification_workflow_contract.sh` | PASS |
| `cases/artifact_stable_candidate_generation.sh` | PASS |
| `cargo test -p sifr --bin sifr self_update_receipt::` | 16 passed, 0 failed |
| `scripts/check_file_size_guardrails.py` | PASS (2857 files, limit 900; largest touched 896) |
| `scripts/check_hir_maintainability_guardrails.py` | PASS |
| Profile registration | `distribution_release:qualification` in merge/nightly/release; `rust_interop:stable-candidate` in create-pr/merge/nightly/release; `release_report.REQUIRED_SUITES` and `verification/profiles/release.json` agree |
| Capability naming | `build_release_artifacts.sh`, `demos/stable_candidate_qualification_demo.sh` (executable, `bash -n` clean), `artifact_stable_candidate_generation.sh`, `release_qualification_workflow_contract.sh` — no phase/milestone identifiers; all references updated (`create_new_version.sh`, `sysroot_release/runner.py`, `preview_distribution_demo/README.md`, `internal_docs/distribution_pipeline.md`) |

Not run: `scripts/run_all_tests.sh --profile create-pr`/`merge`, the GitHub-hosted matrix jobs, and `demos/stable_candidate_qualification_demo.sh` (it requires a clean checkout and the worktree carries the untracked pass-4 review file; its planner path is the one exercised end-to-end by `test_materialized_planner_contract`).

---

**CHANGES_REQUESTED**
